use centurions_content::{Content, InventoryRecord, Warrior, combat::MonsterRecord, word};
use centurions_core::{Event, Session, combat::firearm, equipment, items};
use serde_json::Value;

fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../evidence/fixtures/earned-crate-equipment.json"
    ))
    .unwrap()
}

fn content() -> Content {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap()
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

fn records(snapshot: &Value, key: &str) -> Vec<InventoryRecord> {
    bytes(&snapshot[key])
        .as_chunks::<22>()
        .0
        .iter()
        .map(|record| InventoryRecord(*record))
        .collect()
}

fn numbers(value: &Value) -> Vec<i16> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_i64().unwrap() as i16)
        .collect()
}

fn added(before: &Value, after: &Value, key: &str) -> Vec<i16> {
    numbers(&after[key])[numbers(&before[key]).len()..].to_vec()
}

fn effects(before: &Value, after: &Value) -> Vec<Event> {
    added(before, after, "sounds")
        .into_iter()
        .map(|resource_id| Event::Sound { resource_id })
        .chain(
            added(before, after, "warnings")
                .into_iter()
                .map(|warning| Event::CollectionRejected { warning }),
        )
        .collect()
}

fn state(fixture: &Value, case: &Value, snapshot: &Value) -> Session {
    let mut result: Session = serde_json::from_value(
        fixture["foundations"][case["foundation"].as_u64().unwrap() as usize]["session"].clone(),
    )
    .unwrap();
    for (warrior, native) in result.warriors.iter_mut().zip(party(snapshot)) {
        warrior.bytes = native.bytes;
    }
    result
}

fn assert_party(actual: &[Warrior], snapshot: &Value) {
    for (member, (actual, expected)) in actual.iter().zip(party(snapshot)).enumerate() {
        if let Some(offset) = actual
            .bytes
            .iter()
            .zip(&expected.bytes)
            .position(|(a, b)| a != b)
        {
            panic!(
                "member {member} byte {offset:x}: {} != {}",
                actual.bytes[offset], expected.bytes[offset]
            );
        }
    }
}

fn assert_retained(before: &Value, after: &Value) {
    for key in [
        "ammunition",
        "party_buffs",
        "memory",
        "selected",
        "powers",
        "flags",
        "references",
        "used",
        "all_records",
        "terrain",
        "target",
        "target_count",
        "target_descriptors",
        "mask",
        "rng_seed",
        "rng_draws",
        "rng_sites",
        "draws",
        "active_ui_member",
        "displayed_member",
        "combat",
        "learned_marker",
        "heal_marks_affected",
        "health_last_body",
        "unavailable",
        "map_id",
        "day",
        "hour",
        "minute",
        "fatigue",
        "currency",
    ] {
        assert_eq!(before[key], after[key], "retained native {key}");
    }
}

fn assert_prepared(state: &Session, case: &Value, content: &Content) {
    let member = case["receiver"].as_u64().unwrap() as usize;
    let before = state.clone();
    let prepared = firearm::prepare(
        &state.warriors[member],
        &content.items[165],
        &content.items[280],
        &content.map(3).unwrap().combat.as_ref().unwrap().attacks,
    )
    .unwrap();
    assert_eq!(prepared.packet, bytes(&case["prepared"]["packet"]));
    assert_eq!(
        prepared.slot,
        case["equipped_slot"].as_u64().unwrap() as usize
    );
    assert_eq!(prepared.shots, 1);
    assert_eq!(prepared.rounds, 1);
    assert_eq!(prepared.weight, 0);
    assert_eq!(prepared.sound, 406);
    assert!(!prepared.armor_filter);
    assert_eq!(*state, before);
    assert_retained(&case["compacted"], &case["prepared"]);
    assert_party(&state.warriors, &case["prepared"]);
    assert_eq!(case["prepared"]["identity"], "000300000000");
    assert_eq!(case["prepared"]["power"], 1);
}

fn assert_session(actual: &Session, expected: &Session) {
    fn compare(actual: &Value, expected: &Value, path: &str) {
        if actual == expected {
            return;
        }
        match (actual, expected) {
            (Value::Object(a), Value::Object(b)) => {
                for (key, value) in a {
                    compare(value, &b[key], &format!("{path}.{key}"));
                }
            }
            (Value::Array(a), Value::Array(b)) => {
                assert_eq!(a.len(), b.len(), "{path} length");
                for (index, (a, b)) in a.iter().zip(b).enumerate() {
                    compare(a, b, &format!("{path}[{index}]"));
                }
            }
            _ => panic!("{path}: {actual} != {expected}"),
        }
    }
    compare(
        &serde_json::to_value(actual).unwrap(),
        &serde_json::to_value(expected).unwrap(),
        "Session",
    );
}

#[test]
fn all_thirteen_collection_cases_match_native_records_and_warning_order() {
    let fixture = fixture();
    let content = content();
    for case in fixture["cases"].as_array().unwrap().iter().take(13) {
        let member = case["receiver"].as_u64().unwrap() as usize;
        let slot = case["slot"].as_u64().unwrap() as usize;
        let initial = state(&fixture, case, &case["before"]);
        let crate_mode = case["crate_mode"].as_bool().unwrap();
        let item = if crate_mode {
            &initial.crate_items[slot]
        } else {
            &initial.booty_items[slot]
        };
        let result = equipment::collect_item(
            &initial.warriors[member],
            item,
            &content,
            content.map(3).unwrap(),
        )
        .unwrap();
        assert_eq!(result.collected, case["collected"] == 1, "{}", case["kind"]);
        assert_eq!(
            result.sounds,
            added(&case["before"], &case["picked"], "sounds")
        );
        assert_eq!(
            result.warnings,
            added(&case["before"], &case["picked"], "warnings")
        );
        let mut actual = initial.clone();
        actual.warriors[member] = result.warrior;
        assert_party(&actual.warriors, &case["picked"]);
        assert_eq!(result.item, records(&case["picked"], "loot")[slot]);
        assert_retained(&case["before"], &case["after"]);
        assert_eq!(case["before"]["packet"], case["picked"]["packet"]);
        assert_eq!(case["before"]["markers"], case["picked"]["markers"]);
        if crate_mode {
            let mut actual = initial.clone();
            let events = equipment::take_crate(&mut actual, &content, slot, member).unwrap();
            assert_eq!(events, effects(&case["before"], &case["picked"]));
            assert_party(&actual.warriors, &case["picked"]);
            let mut expected = initial;
            for (warrior, native) in expected.warriors.iter_mut().zip(party(&case["picked"])) {
                warrior.bytes = native.bytes;
            }
            expected.crate_items = records(&case["closed"], "crate");
            expected.loaded_item_field32 =
                Some(case["closed"]["item_context"].as_i64().unwrap() as i16);
            expected.ammunition_work.item = Some(bytes(&case["closed"]["item_work"]));
            assert_session(&actual, &expected);
        }
    }
}

#[test]
fn outdoor_wear_replacement_compaction_and_enhanced_preparation_match_native() {
    let fixture = fixture();
    let content = content();
    for index in [5, 6, 9, 10] {
        let case = &fixture["cases"][index];
        let member = case["receiver"].as_u64().unwrap() as usize;
        let initial = state(&fixture, case, &case["wear_before"]);
        let slot = initial.warriors[member]
            .inventory()
            .unwrap()
            .iter()
            .position(|item| item.id() == 165)
            .unwrap();
        let result = equipment::wear(
            &initial.warriors[member],
            &content,
            content.map(3).unwrap(),
            slot,
            false,
        )
        .unwrap();
        assert_eq!(result.changed, case["wear_return"] == 1);
        let mut expected_sounds = added(&case["wear_before"], &case["worn"], "sounds");
        if result.warnings.contains(&29) {
            // This historical probe returned at warn before its Sound6000 call.
            expected_sounds.push(6000);
        }
        assert_eq!(result.sounds, expected_sounds);
        assert_eq!(
            result.warnings,
            added(&case["wear_before"], &case["worn"], "warnings")
        );
        let mut actual = initial.clone();
        actual.warriors[member] = result.warrior;
        assert_party(&actual.warriors, &case["worn"]);
        assert_retained(&case["wear_before"], &case["after"]);
        if case["wear_return"] == 1 {
            items::compact(&mut actual.warriors[member]).unwrap();
            assert_party(&actual.warriors, &case["compacted"]);
            assert_prepared(&actual, case, &content);
            let mut public = initial.clone();
            let events = equipment::equip(&mut public, &content, member, slot).unwrap();
            assert_eq!(events, effects(&case["wear_before"], &case["worn"]));
            let mut expected = initial;
            expected.warriors[member] = actual.warriors[member].clone();
            expected.loaded_item_field32 =
                Some(case["compacted"]["item_context"].as_i64().unwrap() as i16);
            expected.ammunition_work.item = Some(bytes(&case["compacted"]["item_work"]));
            expected.health_context.as_mut().unwrap().affected[member] = 1;
            assert_session(&public, &expected);
            let markers = bytes(&case["compacted"]["markers"]);
            assert_eq!(
                i16::from_be_bytes(markers[member * 2..member * 2 + 2].try_into().unwrap()),
                1
            );
        } else {
            let mut public = initial.clone();
            assert!(equipment::equip(&mut public, &content, member, slot).is_err());
            assert_eq!(public, initial);
        }
    }
}

#[test]
fn continuous_stack_firearm_acquisition_preserves_native_normalization_and_caches() {
    let fixture = fixture();
    let content = content();
    let case = &fixture["cases"][13];
    let mut actual = state(&fixture, case, &case["before"]);
    let mut expected = actual.clone();
    for (slot, picked, closed) in [
        (22, "needle_picked", "needle_closed"),
        (
            case["weapon_crate_slot"].as_u64().unwrap() as usize,
            "weapon_picked",
            "weapon_closed",
        ),
    ] {
        let before = if picked == "needle_picked" {
            &case["before"]
        } else {
            &case["needle_closed"]
        };
        let events = equipment::take_crate(&mut actual, &content, slot, 4).unwrap();
        assert_eq!(events, effects(before, &case[picked]));
        assert_party(&actual.warriors, &case[picked]);
        for (warrior, native) in expected.warriors.iter_mut().zip(party(&case[picked])) {
            warrior.bytes = native.bytes;
        }
        expected.crate_items = records(&case[closed], "crate");
        expected.loaded_item_field32 = Some(case[closed]["item_context"].as_i64().unwrap() as i16);
        expected.ammunition_work.item = Some(bytes(&case[closed]["item_work"]));
        assert_session(&actual, &expected);
    }
    let slot = actual.warriors[4]
        .inventory()
        .unwrap()
        .iter()
        .position(|item| item.id() == 165)
        .unwrap();
    assert_eq!(
        equipment::equip(&mut actual, &content, 4, slot).unwrap(),
        effects(&case["wear_before"], &case["worn"])
    );
    for (warrior, native) in expected.warriors.iter_mut().zip(party(&case["compacted"])) {
        warrior.bytes = native.bytes;
    }
    expected.loaded_item_field32 = Some(case["compacted"]["item_context"].as_i64().unwrap() as i16);
    expected.ammunition_work.item = Some(bytes(&case["compacted"]["item_work"]));
    expected.health_context.as_mut().unwrap().affected[4] = 1;
    assert_session(&actual, &expected);
    assert_eq!(word(&actual.warriors[4].bytes, 0x9d6).unwrap(), 970);
    assert_prepared(&actual, case, &content);
    assert_retained(&case["before"], &case["after"]);
}

#[test]
fn unverified_firearm_operations_and_other_instances_preserve_state() {
    let fixture = fixture();
    let content = content();
    let case = &fixture["cases"][6];
    let initial = state(&fixture, case, &case["compacted"]);
    let member = 4;
    let slot = 0;
    let mut invalid_removal = initial.warriors[member].clone();
    invalid_removal.bytes[0x132 + 19] = 1;
    assert!(equipment::remove(&invalid_removal, &content, content.map(3).unwrap(), slot).is_err());
    let unequipped = state(&fixture, case, &case["closed"]);
    let wear_slot = unequipped.warriors[member]
        .inventory()
        .unwrap()
        .iter()
        .position(|item| item.id() == 165)
        .unwrap();
    let mut invalid_wear = unequipped.warriors[member].clone();
    invalid_wear.bytes[0x132 + wear_slot * 22 + 19] = 1;
    assert!(
        equipment::wear(
            &invalid_wear,
            &content,
            content.map(3).unwrap(),
            wear_slot,
            true
        )
        .is_err()
    );
    for (level, bonus) in [(1, 0), (2, 10), (3, 0), (3, 11), (4, 10)] {
        let mut changed = unequipped.warriors[member].clone();
        changed.bytes[0x132 + wear_slot * 22 + 13] = level;
        changed.bytes[0x132 + wear_slot * 22 + 18] = bonus;
        let item = changed.inventory().unwrap()[wear_slot].clone();
        assert!(
            equipment::collect_item(&changed, &item, &content, content.map(3).unwrap()).is_err()
        );
    }
    let mut random = initial.random.clone();
    let mut warrior = initial.warriors[member].clone();
    let mut reserve = initial.ammunition[80];
    assert!(firearm::reload(&mut warrior, &mut reserve, false, &mut random).is_err());
    assert_eq!(warrior, initial.warriors[member]);
    assert_eq!(reserve, initial.ammunition[80]);
    assert!(
        firearm::fit_ammunition(
            &warrior,
            slot,
            &content,
            &initial.ammunition,
            false,
            &mut random
        )
        .is_err()
    );
    let prepared = firearm::prepare(
        &warrior,
        &content.items[165],
        &content.items[280],
        &content.map(3).unwrap().combat.as_ref().unwrap().attacks,
    )
    .unwrap();
    let mut enemy = MonsterRecord::read(&vec![0; 288], 0).unwrap();
    let before_enemy = enemy.clone();
    assert!(firearm::resolve(&mut warrior, &mut enemy, &prepared, &mut random, 0).is_err());
    assert_eq!(enemy, before_enemy);
    assert_eq!(warrior, initial.warriors[member]);
    assert_eq!(random, initial.random);
    let mut public = initial.clone();
    public.warriors[member].bytes[0x132 + 19] = 1;
    let before_public = public.clone();
    assert!(equipment::unequip(&mut public, &content, member, slot).is_err());
    assert_eq!(public, before_public);
}
