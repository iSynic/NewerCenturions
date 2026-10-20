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

// Exercised only on the hosted runner's isolated X display, never a user's desktop.
#[cfg(target_os = "linux")]
#[test]
#[ignore = "requires the isolated native-dialog driver"]
fn native_picker_save_open_and_cancel() {
    let directory = std::env::temp_dir().join(format!("centurions-picker-{}", std::process::id()));
    let path = directory.join("selected 'α'.json");
    assert_eq!(
        choose(true, &path, "Centurions picker save").unwrap(),
        Some(path.clone())
    );
    std::fs::write(&path, b"picker receipt").unwrap();
    assert_eq!(
        choose(false, &path, "Centurions picker open").unwrap(),
        Some(path.clone())
    );
    assert_eq!(
        choose(false, &path, "Centurions picker cancel").unwrap(),
        None
    );
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(directory).unwrap();
}
