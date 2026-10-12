fn fixture() -> serde_json::Value {
    crate::support::fixture("parts-weapons.json")
}
#[test]
fn required_parts_enemy_preparation_matches_native_item_context_and_packet() {
    let fixture = fixture();

    crate::binary_endgame_weapons::compare_required_endgame_enemy_preparation_matches_native_item_context_and_packet(fixture);
}

#[test]
fn required_parts_enemy_firearms_match_whole_party_and_native_randomness() {
    let fixture = fixture();

    crate::binary_endgame_weapons::compare_required_endgame_enemy_firearms_match_whole_party_and_native_randomness(fixture);
}

#[test]
fn required_parts_default_melee_selection_matches_native_records_saves_and_both_attack_indices() {
    let fixture = fixture();

    crate::binary_endgame_weapons::compare_required_endgame_default_melee_selection_matches_native_records_saves_and_both_attack_indices(fixture);
}

#[test]
fn required_parts_adjacent_melee_caller_matches_native_action_count_and_cost() {
    let fixture = fixture();

    crate::binary_endgame_weapons::compare_required_endgame_adjacent_melee_caller_matches_native_action_count_and_cost(fixture);
}

#[test]
fn required_parts_enemy_firearms_match_native_friendly_body_targets() {
    let fixture = fixture();

    crate::binary_endgame_weapons::compare_required_endgame_enemy_firearms_match_native_friendly_body_targets(fixture);
}
