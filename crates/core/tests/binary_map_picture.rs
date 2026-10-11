use centurions_core::story_map_picture;
use serde_json::Value;

#[test]
fn original_map_picture_word_assignment_precedes_the_redraw_boundary() {
    let fixture: Value =
        serde_json::from_str(include_str!("../../../evidence/fixtures/map-picture.json")).unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 26);
    for case in cases {
        let mut word = case["before"].as_i64().unwrap() as i16;
        story_map_picture::assign(&mut word, case["parameter"].as_i64().unwrap() as i16);
        assert_eq!(word, case["after"].as_i64().unwrap() as i16);
        assert_eq!(case["redraws"], 1);
        assert_eq!(case["ordered_boundaries"][0]["stored_word"], word);
        assert_eq!(case["rng_draws"], 0);
        assert_eq!(case["unchanged"], true);
        assert_eq!(case["continuation"], "14673c");
    }
}
