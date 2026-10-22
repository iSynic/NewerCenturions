use crate::{native_common, prelude::*, support};

use centurions_core::items;

use native_common::snapshot_party as party;

use native_common::snapshot_ammunition as stock;

use native_common::snapshot_rng as random;

use native_common::scalar_words as numbers;

fn added(before: &Value, after: &Value, field: &str) -> Vec<i16> {
    numbers(&after[field])[before[field].as_array().unwrap().len()..].to_vec()
}
use support::assert_bytes;
use support::compare_party as assert_party;

fn slot(warrior: &Warrior, id: i16) -> usize {
    warrior
        .inventory()
        .unwrap()
        .iter()
        .position(|item| item.id() == id)
        .unwrap()
}

fn assert_equipment(
    result: equipment::EquipmentResult,
    warriors: &mut [Warrior],
    before: &Value,
    after: &Value,
    changed: bool,
) {
    assert_eq!(result.changed, changed);
    assert_eq!(result.sounds, added(before, after, "sounds"));
    assert_eq!(result.warnings, added(before, after, "warnings"));
    let mut markers = before.bytes("markers");
    if changed {
        markers[6..8].copy_from_slice(&1_i16.to_be_bytes());
    }
    support::native_expect! {
        bytes &markers => &after.bytes("markers"), "equipment changed marker";
    };
    warriors[3] = result.warrior;
    assert_party(warriors, after);
    assert_eq!(random(before), random(after));
    assert_eq!(stock(before), stock(after));
}

fn collect(
    warriors: &mut [Warrior],
    crate_items: &mut Vec<InventoryRecord>,
    id: i16,
    content: &Content,
    before: &Value,
    after: &Value,
) {
    let source = crate_items.iter().position(|item| item.id() == id).unwrap();
    let result = equipment::collect_item(
        &warriors[3],
        &crate_items[source],
        content,
        content.map(3).unwrap(),
    )
    .unwrap();
    assert!(result.collected);
    assert_eq!(result.sounds, added(before, after, "sounds"));
    assert_eq!(result.warnings, added(before, after, "warnings"));
    warriors[3] = result.warrior;
    crate_items[source] = result.item;
    let length = crate_items.len();
    crate_items.retain(|item| item.id() != 0);
    crate_items.resize(length, InventoryRecord([0; 22]));
    assert_party(warriors, after);
    let actual: Vec<_> = crate_items.iter().flat_map(|record| record.0).collect();
    assert_bytes(&actual, &after.bytes("crate"), "complete closed crate");
    assert_eq!(random(before), random(after));
    assert_eq!(stock(before), stock(after));
}

#[test]
fn profile_gates_and_manual_and_automatic_equipment_match_native_records() {
    let f = support::fixture("personal-shield.json");
    let content = content();
    let mission = content.map(3).unwrap();

    for case in f.array("gate_cases") {
        let warriors = party(&case["before"]);
        assert_eq!(
            equipment::can_use(
                &warriors[case.n::<usize>("member")],
                &content.items[case.n::<usize>("item_id")],
                mission
            )
            .unwrap(),
            case["allowed"] == 1
        );
        assert_party(&warriors, &case["after"]);
    }
    enum Step {
        Collect(i16),
        Wear(i16, bool),
        Remove(i16, bool),
        Refit,
        Compact,
    }
    for case in f.array("equipment_cases") {
        let mut warriors = party(&case["before"]);
        let mut crate_items = support::inventory(&case["before"].bytes("crate"));
        let mut before = &case["before"];
        for (action, snapshot) in [
            (Step::Collect(600), "shield_picked"),
            (Step::Wear(600, false), "empty_shield_worn"),
            (Step::Collect(637), "battery_picked"),
            (Step::Wear(637, true), "battery_worn"),
            (Step::Refit, "refitted"),
            (Step::Wear(600, true), "worn"),
            (Step::Compact, "compacted"),
            (Step::Remove(637, false), "battery_remove_rejected"),
            (Step::Remove(600, true), "shield_removed"),
            (Step::Remove(637, true), "battery_removed"),
            (Step::Wear(637, true), "battery_reworn"),
            (Step::Wear(600, true), "after"),
            (Step::Compact, "after"),
        ] {
            let after = &case[snapshot];
            match action {
                Step::Collect(id) => {
                    collect(&mut warriors, &mut crate_items, id, &content, before, after)
                }
                Step::Wear(id, changed) | Step::Remove(id, changed) => {
                    let slot = slot(&warriors[3], id);
                    let result = if matches!(action, Step::Wear(..)) {
                        equipment::wear(&warriors[3], &content, mission, slot, false)
                    } else {
                        equipment::remove(&warriors[3], &content, mission, slot)
                    }
                    .unwrap();
                    assert_equipment(result, &mut warriors, before, after, changed);
                }
                Step::Refit => {
                    let mut actual = random(before);
                    if case["kind"] == "automatic" {
                        use centurions_core::ammunition::WorkBuffers;
                        let work = |snapshot: &Value| WorkBuffers {
                            item: Some(snapshot.bytes("item_work")),
                            ammo: Some(snapshot.bytes("ammo")),
                            weapon: Some(snapshot.bytes("weapon")),
                        };
                        let foundation = usize::from(case["source"] == 1);
                        let mut state: Session =
                            serde_json::from_value(f["foundations"][foundation]["session"].clone())
                                .unwrap();
                        state.warriors = warriors.clone();
                        state.ammunition = stock(before);
                        state.native_work.ammunition_work = work(before);
                        state.native_work.inventory_record = Some(InventoryRecord(
                            before.bytes("selected_record").try_into().unwrap(),
                        ));
                        state.random = actual;
                        let mut resumed = support::checkpoint(&state, &content, "shield-refit");
                        let events = support::replay_command(
                            Command::Reload { warrior: 3 },
                            &mut state,
                            &mut resumed,
                            &content,
                        );
                        let mut sounds = added(before, after, "sounds");
                        // This capture stubs warn; ammunition-management executes its sound.
                        sounds.push(6000);
                        let expected: Vec<_> = sounds
                            .into_iter()
                            .map(|resource_id| Event::Sound { resource_id })
                            .chain(
                                added(before, after, "warnings")
                                    .into_iter()
                                    .map(|warning| Event::CollectionRejected { warning }),
                            )
                            .collect();
                        assert_eq!(events, expected);
                        assert_eq!(state.ammunition, stock(after));
                        assert_eq!(state.native_work.ammunition_work, work(after));
                        assert_eq!(state.random, random(after));
                        warriors = state.warriors.clone();
                        assert_party(&warriors, after);
                        support::checkpoint(&state, &content, "shield-refitted");
                        before = after;
                        continue;
                    }
                    let result = firearm::fit_ammunition(
                        &warriors[3],
                        slot(&warriors[3], 637),
                        &content,
                        &stock(before),
                        false,
                        &mut actual,
                    )
                    .unwrap();
                    assert!(result.loaded);
                    assert_eq!(result.loaded_item_field32, 24);
                    assert_eq!(result.sounds, added(before, after, "sounds"));
                    warriors[3] = result.warrior;
                    assert_party(&warriors, after);
                    assert_eq!(result.ammunition, stock(after));
                    assert_eq!(actual, random(after));
                }
                Step::Compact => {
                    items::compact(&mut warriors[3]).unwrap();
                    assert_party(&warriors, after);
                }
            }
            before = after;
        }
    }
}

#[test]
fn five_native_manual_controls_match_full_records_stock_rng_audio_and_charge_gate() {
    let f = support::fixture("personal-shield.json");
    let content = content();
    let mission = content.map(3).unwrap();

    for case in f.array("manual_controls") {
        let mut warriors = party(&case["before"]);
        let mut actual = random(&case["before"]);
        let result = firearm::fit_ammunition(
            &warriors[3],
            slot(&warriors[3], 637),
            &content,
            &stock(&case["before"]),
            false,
            &mut actual,
        )
        .unwrap();
        assert_eq!(result.loaded, case["reserve307"] != 0);
        assert_eq!(result.loaded_item_field32, 24);
        assert_eq!(
            result.sounds,
            added(&case["before"], &case["charged"], "sounds")
        );
        warriors[3] = result.warrior;
        assert_party(&warriors, &case["charged"]);
        assert_eq!(result.ammunition, stock(&case["charged"]));
        assert_eq!(actual, random(&case["charged"]));

        let before_wear;
        if case["repeat"] == true {
            let repeat = firearm::fit_ammunition(
                &warriors[3],
                slot(&warriors[3], 637),
                &content,
                &stock(&case["charged"]),
                false,
                &mut actual,
            )
            .unwrap();
            assert!(!repeat.loaded);
            assert_eq!(repeat.sounds, vec![404]);
            assert_eq!(repeat.warrior.bytes, warriors[3].bytes);
            assert_eq!(repeat.ammunition, stock(&case["charged"]));
            assert_eq!(actual, random(&case["after"]));

            before_wear = case["charged"].clone();
            let result = equipment::wear(
                &warriors[3],
                &content,
                mission,
                slot(&warriors[3], 600),
                false,
            )
            .unwrap();
            assert!(result.changed);
            assert_eq!(result.sounds, vec![231]);
            assert!(result.warnings.is_empty());
            warriors[3] = result.warrior;
            assert_party(&warriors, &case["after"]);
        } else {
            before_wear = case["charged"].clone();
            let result = equipment::wear(
                &warriors[3],
                &content,
                mission,
                slot(&warriors[3], 600),
                false,
            )
            .unwrap();
            assert_equipment(
                result,
                &mut warriors,
                &before_wear,
                &case["after"],
                case["shield_wear_return"] == 1,
            );
        }

        assert_eq!(stock(&before_wear), stock(&case["after"]));
    }
}
