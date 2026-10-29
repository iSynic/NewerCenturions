//! Ordinary scheduled caller: native consumption applies, item script return is ignored.
use crate::{prelude::*, support};

#[test]
fn scheduled_typed_script_item_returns_once_without_decoding_the_manual_return() {
    let content = content();
    let case = &support::fixture("encounter-script-items.json")["cases"][0];
    let mut state = support::mission_session(&content);
    state.warriors = support::warriors(&case["before"]);
    let record = support::script_record(&[(5, case.n("encounter"))]);
    support::pause_script(&mut state, record, 0);
    support::saved_command(Command::Continue, &mut state, &content);
    state
        .native_work
        .party_mask
        .copy_from_slice(&case.words("mask_before"));
    let before = state.clone();
    let events = support::saved_command(
        Command::UseItem {
            warrior: case.n("member"),
            slot: case.n("slot"),
        },
        &mut state,
        &content,
    );
    assert_eq!(support::record_bytes(&state.warriors), case.bytes("after"));
    assert_eq!(state.random, before.random);
    assert_eq!(state.native_work.party_mask.as_slice(), case.words("mask"));
    assert_eq!(state.selected_warrior, before.selected_warrior);
    assert_eq!(support::event_sounds(&events), case.n::<Vec<i16>>("sounds"));
    assert!(state.exploration.as_ref().unwrap().script.is_none());
    assert!(state.exploration.as_ref().unwrap().presentation.is_none());
    assert!(!events.contains(&Event::StoryPresented));
}
