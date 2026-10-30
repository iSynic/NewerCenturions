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

// Wrapped zero capacity divides by zero; the modern IEEE policy matches the captured FPU route.
#[test]
fn zero_capacity_movement_follows_the_modern_ieee_policy() {
    let fpu = support::fixture("sane-fpu-nonarithmetic.json");
    for case in support::fixture("movement-boundary.json").array("cases") {
        let mut warrior = Warrior::read(&case.bytes("warrior_before")).unwrap();
        calculate_movement(&mut warrior, case.n("race_movement")).unwrap();
        let mut rows = fpu.array("movement").iter();
        let row = rows.find(|r| r["route"] == "FPU" && r["case"] == case["case"]);
        let (record, case) = (warrior.record(), case.text("case"));
        let actual = (record.load_max().unwrap(), record.movement_max().unwrap());
        assert_eq!(actual, (row.unwrap().n("capacity"), row.unwrap().n("movement")), "{case}");
    }
}
