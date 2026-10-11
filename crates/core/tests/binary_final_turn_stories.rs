use std::sync::Arc;

use centurions_content::{
    Warrior,
    combat::MonsterRecord,
    mission::{OpeningMission, StoryTables},
};
use centurions_core::{
    RandomState,
    combat::{
        Combat, EnemySetup, SquadPlacement, TacticalMap,
        final_turn_stories::{self, TurnEffect},
    },
};

fn decode(value: &str) -> Vec<u8> {
    (0..value.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&value[i..i + 2], 16).unwrap())
        .collect()
}
fn bytes(value: &serde_json::Value, key: &str) -> Vec<u8> {
    decode(value[key].as_str().unwrap())
}
fn words(bytes: &[u8]) -> Vec<i16> {
    bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| i16::from_be_bytes(*pair))
        .collect()
}
fn fixture() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "../../../evidence/fixtures/final-turn-stories.json"
    ))
    .unwrap()
}
fn mission(fixture: &serde_json::Value) -> OpeningMission {
    OpeningMission {
        name: "native fixture".into(),
        start: (0, 0),
        header: vec![],
        regions: vec![],
        floor: vec![],
        objects: vec![],
        events: vec![],
        tile_properties: bytes(fixture, "properties"),
        terrain: 2,
        default_tile: 1155,
        minutes_per_tick: 5,
        shared: Arc::new(StoryTables {
            door_scripts: bytes(fixture, "scripts"),
            ..Default::default()
        }),
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
                records: bytes(case, "records_before")
                    .as_chunks::<288>()
                    .0
                    .iter()
                    .map(|record| MonsterRecord {
                        bytes: record.to_vec(),
                    })
                    .collect(),
                positions: positions
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|pair| (pair[0], pair[1]))
                    .collect(),
            },
            order,
            cursor: 1,
            round: 0,
            death_queue: Default::default(),
            spell_fields: vec![0; 1200],
            field_used: vec![0; 128],
            attack_packet: Some(vec![0; 58]),
            attack_identity: Some([0; 3]),
            weapon_work: centurions_core::combat::weapon_work::WeaponWork::zero(),
            unavailable: 0,
            round_scripts: Default::default(),
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
        RandomState::new(case["seed_before"].as_u64().unwrap() as u32),
    )
}

#[test]
fn original_final_turn_stories_match_all_body_map_order_party_and_rng_bytes() {
    let fixture = fixture();
    let mission = mission(&fixture);
    let definitions = bytes(&fixture, "monsters");
    let extras = bytes(&fixture, "extras");
    for (index, case) in fixture["cases"].as_array().unwrap().iter().enumerate() {
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
        let records: Vec<u8> = combat
            .enemies
            .records
            .iter()
            .flat_map(|record| record.bytes.iter().copied())
            .collect();
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
        let party: Vec<u8> = warriors
            .iter()
            .flat_map(|warrior| warrior.bytes.iter().copied())
            .collect();
        assert_eq!(party, bytes(case, "warriors_after"), "party case{index}");
        assert_eq!(
            random.seed as u64,
            case["seed_after"].as_u64().unwrap(),
            "seed case{index}"
        );
        assert_eq!(
            random.draws,
            case["rng_draws"].as_u64().unwrap(),
            "draws case{index}"
        );
        assert_eq!(
            outcome.skipped_turn,
            case["skipped_turn"].as_bool().unwrap(),
            "skip case{index}"
        );
        assert_eq!(
            i64::from(outcome.increment_kills),
            case["kills_after"].as_i64().unwrap(),
            "kills case{index}"
        );
        let initial_hostile = i16::from(case["faction"].as_i64().unwrap() != 0);
        assert_eq!(
            i64::from(initial_hostile + outcome.hostile_additions),
            case["hostile_count_after"].as_i64().unwrap(),
            "hostile count case{index}"
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
        let native: Vec<_> = case["events"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| ["sound", "text", "kill"].contains(&event[0].as_str().unwrap()))
            .cloned()
            .collect();
        assert_eq!(effects, native, "effect order case{index}");
    }
}

#[test]
fn unsupported_summon_actions_and_capacity_overflow_preserve_complete_state() {
    let fixture = fixture();
    let mission = mission(&fixture);
    let definitions = bytes(&fixture, "monsters");
    let case = &fixture["cases"][0];
    for variant in 0..3 {
        let chosen = if variant == 2 {
            &fixture["cases"][24]
        } else {
            case
        };
        let (mut combat, mut warriors, mut random) = input(chosen);
        let mut extras = bytes(&fixture, "extras");
        match variant {
            0 => {
                combat.enemies.records[0].bytes[0x5e..0x60].copy_from_slice(&346_i16.to_be_bytes())
            }
            1 => extras[6260 + 2..6260 + 4].copy_from_slice(&65_i16.to_be_bytes()),
            _ => {
                random = RandomState::new(16);
                combat.enemies.records.resize_with(99, || MonsterRecord {
                    bytes: vec![0; 288],
                });
                combat.enemies.positions.resize(99, (0, 0));
            }
        }
        let before = (combat.clone(), warriors.clone(), random.clone());
        assert!(
            final_turn_stories::initialize(
                &mut combat,
                &mut warriors,
                &mission,
                &definitions,
                &extras,
                &mut random
            )
            .is_err()
        );
        assert_eq!((combat, warriors, random), before);
    }
}
