use centurions_content::{Warrior, combat::MonsterRecord, word};
use centurions_core::{
    RandomState, RuleError, Session,
    combat::{
        damage::{DamageFlags, DamagePacket, resolve_damage},
        lifecycle,
        spells::{
            BodySpellResult, DirectSpell, FieldUsage, resolve_bodies,
            resolve_bodies_with_item_cache, resolve_collision,
        },
    },
};
use serde_json::Value;

fn bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|offset| u8::from_str_radix(&hex[offset..offset + 2], 16).unwrap())
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
    serde_json::from_str(
        &std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../evidence/fixtures/compress-psi.json"),
        )
        .unwrap(),
    )
    .unwrap()
}

fn party(hex: &str) -> Vec<Warrior> {
    bytes(hex)
        .as_chunks::<3138>()
        .0
        .iter()
        .map(|record| Warrior::read(record).unwrap())
        .collect()
}

fn enemies(hex: &str) -> Vec<MonsterRecord> {
    bytes(hex)
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
    if let Some(offset) = actual.iter().zip(expected).position(|(a, b)| a != b) {
        panic!(
            "{context} byte {offset:x}: {} != {}",
            actual[offset], expected[offset]
        );
    }
}

fn assert_records(result: &BodySpellResult, party_hex: &str, monster_hex: &str) {
    assert_bytes(
        &result
            .warriors
            .iter()
            .flat_map(|w| w.bytes.iter().copied())
            .collect::<Vec<_>>(),
        &bytes(party_hex),
        "complete native party",
    );
    assert_bytes(
        &result
            .enemies
            .iter()
            .flat_map(|m| m.bytes.iter().copied())
            .collect::<Vec<_>>(),
        &bytes(monster_hex),
        "complete native monsters",
    );
}

fn audio(result: &BodySpellResult) -> Vec<i16> {
    let mut ordered = result
        .targets
        .iter()
        .map(|(body, outcome)| (*body, outcome.sounds.clone()))
        .collect::<Vec<_>>();
    ordered.extend(result.race_excluded.iter().map(|body| (*body, vec![651])));
    ordered.sort_by_key(|(body, _)| *body);
    result
        .sounds
        .iter()
        .copied()
        .chain(ordered.into_iter().flat_map(|(_, sounds)| sounds))
        .collect()
}

fn verify_receipts(case: &Value, actual: &RandomState) {
    let mut expected = random(&case["before"]);
    for receipt in case["rng"].as_array().unwrap() {
        assert_eq!(
            expected.bound(receipt["bound"].as_i64().unwrap() as i16),
            receipt["result"].as_i64().unwrap() as i16,
        );
    }
    assert_eq!(&expected, actual);
    assert_eq!(expected, random(&case["after"]));
}

fn unchanged_native_buffers(before: &Value, after: &Value) {
    for key in [
        "map",
        "party_positions",
        "monster_positions",
        "fields",
        "packet",
        "identity",
        "count",
        "memory",
        "selected",
        "powers",
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
        "first_item_cache",
        "death_queue",
        "death_queue_count",
    ] {
        assert_eq!(before[key], after[key], "native unchanged {key}");
    }
}

#[test]
fn three_complete_compress_resolutions_match_party_npc_records_rng_and_audio() {
    let fixture = fixture();
    let cases = fixture["cast_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 4);
    for case in cases.iter().skip(1) {
        let before = &case["before"];
        let after = &case["after"];
        let packet = bytes(before["packet"].as_str().unwrap());
        let mask = words(before["mask"].as_str().unwrap());
        let warriors = party(before["party"].as_str().unwrap());
        let monsters = enemies(before["records"].as_str().unwrap());
        assert_eq!(packet[19], 8);
        assert_eq!(word(&packet, 40).unwrap(), 0);
        let work = bytes(before["weapon_work"].as_str().unwrap());
        let ammunition = bytes(before["ammo_work"].as_str().unwrap());
        let mut actual = random(before);
        let result = resolve_bodies(
            &warriors,
            &monsters,
            &mask,
            &DirectSpell {
                packet: &packet,
                level: 2,
                count: 2,
                armor_filter: word(&work, 0x88)
                    .unwrap()
                    .wrapping_add(word(&ammunition, 0x88).unwrap())
                    != 0,
                weapon_sound: word(&work, 0x0e).unwrap(),
            },
            &mut actual,
        )
        .unwrap();
        assert_records(
            &result,
            after["party"].as_str().unwrap(),
            after["records"].as_str().unwrap(),
        );
        verify_receipts(case, &actual);
        // The outer caller also owns native showresults audio, captured separately.
        assert_eq!(
            audio(&result),
            case["native_calls"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|call| call["address"] == "133a18")
                .map(|call| call["arguments"][0].as_i64().unwrap() as i16)
                .collect::<Vec<_>>()
        );
        assert!(result.race_excluded.is_empty());
        assert_eq!(result.targets.len(), 1);
        assert_eq!(
            result.targets[0].0,
            case["target"].as_u64().unwrap() as usize
        );
        unchanged_native_buffers(before, after);
        if case["target"] == 18 {
            assert_eq!(word(&monsters[8].bytes, 0x10).unwrap(), 0);
            assert_eq!(monsters[8].bytes[0x90], 101);
            assert_eq!(result.targets[0].1.damage, 0);
            assert_eq!(actual.draws - random(before).draws, 2);
        } else {
            assert_eq!(actual.draws - random(before).draws, 6);
        }
    }
}

#[test]
fn all_eight_mass_helper_controls_match_native_signed_products_and_complete_records() {
    let fixture = fixture();
    let cases = fixture["mass_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 8);
    for case in cases {
        let before = &case["before"];
        let after = &case["after"];
        let mut monsters = enemies(before["records"].as_str().unwrap());
        let packet = bytes(before["packet"].as_str().unwrap());
        let mut actual = random(before);
        let result = resolve_damage(
            &mut monsters[8].bytes,
            true,
            &DamagePacket {
                bytes: packet,
                attack_count: 2,
            },
            DamageFlags {
                shield: true,
                armor: true,
                saves: true,
                armor_filter: false,
            },
            &mut actual,
        )
        .unwrap();
        assert_eq!(result.total, case["damage"].as_i64().unwrap() as i16);
        assert_eq!(result.total, after["last_amount"].as_i64().unwrap() as i16);
        assert_bytes(
            &monsters
                .iter()
                .flat_map(|m| m.bytes.iter().copied())
                .collect::<Vec<_>>(),
            &bytes(after["records"].as_str().unwrap()),
            "complete native helper records",
        );
        assert_eq!(before["party"], after["party"]);
        assert_eq!(word(&monsters[8].bytes, 0x110).unwrap(), 9);
        unchanged_native_buffers(before, after);
        verify_receipts(case, &actual);
        let events = case["effect_events"].as_array().unwrap();
        let pre = words(
            events.iter().find(|e| e["address"] == "1508dc").unwrap()["components"]
                .as_str()
                .unwrap(),
        );
        let post = words(
            events.iter().find(|e| e["address"] == "15092e").unwrap()["components"]
                .as_str()
                .unwrap(),
        );
        assert_eq!(pre.len(), 9);
        let mass = case["mass_control"].as_i64().unwrap() as i16;
        assert_eq!(
            post,
            pre.iter()
                .map(|amount| amount.wrapping_mul(mass.wrapping_abs()))
                .collect::<Vec<_>>()
        );
        let products = events
            .iter()
            .filter(|e| e["address"] == "15091e")
            .collect::<Vec<_>>();
        assert_eq!(products.len(), 9);
        for (index, product) in products.into_iter().enumerate() {
            assert_eq!(product["component_index"].as_u64().unwrap() as usize, index);
            assert_eq!(
                product["d1"].as_i64().unwrap() as i32,
                i32::from(pre[index]) * i32::from(mass.wrapping_abs())
            );
        }
        assert!(case["sounds"].as_array().unwrap().is_empty());
    }
}

#[test]
fn all_seven_reached_resolutions_match_native_records_cleanup_and_rng() {
    let fixture = fixture();
    let case = &fixture["cast_cases"][0];
    assert!(case["complete_turn"].as_bool().unwrap());
    let mut state: Session =
        serde_json::from_value(fixture["foundation"]["session"].clone()).unwrap();
    let before = &case["before"];
    let cache = bytes(before["first_item_cache"].as_str().unwrap());
    let mut warriors = party(before["party"].as_str().unwrap());
    let mut combat = state.combat.take().unwrap();
    combat.enemies.records = enemies(before["records"].as_str().unwrap());
    combat.unavailable = before["unavailable"].as_i64().unwrap() as i8;
    let mut receipts_random = random(before);
    let receipts = case["rng"].as_array().unwrap();
    let mut cursor = 0;
    let mut impacts = vec![];
    let mut party_deaths = 0;
    let resolutions = case["resolutions"].as_array().unwrap();
    assert_eq!(resolutions.len(), 7);
    for resolution in resolutions {
        while receipts_random.draws < resolution["draws_before"].as_u64().unwrap() {
            let receipt = &receipts[cursor];
            cursor += 1;
            assert_eq!(
                receipts_random.bound(receipt["bound"].as_i64().unwrap() as i16),
                receipt["result"].as_i64().unwrap() as i16
            );
        }
        // The caller's selection and charges supply the actor at this resolution boundary.
        combat.enemies.records[6] =
            enemies(resolution["records_after"].as_str().unwrap())[6].clone();
        let mut actual = receipts_random.clone();
        let packet = bytes(resolution["packet"].as_str().unwrap());
        let mask = words(resolution["mask"].as_str().unwrap());
        let result = resolve_bodies_with_item_cache(
            &warriors,
            &combat.enemies.records,
            &mask,
            &DirectSpell {
                packet: &packet,
                level: resolution["spell_id"].as_i64().unwrap() as i16 % 1000 / 100 - 1,
                count: resolution["count"].as_i64().unwrap() as i16,
                armor_filter: false,
                weapon_sound: 402,
            },
            &cache,
            &mut actual,
        )
        .unwrap();
        while receipts_random.draws < resolution["draws_after"].as_u64().unwrap() {
            let receipt = &receipts[cursor];
            cursor += 1;
            assert_eq!(
                receipts_random.bound(receipt["bound"].as_i64().unwrap() as i16),
                receipt["result"].as_i64().unwrap() as i16
            );
        }
        assert_eq!(actual, receipts_random);
        impacts.extend(audio(&result));
        let applied = words(resolution["applied_mask"].as_str().unwrap());
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
        let defeated = result
            .targets
            .iter()
            .filter_map(|(body, result)| result.killed.then_some(*body as i16))
            .collect::<Vec<_>>();
        warriors = result.warriors;
        combat.enemies.records = result.enemies;
        for body in defeated {
            if body < 10 {
                party_deaths += 1;
            }
            lifecycle::kill_body(&mut combat, &mut warriors, body).unwrap();
        }
        let completed = BodySpellResult {
            warriors: warriors.clone(),
            enemies: combat.enemies.records.clone(),
            sounds: vec![],
            targets: vec![],
            race_excluded: vec![],
        };
        assert_records(
            &completed,
            resolution["party_after"].as_str().unwrap(),
            resolution["records_after"].as_str().unwrap(),
        );
    }
    assert_eq!(cursor, receipts.len());
    assert_eq!(receipts_random, random(&case["after"]));
    assert_eq!(receipts_random.draws - random(before).draws, 115);
    assert_eq!(impacts, [503, 503, 503, 402, 503, 108, 108]);
    assert_eq!(party_deaths, 1);
    assert_eq!(
        combat.unavailable,
        case["after"]["unavailable"].as_i64().unwrap() as i8
    );
    assert_eq!(
        combat.order,
        words(case["after"]["order"].as_str().unwrap())
    );
    assert_eq!(
        combat.map.tiles,
        words(case["after"]["map"].as_str().unwrap())
    );
}

#[test]
fn unverified_categories_mass_packets_and_negative_hp_application_are_atomic() {
    let fixture = fixture();
    let before = &fixture["cast_cases"][3]["before"];
    let warriors = party(before["party"].as_str().unwrap());
    let mut monsters = enemies(before["records"].as_str().unwrap());
    let packet = bytes(before["packet"].as_str().unwrap());
    let mask = words(before["mask"].as_str().unwrap());
    let mut packets = vec![];
    for (offset, value) in [
        (1, 3),
        (4, 0),
        (9, 0),
        (13, 0),
        (19, 7),
        (20, 7),
        (34, 1),
        (42, 0),
        (50, 0x20),
    ] {
        let mut invalid = packet.clone();
        invalid[offset] = value;
        packets.push(invalid);
    }
    // The helper permits low-word negative damage; no complete HP application was executed.
    monsters[8].bytes[0x10..0x12].copy_from_slice(&i16::MAX.to_be_bytes());
    packets.push(packet);
    for packet in packets {
        let mut actual = random(before);
        let context = DirectSpell {
            packet: &packet,
            level: 2,
            count: 2,
            armor_filter: false,
            weapon_sound: 402,
        };
        assert!(matches!(
            resolve_bodies(&warriors, &monsters, &mask, &context, &mut actual),
            Err(RuleError::Unsupported { .. })
        ));
        assert_eq!(actual, random(before));
        let mut usage = FieldUsage {
            slot: 2,
            used: vec![0; 128],
        };
        let old_usage = usage.used.clone();
        assert!(matches!(
            resolve_collision(
                &warriors,
                &monsters,
                &mask,
                &context,
                &mut usage,
                &mut actual
            ),
            Err(RuleError::Unsupported { .. })
        ));
        assert_eq!(actual, random(before));
        assert_eq!(usage.used, old_usage);
    }
}
