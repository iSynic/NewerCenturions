use crate::prelude::*;

use centurions_core::story_allies::add_ally;

#[test]
fn joined_biologists_match_complete_original_records_and_rng() {
    let fixture: serde_json::Value = crate::support::fixture("story-allies.json");
    let definition = fixture.bytes("definition");
    for case in fixture.array("cases") {
        let mut records = case.bytes("before");
        let mut count = case.n::<usize>("count_before");
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let mut warnings = Vec::new();
        for _ in 0..case.n::<u64>("calls") {
            let result = add_ally(
                &records,
                count,
                &definition,
                case.n::<i16>("difficulty"),
                &mut random,
            )
            .unwrap();
            records = result.records;
            count = result.count;
            if let Some(warning) = result.warning {
                warnings.push(warning);
            }
        }
        crate::support::native_expect! {
        bytes records => &case["after"], format!("case count {} seed {}",case["count_before"],case["seed_before"]);
        scalar count => &case["count_after"], "binary_story_allies";
        scalar random.seed => &case["seed_after"], "binary_story_allies";
        scalar random.draws => &case["rng_draws"], "binary_story_allies";
        scalar (warnings) => &case["warnings"], "binary_story_allies";
        }
    }
}
