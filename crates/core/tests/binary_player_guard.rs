use centurions_content::{Content, Warrior, combat::MonsterRecord, word};
use centurions_core::{
    Command, Event, Phase, RandomState, Session, apply,
    combat::{Combat, EnemySetup, SquadPlacement, TacticalMap, runtime, weapon_work::WeaponWork},
};
use serde_json::{Value, json};

fn bytes(value: &Value) -> Vec<u8> {
    value
        .as_str()
        .unwrap()
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

fn words(value: &Value) -> Vec<i16> {
    bytes(value)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| i16::from_be_bytes(*pair))
        .collect()
}

fn positions(value: &Value) -> Vec<(i16, i16)> {
    words(value)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| (pair[0], pair[1]))
        .collect()
}

fn warriors(snapshot: &Value) -> Vec<Warrior> {
    bytes(&snapshot["party"])
        .as_chunks::<3138>()
        .0
        .iter()
        .map(|record| Warrior::read(record).unwrap())
        .collect()
}

fn enemies(snapshot: &Value) -> Vec<MonsterRecord> {
    bytes(&snapshot["records"])
        .as_chunks::<288>()
        .0
        .iter()
        .map(|record| MonsterRecord {
            bytes: record.to_vec(),
        })
        .collect()
}

fn work(snapshot: &Value) -> WeaponWork {
    let weapon = bytes(&snapshot["weapon_work"]);
    let ammunition = bytes(&snapshot["ammo_work"]);
    WeaponWork {
        sound: Some(word(&weapon, 0x0e).unwrap()),
        weapon_armor_filter: Some(word(&weapon, 0x88).unwrap()),
        ammunition_armor_filter: Some(word(&ammunition, 0x88).unwrap()),
    }
}

fn content() -> Content {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap()
}

fn fixture() -> Value {
    serde_json::from_str(include_str!("../../../evidence/fixtures/player-guard.json")).unwrap()
}

fn state(content: &Content, snapshot: &Value) -> Session {
    let mut state = Session::new(content);
    apply(Command::ImportStarters, &mut state, content).unwrap();
    apply(
        Command::EnterMission {
            name: "The Thunder of Guns".into(),
        },
        &mut state,
        content,
    )
    .unwrap();
    state.warriors = warriors(snapshot);
    state.psi_memory = Some(bytes(&snapshot["memory"]));
    state.party_buffs = bytes(&snapshot["party_buffs"]);
    state.loaded_item_field32 = Some(snapshot["item_context"].as_i64().unwrap() as i16);
    state.attack_count = snapshot["count"].as_i64().unwrap() as i16;
    state.weapon_work = work(snapshot);
    state.attack_packet = Some(bytes(&snapshot["packet"]));
    state.attack_identity = Some(words(&snapshot["identity"]).try_into().unwrap());
    state.random = RandomState {
        seed: snapshot["seed"].as_u64().unwrap() as u32,
        draws: snapshot["draws"].as_u64().unwrap(),
    };
    state.combat = Some(Combat {
        battle: 85,
        map: TacticalMap {
            tiles: words(&snapshot["map"]),
            shift: (0, 0),
            edge_word: 0,
        },
        squad: SquadPlacement {
            positions: positions(&snapshot["party_positions"]),
            under_tiles: vec![1155; 6],
        },
        enemies: EnemySetup {
            records: enemies(snapshot),
            positions: positions(&snapshot["monster_positions"]),
        },
        order: words(&snapshot["order"]),
        cursor: snapshot["cursor"].as_u64().unwrap() as usize,
        round: snapshot["round"].as_i64().unwrap() as i16,
        round_scripts: Default::default(),
        death_queue: Default::default(),
        spell_fields: bytes(&snapshot["fields"]),
        field_used: bytes(&snapshot["used"]),
        attack_packet: Some(bytes(&snapshot["packet"])),
        attack_identity: Some(words(&snapshot["identity"]).try_into().unwrap()),
        weapon_work: state.weapon_work,
        unavailable: snapshot["unavailable"].as_i64().unwrap() as i8,
    });
    state.phase = Phase::Combat;
    state
}

fn assert_snapshot(state: &Session, snapshot: &Value) {
    let combat = state.combat.as_ref().unwrap();
    assert_party(state, snapshot);
    assert_eq!(combat.enemies.records, enemies(snapshot));
    assert_eq!(combat.map.tiles, words(&snapshot["map"]));
    assert_eq!(
        combat.squad.positions,
        positions(&snapshot["party_positions"])
    );
    assert_eq!(
        combat.enemies.positions,
        positions(&snapshot["monster_positions"])
    );
    assert_eq!(combat.order, words(&snapshot["order"]));
    assert_eq!(combat.cursor, snapshot["cursor"].as_u64().unwrap() as usize);
    assert_eq!(combat.round, snapshot["round"].as_i64().unwrap() as i16);
    assert_eq!(combat.spell_fields, bytes(&snapshot["fields"]));
    assert_eq!(combat.field_used, bytes(&snapshot["used"]));
    assert_eq!(combat.attack_packet, Some(bytes(&snapshot["packet"])));
    assert_eq!(
        combat.attack_identity.unwrap().to_vec(),
        words(&snapshot["identity"])
    );
    assert_eq!(
        combat.unavailable,
        snapshot["unavailable"].as_i64().unwrap() as i8
    );
    assert_eq!(combat.weapon_work, work(snapshot));
    assert_eq!(state.weapon_work, work(snapshot));
    assert_eq!(state.psi_memory, Some(bytes(&snapshot["memory"])));
    assert_eq!(state.party_buffs, bytes(&snapshot["party_buffs"]));
    assert_eq!(
        state.attack_count,
        snapshot["count"].as_i64().unwrap() as i16
    );
    assert_eq!(
        state.loaded_item_field32,
        Some(snapshot["item_context"].as_i64().unwrap() as i16)
    );
    assert_eq!(state.random.seed, snapshot["seed"].as_u64().unwrap() as u32);
    assert_eq!(state.random.draws, snapshot["draws"].as_u64().unwrap());
}

fn assert_party(state: &Session, snapshot: &Value) {
    let actual: Vec<_> = state
        .warriors
        .iter()
        .flat_map(|warrior| warrior.bytes.iter().copied())
        .collect();
    let expected = bytes(&snapshot["party"]);
    assert_eq!(actual.len(), expected.len());
    if let Some(offset) = actual.iter().zip(&expected).position(|(a, b)| a != b) {
        panic!(
            "party byte {offset:#x}: {} != {}",
            actual[offset], expected[offset]
        );
    }
}

#[test]
fn accepted_guard_matches_original_complete_command_and_getup() {
    let content = content();
    let fixture = fixture();
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 4);
    for case in fixture["cases"].as_array().unwrap() {
        let member = case["member"].as_u64().unwrap() as usize;
        let mut state = state(&content, &case["before"]);
        let before = state.clone();
        assert_snapshot(&state, &case["before"]);
        let events = if case["action_points"] == 2 {
            runtime::player_guard(&mut state, &content).unwrap()
        } else {
            // AP-zero is a controlled native prefix cut, outside public input admission.
            state.warriors[member].bytes[0x16..0x18].copy_from_slice(&1_i16.to_be_bytes());
            assert_party(&state, &case["guarded"]);
            let mut events = vec![Event::Sound { resource_id: 10121 }];
            events.extend(runtime::end_turn(&mut state, &content).unwrap());
            events
        };
        assert_snapshot(&state, &case["after"]);
        assert_party(&state, &case["guarded"]);
        assert_eq!(word(&state.warriors[member].bytes, 0x16).unwrap(), 1);
        assert_eq!(word(&state.warriors[member].bytes, 0x3c).unwrap(), 5);
        assert_eq!(
            word(&state.warriors[member].bytes, 0x40).unwrap() as i64,
            case["action_points"].as_i64().unwrap()
        );
        assert_eq!(
            case["guarded"]["ordered_calls"],
            json!([
                {"address": "133a18", "argument": 10121, "return": "1161b6"},
                {"address": "153720", "argument": 1, "return": "1161c0"},
                {"address": "125a3a", "argument": 0, "return": "1161de"}
            ])
        );
        assert!(matches!(
            events.first(),
            Some(Event::Sound { resource_id: 10121 })
        ));
        let sounds: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                Event::Sound { resource_id } => Some(*resource_id),
                _ => None,
            })
            .collect();
        assert_eq!(serde_json::to_value(sounds).unwrap(), case["sounds"]);
        assert_eq!(state.combat.as_ref().unwrap().active_body(), Some(10));
        assert_eq!(case["after"]["enemy_phase"], 1);
        let mut receipt = before.random.clone();
        for draw in case["rng"].as_array().unwrap() {
            assert_eq!(
                receipt.bound(draw["bound"].as_i64().unwrap() as i16),
                draw["result"].as_i64().unwrap() as i16
            );
        }
        assert_eq!(state.random, receipt);

        // Native presentation/selection buffers have no persistent Session equivalent.
        for field in [
            "selected",
            "powers",
            "markers",
            "mask",
            "last_amount",
            "monster_index",
            "active_member",
            "death_count",
            "item_context",
            "weapon_work",
            "ammo_work",
        ] {
            assert_eq!(
                case["before"][field], case["after"][field],
                "native {field}"
            );
        }
        assert!(
            bytes(&case["after"]["targets"])
                .iter()
                .all(|byte| *byte == 255)
        );
        assert_eq!(case["after"]["command"], 0);
        let mut expected = before;
        expected.bandage_ready = Some(1); // Getup 125a56 establishes the next interaction.
        for (warrior, native) in expected.warriors.iter_mut().zip(warriors(&case["after"])) {
            warrior.bytes = native.bytes;
        }
        expected.random = state.random.clone();
        let expected_combat = expected.combat.as_mut().unwrap();
        expected_combat.enemies.records = enemies(&case["after"]);
        expected_combat.cursor = case["after"]["cursor"].as_u64().unwrap() as usize;
        expected_combat.field_used = bytes(&case["after"]["used"]);
        expected_combat.attack_packet = Some(bytes(&case["after"]["packet"]));
        assert_eq!(state, expected);
    }
}

#[test]
fn ordinary_guard_rejects_controlled_zero_ap_without_changes() {
    let content = content();
    for case in fixture()["cases"].as_array().unwrap() {
        if case["action_points"] != 0 {
            continue;
        }
        let mut state = state(&content, &case["before"]);
        let before = state.clone();
        assert!(runtime::player_guard(&mut state, &content).is_err());
        assert_eq!(state, before);
    }
}

#[test]
fn guarded_player_reaction_and_enemy_counterattack_match_original_complete_callers() {
    let content = content();
    let guard = fixture();
    let reactions: Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/player-guard-reaction.json"
    ))
    .unwrap();
    assert_eq!(reactions["cases"].as_array().unwrap().len(), 2);
    for case in reactions["cases"].as_array().unwrap() {
        let member = case["member"].as_u64().unwrap() as usize;
        let source = case["source_guard_case"].as_u64().unwrap() as usize;
        for (field, native) in guard["cases"][source]["after"].as_object().unwrap() {
            if field != "ordered_calls" {
                assert!(
                    native == &case["restored"][field],
                    "restored native {field}"
                );
            }
        }
        let mut state = state(&content, &case["before"]);
        let before = state.clone();
        assert_snapshot(&state, &case["before"]);
        assert_eq!(word(&state.warriors[member].bytes, 0x16).unwrap(), 1);
        let attacks = &content.map(0).unwrap().combat.as_ref().unwrap().attacks;
        let events = runtime::check_neighbors_with_buffs(
            state.combat.as_mut().unwrap(),
            &mut state.warriors,
            &mut state.random,
            10,
            &content.items,
            attacks,
            &state.party_buffs,
        )
        .unwrap();
        assert_snapshot(&state, &case["after"]);
        assert_eq!(word(&state.warriors[member].bytes, 0x16).unwrap(), 0);
        assert_eq!(word(&state.warriors[member].bytes, 0xacc).unwrap(), 1);
        assert_eq!(word(&state.warriors[member].bytes, 0x3c).unwrap(), 5);
        assert_eq!(word(&state.warriors[member].bytes, 0x40).unwrap(), 2);
        assert_eq!(word(&state.warriors[member].bytes, 0x50).unwrap(), 27);
        let sounds: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                Event::Sound { resource_id } => Some(*resource_id),
                _ => None,
            })
            .collect();
        assert_eq!(serde_json::to_value(sounds).unwrap(), case["sounds"]);
        let mut receipt = before.random.clone();
        for draw in case["rng"].as_array().unwrap() {
            assert_eq!(
                receipt.bound(draw["bound"].as_i64().unwrap() as i16),
                draw["result"].as_i64().unwrap() as i16
            );
        }
        assert_eq!(state.random, receipt);
        assert_eq!(case["result"], 0);
        let calls = case["native_calls"].as_array().unwrap();
        let player = calls
            .iter()
            .find(|call| call["address"] == "100af0")
            .unwrap();
        assert_eq!(player["arguments"], json!([member, 10]));
        assert_eq!(player["return"], "11143c");
        assert_eq!(player["guard"], 0);
        let enemy = calls
            .iter()
            .find(|call| call["address"] == "101eb6")
            .unwrap();
        assert_eq!(enemy["arguments"], json!([10, member, 0]));
        assert_eq!(enemy["return"], "11171a");
        assert_eq!(enemy["guard"], 0);
        let setters: Vec<_> = calls
            .iter()
            .filter(|call| matches!(call["address"].as_str(), Some("130d38" | "130e54")))
            .collect();
        assert_eq!(setters.len(), 4);
        for call in setters {
            assert_eq!(call["arguments"], json!([0]));
        }
        for returned in case["attack_returns"].as_array().unwrap() {
            assert_eq!(returned["result"], 0);
            assert_eq!(returned["state"]["guard_word"], 0);
        }
        for field in [
            "selected",
            "powers",
            "markers",
            "mask",
            "targets",
            "last_amount",
            "monster_index",
            "active_member",
            "death_count",
            "command",
            "buttons",
            "health_context",
            "spell_semaphore",
        ] {
            assert!(
                case["before"][field] == case["after"][field],
                "unchanged native {field}"
            );
        }
        // The Session retains the three proved work WORDs; native raw buffers also capture the setters.
        for field in ["weapon_work", "ammo_work", "item_work"] {
            assert_eq!(bytes(&case["after"][field]), content.items[0]);
        }
        assert_eq!(
            case["after"]["work_projection"],
            json!({
                "sound": 0, "weapon_armor_filter": 0, "ammunition_armor_filter": 0, "armor_filter": false
            })
        );
        let mut expected = before;
        for (warrior, native) in expected.warriors.iter_mut().zip(warriors(&case["after"])) {
            warrior.bytes = native.bytes;
        }
        expected.random = state.random.clone();
        let expected_combat = expected.combat.as_mut().unwrap();
        expected_combat.enemies.records = enemies(&case["after"]);
        expected_combat.attack_packet = Some(bytes(&case["after"]["packet"]));
        expected_combat.weapon_work = work(&case["after"]);
        assert_eq!(state, expected);
    }
}

#[test]
fn public_guard_preserves_complete_native_state_and_resumes_from_save() {
    let content = content();
    for (index, case) in fixture()["cases"].as_array().unwrap().iter().enumerate() {
        if case["action_points"] != 2 {
            continue;
        }
        for waiting_child in [false, true] {
            let mut actual = state(&content, &case["before"]);
            if waiting_child {
                actual.combat_story = Some(runtime::DeathStory {
                    script: 1,
                    waiting_child: true,
                    ..Default::default()
                });
            }
            let path = std::env::temp_dir().join(format!(
                "centurions-public-guard-{}-{index}-{waiting_child}.json",
                std::process::id()
            ));
            centurions_core::save(&path, &actual, &content).unwrap();
            let mut resumed = centurions_core::load(&path, &content).unwrap();
            let mut expected = actual.clone();
            let expected_events = runtime::player_guard(&mut expected, &content).unwrap();
            let combat = expected.combat.as_ref().unwrap();
            expected.attack_packet = combat.attack_packet.clone();
            expected.attack_identity = combat.attack_identity;
            expected.weapon_work = combat.weapon_work;
            let mut health = centurions_core::story_health::context(
                &expected.warriors,
                expected.selected_warrior,
                expected.health_context.as_ref(),
            );
            health.unavailable = combat.unavailable;
            health.effect = bytes(&case["after"]["packet"])[18] as i8;
            expected.health_context = Some(health);
            let events = apply(Command::Guard, &mut actual, &content).unwrap();
            assert_eq!(events, expected_events);
            assert_eq!(actual, expected);
            assert_snapshot(&actual, &case["after"]);
            assert_eq!(
                apply(Command::Guard, &mut resumed, &content).unwrap(),
                events
            );
            assert_eq!(resumed, actual);
            centurions_core::save(&path, &actual, &content).unwrap();
            assert_eq!(centurions_core::load(&path, &content).unwrap(), actual);
            std::fs::remove_file(path).unwrap();
        }
    }
}

#[test]
fn public_guard_rejects_interrupted_or_unavailable_input_atomically() {
    let content = content();
    let fixture = fixture();
    let baseline = state(&content, &fixture["cases"][0]["before"]);
    let member = baseline.combat.as_ref().unwrap().active_body().unwrap() as usize;
    for variant in 0..6 {
        let mut state = baseline.clone();
        match variant {
            0 => state.warriors[member].bytes[0x40..0x42].copy_from_slice(&0_i16.to_be_bytes()),
            1 => state.warriors[member].bytes[0x50..0x52].copy_from_slice(&0_i16.to_be_bytes()),
            2 => state.warriors[member].bytes[0x9df] = 0,
            3 => {
                let combat = state.combat.as_mut().unwrap();
                combat.cursor = 2;
                assert!(combat.active_body().unwrap() >= 10);
            }
            4 => {
                state.exploration.as_mut().unwrap().presentation =
                    Some(centurions_core::Presentation::Loot)
            }
            _ => {
                state.combat_story = Some(runtime::DeathStory {
                    script: 1,
                    actor: member as i16,
                    ..Default::default()
                })
            }
        }
        let before = state.clone();
        assert!(
            apply(Command::Guard, &mut state, &content).is_err(),
            "variant {variant}"
        );
        assert_eq!(state, before, "variant {variant}");
    }
    let mut state = Session::new(&content);
    let before = state.clone();
    assert!(apply(Command::Guard, &mut state, &content).is_err());
    assert_eq!(state, before);
}

#[test]
fn guard_rolls_back_arming_sound_and_initiative_on_later_round_failure() {
    let content = content();
    let fixture = fixture();
    let mut state = state(&content, &fixture["cases"][0]["before"]);
    let member = state.combat.as_ref().unwrap().active_body().unwrap();
    let combat = state.combat.as_mut().unwrap();
    combat.cursor = 109;
    combat.order[109] = member;
    state.warriors[0].bytes[0x8a..0x8c].copy_from_slice(&1_i16.to_be_bytes());
    let before = state.clone();
    assert!(matches!(
        apply(Command::Guard, &mut state, &content),
        Err(centurions_core::RuleError::Unsupported {
            action: "combat.round",
            ..
        })
    ));
    assert_eq!(state, before);
}
