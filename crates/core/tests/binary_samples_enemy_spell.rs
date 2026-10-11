use centurions_content::{Warrior, combat::MonsterRecord, word};
use centurions_core::{
    RandomState, RuleError,
    combat::spells::{DirectSpell, FieldUsage, resolve_bodies, resolve_collision},
};
use serde_json::Value;

fn bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&hex[index..index + 2], 16).unwrap())
        .collect()
}

fn words(hex: &str) -> Vec<i16> {
    bytes(hex)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|value| i16::from_be_bytes(*value))
        .collect()
}

fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../evidence/fixtures/samples-enemy-spell.json"
    ))
    .unwrap()
}

fn party(snapshot: &Value) -> Vec<Warrior> {
    bytes(snapshot["party"].as_str().unwrap())
        .as_chunks::<3138>()
        .0
        .iter()
        .map(|record| Warrior::read(record).unwrap())
        .collect()
}

fn enemies(snapshot: &Value) -> Vec<MonsterRecord> {
    bytes(snapshot["records"].as_str().unwrap())
        .as_chunks::<288>()
        .0
        .iter()
        .map(|record| MonsterRecord {
            bytes: record.to_vec(),
        })
        .collect()
}

fn random(snapshot: &Value) -> RandomState {
    RandomState {
        seed: snapshot["seed"].as_u64().unwrap() as u32,
        draws: snapshot["draws"].as_u64().unwrap(),
    }
}

fn assert_bytes(actual: &[u8], expected: &[u8], context: &str) {
    assert_eq!(actual.len(), expected.len(), "{context} length");
    if let Some(index) = actual.iter().zip(expected).position(|(a, b)| a != b) {
        panic!(
            "{context} byte {index:x}: {} != {}",
            actual[index], expected[index]
        );
    }
}

#[test]
fn samples_party_race_exclusion_matches_complete_native_records_rng_and_ordered_audio() {
    let fixture = fixture();
    let mut tested = 0;
    for (index, case) in fixture["cases"].as_array().unwrap().iter().enumerate() {
        if case["complete_turn"].as_bool().unwrap() {
            continue;
        }
        let before = &case["before"];
        let after = &case["after"];
        let mask = words(before["mask"].as_str().unwrap());
        if mask[10..].iter().any(|flag| *flag > 0) {
            continue;
        }
        tested += 1;
        let warriors = party(before);
        let monsters = enemies(before);
        assert_eq!(warriors.len(), 6);
        assert_eq!(monsters.len(), 14);
        let packet = bytes(before["packet"].as_str().unwrap());
        assert_eq!(word(&packet, 50).unwrap(), 0x2000);
        assert_eq!(word(&warriors[3].bytes, 0x32).unwrap(), 3);
        let work = bytes(before["weapon_work"].as_str().unwrap());
        let ammunition = bytes(before["ammo_work"].as_str().unwrap());
        let mut actual_random = random(before);
        let result = resolve_bodies(
            &warriors,
            &monsters,
            &mask,
            &DirectSpell {
                packet: &packet,
                level: 1,
                count: before["count"].as_i64().unwrap() as i16,
                armor_filter: word(&work, 0x88)
                    .unwrap()
                    .wrapping_add(word(&ammunition, 0x88).unwrap())
                    != 0,
                weapon_sound: word(&work, 0x0e).unwrap(),
            },
            &mut actual_random,
        )
        .unwrap();
        assert_bytes(
            &result
                .warriors
                .iter()
                .flat_map(|warrior| warrior.bytes.iter().copied())
                .collect::<Vec<_>>(),
            &bytes(after["party"].as_str().unwrap()),
            &format!("case {index} complete party"),
        );
        assert_bytes(
            &result
                .enemies
                .iter()
                .flat_map(|monster| monster.bytes.iter().copied())
                .collect::<Vec<_>>(),
            &bytes(after["records"].as_str().unwrap()),
            &format!("case {index} complete monsters"),
        );
        assert_eq!(result.race_excluded, [3]);
        let applied = words(case["resolutions"][0]["applied_mask"].as_str().unwrap());
        assert_eq!(
            result
                .targets
                .iter()
                .map(|(body, _)| *body)
                .collect::<Vec<_>>(),
            applied
                .iter()
                .enumerate()
                .filter_map(|(body, flag)| (*flag > 0).then_some(body))
                .collect::<Vec<_>>()
        );
        assert_bytes(
            &result.warriors[3].bytes,
            &warriors[3].bytes,
            "excluded warrior",
        );
        assert_eq!(actual_random, random(after));
        let mut receipts = random(before);
        for receipt in case["rng"].as_array().unwrap() {
            assert_eq!(
                receipts.bound(receipt["bound"].as_i64().unwrap() as i16),
                receipt["result"].as_i64().unwrap() as i16
            );
        }
        assert_eq!(receipts, actual_random);

        let mut ordered = result
            .targets
            .iter()
            .map(|(body, outcome)| (*body, outcome.sounds.clone()))
            .collect::<Vec<_>>();
        ordered.extend(result.race_excluded.iter().map(|body| (*body, vec![651])));
        ordered.sort_by_key(|(body, _)| *body);
        let actual_sounds = result
            .sounds
            .into_iter()
            .chain(ordered.into_iter().flat_map(|(_, sounds)| sounds))
            .collect::<Vec<_>>();
        assert_eq!(
            actual_sounds,
            case["sounds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|sound| sound.as_i64().unwrap() as i16)
                .collect::<Vec<_>>()
        );
        assert_eq!(actual_sounds, [204, 204, 204, 651, 204, 204]);

        for key in [
            "map",
            "party_positions",
            "monster_positions",
            "fields",
            "packet",
            "identity",
            "count",
            "memory",
            "party_buffs",
            "ammunition",
            "weapon_work",
            "ammo_work",
            "item_work",
            "item_context",
            "cursor",
            "order",
            "round",
            "unavailable",
        ] {
            assert_eq!(
                before[key], after[key],
                "native unchanged {key} case {index}"
            );
        }
    }
    assert_eq!(tested, 2);
}

#[test]
fn restricted_monster_targets_remain_diagnostic_without_native_alias_memory() {
    let fixture = fixture();
    let mut tested = 0;
    for case in fixture["cases"].as_array().unwrap() {
        if case["complete_turn"].as_bool().unwrap() {
            continue;
        }
        let before = &case["before"];
        let mask = words(before["mask"].as_str().unwrap());
        if !mask[10..].iter().any(|flag| *flag > 0) {
            continue;
        }
        tested += 1;
        let warriors = party(before);
        let monsters = enemies(before);
        let packet = bytes(before["packet"].as_str().unwrap());
        let mut actual_random = random(before);
        let result = resolve_bodies(
            &warriors,
            &monsters,
            &mask,
            &DirectSpell {
                packet: &packet,
                level: 1,
                count: 1,
                armor_filter: false,
                weapon_sound: 0,
            },
            &mut actual_random,
        );
        assert!(matches!(result, Err(RuleError::Unsupported { .. })));
        assert_eq!(actual_random, random(before));
        let mut usage = FieldUsage {
            slot: 7,
            used: vec![0; 128],
        };
        usage.used[8] = 1;
        let previous_usage = usage.used.clone();
        let collision = resolve_collision(
            &warriors,
            &monsters,
            &mask,
            &DirectSpell {
                packet: &packet,
                level: 1,
                count: 1,
                armor_filter: false,
                weapon_sound: 0,
            },
            &mut usage,
            &mut actual_random,
        );
        assert!(matches!(collision, Err(RuleError::Unsupported { .. })));
        assert_eq!(actual_random, random(before));
        assert_eq!(usage.used, previous_usage);
        assert_eq!(usage.slot, 7);
        assert_bytes(
            &warriors
                .iter()
                .flat_map(|warrior| warrior.bytes.iter().copied())
                .collect::<Vec<_>>(),
            &bytes(before["party"].as_str().unwrap()),
            "unchanged party inputs",
        );
        assert_bytes(
            &monsters
                .iter()
                .flat_map(|monster| monster.bytes.iter().copied())
                .collect::<Vec<_>>(),
            &bytes(before["records"].as_str().unwrap()),
            "unchanged monster inputs",
        );
    }
    assert_eq!(tested, 2);
}

#[test]
fn collision_race_gate_precedes_usage_and_unsupported_restrictions_remain_atomic() {
    let fixture = fixture();
    let before = &fixture["cases"][1]["before"];
    let warriors = party(before);
    let monsters = enemies(before);
    let packet = bytes(before["packet"].as_str().unwrap());
    let mut mask = vec![0; 110];
    mask[3] = 1;
    let context = DirectSpell {
        packet: &packet,
        level: 1,
        count: 1,
        armor_filter: false,
        weapon_sound: 0,
    };
    let mut usage = FieldUsage {
        slot: 7,
        used: vec![0; 128],
    };
    usage.used[8] = 1;
    let previous_usage = usage.used.clone();
    let mut actual_random = random(before);
    let result = resolve_collision(
        &warriors,
        &monsters,
        &mask,
        &context,
        &mut usage,
        &mut actual_random,
    )
    .unwrap();
    assert!(result.targets.is_empty());
    assert_eq!(result.race_excluded, [3]);
    assert_eq!(usage.used, previous_usage);
    assert_eq!(usage.slot, 7);
    for (actual, expected) in result.warriors.iter().zip(&warriors) {
        assert_bytes(&actual.bytes, &expected.bytes, "excluded collision party");
    }
    for (actual, expected) in result.enemies.iter().zip(&monsters) {
        assert_bytes(
            &actual.bytes,
            &expected.bytes,
            "excluded collision monsters",
        );
    }
    let mut prefix = random(before);
    let receipt = &fixture["cases"][4]["rng"][0];
    assert_eq!(prefix.bound(receipt["bound"].as_i64().unwrap() as i16), 1);
    assert_eq!(actual_random, prefix);

    for restriction in [0x4000_i16, i16::MIN, -1] {
        let mut unsupported = packet.clone();
        unsupported[50..52].copy_from_slice(&restriction.to_be_bytes());
        let mut actual_random = random(before);
        let result = resolve_bodies(
            &warriors,
            &monsters,
            &mask,
            &DirectSpell {
                packet: &unsupported,
                ..context
            },
            &mut actual_random,
        );
        assert!(matches!(result, Err(RuleError::Unsupported { .. })));
        assert_eq!(actual_random, random(before));
    }
    for race in [0i16, 17, -1] {
        let mut unsupported = warriors.clone();
        unsupported[3].bytes[0x32..0x34].copy_from_slice(&race.to_be_bytes());
        let mut actual_random = random(before);
        let result = resolve_collision(
            &unsupported,
            &monsters,
            &mask,
            &context,
            &mut usage,
            &mut actual_random,
        );
        assert!(matches!(result, Err(RuleError::Unsupported { .. })));
        assert_eq!(actual_random, random(before));
        assert_eq!(usage.used, previous_usage);
    }
    let mut unrepresented = vec![0; 110];
    unrepresented[109] = 1;
    let mut actual_random = random(before);
    let result = resolve_bodies(
        &warriors,
        &monsters,
        &unrepresented,
        &context,
        &mut actual_random,
    );
    assert!(matches!(result, Err(RuleError::Unsupported { .. })));
    assert_eq!(actual_random, random(before));
}
