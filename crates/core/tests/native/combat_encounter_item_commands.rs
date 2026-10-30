use crate::{prelude::*, support};
use centurions_content::records::Condition;

// Declared source compositions use the old native before records, with a
// controlled typed branch. They do not extend those captures' qualification.
fn input(content: &mut Content, matched: bool, supplied: i16, quantity: i16) -> Session {
    content.items[700][20..22].copy_from_slice(&supplied.to_be_bytes());
    let table =
        &mut Arc::make_mut(&mut content.scenario.as_mut().unwrap().maps[0].shared).typed_encounters;
    let record = &mut table[8 * 516..9 * 516];
    record[..156].fill(0);
    record[136..138].copy_from_slice(&(if matched { 700_i16 } else { 840 }).to_be_bytes());
    record[146] = 1;
    record[152] = 1;
    for (branch, first, last) in [(0, 5_i16, 7_i16), (3, 6, 8)] {
        record[branch * 8..branch * 8 + 3].copy_from_slice(&[9, 1, 9]);
        for (slot, argument) in [first, 950, last].into_iter().enumerate() {
            let offset = 32 + branch * 16 + slot * 2;
            record[offset..offset + 2].copy_from_slice(&argument.to_be_bytes());
        }
    }
    let case = &support::fixture("encounter-medical.json")["cases"][0];
    let mut state = crate::command_support_psi::combat_state(content, case);
    state.warriors[4]
        .record_mut()
        .set_inventory_quantity(3, quantity);
    // This before record carries one five-unit needle in its weight of 170.
    state.warriors[4]
        .record_mut()
        .set_load(165 + quantity * 5);
    // Keep the selected actor independent from Items' explicit inventory owner.
    state.char_select_new = 1;
    state.globals.in_spell = 0;
    state.combat.as_mut().unwrap().in_spell = 0;
    state.globals.party_track.fill(1);
    state.combat.as_mut().unwrap().track_tail = vec![1; 104];
    state
}

#[test]
fn combat_typed_items_keep_charges_native_work_and_saved_branch_returns() {
    for (matched, supplied, quantity) in
        [(true, 1, 2), (false, 1, 1), (true, -1, 0), (false, -1, -2)]
    {
        let mut content = support::content();
        let mut state = input(&mut content, matched, supplied, quantity);
        let before = state.clone();
        let events = support::saved_command(
            Command::UseItem {
                warrior: 4,
                slot: 3,
            },
            &mut state,
            &content,
        );
        assert!(state.medical_continuation.is_none() && state.psi_selection.is_none());
        assert!(state.outdoor_psi.is_none());
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, Event::SpellResolved { .. }))
        );
        assert_eq!(state.random, before.random);
        assert_eq!(state.char_select_new, before.char_select_new);
        assert_eq!(
            state
                .globals
                .health_context
                .char_select_new,
            4
        );
        assert_eq!(current_combat(&state).in_spell, 1);
        assert_eq!(state.globals.attack_count, 1);
        assert_eq!(current_combat(&state).cast_identity, [0, 3, 0]);
        assert_eq!(
            current_combat(&state).spell_info,
            centurions_core::combat::spells::load_packet(
                &content.map(0).unwrap().combat.as_ref().unwrap().attacks,
                1401,
            )
            .unwrap()
        );
        assert_eq!(state.globals.party_track, [0, 0, 0, 0, 1, 0]);
        let tail = &state.combat.as_ref().unwrap().track_tail;
        assert!(tail.iter().all(|bit| *bit == 0));
        let inventory = state.warriors[4].inventory().unwrap();
        assert_eq!(
            inventory
                .iter()
                .find(|item| item.id() == 700)
                .map(|item| item.charge().unwrap()),
            if quantity == 1 {
                None
            } else {
                Some(if quantity > 0 { quantity - 1 } else { quantity })
            }
        );
        for (actual, original) in state.warriors.iter().zip(&before.warriors) {
            assert_eq!(actual.stamina().unwrap(), original.stamina().unwrap());
            assert_eq!(
                actual.record().action_points().unwrap(),
                original.record().action_points().unwrap()
            );
            assert_eq!(
                actual.record().psi_points().unwrap(),
                original.record().psi_points().unwrap()
            );
        }
        let first = if matched { 5 } else { 6 };
        let last = if matched { 7 } else { 8 };
        assert_eq!(
            support::event_sounds(&events),
            if quantity == 1 {
                vec![655, first]
            } else {
                vec![first]
            }
        );
        assert!(matches!(
            state.exploration.as_ref().unwrap().presentation,
            Some(Presentation::Dialogue { id: 950, .. })
        ));
        let events = support::saved_command(Command::Continue, &mut state, &content);
        assert_eq!(support::event_sounds(&events), [last]);
        assert!(state.exploration.as_ref().unwrap().script.is_none());
        assert!(state.exploration.as_ref().unwrap().presentation.is_none());
        support::checkpoint(&state, &content, "typed-item-returned");
    }

    let mut content = support::content();
    let mut state = input(&mut content, true, 1, 2);
    content.items[700][42..44].copy_from_slice(&(-8_i16).to_be_bytes());
    let attacks = &mut Arc::make_mut(&mut content.scenario.as_mut().unwrap().maps[0].shared)
        .combat
        .as_mut()
        .unwrap()
        .attacks;
    // 1401 is school0/tier3/column0 in the original 58-byte packet table.
    attacks[21 * 58 + 21] = 0;
    state.warriors[0]
        .record_mut()
        .set_condition(Condition::Helpless, 3);
    let before = support::checkpoint(&state, &content, "typed-item-refusal");
    let mut resumed = before.clone();
    let command = Command::UseItem {
        warrior: 4,
        slot: 3,
    };
    let error = apply(command.clone(), &mut state, &content).unwrap_err();
    let RuleError::RefusedAfterEffects { feedback } = error else {
        panic!("expected refusal after the native random-power prefix");
    };
    let RuleError::RefusedAfterEffects { feedback: restored } =
        apply(command, &mut resumed, &content).unwrap_err()
    else {
        panic!("restored command lost the native refusal prefix");
    };
    assert_eq!(feedback, restored);
    assert_eq!(
        feedback,
        vec![centurions_core::feedback::GameFeedback::Warning { index: 23 }]
    );
    support::same_session(&state, &resumed);
    let mut random = before.random.clone();
    let power = random.bound(7);
    assert_eq!(state.random, random);
    assert_eq!(state.globals.attack_count, power);
    assert_eq!(current_combat(&state).cast_identity, [0, 3, 0]);
    assert_eq!(
        current_combat(&state).spell_info,
        centurions_core::combat::spells::load_packet(
            &content.map(0).unwrap().combat.as_ref().unwrap().attacks,
            1401,
        )
        .unwrap()
    );
    assert_eq!(
        state.warriors[4].inventory().unwrap()[3]
            .charge()
            .unwrap(),
        2
    );
    assert_eq!(
        state.warriors[4].record().load().unwrap(),
        before.warriors[4].record().load().unwrap()
    );
    assert!(matches!(
        state.exploration.as_ref().unwrap().presentation,
        Some(Presentation::Typed { id: 8, .. })
    ));
    assert_eq!(current_combat(&state).in_spell, 0);

    // One source-composed ordinary variant covers the three former needle
    // whitelists without repeating the independent standard-needle cases.
    let mut content = support::content();
    let mut state = input(&mut content, true, 1, 2);
    for (offset, value) in [(36, 7_i16), (42, -2), (76, 2)] {
        content.items[700][offset..offset + 2].copy_from_slice(&value.to_be_bytes());
    }
    let mut item = state.warriors[4].record().inventory_slot(3);
    item.0[8] = 1;
    state.warriors[4].record_mut().set_inventory_slot(3, &item);
    // Two owned units now each weigh two more than the declared input above.
    let weight = state.warriors[4]
        .record()
        .load()
        .unwrap()
        .wrapping_add(4);
    state.warriors[4].record_mut().set_load(weight);
    let position = state.exploration.as_mut().unwrap();
    position.presentation = None;
    position.script = None;
    // Keep the captured inventory/current owner 4 distinct from selected caster
    // 1, with a declared adjacent target layout and observable caster movement.
    let combat = state.combat.as_mut().unwrap();
    for (member, point) in [(1, (20, 20)), (4, (21, 20))] {
        combat.squad.pos[member] = point;
        combat.squad.char_under[member] = 1155;
        let index = OpeningMission::index(i32::from(point.0) + 5, i32::from(point.1) + 5).unwrap();
        combat.field.tiles[index] = member as i16;
    }
    state.warriors[1].record_mut().set_movement(9);
    centurions_content::records::EquipmentStat::Equipment(15)
        .write(&mut state.warriors[1], 0)
        .unwrap();
    state.warriors[1].record_mut().set_toggle(true);
    state.warriors[4].record_mut().set_stamina(5);
    let before = state.clone();
    let events = support::saved_command(
        Command::UseItem {
            warrior: 4,
            slot: 3,
        },
        &mut state,
        &content,
    );
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::SpellResolved { .. }))
    );
    let pending = state.medical_continuation.as_ref().unwrap();
    assert_eq!(pending.origin, MedicalOrigin::OwnedItem);
    assert_eq!(pending.actor, 1);
    assert_eq!(state.char_select_new, 1);
    assert!(!state.warriors[1].record().toggle());
    assert_eq!((pending.item, pending.spell, pending.power), (700, 1401, 2));
    assert_eq!(current_combat(&state).spell_info[0], 2);
    assert_eq!(state.warriors[4].stamina().unwrap(), 5);
    assert_eq!(
        state.warriors[4].inventory().unwrap()[3]
            .charge()
            .unwrap(),
        1
    );
    assert_eq!(
        state.warriors[4].record().load().unwrap(),
        before.warriors[4]
            .record()
            .load()
            .unwrap()
            .wrapping_sub(7)
    );
    state = support::checkpoint(&state, &content, "ordinary-item-target");
    let (x, y) = current_combat(&state).squad.pos[4];
    let events = support::saved_command(
        Command::SelectMedicalTarget {
            selection: MedicalTarget::LivingPoint { x, y },
        },
        &mut state,
        &content,
    );
    assert!(state.medical_continuation.is_none());
    assert!(state.warriors[4].stamina().unwrap() > 5);
    assert!(events.iter().any(|event| matches!(event,
        Event::SpellResolved { caster: Some(1), target: 4, damage, .. } if *damage < 0)));
    assert_eq!(state.globals.define_spells, before.globals.define_spells);
    for member in [1, 4] {
        let actual = state.warriors[member].record();
        let original = before.warriors[member].record();
        assert_eq!(
            state.warriors[member].psi_points().unwrap(),
            before.warriors[member].psi_points().unwrap()
        );
        assert_eq!(
            actual.action_points().unwrap(),
            original
                .action_points()
                .unwrap()
                .wrapping_sub(if member == 1 { 2 } else { 0 })
        );
        assert_eq!(
            actual.movement().unwrap(),
            if member == 1 {
                6
            } else {
                original.movement().unwrap()
            }
        );
        // Item payment does not count a learned scan or credit party healing as
        // attack damage to either the selected caster or the inventory owner.
        assert_eq!(
            &state.warriors[member].bytes[0xa42..0xa76],
            &before.warriors[member].bytes[0xa42..0xa76]
        );
        assert_eq!(actual.spells_so_far().unwrap(), original.spells_so_far().unwrap());
    }

    for mode in [1_i16, 7, 8] {
        let global = mode == 7;
        let mut content = support::content();
        let mut state = input(&mut content, true, 1, 2);
        if mode != 1 {
            content.items[700][56..58].copy_from_slice(&mode.to_be_bytes());
        }
        if global {
            content.items[700][76..78].copy_from_slice(&0_i16.to_be_bytes());
        }
        if mode == 8 {
            content.items[700][60..62].copy_from_slice(&0_i16.to_be_bytes());
            content.items[0][60..62].copy_from_slice(&0_i16.to_be_bytes());
        }
        let attacks = &mut Arc::make_mut(&mut content.scenario.as_mut().unwrap().maps[0].shared)
            .combat
            .as_mut()
            .unwrap()
            .attacks;
        let packet = &mut attacks[21 * 58..22 * 58];
        packet[40..42].copy_from_slice(&(if mode == 1 { -57_i16 } else { 0 }).to_be_bytes());
        if mode == 8 {
            packet[2] = 0;
            packet[3] = 0;
        }
        let position = state.exploration.as_mut().unwrap();
        position.presentation = None;
        position.script = None;
        state.warriors[1].record_mut().set_movement(9);
        state.combat.as_mut().unwrap().blank_round = 9;
        state.globals.round_scripts.blank_round = Some(9);
        let current = state.warriors[4].record();
        let current_item = if current.toggle() {
            current.equipped_weapon().unwrap()
        } else {
            current.melee_weapon().unwrap()
        };
        let current_definition = content.items[current_item as usize].clone();
        let before = state.clone();
        support::saved_command(
            Command::UseItem {
                warrior: 4,
                slot: 3,
            },
            &mut state,
            &content,
        );
        assert!(state.medical_continuation.is_none());
        let pending = state.psi_selection.as_ref().unwrap();
        assert_eq!((pending.actor, pending.item), (1, Some(700)));
        assert_eq!(
            state.globals.loaded_items.item.as_ref().unwrap(),
            &current_definition
        );
        assert_eq!(current_combat(&state).blank_round, 9);
        for member in [1, 4] {
            assert_eq!(
                state.warriors[member].record().action_points().unwrap(),
                before.warriors[member].record().action_points().unwrap()
            );
            assert_eq!(
                state.warriors[member].record().movement().unwrap(),
                before.warriors[member].record().movement().unwrap()
            );
        }
        assert_eq!(
            state.warriors[4].inventory().unwrap()[3]
                .charge()
                .unwrap(),
            1
        );
        state = support::checkpoint(&state, &content, "item-signed-effect-wait");
        if global {
            assert_eq!(state.globals.attack_count, 0);
            let (x, y) = current_combat(&state).squad.pos[1];
            support::saved_command(
                Command::FirePreparedPsi { x, y, rotation: 0 },
                &mut state,
                &content,
            );
            assert!(state.psi_selection.is_none());
            assert_eq!(state.globals.attack_count, 1);
            assert_eq!(current_combat(&state).blank_round, 0);
            assert_eq!(
                state.warriors[1].record().action_points().unwrap(),
                before.warriors[1]
                    .record()
                    .action_points()
                    .unwrap()
                    .wrapping_sub(2)
            );
            assert_eq!(state.warriors[1].record().movement().unwrap(), 6);
            assert_eq!(
                state.globals.loaded_items.item.as_ref().unwrap(),
                &current_definition
            );
        } else if mode == 8 {
            assert_eq!(current_combat(&state).spell_info[2], 0);
            let events = support::saved_command(Command::CancelPsiSelection, &mut state, &content);
            assert_eq!(support::event_sounds(&events), [6000, 20005]);
            assert!(state.psi_selection.is_none());
            assert_eq!(current_combat(&state).in_spell, 0);
            assert_eq!(current_combat(&state).blank_round, 9);
            assert_eq!(state.warriors[1].record().movement().unwrap(), 6);
            assert_eq!(
                state.warriors[1].record().action_points().unwrap(),
                before.warriors[1].record().action_points().unwrap()
            );
            assert_eq!(
                state.warriors[4].inventory().unwrap()[3]
                    .charge()
                    .unwrap(),
                1
            );
        } else {
            let before_portrait = state.clone();
            assert!(
                matches!(apply(Command::FirePreparedPsiOnWarrior { warrior: 4 }, &mut state, &content),
                Err(RuleError::Refused { feedback }) if feedback == vec![centurions_core::feedback::GameFeedback::Sound { resource_id: 143 }])
            );
            support::same_session(&state, &before_portrait);
        }
    }
}
