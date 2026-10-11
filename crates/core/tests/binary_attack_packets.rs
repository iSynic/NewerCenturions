use centurions_content::{Warrior, combat::MonsterRecord};
use centurions_core::{
    RandomState,
    combat::{Combat, EnemySetup, SquadPlacement, TacticalMap, ally_turns, attack, enemy_weapons},
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
        "../../../evidence/fixtures/attack-packets.json"
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
fn combat(records: &[u8], member: i16) -> Combat {
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
            positions: vec![(40 + member, 40); records.len() / 288],
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
fn attack_packets_match_native_complete_writers_and_retention() {
    let fixture = fixture();
    let attacks = data("Data Files/Data S");
    let items = data("Data Files/Data ID");
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 528);
    assert_eq!(
        &attacks[3 * 2842 + 2 * 58..3 * 2842 + 3 * 58],
        bytes(fixture["base4103"].as_str().unwrap())
    );
    for (index, case) in cases.iter().enumerate() {
        let kind = case["kind"].as_str().unwrap();
        let mut warrior = Warrior::read(&bytes(case["warrior_before"].as_str().unwrap())).unwrap();
        let records = bytes(case["records_before"].as_str().unwrap());
        let mut enemy = MonsterRecord {
            bytes: records[..288].to_vec(),
        };
        let retained = bytes(case["packet_before"].as_str().unwrap());
        let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        let slot = case["attack"].as_u64().unwrap() as usize;
        let member = case["member"].as_i64().unwrap() as i16;
        let (result, packet, after_records) = match kind {
            "unarmed" => {
                let (result, packet) = attack::unarmed_attack_with_packet(
                    &mut warrior,
                    &mut enemy,
                    &mut random,
                    &attacks,
                    &retained,
                )
                .unwrap();
                (result, packet, enemy.bytes.clone())
            }
            "natural" => {
                let (result, packet) = attack::natural_attack_with_packet(
                    &mut warrior,
                    &mut enemy,
                    slot,
                    &mut random,
                    &attacks,
                    &retained,
                )
                .unwrap();
                (result, packet, enemy.bytes.clone())
            }
            "melee" => {
                let id = i16::from_be_bytes(enemy.bytes[0x28..0x2a].try_into().unwrap()) as usize;
                let (result, packet) = enemy_weapons::melee_with_packet(
                    &mut warrior,
                    &mut enemy,
                    &items[id * 150..(id + 1) * 150],
                    &mut random,
                    0,
                    40 + member,
                    40 + member,
                    &attacks,
                    &retained,
                )
                .unwrap();
                (result, packet, enemy.bytes.clone())
            }
            "ally" => {
                let mut c = combat(&records, member);
                let mut party = vec![warrior.clone(); 6];
                let (result, packet) = ally_turns::natural_attack_enemy_with_packet(
                    &mut c,
                    &mut party,
                    &mut random,
                    10,
                    11,
                    slot,
                    &attacks,
                    &retained,
                )
                .unwrap();
                let records = c
                    .enemies
                    .records
                    .iter()
                    .flat_map(|record| record.bytes.clone())
                    .collect();
                warrior = party[0].clone();
                (result, packet, records)
            }
            _ => panic!("unknown native packet kind"),
        };
        assert_bytes(
            &packet,
            &bytes(case["packet_after"].as_str().unwrap()),
            &format!("case{index} {kind} packet"),
        );
        assert_bytes(
            &warrior.bytes,
            &bytes(case["warrior_after"].as_str().unwrap()),
            &format!("case{index} {kind} warrior"),
        );
        assert_bytes(
            &after_records,
            &bytes(case["records_after"].as_str().unwrap()),
            &format!("case{index} {kind} enemies"),
        );
        assert_eq!(
            random.seed,
            case["seed_after"].as_u64().unwrap() as u32,
            "case{index} {kind} seed"
        );
        assert_eq!(
            random.draws,
            case["rng_draws"].as_u64().unwrap(),
            "case{index} {kind} draws"
        );
        assert_eq!(
            case["identity_before"], case["identity_after"],
            "case{index} {kind} descriptor identity"
        );
        assert_eq!(
            case["count_before"], case["count_after"],
            "case{index} {kind} count"
        );
        assert_eq!(
            result.sounds,
            case["sounds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|id| id.as_i64().unwrap() as i16)
                .collect::<Vec<_>>(),
            "case{index} {kind} sounds"
        );
    }
}

#[test]
fn unrecovered_packet_writer_preserves_records_and_randomness() {
    let fixture = fixture();
    let case = &fixture["cases"][0];
    let mut warrior = Warrior::read(&bytes(case["warrior_before"].as_str().unwrap())).unwrap();
    warrior.bytes[0x9c..0x9e].copy_from_slice(&1_i16.to_be_bytes());
    let mut enemy = MonsterRecord {
        bytes: bytes(case["records_before"].as_str().unwrap()),
    };
    let mut random = RandomState::new(12);
    let before = (warrior.clone(), enemy.clone(), random.clone());
    assert!(
        attack::natural_attack_with_packet(
            &mut warrior,
            &mut enemy,
            0,
            &mut random,
            &data("Data Files/Data S"),
            &bytes(case["packet_before"].as_str().unwrap())
        )
        .is_err()
    );
    assert_eq!((warrior, enemy, random), before);
}
