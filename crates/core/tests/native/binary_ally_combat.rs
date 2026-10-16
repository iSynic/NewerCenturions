use crate::prelude::*;

use centurions_core::combat::allies::place_allies;
use centurions_core::combat::allies::synchronize;
use centurions_core::combat::runtime::neighbors;

use crate::support::field_bytes as bytes;
use crate::support::monsters as records;

#[test]
fn ally_placement_matches_original_records_positions_and_map() {
    let fixture = crate::support::fixture("ally-combat.json");
    let properties = bytes(&fixture, "properties");
    let mission = OpeningMission {
        tile_properties: properties[64..12064].to_vec(),
        shared: Default::default(),
        ..crate::support::mission()
    };
    for case in fixture
        .array("cases")
        .iter()
        .filter(|v| v["kind"] == "placement")
    {
        let mut map = TacticalMap {
            tiles: words(&bytes(case, "map_before")),
            shift: (-39, -15),
            edge_word: 0,
        };
        let menu = bytes(case, "menu_before");
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
        assert_eq!(
            result
                .records
                .iter()
                .flat_map(|m| &m.bytes)
                .copied()
                .collect::<Vec<_>>(),
            bytes(case, "records_after")
        );
        assert_eq!(
            result.positions,
            words(&bytes(case, "positions"))
                .as_chunks::<2>()
                .0
                .iter()
                .map(|p| (p[0], p[1]))
                .collect::<Vec<_>>()
        );
        assert_eq!(map.tiles, words(&bytes(case, "map_after")));
        assert_eq!(menu, bytes(case, "menu_after"));
        crate::support::expect(result.records.len(), &case["enemy_count"]);
        assert_eq!(random, RandomState::new(12));
    }
}

#[test]
fn original_bodycount_preserves_selection_order_rng_and_empty_survivor_quirk() {
    let fixture = crate::support::fixture("ally-combat.json");
    for case in fixture
        .array("cases")
        .iter()
        .filter(|v| v["kind"] == "synchronization")
    {
        let mut records = records(&bytes(case, "records_before"));
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let result = synchronize(
            &bytes(case, "menu_before"),
            case.n::<usize>("count"),
            &mut records,
            &mut random,
        )
        .unwrap();
        assert_eq!(
            result.menu,
            bytes(case, "menu_after"),
            "count {} dead {} seed {}",
            case["count"],
            case["dead"],
            case["seed_before"]
        );
        crate::support::expect(result.count, &case["count_after"]);
        assert_eq!(result.modal, case.n::<u64>("dialogs") > 0);
        crate::support::expect_array(&(result.sounds), &case["sounds"]);
        assert_eq!(
            records
                .iter()
                .flat_map(|m| &m.bytes)
                .copied()
                .collect::<Vec<_>>(),
            bytes(case, "records_after")
        );
        crate::support::native_expect! {
        scalar random.seed => &case["seed_after"], "binary_ally_combat";
        scalar random.draws => &case["rng_draws"], "binary_ally_combat";
        }
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
        let combat = Combat {
            map: TacticalMap {
                tiles: words(&bytes(case, "map")),
                shift: (-5, -5),
                edge_word: 0,
            },
            squad: SquadPlacement {
                positions: vec![(25, 25)],
                under_tiles: vec![1155],
            },
            enemies: EnemySetup {
                records: records(&bytes(case, "records_before")),
                positions: vec![(24, 25), (25, 24)],
            },
            order: vec![],
            cursor: 0,
            ..crate::support::battle()
        };
        let warriors = vec![Warrior {
            name: "fixture".into(),
            original_sha256: String::new(),
            bytes: bytes(case, "warrior_before"),
        }];
        let actual = neighbors(&combat, &warriors, case.n::<i16>("actor")).unwrap();
        let expected = words(&bytes(case, "targets"))[..case.n::<usize>("target_count")].to_vec();
        assert_eq!(
            actual, expected,
            "actor {} dead {}",
            case["actor"], case["dead"]
        );
        assert_eq!(bytes(case, "records_before"), bytes(case, "records_after"));
        assert_eq!(bytes(case, "warrior_before"), bytes(case, "warrior_after"));
    }
}
