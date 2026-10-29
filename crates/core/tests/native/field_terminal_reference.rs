//! Original field, condition and queue prefixes through the terminal caller boundary.
use super::detached_attacks::{initial, saved_return};
use super::outdoor_deaths::verify;
use crate::combat::death_scripts::{DeathEntry, DeathQueue};
use crate::combat::runtime::{CombatReturn, TurnFlow, retained_turn};
use crate::test_support::{bytes, content, fixture};
use crate::{Command, Content, Event, Session};
use serde_json::{Value, json};

fn number(before: &Value, name: &str) -> i16 {
    let raw = bytes(&before[name]);
    if raw.len() == 1 {
        i16::from(raw[0] as i8)
    } else {
        i16::from_be_bytes(raw.try_into().unwrap())
    }
}

fn prepare(source: &Value) -> (Session, Content) {
    let mut content = content();
    if let Some(packet) = source.get("attack_override") {
        let raw = bytes(&packet["hex"]);
        let offset = packet["index"].as_u64().unwrap() as usize * 58;
        for map in &mut content.scenario.as_mut().unwrap().maps {
            if let Some(combat) = std::sync::Arc::make_mut(&mut map.shared).combat.as_mut() {
                combat.attacks[offset..offset + 58].copy_from_slice(&raw);
            }
        }
    }
    let mut state = initial(source, &content);
    let before = &source["before"];
    state.exploration.as_mut().unwrap().event_headers = bytes(&before["event_headers"]);
    state
        .native_work
        .shop_work
        .copy_file(&bytes(&before["current_shop_work"]));
    state.allies = bytes(&before["allies"]);
    state.native_work.turn_round = Some(1);
    state.native_work.character_bare_hand_bonus = 0;
    state.native_work.round_work.counter = Some(number(before, "round_counter") as i8);
    state.native_work.round_work.slots = Some(
        bytes(&before["rounds"])
            .as_chunks::<2>()
            .0
            .iter()
            .map(|raw| i16::from_be_bytes(*raw))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap(),
    );
    state.native_work.round_work.slot = Some(number(before, "round_slot"));
    state.native_work.round_work.processing = Some(number(before, "round_flag") as u8);
    state.native_work.script_enemy = number(before, "script_enemy") as usize;
    let raw = bytes(&before["death_queue"]);
    state.native_work.retained_death_queue = Some(DeathQueue {
        entries: raw
            .as_chunks::<14>()
            .0
            .iter()
            .map(|entry| DeathEntry {
                monster: i16::from_be_bytes(entry[..2].try_into().unwrap()),
                faction: i16::from_be_bytes(entry[2..4].try_into().unwrap()),
                script: i16::from_be_bytes(entry[4..6].try_into().unwrap()),
                x: i32::from_be_bytes(entry[6..10].try_into().unwrap()),
                y: i32::from_be_bytes(entry[10..14].try_into().unwrap()),
            })
            .collect(),
        count: number(before, "death_queue_count") as usize,
        cursor: 0,
        pointer: i32::from_be_bytes(bytes(&before["death_queue_pointer"]).try_into().unwrap()),
        processing: number(before, "death_queue_flag") != 0,
    });
    state.native_work.death_processing.queue = number(before, "death_queue_flag") as u8;
    (state, content)
}

#[test]
fn field_condition_and_queue_returns_match_native_terminal_prefixes() {
    for case in fixture("field-terminals.json")["cases"].as_array().unwrap() {
        let source = &case["declarations"]["source_input"];
        let (mut state, content) = prepare(source);
        let (events, terminated) = match source["parent"].as_str().unwrap() {
            "collide" => {
                let result = retained_turn::collide_fields(
                    &mut state,
                    &content,
                    source["body"].as_i64().unwrap() as i16,
                )
                .unwrap_or_else(|error| panic!("{}: {error}", case["name"]));
                (result.events, result.flow == TurnFlow::Terminated)
            }
            "reduce" => {
                let result = retained_turn::reduction::apply(&mut state, &content)
                    .unwrap_or_else(|error| panic!("{}: {error}", case["name"]));
                (result.events, result.flow == TurnFlow::Terminated)
            }
            "emptyque" => {
                let result = retained_turn::queue::drain(&mut state, &content)
                    .unwrap_or_else(|error| panic!("{}: {error}", case["name"]));
                (result.events, result.flow == TurnFlow::Terminated)
            }
            "walking" => {
                saved_return(&mut state, &content);
                let caller: CombatReturn =
                    serde_json::from_value(json!({"WarriorMove":source["frame"].clone()})).unwrap();
                state.combat_story.as_mut().unwrap().return_to = Some(Box::new(caller));
                let events = crate::apply(Command::Continue, &mut state, &content)
                    .unwrap_or_else(|error| panic!("{}: {error}", case["name"]));
                let terminated = events
                    .iter()
                    .any(|event| matches!(event, Event::MissionEnded { defeated: true }));
                (events, terminated)
            }
            parent => panic!("unexpected native caller {parent}"),
        };
        assert_eq!(
            terminated,
            case["comparison"]["terminal"].as_bool().unwrap(),
            "{} terminal flow",
            case["name"]
        );
        verify(&state, &content, &events, case);
        let queue = state.native_work.retained_death_queue.as_ref().unwrap();
        assert_eq!(
            queue
                .entries
                .iter()
                .flat_map(|entry| entry.bytes())
                .collect::<Vec<_>>(),
            bytes(&case["memory"]["death_queue"]["hex"]),
            "{} queue entries",
            case["name"]
        );
        assert_eq!(
            queue.count,
            number(
                &json!({"count":case["memory"]["death_queue_count"]["hex"]}),
                "count"
            ) as usize
        );
        assert_eq!(
            state.native_work.death_processing.queue,
            number(
                &json!({"flag":case["memory"]["death_queue_flag"]["hex"]}),
                "flag"
            ) as u8
        );
    }
}
