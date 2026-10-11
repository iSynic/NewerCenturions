use centurions_content::{Content, Warrior};
use centurions_core::{Command, Event, RandomState, Session, apply, load, save};
use serde_json::Value;

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
        .map(|word| i16::from_be_bytes(*word))
        .collect()
}

fn positions(value: &Value) -> Vec<(i16, i16)> {
    words(value)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| (p[0], p[1]))
        .collect()
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
    serde_json::from_str(include_str!(
        "../../../evidence/fixtures/player-support-psi.json"
    ))
    .unwrap()
}

fn outdoor(content: &Content, case: &Value) -> Session {
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
    state.warriors = bytes(&case["before"]["party"])
        .as_chunks::<3138>()
        .0
        .iter()
        .map(|record| Warrior::read(record).unwrap())
        .collect();
    state.psi_memory = Some(bytes(&case["before"]["memory"]));
    state.party_buffs = bytes(&case["before"]["party_buffs"]);
    state.attack_packet = Some(bytes(&case["before"]["packet"]));
    let identity = bytes(&case["before"]["identity"]);
    state.attack_identity = Some(std::array::from_fn(|i| {
        i16::from_be_bytes(identity[i * 2..i * 2 + 2].try_into().unwrap())
    }));
    state.attack_count = case["before"]["count"].as_i64().unwrap() as i16;
    state.random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
    state
}

#[test]
fn outdoor_shield_matches_complete_native_records_and_exact_session_across_reload() {
    let content = content();
    let fixture = fixture();
    for case in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["spell_id"] == 1103)
    {
        let mut state = outdoor(&content, case);
        let before = state.clone();
        let path = std::env::temp_dir().join(format!(
            "centurions-support-{}-{}-{}-{}.json",
            std::process::id(),
            case["seed_before"],
            case["power"],
            case["variant"]
        ));
        save(&path, &state, &content).unwrap();
        let mut resumed = load(&path, &content).unwrap();
        let command = Command::CastPartyPsi {
            warrior: 5,
            spell: 1103,
            power: case["power"].as_i64().unwrap() as i16,
        };
        let events = apply(command.clone(), &mut state, &content).unwrap();
        assert_eq!(apply(command, &mut resumed, &content).unwrap(), events);
        assert_eq!(resumed, state);
        let expected = &case["after"];
        let mut expected_session = before;
        for (warrior, record) in expected_session
            .warriors
            .iter_mut()
            .zip(bytes(&expected["party"]).as_chunks::<3138>().0)
        {
            warrior.bytes = record.to_vec();
        }
        expected_session.psi_memory = Some(bytes(&expected["memory"]));
        expected_session.party_buffs = bytes(&expected["party_buffs"]);
        expected_session.attack_packet = Some(bytes(&expected["packet"]));
        let identity = bytes(&expected["identity"]);
        expected_session.attack_identity = Some(std::array::from_fn(|i| {
            i16::from_be_bytes(identity[i * 2..i * 2 + 2].try_into().unwrap())
        }));
        expected_session.attack_count = expected["count"].as_i64().unwrap() as i16;
        expected_session.psi_menu_choices[5] = Some(centurions_core::PsiMenuChoice {
            row: centurions_content::word(&identity, 2).unwrap() as u8,
            column: centurions_content::word(&identity, 4).unwrap() as u8,
            power: expected_session.attack_count,
        });
        expected_session.random.seed = expected["seed"].as_u64().unwrap() as u32;
        expected_session.random.draws = expected["draws"].as_u64().unwrap();
        let actual = serde_json::to_value(&state).unwrap();
        let expected_json = serde_json::to_value(&expected_session).unwrap();
        for (key, expected_value) in expected_json.as_object().unwrap() {
            assert!(
                actual[key] == *expected_value,
                "unexpected Session field {key}, seed {} power {} variant {}",
                case["seed_before"],
                case["power"],
                case["variant"]
            );
        }
        let native_events: Vec<_> = case["sounds"]
            .as_array()
            .unwrap()
            .iter()
            .map(|sound| Event::Sound {
                resource_id: sound.as_i64().unwrap() as i16,
            })
            .collect();
        assert_eq!(events, native_events);
        save(&path, &state, &content).unwrap();
        assert_eq!(load(&path, &content).unwrap(), state);
        std::fs::remove_file(path).unwrap();
    }
}

#[test]
fn outdoor_invalid_cost_conditions_and_pending_story_roll_back_the_whole_session() {
    let content = content();
    let fixture = fixture();
    let case = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["spell_id"] == 1103 && c["variant"] == 0)
        .unwrap();
    for command in [
        Command::CastPartyPsi {
            warrior: 6,
            spell: 1103,
            power: 1,
        },
        Command::CastPartyPsi {
            warrior: 0,
            spell: 1103,
            power: 1,
        },
        Command::CastPartyPsi {
            warrior: 5,
            spell: 1103,
            power: 3,
        },
        Command::CastPartyPsi {
            warrior: 5,
            spell: 1101,
            power: 1,
        },
        Command::CastPartyPsi {
            warrior: 5,
            spell: 2101,
            power: 1,
        },
    ] {
        let mut state = outdoor(&content, case);
        let before = state.clone();
        assert!(apply(command.clone(), &mut state, &content).is_err());
        assert_eq!(state, before, "{command:?}");
    }
    let command = Command::CastPartyPsi {
        warrior: 5,
        spell: 1103,
        power: 1,
    };
    for offset in [0x50, 0x58, 0x60, 0x68, 0x90, 0x98, 0xac] {
        let mut state = outdoor(&content, case);
        let value = if matches!(offset, 0x50 | 0x58) {
            0_i16
        } else {
            1_i16
        };
        state.warriors[5].bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
        let before = state.clone();
        assert!(
            apply(command.clone(), &mut state, &content).is_err(),
            "condition {offset:x}"
        );
        assert_eq!(state, before);
    }
    let mut state = outdoor(&content, case);
    state.exploration.as_mut().unwrap().presentation = Some(centurions_core::Presentation::Loot);
    let before = state.clone();
    assert!(apply(command, &mut state, &content).is_err());
    assert_eq!(state, before);
}

#[test]
fn old_saves_default_reconstructable_party_buffs_and_reject_malformed_buffers() {
    let content = content();
    let fixture = fixture();
    let case = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["spell_id"] == 1103 && c["variant"] == 0)
        .unwrap();
    let state = outdoor(&content, case);
    let path = std::env::temp_dir().join(format!(
        "centurions-party-buff-migration-{}.json",
        std::process::id()
    ));
    for version in [1, 2, 3] {
        let mut saved = serde_json::json!({"schema_version": version, "session": state});
        saved["session"]
            .as_object_mut()
            .unwrap()
            .remove("party_buffs");
        std::fs::write(&path, serde_json::to_vec(&saved).unwrap()).unwrap();
        assert_eq!(load(&path, &content).unwrap().party_buffs, vec![0; 20]);
    }
    let mut saved = serde_json::json!({"schema_version": 3, "session": state});
    saved["session"]["party_buffs"] = serde_json::json!([0, 0]);
    std::fs::write(&path, serde_json::to_vec(&saved).unwrap()).unwrap();
    assert!(load(&path, &content).is_err());
    std::fs::remove_file(path).unwrap();
}

fn combat_state(content: &Content, case: &Value) -> Session {
    let mut state = outdoor(content, case);
    let foundation: Value =
        serde_json::from_str(include_str!("../../../evidence/fixtures/player-psi.json")).unwrap();
    let mut combat: centurions_core::combat::Combat =
        serde_json::from_value(foundation["route_foundation"]["combat"].clone()).unwrap();
    combat.map.shift = (0, 0);
    combat.map.tiles = words(&case["before"]["map"]);
    combat.squad.positions = positions(&case["before"]["party_positions"]);
    combat.squad.under_tiles = vec![1155; 6];
    combat.enemies.records = bytes(&case["before"]["records"])
        .as_chunks::<288>()
        .0
        .iter()
        .map(|record| centurions_content::combat::MonsterRecord {
            bytes: record.to_vec(),
        })
        .collect();
    combat.enemies.positions = positions(&case["before"]["monster_positions"]);
    combat.order = words(&case["before"]["order"]);
    combat.cursor = case["before"]["cursor"].as_u64().unwrap() as usize;
    combat.round = case["before"]["round"].as_i64().unwrap() as i16;
    combat.unavailable = case["before"]["unavailable"].as_i64().unwrap() as i8;
    combat.spell_fields = bytes(&case["before"]["fields"]);
    combat.field_used = bytes(&case["before"]["used"]);
    combat.attack_packet = state.attack_packet.clone();
    combat.attack_identity = state.attack_identity;
    combat.weapon_work = centurions_core::combat::weapon_work::WeaponWork::zero();
    state.weapon_work = combat.weapon_work;
    state.combat = Some(combat);
    state.phase = centurions_core::Phase::Combat;
    state.loaded_item_field32 = Some(case["before"]["item_context"].as_i64().unwrap() as i16);
    state
}

#[test]
fn combat_support_and_automatic_initiative_match_complete_native_suffix_and_resume() {
    let content = content();
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/support-psi-advance.json"
    ))
    .unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let mut state = combat_state(&content, case);
        let before = state.clone();
        let path = std::env::temp_dir().join(format!(
            "centurions-support-advance-{}-{}-{}.json",
            std::process::id(),
            case["spell_id"],
            case["seed_before"]
        ));
        save(&path, &state, &content).unwrap();
        let mut resumed = load(&path, &content).unwrap();
        let spell = case["spell_id"].as_i64().unwrap() as i16;
        let command = Command::CastPsi {
            spell,
            power: 1,
            target: Some((20, 20)),
        };
        let events = apply(command.clone(), &mut state, &content).unwrap();
        assert_eq!(apply(command, &mut resumed, &content).unwrap(), events);
        assert!(state == resumed, "save/resume state differs");
        let expected = &case["after"];
        let mut expected_session = before;
        for (warrior, record) in expected_session
            .warriors
            .iter_mut()
            .zip(bytes(&expected["party"]).as_chunks::<3138>().0)
        {
            warrior.bytes = record.to_vec();
        }
        let arena = expected_session.combat.as_mut().unwrap();
        for (monster, record) in arena
            .enemies
            .records
            .iter_mut()
            .zip(bytes(&expected["records"]).as_chunks::<288>().0)
        {
            monster.bytes = record.to_vec();
        }
        arena.map.tiles = words(&expected["map"]);
        arena.squad.positions = positions(&expected["party_positions"]);
        arena.enemies.positions = positions(&expected["monster_positions"]);
        arena.order = words(&expected["order"]);
        arena.cursor = expected["cursor"].as_u64().unwrap() as usize;
        arena.round = expected["round"].as_i64().unwrap() as i16;
        arena.unavailable = expected["unavailable"].as_i64().unwrap() as i8;
        arena.spell_fields = bytes(&expected["fields"]);
        arena.field_used = bytes(&expected["used"]);
        arena.attack_packet = Some(bytes(&expected["packet"]));
        arena.attack_identity = Some(words(&expected["identity"]).try_into().unwrap());
        expected_session.attack_packet = arena.attack_packet.clone();
        expected_session.attack_identity = arena.attack_identity;
        expected_session.attack_count = expected["count"].as_i64().unwrap() as i16;
        let identity = words(&expected["identity"]);
        let actor = case["before"]["active_member"].as_u64().unwrap() as usize;
        expected_session.psi_menu_choices[actor] = Some(centurions_core::PsiMenuChoice {
            row: identity[1] as u8,
            column: identity[2] as u8,
            power: expected_session.attack_count,
        });
        expected_session.loaded_item_field32 =
            Some(expected["item_context"].as_i64().unwrap() as i16);
        expected_session.psi_memory = Some(bytes(&expected["memory"]));
        expected_session.party_buffs = bytes(&expected["party_buffs"]);
        let mut health = centurions_core::story_health::context(
            &expected_session.warriors,
            expected_session.selected_warrior,
            expected_session.health_context.as_ref(),
        );
        health.unavailable = arena.unavailable;
        health.effect = bytes(&expected["packet"])[18] as i8;
        assert_eq!(state.health_context.as_ref().unwrap().effect, health.effect);
        assert_eq!(
            state.attack_packet.as_ref().unwrap()[18] as i8,
            health.effect
        );
        assert_eq!(
            state
                .combat
                .as_ref()
                .unwrap()
                .attack_packet
                .as_ref()
                .unwrap()[18] as i8,
            health.effect
        );
        expected_session.health_context = Some(health);
        expected_session.bandage_ready = Some(1);
        expected_session.random.seed = expected["seed"].as_u64().unwrap() as u32;
        expected_session.random.draws = expected["draws"].as_u64().unwrap();
        let actual = serde_json::to_value(&state).unwrap();
        let expected_json = serde_json::to_value(&expected_session).unwrap();
        for (key, expected_value) in expected_json.as_object().unwrap() {
            assert!(
                actual[key] == *expected_value,
                "unexpected Session field {key}, spell {spell}, seed {}",
                case["seed_before"]
            );
        }
        let sounds: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                Event::Sound { resource_id } => Some(*resource_id),
                _ => None,
            })
            .collect();
        assert_eq!(
            sounds,
            case["sounds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_i64().unwrap() as i16)
                .collect::<Vec<_>>()
        );
        assert!(matches!(
            events.last(),
            Some(Event::TurnStarted { body: 10, round: 1 })
        ));
        save(&path, &state, &content).unwrap();
        assert!(load(&path, &content).unwrap() == state);
        std::fs::remove_file(path).unwrap();
    }
}

#[test]
fn unsupported_later_initiative_and_support_targets_are_atomic() {
    let content = content();
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/support-psi-advance.json"
    ))
    .unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let mut state = combat_state(&content, case);
        state.combat.as_mut().unwrap().enemies.records[0].bytes[0x5e..0x60]
            .copy_from_slice(&32767_i16.to_be_bytes());
        state.combat.as_mut().unwrap().enemies.records[0].bytes[0x5c..0x5e]
            .copy_from_slice(&100_i16.to_be_bytes());
        let before = state.clone();
        let command = Command::CastPsi {
            spell: case["spell_id"].as_i64().unwrap() as i16,
            power: 1,
            target: Some((20, 20)),
        };
        assert!(apply(command, &mut state, &content).is_err());
        assert!(
            state == before,
            "later initiative error changed the session"
        );
    }
    let case = &fixture["cases"][0];
    for command in [
        Command::CastPsi {
            spell: 1101,
            power: 1,
            target: None,
        },
        Command::CastPsi {
            spell: 1101,
            power: 1,
            target: Some((40, 40)),
        },
        Command::CastPsi {
            spell: 1305,
            power: 1,
            target: Some((40, 40)),
        },
        Command::CastPsi {
            spell: 1103,
            power: 1,
            target: None,
        },
        Command::CastPartyPsi {
            warrior: 5,
            spell: 1103,
            power: 1,
        },
    ] {
        let mut state = combat_state(&content, case);
        let before = state.clone();
        assert!(apply(command.clone(), &mut state, &content).is_err());
        assert!(state == before, "{command:?} changed session");
    }
}
