fn fixture() -> serde_json::Value {
    crate::support::fixture("monster-melee.json")
}

#[test]
fn monster_default_melee_matches_native_full_records_packet_and_randomness() {
    let fixture = fixture();

    crate::binary_endgame_melee_bodies::compare_endgame_default194_melee_matches_native_full_records_packet_and_randomness(fixture);
}
