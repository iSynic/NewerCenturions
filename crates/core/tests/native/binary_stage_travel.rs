use crate::{prelude::*, support};

use centurions_core::{MapState, mission};

#[test]
fn accepted_stage_restoration_matches_complete_original_warriors() {
    let fixture: serde_json::Value = support::fixture("stage-travel.json");
    for case in fixture.array("stage_cases") {
        let parameter = case.n::<i16>("parameter");
        let before = case.bytes("before");
        let warriors: Vec<_> = before
            .as_chunks::<WARRIOR_BYTES>()
            .0
            .iter()
            .map(|record| Warrior {
                name: String::new(),
                bytes: record.to_vec(),
                original_sha256: String::new(),
            })
            .collect();
        let accepted = parameter < 0 || case.n::<i64>("answer") != 0;
        let actual = if accepted {
            mission::restore_stage_warriors(&warriors).unwrap()
        } else {
            warriors
        };
        assert_eq!(
            support::record_bytes(&actual),
            case.bytes("after"),
            "stage {parameter}"
        );
        if accepted {
            support::expect(
                mission::stage_target(parameter).unwrap(),
                &case["stage_after"],
            );
        }
    }
}

#[test]
fn packed_map_records_match_original_stage_and_travel_io() {
    let fixture: serde_json::Value = support::fixture("stage-travel.json");
    for family in ["stage_cases", "travel_cases"] {
        for case in fixture[family].as_array().unwrap() {
            for key in ["map_before", "map_after", "saved_origin"] {
                let expected = bytes(case[key].as_str().unwrap());
                assert_eq!(
                    MapState::from_bytes(&expected)
                        .unwrap()
                        .to_bytes()
                        .unwrap(),
                    expected,
                    "{family}/{key}"
                );
            }
        }
    }
}
