use centurions_content::{Content, Warrior, combat::MonsterRecord};
use centurions_core::{
    RandomState,
    combat::{allies::synchronize, runtime::victory_loot},
};
use std::path::Path;

fn bytes(value: &serde_json::Value) -> Vec<u8> {
    let text = value.as_str().unwrap();
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect()
}

#[test]
fn survivor_cleanup_and_rewards_match_original_composite_execution() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let content = Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap();
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../evidence/fixtures/ally-victory.json")).unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let mut warriors: Vec<_> = bytes(&case["warriors_before"])
            .as_chunks::<3138>()
            .0
            .iter()
            .map(|b| Warrior::read(b).unwrap())
            .collect();
        let count = case["count"].as_u64().unwrap() as usize;
        let mut enemies: Vec<_> = bytes(&case["enemies_before"])
            .as_chunks::<288>()
            .0
            .iter()
            .take(count)
            .map(|b| MonsterRecord { bytes: b.to_vec() })
            .collect();
        let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        let result = synchronize(
            &bytes(&case["menu_before"]),
            case["ally_count"].as_u64().unwrap() as usize,
            &mut enemies,
            &mut random,
        )
        .unwrap();
        assert_eq!(result.menu, bytes(&case["menu_after"]));
        assert_eq!(
            result.count,
            case["ally_count_after"].as_u64().unwrap() as usize
        );
        let (share, loot) =
            victory_loot(&mut warriors, &mut enemies, &mut random, Some(&content)).unwrap();
        assert_eq!(share, case["share"].as_i64().unwrap() as i32);
        assert_eq!(
            warriors
                .iter()
                .flat_map(|w| w.bytes.clone())
                .collect::<Vec<_>>(),
            bytes(&case["warriors_after"])
        );
        assert_eq!(
            enemies
                .iter()
                .flat_map(|m| m.bytes.clone())
                .collect::<Vec<_>>(),
            bytes(&case["enemies_after"])[..count * 288]
        );
        assert_eq!(
            loot.iter().flat_map(|i| i.0).collect::<Vec<_>>(),
            bytes(&case["loot_after"])[..loot.len() * 22]
        );
        assert_eq!(random.seed, case["seed_after"].as_u64().unwrap() as u32);
        assert_eq!(random.draws, case["rng_draws"].as_u64().unwrap());
    }
}
