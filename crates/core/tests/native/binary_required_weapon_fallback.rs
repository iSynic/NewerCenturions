use crate::prelude::*;

#[test]
fn ranged_to_natural_fallback_matches_original_full_records_and_randomness() {
    let fixture: serde_json::Value = crate::support::fixture("required-weapon-fallback.json");
    let cases = fixture.array("cases");

    for case in cases.iter() {
        let mut party = case.bytes("warriors_before");
        let member = case.n::<usize>("member");
        let mut warrior = Warrior::read(&party[member * 3138..(member + 1) * 3138]).unwrap();
        let mut enemy = MonsterRecord {
            bytes: case.bytes("enemy_before"),
        };
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let result = natural_attack(&mut warrior, &mut enemy, 0, &mut random, 0).unwrap();
        party[member * 3138..(member + 1) * 3138].copy_from_slice(&warrior.bytes);
        crate::support::native_expect! {
        bytes party => &case["warriors_after"], "case{index} party";
        bytes enemy.bytes => &case["enemy_after"], "case{index} enemy";
        scalar random.seed => &case["seed_after"], "binary_required_weapon_fallback";
        scalar random.draws => &case["rng_draws"], "binary_required_weapon_fallback";
        scalar (result.sounds) => &case["sounds"], "binary_required_weapon_fallback";
        }
    }
}
