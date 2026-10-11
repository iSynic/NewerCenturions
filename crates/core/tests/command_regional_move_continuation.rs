use centurions_content::Content;
use centurions_core::{
    Command, Event, MovementContinuation, Presentation, ScriptOrigin, ScriptState, Session, apply,
    load, save,
};
use serde_json::Value;

fn inputs() -> (Content, Value) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let content = Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap();
    let native = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/regional-move-continuation.json"
    ))
    .unwrap();
    (content, native)
}
fn foundation(native: &Value, index: usize) -> Session {
    serde_json::from_value(native["foundations"][index]["session"].clone()).unwrap()
}
fn bytes(value: &Value) -> Vec<u8> {
    let h = value.as_str().unwrap();
    (0..h.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
        .collect()
}
fn page(state: &mut Session) {
    state.exploration.as_mut().unwrap().presentation = Some(Presentation::Messages {
        pages: vec![(0, "Acknowledged boundary".into())],
        next: None,
    });
}
fn same(left: &Session, right: &Session) {
    let a = serde_json::to_value(left).unwrap();
    let b = serde_json::to_value(right).unwrap();
    for (key, value) in a.as_object().unwrap() {
        assert!(value == &b[key], "session field {key}");
    }
}
fn checkpoint(state: &Session, content: &Content, name: &str) -> Session {
    let path = std::env::temp_dir().join(format!(
        "centurions-walking-{name}-{}.json",
        std::process::id()
    ));
    save(&path, state, content).unwrap();
    let resumed = load(&path, content).unwrap();
    std::fs::remove_file(path).unwrap();
    same(&resumed, state);
    resumed
}

#[test]
fn native_header_prefix_composes_with_existing_script_transaction_and_resumes() {
    let (content, native) = inputs();
    let mut state = foundation(&native, 1);
    page(&mut state);
    state.movement_continuation = Some(MovementContinuation::Header);
    let mut resumed = checkpoint(&state, &content, "active-before");
    // Independent native DD prefix supplies the admitted record and final RNG.
    // Existing script execution is separately verified; this checks their composition.
    let mut expected = state.clone();
    expected.movement_continuation = None;
    expected.random.seed = native["cases"][2]["after"]["seed"].as_u64().unwrap() as u32;
    expected.random.draws += 1;
    expected.exploration.as_mut().unwrap().script = Some(ScriptState {
        record: bytes(&native["cases"][2]["after"]["working_header"]),
        cursor: 0,
        map_slot: 7,
        scheduled: false,
        menu: None,
        persist: false,
        origin: Some(ScriptOrigin { map: 0, x: 7, y: 3 }),
    });
    let expected_events = apply(Command::Continue, &mut expected, &content).unwrap();
    expected.movement_continuation = Some(MovementContinuation::Secrets);
    let events = apply(Command::Continue, &mut state, &content).unwrap();
    assert_eq!(events, expected_events);
    assert_eq!(
        apply(Command::Continue, &mut resumed, &content).unwrap(),
        events
    );
    same(&state, &expected);
    same(&resumed, &state);
    let mut ids = vec![];
    for _ in 0..6 {
        let Some(Presentation::Dialogue { id, .. }) =
            state.exploration.as_ref().unwrap().presentation.as_ref()
        else {
            panic!("expected native text page");
        };
        ids.push(*id);
        let events = apply(Command::Continue, &mut state, &content).unwrap();
        assert_eq!(
            apply(Command::Continue, &mut expected, &content).unwrap(),
            events
        );
        assert_eq!(
            apply(Command::Continue, &mut resumed, &content).unwrap(),
            events
        );
        same(&resumed, &state);
        same(&expected, &state);
    }
    assert_eq!(ids, vec![175, 176, 177, 178, 179, 180]);
    assert_eq!(
        state.movement_continuation,
        Some(MovementContinuation::Secrets)
    );
    assert!(state.exploration.as_ref().unwrap().script.is_some());
    checkpoint(&state, &content, "active-after");
    let before = state.clone();
    assert!(apply(Command::Move { dx: 0, dy: 1 }, &mut state, &content).is_err());
    same(&state, &before);
}

#[test]
fn inactive_native_header_resumes_without_extra_move_time_or_rng() {
    let (content, native) = inputs();
    let mut state = foundation(&native, 1);
    state.exploration.as_mut().unwrap().event_headers[7 * 42 + 7] = 0;
    page(&mut state);
    state.movement_continuation = Some(MovementContinuation::Header);
    let mut expected = state.clone();
    expected.exploration.as_mut().unwrap().presentation = None;
    expected.movement_continuation = None;
    let mut resumed = checkpoint(&state, &content, "inactive");
    assert!(
        apply(Command::Continue, &mut state, &content)
            .unwrap()
            .is_empty()
    );
    assert!(
        apply(Command::Continue, &mut resumed, &content)
            .unwrap()
            .is_empty()
    );
    same(&state, &expected);
    same(&resumed, &state);
}

#[test]
fn late_header_failure_rolls_back_acknowledgement_and_rng() {
    let (content, native) = inputs();
    let mut state = foundation(&native, 1);
    // Admit the native chance/RNG branch, then encounter an unrecovered opcode.
    state.exploration.as_mut().unwrap().event_headers[7 * 42 + 8..7 * 42 + 10]
        .copy_from_slice(&300_i16.to_be_bytes());
    page(&mut state);
    state.movement_continuation = Some(MovementContinuation::Header);
    let before = state.clone();
    assert!(matches!(
        apply(Command::Continue, &mut state, &content),
        Err(centurions_core::RuleError::Unsupported { .. })
    ));
    same(&state, &before);
    checkpoint(&state, &content, "rollback");
}

#[test]
fn regional_battle_retains_outer_header_across_save_and_does_not_dispatch_early() {
    let (content, native) = inputs();
    let mut state = foundation(&native, 0);
    let events = apply(Command::Move { dx: 1, dy: -1 }, &mut state, &content).unwrap();
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::CombatStarted { .. }))
    );
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::Message { id: 175..=180, .. }))
    );
    assert_eq!(
        state.movement_continuation,
        Some(MovementContinuation::Header)
    );
    let p = state.exploration.as_ref().unwrap();
    assert_eq!((p.x, p.y), (7, 3));
    assert_eq!(p.event_headers[7 * 42 + 7], 100);
    assert_eq!(state.phase, centurions_core::Phase::Combat);
    checkpoint(&state, &content, "combat");
}
