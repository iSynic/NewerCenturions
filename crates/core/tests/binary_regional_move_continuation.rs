use centurions_core::{RandomState, walking_header::lookup};
use serde_json::{Value, json};

fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../evidence/fixtures/regional-move-continuation.json"
    ))
    .unwrap()
}

fn bytes(value: &Value) -> Vec<u8> {
    let text = value.as_str().unwrap();
    (0..text.len())
        .step_by(2)
        .map(|offset| u8::from_str_radix(&text[offset..offset + 2], 16).unwrap())
        .collect()
}

fn long(bytes: &[u8], offset: usize) -> i32 {
    i32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn word(bytes: &[u8], offset: usize) -> i16 {
    i16::from_be_bytes(bytes[offset..offset + 2].try_into().unwrap())
}

fn current_position(snapshot: &Value) -> (i32, i32, i32) {
    let location = bytes(&snapshot["location"]);
    (
        long(&location, 16),
        long(&location, 0).wrapping_add(long(&location, 20)),
        long(&location, 4).wrapping_add(long(&location, 24)),
    )
}

fn assert_unaffected_native_buffers(case: &Value, prefix: bool) {
    for (field, before) in case["before"].as_object().unwrap() {
        if prefix
            && matches!(
                field.as_str(),
                "working_header" | "current_header" | "header_key" | "seed" | "draws"
            )
        {
            continue;
        }
        assert_eq!(before, &case["after"][field], "native {field}");
    }
}

#[test]
fn original_caller_captures_current_coordinates_and_skips_header_on_exit() {
    let fixture = fixture();
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 4);
    let before_move = &fixture["foundations"][0]["session"];
    let after_victory = &fixture["foundations"][1]["session"];
    assert_eq!(before_move["exploration"]["x"], 6);
    assert_eq!(before_move["exploration"]["y"], 4);
    assert_eq!(
        before_move["random"],
        json!({"seed": 1906898442_u32, "draws": 17571})
    );
    assert_eq!(after_victory["exploration"]["x"], 7);
    assert_eq!(after_victory["exploration"]["y"], 3);
    assert_eq!(
        after_victory["random"],
        json!({"seed": 380869113_u32, "draws": 24319})
    );
    assert!(after_victory["combat"].is_null());
    assert!(after_victory["exploration"]["script"].is_null());
    assert!(after_victory["exploration"]["presentation"].is_null());

    for case in fixture["cases"].as_array().unwrap() {
        let (_, x, y) = current_position(&case["before"]);
        if case["game_exit_control"].as_bool().unwrap() {
            assert_eq!(case["before"]["game_exit"], 1);
            assert_eq!(case["arguments"], json!([]));
            assert_eq!(case["stop"], "10ae78");
            assert_eq!(case["return"], 0);
            assert_unaffected_native_buffers(case, false);
        } else {
            assert_eq!(case["before"]["game_exit"], 0);
            assert_eq!(case["arguments"], json!([x, y, 0, 0, 0]));
            assert_eq!(case["newland_return"], "10988a");
            if !case["complete_newland_prefix"].as_bool().unwrap() {
                assert_eq!(case["stop"], "1400e4");
                assert_eq!(case["rng"], json!([]));
                assert_unaffected_native_buffers(case, false);
            }
        }
    }
}

#[test]
fn walking_header_lookup_matches_continuous_original_prefix_and_full_buffers() {
    for case in fixture()["cases"].as_array().unwrap() {
        if !case["complete_newland_prefix"].as_bool().unwrap() {
            continue;
        }
        let before = &case["before"];
        let after = &case["after"];
        let headers = bytes(&before["headers"]);
        let original_headers = headers.clone();
        let (map, x, y) = current_position(before);
        let mut random = RandomState {
            seed: before["seed"].as_u64().unwrap() as u32,
            draws: before["draws"].as_u64().unwrap(),
        };
        let mut receipt = random.clone();
        for draw in case["rng"].as_array().unwrap() {
            assert_eq!(receipt.seed, draw["seed_before"].as_u64().unwrap() as u32);
            assert_eq!(
                receipt.bound(draw["bound"].as_i64().unwrap() as i16),
                draw["value"].as_i64().unwrap() as i16
            );
            assert_eq!(receipt.seed, draw["seed_after"].as_u64().unwrap() as u32);
        }
        let selected = lookup(&headers, map, x, y, &mut random).unwrap();
        assert_eq!(headers, original_headers);
        assert_eq!(random, receipt);
        assert_eq!(random.seed, after["seed"].as_u64().unwrap() as u32);
        assert_eq!(random.draws, after["draws"].as_u64().unwrap());
        assert_eq!(after["header_key"], 307);
        assert_eq!(after["current_header"], 7);
        let native_record = bytes(&after["working_header"]);
        assert_eq!(&headers[7 * 42..8 * 42], native_record);
        if case["inactive_header_control"].as_bool().unwrap() {
            assert!(selected.is_none());
            assert_eq!(native_record[7], 0);
            assert_eq!(case["rng"], json!([]));
            assert_eq!(case["effects"], json!(["newland", "reset_text"]));
            assert_eq!(case["stop"], "10988a");
        } else {
            let selected = selected.unwrap();
            assert_eq!(selected.slot, 7);
            assert_eq!(selected.record, native_record);
            assert_eq!(selected.record[7], 100);
            for index in 0..6 {
                assert_eq!(word(&selected.record, 8 + index * 2), 1);
                assert_eq!(word(&selected.record, 24 + index * 2), 175 + index as i16);
            }
            assert_eq!(word(&selected.record, 20), 0);
            assert_eq!(word(&selected.record, 22), 39);
            assert_eq!(word(&selected.record, 38), 45);
            assert_eq!(case["stop"], "123c60");
            assert_eq!(
                case["effects"],
                json!([
                    "newland",
                    "reset_text",
                    ["text", 175],
                    ["text", 176],
                    ["text", 177],
                    ["text", 178],
                    ["text", 179],
                    ["text", 180]
                ])
            );
        }
        assert_unaffected_native_buffers(case, true);
    }
}

#[test]
fn source_boundaries_preserve_first_match_signed_chance_and_atomic_validation() {
    let mut headers = vec![0; 4200];
    headers[..4].copy_from_slice(&10307_i32.to_be_bytes());
    headers[42..46].copy_from_slice(&10307_i32.to_be_bytes());
    headers[42 + 7] = 100;
    let mut random = RandomState::new(12);
    let original_random = random.clone();
    assert!(lookup(&headers, 1, 7, 3, &mut random).unwrap().is_none());
    assert_eq!(random, original_random);
    headers[7] = 255;
    assert!(lookup(&headers, 1, 7, 3, &mut random).unwrap().is_none());
    assert_eq!(random, original_random);
    assert!(lookup(&headers, 2, 7, 3, &mut random).unwrap().is_none());
    assert_eq!(random, original_random);
    assert!(lookup(&headers[..4199], 1, 7, 3, &mut random).is_err());
    assert_eq!(random, original_random);

    let roll = random.clone().bound(100);
    headers[7] = roll as u8;
    assert_eq!(
        lookup(&headers, 1, 7, 3, &mut random)
            .unwrap()
            .unwrap()
            .slot,
        0
    );
    random = original_random;
    headers[7] = (roll - 1) as u8;
    assert!(lookup(&headers, 1, 7, 3, &mut random).unwrap().is_none());
    assert_eq!(random.draws, 1);
}
