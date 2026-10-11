use centurions_content::{Content, Warrior, mission::OpeningMission};
use centurions_core::combat::{TacticalMap, place_squad};
use serde::Deserialize;

#[derive(Deserialize)]
struct Fixture {
    input: String,
    items: String,
    map: String,
    cases: Vec<Case>,
}
#[derive(Deserialize)]
struct Case {
    direction: i16,
    warriors: String,
    tiles: String,
    positions: String,
    under: String,
}
fn decode(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
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
fn formation_matches_whole_warriors_and_tactical_map() {
    for (json, start) in [
        (
            include_str!("../../../evidence/fixtures/formation.json"),
            (17, 13),
        ),
        (
            include_str!("../../../evidence/fixtures/formation-opening.json"),
            (14, 14),
        ),
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
        let content = Content {
            identity: "fixture".into(),
            warriors: vec![],
            items: decode(&fixture.items)
                .as_chunks::<150>()
                .0
                .iter()
                .map(|v| v.to_vec())
                .collect(),
            resources: vec![],
            original_services: None,
            scenario: None,
        };
        for case in fixture.cases {
            let mut warriors: Vec<_> = decode(&fixture.input)
                .as_chunks::<3138>()
                .0
                .iter()
                .map(|v| Warrior::read(v).unwrap())
                .collect();
            let mut map = TacticalMap {
                tiles: words(&fixture.map),
                shift: (-39, -15),
                edge_word: 0,
            };
            let actual = place_squad(
                &mut map,
                &mission,
                &content,
                &mut warriors,
                start,
                case.direction,
                &[12, 14, 12, 14, 10, 12, 10, 10, 14, 16, 0, 12],
            )
            .unwrap();
            assert_eq!(
                warriors
                    .iter()
                    .flat_map(|w| w.bytes.iter().copied())
                    .collect::<Vec<_>>(),
                decode(&case.warriors),
                "direction {}",
                case.direction
            );
            assert_eq!(map.tiles, words(&case.tiles));
            assert_eq!(
                actual
                    .positions
                    .iter()
                    .flat_map(|(x, y)| [*x, *y])
                    .collect::<Vec<_>>(),
                words(&case.positions)
            );
            assert_eq!(actual.under_tiles, words(&case.under));
        }
    }
}
