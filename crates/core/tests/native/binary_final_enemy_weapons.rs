use crate::prelude::*;

use crate::support::items;

fn fixture() -> serde_json::Value {
    crate::support::fixture("final-enemy-weapons.json")
}
#[test]
fn final_enemy_preparation_matches_native_item_context_and_packet() {
    let fixture = fixture();

    crate::binary_endgame_weapons::compare_required_endgame_enemy_preparation_matches_native_item_context_and_packet(fixture);
}

#[test]
fn final_enemy_firearms_match_whole_party_and_native_randomness() {
    let fixture = fixture();

    crate::binary_endgame_weapons::compare_required_endgame_enemy_firearms_match_whole_party_and_native_randomness(fixture);
}

#[test]
fn final_default_melee_selection_matches_native_records_saves_and_both_attack_indices() {
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
        scalar random.seed => &case["seed_after"], "binary_final_enemy_weapons";
        scalar random.draws => &case["rng_draws"], "binary_final_enemy_weapons";
        }
        assert!(result.sounds.is_empty());
    }
}
#[test]
fn final_adjacent_melee_caller_matches_native_action_count_and_cost() {
    let fixture = fixture();

    crate::binary_final_enemy_weapons::compare_final_adjacent_melee_caller_matches_native_action_count_and_cost(fixture);
}

pub(super) fn compare_final_adjacent_melee_caller_matches_native_action_count_and_cost(
    fixture: Value,
) {
    let items = items();
    let cases = fixture.array("caller_cases");

    for (index, case) in cases.iter().enumerate() {
        let raw = case.bytes("warriors_before");
        let mut warriors: Vec<_> = crate::support::warriors(&raw);
        let mut combat = Combat {
            battle: 117,
            map: TacticalMap {
                tiles: case.words("map_before"),
                shift: (0, 0),
                edge_word: 0,
            },
            squad: SquadPlacement {
                positions: vec![(21, 20), (41, 40), (42, 40), (43, 40), (44, 40), (45, 40)],
                under_tiles: vec![1155; 6],
            },
            enemies: EnemySetup {
                records: vec![MonsterRecord {
                    bytes: case.bytes("enemy_before"),
                }],
                positions: vec![(20, 20)],
            },
            order: case.words("order_before"),
            ..crate::support::battle()
        };
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let events = runtime::check_neighbors_with_definitions(
            &mut combat,
            &mut warriors,
            &mut random,
            10,
            &items,
        )
        .unwrap();
        let actual: Vec<_> = crate::support::record_bytes(&warriors);
        crate::support::native_expect! {
        bytes actual => &case["warriors_after"], &format!("case{index} party");
        bytes combat.enemies.records[0].bytes => &case["enemy_after"], &format!("case{index} attacker");
        }
        assert_eq!(combat.map.tiles, case.words("map_after"), "case{index} map");
        assert_eq!(combat.order, case.words("order_after"), "case{index} order");
        crate::support::native_expect! {
        scalar random.seed => &case["seed_after"], "binary_final_enemy_weapons";
        scalar random.draws => &case["rng_draws"], "binary_final_enemy_weapons";
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
            case.array("calls")
                .iter()
                .map(|call| (
                    call[0].as_i64().unwrap() as i16,
                    call[1].as_i64().unwrap() as i16
                ))
                .collect::<Vec<_>>(),
            "case{index} ordered attack calls"
        );
    }
}
#[test]
fn final_enemy_firearms_match_native_friendly_body_targets() {
    let fixture = fixture();

    crate::binary_endgame_weapons::compare_required_endgame_enemy_firearms_match_native_friendly_body_targets(fixture);
}
