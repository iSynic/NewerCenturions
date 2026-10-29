use super::*;
use crate::test_support::decode;
use centurions_content::WARRIOR_BYTES;
use centurions_content::{Warrior, word};
use std::fs;
fn put(bytes: &mut [u8], offset: usize, value: i16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
}
#[path = "../native/time_reference.rs"]
mod native_time;

fn calendar_content(schedule: Vec<u8>) -> Content {
    let mut properties = vec![0; 12000];
    // buttonchoice skips time for tile0; these cases enter timeclick through tile1.
    properties[42..44].copy_from_slice(&1i16.to_be_bytes());
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
                floor: vec![1; 8100],
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
        .chunks(WARRIOR_BYTES)
        .map(|r| Warrior::read(r).unwrap())
        .collect();
    state.phase = Phase::Exploration;
    state.mission = Some("The Thunder of Guns".into());
    state.random = RandomState::new(1);
    let mut position = Exploration::authored(0, content.map(0).unwrap());
    position.hour = hour;
    position.minute = minute;
    state.exploration = Some(position);
    state
}

#[test]
fn calendar_save_resume_preserves_trigger_history() {
    let fixture = crate::test_support::fixture("calendar.json");
    let case = &fixture["cases"][5];
    for (zero, eligible, parent) in [
        (true, false, false),
        (true, true, false),
        (false, false, false),
        (false, true, false),
        (true, true, true),
    ] {
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
        map.tile_properties[42..44].copy_from_slice(&i16::from(!zero).to_be_bytes());
        let mut state = calendar_state(&content, case["input"].as_str().unwrap(), 23, 55);
        if zero {
            state.exploration.as_mut().unwrap().hour = 24;
        }
        state.flags[2] = u8::from(eligible);
        let path = std::env::temp_dir().join(format!("calendar-table-{}.json", std::process::id()));
        let before = state.warriors.clone();
        let command = if parent {
            let tables =
                std::sync::Arc::make_mut(&mut content.scenario.as_mut().unwrap().maps[0].shared);
            tables.combat = Some(centurions_content::combat::CombatContent {
                encounters: vec![0; 10],
                battles: vec![],
                monsters: vec![],
                race_moves: vec![],
                attacks: vec![],
            });
            let extra = &mut tables.combat.as_mut().unwrap().encounters;
            for (offset, value) in [(0, 1), (2, 1), (4, 24), (6, 55)] {
                put(extra, offset, value);
            }
            let mut record = vec![0; 42];
            for (offset, value) in [(8, 63), (10, 109), (26, 99)] {
                put(&mut record, offset, value);
            }
            state.exploration.as_mut().unwrap().script = Some(ScriptState::scheduled(record, 0));
            state.exploration.as_mut().unwrap().presentation = Some(Presentation::ScriptPause);
            Command::Continue
        } else {
            Command::Move { dx: 1, dy: 0 }
        };
        command_resumed(&path, &mut state, &content, command);
        assert_eq!(state.time_continuation.is_some(), eligible && !zero);
        assert_eq!(state.zero_time.is_some(), eligible && zero);
        if zero {
            assert_eq!(state.warriors, before);
            let p = state.exploration.as_ref().unwrap();
            assert_eq!((state.random.seed, state.random.draws), (16807, 1));
            assert_eq!(word(&p.schedule, 0).unwrap(), 5);
        }
        for _ in 0..3 {
            if state.exploration.as_ref().unwrap().presentation.is_none() {
                break;
            }
            command_resumed(&path, &mut state, &content, Command::Continue);
        }
        assert!(state.time_continuation.is_none() && state.zero_time.is_none());
        assert_eq!(state.mission_score, i16::from(eligible) + i16::from(parent));
        let p = state.exploration.as_ref().unwrap();
        assert!(p.event_headers.iter().all(|b| *b == 0));
        assert_eq!((p.day, p.hour, p.minute), (2, 0, if zero { 55 } else { 0 }));
        fs::remove_file(path).unwrap();
    }
}

#[test]
fn hourly_protection_and_word_clock_survive_byte_time_and_saved_commands() {
    let fixture = crate::test_support::fixture("calendar.json");
    let case = &fixture["cases"][0];
    let mut content = calendar_content(decode(case["schedule_before"].as_str().unwrap()));
    for (duration, after, ticks, hour, clock) in [
        (20_i16, 19, 1_i16, 13, (14, 4)),
        (1, 0, 1, 13, (14, 4)),
        (-1, -1, 1, 13, (14, 4)),
        (20, 19, -255, i16::MAX, (i16::MIN, 4)),
        (20, 20, -256, i16::MAX, (i16::MAX, 59)),
    ] {
        let map = &mut content.scenario.as_mut().unwrap().maps[0];
        map.tile_properties[42..44].copy_from_slice(&ticks.to_be_bytes());
        let mut state = calendar_state(&content, case["input"].as_str().unwrap(), hour, 59);
        state.exploration.as_mut().unwrap().day = i16::MIN;
        state.native_work.psi_memory = crate::initial_psi_memory(&state.warriors).unwrap();
        state.party_buffs[4..6].copy_from_slice(&duration.to_be_bytes());
        let path = std::env::temp_dir().join(format!(
            "centurions-hourly-buff-{}-{duration}.json",
            std::process::id()
        ));
        command_resumed(&path, &mut state, &content, Command::Move { dx: 1, dy: 0 });
        assert_eq!(word(&state.party_buffs, 4).unwrap(), after);
        let p = state.exploration.as_ref().unwrap();
        assert_eq!((p.day, p.hour, p.minute), (i16::MIN, clock.0, clock.1));
        fs::remove_file(path).unwrap();
    }
}

/// One step east from a saved copy and from the live state must agree.
fn command_resumed(
    path: &std::path::Path,
    state: &mut Session,
    content: &Content,
    command: Command,
) -> Vec<Event> {
    save(path, state, content).unwrap();
    let mut resumed = load(path, content).unwrap();
    let events = apply(command.clone(), state, content).unwrap();
    assert_eq!(apply(command, &mut resumed, content).unwrap(), events);
    assert_eq!(*state, resumed);
    events
}

#[test]
fn terminal_noon_recovery_commits_ordered_feedback_and_stops_after_saved_defeat() {
    use crate::feedback::NativeFeedback::OutdoorBodyStatus;
    use Event::{Feedback, MissionEnded, Sound};
    let fixture = crate::test_support::fixture("calendar.json");
    let case = &fixture["cases"][0];
    let content = calendar_content(decode(case["schedule_before"].as_str().unwrap()));
    let mut state = calendar_state(&content, case["input"].as_str().unwrap(), 11, 55);
    let health = &mut state.native_work.health_context;
    health.marks_affected = true;
    for warrior in &mut state.warriors {
        let mut record = warrior.record_mut();
        record.set_maximum_stamina(i16::MAX);
        record.set_stamina(32760);
        put(&mut warrior.bytes, 0x3a, 127);
    }
    let count = state.warriors.len();
    let mut random = state.random.clone();
    let mut expected = Vec::new();
    for _ in 0..count {
        expected.push(Sound {
            resource_id: random.bound(2) + 506,
        });
        expected.push(Feedback(OutdoorBodyStatus {
            permanent: true,
            wait: 0,
        }));
    }
    expected.extend([
        Sound { resource_id: 26260 },
        MissionEnded { defeated: true },
    ]);
    let path = std::env::temp_dir().join(format!("centurions-rest-{}.json", std::process::id()));
    let events = command_resumed(&path, &mut state, &content, Command::Move { dx: 1, dy: 0 });
    fs::remove_file(path).unwrap();
    assert!(events.ends_with(&expected), "{events:?}");
    assert_eq!(state.random, random);
    assert_eq!(state.phase, Phase::SquadSelection);
    let health = state.native_work.health_context;
    assert_eq!(health.last_body, (count - 1) as i16);
    assert_eq!(health.affected[count - 1], 0);
    assert_eq!(health.affected[0], -1);
}
