use centurions_core::story::{
    MENU_RECORD_BYTES, MenuSelection, disable_menu_entry, dispatch_menu_branch, menu_selector,
    replace_menu_branch,
};

fn bytes(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

#[test]
fn disabled_encounter_entry_matches_original_record_mutation() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/encounter-choice.json"
    ))
    .unwrap();
    for case in fixture["disable_cases"].as_array().unwrap() {
        let mut menu = bytes(case["before"].as_str().unwrap());
        let original_length = menu.len();
        menu.resize(MENU_RECORD_BYTES, 0x7f);
        let native_item = case["native_item"].as_i64().unwrap() as i16;
        let mutated = disable_menu_entry(&menu, native_item).unwrap();
        assert_eq!(
            &mutated[..original_length],
            bytes(case["after"].as_str().unwrap())
        );
        assert_eq!(&mutated[original_length..], &menu[original_length..]);
        assert_eq!(case["rng_draws"], 0);
    }
}

#[test]
fn encounter_branch_copy_matches_original_complete_script_record() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/encounter-choice.json"
    ))
    .unwrap();
    for case in fixture["dispatch_cases"].as_array().unwrap() {
        let menu = bytes(case["record"].as_str().unwrap());
        let selector = case["selected"].as_i64().unwrap() as i8;
        let selection = MenuSelection {
            native_item: selector as u8,
            selector,
            script_body: dispatch_menu_branch(&menu, selector).unwrap(),
        };
        let script = bytes(case["script_before"].as_str().unwrap());
        assert_eq!(
            replace_menu_branch(&script, &selection).unwrap(),
            bytes(case["script_after"].as_str().unwrap()),
            "{}",
            case["name"]
        );
        assert_eq!(case["rng_draws"], 0);
    }
}

#[test]
fn enabled_encounter_choices_preserve_original_signed_selector() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/encounter-choice.json"
    ))
    .unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let menu = bytes(case["record_before"].as_str().unwrap());
        let native_item = case["inputs"]
            .as_array()
            .unwrap()
            .last()
            .unwrap()
            .as_u64()
            .unwrap();
        if native_item == 9 {
            assert_eq!(case["return_code"], 0);
            continue;
        }
        let index = (0..native_item as usize - 1)
            .filter(|slot| menu[96 + *slot] != 0)
            .count();
        let (actual_item, selector) = menu_selector(&menu, index).unwrap();
        assert_eq!(u64::from(actual_item), native_item, "{}", case["name"]);
        assert_eq!(i64::from(selector), case["return_code"].as_i64().unwrap());
        assert_eq!(case["seed_before"], case["seed_after"]);
        assert_eq!(case["rng_draws"], 0);
        assert!(menu_selector(&menu, 4).is_err());
    }
}
