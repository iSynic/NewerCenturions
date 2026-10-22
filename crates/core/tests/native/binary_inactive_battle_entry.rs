use crate::{prelude::*, support};
use support::assert_bytes;

#[test]
fn inactive_battle_entry_matches_original_complete_records_formation_order_and_rng() {
    let content = support::content();
    let fixture: serde_json::Value = support::fixture("inactive-battle-entry.json");
    let cases = fixture.array("cases");

    for case in cases {
        let mut state = support::mission_session(&content);
        state.warriors = support::warriors(&case["warriors_before"]);
        if case["seed_before"] == 12 {
            state.native_work.health_context = Some(centurions_core::story_health::context(
                &state.warriors,
                0,
                None,
            ));
            support::expect(
                state
                    .native_work
                    .health_context
                    .as_ref()
                    .unwrap()
                    .unavailable,
                &case["unavailable"],
            );
        }
        let map = content.map(case.n::<i32>("map")).unwrap();
        let position = state.exploration.as_mut().unwrap();
        position.map = case.n::<i32>("map");
        position.x = case.n::<i32>("x");
        position.y = case.n::<i32>("y");
        position.floor.clone_from(&map.floor);
        position.events.clone_from(&map.events);
        crate::native_common::keep_tile_set(&mut state, &content);
        state.random = RandomState::new(case.n::<u32>("seed_before"));
        begin_battle(&mut state, &content, case.n::<i16>("battle"), 0).unwrap();
        crate::binary_battle_entry::assert_battle(&state, case);
    }
}
#[test]
fn ordinary_battle_return_restores_all_participation_after_reward_boundary() {
    let fixture: serde_json::Value = support::fixture("inactive-battle-return.json");
    let cases = fixture.array("cases");

    for case in cases.iter().filter(|case| case["aborted"] == false) {
        let before: Vec<_> = support::warriors(&case["before"]);
        let returned = centurions_core::story_party::group_return(&before, false, 0).unwrap();
        let actual: Vec<_> = support::record_bytes(&returned.warriors);
        assert_bytes(&actual, &case.bytes("after"), "ordinary return warriors");
    }
}
