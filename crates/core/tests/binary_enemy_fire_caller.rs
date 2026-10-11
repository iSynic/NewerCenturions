use centurions_content::{Content, Warrior, combat::MonsterRecord};
use centurions_core::{
    Event, RandomState,
    combat::{
        Combat, EnemySetup, SquadPlacement, TacticalMap,
        runtime::{EnemyWork, enemy_action_with_work},
    },
};

fn bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}
fn words(raw: &[u8]) -> Vec<i16> {
    raw.as_chunks::<2>()
        .0
        .iter()
        .map(|word| i16::from_be_bytes(*word))
        .collect()
}
fn assert_bytes(actual: &[u8], expected: &[u8], context: &str) {
    assert_eq!(actual.len(), expected.len(), "{context} size");
    if let Some(offset) = actual.iter().zip(expected).position(|(a, b)| a != b) {
        panic!(
            "{context} byte{offset:x}: {} != {}",
            actual[offset], expected[offset]
        );
    }
}
fn word(raw: &[u8], offset: usize) -> i16 {
    i16::from_be_bytes(raw[offset..offset + 2].try_into().unwrap())
}
#[test]
fn enemy_firearm_whole_caller_matches_native_cast_order_records_and_work_context() {
    let f: serde_json::Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/enemy-fire-caller.json"
    ))
    .unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let content = Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap();
    let mission = content.map(0).unwrap();
    let cases = f["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 240);
    for (index, case) in cases.iter().enumerate() {
        let before_records = bytes(case["records_before"].as_str().unwrap());
        let mut warriors: Vec<_> = bytes(case["warriors_before"].as_str().unwrap())
            .as_chunks::<3138>()
            .0
            .iter()
            .map(|record| Warrior::read(record).unwrap())
            .collect();
        let position_words = words(&bytes(case["warrior_positions"].as_str().unwrap()));
        let mut positions = vec![(20, 20)];
        if before_records.len() > 288 {
            positions.push((24, 20));
        }
        let mut combat = Combat {
            battle: 129,
            map: TacticalMap {
                tiles: words(&bytes(case["map_before"].as_str().unwrap())),
                shift: (0, 0),
                edge_word: 0,
            },
            squad: SquadPlacement {
                positions: position_words
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|p| (p[0], p[1]))
                    .collect(),
                under_tiles: vec![1155; 6],
            },
            enemies: EnemySetup {
                records: before_records
                    .as_chunks::<288>()
                    .0
                    .iter()
                    .map(|record| MonsterRecord {
                        bytes: record.to_vec(),
                    })
                    .collect(),
                positions,
            },
            order: words(&bytes(case["order_before"].as_str().unwrap())),
            cursor: 1,
            round: 0,
            death_queue: Default::default(),
            spell_fields: vec![0; 1200],
            field_used: vec![0; 128],
            attack_packet: Some(vec![0; 58]),
            attack_identity: Some([0; 3]),
            weapon_work: centurions_core::combat::weapon_work::WeaponWork::zero(),
            unavailable: 0,
            round_scripts: Default::default(),
        };
        let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        let mut work = EnemyWork {
            item_context: Some(case["item_context_before"].as_i64().unwrap() as i16),
            attack_count: case["attack_count_before"].as_i64().unwrap() as i16,
            ..EnemyWork::default()
        };
        let events = enemy_action_with_work(
            &mut combat,
            &mut warriors,
            mission,
            &mut random,
            &content.items,
            &mut work,
        )
        .unwrap_or_else(|error| {
            panic!(
                "case{index} MD{} variant{}: {error:?}",
                case["monster_id"], case["variant"]
            )
        });
        let actual_party: Vec<_> = warriors
            .iter()
            .flat_map(|warrior| warrior.bytes.iter().copied())
            .collect();
        let actual_records: Vec<_> = combat
            .enemies
            .records
            .iter()
            .flat_map(|record| record.bytes.iter().copied())
            .collect();
        assert_bytes(
            &actual_party,
            &bytes(case["warriors_after"].as_str().unwrap()),
            &format!("case{index} party"),
        );
        assert_bytes(
            &actual_records,
            &bytes(case["records_after"].as_str().unwrap()),
            &format!("case{index} enemies"),
        );
        assert_eq!(
            combat.map.tiles,
            words(&bytes(case["map_after"].as_str().unwrap())),
            "case{index} map"
        );
        assert_eq!(
            combat.order,
            words(&bytes(case["order_after"].as_str().unwrap())),
            "case{index} order"
        );
        let after_positions = words(&bytes(case["positions_after"].as_str().unwrap()));
        assert_eq!(
            combat.enemies.positions,
            after_positions
                .as_chunks::<2>()
                .0
                .iter()
                .map(|p| (p[0], p[1]))
                .collect::<Vec<_>>(),
            "case{index} positions"
        );
        assert_eq!(
            work.item_context,
            Some(case["item_context_after"].as_i64().unwrap() as i16),
            "case{index} item context"
        );
        assert_eq!(
            work.attack_count,
            case["attack_count_after"].as_i64().unwrap() as i16,
            "case{index} attack count"
        );
        assert_eq!(
            random.seed,
            case["seed_after"].as_u64().unwrap() as u32,
            "case{index} seed"
        );
        assert_eq!(
            random.draws,
            case["rng_draws"].as_u64().unwrap(),
            "case{index} draws"
        );
        assert_eq!(
            events
                .iter()
                .filter_map(|event| match event {
                    Event::Sound { resource_id } => Some(*resource_id),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            case["sounds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_i64().unwrap() as i16)
                .collect::<Vec<_>>(),
            "case{index} sound order"
        );
        let mut expected_hits = vec![];
        let mut prior_party = bytes(case["warriors_before"].as_str().unwrap());
        let mut prior_records = before_records;
        for resolution in case["resolutions"].as_array().unwrap() {
            assert_eq!(resolution["count"], 1, "case{index} native caller count");
            let mask = words(&bytes(resolution["mask"].as_str().unwrap()));
            let after_party = bytes(resolution["party_after"].as_str().unwrap());
            let after_records = bytes(resolution["records_after"].as_str().unwrap());
            for (body, selected) in mask.iter().enumerate() {
                if *selected == 0 {
                    continue;
                }
                let (before, after, hp) = if body < 6 {
                    (
                        &prior_party[body * 3138..(body + 1) * 3138],
                        &after_party[body * 3138..(body + 1) * 3138],
                        0x50,
                    )
                } else if body >= 10 && body < 10 + combat.enemies.records.len() {
                    (
                        &prior_records[(body - 10) * 288..(body - 9) * 288],
                        &after_records[(body - 10) * 288..(body - 9) * 288],
                        0x110,
                    )
                } else {
                    panic!("case{index} unexpected selected body{body}")
                };
                expected_hits.push((
                    10,
                    body as i16,
                    word(before, hp).wrapping_sub(word(after, hp)),
                    word(after, hp) < 1,
                ));
            }
            prior_party = after_party;
            prior_records = after_records;
        }
        let actual_hits = events
            .iter()
            .filter_map(|event| match event {
                Event::AttackResolved {
                    attacker,
                    target,
                    damage,
                    defeated,
                } => Some((*attacker, *target, *damage, *defeated)),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(actual_hits, expected_hits, "case{index} ordered casts");
        if let Some(prelude) = case["prelude"].as_object() {
            assert_eq!(
                prelude["item_context_after"], 0,
                "case{index} native natural reset"
            );
        }
    }
}
