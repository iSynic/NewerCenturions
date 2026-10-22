use crate::{native_common, prelude::*, support};

use centurions_core::items;

use native_common::snapshot_party as party;

fn records(snapshot: &Value, key: &str) -> Vec<InventoryRecord> {
    support::inventory(&bytes(&snapshot[key]))
}
use native_common::scalar_words as numbers;

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
        fixture["foundations"][case.n::<usize>("foundation")]["session"].clone(),
    )
    .unwrap();
    for (warrior, native) in result.warriors.iter_mut().zip(party(snapshot)) {
        warrior.bytes = native.bytes;
    }
    result.native_work.inventory_record = Some(InventoryRecord(
        snapshot.bytes("selected_record").try_into().unwrap(),
    ));
    result
}
use support::compare_party as assert_party;

fn assert_prepared(state: &Session, case: &Value, content: &Content) {
    let member = case.n::<usize>("receiver");
    let prepared = firearm::prepare(
        &state.warriors[member],
        &content.items[165],
        &content.items[280],
        &content.map(3).unwrap().combat.as_ref().unwrap().attacks,
    )
    .unwrap();
    support::native_expect! {
        bytes prepared.packet => &case["prepared"]["packet"], "binary_earned_crate_equipment";
        scalar prepared.slot => &case["equipped_slot"], "binary_earned_crate_equipment";
    }
    assert_party(&state.warriors, &case["prepared"]);
}

#[test]
fn all_thirteen_collection_cases_match_native_records_and_warning_order() {
    let fixture = support::fixture("earned-crate-equipment.json");
    let content = content();
    for case in fixture.array("cases").iter().take(13) {
        let member = case.n::<usize>("receiver");
        let slot = case.n::<usize>("slot");
        let initial = state(&fixture, case, &case["before"]);
        let crate_mode = case.n::<bool>("crate_mode");
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

        if !crate_mode {
            let mut actual = initial.clone();
            actual.exploration.as_mut().unwrap().presentation = Some(Presentation::Loot);
            let events = apply(
                Command::TakeLoot {
                    slot,
                    warrior: member,
                },
                &mut actual,
                &content,
            )
            .unwrap();
            assert_eq!(events, effects(&case["before"], &case["picked"]));
            assert_party(&actual.warriors, &case["picked"]);
            assert_eq!(
                actual.booty_items[slot],
                records(&case["picked"], "loot")[slot]
            );
        }
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
            expected.native_work.loaded_item_field32 =
                Some(case["closed"].n::<i16>("item_context"));
            expected.native_work.ammunition_work.item = Some(case["closed"].bytes("item_work"));
            expected.native_work.inventory_record = Some(InventoryRecord(
                case["closed"].bytes("selected_record").try_into().unwrap(),
            ));
            support::same_captured_session(&actual, &expected);
        }
    }
}

#[test]
fn outdoor_wear_replacement_compaction_and_enhanced_preparation_match_native() {
    let fixture = support::fixture("earned-crate-equipment.json");
    let content = content();
    for index in [5, 6, 9, 10] {
        let case = &fixture["cases"][index];
        let member = case.n::<usize>("receiver");
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

        if case["wear_return"] == 1 {
            items::compact(&mut actual.warriors[member]).unwrap();
            assert_party(&actual.warriors, &case["compacted"]);
            assert_prepared(&actual, case, &content);
            let mut public = initial.clone();
            let events = equipment::equip(&mut public, &content, member, slot).unwrap();
            assert_eq!(events, effects(&case["wear_before"], &case["worn"]));
            let mut expected = initial;
            expected.warriors[member] = actual.warriors[member].clone();
            expected.native_work.loaded_item_field32 =
                Some(case["compacted"].n::<i16>("item_context"));
            expected.native_work.ammunition_work.item = Some(case["compacted"].bytes("item_work"));
            expected.native_work.inventory_record = Some(InventoryRecord(
                case["worn"].bytes("selected_record").try_into().unwrap(),
            ));
            expected
                .native_work
                .health_context
                .as_mut()
                .unwrap()
                .affected[member] = 1;
            support::same_captured_session(&public, &expected);
            let markers = case["compacted"].bytes("markers");
            assert_eq!(
                i16::from_be_bytes(markers[member * 2..member * 2 + 2].try_into().unwrap()),
                1
            );
        } else {
            let mut public = initial.clone();
            let feedback = equipment::equip(&mut public, &content, member, slot).unwrap();
            assert_eq!(
                feedback,
                result
                    .sounds
                    .into_iter()
                    .map(|resource_id| Event::Sound { resource_id })
                    .chain(
                        result
                            .warnings
                            .into_iter()
                            .map(|warning| Event::CollectionRejected { warning })
                    )
                    .collect::<Vec<_>>()
            );
            assert_party(&public.warriors, &case["worn"]);
            assert_eq!(
                public.native_work.ammunition_work.item,
                Some(case["worn"].bytes("item_work"))
            );
            assert_eq!(public.random, initial.random);
        }
    }
}

#[test]
fn continuous_stack_firearm_acquisition_preserves_native_normalization_and_caches() {
    let fixture = support::fixture("earned-crate-equipment.json");
    let content = content();
    let case = &fixture["cases"][13];
    let mut actual = state(&fixture, case, &case["before"]);
    let mut expected = actual.clone();
    for (slot, picked, closed) in [
        (22, "needle_picked", "needle_closed"),
        (
            case.n::<usize>("weapon_crate_slot"),
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
        expected.native_work.loaded_item_field32 =
            Some(case[closed]["item_context"].as_i64().unwrap() as i16);
        expected.native_work.ammunition_work.item = Some(bytes(&case[closed]["item_work"]));
        expected.native_work.inventory_record = Some(InventoryRecord(
            case[closed].bytes("selected_record").try_into().unwrap(),
        ));
        support::same_captured_session(&actual, &expected);
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
    expected.native_work.loaded_item_field32 = Some(case["compacted"].n::<i16>("item_context"));
    expected.native_work.ammunition_work.item = Some(case["compacted"].bytes("item_work"));
    expected
        .native_work
        .health_context
        .as_mut()
        .unwrap()
        .affected[4] = 1;
    support::same_captured_session(&actual, &expected);
    assert_eq!(word(&actual.warriors[4].bytes, 0x9d6).unwrap(), 970);
    assert_prepared(&actual, case, &content);
}
