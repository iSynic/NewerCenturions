use centurions_content::{Content, Warrior, combat::MonsterRecord};
use centurions_core::{
    Event, RandomState,
    combat::{
        Combat, EnemySetup, SquadPlacement, TacticalMap,
        ally_turns::natural_attack_enemy_with_actor, runtime::check_neighbors_with_context,
    },
};

fn bytes(value: &str) -> Vec<u8> {
    (0..value.len())
        .step_by(2)
        .map(|offset| u8::from_str_radix(&value[offset..offset + 2], 16).unwrap())
        .collect()
}
fn words(raw: &[u8]) -> Vec<i16> {
    raw.as_chunks::<2>()
        .0
        .iter()
        .map(|value| i16::from_be_bytes(*value))
        .collect()
}
fn assert_bytes(actual: &[u8], expected: &[u8], context: &str) {
    assert_eq!(actual.len(), expected.len(), "{context} size");
    if let Some(offset) = actual.iter().zip(expected).position(|(a, b)| a != b) {
        panic!(
            "{context} byte{offset:x}: {} != {}",
            actual[offset], expected[offset]
        );
    }
}
fn sound_ids(events: &[Event]) -> Vec<i16> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::Sound { resource_id } => Some(*resource_id),
            _ => None,
        })
        .collect()
}

#[test]
fn monster_elemental_matches_native_signed_conditions_and_whole_caller() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/monster-elemental.json"
    ))
    .unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let content = Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap();
    let mission = content.map(0).unwrap();
    let attacks = &mission.combat.as_ref().unwrap().attacks;
    assert_eq!(fixture["attack_cases"].as_array().unwrap().len(), 256);
    assert_eq!(fixture["caller_cases"].as_array().unwrap().len(), 16);
    for family in ["attack_cases", "caller_cases"] {
        for (index, case) in fixture[family].as_array().unwrap().iter().enumerate() {
            let context = format!("{family} case{index}");
            let source = case["source"].as_i64().unwrap() as i16;
            let target = case["target"].as_i64().unwrap() as i16;
            let before_records = bytes(case["records_before"].as_str().unwrap());
            let positions = if source == 10 {
                vec![(20, 20), (21, 20), (40, 40)]
            } else {
                vec![(21, 20), (40, 40), (20, 20)]
            };
            let mut combat = Combat {
                battle: 15,
                map: TacticalMap {
                    tiles: words(&bytes(case["map_before"].as_str().unwrap())),
                    shift: (0, 0),
                    edge_word: 0,
                },
                squad: SquadPlacement {
                    positions: (0..6).map(|i| (40 + i, 40)).collect(),
                    under_tiles: vec![1155; 6],
                },
                enemies: EnemySetup {
                    records: before_records
                        .as_chunks::<288>()
                        .0
                        .iter()
                        .map(|record| MonsterRecord {
                            bytes: record.to_vec(),
                        })
                        .collect(),
                    positions,
                },
                order: words(&bytes(case["order_before"].as_str().unwrap())),
                cursor: 1,
                round: 0,
                weapon_work: centurions_core::combat::weapon_work::WeaponWork::zero(),
                unavailable: 0,
                death_queue: Default::default(),
                spell_fields: vec![0; 1200],
                field_used: vec![0; 128],
                round_scripts: Default::default(),
                attack_packet: Some(vec![0; 58]),
                attack_identity: Some([0; 3]),
            };
            let mut warriors: Vec<_> = bytes(fixture["party"].as_str().unwrap())
                .as_chunks::<3138>()
                .0
                .iter()
                .map(|record| Warrior::read(record).unwrap())
                .collect();
            let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
            let sounds = if family == "attack_cases" {
                let (result, packet) = natural_attack_enemy_with_actor(
                    &mut combat,
                    &mut warriors,
                    &mut random,
                    (
                        source,
                        target,
                        case["index"].as_u64().unwrap() as usize,
                        case["actor"].as_i64().unwrap() as i16,
                    ),
                    attacks,
                    &[0; 58],
                )
                .unwrap_or_else(|error| panic!("{context}: {error:?}"));
                combat.attack_packet = Some(packet);
                let mut sounds = result.sounds;
                sounds.push(
                    centurions_core::combat::ally_turns::result_audio(
                        &combat.enemies.records[(source - 10) as usize],
                        case["index"].as_u64().unwrap() as usize,
                        result.damage,
                        0,
                    )
                    .unwrap(),
                );
                sounds
            } else {
                let events = check_neighbors_with_context(
                    &mut combat,
                    &mut warriors,
                    &mut random,
                    source,
                    &content.items,
                    attacks,
                )
                .unwrap_or_else(|error| panic!("{context}: {error:?}"));
                sound_ids(&events)
            };
            let actual: Vec<_> = combat
                .enemies
                .records
                .iter()
                .flat_map(|record| record.bytes.iter().copied())
                .collect();
            assert_bytes(
                &actual,
                &bytes(case["records_after"].as_str().unwrap()),
                &format!("{context} records"),
            );
            assert_bytes(
                combat.attack_packet.as_ref().unwrap(),
                &bytes(case["packet_after"].as_str().unwrap()),
                &format!("{context} packet"),
            );
            assert_eq!(
                combat.map.tiles,
                words(&bytes(case["map_after"].as_str().unwrap())),
                "{context} map"
            );
            assert_eq!(
                combat.order,
                words(&bytes(case["order_after"].as_str().unwrap())),
                "{context} order"
            );
            let positions = words(&bytes(case["positions_after"].as_str().unwrap()));
            assert_eq!(
                combat.enemies.positions,
                positions
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|p| (p[0], p[1]))
                    .collect::<Vec<_>>(),
                "{context} positions"
            );
            assert_bytes(
                &warriors
                    .iter()
                    .flat_map(|warrior| warrior.bytes.iter().copied())
                    .collect::<Vec<_>>(),
                &bytes(fixture["party"].as_str().unwrap()),
                &format!("{context} party"),
            );
            assert_eq!(
                random.seed,
                case["seed_after"].as_u64().unwrap() as u32,
                "{context} seed"
            );
            assert_eq!(
                random.draws,
                case["rng_draws"].as_u64().unwrap(),
                "{context} draws"
            );
            assert_eq!(
                serde_json::json!(sounds),
                case["sounds"],
                "{context} sounds"
            );
        }
    }
}
