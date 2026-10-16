use crate::prelude::*;

use crate::native_common::snapshot_party as warriors;
use crate::support::positions;

use crate::native_common::snapshot_enemies as enemies;

use crate::native_common::psi_work as work;
fn state(content: &Content, snapshot: &Value) -> Session {
    let mut state = crate::native_common::psi_snapshot(content, snapshot);
    state.combat.as_mut().unwrap().round_scripts.counter = snapshot.n("round");
    state
}

fn assert_snapshot(state: &Session, snapshot: &Value) {
    let combat = state.combat.as_ref().unwrap();
    assert_eq!(combat.round_scripts.counter, snapshot.n::<i8>("round"));
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
    crate::support::compare_party(&state.warriors, snapshot);
}

#[test]
fn inactive_turns_match_original_recovery_retreat_and_saved_command() {
    let content = content();
    for case in crate::support::fixture("inactive-rounds.json").array("cases") {
        let mut state = state(&content, &case["before"]);
        let mut resumed = crate::support::checkpoint(&state, &content, "inactive-turns");
        let mut events = replay_command(Command::EndTurn, &mut state, &mut resumed, &content);
        if case.n::<i16>("stun") == 3 {
            crate::support::reject_command(&state, &content, Command::EndTurn);
        }
        for _ in 0..5 {
            if !state.combat.as_ref().is_some_and(|c| c.selection_pending) {
                break;
            }
            resumed = crate::support::checkpoint(&state, &content, "inactive-pending");
            let next = replay_command(
                Command::ResumeCombatSelection,
                &mut state,
                &mut resumed,
                &content,
            );
            events.extend(next);
        }
        assert_eq!(
            crate::support::event_sounds(&events),
            case.n::<Vec<i16>>("sounds")
        );
        if case.n::<u8>("retreated") == 0 {
            assert_snapshot(&state, &case["after"]);
            assert_eq!(
                state.combat.as_ref().unwrap().inactive_rounds,
                case.n::<i16>("inactive_rounds")
            );
            resumed = crate::support::checkpoint(&state, &content, "inactive-recovered");
            assert_eq!(state, resumed);
        } else {
            assert!(state.combat.is_none());
            assert_eq!(state.phase, Phase::Exploration);
            assert_party(&state, &case["after"]);
            assert_eq!(
                (state.random.seed, state.random.draws),
                (case["after"].n("seed"), case["after"].n("draws"))
            );
        }
    }
}

#[test]
fn guarded_player_reaction_and_enemy_counterattack_match_original_complete_callers() {
    let content = content();
    let guard = crate::support::fixture("player-guard.json");
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
        crate::support::replay_rng(&mut receipt, case.array("rng"));
        assert_eq!(state.random, receipt);

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
    let fixture = crate::support::fixture("player-guard.json");
    for (index, case) in fixture.array("cases").iter().enumerate() {
        let public = case["action_points"] == 2;
        for waiting_child in [false, true]
            .into_iter()
            .take(if index == 0 { 2 } else { 1 })
        {
            let mut actual = state(&content, &case["before"]);
            if waiting_child {
                actual.combat_story = Some(runtime::DeathStory {
                    script: 1,
                    waiting_child: true,
                    ..Default::default()
                });
            }
            let mut expected = state(&content, &case["after"]);
            for (warrior, original) in expected.warriors.iter_mut().zip(&actual.warriors) {
                warrior.original_sha256 = original.original_sha256.clone();
            }
            expected.combat_story = actual.combat_story.clone();
            expected.bandage_ready = Some(1);
            let events = if public {
                let mut resumed = crate::support::checkpoint(&actual, &content, "guard-before");
                let events = replay_command(Command::Guard, &mut actual, &mut resumed, &content);
                let combat = expected.combat.as_ref().unwrap();
                let mut health = centurions_core::story_health::context(
                    &expected.warriors,
                    expected.selected_warrior,
                    None,
                );
                health.unavailable = combat.unavailable;
                health.effect = case["after"].bytes("packet")[18] as i8;
                expected.health_context = Some(health);
                crate::support::checkpoint(&actual, &content, "guard-after");
                events
            } else {
                // Native AP-zero prefix starts after public admission; retain its original suffix.
                crate::support::reject_command(&actual, &content, Command::Guard);
                expected.attack_packet = actual.attack_packet.clone();
                let member = case.n::<usize>("member");
                actual.warriors[member].bytes[0x16..0x18].copy_from_slice(&1_i16.to_be_bytes());
                assert_party(&actual, &case["guarded"]);
                let mut events = vec![Event::Sound { resource_id: 10121 }];
                events.extend(runtime::end_turn(&mut actual, &content).unwrap());
                events
            };
            crate::support::same_session(&actual, &expected);
            let mut expected_events: Vec<_> = case
                .n::<Vec<i16>>("sounds")
                .into_iter()
                .map(|resource_id| Event::Sound { resource_id })
                .collect();
            expected_events.push(Event::TurnStarted {
                body: expected.combat.as_ref().unwrap().active_body().unwrap(),
                round: expected.combat.as_ref().unwrap().round,
            });
            assert_eq!(events, expected_events);
        }
    }
}

#[test]
fn public_guard_rejects_interrupted_or_unavailable_input_atomically() {
    let content = content();
    let fixture = crate::support::fixture("player-guard.json");
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
    let fixture = crate::support::fixture("player-guard.json");
    let mut state = state(&content, &fixture["cases"][0]["before"]);
    let member = state.combat.as_ref().unwrap().active_body().unwrap();
    let combat = state.combat.as_mut().unwrap();
    combat.cursor = 109;
    combat.order[109] = member;
    // A non-round action must roll back the already-armed guard and its sound.
    combat.round_scripts.slots[0] = 33;
    let before = state.clone();
    assert!(matches!(
        apply(Command::Guard, &mut state, &content),
        Err(centurions_core::RuleError::Unsupported {
            action: "combat.round_story",
            ..
        })
    ));
    assert_eq!(state, before);
}
