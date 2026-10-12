use crate::prelude::*;

use centurions_core::combat::place_squad;

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
#[test]
fn formation_matches_whole_warriors_and_tactical_map() {
    for (json, start) in [
        (
            include_str!("../../../../evidence/fixtures/formation.json"),
            (17, 13),
        ),
        (
            include_str!("../../../../evidence/fixtures/formation-opening.json"),
            (14, 14),
        ),
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
            let mut warriors: Vec<_> = crate::support::warriors(&decode(&fixture.input));
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
                crate::support::record_bytes(&warriors),
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
