use crate::prelude::*;

use centurions_core::combat::natural_conditions::disease_tick;

#[test]
fn disease_tick_matches_original_signed_guards_health_and_kill_decision() {
    let fixture: serde_json::Value = crate::support::fixture("disease-tick.json");
    for case in fixture.array("cases").iter() {
        let mut warrior = case.bytes("input");
        let tick = disease_tick(&warrior).unwrap();
        if let Some(tick) = tick {
            warrior[0x50..0x52].copy_from_slice(&tick.health.to_be_bytes());
        }
        crate::support::native_expect! {
        bytes warrior => &case["after"], "case {index}";
        scalar tick.is_some_and(|t| t.killed) => &case["killed"], "binary_disease_tick";
        }
    }
}
