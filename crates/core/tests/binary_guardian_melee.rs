use centurions_content::{Content, Warrior, combat::MonsterRecord, word};
use centurions_core::{
    Command, Phase, RandomState, RuleError, Session, apply,
    combat::{Combat, EnemySetup, SquadPlacement, TacticalMap, ally_turns, enemy_weapons},
};

fn bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|at| u8::from_str_radix(&hex[at..at + 2], 16).unwrap())
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
    if let Some(at) = actual.iter().zip(expected).position(|(a, b)| a != b) {
        panic!("{context} byte {at:x}: {} != {}", actual[at], expected[at]);
    }
}

fn verify(target_kind: i64) {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/guardian-melee.json"
    ))
    .unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let content = Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap();
    let attacks = &content.map(0).unwrap().combat.as_ref().unwrap().attacks;
    let weapon = &content.items[187];
    let mut checked = 0;
    for (case_index, case) in fixture["cases"].as_array().unwrap().iter().enumerate() {
        if case["target_kind"].as_i64().unwrap() != target_kind {
            continue;
        }
        let context = format!("guardian case{case_index}");
        let source = case["source"].as_i64().unwrap() as i16;
        let target = case["target"].as_i64().unwrap() as i16;
        let actor = case["actor"].as_i64().unwrap() as i16;
        let index = case["index"].as_u64().unwrap() as usize;
        let mut warriors: Vec<_> = bytes(case["warriors_before"].as_str().unwrap())
            .as_chunks::<3138>()
            .0
            .iter()
            .map(|record| Warrior::read(record).unwrap())
            .collect();
        let mut order = vec![-1; 110];
        order[1] = actor;
        let mut combat = Combat {
            battle: 85,
            map: TacticalMap {
                tiles: words(&bytes(case["map_before"].as_str().unwrap())),
                shift: (0, 0),
                edge_word: 0,
            },
            squad: SquadPlacement {
                positions: vec![(21, 20); 6],
                under_tiles: vec![1155; 6],
            },
            enemies: EnemySetup {
                records: bytes(case["records_before"].as_str().unwrap())
                    .as_chunks::<288>()
                    .0
                    .iter()
                    .map(|record| MonsterRecord {
                        bytes: record.to_vec(),
                    })
                    .collect(),
                positions: bytes(case["positions_before"].as_str().unwrap())
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .map(|position| {
                        (
                            i16::from_be_bytes(position[..2].try_into().unwrap()),
                            i16::from_be_bytes(position[2..].try_into().unwrap()),
                        )
                    })
                    .collect(),
            },
            order,
            cursor: 1,
            round: 0,
            weapon_work: centurions_core::combat::weapon_work::WeaponWork::zero(),
            unavailable: 0,
            death_queue: Default::default(),
            spell_fields: vec![0; 1200],
            field_used: vec![0; 128],
            round_scripts: Default::default(),
            attack_packet: Some(vec![0; 58]),
            attack_identity: Some([2, 4, 6]),
        };
        let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        let (result, packet) = if target_kind == 0 {
            enemy_weapons::melee_warrior_with_packet(
                &mut combat,
                &mut warriors,
                &mut random,
                (source, target, index, actor),
                weapon,
                0,
                attacks,
                &[0; 58],
            )
        } else {
            enemy_weapons::melee_enemy_with_packet(
                &mut combat,
                &mut random,
                (source, target, index, actor),
                weapon,
                0,
                attacks,
                &[0; 58],
            )
        }
        .unwrap_or_else(|error| panic!("{context}: {error:?}"));
        assert_bytes(
            &combat
                .enemies
                .records
                .iter()
                .flat_map(|record| record.bytes.iter().copied())
                .collect::<Vec<_>>(),
            &bytes(case["records_after"].as_str().unwrap()),
            &format!("{context} records"),
        );
        assert_bytes(
            &warriors
                .iter()
                .flat_map(|warrior| warrior.bytes.iter().copied())
                .collect::<Vec<_>>(),
            &bytes(case["warriors_after"].as_str().unwrap()),
            &format!("{context} warriors"),
        );
        assert_bytes(
            &packet,
            &bytes(case["packet_after"].as_str().unwrap()),
            &format!("{context} packet"),
        );
        assert_eq!(
            random.seed,
            case["seed_after"].as_u64().unwrap() as u32,
            "{context}"
        );
        assert_eq!(
            random.draws,
            case["rng_draws"].as_u64().unwrap(),
            "{context}"
        );
        let mut sounds = result.sounds;
        sounds.push(
            ally_turns::result_audio(
                &combat.enemies.records[(source - 10) as usize],
                index,
                result.damage,
                word(weapon, 14).unwrap(),
            )
            .unwrap(),
        );
        assert_eq!(serde_json::json!(sounds), case["sounds"], "{context}");
        assert!(!result.killed, "{context}");
        assert_eq!(case["result"], 0, "{context}");
        assert_eq!(
            combat.map.tiles,
            words(&bytes(case["map_after"].as_str().unwrap())),
            "{context} map"
        );
        let positions: Vec<_> = combat
            .enemies
            .positions
            .iter()
            .flat_map(|(x, y)| x.to_be_bytes().into_iter().chain(y.to_be_bytes()))
            .collect();
        assert_bytes(
            &positions,
            &bytes(case["positions_after"].as_str().unwrap()),
            &format!("{context} positions"),
        );
        checked += 1;
    }
    assert_eq!(checked, 288);
}

#[test]
fn guardian_melee_matches_native_warrior_poison_packet_audio_and_retained_actor() {
    verify(0);
}

#[test]
fn guardian_melee_matches_native_monster_poison_packet_audio_and_retained_actor() {
    verify(1);
}

fn caller_combat(case: &serde_json::Value) -> Combat {
    let mut order = vec![-1; 110];
    order[1] = 22;
    Combat {
        battle: 85,
        map: TacticalMap {
            tiles: words(&bytes(case["map_before"].as_str().unwrap())),
            shift: (0, 0),
            edge_word: 0,
        },
        squad: SquadPlacement {
            positions: vec![(60, 60); 6],
            under_tiles: vec![1155; 6],
        },
        enemies: EnemySetup {
            records: bytes(case["records_before"].as_str().unwrap())
                .as_chunks::<288>()
                .0
                .iter()
                .map(|record| MonsterRecord {
                    bytes: record.to_vec(),
                })
                .collect(),
            positions: words(&bytes(case["positions_before"].as_str().unwrap()))
                .as_chunks::<2>()
                .0
                .iter()
                .map(|position| (position[0], position[1]))
                .collect(),
        },
        order,
        cursor: 1,
        round: 0,
        weapon_work: centurions_core::combat::weapon_work::WeaponWork::zero(),
        unavailable: 0,
        death_queue: Default::default(),
        spell_fields: vec![0; 1200],
        field_used: vec![0; 128],
        round_scripts: Default::default(),
        attack_packet: Some(vec![0; 58]),
        attack_identity: Some([2, 4, 6]),
    }
}

#[test]
fn guardian_reaction_caller_matches_native_retained_actor_reload_and_natural_attacks() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/guardian-melee.json"
    ))
    .unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let content = Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap();
    let attacks = &content.map(0).unwrap().combat.as_ref().unwrap().attacks;
    let cases = fixture["caller_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 16);
    for (case_index, case) in cases.iter().enumerate() {
        let context = format!("guardian caller{case_index}");
        let mut combat = caller_combat(case);
        let mut warriors: Vec<_> = bytes(case["warriors_before"].as_str().unwrap())
            .as_chunks::<3138>()
            .0
            .iter()
            .map(|record| Warrior::read(record).unwrap())
            .collect();
        let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        let events = centurions_core::combat::runtime::check_neighbors_with_context(
            &mut combat,
            &mut warriors,
            &mut random,
            22,
            &content.items,
            attacks,
        )
        .unwrap_or_else(|error| panic!("{context}: {error:?}"));
        let actual: Vec<_> = combat
            .enemies
            .records
            .iter()
            .flat_map(|record| record.bytes.iter().copied())
            .collect();
        assert_bytes(
            &actual,
            &bytes(case["records_after"].as_str().unwrap()),
            &format!("{context} records"),
        );
        let actual: Vec<_> = warriors
            .iter()
            .flat_map(|warrior| warrior.bytes.iter().copied())
            .collect();
        assert_bytes(
            &actual,
            &bytes(case["warriors_after"].as_str().unwrap()),
            &format!("{context} warriors"),
        );
        assert_bytes(
            combat.attack_packet.as_ref().unwrap(),
            &bytes(case["packet_after"].as_str().unwrap()),
            &context,
        );
        assert_eq!(
            combat.map.tiles,
            words(&bytes(case["map_after"].as_str().unwrap())),
            "{context}"
        );
        assert_eq!(
            combat.order,
            words(&bytes(case["order_after"].as_str().unwrap())),
            "{context}"
        );
        let actual: Vec<_> = combat
            .enemies
            .positions
            .iter()
            .flat_map(|(x, y)| x.to_be_bytes().into_iter().chain(y.to_be_bytes()))
            .collect();
        assert_bytes(
            &actual,
            &bytes(case["positions_after"].as_str().unwrap()),
            &context,
        );
        assert_eq!(
            random.seed,
            case["seed_after"].as_u64().unwrap() as u32,
            "{context}"
        );
        assert_eq!(
            random.draws,
            case["rng_draws"].as_u64().unwrap(),
            "{context}"
        );
        let sounds: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                centurions_core::Event::Sound { resource_id } => Some(*resource_id),
                _ => None,
            })
            .collect();
        assert_eq!(serde_json::json!(sounds), case["sounds"], "{context}");
        let calls: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                centurions_core::Event::AttackResolved {
                    attacker, target, ..
                } => Some((*attacker, *target)),
                _ => None,
            })
            .collect();
        let expected: Vec<_> = case["calls"]
            .as_array()
            .unwrap()
            .iter()
            .map(|call| {
                (
                    call[0].as_i64().unwrap() as i16,
                    call[1].as_i64().unwrap() as i16,
                )
            })
            .collect();
        assert_eq!(calls, expected, "{context}");
        let steps = case["attack_steps"].as_array().unwrap();
        assert_eq!(steps[0]["actor_current_before"], 0);
        assert_eq!(steps[0]["actor_current_after"], 187);
        assert_eq!(steps[1]["actor_current_before"], 187);
        assert_eq!(steps[1]["actor_current_after"], 0);
    }
}

#[test]
fn unsupported_guardian_counterattack_rolls_back_complete_enemy_turn_command() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/guardian-melee.json"
    ))
    .unwrap();
    let case = &fixture["caller_cases"][0];
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let content = Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap();
    let mut state = Session::new(&content);
    apply(Command::ImportStarters, &mut state, &content).unwrap();
    apply(
        Command::EnterMission {
            name: "The Thunder of Guns".into(),
        },
        &mut state,
        &content,
    )
    .unwrap();
    state.phase = Phase::Combat;
    state.exploration.as_mut().unwrap().presentation = None;
    state.combat = Some(caller_combat(case));
    state.warriors = bytes(case["warriors_before"].as_str().unwrap())
        .as_chunks::<3138>()
        .0
        .iter()
        .map(|record| Warrior::read(record).unwrap())
        .collect();
    state.random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
    // Primary 2 avoids the scientist branch but cannot reload the transient 187 as a natural attack.
    state.combat.as_mut().unwrap().enemies.records[12].bytes[0x26..0x28]
        .copy_from_slice(&2_i16.to_be_bytes());
    state.validate(&content).unwrap();
    let before = state.clone();

    let mut partial = state.clone();
    let error = centurions_core::combat::runtime::enemy_turn(&mut partial, &content).unwrap_err();
    assert!(matches!(
        error,
        RuleError::Unsupported {
            action: "combat.monster_attack",
            ..
        }
    ));
    let combat = partial.combat.as_ref().unwrap();
    assert_eq!(
        word(
            &before.combat.as_ref().unwrap().enemies.records[12].bytes,
            0x4e
        )
        .unwrap(),
        0
    );
    assert_eq!(word(&combat.enemies.records[12].bytes, 0x4e).unwrap(), 187);
    let guardian = case["calls"][0][0].as_i64().unwrap() as usize - 10;
    assert_eq!(combat.enemies.records[guardian].bytes[0x104], 0);
    assert!(partial.random.draws > before.random.draws + 2);
    assert_ne!(
        combat.attack_packet,
        before.combat.as_ref().unwrap().attack_packet
    );

    assert!(matches!(
        apply(Command::RunEnemyTurn, &mut state, &content),
        Err(RuleError::Unsupported {
            action: "combat.monster_attack",
            ..
        })
    ));
    assert_eq!(state, before);
}
