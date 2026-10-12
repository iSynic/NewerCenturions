use crate::prelude::*;

use std::path::PathBuf;

fn fixture() -> (Content, Session) {
    let content = crate::support::content();
    let mut state = Session::new(&content);
    apply(Command::ImportStarters, &mut state, &content).unwrap();
    apply(
        Command::EnterMission {
            name: "The Thunder of Guns".into(),
        },
        &mut state,
        &content,
    )
    .unwrap();
    (content, state)
}

fn path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("centurions-{name}-{}.json", std::process::id()))
}

fn stage_choice(state: &mut Session, parameter: i16) {
    let position = state.exploration.as_mut().unwrap();
    position.script = Some(ScriptState {
        record: vec![0; 42],
        cursor: 1,
        map_slot: 0,
        scheduled: true,
        menu: None,
        persist: false,
        origin: None,
    });
    position.presentation = Some(Presentation::StageChange { parameter });
}

#[test]
fn failed_story_transaction_discards_mutations_and_ordered_effects() {
    let (content, mut state) = fixture();
    let position = state.exploration.as_mut().unwrap();
    let mut record = vec![0; 42];
    for (offset, value) in [(8, 47_i16), (10, 9), (12, 200), (24, 2), (26, 6000)] {
        record[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
    }
    position.script = Some(ScriptState {
        record,
        cursor: 0,
        map_slot: 0,
        scheduled: true,
        menu: None,
        persist: false,
        origin: None,
    });
    position.presentation = Some(Presentation::Dialogue {
        id: 0,
        text: String::new(),
    });
    let before = state.clone();
    assert!(apply(Command::Continue, &mut state, &content).is_err());
    assert_eq!(state, before);
    assert!(apply(Command::SelectMenuEntry { index: 4 }, &mut state, &content).is_err());
    assert_eq!(state, before);
}

#[test]
fn visited_map_consequences_and_pending_choices_survive_v3_reload() {
    let (content, mut state) = fixture();
    let position = state.exploration.as_mut().unwrap();
    position.floor[81] = 123;
    position.objects[82] = -205;
    position.events[83] = 47;
    position.visibility[84] = 1;
    position.event_headers[42] = 19;
    position.region_state[37] = 9;
    let original = position.map_state();
    state.flags[17] = 1;
    state.mission_score = 23;
    stage_choice(&mut state, 2);
    apply(Command::Choose { answer: true }, &mut state, &content).unwrap();
    assert_eq!(state.visited_maps.get(&0), Some(&original));
    let destination = path("visited-choice");
    let abandoned = destination.with_extension("json.tmp");
    std::fs::write(&abandoned, b"interrupted save").unwrap();
    save(&destination, &state, &content).unwrap();
    assert_eq!(load(&destination, &content).unwrap(), state);
    assert_eq!(std::fs::read(&abandoned).unwrap(), b"interrupted save");
    std::fs::remove_file(abandoned).unwrap();
    stage_choice(&mut state, 1);
    apply(Command::Choose { answer: true }, &mut state, &content).unwrap();
    assert_eq!(state.exploration.as_ref().unwrap().map_state(), original);
    assert_eq!(state.flags[17], 1);
    assert_eq!(state.mission_score, 23);
    std::fs::remove_file(destination).unwrap();
}

#[test]
fn v2_migration_preserves_recovered_history_and_rejects_unknown_maps() {
    let (content, mut state) = fixture();
    state.exploration.as_mut().unwrap().event_headers[42] = 17;
    let destination = path("legacy-story");
    save(&destination, &state, &content).unwrap();
    let mut document: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&destination).unwrap()).unwrap();
    document["schema_version"] = 2.into();
    let session = document["session"].as_object_mut().unwrap();
    for field in [
        "menus",
        "flags",
        "mission_score",
        "mission_stage",
        "visited_maps",
        "allies",
        "ally_count",
        "party_mask",
        "selected_warrior",
    ] {
        session.remove(field);
    }
    let position = session["exploration"].as_object_mut().unwrap();
    for field in ["floor", "objects", "events", "visibility"] {
        position.remove(field);
    }
    std::fs::write(&destination, serde_json::to_vec(&document).unwrap()).unwrap();
    let migrated = load(&destination, &content).unwrap();
    assert_eq!(migrated.exploration.as_ref().unwrap().event_headers[42], 17);
    assert_eq!(
        migrated.exploration.as_ref().unwrap().floor,
        content.map(0).unwrap().floor
    );
    assert_eq!(migrated.flags[0], 255);
    document["session"]["exploration"]["map"] = 1.into();
    std::fs::write(&destination, serde_json::to_vec(&document).unwrap()).unwrap();
    assert!(load(&destination, &content).is_err());
    std::fs::remove_file(destination).unwrap();
}

#[test]
fn queued_death_child_battle_resumes_parent_without_restoring_roster_or_duplicate_rewards() {
    use centurions_core::{
        Event, MapState,
        combat::{self, death_scripts::DeathEntry, runtime::DeathStory},
    };
    let (content, mut state) = fixture();
    let position = state.exploration.as_mut().unwrap();
    position.restore_map(1, MapState::authored(content.map(1).unwrap()));
    position.x = 77;
    position.y = 9;
    position.presentation = None;
    position.script = Some(ScriptState {
        record: content.map(1).unwrap().script(55).unwrap().to_vec(),
        cursor: 1,
        map_slot: 8,
        scheduled: true,
        menu: None,
        persist: false,
        origin: None,
    });
    combat::begin_battle(&mut state, &content, 14, 0).unwrap();
    // Controlled defeat is the same boundary used by the native nested-caller fixture.
    let arena = state.combat.as_mut().unwrap();
    arena.enemies.records[0].bytes[0x110..0x112].fill(0);
    combat::lifecycle::kill_body(arena, &mut state.warriors, 10).unwrap();
    arena.death_queue.begin();
    let (_, entry) = arena.death_queue.current().unwrap();
    assert_eq!(entry.script, 51);
    state.combat_story = Some(DeathStory {
        body: 0,
        script: 51,
        actor: 0,
        entry: DeathEntry { ..entry },
        ..Default::default()
    });
    let checkpoint = path("nested-death");
    save(&checkpoint, &state, &content).unwrap();
    let mut resumed = load(&checkpoint, &content).unwrap();
    let mut started = vec![14];
    for _ in 0..20 {
        let events = apply(Command::Continue, &mut state, &content).unwrap();
        assert_eq!(
            apply(Command::Continue, &mut resumed, &content).unwrap(),
            events
        );
        assert_eq!(state, resumed);
        for event in events {
            if let Event::CombatStarted { battle, .. } = event {
                started.push(battle);
            }
        }
        if state
            .combat_story
            .as_ref()
            .is_some_and(|story| story.waiting_child)
        {
            break;
        }
    }
    assert_eq!(started, [14, 15]);
    assert_eq!(state.combat.as_ref().unwrap().battle, 15);
    assert_eq!(state.combat.as_ref().unwrap().enemies.records.len(), 1);
    assert_eq!(
        state.combat.as_ref().unwrap().enemies.records[0].bytes[0x103],
        67
    );
    for candidate in [&mut state, &mut resumed] {
        candidate.combat.as_mut().unwrap().enemies.records[0].bytes[0x110..0x112].fill(0);
    }
    let child_events = combat::runtime::confirm_survivors(&mut state, &content).unwrap();
    assert_eq!(
        combat::runtime::confirm_survivors(&mut resumed, &content).unwrap(),
        child_events
    );
    assert_eq!(state, resumed);
    assert!(state.combat.is_none());
    let rewarded = state.warriors.clone();
    let rewarded_random = state.random.clone();
    let events = apply(Command::Continue, &mut state, &content).unwrap();
    assert_eq!(
        apply(Command::Continue, &mut resumed, &content).unwrap(),
        events
    );
    assert_eq!(state, resumed);
    assert!(state.combat_story.is_none());
    assert_eq!(state.warriors, rewarded);
    assert_eq!(state.random, rewarded_random);
    assert_eq!(
        state
            .exploration
            .as_ref()
            .unwrap()
            .script
            .as_ref()
            .unwrap()
            .cursor,
        1
    );
    for _ in 0..20 {
        let events = apply(Command::Continue, &mut state, &content).unwrap();
        assert_eq!(
            apply(Command::Continue, &mut resumed, &content).unwrap(),
            events
        );
        assert_eq!(state, resumed);
        for event in events {
            if let Event::CombatStarted { battle, .. } = event {
                started.push(battle);
            }
        }
        if state.combat.is_some() {
            break;
        }
    }
    assert_eq!(started, [14, 15, 15]);
    assert!(state.combat_story.is_none());
    std::fs::remove_file(checkpoint).unwrap();
}
