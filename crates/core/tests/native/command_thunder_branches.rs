//! Assisted authored branch checkpoints; native predicate receipts remain separate evidence.
use crate::{prelude::*, support};
use support::{checkpoint, replay_command};

fn frame(content: &Content, map: i32, script: Option<i16>, slot: usize, parameter: i16) -> Session {
    let mut state = support::mission_session(content);
    let authored = content.map(map).unwrap();
    let position = state.exploration.as_mut().unwrap();
    position.restore_map(map, centurions_core::MapState::authored(authored));
    let record = script.map_or_else(
        || position.event_headers[slot * 42..(slot + 1) * 42].to_vec(),
        |id| authored.script(id).unwrap().to_vec(),
    );
    let cursor = (0..8)
        .find(|i| {
            word(&record, 8 + i * 2).unwrap() != 0
                && word(&record, 24 + i * 2).unwrap() == parameter
        })
        .unwrap();
    position.script = Some(support::script_state(record, cursor as u8));
    position.presentation = Some(Presentation::Dialogue {
        id: 0,
        text: String::new(),
    });
    state
}

fn continue_saved(state: &mut Session, content: &Content) {
    use std::io::Write;
    let before = state.clone();
    let mut resumed = checkpoint(state, content, "branch-before");
    let events = replay_command(Command::Continue, state, &mut resumed, content);
    checkpoint(state, content, "branch-after");
    let mut record = std::fs::OpenOptions::new()
        .append(true)
        .open(support::root().join("analysis/thunder-verification/branches.jsonl"))
        .unwrap();
    writeln!(record, "{}", serde_json::json!({"command":Command::Continue,"before":before,"after":state,"events":events})).unwrap();
}

fn dialogue(state: &Session) -> Option<i16> {
    match state.exploration.as_ref().unwrap().presentation {
        Some(Presentation::Dialogue { id, .. }) => Some(id),
        _ => None,
    }
}

#[test]
fn authored_item_variable_ally_and_random_branches_resume_without_repeating_effects() {
    let content = content();
    let directory = support::root().join("analysis/thunder-verification");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::File::create(directory.join("branches.jsonl")).unwrap();
    // Original source owners and the first authored dialogue after the predicate.
    // None also covers typed encounters; their identity is checked below.
    for (map, script, slot, parameter, absent, present) in [
        (0, Some(106), 0, 203, None, Some(390)),
        (1, None, 10, 133, Some(256), None),
        (4, None, 11, 390, None, Some(829)),
        (5, None, 2, 183, Some(342), None),
        (5, None, 15, 185, None, Some(-355)),
        (8, None, 49, 602, Some(1220), Some(1237)),
    ] {
        for setup in 0..if matches!(parameter, 203 | 390) { 3 } else { 2 } {
            let supplied = setup == 1;
            let mut state = frame(&content, map, script, slot, parameter);
            if setup != 0 {
                let record = state
                    .exploration
                    .as_ref()
                    .unwrap()
                    .script
                    .as_ref()
                    .unwrap()
                    .record
                    .clone();
                for i in 0..8 {
                    if word(&record, 8 + i * 2).unwrap() != 21 {
                        continue;
                    }
                    let extra = word(&record, 24 + i * 2).unwrap() as usize * 10;
                    let id = word(
                        &content
                            .map(map)
                            .unwrap()
                            .combat
                            .as_ref()
                            .unwrap()
                            .encounters,
                        extra,
                    )
                    .unwrap();
                    let item =
                        centurions_content::records::ItemDefinition(&content.items[id as usize])
                            .fresh_stock()
                            .unwrap();
                    let count = state.warriors[0].inventory().unwrap().len();
                    state.warriors[0]
                        .record_mut()
                        .append_inventory(count, &item);
                    if setup == 2 {
                        break;
                    }
                }
            }
            continue_saved(&mut state, &content);
            assert_eq!(
                dialogue(&state),
                if supplied { present } else { absent },
                "map{map} script{script:?} extra{parameter} supplied{supplied}"
            );
            if parameter == 203 && !supplied || parameter == 133 && supplied {
                let expected = if parameter == 203 { 0 } else { 2 };
                assert!(
                    matches!(state.exploration.as_ref().unwrap().presentation, Some(Presentation::Typed { id, .. }) if id == expected)
                );
            }
        }
    }
    for (map, script, slot, parameter, flag, threshold, message) in [
        (3, None, 11, 439, 40, 4, 924),
        (0, Some(248), 0, 472, 41, 5, 955),
    ] {
        for matched in [false, true] {
            let mut state = frame(&content, map, script, slot, parameter);
            state.flags[flag] = threshold - if matched { 1 } else { 2 };
            continue_saved(&mut state, &content);
            assert_eq!(state.flags[flag], threshold - u8::from(!matched));
            assert_eq!(dialogue(&state), matched.then_some(message));
        }
    }
    for allied in [false, true] {
        let mut state = frame(&content, 8, None, 42, 580);
        if allied {
            let definitions = &content.map(8).unwrap().combat.as_ref().unwrap().monsters;
            state.allies[..288].copy_from_slice(&definitions[112 * 288..113 * 288]);
            state.ally_count = 1;
        }
        continue_saved(&mut state, &content);
        assert_eq!(dialogue(&state), Some(if allied { 1186 } else { 1175 }));
    }
    let mut outcomes = std::collections::BTreeSet::new();
    for seed in 1..=4 {
        let mut state = frame(&content, 0, Some(161), 0, 318);
        // A recorded setup seed and party selection, never a mid-action RNG edit.
        state.random = RandomState::new(seed);
        state.party_mask.fill(1);
        continue_saved(&mut state, &content);
        let id = dialogue(&state).unwrap();
        assert!((603..=606).contains(&id));
        outcomes.insert(id);
    }
    assert_eq!(outcomes.len(), 4);
    let foundation = support::fixture("earned-loaded-shot-replay.json");
    for case in support::fixture("thunder-round-replacement.json").array("cases") {
        let mut state: Session =
            serde_json::from_value(foundation["foundations"][0]["session"].clone()).unwrap();
        let combat = state.combat.as_mut().unwrap();
        combat.order.fill(-1);
        combat.order[109] = 4;
        combat.cursor = 109;
        combat.round_scripts.slots = [case.n("script"), 0, 0, 0, 0];
        combat.round_scripts.counter = case.n::<i8>("counter").wrapping_sub(1);
        state.random = RandomState::new(case.n("seed_before"));
        support::saved_command(Command::EndTurn, &mut state, &content);
        let scripts = &current_combat(&state).round_scripts;
        assert_eq!(scripts.slots[0], case.n::<i16>("slot_after"));
        assert_eq!(
            scripts.pending.as_ref().and_then(|p| p.script),
            (case.n::<i16>("selected") >= 0).then(|| case.n("selected"))
        );
        checkpoint(&state, &content, "round-branch-after");
    }
    let mut state: Session =
        serde_json::from_value(foundation["foundations"][0]["session"].clone()).unwrap();
    let definition = &content.map(0).unwrap().combat.as_ref().unwrap().monsters[26 * 288..27 * 288];
    support::expect(
        word(definition, 0x6a).unwrap(),
        &support::fixture("death-feedback.json")["alternate"]["record_sound"],
    );
    let mut enemy = centurions_core::combat::monster::summon(
        MonsterRecord {
            bytes: definition.to_vec(),
        },
        1,
        0,
        &mut state.random,
    )
    .unwrap();
    let combat = state.combat.as_mut().unwrap();
    enemy.bytes[0x11e..0x120].copy_from_slice(&combat.enemies.records[0].bytes[0x11e..0x120]);
    enemy.bytes[0x110..0x112].copy_from_slice(&1_i16.to_be_bytes());
    enemy.bytes[0xce] = 2;
    combat.enemies.records[0] = enemy;
    combat.order.fill(-1);
    combat.order[109] = 4;
    combat.cursor = 109;
    let events = support::saved_command(Command::EndTurn, &mut state, &content);
    assert!(events.iter().any(|event| matches!(
        event,
        Event::Feedback(centurions_core::feedback::NativeFeedback::CombatResult {
            body: 10,
            code: -14,
            audible: false,
            ..
        })
    )));
    assert!(support::event_sounds(&events).contains(&word(definition, 0x6a).unwrap()));
    assert!(current_combat(&state).enemies.records[0].stamina().unwrap() < 1);
    checkpoint(&state, &content, "alternate-death-after");
}
