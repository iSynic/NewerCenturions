use crate::prelude::*;

#[test]
fn weapon_zero_uses_retained_packet_and_global_spell_identity() {
    let f: serde_json::Value = crate::support::fixture("enemy-weapon-zero-caller.json");

    compare_cases(f.array("cases"));
}
fn compare_cases(cases: &[Value]) {
    crate::enemy_callers::compare(cases, crate::enemy_callers::Caller::RetainedSpell);
}
