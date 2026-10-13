use crate::prelude::*;

fn positions(value: &Value) -> Vec<(i16, i16)> {
    words(value)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| (pair[0], pair[1]))
        .collect()
}
use crate::native_common::snapshot_party as warriors;

use crate::native_common::snapshot_enemies as enemies;

fn work(snapshot: &Value) -> WeaponWork {
    let weapon = snapshot.bytes("weapon_work");
    let ammunition = snapshot.bytes("ammo_work");
    WeaponWork {
        sound: Some(word(&weapon, 0x0e).unwrap()),
        weapon_armor_filter: Some(word(&weapon, 0x88).unwrap()),
        ammunition_armor_filter: Some(word(&ammunition, 0x88).unwrap()),
    }
}

fn fixture() -> Value {
    crate::support::fixture("player-guard.json")
}

fn state(content: &Content, snapshot: &Value) -> Session {
    let mut state = crate::support::mission_session(content);
    state.warriors = warriors(snapshot);
    state.psi_memory = Some(snapshot.bytes("memory"));
    state.party_buffs = snapshot.bytes("party_buffs");
    state.loaded_item_field32 = Some(snapshot.n::<i16>("item_context"));
    state.attack_count = snapshot.n::<i16>("count");
    state.weapon_work = work(snapshot);
    state.attack_packet = Some(snapshot.bytes("packet"));
    state.attack_identity = Some(snapshot.words("identity").try_into().unwrap());
    state.random = RandomState {
        seed: snapshot.n::<u32>("seed"),
        draws: snapshot.n::<u64>("draws"),
    };
    state.combat = Some(Combat {
        battle: 85,
        map: TacticalMap {
            tiles: snapshot.words("map"),
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
        order: snapshot.words("order"),
        cursor: snapshot.n::<usize>("cursor"),
        round: snapshot.n::<i16>("round"),
        spell_fields: snapshot.bytes("fields"),
        field_used: snapshot.bytes("used"),
        attack_packet: Some(snapshot.bytes("packet")),
        attack_identity: Some(snapshot.words("identity").try_into().unwrap()),
        weapon_work: state.weapon_work,
        unavailable: snapshot.n::<i8>("unavailable"),
        ..crate::support::battle()
    });
    state.phase = Phase::Combat;
    state
}

fn assert_snapshot(state: &Session, snapshot: &Value) {
    let combat = state.combat.as_ref().unwrap();
    assert_party(state, snapshot);
    assert_eq!(combat.enemies.records, enemies(snapshot));
    assert_eq!(combat.map.tiles, snapshot.words("map"));
    assert_eq!(
        combat.squad.positions,
        positions(&snapshot["party_positions"])
    );
    assert_eq!(
        combat.enemies.positions,
        positions(&snapshot["monster_positions"])
    );
    crate::support::native_expect! {
    words combat.order => &snapshot["order"], "binary_player_guard";
    scalar combat.cursor => &snapshot["cursor"], "binary_player_guard";
    scalar combat.round => &snapshot["round"], "binary_player_guard";
    bytes combat.spell_fields => &snapshot["fields"], "binary_player_guard";
    bytes combat.field_used => &snapshot["used"], "binary_player_guard";
    }
    assert_eq!(combat.attack_packet, Some(snapshot.bytes("packet")));
    crate::support::native_expect! {
    words combat.attack_identity.unwrap().to_vec() => &snapshot["identity"], "binary_player_guard";
    scalar combat.unavailable => &snapshot["unavailable"], "binary_player_guard";
    }
    assert_eq!(combat.weapon_work, work(snapshot));
    assert_eq!(state.weapon_work, work(snapshot));
    assert_eq!(state.psi_memory, Some(snapshot.bytes("memory")));
    crate::support::native_expect! {
    bytes state.party_buffs => &snapshot["party_buffs"], "binary_player_guard";
    scalar state.attack_count => &snapshot["count"], "binary_player_guard";
    }
    assert_eq!(
        state.loaded_item_field32,
        Some(snapshot.n::<i16>("item_context"))
    );
    crate::support::native_expect! {
    scalar state.random.seed => &snapshot["seed"], "binary_player_guard";
    scalar state.random.draws => &snapshot["draws"], "binary_player_guard";
    }
}

fn assert_party(state: &Session, snapshot: &Value) {
    let actual: Vec<_> = crate::support::record_bytes(&state.warriors);
    let expected = snapshot.bytes("party");
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

    for case in fixture.array("cases") {
        let member = case.n::<usize>("member");
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
        crate::support::expect(
            word(&state.warriors[member].bytes, 0x40).unwrap() as i64,
            &case["action_points"],
        );

        assert!(matches!(
            events.first(),
            Some(Event::Sound { resource_id: 10121 })
        ));
        let sounds: Vec<_> = crate::support::event_sounds(&events);
        assert_eq!(serde_json::to_value(sounds).unwrap(), case["sounds"]);
        assert_eq!(state.combat.as_ref().unwrap().active_body(), Some(10));

        let mut receipt = before.random.clone();
        for draw in case.array("rng") {
            crate::support::expect(receipt.bound(draw.n::<i16>("bound")), &draw["result"]);
        }
        assert_eq!(state.random, receipt);

        assert!(
            case["after"]
                .bytes("targets")
                .iter()
                .all(|byte| *byte == 255)
        );

        let mut expected = before;
        expected.bandage_ready = Some(1); // Getup 125a56 establishes the next interaction.
        for (warrior, native) in expected.warriors.iter_mut().zip(warriors(&case["after"])) {
            warrior.bytes = native.bytes;
        }
        expected.random = state.random.clone();
        let expected_combat = expected.combat.as_mut().unwrap();
        expected_combat.enemies.records = enemies(&case["after"]);
        expected_combat.cursor = case["after"].n::<usize>("cursor");
        expected_combat.field_used = case["after"].bytes("used");
        expected_combat.attack_packet = Some(case["after"].bytes("packet"));
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
    let reactions: Value = crate::support::fixture("player-guard-reaction.json");

    for case in reactions.array("cases") {
        let member = case.n::<usize>("member");
        let source = case.n::<usize>("source_guard_case");
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
        let sounds: Vec<_> = crate::support::event_sounds(&events);
        assert_eq!(serde_json::to_value(sounds).unwrap(), case["sounds"]);
        let mut receipt = before.random.clone();
        for draw in case.array("rng") {
            crate::support::expect(receipt.bound(draw.n::<i16>("bound")), &draw["result"]);
        }
        assert_eq!(state.random, receipt);

        let calls = case.array("native_calls");
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
        for returned in case.array("attack_returns") {
            assert_eq!(returned["result"], 0);
            assert_eq!(returned["state"]["guard_word"], 0);
        }

        // The Session retains the three proved work WORDs; native raw buffers also capture the setters.
        for field in ["weapon_work", "ammo_work", "item_work"] {
            assert_eq!(bytes(&case["after"][field]), content.items[0]);
        }

        let mut expected = before;
        for (warrior, native) in expected.warriors.iter_mut().zip(warriors(&case["after"])) {
            warrior.bytes = native.bytes;
        }
        expected.random = state.random.clone();
        let expected_combat = expected.combat.as_mut().unwrap();
        expected_combat.enemies.records = enemies(&case["after"]);
        expected_combat.attack_packet = Some(case["after"].bytes("packet"));
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
            health.effect = case["after"].bytes("packet")[18] as i8;
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
