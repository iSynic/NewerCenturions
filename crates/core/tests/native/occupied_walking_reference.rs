//! Original occupied and involuntary walking over retained callers and shared work.
use super::detached_attacks::saved_return;
use super::outdoor_deaths::{prepare, verify};
use crate::combat::runtime::CombatReturn;
use crate::feedback::GameFeedback;
use crate::test_support::{bytes, fixture};
use crate::{Command, Content, Event, Session};
use centurions_content::InventoryRecord;
use serde_json::{Value, json};

fn word(raw: &Value) -> i16 {
    i16::from_be_bytes(bytes(raw).try_into().unwrap())
}

/// The captured walk caller of one case, resumed at its occupied destination.
fn walking(source: &Value) -> (Session, Content) {
    let before = &source["before"];
    let (mut state, content) = prepare(source);
    state.ammunition = bytes(&before["ammunition_reserves"])[..800]
        .as_chunks::<4>()
        .0
        .iter()
        .map(|raw| i32::from_be_bytes(*raw))
        .collect();
    state.globals.inventory_record =
        InventoryRecord::read(&bytes(&before["selected_inventory_record"])).unwrap();
    state.globals.possessed = word(&before["confusion_global"]);
    state.holdover = bytes(&before["allies"]);
    let current = state.globals.battle_order.order[0];
    saved_return(&mut state, &content);
    let locals = &source["loop_locals"];
    let caller: CombatReturn = serde_json::from_value(json!({"WarriorMove": {
            "actor":source["original_frame_actor"],"direction":[1,0],"plan":{"Ground":{"tile":1155}},
            "phase":"AfterEntry","cost":0,"interrupted":false,"callback_returned":false,
            "locals":{"previous_tile":locals["previous_tile"],"retries":locals["retries"],"damage_snapshot":locals["damage_snapshot"]},
            "guard":{"actor":current,"mode":0,"direction":[1,0],"started":true,"changed":false,"size":1,
                "phase":{"Guards":{"actor":current,"departure":false,"count":1,"index":0,"changed":false,"interrupted":false,"phase":"AwaitAttack","result":false}}}
        }})).unwrap();
    state.combat_story.as_mut().unwrap().return_to = Some(Box::new(caller));
    (state, content)
}

#[test]
fn occupied_and_involuntary_walking_match_native_shared_work_and_choices() {
    for case in fixture("occupied-walking.json")["cases"]
        .as_array()
        .unwrap()
    {
        let source = &case["declarations"]["source_input"];
        let (mut state, content) = walking(source);
        let locals = &source["loop_locals"];
        if case["name"] == "retry-blocked-old19" {
            let path = std::env::temp_dir().join(format!(
                "centurions-walk-caller-{}-{}.json",
                std::process::id(),
                case["name"].as_str().unwrap()
            ));
            crate::save(&path, &state, &content).unwrap();
            let document: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            assert_eq!(
                document["session"]["combat_story"]["return_to"]["WarriorMove"]["locals"],
                json!({"previous_tile":locals["previous_tile"],"retries":locals["retries"],"damage_snapshot":locals["damage_snapshot"]})
            );
            let restored = crate::load(&path, &content).unwrap();
            assert!(state == restored);
            assert_eq!(restored.globals.possessed, 1);
            state = restored;
            std::fs::remove_file(path).unwrap();
        }
        let mut events = crate::apply(Command::Continue, &mut state, &content)
            .unwrap_or_else(|error| panic!("{}: {error}", case["name"]));
        let questions = case["comparison"]["controlled_question_count"]
            .as_u64()
            .unwrap() as usize;
        if questions != 0 {
            assert_eq!(questions, 1);
            let pending = state
                .out_of_combat_walk_choice
                .as_ref()
                .expect("captured question not offered");
            assert_eq!(pending.body, source["target"].as_i64().unwrap() as i16);
            let exchange = if source["question_answer"] == 1 {
                None
            } else if pending.exchange {
                Some(true)
            } else {
                Some(false)
            };
            let path = std::env::temp_dir().join(format!(
                "centurions-walk-choice-{}-{}.json",
                std::process::id(),
                case["name"].as_str().unwrap()
            ));
            crate::save(&path, &state, &content).unwrap();
            let restored = crate::load(&path, &content).unwrap();
            assert!(state == restored);
            std::fs::remove_file(path).unwrap();
            state = restored;
            events.extend(
                crate::apply(
                    Command::AnswerOccupiedChoice { exchange },
                    &mut state,
                    &content,
                )
                .unwrap_or_else(|error| panic!("{} answer: {error}", case["name"])),
            );
        } else {
            assert!(state.out_of_combat_walk_choice.is_none());
        }
        let mut omitted = 0;
        events.retain(|event| {
            if matches!(event, Event::Sound { resource_id: 20005 }) && omitted < questions {
                omitted += 1;
                false
            } else {
                true
            }
        });
        assert_eq!(
            omitted, questions,
            "{} modeled question sound projection",
            case["name"]
        );
        verify(&state, &content, &events, case);
        assert_eq!(
            state.globals.possessed,
            word(&case["memory"]["confusion_global"]["hex"]),
            "{} confusion WORD",
            case["name"]
        );
        assert_eq!(
            state.globals.character_bare_hand_bonus,
            word(&case["memory"]["character_work_bonus"]["hex"])
        );
        assert_eq!(
            state
                .ammunition
                .iter()
                .flat_map(|value| value.to_be_bytes())
                .collect::<Vec<_>>(),
            bytes(&case["memory"]["ammunition_reserves"]["hex"])[..800],
            "{} ammunition reserves",
            case["name"]
        );
        assert_eq!(
            state.globals.inventory_record.0.to_vec(),
            bytes(&case["memory"]["selected_inventory_record"]["hex"]),
            "{} retained inventory record",
            case["name"]
        );
        let warnings: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                Event::Feedback(GameFeedback::Warning { index }) => Some(*index),
                _ => None,
            })
            .collect();
        let native: Vec<_> = case["calls"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|call| call["id"] == "warning")
            .map(|call| word(&call["arguments"]["id"]))
            .collect();
        assert_eq!(warnings, native, "{} warning requests", case["name"]);
    }
}

#[test]
fn occupied_firing_past_seven_triples_writes_the_defeated_word_and_packet() {
    let cases = fixture("occupied-walking.json");
    let case = cases["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["name"] == "occupied-firing-partial-shot")
        .unwrap();
    let source = &case["declarations"]["source_input"];
    let (mut state, mut content) = walking(source);
    let actor = source["actor"].as_i64().unwrap() as usize;
    // Controlled linkage: eight shots, so the eighth triple lands on WORD44374c and 443750.
    let weapon = state.warriors[actor].record().equipped_weapon().unwrap();
    content.items[usize::from(weapon.unsigned_abs())][76..78].copy_from_slice(&8_i16.to_be_bytes());
    state.warriors[actor]
        .record_mut()
        .set_inventory_quantity(0, 100);
    crate::apply(Command::Continue, &mut state, &content).unwrap();
    let target = source["target"].as_i64().unwrap() as i16;
    assert_eq!(state.globals.kill_counts.kill_mon, target);
    assert_eq!(state.globals.spell_info[2..4], [0, 2]);
}
