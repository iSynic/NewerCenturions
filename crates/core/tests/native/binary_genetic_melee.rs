use crate::{native_common, prelude::*, support};

use support::items;

#[test]
fn genetic_loaded_melee178_matches_native_records_saves_and_both_attack_indices() {
    let fixture = support::fixture("genetic-melee.json");
    let items = items();
    let cases = fixture.array("melee_cases");

    for (index, case) in cases.iter().enumerate() {
        let mut party = case.bytes("warriors_before");
        let member = case.n::<usize>("member");
        let mut warrior = Warrior::read(&party[member * 3138..(member + 1) * 3138]).unwrap();
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
        party[member * 3138..(member + 1) * 3138].copy_from_slice(&warrior.bytes);
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
    let fixture = support::fixture("genetic-melee.json");
    let items = items();
    let cases = fixture.array("caller_cases");

    for (index, case) in cases.iter().enumerate() {
        let raw = case.bytes("warriors_before");
        let mut warriors: Vec<_> = support::warriors(&raw);
        let mut combat = native_common::neighbor_combat(case, case.n("member"));
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let events = runtime::check_neighbors_with_definitions(
            &mut combat,
            &mut warriors,
            &mut random,
            10,
            &items,
        )
        .unwrap();
        let actual: Vec<_> = support::record_bytes(&warriors);
        support::native_expect! {
            bytes actual => &case["warriors_after"], &format!("case{index} party");
            bytes combat.enemies.records[0].bytes => &case["enemy_after"], &format!("case{index} attacker");
        }
        assert_eq!(combat.map.tiles, case.words("map_after"), "case{index} map");
        assert_eq!(combat.order, case.words("order_after"), "case{index} order");
        support::native_expect! {
            rng random => (&case["seed_after"], &case["rng_draws"]), "binary_genetic_melee";
            scalar (support::event_sounds(&events)) => &case["sounds"], "binary_genetic_melee";
        }
        let resolved: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                Event::AttackResolved {
                    attacker, target, ..
                } => Some((*attacker, *target)),
                _ => None,
            })
            .collect();
        assert_eq!(
            resolved,
            support::attack_calls(&case["calls"]),
            "case{index} ordered attack calls"
        );
    }
}
