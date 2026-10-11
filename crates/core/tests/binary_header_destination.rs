use centurions_core::story_destination::{
    Location, destination, disable_on_exhaustion, persistent_record,
};

fn bytes(text: &str) -> Vec<u8> {
    (0..text.len())
        .step_by(2)
        .map(|offset| u8::from_str_radix(&text[offset..offset + 2], 16).unwrap())
        .collect()
}

#[test]
fn door_destinations_and_suppression_match_original_finish() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/header-destination.json"
    ))
    .unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let input: Vec<i32> = case["input"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_i64().unwrap() as i32)
            .collect();
        let current = Location {
            map: input[0],
            x: input[1],
            y: input[2],
            level: input[3] as u8,
        };
        let record = bytes(case["record_before"].as_str().unwrap());
        let skipped = input[8] == 0 && input[11] == -1;
        let plan = if skipped {
            None
        } else {
            destination(&record, current, input[9] != 0, input[10] != 0).unwrap()
        };
        let actual = &case["location_after"];
        let target = plan.map_or(current, |plan| plan.target);
        assert_eq!(target.map, actual[0].as_i64().unwrap() as i32, "{input:?}");
        assert_eq!(target.x, actual[1].as_i64().unwrap() as i32, "{input:?}");
        assert_eq!(target.y, actual[2].as_i64().unwrap() as i32, "{input:?}");
        assert_eq!(target.level, actual[3].as_u64().unwrap() as u8, "{input:?}");
        let reload = plan.is_some_and(|plan| plan.reload);
        assert_eq!(reload, case["calls"].as_array().unwrap().len() == 2);
        if input[8] == 0 && !skipped {
            assert_eq!(
                bytes(case["record_after"].as_str().unwrap())[7] == 255,
                disable_on_exhaustion(input[11] as i16, 0)
            );
        }
        if input[8] == 25 && plan.is_some() {
            let after = bytes(case["record_after"].as_str().unwrap());
            let mut expected = persistent_record(&record, current.x, current.y).unwrap();
            expected[7] = 255;
            assert_eq!(after, expected, "{input:?}");
            assert_eq!(bytes(case["persisted"].as_str().unwrap()), expected);
        }
        if input[8] == 24 && reload && case["actual_file_services"] == true {
            assert_eq!(case["origin_before_sha256"], case["origin_saved_sha256"]);
        }
    }
    assert_eq!(fixture["rng_draws"], 0);
}
