#[test]
fn required_disease_attack_matches_original_complete_records_and_random_consumption() {
    let fixture: serde_json::Value = crate::support::fixture("required-natural.json");
    crate::binary_natural_poison::compare_poison_natural_attack_matches_original_complete_records_and_random_consumption(fixture);
}
