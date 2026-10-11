use std::sync::Arc;

use centurions_content::{
    Warrior,
    combat::MonsterRecord,
    mission::{OpeningMission, StoryTables},
};
use centurions_core::combat::{
    Combat, EnemySetup, SquadPlacement, TacticalMap,
    mission_deaths::{self, DeathContext},
};

fn bytes(value: &serde_json::Value, key: &str) -> Vec<u8> {
    let hex = value[key].as_str().unwrap();
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
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
        "../../../evidence/fixtures/mission-deaths.json"
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
        tile_properties: vec![],
        terrain: 2,
        default_tile: 1155,
        minutes_per_tick: 5,
        shared: Arc::new(StoryTables {
            door_scripts: bytes(fixture, "scripts"),
            ..Default::default()
        }),
    }
}
fn input(case: &serde_json::Value) -> (Combat, Vec<Warrior>, i8) {
    (
        Combat {
            round_scripts: Default::default(),
            battle: 146,
            map: TacticalMap {
                tiles: words(&bytes(case, "map_before")),
                edge_word: 0,
                shift: (0, 0),
            },
            squad: SquadPlacement {
                positions: vec![(40, 40); 6],
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
                positions: vec![(20, 20); 100],
            },
            order: words(&bytes(case, "order_before")),
            cursor: 1,
            round: 0,
            death_queue: Default::default(),
            spell_fields: vec![0; 1200],
            field_used: vec![0; 128],
            attack_packet: Some(vec![0; 58]),
            attack_identity: Some([0; 3]),
            weapon_work: centurions_core::combat::weapon_work::WeaponWork::zero(),
            unavailable: 0,
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
        case["unavailable_before"].as_i64().unwrap() as i8,
    )
}
fn context(case: &serde_json::Value) -> DeathContext {
    DeathContext {
        script: if case["monster_id"].as_i64().unwrap() == 119 {
            336
        } else {
            337
        },
        actor: case["actor_before"].as_u64().unwrap() as usize,
        deferred_faction: (case["processing_before"].as_i64().unwrap() != 0)
            .then(|| case["faction"].as_i64().unwrap() as i16),
    }
}

#[test]
fn original_direct_and_queued_death_story_phases_match_complete_native_state() {
    let fixture = fixture();
    let mission = mission(&fixture);
    let extras = bytes(&fixture, "extras");
    for (index, case) in fixture["cases"].as_array().unwrap().iter().enumerate() {
        let (mut combat, mut warriors, mut unavailable) = input(case);
        let result = mission_deaths::apply(
            &mut combat,
            &mut warriors,
            &mission,
            &extras,
            context(case),
            &mut unavailable,
        )
        .unwrap();
        let records: Vec<u8> = combat
            .enemies
            .records
            .iter()
            .flat_map(|record| record.bytes.iter().copied())
            .collect();
        let party: Vec<u8> = warriors
            .iter()
            .flat_map(|warrior| warrior.bytes.iter().copied())
            .collect();
        assert_eq!(records, bytes(case, "records_after"), "records case{index}");
        assert_eq!(party, bytes(case, "warriors_after"), "party case{index}");
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
        assert_eq!(
            unavailable as i64,
            case["unavailable_after"].as_i64().unwrap(),
            "unavailable case{index}"
        );
        assert_eq!(
            result.ended_early,
            case["return_value"].as_i64().unwrap() == -1,
            "return case{index}"
        );
        assert_eq!(
            result.text,
            if context(case).script == 336 {
                1247
            } else {
                1248
            }
        );
        assert_eq!(case["rng_draws"].as_u64().unwrap(), 0);
        assert_eq!(case["actor_after"].as_u64().unwrap(), 0);
        let queued = case["queued"].as_u64().unwrap();
        assert_eq!(context(case).actor, if queued == 0 { 3 } else { 0 });
        assert_eq!(
            case["queued_count"].as_u64().unwrap(),
            u64::from(queued != 0)
        );
        assert_eq!(case["queue_count_after"].as_u64().unwrap(), 0);
    }
}

#[test]
fn altered_death_script_and_invalid_actor_are_transactional_diagnostics() {
    let fixture = fixture();
    let mission = mission(&fixture);
    let extras = bytes(&fixture, "extras");
    let case = &fixture["cases"][0];
    for variant in 0..3 {
        let (mut combat, mut warriors, mut unavailable) = input(case);
        let mut context = context(case);
        match variant {
            0 => context.script = 999,
            1 => context.actor = 100,
            _ => warriors[0].bytes.clear(),
        }
        let before = (combat.clone(), warriors.clone(), unavailable);
        assert!(
            mission_deaths::apply(
                &mut combat,
                &mut warriors,
                &mission,
                &extras,
                context,
                &mut unavailable
            )
            .is_err()
        );
        assert_eq!((combat, warriors, unavailable), before);
    }
}
