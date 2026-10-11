use centurions_content::{Warrior, combat::MonsterRecord};
use centurions_core::{RandomState, combat::runtime::victory_records};
fn decode(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}
#[test]
fn victory_rewards_match_original_branch() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../evidence/fixtures/victory.json")).unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let mut warriors: Vec<Warrior> = decode(case["warriors"].as_str().unwrap())
            .as_chunks::<3138>()
            .0
            .iter()
            .map(|b| Warrior::read(b).unwrap())
            .collect();
        let mut enemies = vec![
            MonsterRecord {
                bytes: decode(case["enemy"].as_str().unwrap())
            };
            case["body"].as_u64().unwrap() as usize
        ];
        let mut random = RandomState::new(12);
        victory_records(&mut warriors, &mut enemies, &mut random).unwrap();
        let expected = decode(case["after_warriors"].as_str().unwrap());
        for (i, (a, b)) in warriors
            .iter()
            .flat_map(|w| w.bytes.iter())
            .zip(&expected)
            .enumerate()
        {
            assert_eq!(
                a,
                b,
                "warrior {} byte {:#x} group {}",
                i / 3138,
                i % 3138,
                case["body"]
            );
        }
        assert_eq!(
            enemies[0].bytes,
            decode(case["after_enemy"].as_str().unwrap())
        );
        assert_eq!(random.seed, case["seed"].as_u64().unwrap() as u32);
        assert_eq!(random.draws, case["draws"].as_u64().unwrap());
    }
}

#[test]
fn army_rewards_and_sorted_loot_match_original_full_records_and_randomness() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let content = centurions_content::Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap();
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../evidence/fixtures/army-victory.json")).unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let mut warriors: Vec<_> = decode(case["warriors"].as_str().unwrap())
            .as_chunks::<3138>()
            .0
            .iter()
            .map(|b| Warrior::read(b).unwrap())
            .collect();
        let mut enemies: Vec<_> = decode(case["enemy"].as_str().unwrap())
            .as_chunks::<288>()
            .0
            .iter()
            .map(|b| MonsterRecord { bytes: b.to_vec() })
            .collect();
        let mut random = RandomState::new(case["initial_seed"].as_u64().unwrap() as u32);
        let (_, loot) = centurions_core::combat::runtime::victory_loot(
            &mut warriors,
            &mut enemies,
            &mut random,
            Some(&content),
        )
        .unwrap();
        let expected = decode(case["after_warriors"].as_str().unwrap());
        for (i, (a, b)) in warriors
            .iter()
            .flat_map(|w| w.bytes.iter())
            .zip(&expected)
            .enumerate()
        {
            assert_eq!(
                a,
                b,
                "warrior {} offset {:#x} seed {}",
                i / 3138,
                i % 3138,
                case["initial_seed"]
            );
        }
        let expected = decode(case["after_enemy"].as_str().unwrap());
        for (i, (a, b)) in enemies
            .iter()
            .flat_map(|m| m.bytes.iter())
            .zip(&expected)
            .enumerate()
        {
            assert_eq!(
                a,
                b,
                "enemy {} offset {:#x} seed {}",
                i / 288,
                i % 288,
                case["initial_seed"]
            );
        }
        let expected: Vec<_> = decode(case["loot"].as_str().unwrap())
            .as_chunks::<22>()
            .0
            .iter()
            .filter(|b| b[..2] != [0, 0])
            .map(|b| b.to_vec())
            .collect();
        assert_eq!(
            loot.iter().map(|i| i.0.to_vec()).collect::<Vec<_>>(),
            expected,
            "loot seed {}",
            case["initial_seed"]
        );
        assert_eq!(random.seed, case["seed"].as_u64().unwrap() as u32);
        assert_eq!(random.draws, case["draws"].as_u64().unwrap());
    }
}
