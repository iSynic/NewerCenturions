use centurions_content::Warrior;
use centurions_core::{RandomState, combat::spells::resolve_warriors};

fn decode(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
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
fn enemy_spell_damage_matches_original_complete_party_and_random_consumption() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/enemy-spell-damage.json"
    ))
    .unwrap();
    for (index, case) in fixture["cases"].as_array().unwrap().iter().enumerate() {
        let warriors = decode(case["before"].as_str().unwrap())
            .as_chunks::<3138>()
            .0
            .iter()
            .map(|w| Warrior::read(w).unwrap())
            .collect::<Vec<_>>();
        let packet = decode(case["packet"].as_str().unwrap());
        let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        let result = resolve_warriors(
            &warriors,
            &packet,
            (case["spell_id"].as_i64().unwrap() % 1000 / 100 - 1) as i16,
            case["count"].as_i64().unwrap() as i16,
            &[1; 6],
            false,
            &mut random,
        )
        .unwrap();
        let actual = result
            .warriors
            .iter()
            .flat_map(|w| w.bytes.iter().copied())
            .collect::<Vec<_>>();
        assert_bytes(
            &actual,
            &decode(case["after"].as_str().unwrap()),
            &format!("case {index}"),
        );
        assert_eq!(
            random.seed,
            case["seed_after"].as_u64().unwrap() as u32,
            "seed case {index}"
        );
        assert_eq!(
            random.draws,
            case["rng_draws"].as_u64().unwrap(),
            "draws case {index}"
        );
        assert_eq!(
            result
                .targets
                .iter()
                .flat_map(|(_, target)| target.sounds.iter().copied())
                .collect::<Vec<_>>(),
            case["sounds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|s| s.as_i64().unwrap() as i16)
                .collect::<Vec<_>>(),
            "sounds case {index}"
        );
    }
}

#[test]
fn enemy_spell_selection_matches_original_targets_caster_and_random_consumption() {
    use centurions_content::{combat::MonsterRecord, mission::OpeningMission};
    use centurions_core::combat::{
        Combat, EnemySetup, SquadPlacement, TacticalMap, spells::select_targets,
    };
    let mut additional: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../evidence/fixtures/additional-spells.json"),
        )
        .unwrap(),
    )
    .unwrap();
    additional["cases"] = additional["selection_cases"].take();
    let fixtures: [serde_json::Value; 4] = [
        serde_json::from_str(include_str!(
            "../../../evidence/fixtures/enemy-spell-selection.json"
        ))
        .unwrap(),
        serde_json::from_str(
            &std::fs::read_to_string(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../evidence/fixtures/firearm-targets.json"),
            )
            .unwrap(),
        )
        .unwrap(),
        additional,
        serde_json::from_str(
            &std::fs::read_to_string(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../evidence/fixtures/condition-targets.json"),
            )
            .unwrap(),
        )
        .unwrap(),
    ];
    for fixture in fixtures {
        let properties = decode(fixture["properties"].as_str().unwrap());
        let areas = decode(fixture["areas"].as_str().unwrap());
        let mission = OpeningMission {
            name: "fixture".into(),
            start: (2, 2),
            header: vec![],
            regions: vec![],
            floor: vec![],
            objects: vec![],
            events: vec![],
            tile_properties: properties[64..12064].to_vec(),
            terrain: 2,
            default_tile: 155,
            minutes_per_tick: 5,
            shared: std::sync::Arc::new(Default::default()),
        };
        for (index, case) in fixture["cases"].as_array().unwrap().iter().enumerate() {
            let warriors = decode(case["party"].as_str().unwrap())
                .as_chunks::<3138>()
                .0
                .iter()
                .map(|w| Warrior::read(w).unwrap())
                .collect::<Vec<_>>();
            let distance = case["distance"].as_i64().unwrap() as i16;
            let tiles = decode(case["map"].as_str().unwrap())
                .as_chunks::<2>()
                .0
                .iter()
                .map(|v| i16::from_be_bytes(*v))
                .collect();
            let combat = Combat {
                battle: 28,
                map: TacticalMap {
                    tiles,
                    shift: (0, 0),
                    edge_word: 0,
                },
                squad: SquadPlacement {
                    positions: (0..6)
                        .map(|i| {
                            (
                                20 + distance + i * if case["weapon_id"].is_null() { 1 } else { 4 },
                                20,
                            )
                        })
                        .collect(),
                    under_tiles: vec![1155; 6],
                },
                enemies: EnemySetup {
                    records: vec![MonsterRecord {
                        bytes: decode(case["caster_before"].as_str().unwrap()),
                    }],
                    positions: vec![(20, 20)],
                },
                order: vec![-1, 10],
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
            let packet = decode(case["packet"].as_str().unwrap());
            let result = if case["weapon_id"].is_null() {
                select_targets(
                    &combat,
                    &warriors,
                    &mission,
                    &areas,
                    0,
                    &packet,
                    &mut random,
                )
            } else {
                centurions_core::combat::spells::select_firearm_targets(
                    &combat,
                    &warriors,
                    &mission,
                    &areas,
                    0,
                    &centurions_core::combat::firearm::PreparedFirearm {
                        packet,
                        slot: 0,
                        shots: case["shots"].as_i64().unwrap() as i16,
                        rounds: 0,
                        weight: 0,
                        armor_filter: false,
                        sound: 0,
                    },
                    &mut random,
                )
            };
            if case["invalid_shape"].as_bool().unwrap_or(false) {
                assert!(
                    matches!(result, Err(centurions_core::RuleError::Unsupported { .. })),
                    "invalid shape case {index}"
                );
                assert_eq!(random.seed, case["seed_before"].as_u64().unwrap() as u32);
                assert_eq!(random.draws, 0);
                continue;
            }
            let result = result.unwrap();
            assert_bytes(
                &result.caster.bytes,
                &decode(case["caster_after"].as_str().unwrap()),
                &format!("caster case {index}"),
            );
            assert_eq!(
                result.range,
                case["range"].as_i64().unwrap() as i16,
                "range case {index}"
            );
            assert_eq!(
                result.nearest,
                case["nearest"].as_i64().unwrap() as i16,
                "nearest case {index}"
            );
            assert_eq!(
                result.count,
                case["count"].as_i64().unwrap() as i16,
                "count case {index}"
            );
            assert_eq!(
                result
                    .distances
                    .iter()
                    .flat_map(|v| v.to_be_bytes())
                    .collect::<Vec<_>>(),
                decode(case["distances"].as_str().unwrap()),
                "distances case {index}"
            );
            let mut targets = vec![255; 42];
            for (slot, target) in result.targets.iter().enumerate() {
                targets[slot * 6..slot * 6 + 6].copy_from_slice(
                    &[target.first, target.second, target.kind]
                        .iter()
                        .flat_map(|v| v.to_be_bytes())
                        .collect::<Vec<_>>(),
                );
            }
            assert_bytes(
                &targets,
                &decode(case["targets"].as_str().unwrap()),
                &format!("targets case {index}"),
            );
            assert_eq!(
                !result.targets.is_empty(),
                case["cast"].as_bool().unwrap(),
                "cast case {index}"
            );
            assert_eq!(
                random.seed,
                case["seed_after"].as_u64().unwrap() as u32,
                "seed case {index}"
            );
            assert_eq!(
                random.draws,
                case["rng_draws"].as_u64().unwrap(),
                "draws case {index}"
            );
        }
    }
}

#[test]
fn spell_target_masks_reflection_and_fields_match_original_complete_buffers() {
    use centurions_content::combat::MonsterRecord;
    use centurions_core::combat::{
        Combat, EnemySetup, SquadPlacement, TacticalMap,
        spells::{SpellContext, SpellTarget, target_mask},
    };
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/spell-targets.json"
    ))
    .unwrap();
    let areas = decode(fixture["areas"].as_str().unwrap());
    let tiles: Vec<i16> = decode(fixture["map"].as_str().unwrap())
        .as_chunks::<2>()
        .0
        .iter()
        .map(|v| i16::from_be_bytes(*v))
        .collect();
    let party = decode(fixture["party"].as_str().unwrap());
    for (index, case) in fixture["cases"].as_array().unwrap().iter().enumerate() {
        let variant = case["variant"].as_i64().unwrap();
        let mut warriors = party
            .as_chunks::<3138>()
            .0
            .iter()
            .map(|w| Warrior::read(w).unwrap())
            .collect::<Vec<_>>();
        if variant == 1 {
            for warrior in &mut warriors {
                warrior.bytes[0x9a..0x9c].copy_from_slice(&1_i16.to_be_bytes());
            }
        }
        let monsters = decode(case["records"].as_str().unwrap())
            .as_chunks::<288>()
            .0
            .iter()
            .map(|m| MonsterRecord { bytes: m.to_vec() })
            .collect();
        let combat = Combat {
            battle: 28,
            map: TacticalMap {
                tiles: tiles.clone(),
                shift: (0, 0),
                edge_word: 0,
            },
            squad: SquadPlacement {
                positions: (0..6).map(|i| (20 + i, 20)).collect(),
                under_tiles: vec![1155; 6],
            },
            enemies: EnemySetup {
                records: monsters,
                positions: vec![(20, 22), (22, 22)],
            },
            order: vec![-1, 10],
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
        let mut fields = vec![0; 1200];
        if variant == 2 {
            for slot in fields.as_chunks_mut::<20>().0 {
                slot[15] = 1;
            }
        }
        let packet = decode(case["packet"].as_str().unwrap());
        let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        let result = target_mask(
            &combat,
            &warriors,
            &areas,
            &SpellTarget {
                first: case["first"].as_i64().unwrap() as i16,
                second: case["second"].as_i64().unwrap() as i16,
                kind: case["kind"].as_i64().unwrap() as i16,
            },
            &SpellContext {
                packet: &packet,
                id: case["spell_id"].as_i64().unwrap() as i16,
                count: case["count"].as_i64().unwrap() as i16,
                caster: 10,
                initiative: 1,
                collision: false,
            },
            &fields,
            &mut random,
        )
        .unwrap();
        assert_bytes(
            &result
                .mask
                .iter()
                .flat_map(|v| v.to_be_bytes())
                .collect::<Vec<_>>(),
            &decode(case["mask"].as_str().unwrap()),
            &format!("mask case {index}"),
        );
        assert_bytes(
            &result.fields,
            &decode(case["fields"].as_str().unwrap()),
            &format!("fields case {index}"),
        );
        assert_eq!(
            random.seed,
            case["seed_after"].as_u64().unwrap() as u32,
            "seed case {index}"
        );
        assert_eq!(
            random.draws,
            case["rng_draws"].as_u64().unwrap(),
            "draws case {index}"
        );
    }
}
