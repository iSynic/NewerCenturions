use centurions_content::{Content, Warrior};
use centurions_core::{Command, MapState, Presentation, ScriptState, Session, apply, load, save};
use serde_json::Value;
use std::path::Path;

fn bytes(value: &Value) -> Vec<u8> {
    let text = value.as_str().unwrap();
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect()
}

#[test]
fn authored_terminal_choice_matches_native_records_and_resumes() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/terminal-stage.json"
    ))
    .unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let content = Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap();
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
    state.warriors = bytes(&fixture["warriors_before"])
        .as_chunks::<3138>()
        .0
        .iter()
        .map(|record| Warrior::read(record).unwrap())
        .collect();
    state.flags = bytes(&fixture["flags_before"]);
    state.mission_stage = 5;
    state.mission_score = 17;
    state.allies = bytes(&fixture["menu_before"]);
    state.ally_count = fixture["menu_count_before"].as_u64().unwrap() as usize;
    state.health_context = Some(centurions_core::story_health::context(
        &state.warriors,
        0,
        None,
    ));
    assert_eq!(state.health_context.as_ref().unwrap().unavailable, 1);
    let position = state.exploration.as_mut().unwrap();
    position.restore_map(
        9,
        MapState::from_native(&bytes(&fixture["map_before"])).unwrap(),
    );
    position.x = 40;
    position.y = 80;
    position.script = Some(ScriptState {
        record: bytes(&fixture["script"]),
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
    assert_eq!(state.flags, bytes(&fixture["flags_after"]));
    assert_eq!(state.allies, bytes(&fixture["menu_after"]));
    assert_eq!(state.ally_count, 0);
    assert_eq!(state.health_context.as_ref().unwrap().unavailable, 1);
    assert_eq!(
        state
            .warriors
            .iter()
            .flat_map(|w| w.bytes.iter().copied())
            .collect::<Vec<_>>(),
        bytes(&fixture["warriors_after"])
    );
    let position = state.exploration.as_ref().unwrap();
    assert_eq!((position.map, position.x, position.y), (0, 2, 2));
    assert_eq!(
        position.presentation,
        Some(Presentation::Prologue { resource_id: 1000 })
    );
    assert_eq!(
        position.map_state().native_bytes().unwrap(),
        bytes(&fixture["map_after"])
    );
    assert_eq!(
        state.visited_maps[&9].native_bytes().unwrap(),
        bytes(&fixture["saved_origin"])
    );
    std::fs::remove_file(checkpoint).unwrap();
}
