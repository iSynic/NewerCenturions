use centurions_content::Warrior;
use centurions_core::{MapState, mission};

fn bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}

#[test]
fn accepted_stage_restoration_matches_complete_original_warriors() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../evidence/fixtures/stage-travel.json")).unwrap();
    for case in fixture["stage_cases"].as_array().unwrap() {
        let parameter = case["parameter"].as_i64().unwrap() as i16;
        let before = bytes(case["before"].as_str().unwrap());
        let warriors: Vec<_> = before
            .as_chunks::<3138>()
            .0
            .iter()
            .map(|b| Warrior {
                name: String::new(),
                bytes: b.to_vec(),
                original_sha256: String::new(),
            })
            .collect();
        let accepted = parameter < 0 || case["answer"].as_i64().unwrap() != 0;
        let actual = if accepted {
            mission::restore_stage_warriors(&warriors).unwrap()
        } else {
            warriors
        };
        assert_eq!(
            actual
                .iter()
                .flat_map(|w| w.bytes.clone())
                .collect::<Vec<_>>(),
            bytes(case["after"].as_str().unwrap()),
            "stage {parameter}"
        );
        if accepted {
            assert_eq!(
                mission::stage_target(parameter).unwrap(),
                case["stage_after"].as_u64().unwrap() as usize
            );
        }
        assert_eq!(case["flags_before"], case["flags_after"]);
        assert_eq!(case["time_before"], case["time_after"]);
    }
}

#[test]
fn packed_map_records_match_original_stage_and_travel_io() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../evidence/fixtures/stage-travel.json")).unwrap();
    for family in ["stage_cases", "travel_cases"] {
        for case in fixture[family].as_array().unwrap() {
            for key in ["map_before", "map_after", "saved_origin"] {
                let expected = bytes(case[key].as_str().unwrap());
                assert_eq!(
                    MapState::from_native(&expected)
                        .unwrap()
                        .native_bytes()
                        .unwrap(),
                    expected,
                    "{family}/{key}"
                );
            }
        }
    }
}
