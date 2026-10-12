use crate::prelude::*;

#[test]
fn enemy_firearm_whole_caller_matches_native_cast_order_records_and_work_context() {
    let f: serde_json::Value = crate::support::fixture("enemy-fire-caller.json");

    crate::binary_enemy_fire_caller::compare_enemy_firearm_whole_caller_matches_native_cast_order_records_and_work_context(f);
}
pub(super) fn compare_enemy_firearm_whole_caller_matches_native_cast_order_records_and_work_context(
    f: Value,
) {
    crate::enemy_callers::compare(f.array("cases"), crate::enemy_callers::Caller::Firearm);
}
