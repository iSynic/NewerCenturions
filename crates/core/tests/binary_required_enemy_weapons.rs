use centurions_content::{Warrior, combat::MonsterRecord};
use centurions_core::{RandomState, combat::enemy_weapons};

fn bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}
fn fixture() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "../../../evidence/fixtures/required-enemy-weapons.json"
    ))
    .unwrap()
}
fn attacks() -> Vec<u8> {
    std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../NewCenturions/TheNewCenturions/Data Files/Data S"
    ))
    .unwrap()
}

#[test]
fn required_enemy_preparation_matches_native_item_context_and_packet() {
    let fixture = fixture();
    let attacks = attacks();
    let cases = fixture["preparation_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 108);
    for (index, case) in cases.iter().enumerate() {
        let mut enemy = MonsterRecord {
            bytes: bytes(case["enemy_before"].as_str().unwrap()),
        };
        let prepared = enemy_weapons::prepare(
            &mut enemy,
            &bytes(case["weapon"].as_str().unwrap()),
            &bytes(case["ammo"].as_str().unwrap()),
            &attacks,
        )
        .unwrap();
        assert_eq!(
            prepared.packet,
            bytes(case["packet"].as_str().unwrap()),
            "case{index} packet"
        );
        assert_eq!(
            enemy.bytes,
            bytes(case["enemy_after"].as_str().unwrap()),
            "case{index} enemy"
        );
    }
}

#[test]
fn required_enemy_firearms_match_whole_party_and_native_randomness() {
    let fixture = fixture();
    let attacks = attacks();
    let cases = fixture["firearm_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 216);
    for (index, case) in cases.iter().enumerate() {
        let mut party = bytes(case["warriors_before"].as_str().unwrap());
        let member = case["member"].as_u64().unwrap() as usize;
        let mut warrior = Warrior::read(&party[member * 3138..(member + 1) * 3138]).unwrap();
        let mut enemy = MonsterRecord {
            bytes: bytes(case["enemy_before"].as_str().unwrap()),
        };
        let prepared = enemy_weapons::prepare(
            &mut enemy,
            &bytes(case["weapon"].as_str().unwrap()),
            &bytes(case["ammo"].as_str().unwrap()),
            &attacks,
        )
        .unwrap();
        assert_eq!(
            prepared.packet,
            bytes(case["packet"].as_str().unwrap()),
            "case{index} packet"
        );
        let mut random = RandomState::new(case["seed"].as_u64().unwrap() as u32);
        let result =
            enemy_weapons::resolve(&mut warrior, &mut enemy, &prepared, &mut random, 0).unwrap();
        party[member * 3138..(member + 1) * 3138].copy_from_slice(&warrior.bytes);
        assert_eq!(
            party,
            bytes(case["warriors_after"].as_str().unwrap()),
            "case{index} full party"
        );
        assert_eq!(
            enemy.bytes,
            bytes(case["enemy_after"].as_str().unwrap()),
            "case{index} enemy"
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
        assert_eq!(
            result.sounds,
            case["sounds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_i64().unwrap() as i16)
                .collect::<Vec<_>>(),
            "case{index} sounds"
        );
    }
}

#[test]
fn required_melee177_matches_native_damage_statistics_and_facing() {
    let fixture = fixture();
    let items = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../NewCenturions/TheNewCenturions/Data Files/Data ID"
    ))
    .unwrap();
    let weapon = &items[177 * 150..178 * 150];
    let cases = fixture["melee_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 36);
    for (index, case) in cases.iter().enumerate() {
        let mut party = bytes(case["warriors_before"].as_str().unwrap());
        let member = case["member"].as_u64().unwrap() as usize;
        let mut warrior = Warrior::read(&party[member * 3138..(member + 1) * 3138]).unwrap();
        let mut enemy = MonsterRecord {
            bytes: bytes(case["enemy_before"].as_str().unwrap()),
        };
        let mut random = RandomState::new(case["seed"].as_u64().unwrap() as u32);
        let result = enemy_weapons::melee(
            &mut warrior,
            &mut enemy,
            weapon,
            &mut random,
            0,
            0,
            40 + member as i16,
        )
        .unwrap();
        party[member * 3138..(member + 1) * 3138].copy_from_slice(&warrior.bytes);
        assert_eq!(
            party,
            bytes(case["warriors_after"].as_str().unwrap()),
            "case{index} party"
        );
        assert_eq!(
            enemy.bytes,
            bytes(case["enemy_after"].as_str().unwrap()),
            "case{index} enemy"
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
        assert_eq!(
            result.sounds,
            case["sounds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_i64().unwrap() as i16)
                .collect::<Vec<_>>(),
            "case{index} sounds"
        );
    }
}

#[test]
fn unsupported_enemy_weapon_records_preserve_party_enemy_and_randomness() {
    let fixture = fixture();
    let case = &fixture["melee_cases"][0];
    let party = bytes(case["warriors_before"].as_str().unwrap());
    let mut warrior = Warrior::read(&party[..3138]).unwrap();
    let mut enemy = MonsterRecord {
        bytes: bytes(case["enemy_before"].as_str().unwrap()),
    };
    enemy.bytes[0x28..0x2a].copy_from_slice(&178_i16.to_be_bytes());
    enemy.bytes[0x4e..0x50].fill(0);
    enemy.bytes[0x40..0x42].copy_from_slice(&1_i16.to_be_bytes());
    enemy.bytes[0x96] = 10;
    let before = (warrior.clone(), enemy.clone(), RandomState::new(12));
    let mut random = before.2.clone();
    let items = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../NewCenturions/TheNewCenturions/Data Files/Data ID"
    ))
    .unwrap();
    assert!(
        enemy_weapons::melee(
            &mut warrior,
            &mut enemy,
            &items[177 * 150..178 * 150],
            &mut random,
            0,
            0,
            40,
        )
        .is_err()
    );
    assert_eq!(warrior, before.0);
    assert_eq!(enemy, before.1);
    assert_eq!(random, before.2);
    let case = &fixture["firearm_cases"][0];
    let mut enemy = MonsterRecord {
        bytes: bytes(case["enemy_before"].as_str().unwrap()),
    };
    let mut prepared = enemy_weapons::prepare(
        &mut enemy,
        &bytes(case["weapon"].as_str().unwrap()),
        &bytes(case["ammo"].as_str().unwrap()),
        &attacks(),
    )
    .unwrap();
    prepared.packet.clear();
    let before = (warrior.clone(), enemy.clone(), random.clone());
    assert!(enemy_weapons::resolve(&mut warrior, &mut enemy, &prepared, &mut random, 0).is_err());
    assert_eq!(warrior, before.0);
    assert_eq!(enemy, before.1);
    assert_eq!(random, before.2);
}
