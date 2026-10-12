use crate::prelude::*;

use crate::support::items;

fn fixture() -> serde_json::Value {
    crate::support::fixture("map-seven-weapons.json")
}
#[test]
fn map_seven_enemy_preparation_matches_native_item_context_and_packet() {
    let fixture = fixture();

    crate::binary_endgame_weapons::compare_required_endgame_enemy_preparation_matches_native_item_context_and_packet(fixture);
}

#[test]
fn map_seven_enemy_firearms_match_whole_party_and_native_randomness() {
    let fixture = fixture();

    crate::binary_endgame_weapons::compare_required_endgame_enemy_firearms_match_whole_party_and_native_randomness(fixture);
}

#[test]
fn map_seven_default_melee_selection_matches_native_records_saves_and_both_attack_indices() {
    let fixture = fixture();
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
        crate::support::native_expect! {
        bytes party => &case["warriors_after"], &format!("case{index} party");
        bytes enemy.bytes => &case["enemy_after"], &format!("case{index} attacker");
        scalar random.seed => &case["seed_after"], "binary_map_seven_weapons";
        scalar random.draws => &case["rng_draws"], "binary_map_seven_weapons";
        }
        assert_eq!(
            case.n::<Vec<i16>>("sounds"),
            vec![if result.damage != 0 {
                i16::from_be_bytes(items[weapon][14..16].try_into().unwrap())
            } else {
                650
            }],
            "case{index} native melee hit/miss audio"
        );
        assert!(result.sounds.is_empty());
    }
}
#[test]
fn map_seven_adjacent_melee_caller_matches_native_action_count_and_cost() {
    let fixture = fixture();

    crate::binary_endgame_weapons::compare_required_endgame_adjacent_melee_caller_matches_native_action_count_and_cost(fixture);
}

#[test]
fn map_seven_enemy_firearms_match_native_friendly_body_targets() {
    let fixture = fixture();

    crate::binary_endgame_weapons::compare_required_endgame_enemy_firearms_match_native_friendly_body_targets(fixture);
}
