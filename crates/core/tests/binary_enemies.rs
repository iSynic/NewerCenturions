use centurions_content::{combat::BattleRecord, mission::OpeningMission};
use centurions_core::{
    RandomState,
    combat::{TacticalMap, place_enemies},
};
use serde::Deserialize;
#[derive(Deserialize)]
struct Fixture {
    battle: String,
    definitions: String,
    map: String,
    cases: Vec<Case>,
}
#[derive(Deserialize)]
struct Case {
    base: i16,
    seed: u32,
    after_seed: u32,
    draws: u64,
    count: usize,
    records: String,
    tiles: String,
    positions: String,
}
fn decode(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
fn words(value: &str) -> Vec<i16> {
    decode(value)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|v| i16::from_be_bytes(*v))
        .collect()
}
#[test]
fn enemy_setup_matches_whole_records_positions_map_and_randomness() {
    for json in [
        include_str!("../../../evidence/fixtures/enemies.json"),
        include_str!("../../../evidence/fixtures/enemies-opening.json"),
    ] {
        let fixture: Fixture = serde_json::from_str(json).unwrap();
        let map_fixture: serde_json::Value =
            serde_json::from_str(include_str!("../../../evidence/fixtures/combat-map.json"))
                .unwrap();
        let properties = decode(map_fixture["properties"].as_str().unwrap());
        let mission = OpeningMission {
            name: "fixture".into(),
            start: (2, 2),
            header: vec![],
            regions: vec![],
            floor: vec![],
            objects: vec![],
            events: vec![],
            tile_properties: properties[64..12064].to_vec(),
            terrain: 2,
            default_tile: 0,
            minutes_per_tick: 5,
            shared: std::sync::Arc::new(centurions_content::mission::StoryTables {
                item_cache: vec![],
                areas: vec![],
                races: vec![],
                profiles: vec![],
                door_scripts: vec![],
                dialogue: vec![],
                treasure: vec![],
                prologue: String::new(),
                item_descriptions: vec![],
                schedule: vec![],
                scheduled_scripts: vec![],
                menus: vec![],
                typed_encounters: vec![],
                movies: Default::default(),
                combat: None,
            }),
        };
        let battle = BattleRecord::read(&decode(&fixture.battle), 0).unwrap();
        for case in fixture.cases {
            let mut map = TacticalMap {
                tiles: words(&fixture.map),
                shift: (-39, -15),
                edge_word: 0,
            };
            let mut random = RandomState::new(case.seed);
            let result = place_enemies(
                &mut map,
                &mission,
                &battle,
                &decode(&fixture.definitions),
                (case.base, case.base),
                &mut random,
            )
            .unwrap();
            assert_eq!(result.records.len(), case.count);
            assert_eq!(
                result
                    .records
                    .iter()
                    .flat_map(|m| m.bytes.iter().copied())
                    .collect::<Vec<_>>(),
                decode(&case.records)
            );
            assert_eq!(
                result
                    .positions
                    .iter()
                    .flat_map(|(x, y)| [*x, *y])
                    .collect::<Vec<_>>(),
                words(&case.positions)
            );
            assert_eq!(map.tiles, words(&case.tiles));
            assert_eq!((random.seed, random.draws), (case.after_seed, case.draws));
        }
    }
}
