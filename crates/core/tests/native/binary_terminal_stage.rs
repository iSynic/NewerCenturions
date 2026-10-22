use crate::{prelude::*, support};

use centurions_core::MapState;

#[test]
fn authored_terminal_choice_matches_native_records_and_resumes() {
    let fixture: Value = support::fixture("terminal-stage.json");

    let content = support::content();
    let mut state = support::mission_session(&content);
    state.warriors = support::warriors(&fixture["warriors_before"]);
    state.flags = fixture.bytes("flags_before");
    state.mission_stage = 5;
    state.mission_score = 17;
    state.allies = fixture.bytes("menu_before");
    state.ally_count = fixture.n::<usize>("menu_count_before");
    state.native_work.health_context = Some(centurions_core::story_health::context(
        &state.warriors,
        0,
        None,
    ));
    assert_eq!(
        state
            .native_work
            .health_context
            .as_ref()
            .unwrap()
            .unavailable,
        1
    );
    let position = state.exploration.as_mut().unwrap();
    position.restore_map(
        9,
        MapState::from_native(&fixture.bytes("map_before")).unwrap(),
    );
    position.x = 40;
    position.y = 80;
    position.script = Some(centurions_core::ScriptState::scheduled(
        fixture.bytes("script"),
        0,
    ));
    position.presentation = Some(Presentation::Dialogue {
        id: 0,
        text: String::new(),
    });
    let random = state.random.clone();
    let mut dialogue = vec![];
    for _ in 0..6 {
        apply(Command::Continue, &mut state, &content).unwrap();
        if let Some(Presentation::Dialogue { id, .. }) =
            &state.exploration.as_ref().unwrap().presentation
        {
            dialogue.push(*id);
        }
    }
    assert_eq!(dialogue, [1322, 1323, 1324, 1325, 1326]);
    assert_eq!(
        state.exploration.as_ref().unwrap().presentation,
        Some(Presentation::StageChange { parameter: 1 })
    );
    let checkpoint =
        std::env::temp_dir().join(format!("centurions-terminal-{}.json", std::process::id()));
    save(&checkpoint, &state, &content).unwrap();
    let mut resumed = load(&checkpoint, &content).unwrap();
    let before_no = resumed.warriors.clone();
    apply(Command::Choose { answer: false }, &mut resumed, &content).unwrap();
    assert_eq!(resumed.mission_stage, 5);
    assert_eq!(resumed.warriors, before_no);
    resumed = load(&checkpoint, &content).unwrap();
    let events = apply(Command::Choose { answer: true }, &mut state, &content).unwrap();
    assert_eq!(
        apply(Command::Choose { answer: true }, &mut resumed, &content).unwrap(),
        events
    );
    assert_eq!(resumed, state);
    assert_eq!(state.mission_stage, 0);
    assert_eq!(state.mission_score, 17);
    assert_eq!(state.random, random);
    support::native_expect! {
        bytes state.flags => &fixture["flags_after"], "binary_terminal_stage";
        bytes state.allies => &fixture["menu_after"], "binary_terminal_stage";
    }
    assert_eq!(state.ally_count, 0);
    assert_eq!(
        state
            .native_work
            .health_context
            .as_ref()
            .unwrap()
            .unavailable,
        1
    );
    assert_eq!(
        support::record_bytes(&state.warriors),
        fixture.bytes("warriors_after")
    );
    let position = state.exploration.as_ref().unwrap();
    assert_eq!((position.map, position.x, position.y), (0, 2, 2));
    assert_eq!(
        position.presentation,
        Some(Presentation::Prologue { resource_id: 1000 })
    );
    support::native_expect! {
        bytes position.map_state().native_bytes().unwrap() => &fixture["map_after"], "binary_terminal_stage";
        bytes state.visited_maps[&9].native_bytes().unwrap() => &fixture["saved_origin"], "binary_terminal_stage";
    }
    std::fs::remove_file(checkpoint).unwrap();
}
