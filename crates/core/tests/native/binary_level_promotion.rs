use crate::prelude::*;

use crate::support::field_bytes as bytes;
use centurions_core::story_inventory;
use centurions_core::story_promotion;
use centurions_core::story_promotion::LibraryRandom;
use centurions_core::story_treasure;

fn patched(mut input: Vec<u8>, patches: &Value) -> Vec<u8> {
    for p in patches.as_array().unwrap() {
        input[p[0].as_u64().unwrap() as usize] = p[1].as_u64().unwrap() as u8;
    }
    input
}
fn party(input: &[u8]) -> Vec<Warrior> {
    crate::support::warriors(input)
}
use crate::support::record_bytes as flatten;

#[test]
fn native_close_promotes_each_living_member_once_and_preserves_valid_spells() {
    let fixture: Value = crate::support::fixture("level-promotion.json");

    let content = crate::support::content();
    let map = content.map(0).unwrap();
    let baseline = bytes(&fixture, "warriors");
    let cases = fixture.array("cases");

    for (index, case) in cases
        .iter()
        .enumerate()
        .filter(|(_, c)| c["kind"] == "close")
    {
        let before = patched(baseline.clone(), &case["before_changes"]);
        let warriors = party(&before);
        let initial = warriors.clone();
        let mut library = LibraryRandom {
            seed: case.n::<u32>("library_seed_before"),
            draws: 0,
        };
        let result = story_promotion::close(
            &warriors,
            &map.races,
            &map.profiles,
            &map.combat.as_ref().unwrap().attacks,
            case.n::<bool>("registered"),
            &mut library,
        )
        .unwrap_or_else(|e| panic!("case{index}: {e:?}"));
        let expected = patched(before, &case["after_changes"]);
        assert_eq!(flatten(&result.warriors), expected, "case{index}");
        assert_eq!(digest(&expected), case["after_sha256"], "case{index}");
        assert_eq!(warriors, initial, "input party mutated");
        crate::support::native_expect! {
        scalar library.seed => &case["library_seed_after"], "binary_level_promotion";
        scalar library.draws => &case["library_draws"], "binary_level_promotion";
        }
    }
}

#[test]
fn original_reward_chain_crosses_level_thresholds_at_close() {
    let fixture: Value = crate::support::fixture("level-promotion.json");

    let content = crate::support::content();
    let map = content.map(0).unwrap();
    let baseline = bytes(&fixture, "warriors");
    for case in fixture
        .array("cases")
        .iter()
        .filter(|c| c["kind"] == "chain")
    {
        let mut warriors = party(&patched(baseline.clone(), &case["before_changes"]));
        let mut library = LibraryRandom { seed: 1, draws: 0 };
        let mut random = RandomState {
            seed: case.n::<u32>("seed_before"),
            draws: 0,
        };
        for stage in case.array("stages") {
            let n = stage.n::<u64>("stage");
            let table = bytes(stage, "table");
            let loot = if n == 1 {
                let prepared =
                    story_treasure::prepare(&warriors, &table, &content, &mut random, 0).unwrap();
                warriors = prepared.warriors;
                prepared.items
            } else {
                let amount = i32::from_be_bytes(table[80..84].try_into().unwrap());
                warriors = story_inventory::award_story_experience(&warriors, amount)
                    .unwrap()
                    .warriors;
                vec![]
            };
            warriors = story_promotion::close(
                &warriors,
                &map.races,
                &map.profiles,
                &map.combat.as_ref().unwrap().attacks,
                true,
                &mut library,
            )
            .unwrap()
            .warriors;
            assert_eq!(
                flatten(&warriors),
                bytes(stage, "warriors"),
                "chain stage{n}"
            );
            let mut crate_bytes = vec![0; 22000];
            for (i, item) in loot.iter().enumerate() {
                crate_bytes[i * 22..i * 22 + 22].copy_from_slice(&item.0);
            }
            assert_eq!(crate_bytes, bytes(stage, "crate"), "chain stage{n}");
            crate::support::native_expect! {
            scalar library.seed => &stage["library_seed"], "binary_level_promotion";
            scalar library.draws => &stage["library_draws"], "binary_level_promotion";
            scalar random.seed => &stage["seed"], "binary_level_promotion";
            scalar random.draws => &stage["draws"], "binary_level_promotion";
            }
            assert_eq!(stage["currency"], 937);
        }
    }
}
