use super::{Request, WindowMode};

// Protects the escape route from fullscreen and prevents held Alt+Enter toggling every frame.
#[test]
fn fullscreen_shortcut_and_resolution_changes_restore_windowed_mode() {
    let mut mode = WindowMode::default();
    assert!(!mode.fullscreen);
    assert!(!mode.shortcut(false, true));
    assert!(mode.shortcut(true, true));
    assert_eq!(mode.change(true, None), [Request::Fullscreen(true)]);
    assert!(!mode.shortcut(true, true));
    assert!(!mode.shortcut(true, false));
    assert!(mode.shortcut(true, true));
    assert_eq!(mode.change(false, None), [Request::Fullscreen(false)]);
    assert_eq!(mode.change(true, None), [Request::Fullscreen(true)]);
    assert_eq!(
        mode.change(true, Some((640, 480))),
        [Request::Fullscreen(false), Request::Size(640, 480)]
    );
    assert_eq!(
        mode.change(false, Some((800, 600))),
        [Request::Size(800, 600)]
    );
    assert!(mode.change(false, None).is_empty());
}
