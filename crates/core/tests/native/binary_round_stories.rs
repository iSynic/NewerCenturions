use crate::prelude::*;

use crate::support::field_bytes as bytes;
use centurions_core::combat::round_stories;
use centurions_core::combat::round_stories::RoundEffect;

#[test]
fn summoned_difficulty_matches_original_record_widths_rounding_and_rng() {
    let fixture = crate::support::fixture("difficulty-summons.json");
    for case in fixture.array("cases") {
        let before = MonsterRecord {
            bytes: case.bytes("before"),
        };
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let after = centurions_core::combat::monster::summon(
            before,
            case.n::<i16>("faction"),
            case.n::<i16>("difficulty"),
            &mut random,
        )
        .unwrap();
        assert_eq!(after.bytes, case.bytes("after"), "{}", case["case"]);
        assert_eq!(
            (random.seed, random.draws),
            (case.n::<u32>("seed_after"), case.n::<u64>("draws"))
        );
    }
}

fn fixture() -> serde_json::Value {
    crate::support::fixture("round-stories.json")
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
fn input(
    fixture: &serde_json::Value,
    case: &serde_json::Value,
) -> (Combat, Vec<Warrior>, RandomState) {
    let mut order = vec![-1; 110];
    order[1] = 10;
    let positions = words(&bytes(case, "positions_before"));
    (
        Combat {
            difficulty: 0,
            battle: 146,
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
        bytes(fixture, "warriors_before")
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
fn original_round_callers_match_records_map_order_slots_party_and_rng() {
    let fixture = fixture();
    let mission = mission(&fixture);
    let definitions = bytes(&fixture, "monsters");
    let extras = bytes(&fixture, "extras");
    for (index, case) in fixture.array("cases").iter().enumerate() {
        let (mut combat, warriors, mut random) = input(&fixture, case);
        let slot_words = words(&bytes(case, "slots_before"));
        let mut state = RoundScripts {
            counter: case.n::<i8>("round_before"),
            slots: slot_words.try_into().unwrap(),
        };
        let mut effects = Vec::new();
        let mut hostile_additions = 0i16;
        for _ in 0..case.n::<u64>("calls") {
            let result = round_stories::advance(
                &mut combat,
                &mut state,
                &mission,
                &definitions,
                &extras,
                0,
                &mut random,
            )
            .unwrap_or_else(|error| panic!("case{index}: {error}"));
            hostile_additions = hostile_additions.wrapping_add(result.hostile_additions);
            effects.extend(result.effects);
        }
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
        crate::support::expect(state.counter as i64, &case["round_after"]);
        assert_eq!(
            state.slots.to_vec(),
            words(&bytes(case, "slots_after")),
            "slots case{index}"
        );
        crate::support::native_expect! {
        scalar random.seed as u64 => &case["seed_after"], "binary_round_stories";
        scalar random.draws => &case["rng_draws"], "binary_round_stories";
        scalar hostile_additions as i64 + 1 => &case["hostile_count_after"], "binary_round_stories";
        }
        let native: Vec<RoundEffect> = case
            .array("events")
            .iter()
            .filter_map(|event| {
                let id = event[1].as_i64()? as i16;
                match event[0].as_str()? {
                    "sound" => Some(RoundEffect::Sound(id)),
                    "text" => Some(RoundEffect::Text(id)),
                    "spawn" => Some(RoundEffect::Spawn {
                        body: id,
                        monster: event[2].as_i64()? as i16,
                    }),
                    _ => None,
                }
            })
            .collect();
        assert_eq!(effects, native, "ordered effects case{index}");
    }
}
