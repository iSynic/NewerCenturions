//! Native combat-zero death aliases and the terminal party-loss caller boundary.
use super::detached_attacks::{compare, initial, saved_return};
use crate::combat::runtime::reaction_frame::ReactionFrame;
use crate::combat::runtime::{detached_attack, outdoor_death};
use crate::feedback::NativeFeedback;
use crate::test_support::{bytes, content, fixture};
use crate::{Command, Content, Event, Session};
use serde_json::Value;

pub(super) fn prepare(source: &Value) -> (Session, Content) {
    let mut content = content();
    if let Some(overrides) = source["custom_row_overrides"].as_array() {
        for row in overrides {
            assert_eq!(row["table"], "item_definitions");
            content.items[row["index"].as_u64().unwrap() as usize] = bytes(&row["hex"]);
        }
    }
    let mut state = initial(source, &content);
    state.native_work.character_bare_hand_bonus =
        source["character_work_bonus"].as_i64().unwrap_or(0) as i16;
    let before = &source["before"];
    assert!(
        bytes(&before["editor_backup"])
            .iter()
            .all(|byte| *byte == 0)
    );
    state.exploration.as_mut().unwrap().event_headers = bytes(&before["event_headers"]);
    state
        .native_work
        .shop_work
        .copy_file(&bytes(&before["current_shop_work"]));
    if before.get("death_queue").is_some() {
        assert!(bytes(&before["death_queue"]).iter().all(|byte| *byte == 0));
        assert!(
            bytes(&before["death_queue_count"])
                .iter()
                .all(|byte| *byte == 0)
        );
        assert!(
            bytes(&before["death_queue_pointer"])
                .iter()
                .all(|byte| *byte == 0)
        );
        assert_eq!(bytes(&before["death_queue_flag"]), [0]);
        state.native_work.retained_death_queue = Some(Default::default());
        state.native_work.death_processing.queue = 0;
        state.native_work.turn_round = Some(source["turn_round"].as_u64().unwrap() as i16);
    }
    (state, content)
}

pub(super) fn verify(state: &Session, content: &Content, events: &[Event], case: &Value) {
    compare(state, events, case);
    assert_eq!(
        state.native_work.shop_work.record(),
        bytes(&case["memory"]["current_shop_work"]["hex"]),
        "{} retained current shop",
        case["name"]
    );
    for first in [0, 400, 200, 600, 800] {
        let actual: Vec<_> = content.items[first..first + 200]
            .iter()
            .flat_map(|item| item.iter().copied())
            .collect();
        assert_eq!(
            actual,
            bytes(&case["memory"][format!("definition_bank_{first}")]["hex"]),
            "{} definition bank {first}",
            case["name"]
        );
    }
    if case["comparison"]["terminal"] == true {
        assert!(state.warriors.is_empty() && state.exploration.is_none());
        assert!(state.combat_story.is_none());
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, Event::MissionEnded { defeated: true }))
                .count(),
            1
        );
    } else {
        assert_eq!(
            state.exploration.as_ref().unwrap().event_headers,
            bytes(&case["memory"]["event_headers"]["hex"])
        );
    }
    let statuses: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            Event::Feedback(NativeFeedback::OutdoorBodyStatus { permanent, .. }) => {
                Some(*permanent)
            }
            _ => None,
        })
        .collect();
    let native: Vec<_> = case["calls"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|call| call["id"] == "DrawString")
        .filter_map(
            |call| match call["arguments"]["pointer"].as_str().unwrap() {
                "00403286" => Some(false),
                "00403294" => Some(true),
                _ => None,
            },
        )
        .collect();
    assert_eq!(statuses, native, "{} original status label", case["name"]);
}

#[test]
fn outdoor_deaths_match_native_aliases_work_rng_and_terminal_returns() {
    let captures = fixture("outdoor-deaths.json");
    for case in captures["cases"].as_array().unwrap() {
        let source = &case["declarations"]["source_input"];
        let (mut state, content) = prepare(source);
        let terminal = case["comparison"]["terminal"] == true;
        let parent = source["parent"].as_str().unwrap();
        if parent == "checkforenemy-then-getup" {
            saved_return(&mut state, &content);
            let events = crate::apply(Command::Continue, &mut state, &content).unwrap();
            verify(&state, &content, &events, case);
            let queue = state.native_work.retained_death_queue.as_ref().unwrap();
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
                u16::from_be_bytes(
                    bytes(&case["memory"]["death_queue_count"]["hex"])
                        .try_into()
                        .unwrap()
                ) as usize
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
                state.native_work.death_processing.queue,
                bytes(&case["memory"]["death_queue_flag"]["hex"])[0]
            );
            assert_eq!(state.bandage_ready, 1);
            continue;
        }
        let (returned, terminated, events) = match parent {
            "killbody-outdoor-npc" => {
                let mut events = vec![];
                let terminated = outdoor_death::defeat(
                    &mut state,
                    &content,
                    source["body"].as_i64().unwrap() as i16,
                    &mut events,
                )
                .unwrap();
                (false, terminated, events)
            }
            "attack2" => {
                let outcome = detached_attack::run(
                    &mut state,
                    &content,
                    crate::combat::runtime::reaction_calls::AttackCall {
                        attacker: source["attacker"].as_i64().unwrap() as i16,
                        recipient: source["target"].as_i64().unwrap() as i16,
                        index: source["index"].as_u64().unwrap() as usize,
                        retained_actor: 10,
                    },
                )
                .unwrap();
                (outcome.returned, outcome.terminated, outcome.events)
            }
            "checkforenemy-warrior-guard" => {
                let outcome = ReactionFrame::new(10, 0, (1, 0))
                    .unwrap()
                    .finish_after_teardown(&mut state, &content)
                    .unwrap();
                (outcome.interrupted, outcome.terminated, outcome.events)
            }
            _ => panic!("unrecognized native caller {parent}"),
        };
        assert_eq!(terminated, terminal, "{} terminal boundary", case["name"]);
        if !terminal && parent != "killbody-outdoor-npc" {
            assert_eq!(
                returned,
                case["registers"]["D0"].as_u64().unwrap() & 0xffff != 0,
                "{} caller return",
                case["name"]
            );
        }
        verify(&state, &content, &events, case);
        // Returning lethal guards also enter getup; the joined native case
        // supplies its queue and next initiative slot independently.
        if parent == "checkforenemy-warrior-guard" && terminal {
            let (mut saved, content) = prepare(source);
            saved_return(&mut saved, &content);
            let events = crate::apply(Command::Continue, &mut saved, &content).unwrap();
            verify(&saved, &content, &events, case);
        }
        if parent == "checkforenemy-warrior-guard" && !terminal {
            let (mut missing, content) = prepare(source);
            let npc = missing.native_work.npc_backing.slot(0).unwrap().clone();
            *missing.native_work.npc_backing.slot_mut(55).unwrap() = npc;
            missing.native_work.selected_enemy = 55;
            missing.native_work.initiative_work.order[0] = 65;
            saved_return(&mut missing, &content);
            let before = missing.clone();
            assert!(matches!(
                crate::apply(Command::Continue, &mut missing, &content),
                Err(crate::RuleError::Unsupported {
                    action: "combat.outdoor_health",
                    dependency: "outdoor death stamina alias beyond native A5 allocation"
                })
            ));
            assert_eq!(missing, before, "late unavailable alias changed state");
        }
    }
}
