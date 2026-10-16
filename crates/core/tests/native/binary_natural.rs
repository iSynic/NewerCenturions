#[test]
fn natural_attack_matches_original_routine() {
    crate::binary_natural_poison::compare_poison_natural_attack_matches_original_complete_records_and_random_consumption(
        crate::support::fixture("natural.json"),
    );
}
