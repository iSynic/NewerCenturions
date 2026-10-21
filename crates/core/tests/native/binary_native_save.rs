use crate::{prelude::*, support};

#[path = "../support/native_save_receipts.rs"]
mod receipts;

// Protects original-file import against layout drift and incomplete multi-file saves.
#[test]
fn native_writer_records_import_read_only_and_resume_as_versioned_saves() {
    let content = support::content();
    let directory =
        std::env::temp_dir().join(format!("centurions-native-save-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("Data I1");
    let (original, info, native) = receipts::write(&directory, &content);
    let imported = centurions_core::native_save::import(
        &path,
        &content,
        &original.random,
        (original.library_random_seed, original.library_random_draws),
    )
    .unwrap();
    assert_eq!(
        support::record_bytes(&imported.warriors),
        support::record_bytes(&original.warriors)
    );
    assert_eq!(imported.mission, original.mission);
    assert_eq!(imported.ammunition, original.ammunition);
    assert_eq!(imported.crate_items, original.crate_items);
    assert_eq!(imported.flags, original.flags);
    assert_eq!(imported.allies, original.allies);
    assert_eq!(imported.psi_memory, original.psi_memory);
    assert_eq!(imported.random, original.random);
    assert_eq!(imported.mission_score, original.mission_score);
    let position = imported.exploration.as_ref().unwrap();
    let before = original.exploration.as_ref().unwrap();
    assert_eq!(
        (
            position.map,
            position.x,
            position.y,
            position.day,
            position.hour,
            position.minute
        ),
        (
            before.map,
            before.x,
            before.y,
            before.day,
            before.hour,
            before.minute
        )
    );
    assert_eq!(position.map_state().native_bytes().unwrap(), native);
    let reader = support::fixture("native-load.json");
    assert_eq!(reader["return"].as_i64().unwrap(), 1);
    assert_eq!(
        support::record_bytes(&imported.warriors),
        reader.bytes("warriors")
    );
    let read_map = ["478e42", "443086", "4358e6", "4085b8", "40c500", "414390"]
        .into_iter()
        .flat_map(|field| reader.bytes(field))
        .collect::<Vec<_>>();
    assert_eq!(position.map_state().native_bytes().unwrap(), read_map);
    assert_eq!(
        imported.selected_warrior,
        reader["selected"].as_u64().unwrap() as usize
    );
    let converted = directory.join("converted.json");
    save(&converted, &imported, &content).unwrap();
    assert_eq!(load(&converted, &content).unwrap(), imported);
    assert_eq!(std::fs::read(&path).unwrap(), info);
    std::fs::write(directory.join("Data F1"), [0]).unwrap();
    assert!(
        centurions_core::native_save::import(&path, &content, &original.random, (1, 0)).is_err()
    );
    assert_eq!(std::fs::read(&path).unwrap(), info);
    receipts::remove(&directory);
    std::fs::remove_file(converted).unwrap();
    std::fs::remove_dir(directory).unwrap();
}
