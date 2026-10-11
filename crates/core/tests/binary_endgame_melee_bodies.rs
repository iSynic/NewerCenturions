use centurions_content::combat::MonsterRecord;
use centurions_core::{
    RandomState,
    combat::{Combat, EnemySetup, SquadPlacement, TacticalMap, enemy_weapons},
};
fn bytes(value: &str) -> Vec<u8> {
    (0..value.len())
        .step_by(2)
        .map(|offset| u8::from_str_radix(&value[offset..offset + 2], 16).unwrap())
        .collect()
}
fn data(name: &str) -> Vec<u8> {
    std::fs::read(format!(
        "{}/../../NewCenturions/TheNewCenturions/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}
fn fixture() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "../../../evidence/fixtures/endgame-melee-bodies.json"
    ))
    .unwrap()
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
fn combat(records: &[u8]) -> Combat {
    Combat {
        battle: 117,
        map: TacticalMap {
            tiles: vec![1155; 8100],
            shift: (0, 0),
            edge_word: 0,
        },
        squad: SquadPlacement {
            positions: (0..6).map(|i| (40 + i, 40)).collect(),
            under_tiles: vec![1155; 6],
        },
        enemies: EnemySetup {
            records: records
                .as_chunks::<288>()
                .0
                .iter()
                .map(|bytes| MonsterRecord {
                    bytes: bytes.to_vec(),
                })
                .collect(),
            positions: vec![(20, 20), (21, 20), (40, 40)],
        },
        order: vec![-1; 110],
        cursor: 1,
        round: 0,
        weapon_work: centurions_core::combat::weapon_work::WeaponWork::zero(),
        unavailable: 0,
        death_queue: Default::default(),
        spell_fields: vec![0; 1200],
        field_used: vec![0; 128],
        round_scripts: Default::default(),
        attack_packet: Some(vec![0; 58]),
        attack_identity: Some([2, 4, 6]),
    }
}

#[test]
fn endgame_default194_melee_matches_native_full_records_packet_and_randomness() {
    let fixture = fixture();
    let items = data("Data Files/Data ID");
    let attacks = data("Data Files/Data S");
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 120);
    for (index, case) in cases.iter().enumerate() {
        let mut combat = combat(&bytes(case["records_before"].as_str().unwrap()));
        let before = combat.clone();
        let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        let id = case["weapon"].as_u64().unwrap() as usize;
        let result = enemy_weapons::melee_enemy_with_packet(
            &mut combat,
            &mut random,
            (
                10,
                11,
                case["index"].as_u64().unwrap() as usize,
                case["actor"].as_i64().unwrap() as i16,
            ),
            &items[id * 150..(id + 1) * 150],
            0,
            &attacks,
            &[0; 58],
        );
        let (result, packet) =
            result.unwrap_or_else(|error| panic!("case{index} weapon{id}: {error:?}"));
        let actual: Vec<_> = combat
            .enemies
            .records
            .iter()
            .flat_map(|record| record.bytes.iter().copied())
            .collect();
        assert_bytes(
            &actual,
            &bytes(case["records_after"].as_str().unwrap()),
            &format!("case{index} records"),
        );
        assert_bytes(
            &packet,
            &bytes(case["packet_after"].as_str().unwrap()),
            &format!("case{index} packet"),
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
        let sound = if result.damage == 0 {
            650
        } else if case["loaded"] == 0 {
            let b = &before.enemies.records[0].bytes;
            let a = case["index"].as_u64().unwrap() as usize;
            let offset = if b[0x93 + a * 4] == 0 {
                0x95
            } else {
                0x95 + a * 4
            };
            i16::from(b[offset] as i8)
        } else {
            i16::from_be_bytes(items[id * 150 + 14..id * 150 + 16].try_into().unwrap())
        };
        assert_eq!(
            case["sounds"],
            serde_json::json!([sound]),
            "case{index} native sound"
        );
        assert!(result.sounds.is_empty());
    }
}
