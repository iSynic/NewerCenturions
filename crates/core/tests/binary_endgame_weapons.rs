use centurions_content::{Content, Warrior, combat::MonsterRecord};
use centurions_core::{
    Event, RandomState,
    combat::{Combat, EnemySetup, SquadPlacement, TacticalMap, enemy_weapons, runtime, spells},
};

fn bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}
fn fixture() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "../../../evidence/fixtures/endgame-weapons.json"
    ))
    .unwrap()
}
fn attacks() -> Vec<u8> {
    std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../NewCenturions/TheNewCenturions/Data Files/Data S"
    ))
    .unwrap()
}

fn items() -> Vec<Vec<u8>> {
    std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../NewCenturions/TheNewCenturions/Data Files/Data ID"
    ))
    .unwrap()
    .as_chunks::<150>()
    .0
    .iter()
    .map(|record| record.to_vec())
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
            "{context} byte {offset:x}: {} != {}",
            actual[offset], expected[offset]
        );
    }
}

#[test]
fn required_endgame_enemy_preparation_matches_native_item_context_and_packet() {
    let fixture = fixture();
    let attacks = attacks();
    let cases = fixture["preparation_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 144);
    for (index, case) in cases.iter().enumerate() {
        let mut enemy = MonsterRecord {
            bytes: bytes(case["enemy_before"].as_str().unwrap()),
        };
        let prepared = enemy_weapons::prepare(
            &mut enemy,
            &bytes(case["weapon"].as_str().unwrap()),
            &bytes(case["ammo"].as_str().unwrap()),
            &attacks,
        )
        .unwrap();
        assert_eq!(
            prepared.packet,
            bytes(case["packet"].as_str().unwrap()),
            "case{index} packet"
        );
        assert_eq!(
            enemy.bytes,
            bytes(case["enemy_after"].as_str().unwrap()),
            "case{index} enemy"
        );
    }
}

#[test]
fn required_endgame_enemy_firearms_match_whole_party_and_native_randomness() {
    let fixture = fixture();
    let attacks = attacks();
    let cases = fixture["firearm_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 288);
    for (index, case) in cases.iter().enumerate() {
        let mut party = bytes(case["warriors_before"].as_str().unwrap());
        let member = case["member"].as_u64().unwrap() as usize;
        let mut warrior = Warrior::read(&party[member * 3138..(member + 1) * 3138]).unwrap();
        let mut enemy = MonsterRecord {
            bytes: bytes(case["enemy_before"].as_str().unwrap()),
        };
        let prepared = enemy_weapons::prepare(
            &mut enemy,
            &bytes(case["weapon"].as_str().unwrap()),
            &bytes(case["ammo"].as_str().unwrap()),
            &attacks,
        )
        .unwrap();
        assert_eq!(
            prepared.packet,
            bytes(case["packet"].as_str().unwrap()),
            "case{index} packet"
        );
        let mut random = RandomState::new(case["seed"].as_u64().unwrap() as u32);
        let result =
            enemy_weapons::resolve(&mut warrior, &mut enemy, &prepared, &mut random, 0).unwrap();
        party[member * 3138..(member + 1) * 3138].copy_from_slice(&warrior.bytes);
        assert_bytes(
            &party,
            &bytes(case["warriors_after"].as_str().unwrap()),
            &format!("case{index} full party"),
        );
        assert_eq!(
            enemy.bytes,
            bytes(case["enemy_after"].as_str().unwrap()),
            "case{index} enemy"
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
            result.sounds,
            case["sounds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_i64().unwrap() as i16)
                .collect::<Vec<_>>(),
            "case{index} sounds"
        );
    }
}

#[test]
fn required_endgame_default_melee_selection_matches_native_records_saves_and_both_attack_indices() {
    let fixture = fixture();
    let items = items();
    let cases = fixture["melee_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 72);
    for (index, case) in cases.iter().enumerate() {
        let mut party = bytes(case["warriors_before"].as_str().unwrap());
        let member = case["member"].as_u64().unwrap() as usize;
        let mut warrior = Warrior::read(&party[member * 3138..(member + 1) * 3138]).unwrap();
        let mut enemy = MonsterRecord {
            bytes: bytes(case["enemy_before"].as_str().unwrap()),
        };
        let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        let weapon = i16::from_be_bytes(enemy.bytes[0x28..0x2a].try_into().unwrap()) as usize;
        let (result, packet) = enemy_weapons::melee_with_packet(
            &mut warrior,
            &mut enemy,
            &items[weapon],
            &mut random,
            0,
            0,
            40 + member as i16,
            &attacks(),
            &[0; 58],
        )
        .unwrap();
        assert_bytes(
            &packet,
            &bytes(case["packet"].as_str().unwrap()),
            &format!("case{index} packet"),
        );
        party[member * 3138..(member + 1) * 3138].copy_from_slice(&warrior.bytes);
        assert_bytes(
            &party,
            &bytes(case["warriors_after"].as_str().unwrap()),
            &format!("case{index} party"),
        );
        assert_bytes(
            &enemy.bytes,
            &bytes(case["enemy_after"].as_str().unwrap()),
            &format!("case{index} attacker"),
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
            case["sounds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|sound| sound.as_i64().unwrap() as i16)
                .collect::<Vec<_>>(),
            vec![if result.damage != 0 {
                i16::from_be_bytes(items[weapon][14..16].try_into().unwrap())
            } else {
                650
            }],
            "case{index} native melee hit/miss audio"
        );
        assert!(result.sounds.is_empty());
    }
}

#[test]
fn required_endgame_adjacent_melee_caller_matches_native_action_count_and_cost() {
    let fixture = fixture();
    let items = items();
    let cases = fixture["caller_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 10);
    for (index, case) in cases.iter().enumerate() {
        let raw = bytes(case["warriors_before"].as_str().unwrap());
        let mut warriors: Vec<_> = raw
            .as_chunks::<3138>()
            .0
            .iter()
            .map(|record| Warrior::read(record).unwrap())
            .collect();
        let mut combat = Combat {
            battle: 117,
            map: TacticalMap {
                tiles: words(&bytes(case["map_before"].as_str().unwrap())),
                shift: (0, 0),
                edge_word: 0,
            },
            squad: SquadPlacement {
                positions: vec![(21, 20), (41, 40), (42, 40), (43, 40), (44, 40), (45, 40)],
                under_tiles: vec![1155; 6],
            },
            enemies: EnemySetup {
                records: vec![MonsterRecord {
                    bytes: bytes(case["enemy_before"].as_str().unwrap()),
                }],
                positions: vec![(20, 20)],
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
        let events = runtime::check_neighbors_with_definitions(
            &mut combat,
            &mut warriors,
            &mut random,
            10,
            &items,
        )
        .unwrap();
        let actual: Vec<_> = warriors
            .iter()
            .flat_map(|warrior| warrior.bytes.iter().copied())
            .collect();
        assert_bytes(
            &actual,
            &bytes(case["warriors_after"].as_str().unwrap()),
            &format!("case{index} party"),
        );
        assert_bytes(
            &combat.enemies.records[0].bytes,
            &bytes(case["enemy_after"].as_str().unwrap()),
            &format!("case{index} attacker"),
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
                .map(|sound| sound.as_i64().unwrap() as i16)
                .collect::<Vec<_>>(),
            "case{index} native caller sounds"
        );
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
            case["calls"]
                .as_array()
                .unwrap()
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
fn required_endgame_enemy_firearms_match_native_friendly_body_targets() {
    let fixture = fixture();
    let items = items();
    let attacks = attacks();
    let cases = fixture["body_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 256);
    for (index, case) in cases.iter().enumerate() {
        let mut enemy = MonsterRecord {
            bytes: bytes(case["enemy_before"].as_str().unwrap()),
        };
        let mut target = MonsterRecord {
            bytes: bytes(case["target_before"].as_str().unwrap()),
        };
        let id = i16::from_be_bytes(enemy.bytes[0x26..0x28].try_into().unwrap());
        let ammo =
            enemy_weapons::ammo_id(&enemy, case["context"].as_i64().unwrap() as i16).unwrap();
        let prepared = enemy_weapons::prepare(
            &mut enemy,
            &items[id as usize],
            &items[ammo as usize],
            &attacks,
        )
        .unwrap();
        assert_bytes(
            &prepared.packet,
            &bytes(case["packet"].as_str().unwrap()),
            &format!("body{index} packet"),
        );
        let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        let result =
            spells::resolve_enemy_firearm_body(&mut target, &enemy, &prepared, 0, &mut random)
                .unwrap();
        assert_bytes(
            &target.bytes,
            &bytes(case["target_after"].as_str().unwrap()),
            &format!("body{index} target"),
        );
        assert_bytes(
            &enemy.bytes,
            &bytes(case["enemy_after"].as_str().unwrap()),
            &format!("body{index} attacker"),
        );
        assert_eq!(
            random.seed,
            case["seed_after"].as_u64().unwrap() as u32,
            "body{index} seed"
        );
        assert_eq!(
            random.draws,
            case["rng_draws"].as_u64().unwrap(),
            "body{index} draws"
        );
        assert_eq!(
            result.sounds,
            case["sounds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_i64().unwrap() as i16)
                .collect::<Vec<_>>(),
            "body{index} sounds"
        );
    }
}

fn word(raw: &[u8], offset: usize) -> i16 {
    i16::from_be_bytes(raw[offset..offset + 2].try_into().unwrap())
}

#[test]
fn required_endgame_ranged_to_natural_fallback_matches_original_full_records_and_randomness() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/endgame-weapons.json"
    ))
    .unwrap();
    let cases = fixture["natural_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 432);
    for (index, case) in cases.iter().enumerate() {
        let mut party = bytes(case["warriors_before"].as_str().unwrap());
        let member = case["member"].as_u64().unwrap() as usize;
        let mut warrior = Warrior::read(&party[member * 3138..(member + 1) * 3138]).unwrap();
        let mut enemy = MonsterRecord {
            bytes: bytes(case["enemy_before"].as_str().unwrap()),
        };
        let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        let (result, packet) = centurions_core::combat::attack::natural_attack_with_packet(
            &mut warrior,
            &mut enemy,
            case["index"].as_u64().unwrap() as usize,
            &mut random,
            &std::fs::read(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../NewCenturions/TheNewCenturions/Data Files/Data S"
            ))
            .unwrap(),
            &[0; 58],
        )
        .unwrap();
        assert_eq!(
            packet,
            bytes(case["packet"].as_str().unwrap()),
            "case{index} packet"
        );
        party[member * 3138..(member + 1) * 3138].copy_from_slice(&warrior.bytes);
        assert_eq!(
            party,
            bytes(case["warriors_after"].as_str().unwrap()),
            "case{index} party"
        );
        assert_eq!(
            enemy.bytes,
            bytes(case["enemy_after"].as_str().unwrap()),
            "case{index} enemy"
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
            result
                .sounds
                .into_iter()
                .chain([centurions_core::combat::ally_turns::result_audio(
                    &enemy,
                    0,
                    result.damage,
                    0
                )
                .unwrap()])
                .collect::<Vec<_>>(),
            case["sounds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_i64().unwrap() as i16)
                .collect::<Vec<_>>(),
            "case{index} sounds"
        );
    }
}

#[test]
fn required_endgame_firearm_whole_caller_matches_native_cast_order_records_and_work_context() {
    let f: serde_json::Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/endgame-weapons.json"
    ))
    .unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let content = Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap();
    let mission = content.map(0).unwrap();
    let cases = f["full_caller_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 256);
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
            attack_packet: Some(bytes(case["packet_before"].as_str().unwrap())),
            attack_identity: Some(
                case["identity_before"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_i64().unwrap() as i16)
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap(),
            ),
            weapon_work: centurions_core::combat::weapon_work::WeaponWork::zero(),
            unavailable: 0,
            round_scripts: Default::default(),
        };
        let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        let mut work = runtime::EnemyWork {
            item_context: Some(case["item_context_before"].as_i64().unwrap() as i16),
            attack_count: case["attack_count_before"].as_i64().unwrap() as i16,
            packet: combat.attack_packet.clone(),
            identity: combat.attack_identity,
            ..runtime::EnemyWork::default()
        };
        let events = runtime::enemy_action_with_work(
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
        assert_eq!(work.packet, combat.attack_packet, "case{index} packet work");
        assert_eq!(
            work.identity, combat.attack_identity,
            "case{index} identity work"
        );
        assert_eq!(
            combat.attack_packet,
            Some(bytes(case["packet_after"].as_str().unwrap())),
            "case{index} packet"
        );
        assert_eq!(
            combat.attack_identity,
            Some(
                case["identity_after"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_i64().unwrap() as i16)
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap()
            ),
            "case{index} identity"
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
            combat.unavailable,
            case["unavailable_after"].as_i64().unwrap() as i8,
            "case{index} unavailable"
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
            let no_human_warrior = prior_party
                .as_chunks::<3138>()
                .0
                .iter()
                .all(|warrior| warrior[0x9df] == 0 || word(warrior, 0x10) != 0);
            if resolution["kind"] == "natural" && no_human_warrior {
                assert_eq!(
                    resolution["draws_before"], resolution["draws_after"],
                    "case{index} native attack2 early return"
                );
                prior_party = bytes(resolution["party_after"].as_str().unwrap());
                prior_records = bytes(resolution["records_after"].as_str().unwrap());
                continue;
            }
            let mask = if resolution["kind"] == "natural" {
                let mut mask = vec![0; 110];
                mask[resolution["target"].as_u64().unwrap() as usize] = 1;
                mask
            } else {
                assert_eq!(resolution["count"], 1, "case{index} native caller count");
                words(&bytes(resolution["mask"].as_str().unwrap()))
            };
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
                    resolution["attacker"].as_i64().unwrap_or(10) as i16,
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
