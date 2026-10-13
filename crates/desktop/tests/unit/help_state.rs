use super::*;

// Prevents repeated help after reload, skipped-page loss, and loss of the disable preference.
#[test]
fn tutorial_navigation_and_seen_preferences_survive_reload_and_reset() {
    let path = std::env::temp_dir().join(format!("centurions-help-{}.json", std::process::id()));
    let _ = std::fs::remove_file(&path);
    let mut help = Help::default();
    help.preferences(path.clone()).unwrap();
    help.open(121, 124).unwrap();
    assert_eq!(help.page, Some(121));
    assert_eq!(help.action(4).unwrap(), 141);
    assert_eq!(help.page, Some(121));
    help.action(5).unwrap();
    assert_eq!(help.page, Some(122));
    let mut restored = Help::default();
    restored.preferences(path.clone()).unwrap();
    restored.open(121, 124).unwrap();
    assert_eq!(restored.page, Some(123));
    assert_eq!(restored.action(3).unwrap(), 148);
    restored.open(121, 124).unwrap();
    assert_eq!(restored.page, None);
    restored.reset().unwrap();
    restored.open(121, 124).unwrap();
    restored.action(6).unwrap();
    let mut disabled = Help::default();
    disabled.preferences(path.clone()).unwrap();
    disabled.open(121, 124).unwrap();
    assert!(!disabled.enabled);
    assert_eq!(disabled.page, None);
    disabled.resume().unwrap();
    disabled.open(121, 124).unwrap();
    assert_eq!(disabled.page, Some(122));
    disabled.reset().unwrap();
    disabled.open(121, 124).unwrap();
    for page in 121..=124 {
        assert_eq!(disabled.page, Some(page));
        disabled.action(5).unwrap();
    }
    assert_eq!(disabled.page, None);
    #[cfg(feature = "original-fixtures")]
    {
        use crate::application::{Application, ApplicationAction, Screen};
        let content = crate::original_content::load();
        let mut state = centurions_core::Session::new(&content);
        let before = state.clone();
        let mut application = Application::at(Screen::Title, 0.);
        application
            .handle(ApplicationAction::Menu(130, 1), &mut state, &content)
            .unwrap();
        assert_eq!(application.help.page, Some(121));
        application
            .handle(ApplicationAction::Help(3), &mut state, &content)
            .unwrap();
        assert_eq!(application.help.page, None);
        assert_eq!(state, before);
        application.screen = Screen::Game;
        application.help.open(94, 102).unwrap();
        assert!(application.blocks_game());
        application
            .handle(ApplicationAction::Help(3), &mut state, &content)
            .unwrap();
        assert!(!application.blocks_game());
        assert_eq!(state, before);
    }
    std::fs::remove_file(path).unwrap();
}
