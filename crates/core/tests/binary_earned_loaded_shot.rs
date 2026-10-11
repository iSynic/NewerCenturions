use centurions_content::{Content, Warrior, combat::MonsterRecord, word};
use centurions_core::{RandomState, Session, combat::firearm};
use serde_json::Value;

fn fixture() -> (Content, Value) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    (
        Content::load(
            &root.join("NewCenturions"),
            &root.join("analysis/extracted"),
        )
        .unwrap(),
        serde_json::from_str(include_str!(
            "../../../evidence/fixtures/earned-loaded-shot.json"
        ))
        .unwrap(),
    )
}

fn bytes(value: &Value) -> Vec<u8> {
    value
        .as_str()
        .unwrap()
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

fn party(snapshot: &Value) -> Vec<Warrior> {
    bytes(&snapshot["party"])
        .as_chunks::<3138>()
        .0
        .iter()
        .map(|record| Warrior::read(record).unwrap())
        .collect()
}

fn monsters(snapshot: &Value) -> Vec<MonsterRecord> {
    bytes(&snapshot["all_records"])
        .as_chunks::<288>()
        .0
        .iter()
        .map(|record| MonsterRecord {
            bytes: record.to_vec(),
        })
        .collect()
}

fn assert_bytes(actual: &[u8], expected: &[u8], label: &str) {
    assert_eq!(actual.len(), expected.len(), "{label} length");
    if let Some(offset) = actual
        .iter()
        .zip(expected)
        .position(|(actual, expected)| actual != expected)
    {
        panic!(
            "{label} byte {offset}: {} != {}",
            actual[offset], expected[offset]
        );
    }
}

fn assert_party(actual: &[Warrior], snapshot: &Value) {
    assert_eq!(actual.len(), 6);
    for (member, (actual, expected)) in actual.iter().zip(party(snapshot)).enumerate() {
        assert_bytes(&actual.bytes, &expected.bytes, &format!("warrior {member}"));
    }
}

fn assert_monsters(actual: &[MonsterRecord], snapshot: &Value) {
    assert_eq!(actual.len(), 100);
    for (member, (actual, expected)) in actual.iter().zip(monsters(snapshot)).enumerate() {
        assert_bytes(&actual.bytes, &expected.bytes, &format!("NPC {member}"));
    }
}

fn random(snapshot: &Value) -> RandomState {
    RandomState {
        seed: snapshot["seed"].as_u64().unwrap() as u32,
        draws: snapshot["draws"].as_u64().unwrap(),
    }
}

fn put_word(bytes: &mut [u8], offset: usize, value: i16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
}

fn prepare(content: &Content, fixture: &Value, case: &Value) -> firearm::PreparedFirearm {
    let state: Session = serde_json::from_value(
        fixture["foundations"][case["foundation"].as_u64().unwrap() as usize]["session"].clone(),
    )
    .unwrap();
    let records = party(&case["before"]);
    let mission = content
        .map(state.exploration.as_ref().unwrap().map)
        .unwrap();
    firearm::prepare(
        &records[case["actor"].as_u64().unwrap() as usize],
        &content.items[165],
        &content.items[280],
        &mission.combat.as_ref().unwrap().attacks,
    )
    .unwrap()
}

fn caller_debits(warrior: &mut Warrior) {
    let w = &mut warrior.bytes;
    let movement = word(w, 0x3c).unwrap().wrapping_sub(3).max(0);
    let actions = word(w, 0x40).unwrap().wrapping_sub(2);
    put_word(w, 0x3c, movement);
    put_word(w, 0x40, actions);
}

#[test]
fn actual_loaded_shot_preparations_match_native_packets_and_resource_work() {
    let (content, fixture) = fixture();
    for case in fixture["cases"].as_array().unwrap() {
        let prepared = prepare(&content, &fixture, case);
        assert_bytes(
            &prepared.packet,
            &bytes(&case["prepared"]["packet"]),
            "prepared packet",
        );
        assert_eq!(
            centurions_core::combat::player_psi::range(&prepared.packet, prepared.shots).unwrap(),
            case["prepared"]["range"].as_i64().unwrap() as i16
        );
        assert_eq!(
            (
                prepared.slot,
                prepared.shots,
                prepared.rounds,
                prepared.weight,
                prepared.sound,
                prepared.armor_filter
            ),
            (0, 1, 1, 0, 406, false)
        );
        let mut weapon = content.items[165].clone();
        put_word(&mut weapon, 14, 406);
        assert_bytes(
            &weapon,
            &bytes(&case["prepared"]["weapon_work"]),
            "weapon work",
        );
        assert_bytes(
            &content.items[280],
            &bytes(&case["prepared"]["ammo_work"]),
            "ammo work",
        );
        assert_party(&party(&case["before"]), &case["prepared"]);
        assert_monsters(&monsters(&case["before"]), &case["prepared"]);
        assert_eq!(random(&case["before"]), random(&case["prepared"]));
    }
}

#[test]
fn actual_miss_and_lethal_resolver_match_native_before_death_and_getup() {
    let (content, fixture) = fixture();
    for (index, case) in fixture["cases"].as_array().unwrap().iter().enumerate() {
        let prepared = prepare(&content, &fixture, case);
        let actor = case["actor"].as_u64().unwrap() as usize;
        let target = case["target"].as_u64().unwrap() as usize - 10;
        let mut warriors = party(&case["targeted"]);
        let mut records = monsters(&case["targeted"]);
        caller_debits(&mut warriors[actor]);
        let mut random = random(&case["targeted"]);
        let initial = random.clone();
        let result = firearm::resolve(
            &mut warriors[actor],
            &mut records[target],
            &prepared,
            &mut random,
            0,
        )
        .unwrap();
        let checkpoint = case.get("pre_kill").unwrap_or(&case["resolved"]);
        assert_party(&warriors, checkpoint);
        assert_monsters(&records, checkpoint);
        assert_eq!(random, self::random(checkpoint));
        let draw_count = checkpoint["draws"].as_u64().unwrap() - initial.draws;
        assert_eq!(draw_count, [3, 18][index]);
        let mut native_random = initial;
        for draw in &case["rng"].as_array().unwrap()[..draw_count as usize] {
            assert_eq!(
                native_random.bound(draw["bound"].as_i64().unwrap() as i16),
                draw["result"].as_i64().unwrap() as i16
            );
        }
        assert_eq!(random, native_random);
        let phase = case["phase_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|phase| phase["address"] == "151022" && phase["spell_id"] == 4101)
            .unwrap();
        let native_sounds: Vec<i16> = checkpoint["ordered_sounds"].as_array().unwrap()
            [phase["sound_count_before"].as_u64().unwrap() as usize..]
            .iter()
            .map(|sound| sound.as_i64().unwrap() as i16)
            .collect();
        assert_eq!(result.sounds, native_sounds);
        assert_eq!(result.damage, [0, 7][index]);
        assert_eq!(result.killed, index == 1);
        let instance = &warriors[actor].inventory().unwrap()[0].0;
        assert_eq!(word(instance, 2).unwrap(), 0);
        assert_eq!(word(instance, 4).unwrap(), 280);
        assert_eq!(word(instance, 6).unwrap(), 1);
        assert_bytes(
            &prepared.packet,
            &bytes(&case["prepared"]["packet"]),
            "retained prepared packet",
        );
        if index == 1 {
            assert_eq!(word(&records[target].bytes, 0x110).unwrap(), -2);
            assert_eq!(records[target].bytes[0x105], 1);
            assert_eq!(records[target].bytes[0x106], 9);
            assert_eq!(checkpoint["killed_count"], 1);
        }
    }
}

#[test]
fn forged_instances_packets_and_empty_targets_reject_before_randomness() {
    let (content, fixture) = fixture();
    let case = &fixture["cases"][1];
    let mut original = party(&case["targeted"])[4].clone();
    caller_debits(&mut original);
    let original_target = monsters(&case["targeted"])[0].clone();
    let original_random = random(&case["targeted"]);
    for (offset, value) in [
        (0x132 + 3, 1),
        (0x132 + 5, 25),
        (0x132 + 7, 2),
        (0x132 + 8, 1),
        (0x132 + 9, 0),
        (0x132 + 10, 24),
        (0x132 + 13, 2),
        (0x132 + 18, 11),
        (0x132 + 20, 1),
        (0x9dd, 0),
        (0x127, 25),
    ] {
        let mut warrior = original.clone();
        warrior.bytes[offset] = value;
        let before = warrior.clone();
        let mut target = original_target.clone();
        let mut random = original_random.clone();
        assert!(
            firearm::resolve(
                &mut warrior,
                &mut target,
                &prepare(&content, &fixture, case),
                &mut random,
                0
            )
            .is_err()
        );
        assert_eq!(warrior, before);
        assert_eq!(target, original_target);
        assert_eq!(random, original_random);
    }
    for offset in 0..58 {
        let mut prepared = prepare(&content, &fixture, case);
        prepared.packet[offset] ^= 1;
        let mut warrior = original.clone();
        let mut target = original_target.clone();
        let mut random = original_random.clone();
        assert!(firearm::resolve(&mut warrior, &mut target, &prepared, &mut random, 0).is_err());
        assert_eq!(warrior, original);
        assert_eq!(target, original_target);
        assert_eq!(random, original_random);
    }
    for field in 0..6 {
        let mut prepared = prepare(&content, &fixture, case);
        match field {
            0 => prepared.slot = 1,
            1 => prepared.shots = 2,
            2 => prepared.rounds = 2,
            3 => prepared.weight = 1,
            4 => prepared.sound = 405,
            _ => prepared.armor_filter = true,
        }
        let mut warrior = original.clone();
        let mut target = original_target.clone();
        let mut random = original_random.clone();
        assert!(firearm::resolve(&mut warrior, &mut target, &prepared, &mut random, 0).is_err());
        assert_eq!(warrior, original);
        assert_eq!(target, original_target);
        assert_eq!(random, original_random);
    }
    let mut warrior = original.clone();
    let mut empty_target = MonsterRecord {
        bytes: vec![0; 288],
    };
    let before_target = empty_target.clone();
    let mut random = original_random.clone();
    assert!(
        firearm::resolve(
            &mut warrior,
            &mut empty_target,
            &prepare(&content, &fixture, case),
            &mut random,
            0
        )
        .is_err()
    );
    assert_eq!(warrior, original);
    assert_eq!(empty_target, before_target);
    assert_eq!(random, original_random);
}

#[test]
fn firing_admission_does_not_admit_earned_reload_or_ammunition_fitting() {
    let (content, fixture) = fixture();
    for case in fixture["cases"].as_array().unwrap() {
        let state: Session = serde_json::from_value(
            fixture["foundations"][case["foundation"].as_u64().unwrap() as usize]["session"]
                .clone(),
        )
        .unwrap();
        let mut warrior = party(&case["targeted"])[4].clone();
        let before = warrior.clone();
        let mut reserve = state.ammunition[80];
        let original_reserve = reserve;
        let mut random = self::random(&case["targeted"]);
        let original_random = random.clone();
        assert!(firearm::reload(&mut warrior, &mut reserve, true, &mut random).is_err());
        assert_eq!(warrior, before);
        assert_eq!(reserve, original_reserve);
        assert_eq!(random, original_random);
        assert!(
            firearm::fit_ammunition(&warrior, 0, &content, &state.ammunition, false, &mut random)
                .is_err()
        );
        assert_eq!(warrior, before);
        assert_eq!(random, original_random);
    }
}
