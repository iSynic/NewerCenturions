use crate::{prelude::*, support};

use support::buffer as bytes;

#[test]
fn replacement_matches_native_full_warriors_retained_instances_costs_and_audio() {
    let fixture = support::fixture("ammunition-management.json");
    let content = content();
    let mission = content.map(0).unwrap();
    for (index, case) in fixture
        .array("cases")
        .iter()
        .enumerate()
        .filter(|(_, case)| case["kind"] == "equip")
    {
        let before = &case["before"];
        let after = &case["after"];
        let member = case.n::<usize>("member");
        let warrior = Warrior::read(&bytes(&fixture, &before["member_record"])).unwrap();
        let result = equipment::wear(
            &warrior,
            &content,
            mission,
            case.n::<usize>("slot"),
            case.n::<i64>("combat") != 0,
        )
        .unwrap_or_else(|error| panic!("replacement case {index}: {error}"));
        assert_eq!(
            result.warrior.bytes,
            bytes(&fixture, &after["member_record"]),
            "case {index}"
        );
        let mut party = bytes(&fixture, &before["party"]);
        party[member * WARRIOR_BYTES..(member + 1) * WARRIOR_BYTES]
            .copy_from_slice(&result.warrior.bytes);
        assert_eq!(party, bytes(&fixture, &after["party"]), "case {index}");
        assert_eq!(
            result.changed,
            case.n::<i64>("native_helper_return_word") != 0
        );
        let expected = |key: &str| case.n::<Vec<i16>>(key);
        assert_eq!(result.sounds, expected("sounds"), "case {index}");
        assert_eq!(result.warnings, expected("warnings"), "case {index}");
    }
}
