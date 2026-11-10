//! Command with a digit key casts a preset scan through the saved FastSpell command.
use crate::{prelude::*, support};
use centurions_core::feedback::GameFeedback::{Flash, Sound, Warning};

/// A preset that costs too much shows the refusal text, an empty one only sounds, and a payable one
/// is cast as Scan would with those settings; the Psi control warns while psi is disabled.
#[test]
fn preset_slots_refuse_cost_and_empty_then_cast() {
    let content = content();
    let mut state = support::mission_session(&content);
    let actor = state.char_select_new;
    state.globals.define_spells[actor * 80..][..8]
        .copy_from_slice(&support::word_bytes(&[0, 0, 0, 1]));
    state.warriors[actor].record_mut().set_psi_points(50);
    // The Psi control sounds before the admission test, which warns while psi is disabled.
    let press = Command::PressPsi;
    state.psi_disabled = 1;
    let refusal = vec![Sound { resource_id: 141 }, Warning { index: 113 }];
    support::refuse_command(&state, &content, press.clone(), refusal);
    state.psi_disabled = 0;
    let pressed = support::saved_command(press, &mut state, &content);
    assert_eq!(support::event_sounds(&pressed), [141]);
    state.warriors[actor].record_mut().set_psi_points(0);
    let cast = |slot, state: &mut Session| {
        support::saved_command(Command::FastSpell { slot }, state, &content)
    };
    assert!(matches!(
        &cast(1, &mut state)[..],
        [Event::Feedback(Flash {
            address: 0x403ede,
            sound: -143,
            ..
        })]
    ));
    assert_eq!(support::event_sounds(&cast(2, &mut state)), [143]);
    state.warriors[actor].record_mut().set_psi_points(50);
    cast(1, &mut state);
    assert!(state.warriors[actor].psi_points().unwrap() < 50);
    assert_eq!(
        state.outdoor_psi.as_ref().map(|pending| pending.spell),
        Some(1101)
    );
}
