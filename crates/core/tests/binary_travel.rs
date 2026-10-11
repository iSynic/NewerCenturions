use centurions_core::{RandomState, travel::*};

fn bytes(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

fn words(s: &str) -> Vec<i16> {
    bytes(s)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|v| i16::from_be_bytes(*v))
        .collect()
}

fn fixture() -> serde_json::Value {
    serde_json::from_str(include_str!("../../../evidence/fixtures/map-actions.json")).unwrap()
}

#[test]
fn scripted_map_mutations_match_original_complete_map_layers() {
    for case in fixture()["cases"].as_array().unwrap() {
        let extra = bytes(case["extra"].as_str().unwrap());
        let target = i16::from_be_bytes(extra[..2].try_into().unwrap()) as usize;
        for map in 0..2 {
            let before = &case[format!("map{map}_before")];
            let after = &case[format!("map{map}_after")];
            let mut floor = words(before["floor"].as_str().unwrap());
            let mut objects = words(before["objects"].as_str().unwrap());
            let mut headers = bytes(before["headers"].as_str().unwrap());
            if map == target {
                match case["opcode"].as_u64().unwrap() {
                    12 => {
                        let mutation = tile_mutation(&extra).unwrap();
                        match mutation.layer {
                            TileLayer::Floor => floor = apply_tile(&floor, &mutation).unwrap(),
                            TileLayer::Object => objects = apply_tile(&objects, &mutation).unwrap(),
                        }
                    }
                    13 => headers = set_event_chances(&headers, &extra).unwrap(),
                    7 => {
                        let replacement = event_replacement(&extra).unwrap();
                        headers = replace_event_script(
                            &headers,
                            replacement.slot,
                            &bytes(case["replacement"].as_str().unwrap()),
                        )
                        .unwrap();
                    }
                    opcode => panic!("unexpected fixture opcode {opcode}"),
                }
            }
            assert_eq!(floor, words(after["floor"].as_str().unwrap()));
            assert_eq!(objects, words(after["objects"].as_str().unwrap()));
            assert_eq!(headers, bytes(after["headers"].as_str().unwrap()));
        }
        assert_eq!(case["script_before"], case["script_after"]);
        assert_eq!(case["rng_draws"], 0);
    }
}

#[test]
fn normal_secret_search_matches_original_full_map_and_rng() {
    for case in fixture()["secret_cases"].as_array().unwrap() {
        let floor = words(case["before"].as_str().unwrap());
        let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        let result = discover_secrets(
            &floor,
            (
                case["x"].as_i64().unwrap() as i16,
                case["y"].as_i64().unwrap() as i16,
            ),
            case["chance"].as_i64().unwrap() as i16,
            &mut random,
        )
        .unwrap();
        assert_eq!(result.floor, words(case["after"].as_str().unwrap()));
        assert_eq!(random.seed, case["seed_after"].as_u64().unwrap() as u32);
        assert_eq!(random.draws, case["rng_draws"].as_u64().unwrap());
        assert_eq!(
            result.center_revealed,
            case["center_revealed"].as_bool().unwrap()
        );
        assert_eq!(
            result.discoveries.len(),
            case["flashes"].as_u64().unwrap() as usize
        );
    }
}

#[test]
fn secret_search_skill_matches_original_signed_word_average() {
    let mut compared = 0;
    for case in fixture()["secret_cases"].as_array().unwrap() {
        if case["forced_skill"].as_bool() != Some(false) {
            continue;
        }
        let records = bytes(case["warriors"].as_str().unwrap());
        let warriors: Vec<centurions_content::Warrior> = records
            .as_chunks::<3138>()
            .0
            .iter()
            .map(|record| centurions_content::Warrior {
                name: String::new(),
                bytes: record.to_vec(),
                original_sha256: String::new(),
            })
            .collect();
        assert_eq!(
            secret_search_skill(&warriors).unwrap(),
            case["chance"].as_i64().unwrap() as i16
        );
        compared += 1;
    }
    assert!(compared > 0, "missing original skill fixtures");
}
