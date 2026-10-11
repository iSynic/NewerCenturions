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
        round_stories::{self, RoundEffect, RoundScripts},
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
        "../../../evidence/fixtures/round-stories.json"
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
fn input(
    fixture: &serde_json::Value,
    case: &serde_json::Value,
) -> (Combat, Vec<Warrior>, RandomState) {
    let mut order = vec![-1; 110];
    order[1] = 10;
    let positions = words(&bytes(case, "positions_before"));
    (
        Combat {
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
        RandomState::new(case["seed_before"].as_u64().unwrap() as u32),
    )
}

#[test]
fn original_round_callers_match_records_map_order_slots_party_and_rng() {
    let fixture = fixture();
    let mission = mission(&fixture);
    let definitions = bytes(&fixture, "monsters");
    let extras = bytes(&fixture, "extras");
    for (index, case) in fixture["cases"].as_array().unwrap().iter().enumerate() {
        let (mut combat, warriors, mut random) = input(&fixture, case);
        let slot_words = words(&bytes(case, "slots_before"));
        let mut state = RoundScripts {
            counter: case["round_before"].as_i64().unwrap() as i8,
            slots: slot_words.try_into().unwrap(),
        };
        let mut effects = Vec::new();
        let mut hostile_additions = 0i16;
        for _ in 0..case["calls"].as_u64().unwrap() {
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
            state.counter as i64,
            case["round_after"].as_i64().unwrap(),
            "round case{index}"
        );
        assert_eq!(
            state.slots.to_vec(),
            words(&bytes(case, "slots_after")),
            "slots case{index}"
        );
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
            hostile_additions as i64 + 1,
            case["hostile_count_after"].as_i64().unwrap(),
            "hostiles case{index}"
        );
        let native: Vec<RoundEffect> = case["events"]
            .as_array()
            .unwrap()
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
        assert_eq!(case["actor_after"].as_i64().unwrap(), 0);
        assert_eq!(case["round_flag_after"].as_i64().unwrap(), 0);
    }
}

#[test]
fn unverified_round_actions_and_capacity_overflow_are_transactional() {
    let fixture = fixture();
    let mission = mission(&fixture);
    let case = &fixture["cases"][0];
    let definitions = bytes(&fixture, "monsters");
    let extras = bytes(&fixture, "extras");
    for variant in 0..4 {
        let (mut combat, _, mut random) = input(&fixture, case);
        let mut state = RoundScripts {
            counter: 1,
            slots: [363, 364, 338, 340, 0],
        };
        match variant {
            0 => state.slots[0] = 999,
            1 => {
                while combat.enemies.records.len() < 99 {
                    combat
                        .enemies
                        .records
                        .push(combat.enemies.records[0].clone());
                    combat.enemies.positions.push((20, 20));
                }
            }
            2 => combat.death_queue.processing = true,
            _ => combat.enemies.positions.clear(),
        }
        let before = (combat.clone(), state.clone(), random.clone());
        assert!(
            round_stories::advance(
                &mut combat,
                &mut state,
                &mission,
                &definitions,
                &extras,
                0,
                &mut random
            )
            .is_err()
        );
        assert_eq!((combat, state, random), before);
    }
}
