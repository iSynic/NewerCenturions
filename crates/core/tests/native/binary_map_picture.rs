use crate::{prelude::*, support};

use centurions_core::story_map_picture;

#[test]
fn original_map_picture_word_assignment_precedes_the_redraw_boundary() {
    let fixture: Value = support::fixture("map-picture.json");
    let cases = fixture.array("cases");

    for case in cases {
        let mut word = case.n::<i16>("before");
        story_map_picture::assign(&mut word, case.n::<i16>("parameter"));
        support::expect(word, &case["after"]);

        assert_eq!(case["ordered_boundaries"][0]["stored_word"], word);
    }
}
