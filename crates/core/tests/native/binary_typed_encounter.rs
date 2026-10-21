use crate::{native_common, prelude::*, support};

use centurions_core::typed_encounter::{
    PasswordSelection, QuestionPlan, TypedContinuation, continuation, finish_attempt,
    question_labels, question_plan, select_password,
};

fn item_state(state: &mut Session, case: &Value) {
    native_common::restore_party(state, &serde_json::json!({"party": case["after"]}));
    state.ammunition_work.item = Some(case.bytes("item_work"));
    state.loaded_item_field32 = Some(word(&case.bytes("item_work"), 32).unwrap());
}

#[test]
fn password_prefix_and_case_selection_match_original_comparison() {
    let fixture = support::fixture("typed-encounter.json");
    let native =
        centurions_content::native_text::NativeText::load(&content(), std::path::Path::new("."))
            .unwrap();
    let lowercase = native.bytes(0x3ffc50, 256).unwrap();
    assert_eq!(
        lowercase,
        support::fixture("case-fold.json").bytes("lowercase256")
    );
    for case in fixture.array("password_cases") {
        let record = case.bytes("record");
        let input = case.bytes("input");
        assert_eq!(
            select_password(&record, &input, lowercase).unwrap(),
            PasswordSelection::Selected(case.n::<i8>("selector")),
            "{}",
            String::from_utf8_lossy(&input)
        );
        assert_eq!(
            select_password(&record, b"", lowercase).unwrap(),
            PasswordSelection::Retry
        );
    }
    let mut record = fixture.array("password_cases")[0].bytes("record");
    record[476..480].copy_from_slice(&[2, 0x8a, b'x', 0]);
    let content = content();
    let mut state = support::mission_session(&content);
    state.typed_encounters = content.map(0).unwrap().typed_encounters.clone();
    state.typed_encounters[..record.len()].copy_from_slice(&record);
    let mut script = vec![0; 42];
    script[8..10].copy_from_slice(&5_i16.to_be_bytes());
    let position = state.exploration.as_mut().unwrap();
    position.script = Some(support::script_state(script, 0));
    position.presentation = Some(Presentation::Dialogue {
        id: 0,
        text: String::new(),
    });
    apply(Command::Continue, &mut state, &content).unwrap();
    support::saved_command(
        Command::SubmitText { text: "ÄX".into() },
        &mut state,
        &content,
    );
    assert!(matches!(
        state.exploration.as_ref().unwrap().presentation,
        Some(Presentation::Dialogue { id: 507, .. })
    ));
}

#[test]
fn typed_branch_and_last_attempt_match_complete_original_script() {
    let fixture = support::fixture("typed-encounter.json");
    for case in fixture.array("dispatch_cases") {
        let record = case.bytes("record");
        let before = case.bytes("before");
        let after = case.bytes("after");
        let plan = continuation(
            &record,
            case.n::<i8>("selected"),
            case.n::<i16>("remaining"),
        )
        .unwrap();
        let TypedContinuation::Branch { selector, body } = plan else {
            panic!("Expected authored branch");
        };
        support::expect(i64::from(selector), &case["effective_selector"]);
        assert_eq!(&after[8..40], body);
        assert_eq!(&after[..8], &before[..8]);
        assert_eq!(&after[40..], &before[40..]);
    }
    for case in fixture.array("repeat_cases") {
        assert_eq!(
            finish_attempt(case.n::<i16>("before")),
            (case.n::<i16>("after"), case.n::<bool>("repeat"))
        );
    }
    let content = content();
    let items = support::fixture("encounter-items.json");
    let inert = support::fixture("encounter-inert-items.json");
    let scripted = support::fixture("encounter-script-items.json");
    let modifiers = support::fixture("item-modifiers.json");
    let walking_items = support::fixture("walking-encounter-item.json");
    let calendar_items = support::fixture("calendar-encounter-item.json");
    let regional_return = support::fixture("regional-item-return.json");
    for (case, regional, returned) in items
        .array("cases")
        .iter()
        .chain(inert.array("cases"))
        .chain(scripted.array("cases"))
        .chain(modifiers.array("cases"))
        .chain(walking_items.array("cases"))
        .chain(calendar_items.array("cases"))
        .map(|case| (case, false))
        .chain(
            calendar_items
                .array("cases")
                .iter()
                .map(|case| (case, true)),
        )
        .map(|(case, regional)| (case, regional, 0))
        // Source-derived callers reuse the native consumption input and output.
        .chain((1..=2).map(|caller| (&scripted.array("cases")[0], false, caller)))
    {
        let walking = case["caller"] == "walking";
        let calendar = case["caller"] == "calendar" && !regional;
        let returning = walking || calendar || regional || returned != 0;
        let mut content = content.clone();
        if case.get("definition").is_some() {
            content.items[case.n::<usize>("item")] = case.bytes("definition");
        }
        let mut state = support::mission_session(&content);
        state.warriors = support::warriors(&case["before"]);
        if case.get("mask_before").is_some() && case.get("warning").is_none() && !returning {
            let manual = support::fixture("manual-encounter.json");
            let selected = manual
                .array("cases")
                .iter()
                .find(|case| {
                    case["variant"] == 107 && case["script"] == 250 && case["seed_before"] == 12
                })
                .unwrap();
            let position = state.exploration.as_mut().unwrap();
            position.region_state = selected.bytes("before");
            position.x = selected.n("x");
            position.y = selected.n("y");
            apply(Command::Encounter, &mut state, &content).unwrap();
            assert!(
                state
                    .exploration
                    .as_ref()
                    .unwrap()
                    .script
                    .as_ref()
                    .unwrap()
                    .item_return
            );
            assert_eq!(
                state.exploration.as_ref().unwrap().region_state,
                selected.bytes("after")
            );
            assert_eq!(state.random.seed, selected.n::<u32>("seed_after"));
            assert_eq!(state.random.draws, selected.n::<u64>("draws"));
        } else if regional {
            let map = &mut content.scenario.as_mut().unwrap().maps[0];
            let record = &mut std::sync::Arc::make_mut(&mut map.shared).door_scripts[..42];
            record.fill(0);
            record[8..10].copy_from_slice(&5_i16.to_be_bytes());
            record[24..26].copy_from_slice(&case.n::<i16>("encounter").to_be_bytes());
            map.tile_properties[..40].fill(0);
            map.tile_properties[2..4].copy_from_slice(&1_i16.to_be_bytes());
            map.minutes_per_tick = 0;
            let position = state.exploration.as_mut().unwrap();
            position.map = 0;
            position.x = regional_return.n::<i32>("header_x") - 1;
            position.y = regional_return.n("header_y");
            position.floor.fill(0);
            position.event_headers.fill(0);
            position.events.fill(0);
            position.events[10 * 90 + 11] = 1;
            position.region_state.fill(0);
            for (offset, value) in [
                (0, 11_i16),
                (2, 10),
                (4, 11),
                (6, 10),
                (160, 10000),
                (400, -100),
            ] {
                position.region_state[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
            }
            apply(Command::Move { dx: 1, dy: 0 }, &mut state, &content).unwrap();
            assert_eq!(
                state.movement_continuation,
                Some(centurions_core::MovementContinuation::RegionalReturn { header: true })
            );
        } else {
            let mut script = vec![0; 42];
            script[8..10].copy_from_slice(&5_i16.to_be_bytes());
            script[24..26].copy_from_slice(&case.n::<i16>("encounter").to_be_bytes());
            let position = state.exploration.as_mut().unwrap();
            let mut script = support::script_state(script, 0);
            script.item_return = case.get("mask_before").is_some() && !returning;
            if walking {
                script.scheduled = false;
                script.origin = Some(centurions_core::ScriptOrigin {
                    map: position.map,
                    x: position.x,
                    y: position.y,
                    level: state.scenario_level,
                });
            }
            position.script = Some(script);
            position.presentation = Some(Presentation::Dialogue {
                id: 0,
                text: String::new(),
            });
            if calendar {
                position.day = 2;
                position.hour = 0;
                position.minute = 0;
                state.time_continuation = Some(centurions_core::TimeContinuation {
                    remaining_ticks: 0,
                    minutes_per_tick: 5,
                    schedule_cursor: position.schedule.len() / 40,
                });
            }
            apply(Command::Continue, &mut state, &content).unwrap();
        }
        if returned != 0 {
            if returned == 1 {
                state.combat_story = Some(centurions_core::combat::runtime::DeathStory {
                    shared_dispatch: true,
                    child_returned: true,
                    ..Default::default()
                });
            } else {
                state.round_dispatch = Some(Default::default());
                state.scripted_cleanup = Some(centurions_core::story_cleanup::Stage::Complete);
            }
        }
        if case.get("mask_before").is_some() {
            state.party_mask.copy_from_slice(&case.words("mask_before"));
        }
        assert!(matches!(
            state.exploration.as_ref().unwrap().presentation,
            Some(Presentation::Typed { .. })
        ));
        if case.get("encounter_active") == Some(&serde_json::Value::Bool(false)) {
            let position = state.exploration.as_mut().unwrap();
            position.script = None;
            position.presentation = None;
        }
        if let Some(warning) = case.get("warning") {
            if case.get("active_body").is_some() {
                let mut combat = support::battle();
                combat.order[1] = case.n("active_body");
                state.combat = Some(combat);
                state.phase = Phase::Combat;
            }
            let mut before = state.clone();
            let error = apply(
                Command::WithPreferences {
                    command: Box::new(Command::UseItem {
                        warrior: case.n("member"),
                        slot: case.n("slot"),
                    }),
                    preferences: centurions_core::CommandPreferences::default(),
                },
                &mut state,
                &content,
            )
            .unwrap_err();
            if warning != 10 && warning != 17 {
                item_state(&mut before, case);
            }
            let feedback = match (warning.as_i64().unwrap(), error) {
                (24 | 52 | 139 | 148, RuleError::RefusedAfterEffects { feedback }) => feedback,
                (10 | 17, RuleError::Refused { feedback }) => feedback,
                (_, error) => panic!("Expected original item refusal: {error:?}"),
            };
            let warnings = case.get("warnings").map_or_else(
                || vec![warning.as_i64().unwrap() as i16],
                |_| case.n::<Vec<i16>>("warnings"),
            );
            let mut expected_feedback = if case.get("dialog_open").is_some() {
                case.n::<Vec<i16>>("sounds")
                    .into_iter()
                    .map(
                        |resource_id| centurions_core::feedback::NativeFeedback::Sound {
                            resource_id,
                        },
                    )
                    .collect::<Vec<_>>()
            } else {
                vec![]
            };
            expected_feedback.extend(
                warnings
                    .into_iter()
                    .map(|index| centurions_core::feedback::NativeFeedback::Warning { index }),
            );
            assert_eq!(feedback, expected_feedback);
            support::same_session(&state, &before);
            support::checkpoint(&state, &content, "encounter-item-refusal");
            support::native_expect! {
                bytes &support::record_bytes(&state.warriors) => &case.bytes("after"), "refused encounter item";
            };
            continue;
        }
        if case.get("dialog_open").is_some() {
            let mut expected = state.clone();
            item_state(&mut expected, case);
            let events = support::saved_command(
                Command::UseItem {
                    warrior: case.n("member"),
                    slot: case.n("slot"),
                },
                &mut state,
                &content,
            );
            support::same_captured_session(&state, &expected);
            assert_eq!(support::event_sounds(&events), case.n::<Vec<i16>>("sounds"));
            assert!(
                events
                    .iter()
                    .any(|event| matches!(event, Event::ItemUpgraded { .. }))
            );
            support::checkpoint(&state, &content, "modifier-after");
            continue;
        }
        let selector = case.n::<i8>("selector");
        let before = state.clone();
        let mut resumed = support::checkpoint(&state, &content, "encounter-item-before");
        if regional {
            let mut cancelled = resumed.clone();
            apply(Command::CancelMenu, &mut cancelled, &content).unwrap();
            let position = cancelled.exploration.as_ref().unwrap();
            assert_eq!(
                (position.x, position.y, position.last_move),
                (9, 11, [0; 2])
            );
            assert!(cancelled.movement_continuation.is_none());
            assert_eq!(cancelled.random, before.random);
        }
        if selector == -1 && !returning && case.n::<Vec<i16>>("sounds") == vec![655] {
            let path = std::env::temp_dir().join(format!(
                "centurions-item-return-{}.json",
                std::process::id()
            ));
            save(&path, &state, &content).unwrap();
            let mut legacy: serde_json::Value =
                serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            legacy["schema_version"] = 5.into();
            support::legacy_save(&mut legacy);
            support::remove_field(&mut legacy["session"], "inventory_record");
            std::fs::write(&path, serde_json::to_vec(&legacy).unwrap()).unwrap();
            assert!(load(&path, &content).is_err());
            support::remove_field(
                &mut legacy["session"]["exploration"]["script"],
                "item_return",
            );
            std::fs::write(&path, serde_json::to_vec(&legacy).unwrap()).unwrap();
            let mut expected = state.clone();
            expected.inventory_record = None;
            expected.npc_backing = Default::default();
            expected
                .exploration
                .as_mut()
                .unwrap()
                .script
                .as_mut()
                .unwrap()
                .item_return = false;
            support::same_captured_session(&load(&path, &content).unwrap(), &expected);
            std::fs::remove_file(path).unwrap();
        }
        let mut events = replay_command(
            Command::UseItem {
                warrior: case.n("member"),
                slot: case.n("slot"),
            },
            &mut state,
            &mut resumed,
            &content,
        );
        if returned == 1 {
            events.extend(replay_command(
                Command::Continue,
                &mut state,
                &mut resumed,
                &content,
            ));
        }
        support::native_expect! {
            bytes &support::record_bytes(&state.warriors) => &case.bytes("after"), "encounter item party";
        };
        assert_eq!(state.random, before.random);
        assert_eq!(state.party_mask.as_slice(), case.words("mask"));
        assert_eq!(
            state.ammunition_work.item.as_ref().unwrap(),
            &case.bytes("item_work")
        );
        if returning {
            if returned == 0 {
                assert_eq!(case.n::<i16>("caller_return"), -1);
                assert_eq!(case.n::<i16>("caller_stack_delta"), 14);
            }
            let mut expected = before;
            item_state(&mut expected, case);
            expected.selected_warrior = case.n("member");
            expected.party_mask.copy_from_slice(&case.words("mask"));
            if returned != 0 {
                expected.combat_story = None;
                expected.round_dispatch = None;
                expected.scripted_cleanup = None;
            }
            let position = expected.exploration.as_mut().unwrap();
            position.script = None;
            position.presentation = None;
            if regional {
                assert_eq!(regional_return.n::<i16>("regional_return"), -1);
                assert_eq!(regional_return.n::<i16>("regional_stack_delta"), 16);
                position.last_move = [regional_return.n("last_dx"), regional_return.n("last_dy")];
                expected.movement_continuation = None;
            }
            if calendar {
                expected.time_continuation = None;
                assert!(events.contains(&Event::TimeAdvanced {
                    day: 2,
                    hour: 0,
                    minute: 0
                }));
            }
            support::same_captured_session(&state, &expected);
            let mut sounds = case.n::<Vec<i16>>("sounds");
            if returned == 1 {
                sounds.push(20);
            }
            assert_eq!(support::event_sounds(&events), sounds);
            assert!(!events.contains(&Event::StoryPresented));
            support::checkpoint(&state, &content, "walking-item-after");
            continue;
        }
        let id = if selector == -1 {
            assert_eq!(case.n::<i16>("script"), 216);
            assert_eq!(case.n::<i16>("return_code"), -316);
            let args = case.bytes("script_args");
            assert_eq!(word(&args, 10).unwrap(), 216);
            let mut sounds = case.n::<Vec<i16>>("sounds");
            // The capture stops before newland; authored script216 starts with sound406.
            sounds.push(406);
            assert_eq!(support::event_sounds(&events), sounds);
            assert!(
                !state
                    .exploration
                    .as_ref()
                    .unwrap()
                    .script
                    .as_ref()
                    .unwrap()
                    .item_return
            );
            851
        } else {
            let selected = selector as usize - 1;
            let record = case.bytes("record");
            let dialogue = (0..8).find(|i| record[selected * 8 + i] == 1).unwrap();
            word(&record, 32 + selected * 16 + dialogue * 2).unwrap()
        };
        assert!(
            matches!(state.exploration.as_ref().unwrap().presentation, Some(Presentation::Dialogue { id: actual, .. }) if actual == id)
        );
        assert!(events.contains(&Event::StoryPresented));
        support::checkpoint(&state, &content, "encounter-item-after");
    }
    for case in support::fixture("encounter-actions.json").array("cases") {
        let mut state = support::mission_session(&content);
        let mut script = vec![0; 42];
        script[8..10].copy_from_slice(&5_i16.to_be_bytes());
        script[24..26].copy_from_slice(&case.n::<i16>("encounter").to_be_bytes());
        let position = state.exploration.as_mut().unwrap();
        position.script = Some(support::script_state(script, 0));
        position.presentation = Some(Presentation::Dialogue {
            id: 0,
            text: String::new(),
        });
        apply(Command::Continue, &mut state, &content).unwrap();
        let before = state.clone();
        assert!(
            apply(Command::BeginEncounterActions, &mut state, &content)
                .unwrap()
                .is_empty()
        );
        let pending = state.clone();
        assert!(apply(Command::Continue, &mut state, &content).is_err());
        support::same_session(&state, &pending);
        let file = std::env::temp_dir().join(format!(
            "centurions-encounter-action-migration-{}.json",
            std::process::id()
        ));
        save(&file, &state, &content).unwrap();
        let mut legacy: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
        legacy["schema_version"] = 5.into();
        support::legacy_save(&mut legacy);
        std::fs::write(&file, serde_json::to_vec(&legacy).unwrap()).unwrap();
        assert!(load(&file, &content).is_err());
        std::fs::remove_file(file).unwrap();
        for step in case.array("steps") {
            let mut resumed = support::checkpoint(&state, &content, "encounter-action-pending");
            let command = Command::EncounterActionControl {
                item: step.n("control"),
            };
            let effects: Vec<_> = step
                .array("effects")
                .iter()
                .filter_map(|effect| {
                    effect.get("sound").map(|sound| Event::Sound {
                        resource_id: sound.as_i64().unwrap() as i16,
                    })
                })
                .collect();
            if step
                .array("effects")
                .iter()
                .any(|effect| effect.get("warning").is_some())
            {
                let prior = state.clone();
                assert!(
                    matches!(apply(command.clone(), &mut state, &content), Err(RuleError::Refused { feedback })
                    if feedback == vec![centurions_core::feedback::NativeFeedback::Warning { index: 58 }])
                );
                assert!(apply(command, &mut resumed, &content).is_err());
                support::same_session(&state, &prior);
                support::same_session(&resumed, &prior);
            } else {
                let events = replay_command(command, &mut state, &mut resumed, &content);
                if !step.n::<bool>("closed") || !step.n::<bool>("accepted") {
                    assert_eq!(events, effects);
                }
            }
            assert_eq!(state.random, before.random);
            if !step.n::<bool>("closed") {
                assert_eq!(
                    state.encounter_actions.unwrap().to_vec(),
                    step.words("selected")
                        .iter()
                        .map(|value| *value != 0)
                        .collect::<Vec<_>>()
                );
            } else {
                assert!(state.encounter_actions.is_none());
                if !step.n::<bool>("accepted") {
                    support::same_session(&state, &before);
                } else if step.n::<i8>("selector") == 1 {
                    let position = state.exploration.as_ref().unwrap();
                    let TypedContinuation::Branch { body, .. } =
                        continuation(&case.bytes("record"), 1, 99).unwrap()
                    else {
                        panic!("native branch");
                    };
                    assert_eq!(&position.script.as_ref().unwrap().record[8..40], body);
                    assert!(matches!(
                        position.presentation,
                        Some(Presentation::Dialogue { id: 1123, .. })
                    ));
                } else {
                    let position = state.exploration.as_ref().unwrap();
                    assert!(matches!(
                        position.presentation,
                        Some(Presentation::Typed { id: 10, .. })
                    ));
                    assert_eq!(
                        position
                            .script
                            .as_ref()
                            .unwrap()
                            .menu
                            .as_ref()
                            .unwrap()
                            .remaining,
                        98
                    );
                }
            }
        }
        support::checkpoint(&state, &content, "encounter-action-after");
    }
}

#[test]
fn question_condition_and_continuation_match_original_blocks() {
    let fixture = support::fixture("typed-encounter.json");
    for case in fixture.array("question_cases") {
        let extra = case.bytes("extra");
        let plan = question_plan(&extra, case.n::<bool>("answer")).unwrap();
        let (yes, no) = question_labels(&extra).unwrap();
        assert_eq!((yes, no), (185, 186));
        match plan {
            QuestionPlan::Continue => {}
            QuestionPlan::MoveAway => {}
            QuestionPlan::ReplaceScript(id) => support::expect(i64::from(id), &case["loaded"]),
            QuestionPlan::MenuBranch(branch) | QuestionPlan::TypedBranch(branch) => {
                let record = case.bytes("branch_record");
                let expected =
                    centurions_core::story::dispatch_menu_branch(&record, branch as i8 + 1)
                        .unwrap();
                assert_eq!(&case.bytes("after")[8..40], expected);
            }
            QuestionPlan::EndScript => assert_eq!(case["cursor"], 9),
        }
        assert_eq!(word(&extra, 8).unwrap(), 186);
    }
}

#[test]
fn typed_cancel_and_header_persistence_preserve_original_boundaries() {
    let fixture = support::fixture("typed-encounter.json");
    let record = bytes(fixture["password_cases"][0]["record"].as_str().unwrap());
    assert_eq!(
        continuation(&record, 0, 2).unwrap(),
        TypedContinuation::Cancel
    );
    assert_eq!(
        continuation(&record, -1, 2).unwrap(),
        TypedContinuation::Abort
    );
    assert!(continuation(&record, -4, 2).is_err());
    let cancel = &fixture["cancel_case"];
    let menus = support::fixture("encounter-choice.json");
    let movement = &support::fixture("move-away.json")["cases"][0];
    let content = content();
    for (typed, scheduled, manual, native) in [
        (true, false, false, cancel),
        (true, true, false, cancel),
        (false, false, false, &menus["cancel_case"]),
        (false, true, true, &menus["cancel_case"]),
    ] {
        let read = |key: &str| {
            native.bytes(&if typed {
                key.into()
            } else {
                format!("{key}_script")
            })
        };
        let id = if typed {
            native.n("save_id")
        } else {
            native.n("save_menu")
        };
        let mut state = support::mission_session(&content);
        let position = state.exploration.as_mut().unwrap();
        position.x = movement.n("x");
        position.y = movement.n("y");
        position.last_move = [movement.n("dx"), movement.n("dy")];
        let mut script = support::script_state(read("working"), 0);
        script.scheduled = scheduled;
        script.item_return = manual;
        script.map_slot = 2;
        script.menu = Some(centurions_core::MenuContinuation {
            id,
            remaining: 1,
            original: read("original"),
            typed,
        });
        position.event_headers[84..126].copy_from_slice(&read("working"));
        position.script = Some(script);
        position.presentation = Some(if typed {
            Presentation::Typed {
                id,
                text: String::new(),
            }
        } else {
            Presentation::Menu {
                id,
                text: String::new(),
                labels: vec![],
            }
        });
        let mut expected = state.clone();
        let position = expected.exploration.as_mut().unwrap();
        let backstep = typed || !manual;
        if backstep {
            position.x = movement.n("x_after");
            position.y = movement.n("y_after");
        }
        position.presentation = None;
        position.script = None;
        if !scheduled {
            position.event_headers[84..126].copy_from_slice(&read("persisted"));
        }
        let events = support::saved_command(Command::CancelMenu, &mut state, &content);
        support::same_captured_session(&state, &expected);
        assert_eq!(
            events,
            if backstep {
                vec![Event::Moved {
                    x: movement.n("x_after"),
                    y: movement.n("y_after"),
                }]
            } else {
                vec![]
            }
        );
        support::checkpoint(&state, &content, "typed-cancel-after");
    }
}
