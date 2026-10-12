use crate::prelude::*;

use crate::support::items;

fn fixture() -> serde_json::Value {
    crate::support::fixture("required-melee.json")
}

#[test]
fn default_melee181_selection_matches_native_records_saves_and_both_attack_indices() {
    let fixture = fixture();
    let items = items();
    let cases = fixture.array("attack_cases");

    for (index, case) in cases.iter().enumerate() {
        let mut party = case.bytes("warriors_before");
        let member = case.n::<usize>("member");
        let mut warrior = Warrior::read(&party[member * 3138..(member + 1) * 3138]).unwrap();
        let mut enemy = MonsterRecord {
            bytes: case.bytes("enemy_before"),
        };
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let result = enemy_weapons::melee(
            &mut warrior,
            &mut enemy,
            &items[181],
            &mut random,
            0,
            0,
            40 + member as i16,
        )
        .unwrap();
        party[member * 3138..(member + 1) * 3138].copy_from_slice(&warrior.bytes);
        crate::support::native_expect! {
        bytes party => &case["warriors_after"], &format!("case{index} party");
        bytes enemy.bytes => &case["enemy_after"], &format!("case{index} attacker");
        scalar random.seed => &case["seed_after"], "binary_required_melee";
        scalar random.draws => &case["rng_draws"], "binary_required_melee";
        }
        assert!(result.sounds.is_empty());
    }
}
#[test]
fn adjacent_melee181_caller_matches_native_action_count_and_cost() {
    let fixture = fixture();

    crate::binary_final_enemy_weapons::compare_final_adjacent_melee_caller_matches_native_action_count_and_cost(fixture);
}
