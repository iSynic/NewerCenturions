use crate::prelude::*;

#[test]
fn enemy_spell_whole_caller_matches_native_casts_fields_records_and_work_context() {
    let f: serde_json::Value = crate::support::fixture("enemy-spell-caller.json");

    crate::enemy_callers::compare(f.array("cases"), crate::enemy_callers::Caller::Spell);
}

#[test]
fn enemy_spell_attempts_preserve_negative_action_points() {
    let f: serde_json::Value = crate::support::fixture("enemy-spell-caller-low-action.json");
    let cases = f.array("cases");

    assert!(cases.iter().any(|case| {
        !case["diagnostic"].as_bool().unwrap_or(false)
            && (case.bytes("records_after")[0x109] as i8) < 0
    }));
    crate::enemy_callers::compare(cases, crate::enemy_callers::Caller::Spell);
}
