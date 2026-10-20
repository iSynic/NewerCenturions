use super::*;

// Cancellation and helper failure must never select a stale or unintended save file.
#[test]
fn dialog_results_distinguish_selection_cancellation_and_failure() {
    let path = std::env::temp_dir().join("warrior 'α'.json");
    let output = format!("{}\n", path.display());
    assert_eq!(
        linux_selection(Some(0), output.as_bytes(), b"").unwrap(),
        Some(path)
    );
    assert_eq!(
        linux_selection(Some(1), b"stale.json\n", b"").unwrap(),
        None
    );
    assert!(linux_selection(Some(0), b"relative.json\n", b"").is_err());
    assert!(linux_selection(Some(0), b"\n", b"").is_err());
    assert!(linux_selection(Some(2), b"stale.json\n", b"display unavailable").is_err());
}
