//! Source-derived scripted effects and prepared status-weapon command routes.
use crate::{prelude::*, support};
use centurions_core::feedback::GameFeedback::Warning;

#[test]
fn scripted_save_selection_uses_live_ready() {
    let mut content = content();
    let tables = Arc::make_mut(&mut content.scenario.as_mut().unwrap().maps[0].shared);
    tables.combat.as_mut().unwrap().encounters[..10]
        .copy_from_slice(&support::word_bytes(&[5, 0, 0, 0, 0]));
    for (in_spell, mode) in [(0, 3_i8), (1, 3), (-1, 3), (1, -1)] {
        let mut state = support::mission_session(&content);
        let mut combat = support::battle();
        combat.in_spell = in_spell;
        combat.spell_info[9] = mode as u8;
        state.combat = Some(combat);
        state.phase = Phase::Combat;
        state.globals.in_spell = 99;
        for warrior in &mut state.warriors {
            centurions_content::records::DevelopmentField::Resistance(0)
                .write(&mut warrior.bytes, 100);
        }
        let record = support::script_record(&[(52, 0)]);
        support::pause_script(&mut state, record, 0);
        let before = state.clone();
        let skip = mode >= 2 && in_spell != 0;
        let mut random = state.random.clone();
        if !skip {
            for _ in &state.warriors {
                random.bound(100);
            }
        }
        support::saved_command(Command::Continue, &mut state, &content);
        assert_eq!(state.globals.party_track, [i16::from(skip); 6]);
        assert_eq!(state.random, random);
        assert_eq!(state.warriors, before.warriors);
        assert_eq!(state.combat.as_ref().unwrap().in_spell, in_spell);
    }
}

#[test]
fn scripted_party_condition_keeps_mask_rng_and_saved_return() {
    let mut content = content();
    let tables = Arc::make_mut(&mut content.scenario.as_mut().unwrap().maps[0].shared);
    for (offset, value) in [(0, 1201_i16), (2, 2), (6, 1)] {
        tables.combat.as_mut().unwrap().encounters[offset..offset + 2]
            .copy_from_slice(&value.to_be_bytes());
    }
    let mut state = support::mission_session(&content);
    let mut expected_random = state.random.clone();
    let duration = expected_random.range(2, 3);
    let previous: Vec<_> = state
        .warriors
        .iter()
        .map(|w| w.record().condition_duration(14).unwrap())
        .collect();
    let record = support::script_record(&[(18, 0), (1, 1), (9, 611)]);
    support::pause_script(&mut state, record, 0);
    support::saved_command(Command::Continue, &mut state, &content);
    for (warrior, before) in state.warriors.iter().zip(previous) {
        assert_eq!(
            warrior.record().condition_duration(14).unwrap(),
            before + duration
        );
    }
    assert_eq!(state.random, expected_random);
    let events = support::saved_command(Command::Continue, &mut state, &content);
    assert_eq!(support::event_sounds(&events), [611]);
    assert!(state.exploration.as_ref().unwrap().script.is_none());
}

#[test]
fn player_and_monster_status_weapons_use_shared_effects_and_saved_commands() {
    for (role, mon) in (0..4).map(|role| (role, role != 0)) {
        let mut content = content();
        let mut state = crate::native_common::loaded_shot_frame(0);
        let (weapon, command) = if mon {
            // This authored caller includes Getup's selector and shared-WORD writes.
            crate::native_common::monster_command_input(&mut state, 0, [0, 0]);
            let combat = state.combat.as_mut().unwrap();
            combat.battle_order_index =
                combat.battle_order.iter().position(|id| *id == 10).unwrap();
            let record = &mut combat.roster.records[0];
            record.record_mut().set_action_points(6);
            record.record_mut().set_movement(0);
            record.record_mut().set_psi_points(100);
            record.bytes[0x48..0x4a].copy_from_slice(&100_i16.to_be_bytes());
            if role >= 2 {
                record.record_mut().set_faction(0);
                record.bytes[0x26..0x28].copy_from_slice(&1_i16.to_be_bytes());
                record.bytes[0x48..0x4a].copy_from_slice(&0_i16.to_be_bytes());
                record.bytes[0x42..0x44].copy_from_slice(&100_i16.to_be_bytes());
                record.bytes[0x7e] = 1;
                for slot in 0..9 {
                    record.bytes[0x12 + slot * 2..0x14 + slot * 2]
                        .copy_from_slice(&2206_i16.to_be_bytes());
                }
                state.globals.loaded_item_field32 = 0;
                state.enemy_spell_disabled = u8::from(role == 3);
            }
            (
                record.record().primary_weapon().unwrap() as usize,
                Command::RunEnemyTurn,
            )
        } else {
            state.warriors[4].record_mut().set_action_points(6);
            let command = Command::SelectCombatWeapon { slot: 3 };
            support::saved_command(command, &mut state, &content);
            (1, Command::FireSelectedTargets)
        };
        for (offset, value) in [(44, 2206_i16), (56, 1), (60, 30), (76, 1 + 7 * !mon as i16)] {
            content.items[weapon][offset..offset + 2].copy_from_slice(&value.to_be_bytes());
        }
        let map = &mut content.scenario.as_mut().unwrap().maps
            [state.exploration.as_ref().unwrap().map as usize];
        let packet = &mut Arc::make_mut(&mut map.shared)
            .combat
            .as_mut()
            .unwrap()
            .attacks[2842 + 406 + 5 * 58..2842 + 406 + 6 * 58];
        packet[9] = 3;
        if role >= 2 {
            packet[1] = 12;
        }
        if role == 0 {
            (packet[1], packet[23]) = (1, 3);
        }
        if role == 1 {
            packet[40..42].copy_from_slice(&57_i16.to_be_bytes());
        }
        let loaded = |s: &Session| s.warriors[4].record().inventory_quantity(0).unwrap();
        let before = loaded(&state);
        if !mon {
            // 118544..1185d2: below two action points the fire button only warns 0x11.
            let (saved, warn) = (state.clone(), Warning { index: 0x11 });
            state.warriors[4].record_mut().set_action_points(1);
            let before = state.clone();
            let refused = apply(Command::BeginFireTargets, &mut state, &content);
            assert!(matches!(refused, Err(RuleError::Refused { feedback }) if feedback == [warn]));
            assert_eq!(std::mem::replace(&mut state, saved), before);
            // 118944..118958 draws Rand(7) for WORD4435cc and keeps WORD4435be 8; a
            // byte-1 packet then lists targets (11b2f6) for the fire key (11ae08).
            let mut rng = state.random.clone();
            support::saved_command(Command::BeginFireTargets, &mut state, &content);
            let (q, r) = (state.target_quantity.clone().unwrap(), state.random.clone());
            assert_eq!((q.power, q.num_of_targets(), r), (rng.bound(7), 8, rng));
            support::saved_command(Command::ShootEnemy { body: 10 }, &mut state, &content);
        }
        let events = support::saved_command(command, &mut state, &content);
        let resolved = |e: &Event| matches!(e, Event::SpellResolved { spell: 2206, .. });
        assert_eq!(events.iter().any(resolved), role != 3);
        if role == 3 {
            continue;
        }
        if role < 2 {
            assert!(events.iter().any(|event| matches!(event, Event::SpellResolved { damage, .. } if if mon { *damage < 0 } else { *damage > 0 })), "role {role}: {events:?}");
        }
        if role >= 2 {
            assert!(state.warriors.iter().any(|w| {
                w.record()
                    .condition(centurions_content::records::Condition::Poisoned)
                    .unwrap()
                    > 0
            }));
        } else if !mon {
            assert!(
                state.combat.as_ref().unwrap().roster.records[0]
                    .record()
                    .condition(centurions_content::records::Condition::Poisoned)
                    > 0
            );
            assert_eq!(state.warriors[4].record().action_points().unwrap(), 3);
            assert_eq!(loaded(&state), before - 1);
        }
    }
}

#[test]
fn type_24_flight_draws_before_the_shot_resolves_and_its_ray_resolves_nothing() {
    for mode in [1, 6] {
        let mut content = content();
        let item = &mut content.items[1];
        (item[55], item[57], item[61]) = (24, mode, 30);
        let mut state = crate::native_common::loaded_shot_frame(0);
        let select = Command::SelectCombatWeapon { slot: 3 };
        support::saved_command(select, &mut state, &content);
        let mut random = state.random.clone();
        let jitter: [i8; 16] = std::array::from_fn(|_| (random.bound(10) - 5) as i8);
        let events = support::saved_command(Command::ShootEnemy { body: 10 }, &mut state, &content);
        let flight = events.iter().find_map(|event| match event {
            Event::AttackResolved { flight, .. } | Event::SpellResolved { flight, .. } => flight.0,
            _ => None,
        });
        let expected = (mode == 1).then_some((24, jitter));
        assert_eq!(flight.map(|f| (f.kind, f.jitter)), expected);
        assert!(mode == 1 || state.random == random);
    }
}
