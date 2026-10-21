use centurions_core::Session;
use serde_json::Value;

/// Historical native snapshots do not record WORD4435d4. All their existing
/// fields remain exact; the added word has separate original command/save controls.
pub fn captured_session(state: &Session) -> Session {
    let mut captured = state.clone();
    captured.weapon_work.natural_attack_index = None;
    if let Some(combat) = &mut captured.combat {
        combat.weapon_work.natural_attack_index = None;
    }
    captured
}

/// Build a pre-13 migration specimen without the newly serialized native word.
pub fn legacy_save(document: &mut Value) {
    let state = serde_json::from_value(document["session"].clone()).unwrap();
    document["session"] = serde_json::to_value(captured_session(&state)).unwrap();
}
