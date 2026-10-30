use crate::{prelude::*, snapshots, support};

use support::compare_record_bytes as assert_party;
use support::warriors as party;

use support::same_session as assert_session;

fn install(state: &mut Session, snapshot: &Value) {
    for (warrior, native) in state.warriors.iter_mut().zip(party(&snapshot["party"])) {
        warrior.bytes = native.bytes;
    }
    state.random = crate::native_common::random_at(snapshot, "seed", "draws");
    let combat = state.combat.as_mut().unwrap();
    snapshots::records(combat, &|key| snapshot.bytes(key));
    snapshots::geometry(combat, &|key| snapshot.bytes(key), (0, 0));
    combat.battle_order_index = snapshot.n::<usize>("cursor");
    combat.combat_round = snapshot.n::<i16>("round");
    if let Some(counter) = snapshot.get("round_counter") {
        combat.round_macros.counter = counter.as_i64().unwrap() as i8;
        combat.blank_round = snapshot.n("inactive_rounds");
    }
    combat.round_macros.slots = snapshot.words("round_slots").try_into().unwrap();
    combat.kill_party = snapshot.n::<i8>("unavailable");
    snapshots::spell_work(combat, &|key| snapshot.bytes(key));

    combat
        .weapon_traits
        .load_pair(&snapshot.bytes("weapon_work"), &snapshot.bytes("ammo_work"))
        .unwrap();
}

fn state(fixture: &Value, case: &Value, pane: &str) -> Session {
    let mut state = serde_json::from_value(
        fixture["foundations"][case.n::<usize>("foundation")]["session"].clone(),
    )
    .unwrap();
    install(&mut state, &case[pane]);
    crate::native_common::source_victory_context(&mut state);
    state
}
use support::event_sounds as sounds;

fn pending(case: &Value) -> MedicalContinuation {
    let item = case
        .get("item")
        .map_or(700, |id| id.as_i64().unwrap() as i16);
    MedicalContinuation {
        origin: MedicalOrigin::OwnedNeedle,
        actor: 4,
        item,
        spell: 1401,
        power: 1,
    }
}

#[test]
fn native_owned_needle_preparations_match_complete_party_records() {
    let content = support::content();
    let fixture: Value = support::fixture("earned-combat-medical.json");
    let powers = support::fixture("medical-item-powers.json");
    let low_ap = support::fixture("medical-low-ap.json");
    for (index, case) in fixture.array("cases")[..6]
        .iter()
        .chain(powers.array("cases"))
        .chain(low_ap.array("cases"))
        .enumerate()
    {
        let mut state = state(&fixture, case, "pane_opened");
        let events = apply(
            Command::UseItem {
                warrior: 4,
                slot: 3,
            },
            &mut state,
            &content,
        )
        .unwrap();
        let warriors = &state.warriors;
        let combat = current_combat(&state);
        assert_party(warriors, &party(&case["prepared"]["party"]));
        assert_eq!(
            combat.spell_info,
            case["prepared"].bytes("packet")
        );
        assert_eq!(
            combat.cast_identity,
            case["prepared"].words("identity")[..3]
        );
        support::expect(state.globals.attack_count, &case["prepared"]["count"]);
        assert_eq!(
            state.globals.loaded_item_field32,
            word(&case["prepared"].bytes("item_work"), 32).unwrap()
        );
        let mut expected_work = current_combat(&state).weapon_traits;
        expected_work
            .load_pair(
                &case["prepared"].bytes("weapon_work"),
                &case["prepared"].bytes("ammo_work"),
            )
            .unwrap();
        assert_eq!(combat.weapon_traits, expected_work);
        assert_eq!(sounds(&events), if index == 0 { vec![] } else { vec![655] });
        if case["low_ap"] == true {
            let mut resumed = support::checkpoint(&state, &content, "one-ap-medical");
            let point = current_combat(&state).squad.pos[0];
            let events = apply(
                Command::SelectMedicalTarget {
                    selection: MedicalTarget::LivingPoint {
                        x: point.0,
                        y: point.1,
                    },
                },
                &mut resumed,
                &content,
            )
            .unwrap();
            let mut expected = state.clone();
            crate::command_learned_portrait_heal::project(&mut expected, &case["after"]);
            expected.medical_continuation = None;
            expected.globals.loaded_items = centurions_core::ammunition::LoadedItems {
                item: Some(case["after"].bytes("item_work")),
                weapon: Some(case["after"].bytes("weapon_work")),
                ammo: Some(case["after"].bytes("ammo_work")),
            };
            support::same_captured_session(&resumed, &expected);
            assert_eq!(sounds(&events), case.n::<Vec<i16>>("use_sounds")[1..]);
        }
    }
}

#[test]
fn native_living_heals_and_cancel_match_complete_medical_rule_state() {
    let content = support::content();
    let fixture: Value = support::fixture("earned-combat-medical.json");
    let powers = support::fixture("medical-item-powers.json");
    for (index, case) in fixture.array("cases")[..5]
        .iter()
        .chain(powers.array("cases"))
        .enumerate()
    {
        let mut state = state(&fixture, case, "prepared");
        let before = state.clone();
        let target = if index == 4 {
            MedicalTarget::ConfirmedCancel
        } else {
            let member = case.n::<usize>("target");
            let position = current_combat(&state).squad.pos[member];
            MedicalTarget::LivingPoint {
                x: position.0,
                y: position.1,
            }
        };
        let result = medical::resolve(
            state.combat.as_mut().unwrap(),
            &mut state.warriors,
            content
                .map(state.exploration.as_ref().unwrap().map)
                .unwrap(),
            &mut state.random,
            &pending(case),
            &target,
        )
        .unwrap();
        let mut expected = before.clone();
        install(&mut expected, &case["after"]);
        support::same_captured_session(&state, &expected);
        let mut native_random = before.random;
        support::replay_rng(&mut native_random, case.array("rng"));
        assert_eq!(state.random, native_random);
        let native_sounds: Vec<i16> = case
            .array("use_sounds")
            .iter()
            .map(|v| v.as_i64().unwrap() as i16)
            .filter(|sound| *sound != 655)
            .collect();
        assert_eq!(sounds(&result.events), native_sounds);
        assert_eq!(
            result.follow_up,
            if index == 4 {
                medical::FollowUp::None
            } else {
                medical::FollowUp::FinishPlayerAction
            }
        );
    }
}

// Derived command regressions; the modified layouts are not native receipts.
#[test]
fn medical_selection_survives_pane_compaction_and_natural_required_levels() {
    use centurions_content::records::{EquipmentStat, InventorySlotsMut, ItemDefinition};
    let content = support::content();
    let fixture = support::fixture("earned-combat-medical.json");
    for combat in [true, false] {
        for (id, duplicate, slot) in [
            (700, false, 0),
            (701, false, 0),
            (701, true, 0),
            (701, true, 1),
        ] {
            let mut state = state(&fixture, &fixture["cases"][0], "pane_opened");
            state.movement_continuation = None;
            state.exploration.as_mut().unwrap().script = None;
            if !combat {
                state.phase = Phase::Exploration;
                state.combat = None;
            }
            let mut needle = ItemDefinition(&content.items[id]).fresh_stock().unwrap();
            assert_eq!(needle.0[13], if id == 701 { 3 } else { 1 });
            if duplicate {
                needle.set_charge(2);
            }
            let mut shield = ItemDefinition(&content.items[637]).fresh_stock().unwrap();
            shield.set_equipped(1);
            let second = if duplicate {
                needle.clone()
            } else {
                ItemDefinition(&content.items[1]).fresh_stock().unwrap()
            };
            let ration = ItemDefinition(&content.items[800]).fresh_stock().unwrap();
            let actor = &mut state.warriors[4];
            for (index, item) in [needle, second, shield, ration].iter().enumerate() {
                actor.record_mut().set_inventory_slot(index, item);
            }
            InventorySlotsMut(&mut actor.bytes).set_count(4);
            for (kind, item) in [(15, 0), (2, 0), (20, 637)] {
                EquipmentStat::Equipment(kind).write(actor, item).unwrap();
            }
            actor.record_mut().set_toggle(false);
            let random = state.random.clone();
            apply(Command::UseItem { warrior: 4, slot }, &mut state, &content).unwrap();
            let items = state.warriors[4].inventory().unwrap();
            let actual: Vec<_> = items
                .iter()
                .map(|item| (item.id(), item.charge().unwrap()))
                .collect();
            if duplicate {
                // The selected Items action consumes its literal row.
                let consumed_row = slot;
                assert_eq!(
                    actual[..2],
                    [
                        (701, if consumed_row == 0 { 1 } else { 2 }),
                        (701, if consumed_row == 1 { 1 } else { 2 })
                    ]
                );
                assert_eq!(items[2].id(), 637);
            } else {
                assert_eq!(
                    items.iter().map(InventoryRecord::id).collect::<Vec<_>>(),
                    [1, 637, 800]
                );
            }
            assert_eq!(state.random, random);
            let selected = if combat {
                state
                    .medical_continuation
                    .as_ref()
                    .map(|pending| pending.item)
            } else {
                state.outdoor_psi.as_ref().and_then(|pending| pending.item)
            };
            assert_eq!(selected, Some(id as i16));
        }
    }
}

#[test]
fn native_range_and_admission_failures_are_atomic() {
    let content = support::content();
    let fixture: Value = support::fixture("earned-combat-medical.json");
    for case in &fixture.array("cases")[6..] {
        let state = state(&fixture, case, "pane_opened");
        let before = state.clone();
        assert!(
            matches!(medical::begin(&state, &content, medical::BeginRequest { warrior: 4, slot: 3 }),
            Err(RuleError::Refused { feedback }) if feedback.is_empty())
        );
        assert_session(&state, &before);
    }
    let case = &fixture["cases"][5];
    let mut state = state(&fixture, case, "prepared");
    let before = state.clone();
    let position = current_combat(&state).squad.pos[case.n::<usize>("target")];
    assert!(matches!(
        medical::resolve(
            state.combat.as_mut().unwrap(),
            &mut state.warriors,
            content
                .map(state.exploration.as_ref().unwrap().map)
                .unwrap(),
            &mut state.random,
            &pending(case),
            &MedicalTarget::LivingPoint {
                x: position.0,
                y: position.1
            }
        ),
        Err(RuleError::Refused { feedback }) if feedback == vec![centurions_core::feedback::GameFeedback::Combat {
            kind: centurions_core::feedback::CombatRefusal::OutOfRange,
        }]
    ));
    assert_session(&state, &before);
}
