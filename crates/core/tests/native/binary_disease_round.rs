#[test]
fn disease_round_matches_original_healing_death_cleanup_duration_and_bleeding() {
    verify(include_str!(
        "../../../../evidence/fixtures/disease-round.json"
    ));
}
use crate::native_common::compare_health_round as verify;
