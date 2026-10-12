use crate::prelude::*;

use centurions_core::items;

fn fixture() -> Value {
    crate::support::fixture("owned-player-firearm.json")
}
use crate::native_common::snapshot_party as party;

use crate::native_common::snapshot_ammunition as stock;

use crate::native_common::snapshot_rng as random;

use crate::native_common::scalar_words as numbers;

fn added_sounds(before: &Value, after: &Value) -> Vec<i16> {
    numbers(&after["sounds"])[before.array("sounds").len()..].to_vec()
}
use crate::support::compare_party as assert_party;

fn work(snapshot: &Value) -> WeaponWork {
    let weapon = snapshot.bytes("weapon");
    let ammo = snapshot.bytes("ammo");
    WeaponWork {
        sound: Some(word(&weapon, 14).unwrap()),
        weapon_armor_filter: Some(word(&weapon, 136).unwrap()),
        ammunition_armor_filter: Some(word(&ammo, 136).unwrap()),
    }
}

fn state(fixture: &Value, snapshot: &Value) -> Session {
    let mut state: Session =
        serde_json::from_value(fixture["foundation"]["session"].clone()).unwrap();
    let records = party(snapshot);
    for (warrior, record) in state.warriors.iter_mut().zip(records) {
        warrior.bytes = record.bytes;
    }
    state.crate_items = crate::support::inventory(&snapshot.bytes("crate"));
    state.ammunition = stock(snapshot);
    state.random = random(snapshot);
    state.weapon_work = work(snapshot);
    state
}

fn compare_receipts(initial: &RandomState, after: &Value, before: &Value) {
    let mut expected = initial.clone();
    let draws = after.array("draws");
    for draw in &draws[before.array("draws").len()..] {
        crate::support::expect(expected.bound(draw.n::<i16>("bound")), &draw["result"]);
    }
    assert_eq!(expected, random(after));
}

#[test]
fn earned_pair_profiles_and_serial_collection_wear_fit_match_complete_native_records() {
    let f = fixture();
    let content = content();
    let mission = content.map(3).unwrap();
    for case in f.array("gate_cases") {
        let records = party(&case["before"]);
        assert_eq!(
            equipment::can_use(
                &records[case.n::<usize>("member")],
                &content.items[2],
                mission,
            )
            .unwrap(),
            case["allowed"] == 1
        );
        assert_party(&records, &case["after"]);
    }
    let case = &f["equipment_cases"][0];
    let mut state = state(&f, &case["before"]);
    for stage in case.array("equipment") {
        let member = stage.n::<usize>("member");
        let crate_slot = stage.n::<usize>("crate_slot");
        assert_party(&state.warriors, &stage["before"]);
        let events = equipment::take_crate(&mut state, &content, crate_slot, member).unwrap();
        assert_eq!(
            events,
            added_sounds(&stage["before"], &stage["picked"])
                .into_iter()
                .map(|resource_id| Event::Sound { resource_id })
                .collect::<Vec<_>>()
        );
        assert_party(&state.warriors, &stage["picked"]);
        let actual_crate = state
            .crate_items
            .iter()
            .flat_map(|item| item.0)
            .collect::<Vec<_>>();
        let expected_crate = stage["worn"].bytes("crate");
        assert!(
            actual_crate == expected_crate,
            "crate differences {:?}",
            actual_crate
                .iter()
                .zip(&expected_crate)
                .enumerate()
                .filter(|(_, (a, b))| a != b)
                .take(10)
                .collect::<Vec<_>>()
        );
        let slot = state.warriors[member].inventory().unwrap().len() - 1;
        let worn =
            equipment::wear(&state.warriors[member], &content, mission, slot, false).unwrap();
        assert!(worn.changed);
        assert_eq!(worn.sounds, added_sounds(&stage["picked"], &stage["worn"]));
        assert!(worn.warnings.is_empty());
        state.warriors[member] = worn.warrior;
        assert_party(&state.warriors, &stage["worn"]);
        items::compact(&mut state.warriors[member]).unwrap();
        assert_party(&state.warriors, &stage["compacted"]);
        let initial = state.random.clone();
        let fitted = firearm::fit_ammunition(
            &state.warriors[member],
            stage.n::<usize>("inventory_slot"),
            &content,
            &state.ammunition,
            false,
            &mut state.random,
        )
        .unwrap();
        assert!(fitted.loaded);
        assert_eq!(fitted.loaded_item_field32, 2);
        assert_eq!(
            fitted.sounds,
            added_sounds(&stage["compacted"], &stage["fitted"])
        );
        state.warriors[member] = fitted.warrior;
        state.ammunition = fitted.ammunition;
        state
            .weapon_work
            .load_ammunition(&content.items[201])
            .unwrap();
        assert_party(&state.warriors, &stage["fitted"]);
        assert_eq!(state.ammunition, stock(&stage["fitted"]));
        assert_eq!(state.random, random(&stage["fitted"]));
        assert_eq!(state.weapon_work, work(&stage["fitted"]));
        compare_receipts(&initial, &stage["fitted"], &stage["compacted"]);
    }
    assert_party(&state.warriors, &case["after"]);
    assert_eq!(state.ammunition[1], 264);
}

#[test]
fn complete_preparation_and_four_controlled_resolutions_match_native_packet_records_and_rng() {
    let f = fixture();
    let content = content();
    let attacks = &content.map(3).unwrap().combat.as_ref().unwrap().attacks;
    assert_eq!(word(&content.items[201], 60).unwrap(), -1);
    for case in f.array("fire_cases") {
        let member = case.n::<usize>("member");
        let mut records = party(&case["prepared"]);
        let prepared = firearm::prepare(
            &records[member],
            &content.items[2],
            &content.items[201],
            attacks,
        )
        .unwrap();
        assert_eq!(prepared.packet, case["prepared"].bytes("packet"));
        assert_eq!(
            (
                prepared.slot,
                prepared.shots,
                prepared.rounds,
                prepared.weight,
                prepared.sound,
                prepared.armor_filter
            ),
            (0, 1, 1, 0, 402, false)
        );
        let mut prepared_work = work(&case["combat_before"]);
        prepared_work
            .prepare(&content.items[2], &content.items[201])
            .unwrap();
        assert_eq!(prepared_work.sound, Some(402));
        assert!(!prepared_work.armor_filter().unwrap());
        assert_party(&records, &case["prepared"]);
        // The pure resolver begins after the caller's independently captured debits.
        records = party(&case["targeted"]);
        records[member].bytes[0x40..0x42].copy_from_slice(&1_i16.to_be_bytes());
        records[member].bytes[0x3c..0x3e].copy_from_slice(&2_i16.to_be_bytes());
        let mut target = MonsterRecord {
            bytes: case["targeted"].bytes("target"),
        };
        let mut random = random(&case["targeted"]);
        let initial = random.clone();
        let result =
            firearm::resolve(&mut records[member], &mut target, &prepared, &mut random, 0).unwrap();
        assert_party(&records, &case["fired"]);
        assert_eq!(target.bytes, case["fired"].bytes("target"));
        assert_eq!(random, self::random(&case["fired"]));
        compare_receipts(&initial, &case["fired"], &case["targeted"]);
        assert!(!result.killed);
        assert_eq!(result.damage, 10000 - word(&target.bytes, 0x110).unwrap());
        let helper_audio = case["fired"]
            .array("native_calls")
            .iter()
            .filter(|call| call["name"] == "Sound" && call["return"] == "151d88")
            .map(|call| call["arguments"][0].as_i64().unwrap() as i16)
            .collect::<Vec<_>>();
        assert_eq!(result.sounds, helper_audio);
    }
}

#[test]
fn manual_full_partial_cancel_and_combat_fills_match_complete_native_records() {
    let f = fixture();
    let content = content();
    for case in f.array("ammunition_cases") {
        let kind = case.text("kind");
        if !kind.starts_with("manual_") && kind != "combat_fit_and_advance" {
            continue;
        }
        let mut records = party(&case["before"]);
        let mut random = random(&case["before"]);
        let initial = random.clone();
        let after = case.get("fitted").unwrap_or(&case["after"]);
        let fitted = firearm::fit_ammunition(
            &records[2],
            0,
            &content,
            &stock(&case["before"]),
            kind == "combat_fit_and_advance",
            &mut random,
        )
        .unwrap();
        assert_eq!(fitted.loaded_item_field32, 2);
        assert_eq!(
            fitted.loaded,
            !matches!(kind, "manual_missing_201" | "manual_wrong_caliber")
        );
        records[2] = fitted.warrior;
        assert_party(&records, after);
        assert_eq!(fitted.ammunition, stock(after));
        assert_eq!(fitted.sounds, added_sounds(&case["before"], after));
        assert_eq!(random, self::random(after));
        compare_receipts(&initial, after, &case["before"]);
    }
}

#[test]
fn independent_silent_reload_full_partial_and_wrong_caliber_match_native_records() {
    let f = fixture();
    let mut compared = 0;
    for case in f.array("ammunition_cases") {
        let kind = case.text("kind");
        if !kind.starts_with("reload_") {
            continue;
        }
        // These controls call DoReload14edd0 directly, even with combat=0.
        // Its nested Refit111 is distinct from the outdoor Reload command.
        let mut records = party(&case["before"]);
        let mut reserve = stock(&case["before"]);
        let mut random = random(&case["before"]);
        let initial = random.clone();
        let (complete, sounds) =
            firearm::reload(&mut records[2], &mut reserve[1], true, &mut random).unwrap();
        assert_party(&records, &case["after"]);
        assert_eq!(reserve, stock(&case["after"]));
        assert_eq!(random, self::random(&case["after"]));
        compare_receipts(&initial, &case["after"], &case["before"]);
        assert_eq!(complete, kind == "reload_empty");
        assert_eq!(
            if complete { vec![] } else { vec![25] },
            numbers(&case["after"]["warnings"])[numbers(&case["before"]["warnings"]).len()..]
        );
        assert!(sounds.is_empty());
        assert!(added_sounds(&case["before"], &case["after"]).is_empty());
        compared += 1;
    }
    assert_eq!(compared, 3);
}

#[test]
fn pickup_rejections_and_uncertified_instances_are_atomic() {
    let f = fixture();
    let content = content();
    let mission = content.map(3).unwrap();
    for case in f
        .array("admission_cases")
        .iter()
        .filter(|case| case.text("kind").starts_with("pickup_"))
    {
        let mut records = party(&case["before"]);
        let item = InventoryRecord(case["before"].bytes("crate")[66..88].try_into().unwrap());
        let result = equipment::collect_item(&records[2], &item, &content, mission).unwrap();
        assert!(!result.collected);
        records[2] = result.warrior;
        assert_party(&records, &case["after"]);
        assert_eq!(result.item, item);
        assert_eq!(result.sounds, added_sounds(&case["before"], &case["after"]));
        assert_eq!(result.warnings, numbers(&case["after"]["warnings"]));
    }
    let base = &f["equipment_cases"][0]["equipment"][0];
    for (offset, value) in [(13, 2), (14, 1), (15, 1), (18, 1), (20, 1)] {
        let mut item = InventoryRecord(base["before"].bytes("crate")[66..88].try_into().unwrap());
        item.0[offset] = value;
        let warrior = party(&base["before"])[2].clone();
        let before = warrior.clone();
        assert!(equipment::collect_item(&warrior, &item, &content, mission).is_err());
        assert_eq!(warrior, before);
    }
    let after = &base["fitted"];
    let attacks = &mission.combat.as_ref().unwrap().attacks;
    for (offset, value) in [
        (4, 1),
        (5, 219),
        (6, 1),
        (7, 13),
        (14, 1),
        (15, 1),
        (18, 1),
        (20, 1),
    ] {
        let mut warrior = party(after)[2].clone();
        warrior.bytes[0x132 + offset] = value;
        let before = warrior.clone();
        let mut rng = random(after);
        let before_rng = rng.clone();
        assert!(
            firearm::fit_ammunition(&warrior, 0, &content, &stock(after), false, &mut rng).is_err()
        );
        assert_eq!(warrior, before);
        assert_eq!(rng, before_rng);
        assert!(
            firearm::prepare(&warrior, &content.items[2], &content.items[201], attacks).is_err()
        );
    }
    let empty = f
        .array("ammunition_cases")
        .iter()
        .find(|case| case["kind"] == "prepare_empty")
        .unwrap();
    let warrior = party(&empty["before"])[2].clone();
    assert!(firearm::prepare(&warrior, &content.items[2], &content.items[201], attacks).is_err());
    assert_party(&party(&empty["before"]), &empty["after"]);
    assert_eq!(random(&empty["before"]), random(&empty["after"]));
    let mut loaded = party(after)[2].clone();
    let initial = loaded.clone();
    let mut enhanced = loaded.clone();
    enhanced.bytes[0x132 + 14] = 1;
    let initial_enhanced = enhanced.clone();
    assert!(equipment::remove(&enhanced, &content, mission, 0).is_err());
    assert_eq!(enhanced, initial_enhanced);
    assert_eq!(loaded, initial);
    let mut reserve = 276;
    let initial_reserve = reserve;
    let mut rng = random(after);
    let initial_rng = rng.clone();
    assert!(firearm::reload(&mut loaded, &mut reserve, true, &mut rng).is_err());
    assert_eq!(loaded, initial);
    assert_eq!(reserve, initial_reserve);
    assert_eq!(rng, initial_rng);
}
