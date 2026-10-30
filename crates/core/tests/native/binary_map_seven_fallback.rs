use crate::{prelude::*, support};

#[test]
fn map_seven_ranged_to_natural_fallback_matches_original_full_records_and_randomness() {
    let fixture: serde_json::Value = support::fixture("map-seven-fallback.json");
    let cases = fixture.array("cases");

    for case in cases.iter() {
        let mut party = case.bytes("warriors_before");
        let member = case.n::<usize>("member");
        let mut warrior = crate::support::warrior_at(&party, member);
        let mut monster = MonsterRecord {
            bytes: case.bytes("enemy_before"),
        };
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let result = natural_attack(
            &mut warrior,
            &mut monster,
            case.n::<usize>("attack"),
            &mut random,
            0,
        )
        .unwrap();
        crate::support::put_warrior(&mut party, member, &warrior);
        support::native_expect! {
            bytes party => &case["warriors_after"], "case{index} party";
            bytes monster.bytes => &case["enemy_after"], "case{index} enemy";
            rng random => (&case["seed_after"], &case["rng_draws"]), "binary_map_seven_fallback";
            scalar (result.sounds) => &case["sounds"], "binary_map_seven_fallback";
        }
    }
}
