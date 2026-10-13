use super::*;

// Protects committing the selected level while an existing mission remains untouched during setup.
#[test]
fn difficulty_controls_commit_all_original_levels_at_mission_start() {
    let content = crate::original_content::load();
    let mut state = Session::new(&content);
    for level in -2..=2 {
        let before = state.clone();
        let mut setup = Application::at(Screen::Squad, 0.);
        setup.squad = vec![0];
        setup
            .handle(ApplicationAction::Difficulty(level), &mut state, &content)
            .unwrap();
        assert_eq!(state, before);
        setup
            .handle(ApplicationAction::Done, &mut state, &content)
            .unwrap();
        assert_eq!(state.difficulty, level);
        assert_eq!(state.phase, Phase::Exploration);
    }
}

// Protects original editor activation, cancellation, undo, commit and saved continuity.
#[test]
fn editor_cancel_undo_member_navigation_and_saved_commit_preserve_native_scope() {
    let content = crate::original_content::load();
    let mut state = Session::new(&content);
    let mut app = Application::at(Screen::Title, 0.);
    app.handle(ApplicationAction::Menu(136, 8), &mut state, &content)
        .unwrap();
    app.handle(ApplicationAction::EditorAnswer(false), &mut state, &content)
        .unwrap();
    assert!(!app.editing);
    app.handle(ApplicationAction::Menu(136, 8), &mut state, &content)
        .unwrap();
    app.handle(ApplicationAction::EditorAnswer(true), &mut state, &content)
        .unwrap();
    apply(Command::ImportStarters, &mut state, &content).unwrap();
    apply(
        Command::EnterMission {
            name: "The Thunder of Guns".into(),
        },
        &mut state,
        &content,
    )
    .unwrap();
    app.screen = Screen::Game;
    let before = state.clone();
    for item in [45, 82, 45, 83, 45, 1] {
        if app.editor.is_none() {
            app.handle(ApplicationAction::OpenEditor(0), &mut state, &content)
                .unwrap();
        }
        app.handle(ApplicationAction::EditorItem(item), &mut state, &content)
            .unwrap();
        assert_eq!(state, before);
        if matches!(item, 82 | 83) {
            assert_eq!(app.editor.as_ref().unwrap().draft, before);
        }
    }
    app.handle(ApplicationAction::OpenEditor(0), &mut state, &content)
        .unwrap();
    app.handle(ApplicationAction::EditorItem(45), &mut state, &content)
        .unwrap();
    app.handle(ApplicationAction::EditorItem(4), &mut state, &content)
        .unwrap();
    assert_eq!(
        word(&state.warriors[0].bytes, 0x9e2).unwrap(),
        word(&before.warriors[0].bytes, 0x9e2).unwrap() + 1
    );
    assert_eq!(state.random, before.random);
    let first_committed = state.clone();
    app.handle(ApplicationAction::OpenEditor(0), &mut state, &content)
        .unwrap();
    for item in [45, 85, 45, 1] {
        app.handle(ApplicationAction::EditorItem(item), &mut state, &content)
            .unwrap();
    }
    assert_eq!(state.warriors[1], first_committed.warriors[1]);
    assert_eq!(
        word(&state.warriors[0].bytes, 0x9e2).unwrap(),
        word(&first_committed.warriors[0].bytes, 0x9e2).unwrap() + 1
    );
    let path = std::env::temp_dir().join(format!("centurions-editor-{}.json", std::process::id()));
    let before_levels = state.clone();
    app.handle(ApplicationAction::OpenEditor(0), &mut state, &content)
        .unwrap();
    for item in [81, 81, 81, 0, 83, 4] {
        app.handle(ApplicationAction::EditorItem(item), &mut state, &content)
            .unwrap();
    }
    assert_eq!(
        word(&state.warriors[0].bytes, 0x3a).unwrap(),
        word(&before_levels.warriors[0].bytes, 0x3a).unwrap() + 3
    );
    assert_eq!(
        state.library_random_draws,
        before_levels.library_random_draws + 3
    );
    assert_eq!(
        &state.warriors[0].bytes[0x9ce..0x9d2],
        &before_levels.warriors[0].bytes[0x9ce..0x9d2]
    );
    let before_scans = state.clone();
    app.handle(ApplicationAction::OpenEditor(1), &mut state, &content)
        .unwrap();
    for item in [39, 2, 10, 11, 32] {
        app.handle(ApplicationAction::EditorItem(item), &mut state, &content)
            .unwrap();
    }
    assert_eq!(state, before_scans);
    assert!(
        app.editor
            .as_ref()
            .unwrap()
            .scans
            .as_ref()
            .unwrap()
            .question
    );
    app.handle(ApplicationAction::ScanAnswer(false), &mut state, &content)
        .unwrap();
    assert!(app.editor.as_ref().unwrap().scans.is_some());
    app.handle(ApplicationAction::EditorItem(32), &mut state, &content)
        .unwrap();
    app.handle(ApplicationAction::ScanAnswer(true), &mut state, &content)
        .unwrap();
    assert_eq!(state, before_scans);
    app.handle(ApplicationAction::EditorItem(4), &mut state, &content)
        .unwrap();
    assert_eq!(&state.warriors[1].bytes[0x9f0..0x9f2], &[0, 1]);
    assert_eq!(
        &state.psi_memory.as_ref().unwrap()[80..160],
        &state.warriors[1].bytes[0xa76..0xac6]
    );
    assert_eq!(
        &state.psi_memory.as_ref().unwrap()[..80],
        &before_scans.psi_memory.as_ref().unwrap()[..80]
    );
    assert_eq!(state.random, before_scans.random);
    assert_eq!(
        state.library_random_draws,
        before_scans.library_random_draws
    );
    centurions_core::save(&path, &state, &content).unwrap();
    assert_eq!(centurions_core::load(&path, &content).unwrap(), state);
    std::fs::remove_file(path).unwrap();
}

// Protects item-dialog commit scope, equipped modifiers and saved warrior provenance.
#[test]
fn item_editor_commits_equipped_changes_and_preserves_saved_provenance() {
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
    let mut app = Application::at(Screen::Game, 0.);
    app.editing = true;
    let before = state.clone();
    app.handle(
        ApplicationAction::OpenItemEditor(0, 0),
        &mut state,
        &content,
    )
    .unwrap();
    app.handle(ApplicationAction::ItemEditorItem(54), &mut state, &content)
        .unwrap();
    assert_eq!(state, before);
    let draft = app.item_editor.as_ref().unwrap().draft.clone();
    assert!(
        app.handle(ApplicationAction::ItemEditorItem(39), &mut state, &content)
            .is_err()
    );
    assert_eq!(app.item_editor.as_ref().unwrap().draft, draft);
    app.handle(ApplicationAction::ItemEditorItem(2), &mut state, &content)
        .unwrap();
    assert_eq!(state, draft);
    assert_eq!(
        state.warriors[0].original_sha256,
        before.warriors[0].original_sha256
    );
    let path = std::env::temp_dir().join(format!(
        "centurions-item-editor-{}.json",
        std::process::id()
    ));
    centurions_core::save(&path, &state, &content).unwrap();
    assert_eq!(centurions_core::load(&path, &content).unwrap(), state);
    std::fs::remove_file(path).unwrap();
}
