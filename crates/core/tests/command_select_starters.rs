use centurions_content::Content;
use centurions_core::{Command, Session, apply};
use std::path::Path;

#[test]
fn invalid_starter_orders_reject_without_session_changes() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let content = Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap();
    let mut state = Session::new(&content);
    let before = state.clone();
    for indices in [
        vec![0, 1, 2, 3, 4],
        vec![0, 1, 2, 3, 4, 4],
        vec![0, 1, 2, 3, 4, 6],
    ] {
        assert!(apply(Command::SelectStarters { indices }, &mut state, &content).is_err());
        assert_eq!(state, before);
    }
}
