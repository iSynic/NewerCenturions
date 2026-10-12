#[test]
fn monster_elemental_matches_native_signed_conditions_and_whole_caller() {
    let fixture = crate::support::fixture("monster-elemental.json");

    crate::binary_monster_disease::compare(fixture, true);
}
