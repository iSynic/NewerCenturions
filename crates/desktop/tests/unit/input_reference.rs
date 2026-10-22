use super::*;

#[test]
fn held_navigation_repeats_without_backlog_and_release_stops_it() {
    let mut input = MovementInput::default();
    for (key, direction) in [
        (KeyCode::Right, (1, 0)),
        (KeyCode::Kp1, (-1, 1)),
        (KeyCode::Kp8, (0, -1)),
        (KeyCode::Kp9, (1, -1)),
    ] {
        input.reset();
        let actual = movement_direction(|pressed| pressed == key);
        assert_eq!(actual, direction);
        assert!(matches!(input.step(actual, 1.), Some(Command::Move { .. })));
        assert!(input.step(actual, 1.05).is_none());
        assert!(input.step(actual, 1.11).is_some());
        assert!(input.step(actual, 10.).is_some());
        assert!(input.step(actual, 10.).is_none());
        input.refuse();
        assert!(input.step(actual, 20.).is_none());
        assert!(input.step((0, 0), 10.).is_none());
        assert!(input.step(actual, 10.).is_some());
    }
    input.reset();
    assert!(input.pointer_step((1, 0), true, true, 1.).is_some());
    assert!(input.pointer_step((1, 0), false, true, 1.05).is_none());
    assert!(input.pointer_step((1, 0), false, true, 1.11).is_some());
    input.refuse();
    assert!(input.pointer_step((1, 0), false, true, 2.).is_none());
    assert!(input.pointer_step((1, 0), false, false, 2.).is_none());
    assert!(input.pointer_step((1, 0), true, true, 3.).is_some());
    input.stop_pointer();
    assert!(input.pointer_step((1, 0), false, true, 4.).is_none());
    let native: serde_json::Value = crate::original_content::fixture("thunder-mouse-hold.json");
    for case in native["cases"].as_array().unwrap() {
        input.pointer_step((1, 0), true, true, 5.);
        assert_eq!(
            input
                .pointer_step((1, 0), false, case["held"].as_bool().unwrap(), 6.)
                .is_some(),
            case["repeat"].as_bool().unwrap()
        );
    }
}

fn state() -> Session {
    let fixture: serde_json::Value = crate::original_content::fixture("player-psi.json");
    serde_json::from_value(fixture["route_foundation"].clone()).unwrap()
}

// Protects native camera clamps and click targeting using the same visible origin.
#[test]
fn outdoor_camera_and_mouse_targeting_match_original_centering() {
    let fixture: serde_json::Value = crate::original_content::fixture("outdoor-viewport.json");
    let mut state = state();
    for case in fixture["cases"].as_array().unwrap() {
        let x = case["position"][0].as_i64().unwrap() as i32;
        let y = case["position"][1].as_i64().unwrap() as i32;
        let expected = (
            case["origin"][0].as_i64().unwrap() as i32,
            case["origin"][1].as_i64().unwrap() as i32,
        );
        assert_eq!(crate::layout::outdoor_origin(x, y), expected);
        let position = state.exploration.as_mut().unwrap();
        position.x = x;
        position.y = y;
        assert!(outdoor_move(&state, x - expected.0, y - expected.1).is_none());
        assert!(matches!(
            outdoor_move(&state, x - expected.0 + 1, y - expected.1),
            Some(Action::Command(Command::Move { dx: 1, dy: 0 }))
        ));
    }
}

fn command(action: Action) -> Command {
    let Action::Command(command) = action else {
        panic!("refill must use the production command boundary");
    };
    command
}

// Protects the actual mechanical button, map routing, saved targeting and cancellation.
#[test]
fn combat_controls_capture_cancel_and_resolve_through_apply() {
    let receipt: serde_json::Value =
        crate::original_content::fixture("thunder-attack-estimates.json");
    let content = crate::original_content::load();
    let native = centurions_content::native_text::NativeText::load(
        &content,
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../analysis/extracted"),
    )
    .unwrap();
    let mut estimate: Session = serde_json::from_value(receipt["input"].clone()).unwrap();
    for case in receipt["cases"].as_array().unwrap() {
        estimate.warriors[4].bytes[0x9dd] = u8::from(case["firing"].as_bool().unwrap());
        let plan = centurions_core::item_display::attack_estimate(
            &estimate,
            &content,
            10,
            &native,
            &Default::default(),
        )
        .unwrap();
        let numbers: Vec<_> = plan
            .effects
            .iter()
            .filter_map(|effect| match effect {
                centurions_core::item_display::output::PanelEffect::NumberToString {
                    value,
                    ..
                } => Some(*value),
                _ => None,
            })
            .collect();
        assert_eq!(serde_json::json!(numbers), case["numbers"]);
        let retained = plan.retained_definitions.as_ref().unwrap();
        for (bytes, field) in [
            (&retained.item, "item_after"),
            (&retained.ammo, "ammunition_after"),
            (&retained.weapon, "weapon_after"),
        ] {
            let expected = case[field].as_str().unwrap();
            let actual = bytes.as_deref().unwrap_or(&[0; 150]);
            assert_eq!(
                actual
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>(),
                expected
            );
        }
    }
    use centurions_core::{apply, load, save};
    let mut state = state();
    state.native_work.psi_memory = Some(vec![0; 480]);
    let actor = state.combat.as_ref().unwrap().active_body().unwrap() as usize;
    for (offset, value) in [
        (0xf4, 100_i16),
        (0xf6, 100),
        (0x40, 6),
        (0xaca, 10),
        (0xacc, 0),
    ] {
        state.warriors[actor].bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
    }
    let combat = state.combat.as_mut().unwrap();
    let origin = combat.squad.positions[actor];
    let point = (origin.0, origin.1 + 1);
    let cell = centurions_content::mission::OpeningMission::index(
        i32::from(point.0 + combat.map.shift.0 + 5),
        i32::from(point.1 + combat.map.shift.1 + 5),
    )
    .unwrap();
    combat.map.tiles[cell] = 10;
    combat.enemies.positions[0] = point;
    combat.enemies.records[0].bytes[0xe..0x10].copy_from_slice(&1_i16.to_be_bytes());
    combat.enemies.records[0].bytes[0x110..0x112].copy_from_slice(&100_i16.to_be_bytes());
    combat.enemies.records[0].bytes[0xb0] = 1;
    combat.enemies.records[0].bytes[0xb1] = 1;
    let start = state.clone();
    let controls = crate::combat_controls::buttons(&state, &content, actor);
    let button = controls
        .iter()
        .find(|c| matches!(c.action, Action::Command(Command::BeginCapture)))
        .unwrap();
    assert!(button.enabled);
    let entry = apply(command(button.action.clone()), &mut state, &content).unwrap();
    assert_eq!(state.random, start.random);
    let path = std::env::temp_dir().join(format!("centurions-capture-{}.json", std::process::id()));
    save(&path, &state, &content).unwrap();
    let pending = load(&path, &content).unwrap();
    std::fs::remove_file(&path).unwrap();
    assert!(state == pending, "saved targeting differs");
    assert!(!available(
        &state,
        actor,
        &Action::Command(Command::EndTurn)
    ));
    apply(Command::CancelCapture, &mut state, &content).unwrap();
    assert!(state == start, "cancellation changed the session");
    apply(Command::BeginCapture, &mut state, &content).unwrap();
    apply(
        Command::CombatClick {
            x: point.0,
            y: point.1,
        },
        &mut state,
        &content,
    )
    .unwrap();
    assert!(state.combat.as_ref().unwrap().capture_actor.is_none());
    assert_eq!(
        centurions_content::word(&state.warriors[actor].bytes, 0x40).unwrap(),
        4
    );
    assert!(state.random.draws > start.random.draws);
    let mut ui = UiState::default();
    ui.events(&entry);
    ui.feedback_input(0., false);
    ui.feedback_input(1., false);
    assert!(!ui.feedback_input(2., false));
    assert!(ui.persistent_feedback.is_some());
}

// Protects restored panel dispatch, time/Psi costs, persistent toggles and modal refusal rollback.
#[test]
fn original_field_controls_dispatch_and_resume_without_mutating_blocked_sessions() {
    use centurions_core::{apply, field_controls::Action as Field};
    let content = crate::original_content::load();
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
    let click = |state: &Session, id| {
        command(
            field_buttons(state)
                .into_iter()
                .find(|(control, _)| *control == id)
                .unwrap()
                .1,
        )
    };
    state.exploration.as_mut().unwrap().fatigue = 50;
    apply(click(&state, 308), &mut state, &content).unwrap();
    assert_eq!(
        (
            state.exploration.as_ref().unwrap().minute,
            state.exploration.as_ref().unwrap().fatigue
        ),
        (25, 48)
    );
    let index = centurions_content::mission::OpeningMission::index(3, 2).unwrap();
    state.exploration.as_mut().unwrap().floor[index] = 3001;
    apply(click(&state, 307), &mut state, &content).unwrap();
    assert_eq!(state.exploration.as_ref().unwrap().floor[index], 2001);
    apply(click(&state, 142), &mut state, &content).unwrap();
    apply(click(&state, 154), &mut state, &content).unwrap();
    assert!(
        state
            .field_controls
            .as_ref()
            .is_some_and(|c| c.searching && c.camping)
    );
    let path = std::env::temp_dir().join(format!(
        "centurions-field-controls-{}.json",
        std::process::id()
    ));
    centurions_core::save(&path, &state, &content).unwrap();
    assert_eq!(centurions_core::load(&path, &content).unwrap(), state);
    let mut downgraded: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    downgraded["schema_version"] = 9.into();
    crate::original_content::legacy_save(&mut downgraded);
    std::fs::write(&path, serde_json::to_vec(&downgraded).unwrap()).unwrap();
    assert!(centurions_core::load(&path, &content).is_err());
    std::fs::remove_file(path).unwrap();
    let warrior = &mut state.warriors[0].bytes;
    warrior[0x9dc] = 1;
    warrior[0xac6..0xac8].copy_from_slice(&1_i16.to_be_bytes());
    warrior[0xac8..0xaca].fill(0);
    warrior[0x5a..0x5c].copy_from_slice(&50_i16.to_be_bytes());
    warrior[0x58..0x5a].copy_from_slice(&50_i16.to_be_bytes());
    warrior[0x50..0x52].copy_from_slice(&1_i16.to_be_bytes());
    apply(click(&state, 309), &mut state, &content).unwrap();
    assert!(centurions_content::word(&state.warriors[0].bytes, 0x50).unwrap() > 1);
    assert!(state.warriors[0].psi_points().unwrap() < 50);
    state.field_controls.as_mut().unwrap().clinic = Some(100);
    apply(click(&state, 302), &mut state, &content).unwrap();
    assert!(matches!(
        state.exploration.as_ref().unwrap().presentation,
        Some(Presentation::Clinic { .. })
    ));
    apply(Command::ClinicControl { item: 1 }, &mut state, &content).unwrap();
    assert!(centurions_core::field_controls::free(&state));
    state.field_controls.as_mut().unwrap().clinic = None;
    state.shop_access = Some(centurions_core::shop::Access {
        level: 0,
        id: 0,
        enhancement: 0,
        remaining: 0,
    });
    apply(click(&state, 304), &mut state, &content).unwrap();
    assert_eq!(
        state.exploration.as_ref().unwrap().presentation,
        Some(Presentation::Shop)
    );
    apply(Command::CloseShop, &mut state, &content).unwrap();
    assert!(centurions_core::field_controls::free(&state));
    state.exploration.as_mut().unwrap().presentation = Some(Presentation::ScriptPause);
    let before = state.clone();
    assert!(!available(
        &state,
        0,
        &Action::Command(Command::FieldControl {
            action: Field::Rest
        })
    ));
    assert!(
        apply(
            Command::FieldControl {
                action: Field::Rest
            },
            &mut state,
            &content
        )
        .is_err()
    );
    assert!(apply(Command::Move { dx: 1, dy: 0 }, &mut state, &content).is_err());
    assert_eq!(state, before);
}

// Protects actual combat controls, reversible viewing, refusal rollback and manual turn costs.
#[test]
fn combat_buttons_execute_actions_and_preserve_cancelled_or_refused_state() {
    use crate::application::{Application, ApplicationAction, Screen};
    use centurions_content::word;
    use centurions_core::{apply, combat::controls, load, save};
    let content = crate::original_content::load();
    let mut state = state();
    state.native_work.psi_memory = Some(vec![0; 480]);
    let actor = state.combat.as_ref().unwrap().active_body().unwrap() as usize;
    let key = |key, state: &Session| {
        command(crate::combat_controls::shortcut(key, state, &content, actor).unwrap())
    };
    let start = state.clone();
    // Native turn-script17 emits -120 after replacing the mutant; it cannot block input.
    let text = content.map(0).unwrap().dialogue(-120).unwrap();
    for (presentation, pressed) in [
        (
            Presentation::Messages {
                pages: vec![(-120, text.clone())],
                next: None,
            },
            false,
        ),
        (Presentation::Dialogue { id: 120, text }, true),
    ] {
        let mut message = start.clone();
        message.exploration.as_mut().unwrap().presentation = Some(presentation);
        let bytes = serde_json::to_vec(&message).unwrap();
        let mut resumed: Session = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(
            crate::input::story_action(&resumed, false).is_none(),
            pressed
        );
        let action = crate::input::story_action(&resumed, pressed).unwrap();
        apply(command(action), &mut resumed, &content).unwrap();
        assert!(resumed.warriors == start.warriors && resumed.random == start.random);
        let position = resumed.exploration.as_ref().unwrap();
        assert!(resumed.combat == start.combat && position.presentation.is_none());
    }
    apply(
        Command::ToggleCombatAuto { warrior: actor },
        &mut state,
        &content,
    )
    .unwrap();
    assert_eq!(
        centurions_core::combat::runtime::controlled_warrior(&state),
        Some(actor)
    );
    apply(
        Command::ToggleCombatAuto { warrior: actor },
        &mut state,
        &content,
    )
    .unwrap();
    let mut automated = state.clone();
    apply(key(KeyCode::A, &automated), &mut automated, &content).unwrap();
    assert!(
        automated
            .combat
            .as_ref()
            .unwrap()
            .automatic
            .iter()
            .all(|v| !v)
    );
    apply(Command::RunControlledTurn, &mut automated, &content).unwrap();
    assert!(automated != state, "Auto performed no action");
    let mut app = Application::at(Screen::Game, 0.);
    for action in [
        ApplicationAction::CombatInformation(actor as i16),
        ApplicationAction::InformationItem(1),
        ApplicationAction::CombatInformation(10),
        ApplicationAction::CreatureItem(4),
    ] {
        app.handle(action, &mut state, &content).unwrap();
    }
    assert!(
        state == start && !app.blocks_game(),
        "viewing changed state or stranded focus"
    );
    for (offset, value) in [
        (0x40, 4_i16),
        (0x12, 4),
        (0x1e, 0),
        (0x3c, 20),
        (0x3e, 20),
        (0x60, 0),
        (0x10, 0),
        (0x98, 0),
    ] {
        state.warriors[actor].bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
    }
    let origin = state.combat.as_ref().unwrap().squad.positions[actor];
    state.combat.as_mut().unwrap().turn_origin = Some(origin);
    let c = state.combat.as_ref().unwrap();
    let point = (origin.0 + 1, origin.1);
    let cell = centurions_content::mission::OpeningMission::index(
        i32::from(point.0 + c.map.shift.0 + 5),
        i32::from(point.1 + c.map.shift.1 + 5),
    )
    .unwrap();
    let ground = c.squad.under_tiles[actor];
    state.combat.as_mut().unwrap().map.tiles[cell] = ground;
    apply(Command::Move { dx: 1, dy: 0 }, &mut state, &content).unwrap();
    assert_eq!(state.combat.as_ref().unwrap().squad.positions[actor], point);
    let random = state.random.clone();
    apply(key(KeyCode::U, &state), &mut state, &content).unwrap();
    assert_eq!(
        state.combat.as_ref().unwrap().squad.positions[actor],
        origin
    );
    assert_eq!(state.random, random);
    let c = state.combat.as_ref().unwrap();
    assert_eq!(c.map.tiles[cell], ground);
    let before = state.clone();
    let path =
        std::env::temp_dir().join(format!("centurions-controls-{}.json", std::process::id()));
    save(&path, &state, &content).unwrap();
    assert!(
        load(&path, &content).unwrap() == state,
        "combat controls did not survive save"
    );
    std::fs::remove_file(path).unwrap();
    state.warriors[actor].bytes[0x3c..0x3e].copy_from_slice(&1_i16.to_be_bytes());
    let refused = state.clone();
    assert!(apply(key(KeyCode::D, &state), &mut state, &content).is_err());
    assert!(state == refused, "refused Delay changed state");
    state = before;
    apply(key(KeyCode::D, &state), &mut state, &content).unwrap();
    assert_eq!(state.combat.as_ref().unwrap().order[109], actor as i16);
    assert_ne!(
        state.combat.as_ref().unwrap().active_body(),
        Some(actor as i16)
    );
    state = start.clone();
    let ap = word(&state.warriors[actor].bytes, 0x40).unwrap();
    let slot = controls::weapons(&state).unwrap()[0];
    apply(Command::SelectCombatWeapon { slot }, &mut state, &content).unwrap();
    assert_eq!(word(&state.warriors[actor].bytes, 0x40).unwrap(), ap - 1);
    state = start;
    let c = state.combat.as_mut().unwrap();
    c.enemies.positions[0] = origin;
    let refused = state.clone();
    assert!(apply(key(KeyCode::E, &state), &mut state, &content).is_err());
    assert!(state == refused, "refused Escape changed state");
    let c = state.combat.as_mut().unwrap();
    c.enemies
        .positions
        .fill((80 - c.map.shift.0, 80 - c.map.shift.1));
    let before = state.clone();
    let events = apply(key(KeyCode::E, &state), &mut state, &content).unwrap();
    app.observe(&events);
    assert!(app.blocks_game());
    app.handle(ApplicationAction::EscapeAnswer(false), &mut state, &content)
        .unwrap();
    assert!(
        state == before && !app.blocks_game(),
        "Escape cancellation changed state"
    );
    let events = apply(Command::RequestCombatEscape, &mut state, &content).unwrap();
    app.observe(&events);
    app.handle(ApplicationAction::EscapeAnswer(true), &mut state, &content)
        .unwrap();
    assert!(
        state
            .combat
            .as_ref()
            .is_none_or(|c| c.squad.positions[actor] == (-1, -1))
    );
    assert!(!app.blocks_game());
}
