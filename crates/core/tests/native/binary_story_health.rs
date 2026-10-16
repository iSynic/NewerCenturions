use crate::prelude::*;

use crate::native_common::patch_bytes as changes;
use centurions_core::story_health::heal_outdoors;
use centurions_core::story_health::party_health;

use crate::native_common::health_event as event;

#[test]
fn clinic_command_treatments_and_saved_return_match_original() {
    let content = crate::support::content();
    let checkpoint =
        std::env::temp_dir().join(format!("centurions-clinic-{}.json", std::process::id()));
    for case in crate::support::fixture("clinic.json").array("cases") {
        let mut state = crate::support::mission_session(&content);
        state.warriors = crate::support::warriors(&case.bytes("before"));
        state.party_currency = case.n("credits_before");
        state.random = RandomState::new(12);
        state.exploration.as_mut().unwrap().script =
            Some(crate::support::script_state(vec![0; 42], 8));
        state.exploration.as_mut().unwrap().presentation = Some(Presentation::Clinic {
            member: 0,
            percent: case.n("percent"),
        });
        save(&checkpoint, &state, &content).unwrap();
        let mut resumed = load(&checkpoint, &content).unwrap();
        let command = Command::ClinicControl {
            item: case.n("item"),
        };
        let before = state.clone();
        let result = apply(command.clone(), &mut state, &content);
        let restored = apply(command, &mut resumed, &content);
        if case.n::<i32>("credits_before") == 0 {
            assert!(
                matches!(result, Err(RuleError::Refused { feedback }) if feedback == vec![centurions_core::feedback::NativeFeedback::Warning { index: 50 }])
            );
            assert!(restored.is_err());
            assert_eq!(state, before);
        } else {
            assert_eq!(result.unwrap(), vec![Event::Sound { resource_id: -431 }]);
            assert!(restored.is_ok());
        }
        assert_eq!(state, resumed);
        assert_eq!(
            digest(&crate::support::record_bytes(&state.warriors)),
            digest(&case.bytes("after"))
        );
        assert_eq!(state.party_currency, case.n::<i32>("credits_after"));
        assert_eq!(
            state.random,
            RandomState {
                seed: case.n("seed_after"),
                draws: case.n("draws")
            }
        );
        let random = state.random.clone();
        for item in [7, 6, 11] {
            apply(Command::ClinicControl { item }, &mut state, &content).unwrap();
        }
        assert_eq!(state.random, random);
        assert!(state.exploration.as_ref().unwrap().presentation.is_none());
    }
    let mut state = crate::support::mission_session(&content);
    let position = state.exploration.as_mut().unwrap();
    position.presentation = Some(Presentation::Dialogue {
        id: 0,
        text: String::new(),
    });
    let mut record = vec![0; 42];
    record[8..10].copy_from_slice(&(-32_i16).to_be_bytes());
    record[24..26].copy_from_slice(&297_i16.to_be_bytes());
    position.script = Some(crate::support::script_state(record, 0));
    let random = state.random.clone();
    let events = apply(Command::Continue, &mut state, &content).unwrap();
    assert!(events.starts_with(&[
        Event::Sound { resource_id: 655 },
        Event::Sound { resource_id: 430 }
    ]));
    assert!(matches!(
        state.exploration.as_ref().unwrap().presentation,
        Some(Presentation::Clinic { percent: 0, .. })
    ));
    apply(Command::ClinicControl { item: 11 }, &mut state, &content).unwrap();
    assert_eq!(state.random, random);
    assert!(state.exploration.as_ref().unwrap().script.is_none());
    for case in crate::support::fixture("clinic.json").array("movement_exits") {
        let position = state.exploration.as_mut().unwrap();
        position.x = 20;
        position.y = 20;
        position.last_move = case.n::<Vec<i8>>("move").try_into().unwrap();
        position.presentation = Some(Presentation::Dialogue {
            id: 0,
            text: String::new(),
        });
        let mut record = vec![0; 42];
        record[8..10].copy_from_slice(&101_i16.to_be_bytes());
        record[5..7].fill(45);
        position.script = Some(crate::support::script_state(record, 0));
        let headers = position.event_headers.clone();
        apply(Command::Continue, &mut state, &content).unwrap();
        let position = state.exploration.as_ref().unwrap();
        assert_eq!((position.x, position.y), (case.n("x"), case.n("y")));
        assert!(position.script.is_none());
        assert_eq!(position.event_headers, headers);
        assert_eq!(state.random, random);
    }
    std::fs::remove_file(checkpoint).unwrap();
}

#[test]
fn outdoor_heal_and_party_callers_match_original_records_effect_order_and_rng() {
    let fixture: serde_json::Value = crate::support::fixture("story-health.json");
    let original = fixture.bytes("warriors");
    for (index, case) in fixture.array("cases").iter().enumerate() {
        let mut before = original.clone();
        changes(&mut before, &case["before_changes"]);
        assert_eq!(digest(&before), case["before_sha256"], "input {index}");
        let warriors: Vec<_> = crate::support::warriors(&before);
        let affected = case.bytes("affected_before");
        let context = HealthContext {
            unavailable: case.n::<i8>("unavailable_before"),
            selected_warrior: case.n::<usize>("member"),
            effect: 77,
            last_body: -7,
            marks_affected: case.n::<bool>("marks_affected"),
            affected: std::array::from_fn(|member| word(&affected, member * 2).unwrap()),
        };
        let mask_bytes = case.bytes("mask");
        let mask = std::array::from_fn(|member| word(&mask_bytes, member * 2).unwrap());
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let outcome = if case["kind"] == "heal" {
            heal_outdoors(
                &warriors,
                context.selected_warrior,
                case.n::<i16>("amount"),
                case.n::<bool>("animate"),
                &context,
                &mut random,
            )
        } else {
            party_health(
                &warriors,
                &case.bytes("extra"),
                case["kind"] == "all",
                &mask,
                &context,
                &mut random,
            )
        }
        .unwrap();
        let actual: Vec<_> = crate::support::record_bytes(&outcome.warriors);
        let mut expected = before.clone();
        changes(&mut expected, &case["after_changes"]);
        assert_eq!(actual, expected, "full warrior state {index}");
        crate::support::native_expect! {
        scalar outcome.context.unavailable => &case["unavailable_after"], "binary_story_health";
        scalar outcome.context.effect => &case["effect_after"], "binary_story_health";
        scalar outcome.context.last_body => &case["last_body_after"], "binary_story_health";
        }
        let actual_affected: Vec<_> = crate::support::word_bytes(&outcome.context.affected);
        assert_eq!(
            actual_affected,
            case.bytes("affected_after"),
            "affected {index}"
        );
        assert_eq!(
            outcome.events.iter().map(event).collect::<Vec<_>>(),
            case.array("events"),
            "events {index}"
        );
        assert_eq!(
            serde_json::to_value(outcome.amounts).unwrap(),
            case["amounts"],
            "amounts {index}"
        );
        crate::support::native_expect! {
        scalar random.seed => &case["seed_after"], "binary_story_health";
        scalar random.draws => &case["draws"], "binary_story_health";
        }
    }
}
