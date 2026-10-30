//! Independent native hourly/calendar records, RNG and continuation comparisons.
use super::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct Fixture {
    cases: Vec<Case>,
}
#[derive(Deserialize)]
struct Case {
    input: String,
    output: String,
    fatigue_before: i16,
    fatigue_after: i16,
    hour_after: i16,
    minute_after: i16,
    #[serde(default)]
    extra: Option<String>,
    #[serde(default)]
    controls: String,
    #[serde(default)]
    allies_before: String,
    #[serde(default)]
    allies_after: String,
}
pub(super) fn compare_hourly() {
    for source in [
        include_str!("../../../../evidence/fixtures/time.json"),
        include_str!("../../../../evidence/fixtures/hourly-energy-controls.json"),
        include_str!("../../../../evidence/fixtures/hourly-energy-reduction.json"),
    ] {
        let fixture: Fixture = serde_json::from_str(source).unwrap();
        for case in fixture.cases {
            let mut content = calendar_content(vec![]);
            let mut state = calendar_state(&content, &case.input, 12, 55);
            state.exploration.as_mut().unwrap().fatigue = case.fatigue_before;
            if let Some(extra) = &case.extra {
                state.holdover = decode(&case.allies_before);
                state.held_over = 6;
                let tables = std::sync::Arc::make_mut(
                    &mut content.scenario.as_mut().unwrap().maps[0].shared,
                );
                tables.combat = Some(centurions_content::combat::CombatContent {
                    encounters: decode(extra),
                    battles: vec![],
                    monsters: vec![],
                    race_moves: vec![],
                    attacks: vec![],
                });
                let mut record = vec![0; 42];
                put(&mut record, 8, 69);
                state.exploration.as_mut().unwrap().script =
                    Some(ScriptState::scheduled(record, 0));
                state.exploration.as_mut().unwrap().presentation = Some(Presentation::Dialogue {
                    id: 0,
                    text: String::new(),
                });
                apply(Command::Continue, &mut state, &content).unwrap();
                assert_eq!(
                    [
                        state.psi_disabled,
                        state.energy_recovery_disabled,
                        state.enemy_spell_disabled
                    ],
                    decode(&case.controls).as_slice(),
                );
                let path = std::env::temp_dir()
                    .join(format!("centurions-energy-{}.json", std::process::id()));
                save(&path, &state, &content).unwrap();
                let mut restored = load(&path, &content).unwrap();
                let events = apply(Command::Move { dx: 1, dy: 0 }, &mut state, &content).unwrap();
                assert_eq!(
                    apply(Command::Move { dx: 1, dy: 0 }, &mut restored, &content).unwrap(),
                    events
                );
                assert_eq!(state, restored);
                fs::remove_file(path).unwrap();
                assert_eq!(state.holdover, decode(&case.allies_after));
            } else {
                advance(&mut state, &content, 1, 5).unwrap();
            }
            let position = state.exploration.unwrap();
            assert_eq!(
                (position.hour, position.minute, position.fatigue),
                (case.hour_after, case.minute_after, case.fatigue_after),
            );
            assert_eq!(
                state
                    .warriors
                    .into_iter()
                    .flat_map(|warrior| warrior.bytes)
                    .collect::<Vec<_>>(),
                decode(&case.output),
            );
            assert_eq!(state.random.draws, 0);
        }
    }
}

pub(super) fn compare_calendar() {
    let fixture = crate::test_support::fixture("calendar.json");
    for case in fixture["cases"].as_array().unwrap() {
        let content = calendar_content(decode(case["schedule_before"].as_str().unwrap()));
        let mut state = calendar_state(
            &content,
            case["input"].as_str().unwrap(),
            case["hour_before"].as_i64().unwrap() as i16,
            case["minute_before"].as_i64().unwrap() as i16,
        );
        let events = advance(
            &mut state,
            &content,
            case["ticks"].as_u64().unwrap() as u8,
            5,
        )
        .unwrap();
        let position = state.exploration.unwrap();
        assert_eq!(
            (
                position.day,
                position.hour,
                position.minute,
                position.fatigue
            ),
            (
                case["day_after"].as_i64().unwrap() as i16,
                case["hour_after"].as_i64().unwrap() as i16,
                case["minute_after"].as_i64().unwrap() as i16,
                case["fatigue_after"].as_i64().unwrap() as i16
            )
        );
        assert_eq!(
            position.schedule,
            decode(case["schedule_after"].as_str().unwrap())
        );
        assert_eq!(
            state
                .warriors
                .into_iter()
                .flat_map(|w| w.bytes)
                .collect::<Vec<_>>(),
            decode(case["output"].as_str().unwrap()),
            "variant {}",
            case["variant"]
        );
        assert_eq!(
            state.random.seed,
            case["seed_after"].as_u64().unwrap() as u32
        );
        assert_eq!(state.random.draws, case["draws"].as_u64().unwrap());
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(
                    e,
                    Event::ScheduleRescheduled {
                        dispatched: true,
                        ..
                    }
                ))
                .count(),
            case["dispatches"].as_u64().unwrap() as usize
        );
    }
}

#[test]
fn hourly_update_matches_original_instruction_execution() {
    compare_hourly();
}

#[test]
fn noon_midnight_and_schedule_match_original_instruction_cases() {
    compare_calendar();
}
