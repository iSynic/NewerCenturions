use centurions_content::{
    Content, InventoryRecord, Warrior,
    combat::{BattleRecord, MonsterRecord},
    word,
};
use centurions_core::{
    RandomState,
    combat::{TacticalMap, place_enemies},
};
use std::path::Path;

fn bytes(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

fn words(s: &str) -> Vec<i16> {
    bytes(s)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|word| i16::from_be_bytes(*word))
        .collect()
}

fn content() -> Content {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap()
}

fn assert_bytes(actual: &[u8], expected: &[u8], context: &str) {
    assert_eq!(actual.len(), expected.len(), "{context} length");
    if let Some(index) = actual.iter().zip(expected).position(|(a, b)| a != b) {
        panic!(
            "{context} differs at byte {index}: {} != {}",
            actual[index], expected[index]
        );
    }
}

#[test]
fn genetic_enemy_setup_matches_original_complete_records_placement_and_rng() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/genetic-enemies.json"
    ))
    .unwrap();
    let content = content();
    let mut mission = content.map(0).unwrap().clone();
    let properties = bytes(fixture["properties"].as_str().unwrap());
    mission.tile_properties = properties[64..12064].to_vec();
    mission.default_tile = word(&properties, 0).unwrap();
    let definitions = bytes(fixture["definitions"].as_str().unwrap());
    for case in fixture["cases"].as_array().unwrap() {
        let mut map = TacticalMap {
            tiles: words(fixture["map"].as_str().unwrap()),
            shift: (-39, -15),
            edge_word: 0,
        };
        let battle = BattleRecord::read(&bytes(case["battle"].as_str().unwrap()), 0).unwrap();
        let base = case["base"].as_i64().unwrap() as i16;
        let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
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
        assert_eq!(
            enemies.records.len(),
            case["count"].as_u64().unwrap() as usize,
            "{context}"
        );
        assert_bytes(
            &enemies
                .records
                .iter()
                .flat_map(|record| record.bytes.iter().copied())
                .collect::<Vec<_>>(),
            &bytes(case["records"].as_str().unwrap()),
            &context,
        );
        assert_eq!(
            enemies
                .positions
                .iter()
                .flat_map(|(x, y)| [*x, *y])
                .collect::<Vec<_>>(),
            words(case["positions"].as_str().unwrap()),
            "{context}"
        );
        assert_eq!(
            map.tiles,
            words(case["tiles"].as_str().unwrap()),
            "{context}"
        );
        assert_eq!(
            random.seed,
            case["seed_after"].as_u64().unwrap() as u32,
            "{context}"
        );
        assert_eq!(random.draws, case["draws"].as_u64().unwrap(), "{context}");
    }
}

#[test]
fn genetic_victory_rewards_match_original_complete_records_loot_and_rng() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/genetic-victory.json"
    ))
    .unwrap();
    let content = content();
    for case in fixture["cases"].as_array().unwrap() {
        let mut warriors = bytes(case["warriors_before"].as_str().unwrap())
            .as_chunks::<3138>()
            .0
            .iter()
            .map(|record| Warrior::read(record).unwrap())
            .collect::<Vec<_>>();
        let mut enemies = bytes(case["enemies_before"].as_str().unwrap())
            .as_chunks::<288>()
            .0
            .iter()
            .map(|record| MonsterRecord {
                bytes: record.to_vec(),
            })
            .collect::<Vec<_>>();
        let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
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
        assert_bytes(
            &warriors
                .iter()
                .flat_map(|record| record.bytes.iter().copied())
                .collect::<Vec<_>>(),
            &bytes(case["warriors_after"].as_str().unwrap()),
            &context,
        );
        assert_bytes(
            &enemies
                .iter()
                .flat_map(|record| record.bytes.iter().copied())
                .collect::<Vec<_>>(),
            &bytes(case["enemies_after"].as_str().unwrap()),
            &context,
        );
        let mut loot = vec![InventoryRecord([0; 22]); 1000];
        loot[..items.len()].clone_from_slice(&items);
        assert_bytes(
            &loot.iter().flat_map(|item| item.0).collect::<Vec<_>>(),
            &bytes(case["loot_after"].as_str().unwrap()),
            &context,
        );
        assert_eq!(share, case["share"].as_i64().unwrap() as i32, "{context}");
        assert_eq!(
            random.seed,
            case["seed_after"].as_u64().unwrap() as u32,
            "{context}"
        );
        assert_eq!(random.draws, case["draws"].as_u64().unwrap(), "{context}");
    }
}
