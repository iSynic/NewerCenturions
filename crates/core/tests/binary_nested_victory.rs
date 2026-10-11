use centurions_content::{Warrior, combat::MonsterRecord};
use centurions_core::{RandomState, combat::runtime::victory_loot};

fn bytes(value: &serde_json::Value) -> Vec<u8> {
    let text = value.as_str().unwrap();
    (0..text.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&text[index..index + 2], 16).unwrap())
        .collect()
}

#[test]
fn nested_battle_rewards_consume_count_once_and_preserve_backing_records() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/nested-victory.json"
    ))
    .unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let mut enemies: Vec<_> = bytes(&case["records_before"])
            .as_chunks::<288>()
            .0
            .iter()
            .map(|record| MonsterRecord {
                bytes: record.to_vec(),
            })
            .collect();
        let mut warriors: Vec<_> = bytes(&case["calls"][0]["party_before"])
            .as_chunks::<3138>()
            .0
            .iter()
            .map(|record| Warrior::read(record).unwrap())
            .collect();
        let mut count = case["count"].as_u64().unwrap() as usize;
        let mut random = RandomState::new(case["setup_seed_after"].as_u64().unwrap() as u32);
        for call in case["calls"].as_array().unwrap() {
            let before_draws = random.draws;
            assert_eq!(random.seed, call["seed_before"].as_u64().unwrap() as u32);
            assert_eq!(
                warriors
                    .iter()
                    .flat_map(|warrior| &warrior.bytes)
                    .copied()
                    .collect::<Vec<_>>(),
                bytes(&call["party_before"])
            );
            let (share, loot) =
                victory_loot(&mut warriors, &mut enemies[..count], &mut random, None).unwrap();
            assert_eq!(share, call["share"].as_i64().unwrap() as i32);
            assert!(loot.is_empty());
            assert!(bytes(&call["loot_after"]).iter().all(|value| *value == 0));
            assert_eq!(
                enemies
                    .iter()
                    .flat_map(|enemy| &enemy.bytes)
                    .copied()
                    .collect::<Vec<_>>(),
                bytes(&call["records_after"])
            );
            assert_eq!(
                warriors
                    .iter()
                    .flat_map(|warrior| &warrior.bytes)
                    .copied()
                    .collect::<Vec<_>>(),
                bytes(&call["party_after"])
            );
            assert_eq!(random.seed, call["seed_after"].as_u64().unwrap() as u32);
            assert_eq!(
                random.draws - before_draws,
                call["rng_draws"].as_u64().unwrap()
            );
            count = call["enemy_count_after"].as_u64().unwrap() as usize;
        }
        assert_eq!(count, 0);
        assert_eq!(enemies[0].bytes[0..2], 7_i16.to_be_bytes());
    }
}
