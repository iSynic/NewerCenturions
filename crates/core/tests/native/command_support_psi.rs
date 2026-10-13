use crate::native_common::decoded_positions as positions;
use crate::prelude::*;

fn fixture() -> Value {
    crate::support::fixture("player-support-psi.json")
}

fn outdoor(content: &Content, case: &Value) -> Session {
    let mut state = crate::support::mission_session(content);
    state.warriors = crate::support::warriors(&case["before"]["party"]);
    state.psi_memory = Some(case["before"].bytes("memory"));
    state.party_buffs = case["before"].bytes("party_buffs");
    state.attack_packet = Some(case["before"].bytes("packet"));
    let identity = case["before"].bytes("identity");
    state.attack_identity = Some(std::array::from_fn(|i| {
        i16::from_be_bytes(identity[i * 2..i * 2 + 2].try_into().unwrap())
    }));
    state.attack_count = case["before"].n::<i16>("count");
    state.random = RandomState::new(case.n::<u32>("seed_before"));
    state
}

#[test]
fn outdoor_shield_matches_complete_native_records_and_exact_session_across_reload() {
    let content = content();
    let fixture = fixture();
    for case in fixture
        .array("cases")
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
            power: case.n::<i16>("power"),
        };
        let events = apply(command.clone(), &mut state, &content).unwrap();
        assert_eq!(apply(command, &mut resumed, &content).unwrap(), events);
        assert_eq!(resumed, state);
        let expected = &case["after"];
        let mut expected_session = before;
        for (warrior, record) in expected_session
            .warriors
            .iter_mut()
            .zip(expected.bytes("party").as_chunks::<3138>().0)
        {
            warrior.bytes = record.to_vec();
        }
        expected_session.psi_memory = Some(expected.bytes("memory"));
        expected_session.party_buffs = expected.bytes("party_buffs");
        expected_session.attack_packet = Some(expected.bytes("packet"));
        let identity = expected.bytes("identity");
        expected_session.attack_identity = Some(std::array::from_fn(|i| {
            i16::from_be_bytes(identity[i * 2..i * 2 + 2].try_into().unwrap())
        }));
        expected_session.attack_count = expected.n::<i16>("count");
        expected_session.psi_menu_choices[5] = Some(centurions_core::PsiMenuChoice {
            row: centurions_content::word(&identity, 2).unwrap() as u8,
            column: centurions_content::word(&identity, 4).unwrap() as u8,
            power: expected_session.attack_count,
        });
        expected_session.random.seed = expected.n::<u32>("seed");
        expected_session.random.draws = expected.n::<u64>("draws");
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
        let native_events: Vec<_> = case
            .array("sounds")
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
    let case = fixture
        .array("cases")
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

fn combat_state(content: &Content, case: &Value) -> Session {
    let mut state = outdoor(content, case);
    let foundation: Value = crate::support::fixture("player-psi.json");
    let mut combat: centurions_core::combat::Combat =
        serde_json::from_value(foundation["route_foundation"]["combat"].clone()).unwrap();
    combat.map.shift = (0, 0);
    combat.map.tiles = case["before"].words("map");
    combat.squad.positions = positions(&case["before"]["party_positions"]);
    combat.squad.under_tiles = vec![1155; 6];
    combat.enemies.records = crate::support::monsters(&case["before"].bytes("records"));
    combat.enemies.positions = positions(&case["before"]["monster_positions"]);
    combat.order = case["before"].words("order");
    combat.cursor = case["before"].n::<usize>("cursor");
    combat.round = case["before"].n::<i16>("round");
    combat.unavailable = case["before"].n::<i8>("unavailable");
    combat.spell_fields = case["before"].bytes("fields");
    combat.field_used = case["before"].bytes("used");
    combat.attack_packet = state.attack_packet.clone();
    combat.attack_identity = state.attack_identity;
    combat.weapon_work = centurions_core::combat::weapon_work::WeaponWork::zero();
    state.weapon_work = combat.weapon_work;
    state.combat = Some(combat);
    state.phase = centurions_core::Phase::Combat;
    state.loaded_item_field32 = Some(case["before"].n::<i16>("item_context"));
    state
}

#[test]
fn combat_support_and_automatic_initiative_match_complete_native_suffix_and_resume() {
    let content = content();
    let fixture: Value = crate::support::fixture("support-psi-advance.json");
    for case in fixture.array("cases") {
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
        let spell = case.n::<i16>("spell_id");
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
            .zip(expected.bytes("party").as_chunks::<3138>().0)
        {
            warrior.bytes = record.to_vec();
        }
        let arena = expected_session.combat.as_mut().unwrap();
        for (monster, record) in arena
            .enemies
            .records
            .iter_mut()
            .zip(expected.bytes("records").as_chunks::<288>().0)
        {
            monster.bytes = record.to_vec();
        }
        arena.map.tiles = expected.words("map");
        arena.squad.positions = positions(&expected["party_positions"]);
        arena.enemies.positions = positions(&expected["monster_positions"]);
        arena.order = expected.words("order");
        arena.cursor = expected.n::<usize>("cursor");
        arena.round = expected.n::<i16>("round");
        arena.unavailable = expected.n::<i8>("unavailable");
        arena.spell_fields = expected.bytes("fields");
        arena.field_used = expected.bytes("used");
        arena.attack_packet = Some(expected.bytes("packet"));
        arena.attack_identity = Some(expected.words("identity").try_into().unwrap());
        expected_session.attack_packet = arena.attack_packet.clone();
        expected_session.attack_identity = arena.attack_identity;
        expected_session.attack_count = expected.n::<i16>("count");
        let identity = expected.words("identity");
        let actor = case["before"].n::<usize>("active_member");
        expected_session.psi_menu_choices[actor] = Some(centurions_core::PsiMenuChoice {
            row: identity[1] as u8,
            column: identity[2] as u8,
            power: expected_session.attack_count,
        });
        expected_session.loaded_item_field32 = Some(expected.n::<i16>("item_context"));
        expected_session.psi_memory = Some(expected.bytes("memory"));
        expected_session.party_buffs = expected.bytes("party_buffs");
        let mut health = centurions_core::story_health::context(
            &expected_session.warriors,
            expected_session.selected_warrior,
            expected_session.health_context.as_ref(),
        );
        health.unavailable = arena.unavailable;
        health.effect = expected.bytes("packet")[18] as i8;
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
        expected_session.random.seed = expected.n::<u32>("seed");
        expected_session.random.draws = expected.n::<u64>("draws");
        let actual = serde_json::to_value(&state).unwrap();
        let expected_json = serde_json::to_value(&expected_session).unwrap();
        for (key, expected_value) in expected_json.as_object().unwrap() {
            assert!(
                actual[key] == *expected_value,
                "unexpected Session field {key}, spell {spell}, seed {}",
                case["seed_before"]
            );
        }
        let sounds: Vec<_> = crate::support::event_sounds(&events);
        crate::support::expect_array(&(sounds), &case["sounds"]);
        assert!(matches!(
            events.last(),
            Some(Event::TurnStarted { body: 10, round: 1 })
        ));
        save(&path, &state, &content).unwrap();
        assert!(load(&path, &content).unwrap() == state);
        std::fs::remove_file(path).unwrap();
    }
}
