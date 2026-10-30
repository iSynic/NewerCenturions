//! Source-derived encounter work and saved round-return transitions.
use crate::{prelude::*, support};

fn start(state: &mut Session, content: &Content, typed: bool) {
    let record = support::script_record(&[(if typed { 5 } else { 4 }, 0)]);
    support::pause_script(state, record, 0);
    support::saved_command(Command::Continue, state, content);
}

#[test]
fn replacement_redraw_keeps_shared_labels_and_executes_the_loaded_header_after_reload() {
    for typed in [false, true] {
        let mut content = content();
        let tables = Arc::make_mut(&mut content.scenario.as_mut().unwrap().maps[0].shared);
        let (stride, header, table) = if typed {
            (516, 156, &mut tables.typed_encounters)
        } else {
            (426, 106, &mut tables.menus)
        };
        let mut base = table[if typed { stride..2 * stride } else { 0..stride }].to_vec();
        base[..96].fill(0);
        base[0..2].copy_from_slice(&[7, 1]);
        base[34..36].copy_from_slice(&1_i16.to_be_bytes());
        base[if typed { 97 } else { 96 }] = 1;
        base[if typed { 152 } else { 101 }] = 5;
        table[..stride].copy_from_slice(&base);
        table[stride..2 * stride].copy_from_slice(&base);
        let text = if typed {
            centurions_core::typed_encounter::password_label(&base).unwrap()
        } else {
            &[]
        };
        let select = if typed {
            Command::SubmitText {
                text: String::from_utf8(text.to_vec()).unwrap(),
            }
        } else {
            Command::SelectMenuEntry { index: 0 }
        };
        for (offset, value) in [(0, if typed { -2_i16 } else { -1 }), (2, 1), (4, 1)] {
            tables.combat.as_mut().unwrap().encounters[offset..offset + 2]
                .copy_from_slice(&value.to_be_bytes());
        }
        let replacement = &mut tables.door_scripts[42..84];
        replacement.copy_from_slice(&support::script_record(&[(9, 611), (24, 0)]));
        let mut state = support::mission_session(&content);
        start(&mut state, &content, typed);
        let before = state.globals.encounter_headers.record(typed).unwrap();
        let initial_random = state.random.clone();
        support::saved_command(select.clone(), &mut state, &content);
        assert!(matches!(
            state.exploration.as_ref().unwrap().presentation,
            Some(Presentation::Dialogue { id: 1, .. })
        ));
        assert_eq!(
            &state.globals.encounter_headers.record(typed).unwrap()[header..],
            &before[header..]
        );
        support::saved_command(Command::Continue, &mut state, &content);
        assert!(matches!(
            state.exploration.as_ref().unwrap().presentation,
            Some(Presentation::Menu { .. } | Presentation::Typed { .. })
        ));
        let events = support::saved_command(select, &mut state, &content);
        assert_eq!(support::event_sounds(&events), [611]);
        assert!(state.exploration.as_ref().unwrap().script.is_none());
        assert_eq!(state.random, initial_random);
    }
}

#[test]
fn round_predicate_reads_kept_headers_once_and_resumes_the_next_slot_after_saved_pages() {
    let foundation = support::fixture("round-stories.json");
    for mode in 1_i16..=3 {
        let mut content = content();
        let tables = Arc::make_mut(&mut content.scenario.as_mut().unwrap().maps[0].shared);
        for table in [&mut tables.menus, &mut tables.typed_encounters] {
            table[..96].fill(0);
            table[0..3].copy_from_slice(&[1, 9, 24]);
            table[32..36].copy_from_slice(&[0, 1, 2, 99]);
        }
        tables.menus[96] = 1;
        tables.menus[101] = 5;
        tables.typed_encounters[152] = 5;
        tables.door_scripts[42..126].fill(0);
        tables.door_scripts[50..52].copy_from_slice(&42_i16.to_be_bytes());
        tables.door_scripts[66..68].copy_from_slice(&900_i16.to_be_bytes());
        tables.door_scripts[92..98].copy_from_slice(&[0, 1, 0, 9, 0, 24]);
        tables.door_scripts[108..112].copy_from_slice(&[0, 1, 2, 100]);
        tables.combat.as_mut().unwrap().encounters.resize(9010, 0);
        for (offset, value) in [(0, 100_i16), (2, 1), (4, mode)] {
            tables.combat.as_mut().unwrap().encounters[9000 + offset..9002 + offset]
                .copy_from_slice(&value.to_be_bytes());
        }
        let mut state = support::mission_session(&content);
        start(&mut state, &content, mode == 2);
        support::saved_command(Command::CancelMenu, &mut state, &content);
        let (mut arena, warriors, random) =
            crate::binary_round_stories::input(&foundation, &foundation.array("cases")[0]);
        state.warriors = warriors;
        state.random = random;
        arena.battle_order_index = 109;
        arena.battle_order[109] = 0;
        arena.round_macros = RoundMacros {
            slots: [1, 2, 0, 0, 0],
            ..Default::default()
        };
        state.combat = Some(arena);
        state.phase = Phase::Combat;
        crate::native_common::source_victory_context(&mut state);
        let mut expected = state.random.clone();
        expected.bound(100);
        support::saved_command(Command::EndTurn, &mut state, &content);
        assert_eq!(state.random, expected);
        if mode != 3 {
            assert!(state.round_dispatch.is_some());
            assert_eq!(
                support::event_sounds(&support::saved_command(
                    Command::Continue,
                    &mut state,
                    &content
                )),
                [611]
            );
            assert_eq!(state.random, expected);
        }
        assert!(state.round_dispatch.is_none());
        assert!(matches!(
            state.exploration.as_ref().unwrap().presentation,
            Some(Presentation::Dialogue { id: 1, .. })
        ));
        let events = support::saved_command(Command::Continue, &mut state, &content);
        assert_eq!(
            support::event_sounds(&events)
                .iter()
                .filter(|id| **id == 612)
                .count(),
            1
        );
        assert!(current_combat(&state).round_macros.pending.is_none());
    }
}

#[test]
fn mission_end_and_defeat_keep_health_context_across_saved_commands() {
    let content = content();
    for command in [Command::EndMission, Command::EndTurn] {
        let mut state = support::mission_session(&content);
        let health = &mut state.globals.health_context;
        health.last_body = 12;
        health.effect = 7;
        health.affected = [3; 6];
        health.marks_affected = true;
        health.kill_party = 2;
        let mut expected = health.clone();
        expected.marks_affected = false;
        expected.kill_party = 0;
        if matches!(command, Command::EndTurn) {
            state.combat = Some(support::battle());
            // Teardown copies the combat packet's health effect.
            state.combat.as_mut().unwrap().spell_info[18] = 7;
            state.combat.as_mut().unwrap().pending_last_defeated = Some(0);
            state.warriors.resize(6, state.warriors[0].clone());
            expected.last_body = 0;
            state.phase = Phase::Combat;
            crate::native_common::source_victory_context(&mut state);
            for warrior in &mut state.warriors {
                warrior.record_mut().set_in_battle(0);
            }
        }
        support::saved_command(command, &mut state, &content);
        assert_eq!(state.phase, Phase::SquadSelection);
        assert_eq!(state.globals.health_context, expected);
    }
}
