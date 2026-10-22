use crate::{prelude::*, support};

use support::items;

#[test]
fn genetic_loaded_melee178_matches_native_records_saves_and_both_attack_indices() {
    let fixture = support::fixture("genetic-melee.json");
    let items = items();
    let cases = fixture.array("melee_cases");

    for (index, case) in cases.iter().enumerate() {
        let mut party = case.bytes("warriors_before");
        let member = case.n::<usize>("member");
        let mut warrior = crate::support::warrior_at(&party, member);
        let mut enemy = MonsterRecord {
            bytes: case.bytes("enemy_before"),
        };
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let weapon = i16::from_be_bytes(enemy.bytes[0x28..0x2a].try_into().unwrap()) as usize;
        let result = enemy_weapons::melee(
            &mut warrior,
            &mut enemy,
            &items[weapon],
            &mut random,
            0,
            0,
            40 + member as i16,
        )
        .unwrap();
        crate::support::put_warrior(&mut party, member, &warrior);
        support::native_expect! {
            bytes party => &case["warriors_after"], &format!("case{index} party");
            bytes enemy.bytes => &case["enemy_after"], &format!("case{index} attacker");
            rng random => (&case["seed_after"], &case["rng_draws"]), "binary_genetic_melee";
        }
        assert_eq!(
            case.n::<Vec<i16>>("sounds"),
            vec![if result.damage != 0 {
                i16::from_be_bytes(items[weapon][14..16].try_into().unwrap())
            } else {
                650
            }],
            "case{index} hit/miss native audio"
        );
        assert!(result.sounds.is_empty());
    }
}

#[test]
fn genetic_adjacent_melee178_caller_matches_native_action_count_and_cost() {
    crate::binary_endgame_weapons::neighbors(&support::fixture("genetic-melee.json"), true);
}
