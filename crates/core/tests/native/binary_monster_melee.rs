#[test]
fn monster_default_melee_matches_native_full_records_packet_and_randomness() {
    let fixture = crate::support::fixture("monster-melee.json");

    crate::binary_endgame_melee_bodies::compare_endgame_default194_melee_matches_native_full_records_packet_and_randomness(fixture);
}
