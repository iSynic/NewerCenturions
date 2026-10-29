//! Original scoped writes; question answers, EXTRA and moveparty services are controlled.
//! Requires the ignored capture.json below under the optional original-fixtures feature.
//! Later movement/time/Secrets writes, including443316, are outside these comparisons.
use crate::support::{self, NativeCase};
use centurions_core::{Command, Presentation, RandomState, ScriptState, apply};
use serde_json::Value;
use std::sync::Arc;

#[test]
fn boat_commands_match_original_condition_setters_and_question_writes() {
    let capture: Value = serde_json::from_slice(
        &std::fs::read(support::root().join("analysis/backlog-50/native-boat-01/capture.json"))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(capture.array("cases").len(), 8);
    for case in capture.array("cases") {
        let name = case.text("name");
        assert_eq!(case.text("status"), "completed", "{name}");
        let input = |address| {
            case["declarations"]
                .array("values")
                .iter()
                .find(|row| row.text("address") == address)
                .unwrap()
                .n::<i64>("value")
        };
        let memory = |name: &str| case["memory"][name].bytes("hex");
        let mut content = support::content();
        let mut state = support::mission_session(&content);
        state.random = RandomState::new(1);
        state.native_work.boat = input("0x442748") as u8;
        state.native_work.boat_facing = input("0x442740") as i8;
        state.native_work.shore_attempts = input("0x437a86") as i16;
        let position = state.exploration.as_mut().unwrap();
        (position.x, position.y) = (2, 2);
        position.presentation = None;
        position.script = None;
        position.schedule.fill(0);
        position.region_state.fill(0);
        position.region_state[520] = 2;
        position.events.fill(0);
        position.floor.fill(35);
        if name.starts_with("opcode103") {
            let extra = &case["declarations"]["services"][0];
            let tables = Arc::make_mut(&mut content.scenario.as_mut().unwrap().maps[0].shared);
            for (index, word) in extra.array("writes").iter().enumerate() {
                tables.combat.as_mut().unwrap().encounters[10 + index * 2..12 + index * 2]
                    .copy_from_slice(&word.n::<i16>("value").to_be_bytes());
            }
            state.field_controls.get_or_insert_default().camping = input("0x44274f") != 0;
            let mut record = case["declarations"]["memory"][1].bytes("hex");
            record[10..12].copy_from_slice(&9_i16.to_be_bytes());
            record[26..28].copy_from_slice(&409_i16.to_be_bytes());
            position.script = Some(ScriptState::scheduled(record, 0));
            position.presentation = Some(Presentation::Dialogue {
                id: 0,
                text: String::new(),
            });
            let events = apply(Command::Continue, &mut state, &content).unwrap();
            let passed = memory("instruction_cursor") == [0, 0];
            assert_eq!(
                support::event_sounds(&events).contains(&409),
                passed,
                "{name}"
            );
            assert_eq!(state.random, RandomState::new(1), "{name}");
        } else {
            let boarding = name.starts_with("boarding");
            let tile = if boarding { 147 } else { 35 };
            position.floor[183] = tile;
            content.scenario.as_mut().unwrap().maps[0].tile_properties =
                support::data("Data Swamp BD")[64..12064].to_vec();
            let events = apply(Command::Move { dx: 0, dy: 1 }, &mut state, &content).unwrap();
            let question = case
                .array("calls")
                .iter()
                .find(|call| call.text("id") == "original_question_controlled_answer");
            assert_eq!(
                state.exploration.as_ref().unwrap().presentation,
                question.map(|_| Presentation::BoatQuestion { boarding }),
                "{name}"
            );
            if boarding {
                // The capture stops before109a22's counter reset and decline walk.
                assert_eq!(
                    state.native_work.shore_attempts,
                    input("0x437a86") as i16
                );
            }
            if question.is_some() {
                let answer = case["declarations"]["services"][0]["registers"].n::<i16>("D0") != 0;
                apply(Command::Choose { answer }, &mut state, &content).unwrap();
            }
            let position = state.exploration.as_ref().unwrap();
            if !boarding {
                assert_eq!(
                    position.floor[182].to_be_bytes().to_vec(),
                    memory("tile_current"),
                    "{name}"
                );
                assert_eq!(
                    state
                        .native_work
                        .shore_attempts
                        .to_be_bytes()
                        .to_vec(),
                    memory("counter"),
                    "{name}"
                );
            } else {
                assert_eq!(
                    position.floor[183].to_be_bytes().to_vec(),
                    memory("tile_target"),
                    "{name}"
                );
            }
            assert_eq!(
                position.last_move.map(|v| v as u8).to_vec(),
                memory("direction"),
                "{name}"
            );
            let sounds = support::event_sounds(&events);
            assert_eq!(sounds.contains(&-148), !boarding, "{name}");
            assert_eq!(sounds.contains(&20005), question.is_some(), "{name}");
        }
        assert_eq!(
            [state.native_work.boat],
            memory("boat").as_slice(),
            "{name}"
        );
        assert_eq!(
            [state.native_work.boat_facing as u8],
            memory("facing").as_slice(),
            "{name}"
        );
        if name.starts_with("opcode103") {
            assert_eq!(
                state
                    .native_work
                    .shore_attempts
                    .to_be_bytes()
                    .to_vec(),
                memory("counter"),
                "{name}"
            );
        }
    }
}
