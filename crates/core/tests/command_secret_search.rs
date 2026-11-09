//! The passive search for secrets, through the saved Continue command.
use crate::{prelude::*, support};

/// An inertial shield makes the walking party find every secret next to it, however poor its skill.
#[test]
fn inertial_shield_reveals_adjacent_secrets_at_zero_skill() {
    let content = content();
    let mut state = support::mission_session(&content);
    state.party_condition[5] = 1;
    for warrior in &mut state.warriors {
        warrior.bytes[0xfc..0xfe].fill(0);
        warrior.bytes[0xb6a..0xb6c].fill(0);
    }
    let position = state.exploration.as_mut().unwrap();
    let index = OpeningMission::index(position.x, position.y).unwrap();
    position.floor[index] = 3000;
    apply(Command::Move { dx: 1, dy: 0 }, &mut state, &content).unwrap();
    assert_eq!(state.exploration.unwrap().floor[index], 2000);
}
