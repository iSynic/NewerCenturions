use crate::{prelude::*, support};

use centurions_core::combat::footprint::find_placement;
use centurions_core::combat::footprint::restore;

#[test]
fn all_footprint_placement_and_restore_match_original() {
    let fixture: serde_json::Value = support::fixture("large-placement.json");
    let mission = support::terrain_mission(&fixture);
    for (index, case) in fixture.array("cases").iter().enumerate() {
        let map = TacticalMap {
            tiles: support::words(&case.bytes("map")),
            shift: (0, 0),
            edge_word: 0,
        };
        let monster = MonsterRecord {
            bytes: case.bytes("monster"),
        };
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let placed =
            find_placement(&map, &monster, &mission, 10, (0, 0), (40, 40), &mut random).unwrap();
        support::native_expect! {
            bytes placed.monster.bytes => &case["after"], "monster case {index}";
            bytes support::word_bytes(&placed
                        .map.tiles) => &case["tiles"], "map case {index}";
        }
        assert_eq!(
            placed.position,
            (case.n::<i16>("x"), case.n::<i16>("y")),
            "position case {index}"
        );
        support::assert_rng(&random, &case["seed_after"], &case["draws"]);
        let restored = restore(&placed.map, &placed.monster, placed.position).unwrap();
        support::native_expect! {
            words restored.tiles => &case["restored"], format!("restore case {index}");
        }
    }
}
