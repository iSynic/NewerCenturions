use crate::prelude::*;

use centurions_core::combat::allies::place_allies;
use centurions_core::combat::allies::synchronize;
use centurions_core::combat::runtime::neighbors;

use crate::support::monsters as records;

#[test]
fn ally_placement_matches_original_records_positions_and_map() {
    let fixture = crate::support::fixture("ally-combat.json");
    let large = crate::support::fixture("large-ally-placement.json");
    let mission = crate::support::terrain_mission(&fixture);
    for case in fixture
        .array("cases")
        .iter()
        .chain(large.array("cases"))
        .filter(|v| v["kind"] == "placement")
    {
        let mut map = TacticalMap {
            tiles: words(&case.bytes("map_before")),
            shift: (-39, -15),
            edge_word: 0,
        };
        let menu = case.bytes("menu_before");
        let mut random = RandomState::new(12);
        let result = place_allies(
            &menu,
            case.n::<usize>("count"),
            &mut map,
            &mission,
            (39, 39),
            (5, 5, 85, 85),
            &mut random,
        )
        .unwrap();
        crate::support::native_expect! {
        bytes crate::support::record_bytes(&result.records) => &case["records_after"], "ally placement";
        words map.tiles => &case["map_after"], "ally placement";
        scalar result.records.len() => &case["enemy_count"], "ally placement";
        }
        assert_eq!(
            result.positions,
            crate::support::positions(&case.bytes("positions"))
        );
        assert_eq!(random, RandomState::new(12));
    }
    let mut map = TacticalMap {
        tiles: vec![0; 8100],
        shift: (-5, -5),
        edge_word: 0,
    };
    map.tiles[37 + 37 * 90] = 1155;
    let before = map.clone();
    let menu = fixture
        .array("cases")
        .iter()
        .find(|c| c["count"] == 2)
        .unwrap()
        .bytes("menu_before");
    let mut random = RandomState::new(12);
    assert!(
        place_allies(
            &menu,
            2,
            &mut map,
            &mission,
            (39, 39),
            (5, 5, 85, 85),
            &mut random
        )
        .is_err()
    );
    assert_eq!(map, before);
    assert_eq!(random, RandomState::new(12));
}

#[test]
fn original_bodycount_preserves_selection_order_rng_and_empty_survivor_quirk() {
    let fixture = crate::support::fixture("ally-combat.json");
    for case in fixture
        .array("cases")
        .iter()
        .filter(|v| v["kind"] == "synchronization")
    {
        let mut records = records(&case.bytes("records_before"));
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let result = synchronize(
            &case.bytes("menu_before"),
            case.n::<usize>("count"),
            &mut records,
            &mut random,
        )
        .unwrap();
        crate::support::native_expect! {
        bytes result.menu => &case["menu_after"], "ally synchronization";
        bytes crate::support::record_bytes(&records) => &case["records_after"], "ally synchronization";
        scalar result.count => &case["count_after"], "ally synchronization";
        scalar result.sounds => &case["sounds"], "ally synchronization";
        scalar random.seed => &case["seed_after"], "binary_ally_combat";
        scalar random.draws => &case["rng_draws"], "binary_ally_combat";
        }
        assert_eq!(result.modal, case.n::<u64>("dialogs") > 0);
    }
}

#[test]
fn adjacent_faction_targets_match_original_warrior_scientist_and_hostile_lists() {
    let fixture = crate::support::fixture("ally-combat.json");
    for case in fixture
        .array("cases")
        .iter()
        .filter(|v| v["kind"] == "targets")
    {
        let mut combat = crate::support::battle();
        combat.map.tiles = case.words("map");
        combat.map.shift = (-5, -5);
        combat.squad.positions = vec![(25, 25)];
        combat.squad.under_tiles = vec![1155];
        combat.enemies.records = records(&case.bytes("records_before"));
        combat.enemies.positions = vec![(24, 25), (25, 24)];
        combat.order.clear();
        combat.cursor = 0;
        let warriors = vec![Warrior {
            name: "fixture".into(),
            original_sha256: String::new(),
            bytes: case.bytes("warrior_before"),
        }];
        let actual = neighbors(&combat, &warriors, case.n::<i16>("actor")).unwrap();
        let expected = words(&case.bytes("targets"))[..case.n::<usize>("target_count")].to_vec();
        assert_eq!(
            actual, expected,
            "actor {} dead {}",
            case["actor"], case["dead"]
        );
    }
}
