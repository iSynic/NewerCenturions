use centurions_content::{Content, Warrior, combat::MonsterRecord, word};
use centurions_core::{
    RandomState, RuleError, Session,
    combat::{
        spell_conditions,
        spells::{
            BodySpellResult, DirectSpell, FieldUsage, resolve_bodies,
            resolve_bodies_with_item_cache, resolve_collision, select_targets,
        },
    },
};
use serde_json::Value;

fn fixture() -> Value {
    serde_json::from_str(include_str!("../../../evidence/fixtures/poison-psi.json")).unwrap()
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

fn words(value: &Value) -> Vec<i16> {
    bytes(value)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|word| i16::from_be_bytes(*word))
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

fn monsters(snapshot: &Value, key: &str) -> Vec<MonsterRecord> {
    bytes(&snapshot[key])
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

fn assert_bytes(actual: &[u8], expected: &[u8], label: &str) {
    assert_eq!(actual.len(), expected.len(), "{label} length");
    if let Some(offset) = actual.iter().zip(expected).position(|(a, b)| a != b) {
        panic!(
            "{label} byte {offset:x}: {} != {}",
            actual[offset], expected[offset]
        );
    }
}

fn assert_records(result: &BodySpellResult, expected: &Value, monster_key: &str) {
    assert_bytes(
        &result
            .warriors
            .iter()
            .flat_map(|warrior| warrior.bytes.iter().copied())
            .collect::<Vec<_>>(),
        &bytes(&expected["party"]),
        "complete party",
    );
    assert_bytes(
        &result
            .enemies
            .iter()
            .flat_map(|monster| monster.bytes.iter().copied())
            .collect::<Vec<_>>(),
        &bytes(&expected[monster_key]),
        "complete monsters",
    );
}

fn audio(result: &BodySpellResult) -> Vec<i16> {
    let mut sounds = result
        .targets
        .iter()
        .map(|(body, result)| (*body, result.sounds.clone()))
        .collect::<Vec<_>>();
    sounds.extend(result.race_excluded.iter().map(|body| (*body, vec![651])));
    sounds.sort_by_key(|(body, _)| *body);
    result
        .sounds
        .iter()
        .copied()
        .chain(sounds.into_iter().flat_map(|(_, sounds)| sounds))
        .collect()
}

fn receipts(case: &Value, initial: &RandomState, actual: &RandomState) {
    let mut expected = initial.clone();
    for draw in case["rng"].as_array().unwrap() {
        assert_eq!(
            expected.bound(draw["bound"].as_i64().unwrap() as i16),
            draw["result"].as_i64().unwrap() as i16
        );
    }
    assert_eq!(&expected, actual);
}

fn unchanged_direct_buffers(before: &Value, after: &Value) {
    for key in [
        "map",
        "party_positions",
        "monster_positions",
        "all_monster_positions",
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
        "round_slots",
        "unavailable",
        "first_item_cache",
        "references",
        "ally_count",
        "fatigue",
        "death_queue",
        "death_queue_count",
        "selected_weapon",
    ] {
        assert_eq!(before[key], after[key], "native unchanged {key}");
    }
}

#[test]
fn twelve_full_direct_poison_cases_match_signed_duration_gates_records_rng_and_audio() {
    let f = fixture();
    let cases = f["direct_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 12);
    for (index, case) in cases.iter().enumerate() {
        let before = &case["before"];
        let after = &case["after"];
        let warriors = party(before);
        let enemies = monsters(before, "all_records");
        let packet = bytes(&before["packet"]);
        let mask = words(&before["mask"]);
        let initial = random(before);
        let mut actual = initial.clone();
        let weapon = bytes(&before["weapon_work"]);
        let ammo = bytes(&before["ammo_work"]);
        let result = resolve_bodies(
            &warriors,
            &enemies,
            &mask,
            &DirectSpell {
                packet: &packet,
                level: words(&before["identity"])[1],
                count: case["count"].as_i64().unwrap() as i16,
                armor_filter: word(&weapon, 136)
                    .unwrap()
                    .wrapping_add(word(&ammo, 136).unwrap())
                    != 0,
                weapon_sound: word(&weapon, 14).unwrap(),
            },
            &mut actual,
        )
        .unwrap();
        assert_records(&result, after, "all_records");
        assert_eq!(actual, random(after), "case {index}");
        receipts(case, &initial, &actual);
        assert_eq!(
            audio(&result),
            case["sounds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|sound| sound.as_i64().unwrap() as i16)
                .collect::<Vec<_>>(),
            "case {index}"
        );
        assert!(
            result
                .targets
                .iter()
                .all(|(_, result)| result.damage == 0 && !result.killed)
        );
        assert_eq!(word(&packet, 40).unwrap(), 4);
        assert_eq!(word(&packet, 50).unwrap(), 0x2000);
        assert_eq!(
            result.race_excluded,
            if index == 9 { vec![4] } else { vec![] }
        );
        assert_eq!(result.targets.len(), usize::from(!matches!(index, 9 | 10)));
        assert!(result.enemies.iter().all(|enemy| enemy.bytes[0xb5] == 0));
        unchanged_direct_buffers(before, after);
        assert_bytes(
            &warriors
                .iter()
                .flat_map(|warrior| warrior.bytes.iter().copied())
                .collect::<Vec<_>>(),
            &bytes(&before["party"]),
            "immutable input party",
        );
        assert_bytes(
            &enemies
                .iter()
                .flat_map(|enemy| enemy.bytes.iter().copied())
                .collect::<Vec<_>>(),
            &bytes(&before["all_records"]),
            "immutable input monsters",
        );
        if index < 9 {
            let duration = after["duration"].as_i64().unwrap() as i16;
            let mut target = warriors[4].bytes.clone();
            let mut condition_random = initial.clone();
            spell_conditions::apply(&mut target, false, 4, duration, &mut condition_random)
                .unwrap();
            assert_bytes(
                &target,
                &result.warriors[4].bytes,
                "standalone condition addition",
            );
            assert_eq!(condition_random, initial);
        }
    }
}

#[test]
fn actual_actor_target_charge_and_resolution_match_native_complete_boundaries() {
    let f = fixture();
    let case = &f["cases"][0];
    let before = &case["before"];
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let content = Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap();
    let state: Session = serde_json::from_value(f["foundation"]["session"].clone()).unwrap();
    let mission = content
        .map(state.exploration.as_ref().unwrap().map)
        .unwrap();
    let mut combat = state.combat.unwrap();
    let warriors = party(before);
    combat.enemies.records = monsters(before, "records");
    let cast = &case["casts"][0];
    let packet = bytes(&cast["packet"]);
    let mut actual = random(before);
    let rng = case["rng"].as_array().unwrap();
    for draw in &rng[..3] {
        assert_eq!(
            actual.bound(draw["bound"].as_i64().unwrap() as i16),
            draw["result"].as_i64().unwrap() as i16
        );
    }
    let selected = select_targets(
        &combat,
        &warriors,
        mission,
        &mission.areas,
        6,
        &packet,
        &mut actual,
    )
    .unwrap();
    assert!(selected.cast);
    assert_eq!(selected.count, 4);
    assert_eq!(
        selected
            .targets
            .iter()
            .flat_map(|target| [target.first, target.second, target.kind])
            .collect::<Vec<_>>(),
        words(&cast["targets"])
    );
    let resolution = &case["resolutions"][0];
    assert_eq!(actual.draws, resolution["draws_before"].as_u64().unwrap());
    let mut expected_random = random(before);
    for draw in &rng[..23] {
        assert_eq!(
            expected_random.bound(draw["bound"].as_i64().unwrap() as i16),
            draw["result"].as_i64().unwrap() as i16
        );
    }
    assert_eq!(actual, expected_random);
    combat.enemies.records[6] = selected.caster;
    assert_bytes(
        &combat
            .enemies
            .records
            .iter()
            .flat_map(|enemy| enemy.bytes.iter().copied())
            .collect::<Vec<_>>(),
        &bytes(&resolution["records_after"]),
        "complete charged monsters",
    );
    let result = resolve_bodies_with_item_cache(
        &warriors,
        &combat.enemies.records,
        &words(&resolution["mask"]),
        &DirectSpell {
            packet: &packet,
            level: 1,
            count: 4,
            armor_filter: false,
            weapon_sound: 402,
        },
        &bytes(&before["first_item_cache"]),
        &mut actual,
    )
    .unwrap();
    let expected = serde_json::json!({ "party": resolution["party_after"], "records": resolution["records_after"] });
    assert_records(&result, &expected, "records");
    assert_eq!(audio(&result), vec![430]);
    for draw in &rng[23..29] {
        assert_eq!(
            expected_random.bound(draw["bound"].as_i64().unwrap() as i16),
            draw["result"].as_i64().unwrap() as i16
        );
    }
    assert_eq!(actual, expected_random);
    for draw in &rng[29..] {
        assert_eq!(
            expected_random.bound(draw["bound"].as_i64().unwrap() as i16),
            draw["result"].as_i64().unwrap() as i16
        );
    }
    assert_eq!(expected_random, random(&case["after_cast"]));
    assert_eq!(random(&case["after_cast"]), random(&case["after_getup"]));
    assert_eq!(before["fields"], case["after_getup"]["fields"]);
    assert_eq!(before["references"], case["after_getup"]["references"]);
}

#[test]
fn unsupported_poison_packets_npc_targets_defenses_and_collisions_are_atomic() {
    let f = fixture();
    let before = &f["direct_cases"][0]["before"];
    let base_packet = bytes(&before["packet"]);
    let warriors = party(before);
    let enemies = monsters(before, "records");
    let base_mask = words(&before["mask"]);
    for (offset, value) in [
        (1, 0),
        (4, 8),
        (9, 3),
        (15, 2),
        (19, 0),
        (20, 8),
        (23, 1),
        (34, 1),
        (41, 5),
        (42, 1),
        (50, 0),
    ] {
        let mut packet = base_packet.clone();
        packet[offset] = value;
        let mut actual = random(before);
        let initial = actual.clone();
        let context = DirectSpell {
            packet: &packet,
            level: 1,
            count: 4,
            armor_filter: false,
            weapon_sound: 402,
        };
        assert!(resolve_bodies(&warriors, &enemies, &base_mask, &context, &mut actual).is_err());
        assert_eq!(actual, initial);
    }
    for count in [-1, 0, 8] {
        let mut actual = random(before);
        let initial = actual.clone();
        let context = DirectSpell {
            packet: &base_packet,
            level: 1,
            count,
            armor_filter: false,
            weapon_sound: 402,
        };
        assert!(resolve_bodies(&warriors, &enemies, &base_mask, &context, &mut actual).is_err());
        assert_eq!(actual, initial);
    }
    let context = DirectSpell {
        packet: &base_packet,
        level: 1,
        count: 4,
        armor_filter: false,
        weapon_sound: 402,
    };
    let mut mask = base_mask.clone();
    mask[10] = 1;
    let mut actual = random(before);
    let initial = actual.clone();
    assert!(matches!(
        resolve_bodies(&warriors, &enemies, &mask, &context, &mut actual),
        Err(RuleError::Unsupported { .. })
    ));
    assert_eq!(actual, initial);
    let mut usage = FieldUsage {
        slot: 8,
        used: bytes(&before["used"]),
    };
    let initial_usage = usage.used.clone();
    assert!(
        resolve_collision(
            &warriors,
            &enemies,
            &base_mask,
            &context,
            &mut usage,
            &mut actual
        )
        .is_err()
    );
    assert_eq!(actual, initial);
    assert_eq!(usage.used, initial_usage);
    let mut defended = warriors.clone();
    defended[4].bytes[0x9a..0x9c].copy_from_slice(&1_i16.to_be_bytes());
    assert!(resolve_bodies(&defended, &enemies, &base_mask, &context, &mut actual).is_err());
    assert_eq!(actual, initial);
    let mut npc = enemies[0].bytes.clone();
    let initial_npc = npc.clone();
    spell_conditions::apply(&mut npc, true, 4, 5, &mut actual).unwrap();
    let mut expected_npc = initial_npc;
    expected_npc[0xb5] = 5;
    assert_eq!(npc, expected_npc);
    assert_eq!(actual, initial);
}
