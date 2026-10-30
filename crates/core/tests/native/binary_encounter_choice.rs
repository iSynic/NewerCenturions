use crate::{prelude::*, support};

use centurions_core::story::{
    MENU_RECORD_BYTES, MenuSelection, disable_menu_entry, dispatch_menu_branch, menu_selector,
    replace_menu_branch,
};

#[test]
fn disabled_encounter_entry_matches_original_record_mutation() {
    let fixture: serde_json::Value = support::fixture("encounter-choice.json");
    for case in fixture.array("disable_cases") {
        let mut menu = case.bytes("before");
        let original_length = menu.len();
        menu.resize(MENU_RECORD_BYTES, 0x7f);
        let menu_item = case.n::<i16>("native_item");
        let mutated = disable_menu_entry(&menu, menu_item).unwrap();
        assert_eq!(&mutated[..original_length], case.bytes("after"));
        assert_eq!(&mutated[original_length..], &menu[original_length..]);
    }
}

#[test]
fn encounter_branch_copy_matches_original_complete_script_record() {
    let fixture: serde_json::Value = support::fixture("encounter-choice.json");
    for case in fixture.array("dispatch_cases") {
        let menu = case.bytes("record");
        let selector = case.n::<i8>("selected");
        let selection = MenuSelection {
            menu_item: selector as u8,
            selector,
            script_body: dispatch_menu_branch(&menu, selector).unwrap(),
        };
        let script = case.bytes("script_before");
        assert_eq!(
            replace_menu_branch(&script, &selection).unwrap(),
            case.bytes("script_after"),
            "{}",
            case["name"]
        );
    }
}

#[test]
fn enabled_encounter_choices_preserve_original_signed_selector() {
    let fixture: serde_json::Value = support::fixture("encounter-choice.json");
    for case in fixture.array("cases") {
        let menu = case.bytes("record_before");
        let menu_item = case.array("inputs").last().unwrap().as_u64().unwrap();
        if menu_item == 9 {
            continue;
        }
        let index = (0..menu_item as usize - 1)
            .filter(|slot| menu[96 + *slot] != 0)
            .count();
        let (actual_item, selector) = menu_selector(&menu, index).unwrap();
        assert_eq!(u64::from(actual_item), menu_item, "{}", case["name"]);
        support::expect(i64::from(selector), &case["return_code"]);

        assert!(menu_selector(&menu, 4).is_err());
    }
}
