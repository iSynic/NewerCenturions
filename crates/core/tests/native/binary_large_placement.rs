use crate::prelude::*;

use centurions_core::combat::footprint::find_placement;
use centurions_core::combat::footprint::restore;

#[test]
fn all_footprint_placement_and_restore_match_original() {
    let fixture: serde_json::Value = crate::support::fixture("large-placement.json");
    let properties = fixture.bytes("properties");
    let mission = OpeningMission {
        tile_properties: properties[64..12064].to_vec(),
        shared: std::sync::Arc::new(centurions_content::mission::StoryTables {
            ..crate::support::stories()
        }),
        ..crate::support::mission()
    };
    for (index, case) in fixture.array("cases").iter().enumerate() {
        let map = TacticalMap {
            tiles: crate::support::words(&case.bytes("map")),
            shift: (0, 0),
            edge_word: 0,
        };
        let monster = MonsterRecord {
            bytes: case.bytes("monster"),
        };
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let placed =
            find_placement(&map, &monster, &mission, 10, (0, 0), (40, 40), &mut random).unwrap();
        crate::support::native_expect! {
        bytes placed.monster.bytes => &case["after"], "monster case {index}";
        bytes crate::support::word_bytes(&placed
                        .map.tiles) => &case["tiles"], "map case {index}";
        }
        assert_eq!(
            placed.position,
            (case.n::<i16>("x"), case.n::<i16>("y")),
            "position case {index}"
        );
        crate::support::native_expect! {
        scalar random.seed => &case["seed_after"], "binary_large_placement";
        scalar random.draws => &case["draws"], "binary_large_placement";
        }
        let restored = restore(&placed.map, &placed.monster, placed.position).unwrap();
        assert_eq!(
            crate::support::word_bytes(&restored.tiles),
            case.bytes("restored"),
            "restore case {index}"
        );
    }
}
