#[test]
fn outdoor_poison_reduction_matches_original_complete_warriors_revival_counter_and_rng() {
    let fixture: serde_json::Value = crate::support::fixture("outdoor-poison.json");

    crate::binary_outdoor_disease::compare_outdoor_disease_reduction_matches_original_complete_warriors_revival_counter_and_rng(fixture);
}
