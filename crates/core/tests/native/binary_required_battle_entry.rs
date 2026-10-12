#[test]
fn required_battle_entry_matches_original_complete_records_and_random_consumption() {
    let fixture: serde_json::Value = crate::support::fixture("required-battle-entry.json");
    crate::binary_battle_entry::compare_underground_battle_entry_matches_original_complete_records_and_random_consumption(fixture);
}
