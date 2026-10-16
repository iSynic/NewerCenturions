use crate::prelude::*;

use centurions_core::combat::place_squad;

#[test]
fn formation_matches_whole_warriors_and_tactical_map() {
    for (name, start) in [
        ("formation.json", (17, 13)),
        ("formation-opening.json", (14, 14)),
    ] {
        let fixture = crate::support::fixture(name);
        let map_fixture: serde_json::Value = crate::support::fixture("combat-map.json");
        let mission = OpeningMission {
            default_tile: 0,
            ..crate::support::terrain_mission(&map_fixture)
        };
        let content = Content {
            items: fixture
                .bytes("items")
                .as_chunks::<150>()
                .0
                .iter()
                .map(|v| v.to_vec())
                .collect(),
            ..crate::support::fixture_content("fixture")
        };
        for case in fixture.array("cases") {
            let mut warriors = crate::support::warriors(&fixture["input"]);
            let mut map = TacticalMap {
                tiles: fixture.words("map"),
                shift: (-39, -15),
                edge_word: 0,
            };
            let actual = place_squad(
                &mut map,
                &mission,
                &content,
                &mut warriors,
                start,
                case.n("direction"),
                &[12, 14, 12, 14, 10, 12, 10, 10, 14, 16, 0, 12],
            )
            .unwrap();
            crate::support::native_expect! {
            bytes crate::support::record_bytes(&warriors) => &case["warriors"], format!("direction{}", case["direction"]);
            words map.tiles => &case["tiles"], "formation map";
            words actual.under_tiles => &case["under"], "formation under-tiles";
            }
            assert_eq!(
                actual.positions,
                crate::support::positions(&case.bytes("positions"))
            );
        }
    }
}
