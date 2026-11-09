//! The combat Load control fits the reserve chosen in its popup for one action point and one
//! reload sound.
use crate::{prelude::*, support};

#[test]
fn load_control_fits_the_chosen_reserve_for_one_action_point_and_one_sound() {
    let content = content();
    let mut state = crate::native_common::loaded_shot_frame(0);
    state.ammunition[81] = 5;
    let control = centurions_core::items::load_control(&state, 4);
    assert_eq!(
        control.unwrap(),
        centurions_core::items::LoadControl::Popup { slot: 0 }
    );
    // Unload, then the compatible reserves 80 and 81: the second is item 3.
    let command = Command::ChooseCombatAmmunition {
        warrior: 4,
        slot: 0,
        item: 3,
    };
    let mut draws = state.random.clone();
    let sound = 403 + draws.bound(2);
    let mut resumed = support::checkpoint(&state, &content, "load-control");
    let events = replay_command(command, &mut state, &mut resumed, &content);
    assert_eq!(support::event_sounds(&events), [sound]);
    assert_eq!(state.random, draws);
    let actor = &state.warriors[4];
    assert_eq!(actor.record().action_points().unwrap(), 4);
    assert_eq!(actor.inventory().unwrap()[0].ammunition_id().unwrap(), 281);
    assert_eq!((state.ammunition[80], state.ammunition[81]), (4, 4));
}
