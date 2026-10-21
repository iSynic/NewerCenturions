use crate::{prelude::*, support};

// Protects signed BYTE/WORD edits, clamps, equipped effects and late-failure rollback.
#[test]
fn item_editing_matches_original_complete_warriors_and_preserves_failed_state() {
    let content = support::content();
    let mut last_state = None;
    for case in support::fixture("item-editing.json").array("cases") {
        let mut state = support::mission_session(&content);
        state.warriors[0] = Warrior::read(&case.bytes("before")).unwrap();
        let before = state.clone();
        let events = apply(
            Command::EditCarriedItem {
                warrior: 0,
                slot: case["slot"].as_u64().unwrap_or(0) as usize,
                control: case.n("control"),
            },
            &mut state,
            &content,
        )
        .unwrap_or_else(|e| panic!("{}: {e}", case["case"]));
        support::native_expect! {
            bytes &state.warriors[0].bytes => &case.bytes("after"), case["case"].as_str().unwrap();
        };
        assert_eq!(events, support::sound_events(case.n::<Vec<i16>>("sounds")));
        assert_eq!(state.random, before.random);
        last_state = Some(state);
    }
    let mut state = last_state.unwrap();
    let before_failure = state.clone();
    assert!(
        apply(
            Command::EditCarriedItem {
                warrior: 0,
                slot: 1,
                control: 39
            },
            &mut state,
            &content
        )
        .is_err()
    );
    assert_eq!(state, before_failure);
}
