use super::*;
use crate::test_support::decode;
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
#[test]
fn hourly_update_matches_original_instruction_execution() {
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
                state.allies = decode(&case.allies_before);
                state.ally_count = 6;
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
                state.exploration.as_mut().unwrap().script = Some(ScriptState {
                    record,
                    cursor: 0,
                    map_slot: 0,
                    item_return: false,
                    scheduled: true,
                    menu: None,
                    persist: false,
                    origin: None,
                });
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
                let mut legacy: serde_json::Value =
                    serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
                legacy["schema_version"] = 5.into();
                legacy["session"]["inventory_record"] = serde_json::Value::Null;
                fs::write(&path, serde_json::to_vec(&legacy).unwrap()).unwrap();
                assert_eq!(
                    load(&path, &content).is_err(),
                    state.energy_recovery_disabled != 0
                );
                legacy["session"]
                    .as_object_mut()
                    .unwrap()
                    .remove("energy_recovery_disabled");
                fs::write(&path, serde_json::to_vec(&legacy).unwrap()).unwrap();
                assert_eq!(load(&path, &content).unwrap().energy_recovery_disabled, 0);
                fs::remove_file(path).unwrap();
                assert_eq!(state.allies, decode(&case.allies_after));
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

fn calendar_content(schedule: Vec<u8>) -> Content {
    let mut properties = vec![0; 12000];
    properties[2..4].copy_from_slice(&1i16.to_be_bytes());
    let fixture = crate::test_support::fixture("calendar.json");
    let mut items = vec![vec![0; 150]; 801];
    items[800] = decode(fixture["food_definition"].as_str().unwrap());
    Content {
        tutorials: vec![],
        identity: "calendar".into(),
        warriors: vec![],
        items,
        resources: vec![],
        original_services: None,
        scenario: Some(centurions_content::mission::ScenarioContent::single(
            centurions_content::mission::OpeningMission {
                name: "calendar".into(),
                start: (2, 2),
                header: vec![0; 4200],
                regions: vec![0; 644],
                floor: vec![0; 8100],
                objects: vec![0; 8100],
                events: vec![0; 8100],
                tile_properties: properties,
                terrain: 2,
                default_tile: 0,
                minutes_per_tick: 5,
                shared: std::sync::Arc::new(centurions_content::mission::StoryTables {
                    schedule,
                    scheduled_scripts: vec![0; 118 * 42],
                    ..Default::default()
                }),
            },
        )),
    }
}

fn calendar_state(content: &Content, input: &str, hour: i16, minute: i16) -> Session {
    let mut state = Session::new(content);
    state.warriors = decode(input)
        .as_chunks::<3138>()
        .0
        .iter()
        .map(|r| Warrior::read(r).unwrap())
        .collect();
    state.phase = Phase::Exploration;
    state.mission = Some("The Thunder of Guns".into());
    state.random = RandomState::new(1);
    state.exploration = Some(Exploration {
        map: 0,
        x: 2,
        y: 2,
        day: 1,
        hour,
        minute,
        fatigue: 4,
        facing: 0,
        last_move: [0; 2],
        script: None,
        presentation: None,
        event_headers: vec![0; 4200],
        schedule: content.map(0).unwrap().schedule.clone(),
        region_state: content.map(0).unwrap().regions.clone(),
        visibility: vec![0; 8100],
        floor: content.map(0).unwrap().floor.clone(),
        objects: content.map(0).unwrap().objects.clone(),
        events: content.map(0).unwrap().events.clone(),
    });
    state
}

#[test]
fn noon_midnight_and_schedule_match_original_instruction_cases() {
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
fn calendar_save_resume_and_legacy_migration_preserve_trigger_history() {
    let fixture = crate::test_support::fixture("calendar.json");
    let case = &fixture["cases"][5];
    for eligible in [false, true] {
        let mut content = calendar_content(decode(case["schedule_before"].as_str().unwrap()));
        let map = &mut content.scenario.as_mut().unwrap().maps[0];
        let tables = std::sync::Arc::make_mut(&mut map.shared);
        tables.dialogue = vec![0; 512];
        for (offset, value) in [(18, 2_i16), (20, 1), (8, 0), (10, 0)] {
            put(&mut tables.schedule, offset, value);
        }
        for (offset, value) in [(8, 1_i16), (10, 109), (24, 1), (26, 1)] {
            put(&mut tables.scheduled_scripts, 117 * 42 + offset, value);
        }
        map.regions[..8].copy_from_slice(&[0, 0, 0, 0, 0, 10, 0, 10]);
        let mut state = calendar_state(&content, case["input"].as_str().unwrap(), 23, 55);
        state.flags[2] = u8::from(eligible);
        apply(Command::Move { dx: 1, dy: 0 }, &mut state, &content).unwrap();
        assert_eq!(state.time_continuation.is_some(), eligible);
        let path = std::env::temp_dir().join(format!(
            "centurions-scheduled-dialogue-{}-{eligible}.json",
            std::process::id()
        ));
        save(&path, &state, &content).unwrap();
        let mut resumed = load(&path, &content).unwrap();
        for _ in 0..3 {
            if state.exploration.as_ref().unwrap().presentation.is_none() {
                break;
            }
            assert_eq!(
                apply(Command::Continue, &mut state, &content).unwrap(),
                apply(Command::Continue, &mut resumed, &content).unwrap()
            );
            assert_eq!(state, resumed);
        }
        assert!(state.time_continuation.is_none());
        assert_eq!(state.mission_score, i16::from(eligible));
        let p = state.exploration.as_ref().unwrap();
        assert_eq!((p.day, p.hour, p.minute), (2, 0, 0));
        fs::remove_file(path).unwrap();
    }
    let content = calendar_content(decode(case["schedule_before"].as_str().unwrap()));
    let mut state = calendar_state(&content, case["input"].as_str().unwrap(), 23, 55);
    let path =
        std::env::temp_dir().join(format!("centurions-calendar-{}.json", std::process::id()));
    save(&path, &state, &content).unwrap();
    let mut resumed = load(&path, &content).unwrap();
    let command = Command::Move { dx: 1, dy: 0 };
    let events = apply(command.clone(), &mut state, &content).unwrap();
    assert_eq!(apply(command, &mut resumed, &content).unwrap(), events);
    assert_eq!(state, resumed);
    save(&path, &state, &content).unwrap();
    assert_eq!(load(&path, &content).unwrap(), state);
    let mut legacy = state.clone();
    legacy.inventory_record = None;
    legacy.exploration.as_mut().unwrap().day = 1;
    legacy.exploration.as_mut().unwrap().schedule.clear();
    fs::write(
        &path,
        serde_json::to_vec(&Save {
            schema_version: 1,
            session: legacy,
        })
        .unwrap(),
    )
    .unwrap();
    let migrated = load(&path, &content).unwrap();
    assert_eq!(
        migrated.exploration.unwrap().schedule,
        content.map(0).unwrap().schedule
    );
    fs::write(
        &path,
        serde_json::to_vec(&Save {
            schema_version: 1,
            session: state.clone(),
        })
        .unwrap(),
    )
    .unwrap();
    assert!(
        load(&path, &content)
            .unwrap_err()
            .to_string()
            .contains("unknown schedule history")
    );
    fs::remove_file(&path).unwrap();
}

#[test]
fn hourly_party_protection_ticks_without_changing_other_rules_and_resumes() {
    let fixture = crate::test_support::fixture("calendar.json");
    let case = &fixture["cases"][0];
    let content = calendar_content(decode(case["schedule_before"].as_str().unwrap()));
    for (duration, after) in [(20_i16, 19_i16), (1, 0), (-1, -1)] {
        let mut state = calendar_state(&content, case["input"].as_str().unwrap(), 13, 55);
        state.psi_memory = Some(crate::initial_psi_memory(&state.warriors).unwrap());
        let mut baseline = state.clone();
        state.party_buffs[4..6].copy_from_slice(&duration.to_be_bytes());
        let path = std::env::temp_dir().join(format!(
            "centurions-hourly-buff-{}-{duration}.json",
            std::process::id()
        ));
        save(&path, &state, &content).unwrap();
        let mut resumed = load(&path, &content).unwrap();
        let command = Command::Move { dx: 1, dy: 0 };
        let events = apply(command.clone(), &mut state, &content).unwrap();
        assert_eq!(
            apply(command.clone(), &mut resumed, &content).unwrap(),
            events
        );
        assert_eq!(state, resumed);
        assert_eq!(apply(command, &mut baseline, &content).unwrap(), events);
        assert_eq!(word(&state.party_buffs, 4).unwrap(), after);
        baseline.party_buffs = state.party_buffs.clone();
        assert_eq!(state, baseline);
        fs::remove_file(path).unwrap();
    }
}
