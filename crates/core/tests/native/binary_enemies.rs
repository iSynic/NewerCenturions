use crate::prelude::*;

use centurions_content::combat::BattleRecord;

use centurions_core::combat::place_enemies;

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
#[test]
fn enemy_setup_matches_whole_records_positions_map_and_randomness() {
    for json in [
        include_str!("../../../../evidence/fixtures/enemies.json"),
        include_str!("../../../../evidence/fixtures/enemies-opening.json"),
    ] {
        let fixture: Fixture = serde_json::from_str(json).unwrap();
        let map_fixture: serde_json::Value = crate::support::fixture("combat-map.json");
        let properties = map_fixture.bytes("properties");
        let mission = OpeningMission {
            tile_properties: properties[64..12064].to_vec(),
            default_tile: 0,
            shared: std::sync::Arc::new(centurions_content::mission::StoryTables {
                ..crate::support::stories()
            }),
            ..crate::support::mission()
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
                crate::support::record_bytes(&result.records),
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
