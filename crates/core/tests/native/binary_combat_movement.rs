use crate::{prelude::*, support};

use centurions_core::combat::calculate_movement;

#[test]
fn movement_matches_instructions_with_bounded_sane_boundary() {
    for case in support::fixture("combat-movement.json").array("cases") {
        let mut warrior = Warrior::read(&case.bytes("input")).unwrap();
        calculate_movement(&mut warrior, case.n("race_movement")).unwrap();
        assert_eq!(warrior.bytes, case.bytes("output"), "{}", warrior.name);
    }
}
