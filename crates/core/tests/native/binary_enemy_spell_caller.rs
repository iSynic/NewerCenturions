use crate::prelude::*;

#[test]
fn enemy_spell_whole_caller_matches_native_casts_fields_records_and_work_context() {
    let f: serde_json::Value = crate::support::fixture("enemy-spell-caller.json");

    crate::binary_enemy_spell_caller::compare_enemy_spell_whole_caller_matches_native_casts_fields_records_and_work_context(f);
}

pub(super) fn compare_enemy_spell_whole_caller_matches_native_casts_fields_records_and_work_context(
    f: Value,
) {
    let cases = f.array("cases");

    compare_cases(cases);
}

#[test]
fn enemy_spell_attempts_preserve_negative_action_points() {
    let f: serde_json::Value = crate::support::fixture("enemy-spell-caller-low-action.json");
    let cases = f.array("cases");

    assert!(cases.iter().any(|case| {
        !case["diagnostic"].as_bool().unwrap_or(false)
            && (case.bytes("records_after")[0x109] as i8) < 0
    }));
    compare_cases(cases);
}
fn compare_cases(cases: &[Value]) {
    crate::enemy_callers::compare(cases, crate::enemy_callers::Caller::Spell);
}
