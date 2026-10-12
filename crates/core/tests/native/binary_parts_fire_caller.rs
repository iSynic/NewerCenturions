#[test]
fn required_parts_firearm_whole_caller_matches_native_cast_order_records_and_work_context() {
    let f: serde_json::Value = crate::support::fixture("parts-fire-caller.json");

    crate::binary_enemy_fire_caller::compare_enemy_firearm_whole_caller_matches_native_cast_order_records_and_work_context(f);
}
