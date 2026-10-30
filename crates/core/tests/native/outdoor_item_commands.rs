//! Selected Items source compositions, separate from the captured UI boundaries.
use crate::{prelude::*, support};
use centurions_content::records::{Condition, InventorySlotsMut};
use centurions_core::combat::globals::BattleOrder;

fn input(content: &Content) -> Session {
    let case = &support::fixture("outdoor-medical.json")["cases"][0];
    let mut state = super::command_support_psi::outdoor(content, case);
    state.char_select_new = 1;
    state.warriors[2]
        .record_mut()
        .set_condition(Condition::Helpless, 7);
    state.warriors[4].record_mut().set_load(321);
    state.warriors[4].record_mut().set_inventory_quantity(3, 2);
    state.globals.in_spell = 9;
    state.globals.party_track.fill(1);
    state.globals.track_tail = vec![1; 104];
    state.globals.battle_order = BattleOrder {
        order: case["before"].words("order"),
        index: case["before"].n("cursor"),
    };
    state.globals.weapon_traits = crate::native_common::psi_work(&case["before"]);
    state
        .globals
        .health_context
        .marks_affected = true;
    state.scenario_duration_tail = Some(0);
    state
}

#[test]
fn outdoor_selected_items_keep_prefix_effects_and_saved_typed_returns() {
    let content = support::content();
    let mut state = input(&content);
    let mut item = state.warriors[4].record().inventory_slot(3);
    item.0[..2].copy_from_slice(&704_i16.to_be_bytes());
    state.warriors[4].record_mut().set_inventory_slot(3, &item);
    let before = state.clone();
    let events = support::saved_command(
        Command::UseItem {
            warrior: 4,
            slot: 3,
        },
        &mut state,
        &content,
    );
    assert!(state.outdoor_psi.is_none());
    assert_eq!(state.char_select_new, 1);
    assert_eq!(
        state.warriors[2]
            .record()
            .condition(Condition::Helpless)
            .unwrap(),
        7
    );
    assert_eq!(state.warriors[4].record().load().unwrap(), 319);
    assert_eq!(
        state.globals.battle_order,
        before.globals.battle_order
    );
    assert_eq!(state.globals.define_spells, before.globals.define_spells);
    assert_eq!(state.holdover, before.holdover);
    assert_eq!(state.held_over, before.held_over);
    assert_eq!(
        state.globals.monsters,
        before.globals.monsters
    );
    assert_eq!(state.globals.in_spell, 0);
    assert!(
        state
            .globals
            .health_context
            .marks_affected
    );
    assert_eq!(state.globals.party_track, [1; 6]);
    assert!(
        state
            .globals
            .track_tail
            .iter()
            .all(|value| *value == 0)
    );
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::SpellResolved { .. }))
    );

    for quantity in [0, -1, -2] {
        let mut content = support::content();
        content.items[700][20..22].copy_from_slice(&(-1_i16).to_be_bytes());
        let mut state = input(&content);
        state.warriors[4]
            .record_mut()
            .set_inventory_quantity(3, quantity);
        state.globals.inventory_record = state.warriors[4].record().inventory_slot(0);
        state.globals.character_bare_hand_bonus = -91;
        let before = state.clone();
        support::saved_command(
            Command::UseItem {
                warrior: 4,
                slot: 3,
            },
            &mut state,
            &content,
        );
        assert_eq!(state.outdoor_psi.as_ref().unwrap().actor, 4);
        assert_eq!(state.char_select_new, 1);
        assert_eq!(state.globals.in_spell, 9);
        assert_eq!(
            state.warriors[4]
                .record()
                .inventory_slot(3)
                .charge()
                .unwrap(),
            quantity
        );
        assert_eq!(state.warriors[4].record().load().unwrap(), 321);
        assert_eq!(state.warriors, before.warriors);
        let expected_record = if quantity == -1 {
            state.warriors[4].record().inventory_slot(3)
        } else {
            before.globals.inventory_record.clone()
        };
        assert_eq!(state.globals.inventory_record, expected_record);
        assert_eq!(
            state.globals.character_bare_hand_bonus,
            if quantity == -1 {
                state.warriors[4].record().bare_hand_bonus().unwrap()
            } else {
                -91
            }
        );
        support::saved_command(
            Command::SelectOutdoorPsiTarget { warrior: 0 },
            &mut state,
            &content,
        );
        assert!(state.outdoor_psi.is_none());
        assert_eq!(state.globals.in_spell, 0);
        assert_eq!(
            state.warriors[4]
                .record()
                .inventory_slot(3)
                .charge()
                .unwrap(),
            quantity
        );
        assert_eq!(state.warriors[4].record().load().unwrap(), 321);
        assert_eq!(state.char_select_new, 1);
        assert_eq!(
            state.globals.battle_order,
            before.globals.battle_order
        );
    }

    let mut content = support::content();
    content.items[700][42..44].copy_from_slice(&(-8_i16).to_be_bytes());
    let packet = &mut Arc::make_mut(&mut content.scenario.as_mut().unwrap().maps[0].shared)
        .combat
        .as_mut()
        .unwrap()
        .attacks[3 * 406..3 * 406 + 58];
    packet[22] = 0;
    let packet = packet.to_vec();
    let mut state = input(&content);
    let before = state.clone();
    let mut random = state.random.clone();
    let count = random.bound(7);
    let mut restored = support::checkpoint(&state, &content, "outdoor-item-refusal");
    for state in [&mut state, &mut restored] {
        assert!(
            matches!(apply(Command::UseItem { warrior: 4, slot: 3 }, state, &content),
            Err(RuleError::RefusedAfterEffects { feedback }) if feedback.is_empty())
        );
        assert_eq!(state.random, random);
        assert_eq!(state.globals.attack_count, count);
        assert_eq!(state.globals.spell_info, packet);
        assert_eq!(state.globals.cast_identity, [0, 3, 0]);
        assert_eq!(state.warriors, before.warriors);
        assert_eq!(state.globals.in_spell, 9);
        assert_eq!(state.globals.party_track, before.globals.party_track);
        assert!(state.outdoor_psi.is_none());
    }
    assert!(state == restored);

    let mut content = support::content();
    let table =
        &mut Arc::make_mut(&mut content.scenario.as_mut().unwrap().maps[0].shared).typed_encounters;
    let record = &mut table[8 * 516..9 * 516];
    record[..156].fill(0);
    record[136..138].copy_from_slice(&700_i16.to_be_bytes());
    record[146] = 1;
    record[152] = 1;
    record[..3].copy_from_slice(&[9, 1, 9]);
    for (slot, argument) in [5_i16, 950, 7].into_iter().enumerate() {
        record[32 + slot * 2..34 + slot * 2].copy_from_slice(&argument.to_be_bytes());
    }
    let mut state = input(&content);
    state.warriors[4].record_mut().set_inventory_quantity(3, 1);
    let following = state.warriors[4].record().inventory_slot(2);
    state.warriors[4]
        .record_mut()
        .set_inventory_slot(4, &following);
    InventorySlotsMut(&mut state.warriors[4].bytes).set_count(5);
    state.globals.character_bare_hand_bonus = -91;
    state.globals.health_context.affected = [11, 12, 13, 14, 15, 16];
    let mut script = vec![0; 42];
    script[8..10].copy_from_slice(&5_i16.to_be_bytes());
    script[24..26].copy_from_slice(&8_i16.to_be_bytes());
    let position = state.exploration.as_mut().unwrap();
    position.script = Some(ScriptState::scheduled(script, 0));
    position.presentation = Some(Presentation::Dialogue {
        id: 0,
        text: String::new(),
    });
    support::saved_command(Command::Continue, &mut state, &content);
    assert!(matches!(
        state.exploration.as_ref().unwrap().presentation,
        Some(Presentation::Typed { id: 8, .. })
    ));
    let before = state.clone();
    let first = support::saved_command(
        Command::UseItem {
            warrior: 4,
            slot: 3,
        },
        &mut state,
        &content,
    );
    assert!(support::event_sounds(&first).contains(&655));
    assert_eq!(state.outdoor_psi.as_ref().unwrap().item, Some(700));
    assert_eq!(state.globals.in_spell, 9);
    assert_eq!(state.char_select_new, 1);
    assert_eq!(state.warriors[4].record().inventory_slot(3), following);
    assert_eq!(state.globals.inventory_record, following);
    assert_eq!(
        state.globals.character_bare_hand_bonus,
        state.warriors[4].record().bare_hand_bonus().unwrap()
    );
    assert_eq!(state.warriors[4].record().load().unwrap(), 316);
    assert_eq!(state.globals.define_spells, before.globals.define_spells);
    let events = support::saved_command(
        Command::SelectOutdoorPsiTarget { warrior: 0 },
        &mut state,
        &content,
    );
    assert!(state.outdoor_psi.is_none());
    assert_eq!(state.globals.in_spell, 9);
    assert_eq!(state.globals.party_track, [1, 0, 0, 0, 1, 0]);
    assert!(
        state
            .globals
            .track_tail
            .iter()
            .all(|value| *value == 0)
    );
    let health = &state.globals.health_context;
    assert!(!health.marks_affected);
    assert_eq!(health.affected, [0; 6]);
    assert!(support::event_sounds(&events).contains(&5));
    assert!(matches!(
        state.exploration.as_ref().unwrap().presentation,
        Some(Presentation::Dialogue { id: 950, .. })
    ));
    let events = support::saved_command(Command::Continue, &mut state, &content);
    assert_eq!(support::event_sounds(&events), [7]);
    assert!(state.exploration.as_ref().unwrap().script.is_none());
    assert!(state.exploration.as_ref().unwrap().presentation.is_none());

}
