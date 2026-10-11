use centurions_content::Warrior;
use centurions_core::combat::calculate_movement;
use serde::Deserialize;

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
fn decode(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}
#[test]
fn movement_matches_instructions_with_bounded_sane_boundary() {
    let fixture: Fixture = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/combat-movement.json"
    ))
    .unwrap();
    for case in fixture.cases {
        let mut warrior = Warrior::read(&decode(&case.input)).unwrap();
        calculate_movement(&mut warrior, case.race_movement).unwrap();
        assert_eq!(warrior.bytes, decode(&case.output), "{}", warrior.name);
    }
}
