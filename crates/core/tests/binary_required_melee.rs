use centurions_content::{Warrior, combat::MonsterRecord};
use centurions_core::{
    Event, RandomState,
    combat::{Combat, EnemySetup, SquadPlacement, TacticalMap, enemy_weapons, runtime},
};

fn bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}
fn fixture() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "../../../evidence/fixtures/required-melee.json"
    ))
    .unwrap()
}
fn items() -> Vec<Vec<u8>> {
    std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../NewCenturions/TheNewCenturions/Data Files/Data ID"
    ))
    .unwrap()
    .as_chunks::<150>()
    .0
    .iter()
    .map(|record| record.to_vec())
    .collect()
}
fn words(raw: &[u8]) -> Vec<i16> {
    raw.as_chunks::<2>()
        .0
        .iter()
        .map(|word| i16::from_be_bytes(*word))
        .collect()
}
fn assert_bytes(actual: &[u8], expected: &[u8], context: &str) {
    assert_eq!(actual.len(), expected.len(), "{context} size");
    if let Some(offset) = actual.iter().zip(expected).position(|(a, b)| a != b) {
        panic!(
            "{context} byte {offset:x}: {} != {}",
            actual[offset], expected[offset]
        );
    }
}

#[test]
fn default_melee181_selection_matches_native_records_saves_and_both_attack_indices() {
    let fixture = fixture();
    let items = items();
    let cases = fixture["attack_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 288);
    for (index, case) in cases.iter().enumerate() {
        let mut party = bytes(case["warriors_before"].as_str().unwrap());
        let member = case["member"].as_u64().unwrap() as usize;
        let mut warrior = Warrior::read(&party[member * 3138..(member + 1) * 3138]).unwrap();
        let mut enemy = MonsterRecord {
            bytes: bytes(case["enemy_before"].as_str().unwrap()),
        };
        let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        let result = enemy_weapons::melee(
            &mut warrior,
            &mut enemy,
            &items[181],
            &mut random,
            0,
            0,
            40 + member as i16,
        )
        .unwrap();
        party[member * 3138..(member + 1) * 3138].copy_from_slice(&warrior.bytes);
        assert_bytes(
            &party,
            &bytes(case["warriors_after"].as_str().unwrap()),
            &format!("case{index} party"),
        );
        assert_bytes(
            &enemy.bytes,
            &bytes(case["enemy_after"].as_str().unwrap()),
            &format!("case{index} attacker"),
        );
        assert_eq!(
            random.seed,
            case["seed_after"].as_u64().unwrap() as u32,
            "case{index} seed"
        );
        assert_eq!(
            random.draws,
            case["rng_draws"].as_u64().unwrap(),
            "case{index} draws"
        );
        assert!(result.sounds.is_empty());
    }
}

#[test]
fn adjacent_melee181_caller_matches_native_action_count_and_cost() {
    let fixture = fixture();
    let items = items();
    let cases = fixture["caller_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 40);
    for (index, case) in cases.iter().enumerate() {
        let raw = bytes(case["warriors_before"].as_str().unwrap());
        let mut warriors: Vec<_> = raw
            .as_chunks::<3138>()
            .0
            .iter()
            .map(|record| Warrior::read(record).unwrap())
            .collect();
        let mut combat = Combat {
            battle: 117,
            map: TacticalMap {
                tiles: words(&bytes(case["map_before"].as_str().unwrap())),
                shift: (0, 0),
                edge_word: 0,
            },
            squad: SquadPlacement {
                positions: vec![(21, 20), (41, 40), (42, 40), (43, 40), (44, 40), (45, 40)],
                under_tiles: vec![1155; 6],
            },
            enemies: EnemySetup {
                records: vec![MonsterRecord {
                    bytes: bytes(case["enemy_before"].as_str().unwrap()),
                }],
                positions: vec![(20, 20)],
            },
            order: words(&bytes(case["order_before"].as_str().unwrap())),
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
        };
        let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        let events = runtime::check_neighbors_with_definitions(
            &mut combat,
            &mut warriors,
            &mut random,
            10,
            &items,
        )
        .unwrap();
        let actual: Vec<_> = warriors
            .iter()
            .flat_map(|warrior| warrior.bytes.iter().copied())
            .collect();
        assert_bytes(
            &actual,
            &bytes(case["warriors_after"].as_str().unwrap()),
            &format!("case{index} party"),
        );
        assert_bytes(
            &combat.enemies.records[0].bytes,
            &bytes(case["enemy_after"].as_str().unwrap()),
            &format!("case{index} attacker"),
        );
        assert_eq!(
            combat.map.tiles,
            words(&bytes(case["map_after"].as_str().unwrap())),
            "case{index} map"
        );
        assert_eq!(
            combat.order,
            words(&bytes(case["order_after"].as_str().unwrap())),
            "case{index} order"
        );
        assert_eq!(
            random.seed,
            case["seed_after"].as_u64().unwrap() as u32,
            "case{index} seed"
        );
        assert_eq!(
            random.draws,
            case["rng_draws"].as_u64().unwrap(),
            "case{index} draws"
        );
        let resolved: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                Event::AttackResolved {
                    attacker, target, ..
                } => Some((*attacker, *target)),
                _ => None,
            })
            .collect();
        assert_eq!(
            resolved,
            case["calls"]
                .as_array()
                .unwrap()
                .iter()
                .map(|call| (
                    call[0].as_i64().unwrap() as i16,
                    call[1].as_i64().unwrap() as i16
                ))
                .collect::<Vec<_>>(),
            "case{index} ordered attack calls"
        );
    }
}
