use crate::prelude::*;

use centurions_core::combat::calculate_movement;

#[derive(Deserialize)]
struct Fixture {
    cases: Vec<Case>,
}
#[derive(Deserialize)]
struct Case {
    input: String,
    output: String,
    race_movement: i8,
}
#[test]
fn movement_matches_instructions_with_bounded_sane_boundary() {
    let fixture: Fixture =
        serde_json::from_value(crate::support::fixture("combat-movement.json")).unwrap();
    for case in fixture.cases {
        let mut warrior = Warrior::read(&decode(&case.input)).unwrap();
        calculate_movement(&mut warrior, case.race_movement).unwrap();
        assert_eq!(warrior.bytes, decode(&case.output), "{}", warrior.name);
    }
}
