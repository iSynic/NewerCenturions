use super::*;

#[path = "../../../core/tests/support/native_save_receipts.rs"]
#[cfg(feature = "original-fixtures")]
mod native_receipts;

// Protects multiple saves, cancellation, Fast Save selection, and failed-load/original-file isolation.
#[test]
#[cfg(feature = "original-fixtures")]
fn selected_files_and_cancellation_preserve_the_session_and_originals() {
    let content = crate::original_content::load();
    let mut state = Session::new(&content);
    let directory =
        std::env::temp_dir().join(format!("centurions-save-dialog-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let first = directory.join("first.json");
    let second = directory.join("second.json");
    let original = directory.join("original");
    std::fs::write(&original, b"original bytes").unwrap();
    let mut files = SaveFiles {
        current: None,
        default: directory.join("session.json"),
        explicit: None,
        protected: vec![original.clone()],
    };
    let initial = state.clone();
    assert!(
        !files
            .perform(&Action::Save, &mut state, &content, |_, _| Ok(None))
            .unwrap()
    );
    assert!(files.current.is_none());
    files
        .perform(&Action::Save, &mut state, &content, |saving, _| {
            assert!(saving);
            Ok(Some(first.clone()))
        })
        .unwrap();
    centurions_core::apply(
        centurions_core::Command::ImportStarters,
        &mut state,
        &content,
    )
    .unwrap();
    let party = state.clone();
    files
        .perform(&Action::Save, &mut state, &content, |_, _| {
            Ok(Some(second.clone()))
        })
        .unwrap();
    files
        .perform(&Action::FastSave, &mut state, &content, |_, _| {
            panic!("Fast Save reopened the picker")
        })
        .unwrap();
    files
        .perform(&Action::Load, &mut state, &content, |saving, _| {
            assert!(!saving);
            Ok(Some(first.clone()))
        })
        .unwrap();
    assert_eq!(state, initial);
    assert!(
        !files
            .perform(&Action::Load, &mut state, &content, |_, _| Ok(None))
            .unwrap()
    );
    files
        .perform(&Action::Load, &mut state, &content, |_, _| {
            Ok(Some(second.clone()))
        })
        .unwrap();
    assert_eq!(state, party);
    let protected_save = directory.join("nested/new.json");
    files.protected.push(directory.clone());
    assert!(
        files
            .perform(&Action::Save, &mut state, &content, |_, _| Ok(Some(
                protected_save.clone()
            )))
            .is_err()
    );
    assert!(!protected_save.exists());
    assert!(!files.default.exists());
    files.protected.clear();
    for action in [Action::Load, Action::Save] {
        assert!(
            files
                .perform(&action, &mut state, &content, |_, _| Ok(Some(
                    original.clone()
                )))
                .is_err()
        );
        assert_eq!(state, party);
        assert_eq!(files.current.as_ref(), Some(&second));
        assert_eq!(std::fs::read(&original).unwrap(), b"original bytes");
    }
    let (native, information, _) = native_receipts::write(&directory, &content);
    state.random = native.random;
    files
        .perform(&Action::Load, &mut state, &content, |_, _| {
            Ok(Some(directory.join("Data I1")))
        })
        .unwrap();
    assert!(files.current.is_none());
    let imported = state.clone();
    assert!(
        !files
            .perform(&Action::FastSave, &mut state, &content, |saving, _| {
                assert!(saving);
                Ok(None)
            })
            .unwrap()
    );
    assert_eq!(state, imported);
    files
        .perform(&Action::FastSave, &mut state, &content, |_, _| {
            Ok(Some(first.clone()))
        })
        .unwrap();
    assert_eq!(load(&first, &content).unwrap(), imported);
    std::fs::write(directory.join("Data F1"), [0]).unwrap();
    assert!(
        files
            .perform(&Action::Load, &mut state, &content, |_, _| Ok(Some(
                directory.join("Data I1")
            )))
            .is_err()
    );
    assert_eq!(state, imported);
    assert_eq!(files.current.as_ref(), Some(&first));
    assert_eq!(
        std::fs::read(directory.join("Data I1")).unwrap(),
        information
    );
    native_receipts::remove(&directory);
    for file in [first, second, original] {
        std::fs::remove_file(file).unwrap();
    }
    std::fs::remove_dir(directory).unwrap();
}

// Runs on every platform without original assets: named saves, atomic replacement and rollback.
#[test]
fn portable_named_save_roundtrip_and_failed_load() {
    let content = Content {
        identity: "portable-save".into(),
        tutorials: vec![],
        warriors: vec![],
        items: vec![],
        resources: vec![],
        original_services: None,
        scenario: None,
    };
    let mut state = Session::new(&content);
    let directory =
        std::env::temp_dir().join(format!("centurions-portable-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let mut files = SaveFiles {
        current: None,
        default: directory.join("first 'α'.json"),
        explicit: None,
        protected: vec![],
    };
    let first = files.default.clone();
    files
        .perform(&Action::Save, &mut state, &content, |_, _| {
            Ok(Some(first.clone()))
        })
        .unwrap();
    let original = state.clone();
    state.random.seed = state.random.seed.wrapping_add(1);
    let second = directory.join("second.json");
    files
        .perform(&Action::Save, &mut state, &content, |_, _| {
            Ok(Some(second.clone()))
        })
        .unwrap();
    files
        .perform(&Action::FastSave, &mut state, &content, |_, _| {
            panic!("Fast Save reopened dialog")
        })
        .unwrap();
    files
        .perform(&Action::Load, &mut state, &content, |_, _| {
            Ok(Some(first.clone()))
        })
        .unwrap();
    assert_eq!(state, original);
    assert!(
        !files
            .perform(&Action::Save, &mut state, &content, |_, _| Ok(None))
            .unwrap()
    );
    std::fs::write(&second, b"incomplete save").unwrap();
    assert!(
        files
            .perform(&Action::Load, &mut state, &content, |_, _| Ok(Some(
                second.clone()
            )))
            .is_err()
    );
    assert_eq!(state, original);
    assert_eq!(files.current.as_ref(), Some(&first));
    std::fs::remove_file(first).unwrap();
    std::fs::remove_file(second).unwrap();
    std::fs::remove_dir(directory).unwrap();
}
