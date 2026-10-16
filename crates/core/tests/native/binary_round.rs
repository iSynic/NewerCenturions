use crate::native_common::compare_health_round as verify;

#[test]
fn round_transition_matches_original_reduce_and_heal() {
    verify(include_str!("../../../../evidence/fixtures/round.json"));
}

#[test]
fn bleeding_round_matches_original_records_and_death_randomness() {
    verify(include_str!(
        "../../../../evidence/fixtures/bleeding-round.json"
    ));
}
