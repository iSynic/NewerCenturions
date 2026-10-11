use centurions_content::word;
use centurions_core::story_schedule::edit;

fn bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&hex[index..index + 2], 16).unwrap())
        .collect()
}

fn fixture() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "../../../evidence/fixtures/schedule-edit.json"
    ))
    .unwrap()
}

#[test]
fn scheduled_record_edits_match_original_complete_table_and_wrapping_day() {
    let fixture = fixture();
    let mut checked = 0;
    for case in fixture["cases"].as_array().unwrap() {
        if case["boundary_only"].as_bool().unwrap() {
            continue;
        }
        let mut schedule = bytes(case["before"].as_str().unwrap());
        let extra = bytes(case["extra"].as_str().unwrap());
        edit(&mut schedule, &extra, case["day"].as_i64().unwrap() as i16).unwrap();
        assert_eq!(schedule, bytes(case["after"].as_str().unwrap()), "{case}");
        let offset = usize::from(word(&extra, 0).unwrap() as u16) * 40;
        assert_eq!(
            schedule[offset..offset + 40],
            bytes(case["scratch_after"].as_str().unwrap()),
            "{case}"
        );
        assert_eq!(case["rng_draws"], 0);
        assert_eq!(case["unchanged_world_and_party"], true);
        checked += 1;
    }
    assert_eq!(checked, 135);
}

#[test]
fn unsigned_native_seek_boundaries_reject_without_changing_the_table() {
    let fixture = fixture();
    let mut checked = 0;
    for case in fixture["cases"].as_array().unwrap() {
        if !case["boundary_only"].as_bool().unwrap() {
            continue;
        }
        let original = bytes(case["before"].as_str().unwrap());
        let mut schedule = original.clone();
        let extra = bytes(case["extra"].as_str().unwrap());
        let unsigned_offset = u32::from(word(&extra, 0).unwrap() as u16) * 40;
        assert_eq!(case["file_cursor_after"], unsigned_offset);
        assert!(edit(&mut schedule, &extra, case["day"].as_i64().unwrap() as i16,).is_err());
        assert_eq!(schedule, original);
        assert_eq!(schedule, bytes(case["after"].as_str().unwrap()));
        checked += 1;
    }
    assert_eq!(checked, 4);
}
