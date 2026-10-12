use crate::prelude::*;

use crate::support::field_bytes as bytes;
use centurions_core::combat::final_turn_stories;
use centurions_core::combat::final_turn_stories::TurnEffect;

fn fixture() -> serde_json::Value {
    crate::support::fixture("final-turn-stories.json")
}
fn mission(fixture: &serde_json::Value) -> OpeningMission {
    OpeningMission {
        name: "native fixture".into(),
        start: (0, 0),
        tile_properties: bytes(fixture, "properties"),
        default_tile: 1155,
        shared: Arc::new(StoryTables {
            door_scripts: bytes(fixture, "scripts"),
            ..Default::default()
        }),
        ..crate::support::mission()
    }
}
fn input(case: &serde_json::Value) -> (Combat, Vec<Warrior>, RandomState) {
    let mut order = vec![-1; 110];
    order[1] = 10;
    let positions = words(&bytes(case, "positions_before"));
    (
        Combat {
            battle: 134,
            map: TacticalMap {
                tiles: words(&bytes(case, "map_before")),
                edge_word: 0,
                shift: (0, 0),
            },
            squad: SquadPlacement {
                positions: (0..6).map(|i| (40 + i, 40)).collect(),
                under_tiles: vec![1155; 6],
            },
            enemies: EnemySetup {
                records: crate::support::monsters(&bytes(case, "records_before")),
                positions: positions
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|pair| (pair[0], pair[1]))
                    .collect(),
            },
            order,
            ..crate::support::battle()
        },
        bytes(case, "warriors_before")
            .as_chunks::<3138>()
            .0
            .iter()
            .map(|record| Warrior {
                name: "fixture".into(),
                original_sha256: String::new(),
                bytes: record.to_vec(),
            })
            .collect(),
        RandomState::new(case.n::<u32>("seed_before")),
    )
}

#[test]
fn original_final_turn_stories_match_all_body_map_order_party_and_rng_bytes() {
    let fixture = fixture();
    let mission = mission(&fixture);
    let definitions = bytes(&fixture, "monsters");
    let extras = bytes(&fixture, "extras");
    for (index, case) in fixture.array("cases").iter().enumerate() {
        let (mut combat, mut warriors, mut random) = input(case);
        let outcome = final_turn_stories::initialize(
            &mut combat,
            &mut warriors,
            &mission,
            &definitions,
            &extras,
            &mut random,
        )
        .unwrap_or_else(|error| panic!("case{index}: {error}"));
        let records: Vec<u8> = crate::support::record_bytes(&combat.enemies.records);
        assert_eq!(records, bytes(case, "records_after"), "records case{index}");
        assert_eq!(
            combat.map.tiles,
            words(&bytes(case, "map_after")),
            "map case{index}"
        );
        assert_eq!(
            combat.order,
            words(&bytes(case, "order_after")),
            "order case{index}"
        );
        let positions: Vec<i16> = combat
            .enemies
            .positions
            .iter()
            .flat_map(|&(x, y)| [x, y])
            .collect();
        assert_eq!(
            positions,
            words(&bytes(case, "positions_after")),
            "positions case{index}"
        );
        let party: Vec<u8> = crate::support::record_bytes(&warriors);
        assert_eq!(party, bytes(case, "warriors_after"), "party case{index}");
        crate::support::native_expect! {
        scalar random.seed as u64 => &case["seed_after"], "binary_final_turn_stories";
        scalar random.draws => &case["rng_draws"], "binary_final_turn_stories";
        scalar outcome.skipped_turn => &case["skipped_turn"], "binary_final_turn_stories";
        scalar i64::from(outcome.increment_kills) => &case["kills_after"], "binary_final_turn_stories";
        }
        let initial_hostile = i16::from(case.n::<i64>("faction") != 0);
        crate::support::expect(
            i64::from(initial_hostile + outcome.hostile_additions),
            &case["hostile_count_after"],
        );
        let effects: Vec<serde_json::Value> = outcome
            .effects
            .iter()
            .filter_map(|effect| match effect {
                TurnEffect::Sound(id) => Some(serde_json::json!(["sound", id])),
                TurnEffect::Text(id) => Some(serde_json::json!(["text", id])),
                TurnEffect::SelfRemoval { body } => Some(serde_json::json!(["kill", body])),
                TurnEffect::Spawn { .. } => None,
            })
            .collect();
        let native: Vec<_> = case
            .array("events")
            .iter()
            .filter(|event| ["sound", "text", "kill"].contains(&event[0].as_str().unwrap()))
            .cloned()
            .collect();
        assert_eq!(effects, native, "effect order case{index}");
    }
}
