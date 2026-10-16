use super::*;
#[test]
fn regional_dispatch_matches_instruction_execution() {
    let fixture = crate::test_support::fixture("regional.json");
    let regions = crate::test_support::bytes(&fixture["regions"]);
    for case in fixture["cases"].as_array().unwrap() {
        let mut random = RandomState::new(case["seed"].as_u64().unwrap() as u32);
        let actual = regional_encounter(&regions, (5, 3), &mut random).unwrap();
        let battle = case["battle"].as_i64().unwrap() as i16;
        let expected = if battle == 0 {
            None
        } else {
            Some((battle, case["advantage"].as_i64().unwrap() as i16))
        };
        assert_eq!(actual, expected, "seed {}", case["seed"]);
        assert_eq!(random.seed, case["after_seed"].as_u64().unwrap() as u32);
        assert_eq!(random.draws, case["draws"].as_u64().unwrap());
    }
}
