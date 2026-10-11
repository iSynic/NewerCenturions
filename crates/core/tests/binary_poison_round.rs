use centurions_content::{Warrior, combat::MonsterRecord};
use centurions_core::{RandomState, combat::initialize_turn};
fn decode(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}
#[test]
fn poison_round_matches_original_healing_death_cleanup_duration_and_bleeding() {
    verify(include_str!("../../../evidence/fixtures/poison-round.json"));
}

fn verify(input: &str) {
    let fixture: serde_json::Value = serde_json::from_str(input).unwrap();
    for (index, case) in fixture["cases"].as_array().unwrap().iter().enumerate() {
        let mut warriors: Vec<_> = decode(case["warriors"].as_str().unwrap())
            .as_chunks::<3138>()
            .0
            .iter()
            .map(|w| Warrior::read(w).unwrap())
            .collect();
        let mut enemies = vec![MonsterRecord {
            bytes: decode(case["enemy"].as_str().unwrap()),
        }];
        let mut random = RandomState::new(1);
        let mut combat = centurions_core::combat::Combat {
            battle: 2,
            map: centurions_core::combat::TacticalMap {
                tiles: decode(case["before_map"].as_str().unwrap())
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|v| i16::from_be_bytes(*v))
                    .collect(),
                shift: (0, 0),
                edge_word: 0,
            },
            squad: centurions_core::combat::SquadPlacement {
                positions: (0..6).map(|i| (20 + i, 20)).collect(),
                under_tiles: vec![1155; 6],
            },
            enemies: centurions_core::combat::EnemySetup {
                records: enemies.clone(),
                positions: vec![(30, 30)],
            },
            order: decode(case["before_order"].as_str().unwrap())
                .as_chunks::<2>()
                .0
                .iter()
                .map(|v| i16::from_be_bytes(*v))
                .collect(),
            cursor: 0,
            round: 1,
            death_queue: Default::default(),
            spell_fields: vec![0; 1200],
            field_used: vec![0; 128],
            attack_packet: Some(vec![0; 58]),
            attack_identity: Some([0; 3]),
            weapon_work: centurions_core::combat::weapon_work::WeaponWork::zero(),
            unavailable: 0,
            round_scripts: Default::default(),
        };
        let events =
            centurions_core::combat::runtime::reduce_round(&mut combat, &mut warriors, &mut random)
                .unwrap();
        if let Some(sounds) = case.get("tick_sounds") {
            assert_eq!(
                events,
                sounds
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|id| centurions_core::Event::Sound {
                        resource_id: id.as_i64().unwrap() as i16
                    })
                    .collect::<Vec<_>>()
            );
        }
        assert_eq!(
            combat
                .map
                .tiles
                .iter()
                .flat_map(|v| v.to_be_bytes())
                .collect::<Vec<_>>(),
            decode(case["map"].as_str().unwrap()),
            "map case {index}"
        );
        assert_eq!(
            combat
                .order
                .iter()
                .flat_map(|v| v.to_be_bytes())
                .collect::<Vec<_>>(),
            decode(case["order"].as_str().unwrap()),
            "order case {index}"
        );
        enemies = combat.enemies.records;
        initialize_turn(
            case["body"].as_i64().unwrap() as i16,
            &mut warriors,
            &mut enemies,
            &[12, 14, 12, 14, 10, 12, 10, 10, 14, 16, 0, 12],
            &mut random,
        )
        .unwrap();
        assert_eq!(
            warriors
                .iter()
                .flat_map(|w| w.bytes.clone())
                .collect::<Vec<_>>(),
            decode(case["after_warriors"].as_str().unwrap()),
            "body {}",
            case["body"]
        );
        assert_eq!(
            enemies[0].bytes,
            decode(case["after_enemy"].as_str().unwrap())
        );
        assert_eq!(random.seed, case["seed"].as_u64().unwrap() as u32);
        assert_eq!(random.draws, case["draws"].as_u64().unwrap());
    }
}
