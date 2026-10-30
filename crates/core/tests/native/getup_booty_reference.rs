//! Getup's early booty return, with original bodycount modal continuations.
use super::detached_attacks::saved_return;
use super::outdoor_deaths::{prepare, verify};
use crate::combat::runtime::{CombatReturn, DeathStory};
use crate::feedback::GameFeedback;
use crate::test_support::{bytes, fixture};
use crate::{Command, Content, Event, Presentation, ScriptState, Session, story_cleanup};
use centurions_content::InventoryRecord;
use serde_json::{Value, json};

fn word(value: &Value) -> i16 {
    let raw = bytes(value);
    if raw.len() == 1 {
        i16::from(raw[0] as i8)
    } else {
        i16::from_be_bytes(raw.try_into().unwrap())
    }
}

fn checkpoint(state: &mut Session, content: &Content) {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let path = std::env::temp_dir().join(format!(
        "centurions-getup-booty-{}-{}.json",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    crate::save(&path, state, content).unwrap();
    let restored = crate::load(&path, content).unwrap();
    assert!(*state == restored);
    *state = restored;
    std::fs::remove_file(path).unwrap();
}

fn input(case: &Value) -> (Session, Content, DeathStory, ScriptState) {
    let source = &case["declarations"]["source_input"];
    let before = &source["before"];
    let (mut state, content) = prepare(source);
    state.holdover = bytes(&before["allies"]);
    state.held_over = word(&before["ally_count"]) as usize;
    state.suspend_allies = word(&before["suppress_allies"]);
    state.command_preferences.auto_refit = word(&before["auto_refit"]) != 0;
    state.globals.coward = word(&before["early_booty"]) as i8;
    state.globals.battle_rewards_suppressed = word(&before["battle_rewards_suppressed"]) as i8;
    state.globals.possessed = word(&before["confusion_global"]);
    state.globals.monsters.num_mon = word(&before["npc_count"]);
    state.globals.loot_quality = Some(bytes(&before["loot_flags"]));
    state.booty_list = bytes(&before["loot_scratch"])
        .as_chunks::<22>()
        .0
        .iter()
        .map(|raw| InventoryRecord(*raw))
        .collect();
    state.flags[..100].copy_from_slice(&bytes(&before["survivor_flags"]));
    state.globals.inventory_record =
        InventoryRecord::read(&bytes(&before["selected_inventory_record"])).unwrap();
    state.ammunition = bytes(&before["ammunition_reserves"])[..800]
        .as_chunks::<4>()
        .0
        .iter()
        .map(|raw| i32::from_be_bytes(*raw))
        .collect();
    state.globals.round_scripts.counter = Some(word(&before["round_counter"]) as i8);
    state.globals.round_scripts.slot = Some(word(&before["round_slot"]));
    state.globals.round_scripts.processing = Some(word(&before["round_flag"]) as u8);
    state.globals.round_scripts.slots = Some(
        bytes(&before["rounds"])
            .as_chunks::<2>()
            .0
            .iter()
            .map(|raw| i16::from_be_bytes(*raw))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap(),
    );
    saved_return(&mut state, &content);
    let caller: CombatReturn = serde_json::from_value(json!({"WarriorMove": {
        "actor":0,"direction":[1,0],"plan":{"Ground":{"tile":1155}},
        "phase":"AfterStep","cost":2,"interrupted":false,"callback_returned":false,
        "locals":{"previous_tile":1155,"retries":-1,"damage_snapshot":0},
        "guard":{"actor":0,"mode":0,"direction":[1,0],"started":true,"changed":false,"size":1,
            "phase":{"Guards":{"actor":0,"departure":false,"count":1,"index":0,"changed":false,"interrupted":false,"phase":"AwaitAttack","result":true}}}
    }})).unwrap();
    // These surrounding callers are constructed source histories, not captured prior play.
    let parent = DeathStory {
        body: 1,
        actor: 2,
        script: 1,
        cursor: 8,
        waiting_child: true,
        child_returned: true,
        ..Default::default()
    };
    let suspended = ScriptState::scheduled(vec![0; 42], 3);
    let story = state.combat_story.as_mut().unwrap();
    story.return_to = Some(Box::new(caller));
    story.parent = Some(Box::new(parent.clone()));
    story.shared_dispatch = true;
    story.suspended_script = Some(suspended.clone());
    state.scripted_cleanup = Some(story_cleanup::Stage::Complete);
    state.battle_after_message = 7;
    state.battle_treasure = 11;
    state.battle_group_script = Some(1);
    state.ordinary_battle_rewards = Some(Default::default());
    (state, content, parent, suspended)
}

fn verify_stage(state: &Session, content: &Content, events: &[Event], case: &Value) {
    let preceding = case["comparison"]["rng_preceding_draws"].as_u64().unwrap();
    let cumulative = preceding + case["draws"].as_u64().unwrap();
    assert_eq!(
        state.random.draws, cumulative,
        "{} cumulative draws",
        case["name"]
    );
    // The unchanged common comparison expects a whole-sequence counter. Only this
    // derived counter is adapted; raw segment captures and every byte stay intact.
    let mut projection = case.clone();
    projection["draws"] = cumulative.into();
    verify(state, content, events, &projection);
    for (name, actual) in [
        ("allies", state.holdover.clone()),
        (
            "loot_scratch",
            state.booty_list.iter().flat_map(|item| item.0).collect(),
        ),
        ("loot_flags", state.globals.loot_quality.clone().unwrap()),
        ("survivor_flags", state.flags[..100].to_vec()),
        (
            "selected_inventory_record",
            state.globals.inventory_record.0.to_vec(),
        ),
        (
            "ammunition_reserves",
            state
                .ammunition
                .iter()
                .flat_map(|value| value.to_be_bytes())
                .collect(),
        ),
    ] {
        let mut expected = bytes(&case["memory"][name]["hex"]);
        if name == "ammunition_reserves" {
            expected.truncate(800);
        }
        assert!(actual == expected, "{} {name}", case["name"]);
    }
    assert_eq!(
        state.held_over,
        word(&case["memory"]["ally_count"]["hex"]) as usize
    );
    assert_eq!(
        state.globals.coward,
        word(&case["memory"]["early_booty"]["hex"]) as i8
    );
    assert_eq!(
        state.globals.battle_rewards_suppressed,
        word(&case["memory"]["battle_rewards_suppressed"]["hex"]) as i8
    );
    assert_eq!(
        state.globals.monsters.num_mon,
        word(&case["memory"]["npc_count"]["hex"])
    );
    assert_eq!(
        state.globals.possessed,
        word(&case["memory"]["confusion_global"]["hex"])
    );
    assert_eq!(state.bandage_ready, 1);
    assert_eq!(
        state.globals.character_bare_hand_bonus,
        word(&case["memory"]["character_work_bonus"]["hex"])
    );
    assert_eq!(
        state
            .globals
            .round_scripts
            .slots
            .unwrap()
            .into_iter()
            .flat_map(i16::to_be_bytes)
            .collect::<Vec<_>>(),
        bytes(&case["memory"]["rounds"]["hex"])
    );
    assert_eq!(
        state.globals.round_scripts.counter,
        Some(word(&case["memory"]["round_counter"]["hex"]) as i8)
    );
    assert_eq!(
        state.globals.round_scripts.slot,
        Some(word(&case["memory"]["round_slot"]["hex"]))
    );
    assert_eq!(
        state.globals.round_scripts.processing,
        Some(word(&case["memory"]["round_flag"]["hex"]) as u8)
    );
    let queue = state.globals.outer_todo_queue.as_ref().unwrap();
    assert_eq!(
        queue
            .entries
            .iter()
            .flat_map(|entry| entry.bytes())
            .collect::<Vec<_>>(),
        bytes(&case["memory"]["death_queue"]["hex"])
    );
    assert_eq!(
        queue.count,
        word(&case["memory"]["death_queue_count"]["hex"]) as usize
    );
    assert_eq!(
        queue.pointer,
        i32::from_be_bytes(
            bytes(&case["memory"]["death_queue_pointer"]["hex"])
                .try_into()
                .unwrap()
        )
    );
    assert_eq!(
        state.globals.death_processing.doing_queue,
        bytes(&case["memory"]["death_queue_flag"]["hex"])[0]
    );
    let rows: Vec<_> = bytes(&case["memory"]["survivor_rows"]["hex"])
        .as_chunks::<2>()
        .0
        .iter()
        .map(|row| i16::from_be_bytes(*row))
        .filter(|row| *row != 0)
        .collect();
    if let Some(selection) = &state.survivors {
        assert_eq!(rows.len(), selection.eligible.len());
        for ((row, index), selected) in rows
            .iter()
            .zip(&selection.eligible)
            .zip(&selection.selected)
        {
            assert_eq!(row.unsigned_abs() as usize - 1, *index);
            assert_eq!(*row < 0, *selected);
        }
    }
    // DialogText represents the native warning's first text, wait, and trailing
    // text at address+80. Pure drawing requests are outside the event projection.
    let text: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            Event::Feedback(GameFeedback::DialogText {
                dialog,
                item,
                address,
                wait,
            }) => Some((*dialog, *item, *address, *wait)),
            _ => None,
        })
        .collect();
    let requests: Vec<_> = case["calls"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|call| call["id"] == "DiStr-presentation-request")
        .collect();
    assert_eq!(requests.len(), text.len() * 2);
    for (pair, (dialog, item, address, wait)) in requests.as_chunks::<2>().0.iter().zip(text) {
        assert_eq!(dialog, 173);
        assert_eq!(item as i16, word(&pair[0]["arguments"]["item"]));
        assert_eq!(item as i16, word(&pair[1]["arguments"]["item"]));
        assert_eq!(
            u32::from_str_radix(pair[0]["arguments"]["text"].as_str().unwrap(), 16).unwrap(),
            address
        );
        assert_eq!(
            u32::from_str_radix(pair[1]["arguments"]["text"].as_str().unwrap(), 16).unwrap(),
            address + 80
        );
        let delays: Vec<_> = case["calls"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|call| call["id"] == "delay")
            .map(|call| word(&call["arguments"]["ticks"]))
            .collect();
        assert_eq!(delays, [wait]);
    }
}

fn unchanged_callers(state: &Session, parent: &DeathStory, script: &ScriptState) {
    assert_eq!(state.combat_story.as_ref(), Some(parent));
    assert_eq!(
        state.exploration.as_ref().unwrap().script.as_ref(),
        Some(script)
    );
    assert_eq!(
        (
            state.battle_after_message,
            state.battle_treasure,
            state.battle_group_script
        ),
        (7, 11, Some(1))
    );
    assert_eq!(state.ordinary_battle_rewards, Some(Default::default()));
    assert!(state.combat.is_none());
}

#[test]
fn getup_early_booty_matches_native_and_preserves_its_callers() {
    let captures = fixture("getup-early-booty.json");
    let find = |name: &str| {
        captures["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["name"] == name)
            .unwrap()
    };
    for name in [
        "getup-early-booty-no-survivors",
        "getup-early-booty-one-survivor-modal",
    ] {
        let case = find(name);
        let (mut state, content, parent, script) = input(case);
        checkpoint(&mut state, &content);
        if name.ends_with("no-survivors") {
            let mut missing = state.clone();
            missing.globals.monsters.slots[99] = None;
            let before = missing.clone();
            let error = crate::apply(Command::Continue, &mut missing, &content).unwrap_err();
            assert!(matches!(
                error.root(),
                crate::RuleError::Unsupported {
                    action: "combat.retained_npc",
                    dependency: "inactive NPC record absent from legacy context"
                }
            ));
            assert!(missing == before, "unrecorded slot99 changed state");
        }
        let events = crate::apply(Command::Continue, &mut state, &content).unwrap();
        verify_stage(&state, &content, &events, case);
        unchanged_callers(&state, &parent, &script);
        if name.ends_with("no-survivors") {
            assert_eq!(state.scripted_cleanup, Some(story_cleanup::Stage::Complete));
            assert!(state.survivors.is_none());
            continue;
        }
        assert_eq!(
            state.exploration.as_ref().unwrap().presentation,
            Some(Presentation::Survivors)
        );
        assert_eq!(
            serde_json::to_value(&state.scripted_cleanup).unwrap()["RetainedGetupSurvivors"]["parent"],
            "Complete"
        );
        checkpoint(&mut state, &content);
        let mut confirmed = state.clone();
        let events = crate::apply(Command::Continue, &mut confirmed, &content).unwrap();
        verify_stage(
            &confirmed,
            &content,
            &events,
            find("getup-early-booty-original-confirm"),
        );
        unchanged_callers(&confirmed, &parent, &script);
        assert_eq!(
            confirmed.scripted_cleanup,
            Some(story_cleanup::Stage::Complete)
        );
        let events =
            crate::apply(Command::SelectSurvivor { row: 0 }, &mut state, &content).unwrap();
        verify_stage(
            &state,
            &content,
            &events,
            find("getup-early-booty-original-toggle"),
        );
        unchanged_callers(&state, &parent, &script);
        checkpoint(&mut state, &content);
        let events = crate::apply(Command::Continue, &mut state, &content).unwrap();
        verify_stage(
            &state,
            &content,
            &events,
            find("getup-early-booty-toggle-then-confirm"),
        );
        unchanged_callers(&state, &parent, &script);
        assert_eq!(state.scripted_cleanup, Some(story_cleanup::Stage::Complete));
        assert!(
            state.survivors.is_none() && state.exploration.as_ref().unwrap().presentation.is_none()
        );
    }
}
