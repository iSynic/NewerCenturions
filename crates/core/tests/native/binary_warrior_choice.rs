use crate::{prelude::*, support};

use centurions_core::story_skill::single_warrior;

#[test]
fn single_living_warrior_selection_matches_original_gate_and_mask() {
    let fixture: serde_json::Value = support::fixture("warrior-choice.json");
    for case in fixture.array("selection_cases") {
        let selected = case.n::<usize>("selected");
        let stamina = case.n::<i16>("stamina");
        let mut warriors = vec![
            Warrior {
                name: "Coriand".into(),
                original_sha256: String::new(),
                bytes: vec![0; WARRIOR_BYTES],
            };
            6
        ];
        warriors[selected].bytes[0x50..0x52].copy_from_slice(&stamina.to_be_bytes());
        let before = warriors.clone();
        let result = single_warrior(&warriors, selected);
        if case.n::<bool>("accepted") {
            let mask = case.bytes("mask");
            let expected = std::array::from_fn(|index| word(&mask, index * 2).unwrap());
            assert_eq!(result.unwrap(), expected);
            assert_eq!(&mask[12..], [0; 4]);
        } else {
            assert!(result.is_err());
        }
        assert_eq!(warriors, before);
    }
    let prefix = &fixture["prefix"];
    assert!(prefix.bytes("after").iter().all(|value| *value == 0));
    assert_eq!(prefix["remaining"], 1);
}
