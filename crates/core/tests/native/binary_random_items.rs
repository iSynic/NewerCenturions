use crate::prelude::*;

use centurions_core::items;

#[test]
fn item_initialization_and_enhancement_match_original_records_and_randomness() {
    for name in ["random-items.json", "difficulty-items.json"] {
        let fixture: serde_json::Value = crate::support::fixture(name);

        let content = crate::support::content();
        let cases = fixture.array("cases");

        for (index, case) in cases
            .iter()
            .enumerate()
            .filter(|(_, case)| name == "random-items.json" || case["difficulty"] != 0)
        {
            let id = case.n::<i16>("id");
            assert_eq!(
                content.items[id.unsigned_abs() as usize],
                case.bytes("definition")
            );
            let mut random = RandomState::new(case.n::<u32>("seed_before"));
            let record = items::initialize_at_difficulty(
                id,
                case.n::<i16>("enhancement"),
                case.n::<i16>("difficulty"),
                &content,
                &mut random,
            )
            .unwrap_or_else(|error| panic!("case{index} item{id}: {error:?}"));
            crate::support::native_expect! {
            bytes record.0.to_vec() => &case["after"], "case{index} item{id}";
            scalar random.seed => &case["seed_after"], "binary_random_items";
            scalar random.draws => &case["rng_draws"], "binary_random_items";
            }
        }
    }
}
