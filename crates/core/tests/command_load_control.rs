//! The combat Load control fits the reserve chosen in its popup for one action point and one
//! reload sound; closing the popup keeps what it loaded.
use crate::{prelude::*, support};
use centurions_core::items::LoadControl;

#[test]
fn load_control_fits_the_chosen_reserve_and_closing_keeps_the_popup_loads() {
    let content = content();
    let choose = |item| Command::ChooseCombatAmmunition {
        warrior: 4,
        slot: 0,
        item,
    };
    let mut state = crate::native_common::loaded_shot_frame(0);
    state.ammunition[81] = 5;
    let control = centurions_core::items::load_control(&state, 4).unwrap();
    assert!(matches!(control, LoadControl::Popup { slot: 0 }));
    // Unload, then the compatible reserves 80 and 81: the second is item 3.
    let mut draws = state.random.clone();
    let sound = 403 + draws.bound(2);
    let events = support::saved_command(choose(3), &mut state, &content);
    assert_eq!(support::event_sounds(&events), [sound]);
    assert_eq!(state.random, draws);
    let actor = &state.warriors[4];
    assert_eq!(actor.record().action_points().unwrap(), 4);
    assert_eq!(actor.inventory().unwrap()[0].ammunition_id().unwrap(), 281);
    assert_eq!((state.ammunition[80], state.ammunition[81]), (4, 4));
    // A magazine that cannot be unloaded still sounds when the popup closes unchosen.
    let mut state = crate::native_common::loaded_shot_frame(0);
    state.warriors[4].record_mut().set_inventory_quantity(0, 0);
    let draws = state.random.draws;
    let events = support::saved_command(choose(0), &mut state, &content);
    assert!(matches!(support::event_sounds(&events)[..], [404 | 405]));
    assert_eq!(state.random.draws, draws + 1);
    let last = (0..200).rev().find(|r| state.ammunition[*r] > 0).unwrap() as i16;
    let ammo = centurions_core::ammunition::definition(&content, last + 200).ok();
    assert_eq!(state.globals.loaded_items.ammo, ammo);
}
