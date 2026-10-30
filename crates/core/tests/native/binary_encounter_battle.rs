use crate::{prelude::*, support};

use centurions_core::combat::encounter_battle;

fn selected_reward_start(content: &Content, entry_treasure: i16) -> Session {
    let fixture = support::fixture("selected-battle-treasure.json");
    let case = &fixture.array("cases")[0];
    let mut state = support::mission_session(content);
    state.warriors = support::warriors(&case.bytes("before"));
    state.globals.party_track = [1, 0, 1, 0, 0, 0];
    let position = state.exploration.as_mut().unwrap();
    position.script = Some(ScriptState::scheduled(case.bytes("record"), 0));
    position.presentation = Some(Presentation::Dialogue {
        id: 0,
        text: String::new(),
    });
    support::saved_command(Command::Continue, &mut state, content);
    assert!(state.selected_battle_treasure.is_some());
    assert_eq!(
        centurions_content::word(&state.globals.extra_record, 8).unwrap(),
        entry_treasure
    );
    state
}

fn selected_reward_return(content: &Content, entry_treasure: i16) -> Session {
    let fixture = support::fixture("selected-battle-treasure.json");
    let case = &fixture.array("cases")[0];
    let mut state = selected_reward_start(content, entry_treasure);
    // The old native fixture controls combat's return. These additional caller
    // histories are source compositions, not captured nested battles.
    state.warriors = support::warriors(&case.bytes("battle_return"));
    let mut arena = support::battle();
    arena.monsters = state.globals.monsters.clone();
    arena.squad.pos.truncate(state.warriors.len());
    arena.squad.char_under.truncate(state.warriors.len());
    state.combat = Some(arena);
    state.exploration.as_mut().unwrap().presentation = Some(Presentation::Survivors);
    state.random = RandomState::new(case.n("seed_before"));
    state.party_currency = case.n("currency_before");
    support::saved_command(Command::Continue, &mut state, content);
    state
}

#[test]
fn selected_reward_rereads_shared_extra_after_saved_first_window() {
    let fixture = support::fixture("selected-battle-treasure.json");
    let case = &fixture.array("cases")[0];
    for (entry, retained, early) in [(0_i16, 8_i16, false), (8, 0, false), (8, 8, true)] {
        let mut content = support::content();
        let tables = Arc::make_mut(&mut content.scenario.as_mut().unwrap().maps[0].shared);
        tables.combat.as_mut().unwrap().encounters[5350..5360]
            .copy_from_slice(&case.bytes("extra"));
        tables.combat.as_mut().unwrap().encounters[5358..5360]
            .copy_from_slice(&entry.to_be_bytes());
        tables.treasure[704..792].copy_from_slice(&case.bytes("treasure"));
        let mut state = selected_reward_return(&content, entry);
        state.globals.extra_record[8..10]
            .copy_from_slice(&retained.to_be_bytes());
        if early {
            state.globals.coward = 1;
        }
        let currency = state.party_currency;
        let events = support::saved_command(Command::Continue, &mut state, &content);
        if retained == 0 || early {
            assert!(state.selected_battle_treasure.is_none());
            assert!(events.contains(&Event::BattleEnded { victory: true }));
            assert_eq!(state.party_currency, currency);
            if !early {
                assert_eq!(support::event_sounds(&events), [5]);
            }
        } else {
            assert!(state.selected_battle_treasure.is_some());
            assert_eq!(support::event_sounds(&events), [20]);
            assert_eq!(state.party_currency, case.n::<i32>("currency_after"));
            assert_eq!(
                support::record_bytes(&state.warriors),
                case.bytes("bonus_prepared")
            );
            support::saved_command(Command::Continue, &mut state, &content);
            assert!(state.selected_battle_treasure.is_none());
        }
    }
    // A selected group with no active member returns through Getup's booty2;
    // that prefix reactivates the party before the selected caller tests it.
    let mut content = support::content();
    let tables = Arc::make_mut(&mut content.scenario.as_mut().unwrap().maps[0].shared);
    tables.combat.as_mut().unwrap().encounters[5350..5360].copy_from_slice(&case.bytes("extra"));
    tables.treasure[704..792].copy_from_slice(&case.bytes("treasure"));
    let mut state = selected_reward_start(&content, 8);
    for warrior in &mut state.warriors {
        warrior.record_mut().set_in_battle(0);
    }
    state.combat.as_mut().unwrap().kill_party = 0;
    let currency = state.party_currency;
    let random = state.random.clone();
    let events = support::saved_command(Command::EndTurn, &mut state, &content);
    assert!(state.combat.is_none() && state.selected_battle_treasure.is_none());
    assert!(state.combat_story.is_none() && state.round_dispatch.is_none());
    assert!(state.exploration.as_ref().unwrap().script.is_none());
    assert_eq!(state.phase, Phase::Exploration);
    assert_eq!(state.globals.coward, 1);
    assert_eq!(
        state.globals.treasure,
        case.bytes("treasure")
    );
    assert_eq!(state.party_currency, currency);
    assert_eq!(state.random, random);
    assert!(
        state
            .warriors
            .iter()
            .all(|warrior| warrior.record().in_battle() == 1)
    );
    assert!(events.contains(&Event::BattleEnded { victory: true }));
    assert_eq!(support::event_sounds(&events), [5]);
    assert!(!events.iter().any(|event| matches!(
        event,
        Event::Feedback(centurions_core::feedback::GameFeedback::Flash {
            address: 0x401b7e,
            ..
        })
    )));
    let mut terminal = selected_reward_start(&content, 8);
    terminal.combat.as_mut().unwrap().kill_party = terminal.warriors.len() as i8;
    let early = terminal.globals.coward;
    let events = support::saved_command(Command::EndTurn, &mut terminal, &content);
    assert!(events.contains(&Event::MissionEnded { defeated: true }));
    assert!(terminal.exploration.is_none() && terminal.selected_battle_treasure.is_none());
    assert_eq!(terminal.globals.coward, early);
}

#[test]
fn selected_parent_returns_after_saved_death_and_round_child_battles() {
    use centurions_core::combat::round_stories::{RoundDispatch, RoundProgram};
    use centurions_core::combat::runtime::DeathStory;

    let fixture = support::fixture("selected-battle-treasure.json");
    let case = &fixture.array("cases")[0];
    for death in [true, false] {
        let mut content = support::content();
        let child_record = if death {
            support::script_record(&[(48, 536), (9, 7)])
        } else {
            support::script_record(&[(100, 1), (1, 1), (48, 536), (9, 7)])
        };
        let tables = Arc::make_mut(&mut content.scenario.as_mut().unwrap().maps[0].shared);
        tables.door_scripts[42..84].copy_from_slice(&child_record);
        let encounters = &mut tables.combat.as_mut().unwrap().encounters;
        encounters[5350..5360].copy_from_slice(&case.bytes("extra"));
        encounters[5360..5370].copy_from_slice(&case.bytes("extra"));
        encounters[5368..5370].fill(0);
        let mut state = selected_reward_start(&content, 8);
        let mut parent = state.selected_battle_treasure.clone();
        let suspended_script = state.exploration.as_mut().unwrap().script.take();
        // Explicit source-composed callback entry; the commands below own child
        // capture, persistence, teardown and the selected parent's restoration.
        if death {
            state.combat_story = Some(DeathStory {
                script: 1,
                actor: 0,
                direct: true,
                shared_dispatch: true,
                suspended_script,
                outer_after_message: state.battle_after_message,
                outer_treasure: state.battle_treasure,
                outer_rewards: state.ordinary_battle_rewards.clone(),
                outer_group: state.battle_group_script,
                outer_npc: state.battle_npc.clone(),
                ..Default::default()
            });
        } else {
            state.round_dispatch = Some(RoundDispatch {
                suspended_script,
                death_depth: Some(0),
                ..Default::default()
            });
            state.combat.as_mut().unwrap().battle_order_index = 110;
            state.combat.as_mut().unwrap().round_macros.slots[0] = 1;
            state.combat.as_mut().unwrap().round_macros.pending = Some(RoundProgram {
                slot: 0,
                script: Some(1),
                cursor: 1,
                text: None,
                delegated: true,
            });
        }
        let position = state.exploration.as_mut().unwrap();
        position.script = Some(ScriptState::scheduled(child_record, 0));
        position.presentation = Some(Presentation::Dialogue {
            id: 0,
            text: String::new(),
        });
        support::saved_command(Command::Continue, &mut state, &content);
        if !death {
            assert!(state.combat.is_none());
            assert!(matches!(
                state.exploration.as_ref().unwrap().presentation,
                Some(Presentation::Dialogue { id: 1, .. })
            ));
            assert!(state.selected_battle_treasure.is_some());
            assert!(!state.round_dispatch.as_ref().unwrap().child_battle);
            parent = state.selected_battle_treasure.clone();
            support::saved_command(Command::Continue, &mut state, &content);
        }
        let captured = if death {
            state.combat_story.as_ref().unwrap().outer_selected.clone()
        } else {
            state
                .round_dispatch
                .as_ref()
                .unwrap()
                .outer_battle
                .as_ref()
                .unwrap()
                .selected
                .clone()
        };
        assert_eq!(captured, parent);
        assert!(state.combat.is_some());
        state.warriors = support::warriors(&case.bytes("battle_return"));
        let mut arena = support::battle();
        arena.monsters = state.globals.monsters.clone();
        arena.squad.pos.truncate(state.warriors.len());
        arena.squad.char_under.truncate(state.warriors.len());
        state.combat = Some(arena);
        state.exploration.as_mut().unwrap().presentation = Some(Presentation::Survivors);
        state.random = RandomState::new(case.n("seed_before"));
        let mut sounds = vec![];
        for _ in 0..6 {
            sounds.extend(support::event_sounds(&support::saved_command(
                Command::Continue,
                &mut state,
                &content,
            )));
            if state.selected_battle_treasure.is_none()
                && state.combat_story.is_none()
                && state.round_dispatch.is_none()
            {
                break;
            }
        }
        assert!(state.selected_battle_treasure.is_none());
        assert!(state.combat_story.is_none() && state.round_dispatch.is_none());
        assert!(state.combat.is_none());
        assert!(state.exploration.as_ref().unwrap().script.is_none());
        assert_eq!(
            centurions_content::word(&state.globals.extra_record, 8).unwrap(),
            0
        );
        assert!(sounds.windows(2).any(|pair| pair == [7, 20]));
        assert_eq!(sounds.last(), Some(&5));
    }
    let mut content = support::content();
    let tables = Arc::make_mut(&mut content.scenario.as_mut().unwrap().maps[0].shared);
    tables.combat.as_mut().unwrap().encounters[5350..5360].copy_from_slice(&case.bytes("extra"));
    tables.combat.as_mut().unwrap().encounters[..10]
        .copy_from_slice(&support::word_bytes(&[1, 1, 24, 0, 0]));
    tables.schedule = vec![0; 40];
    for (offset, value) in [(0, 2_i16), (4, 100), (6, 1), (18, -1)] {
        tables.schedule[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
    }
    tables.scheduled_scripts[42..84].copy_from_slice(&case.bytes("record"));
    let mut selected = support::mission_session(&content);
    selected.warriors = support::warriors(&case.bytes("before"));
    selected.globals.party_track = [1, 0, 1, 0, 0, 0];
    let position = selected.exploration.as_mut().unwrap();
    let index = centurions_content::mission::OpeningMission::index(position.x, position.y).unwrap();
    position.floor[index] = 0;
    let mut clock = vec![0; 42];
    clock[22..24].copy_from_slice(&63_i16.to_be_bytes());
    position.script = Some(ScriptState::scheduled(clock, 7));
    position.presentation = Some(Presentation::ScriptPause);
    support::saved_command(Command::Continue, &mut selected, &content);
    assert!(selected.zero_time.is_some() && selected.selected_battle_treasure.is_some());
    let captured = support::fixture("detached-returns.json");
    let walk = captured
        .array("cases")
        .iter()
        .find(|case| case.text("name") == "walk-shared")
        .unwrap();
    // Compose a real selected caller with independent pre-return walking inputs.
    // The capture qualifies the suffix, not this constructed nested history.
    let mut state =
        super::binary_detached_returns::initial(&walk["declarations"]["source"], &content);
    state.zero_time = selected.zero_time.take();
    let calendar = selected.exploration.as_ref().unwrap();
    let position = state.exploration.as_mut().unwrap();
    position.schedule.clone_from(&calendar.schedule);
    (position.day, position.hour, position.minute) = (calendar.day, calendar.hour, calendar.minute);
    position.floor[index] = 0;
    state.globals.battlefield.saved_land = selected
        .globals
        .battlefield
        .saved_land
        .clone();
    let story = state.combat_story.as_mut().unwrap();
    story.outer_selected = selected.selected_battle_treasure.take();
    story.outer_after_message = selected.battle_after_message;
    story.outer_treasure = selected.battle_treasure;
    story.outer_rewards = selected.ordinary_battle_rewards.take();
    story.outer_group = selected.battle_group_script;
    story.outer_npc = selected.battle_npc.take();
    story.shared_dispatch = true;
    story.suspended_script = selected.exploration.as_mut().unwrap().script.take();
    let mut extra = case.bytes("extra");
    extra[8..10].fill(0);
    state.globals.extra_record = extra;
    assert!(
        state.globals.kill_counts.num_enemy
            > state.globals.kill_counts.kill_mon
    );
    let events = support::saved_command(Command::Continue, &mut state, &content);
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event, Event::TacticalMoved { .. }))
            .count(),
        1
    );
    assert!(events.contains(&Event::TacticalMoved {
        body: 3,
        x: 20,
        y: 19
    }));
    assert_eq!(
        state.globals.battlefield.pos[3],
        (20, 19)
    );
    assert!(state.combat.is_none() && state.combat_story.is_none());
    assert!(state.selected_battle_treasure.is_some());
    assert!(state.zero_time.is_some());
    assert_eq!(
        state.exploration.as_ref().unwrap().presentation,
        Some(Presentation::Loot)
    );
    assert!(
        support::event_sounds(&events)
            .windows(2)
            .any(|pair| pair == [82, 20])
    );
    let events = support::saved_command(Command::Continue, &mut state, &content);
    assert!(state.selected_battle_treasure.is_none());
    assert!(state.zero_time.is_none());
    assert!(state.exploration.as_ref().unwrap().script.is_none());
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::TacticalMoved { .. }))
    );
    assert_eq!(support::event_sounds(&events), [5]);
}

#[test]
fn selected_battle_bonus_has_two_saved_reward_windows_and_matches_native_preparations() {
    for (index, case) in support::fixture("selected-battle-treasure.json")
        .array("cases")
        .iter()
        .enumerate()
    {
        let mut content = support::content();
        let tables = Arc::make_mut(&mut content.scenario.as_mut().unwrap().maps[0].shared);
        tables.combat.as_mut().unwrap().encounters[5350..5360]
            .copy_from_slice(&case.bytes("extra"));
        tables.treasure[704..792].copy_from_slice(&case.bytes("treasure"));
        let mut state = support::mission_session(&content);
        state.warriors = support::warriors(&case.bytes("before"));
        state.globals.party_track = [1, 0, 1, 0, 0, 0];
        let position = state.exploration.as_mut().unwrap();
        position.script = Some(ScriptState::scheduled(case.bytes("record"), 0));
        position.presentation = Some(Presentation::Dialogue {
            id: 0,
            text: String::new(),
        });
        support::saved_command(Command::Continue, &mut state, &content);
        assert_eq!(state.phase, Phase::Combat);
        assert!(state.selected_battle_treasure.is_some());
        // The oracle's combat service supplies this controlled return, not a played battle.
        state.warriors = support::warriors(&case.bytes("battle_return"));
        let mut arena = support::battle();
        arena.monsters = state.globals.monsters.clone();
        arena.squad.pos.truncate(state.warriors.len());
        arena.squad.char_under.truncate(state.warriors.len());
        state.combat = Some(arena);
        state.exploration.as_mut().unwrap().presentation = Some(Presentation::Survivors);
        state.random = RandomState::new(case.n("seed_before"));
        state.party_currency = case.n("currency_before");
        if index == 0 {
            let mut escaped = state.clone();
            escaped.exploration.as_mut().unwrap().presentation = None;
            escaped.combat.as_mut().unwrap().battle_order[1] = 0;
            escaped.warriors[0].record_mut().set_turn_skip_duration(0);
            escaped.warriors[0]
                .record_mut()
                .set_condition(centurions_content::records::Condition::Retreating, 0);
            for warrior in escaped.warriors.iter_mut().skip(1) {
                warrior.record_mut().set_in_battle(0);
            }
            let events = support::saved_command(Command::EscapeCombat, &mut escaped, &content);
            assert!(
                events
                    .iter()
                    .any(|e| matches!(e, Event::BattleEnded { victory: false }))
            );
            assert!(escaped.selected_battle_treasure.is_none());
            assert_eq!(escaped.party_currency, state.party_currency);
        }
        let events = support::saved_command(Command::Continue, &mut state, &content);
        assert_eq!(
            support::record_bytes(&state.warriors),
            case.bytes("ordinary_prepared")
        );
        assert_eq!(support::event_sounds(&events), [20]);
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::BattleEnded { .. }))
        );
        let ordinary = state.clone();
        let events = support::saved_command(Command::Continue, &mut state, &content);
        assert_eq!(
            support::record_bytes(&state.warriors),
            case.bytes("bonus_prepared")
        );
        assert_eq!(support::event_sounds(&events), [20]);
        assert_eq!(state.party_currency, case.n::<i32>("currency_after"));
        support::assert_rng(&state.random, &case["seed_after"], &case["draws"]);
        assert_eq!(
            state.exploration.as_ref().unwrap().presentation,
            Some(Presentation::Loot)
        );
        assert_eq!(
            state
                .exploration
                .as_ref()
                .unwrap()
                .script
                .as_ref()
                .unwrap()
                .cursor,
            1
        );
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::BattleEnded { .. }))
        );
        let events = support::saved_command(Command::Continue, &mut state, &content);
        assert_eq!(support::event_sounds(&events), [5]);
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::BattleEnded { victory: true }))
        );
        assert_eq!(support::record_bytes(&state.warriors), case.bytes("after"));
        assert!(state.selected_battle_treasure.is_none());
        assert!(state.exploration.as_ref().unwrap().script.is_none());
        if index == 0 {
            // An overflowing award saturates the LONG under the modern IEEE policy.
            let mut saturated = ordinary.clone();
            saturated.warriors[0]
                .record_mut()
                .set_upgrade_points(i32::MAX);
            apply(Command::Continue, &mut saturated, &content).unwrap();
            assert_eq!(
                centurions_content::records::CharacterInformation(&saturated.warriors[0].bytes).upgrade_points().unwrap(),
                i32::MAX
            );
        }
    }
}

#[test]
fn encounter_battle_selection_matches_original_signed_words_effects_and_random_consumption() {
    let fixture: serde_json::Value = support::fixture("encounter-battle.json");
    for case in fixture.array("cases") {
        let hex = case.text("extra");
        let extra = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect::<Vec<_>>();
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let result = encounter_battle(&extra, &mut random).unwrap();
        support::expect(result.battle, &case["battle"]);
        assert_eq!(
            result.treasure,
            i16::from_be_bytes(extra[8..10].try_into().unwrap())
        );
        let mut effects = vec![];
        if result.sound != 0 {
            effects.push(serde_json::json!(["sound", result.sound]));
        }
        if result.message != 0 {
            effects.push(serde_json::json!(["message", result.message]));
        }
        assert_eq!(serde_json::json!(effects), case["events"]);
        support::assert_rng(&random, &case["seed_after"], &case["draws"]);
    }
}
