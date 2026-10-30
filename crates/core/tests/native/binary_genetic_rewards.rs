use crate::{prelude::*, support};

use centurions_content::combat::BattleRecord;

use centurions_core::combat::place_enemies;

#[test]
fn genetic_enemy_setup_matches_original_complete_records_placement_and_rng() {
    let fixture: serde_json::Value = support::fixture("genetic-enemies.json");
    let content = content();
    let mut mission = content.map(0).unwrap().clone();
    let properties = fixture.bytes("properties");
    mission.map_stats = properties[64..12064].to_vec();
    mission.default_tile = word(&properties, 0).unwrap();
    let definitions = fixture.bytes("definitions");
    for case in fixture.array("cases") {
        let mut map = CombatField {
            tiles: words(fixture.text("map")),
            map_shift: (-39, -15),
            edge_word: 0,
        };
        let battle = BattleRecord::read(&case.bytes("battle"), 0).unwrap();
        let base = case.n::<i16>("base");
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let enemies = place_enemies(
            &mut map,
            &mission,
            &battle,
            &definitions,
            (base, base),
            &mut random,
        )
        .unwrap();
        let context = format!("battle {} seed {}", case["battle_id"], case["seed_before"]);
        support::native_expect! {
            scalar enemies.records.len() => &case["count"], "binary_genetic_rewards";
            records &enemies.records => &case["records"], &context;
            words &(enemies
                        .mon_pos
                        .iter()
                        .flat_map(|(x, y)| [*x, *y])
                        .collect::<Vec<_>>()) => &case["positions"], "binary_genetic_rewards";
            words &(map.tiles) => &case["tiles"], "binary_genetic_rewards";
            rng random => (&case["seed_after"], &case["draws"]), "binary_genetic_rewards";
        }
    }
}

#[test]
fn genetic_victory_rewards_match_original_complete_records_loot_and_rng() {
    let fixture: serde_json::Value = support::fixture("genetic-victory.json");
    let content = content();
    for case in fixture.array("cases") {
        let mut warriors = support::warriors(&case.bytes("warriors_before"));
        let mut enemies = support::monsters(&case.bytes("enemies_before"));
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let context = format!(
            "battle {} setup seed {}",
            case["battle_id"], case["setup_seed"]
        );
        let (share, items) = centurions_core::combat::runtime::victory_loot(
            &mut warriors,
            &mut enemies,
            &mut random,
            Some(&content),
        )
        .unwrap_or_else(|error| panic!("{context}: {error:?}"));
        support::native_expect! {
            records &warriors => &case["warriors_after"], &context;
            records &enemies => &case["enemies_after"], &context;
        }
        let mut loot = vec![InventoryRecord([0; 22]); 1000];
        loot[..items.len()].clone_from_slice(&items);
        support::native_expect! {
            bytes loot.iter().flat_map(|item| item.0).collect::<Vec<_>>() => &case["loot_after"], &context;
            scalar share => &case["share"], "binary_genetic_rewards";
            rng random => (&case["seed_after"], &case["draws"]), "binary_genetic_rewards";
        }
    }
}
