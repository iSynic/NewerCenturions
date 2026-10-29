use crate::{native_common, prelude::*, snapshots, support};

use native_common::foundation_session as input;
fn project(state: &mut Session, snapshot: &Value, baseline: &Value, prepared_pair: bool) {
    let retained_body = state.native_work.health_context.last_body;
    snapshots::earned_projection(state, snapshot, baseline);
    // Frozen probes start with old body0; an unchanged native field retains the earned origin.
    if snapshot["health_last_body"] == baseline["health_last_body"] {
        state.native_work.health_context.last_body = retained_body;
    }
    if prepared_pair {
        state.native_work.ammunition_work.weapon = Some(snapshot.bytes("weapon_work"));
        state.native_work.ammunition_work.ammo = Some(snapshot.bytes("ammo_work"));
    }
}

fn source_prefix_pair(state: &mut Session, content: &Content, index: usize) {
    // Seed12's last writer is NPC10 fire at command40, after Shoot34.
    // Seed1's last writer is owned needle700/0 at command40. Old neutral
    // tails do not capture these complete reached original loader outputs.
    let (weapon, ammo) = if index == 0 {
        let record = &current_combat(state).enemies.records[0].bytes;
        let weapon = word(record, 0x26).unwrap();
        let ammo = word(record, 0x4c).unwrap();
        (weapon as usize, ammo as usize)
    } else {
        (700, 0)
    };
    let normalized = |id: usize| {
        let mut bytes = content.items[id].clone();
        let class = word(&bytes, 26).unwrap().wrapping_abs();
        bytes[26..28].copy_from_slice(&class.to_be_bytes());
        bytes
    };
    let ammo = normalized(ammo);
    let mut weapon = normalized(weapon);
    if word(&weapon, 44).unwrap() >= 1101 {
        for offset in [12, 14] {
            let value = word(&ammo, offset).unwrap();
            if value != 0 {
                weapon[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
            }
        }
        if word(&ammo, 56).unwrap() != 0 {
            // Original 14c9a0 stores the sign-extended low byte, not the WORD.
            weapon[56..58].copy_from_slice(&i16::from(ammo[57] as i8).to_be_bytes());
        }
    }
    assert_eq!(
        state.native_work.weapon_work.sound,
        word(&weapon, 14).unwrap()
    );
    assert_eq!(
        state.native_work.weapon_work.weapon_armor_filter,
        word(&weapon, 136).unwrap()
    );
    assert_eq!(
        state.native_work.weapon_work.ammunition_armor_filter,
        word(&ammo, 136).unwrap()
    );
    state.native_work.ammunition_work.weapon = Some(weapon);
    state.native_work.ammunition_work.ammo = Some(ammo);
}
use native_common::refresh_item_work as source_item_refresh;

use support::reject_command as atomic;

use support::checkpoint;
use support::event_sounds as sounds;

fn replay(fixture: &Value, case: &Value, content: &Content) -> Session {
    let index = case.n::<usize>("foundation");
    let receipt = native_common::earned_receipt("earned-loaded-shot-replay", index);
    let commands = receipt.array("commands");
    let ordered_events = receipt.array("ordered_events");
    assert_eq!(ordered_events.len(), commands.len());
    let mut state = native_common::earned_start(&receipt);
    state.validate(content).unwrap();
    state = checkpoint(&state, content, &format!("prefix-start-{index}"));
    for (step, (command, recorded)) in commands.iter().zip(ordered_events).enumerate() {
        let command: Command = serde_json::from_value(command.clone()).unwrap();
        let command = match command {
            Command::BeginMedicalItem { warrior, slot } => Command::UseItem { warrior, slot },
            other => other,
        };
        let before = state.clone();
        let events = apply(command.clone(), &mut state, content).unwrap();
        let expected = native_common::prefix_result_events(
            &before,
            &state,
            &command,
            serde_json::from_value(recorded.clone()).unwrap(),
        );
        assert_eq!(events, expected, "foundation {index} command {step}");
        if state.medical_continuation.is_some() {
            state = checkpoint(&state, content, &format!("prefix-medical-{index}"));
        }
    }
    native_common::earned_rejoin(&mut state, &receipt);
    let mut expected_input = input(fixture, case);
    // This earned battle is still inside timeclick's regional newland call.
    expected_input.movement_continuation =
        Some(centurions_core::MovementContinuation::RegionalReturn { header: false });
    // The recorded prefix defeats body16; native killbody retains that body.
    expected_input
        .native_work
        .health_context
        .last_body = 16;
    let last_npc = ordered_events
        .iter()
        .flat_map(|events| events.as_array().unwrap())
        .filter_map(|event| {
            event
                .get("TurnStarted")?
                .get("body")?
                .as_i64()
                .filter(|body| *body >= 10)
        })
        .next_back()
        .unwrap() as usize
        - 10;
    expected_input.combat.as_mut().unwrap().selected_enemy = last_npc;
    snapshots::actor_work(&mut expected_input, &case["before"]);
    source_item_refresh(&mut expected_input, content);
    source_prefix_pair(&mut expected_input, content, index);
    expected_input.native_work.inventory_record = native_common::earned_inventory_work(index);
    expected_input.combat.as_mut().unwrap().turn_origin =
        snapshots::recorded_turn_origin(&receipt, current_combat(&expected_input));
    // Both earned prefixes end at Move(1,0), which writes442736/442737.
    // The historical foundation retained the preceding outdoor Move(1,1).
    expected_input.exploration.as_mut().unwrap().last_move = [1, 0];
    support::same_captured_session(&state, &expected_input);
    let effect = state.native_work.health_context.effect;
    assert_eq!(
        state.native_work.attack_packet[18] as i8,
        effect
    );
    assert_eq!(
        current_combat(&state).attack_packet[18] as i8,
        effect
    );
    state
}

fn native_events(initial: &Session, expected: &Session, case: &Value) -> Vec<Event> {
    let native_sounds: Vec<i16> = case.n::<Vec<i16>>("sounds");
    let native_damage = case
        .array("phase_receipts")
        .iter()
        .find(|p| p["address"] == "15075a" && p["spell_id"] == 4101)
        .map_or(0, |p| p.n::<i16>("d0_return_word"));
    let mut events = Vec::new();
    let cast_end = case
        .array("phase_receipts")
        .iter()
        .find(|p| p["address"] == "10d4fe")
        .unwrap()["sound_count_after"]
        .as_u64()
        .unwrap() as usize;
    // The target key's firstt(76, 77) (combatchoice 11b026).
    events.push(Event::Feedback(
        centurions_core::feedback::NativeFeedback::Tutorial {
            first: 76,
            last: 77,
        },
    ));
    for (sound_index, resource_id) in native_sounds.iter().enumerate() {
        events.push(Event::Sound {
            resource_id: *resource_id,
        });
        if sound_index + 1 == cast_end {
            events.push(Event::AttackResolved {
                attacker: 4,
                target: 10,
                damage: native_damage,
                defeated: native_damage > 0,
                icon: Default::default(),
                flight: Default::default(),
            });
        }
        for r in case.array("resolutions").iter().filter(|r| {
            r["spell_id"] == 2204 && r.n::<usize>("sound_count_after") == sound_index + 1
        }) {
            for (target, flag) in r
                .words("applied_mask")
                .iter()
                .enumerate()
                .filter(|(_, flag)| **flag > 0)
            {
                assert_eq!(*flag, 1);
                let damage = case
                    .array("phase_receipts")
                    .iter()
                    .find(|p| {
                        p["address"] == "15075a"
                            && p["spell_id"] == 2204
                            && p["arguments"][0].as_u64().unwrap() as usize == target
                    })
                    .map_or(0, |p| p.n::<i16>("d0_return_word"));
                events.push(Event::SpellResolved {
                    caster: None,
                    spell: 2204,
                    target: target as i16,
                    damage,
                    defeated: false,
                    icon: Default::default(),
                    flight: Default::default(),
                });
            }
        }
    }
    if current_combat(expected).round != current_combat(initial).round {
        events.push(Event::TurnStarted {
            body: current_combat(expected).active_body().unwrap(),
            round: current_combat(expected).round,
        });
    }
    events
}

#[test]
fn party_range_membership_retains_the_original_truncated_boundary() {
    let content = support::content();
    let receipt = native_common::earned_receipt("earned-loaded-shot-replay", 1);
    let mut state = native_common::earned_start(&receipt);
    for (step, (command, recorded)) in receipt
        .array("commands")
        .iter()
        .zip(receipt.array("ordered_events"))
        .take(12)
        .enumerate()
    {
        let expected: Vec<Event> = serde_json::from_value(recorded.clone()).unwrap();
        assert_eq!(
            apply(
                serde_json::from_value(command.clone()).unwrap(),
                &mut state,
                &content
            )
            .unwrap(),
            expected,
            "prefix command {step}"
        );
    }
    let mission = content
        .map(state.exploration.as_ref().unwrap().map)
        .unwrap();
    let combat = state.combat.as_mut().unwrap();
    assert_eq!(combat.active_body(), Some(14));
    assert_eq!(combat.squad.positions[5], (35, 34));
    let recorded: Vec<Event> =
        serde_json::from_value(receipt["ordered_events"][12].clone()).unwrap();
    let endpoint = recorded
        .iter()
        .filter_map(|event| match event {
            Event::TacticalMoved { body: 14, x, y } => Some((*x, *y)),
            _ => None,
        })
        .next_back()
        .unwrap();
    assert_eq!(endpoint, (40, 38));
    // Project only the original movement endpoint for this selection seam.
    // Getrange 125654..12565e uses checkrange's truncated party distance;
    // the separate NPC comparison at 125888 retains the extended distance.
    combat.enemies.positions[4] = endpoint;
    let actor = &mut combat.enemies.records[4];
    let weapon = word(&actor.bytes, 0x26).unwrap();
    assert_eq!(weapon, 9);
    let ammo = centurions_core::combat::enemy_weapons::ammo_id(
        actor,
        state.native_work.loaded_item_field32,
    )
    .unwrap();
    let prepared = centurions_core::combat::enemy_weapons::prepare(
        actor,
        &content.items[weapon as usize],
        &content.items[ammo as usize],
        &mission.combat.as_ref().unwrap().attacks,
    )
    .unwrap();
    let mut random = state.random.clone();
    let selection = centurions_core::combat::spells::select_firearm_targets(
        combat,
        &state.warriors,
        mission,
        &mission.areas,
        4,
        &prepared,
        &mut random,
    )
    .unwrap();
    assert_eq!(selection.range, 6);
    assert_eq!(selection.count, 1);
    assert_eq!(selection.distances[1], 5);
    assert_eq!(selection.distances[2], 6);
    assert_eq!(selection.distances[5], 6);
    assert_eq!(
        selection.distances[..6]
            .iter()
            .filter(|distance| **distance <= 6)
            .count(),
        3
    );
}

#[test]
fn coherent_earned_prefixes_and_shots_match_complete_native_state_and_resume() {
    let content = support::content();
    let fixture: Value = support::fixture("earned-loaded-shot-replay.json");

    for (index, case) in fixture.array("cases").iter().enumerate() {
        let mut state = replay(&fixture, case, &content);
        let initial = state.clone();
        let mut projected_before = initial.clone();
        project(
            &mut projected_before,
            &case["before"],
            &case["before"],
            false,
        );
        source_item_refresh(&mut projected_before, &content);
        support::same_captured_session(&initial, &projected_before);
        let mut resumed = checkpoint(&state, &content, &format!("shot-before-{index}"));
        let command = Command::ShootEnemy {
            body: case.n::<i16>("target"),
        };
        let events = replay_command(command.clone(), &mut state, &mut resumed, &content);
        let mut expected = initial.clone();
        project(&mut expected, &case["after"], &case["before"], true);
        expected.combat.as_mut().unwrap().inactive_rounds = case["after"].n("inactivity_counter");
        source_item_refresh(&mut expected, &content);
        support::same_captured_session(&state, &expected);
        assert_eq!(
            events,
            native_events(&initial, &expected, case),
            "ordered native effects case {index}"
        );
        support::expect_array(&(sounds(&events)), &case["sounds"]);
        let mut native_random = initial.random.clone();
        support::replay_rng(&mut native_random, case.array("rng"));
        assert_eq!(state.random, native_random);
        let loaded = state.warriors[4]
            .inventory()
            .unwrap()
            .into_iter()
            .find(|item| item.id() == 165)
            .unwrap();
        assert_eq!(word(&loaded.0, 2).unwrap(), 0);
        assert_eq!(state.ammunition, initial.ammunition);
        if current_combat(&state).enemies.records[0].stamina().unwrap() > 0 {
            support::refuse_command(
                &state,
                &content,
                command,
                vec![
                    centurions_core::feedback::NativeFeedback::Tutorial {
                        first: 76,
                        last: 77,
                    },
                    centurions_core::feedback::NativeFeedback::Tutorial {
                        first: 78,
                        last: 78,
                    },
                    centurions_core::feedback::NativeFeedback::Combat {
                        kind: centurions_core::feedback::CombatRefusal::NotEnoughAmmo,
                    },
                ],
            );
        } else {
            atomic(&state, &content, command);
        }
        atomic(&initial, &content, Command::ShootEnemy { body: 109 });
        let mut invalid = initial.clone();
        invalid.warriors[4].bytes[0x40..0x42].copy_from_slice(&1_i16.to_be_bytes());
        support::refuse_command(
            &invalid,
            &content,
            Command::ShootEnemy { body: 10 },
            vec![
                centurions_core::feedback::NativeFeedback::Tutorial {
                    first: 76,
                    last: 77,
                },
                centurions_core::feedback::NativeFeedback::Silent,
            ],
        );
        checkpoint(&state, &content, &format!("shot-after-{index}"));
    }
}

#[test]
fn ordinary_firearm_kills_keep_native_death_feedback_and_saved_return() {
    let content = support::content();
    let foundation = support::fixture("earned-loaded-shot-replay.json");
    let native = support::fixture("death-feedback.json");
    let refresh = support::fixture("combat-item-refresh.json");
    for case in native.array("cases") {
        let weapon = case.n::<i16>("weapon");
        let ammo: i16 = match weapon {
            2 => 201,
            38 => 203,
            _ => 280,
        };
        let mut state = input(&foundation, &foundation.array("cases")[1]);
        let actor = current_combat(&state).active_body().unwrap() as usize;
        let mut item = centurions_content::records::ItemDefinition(&content.items[weapon as usize])
            .fresh_stock()
            .unwrap();
        // Assisted loaded ammunition and range isolate the death-feedback caller.
        item.0[9] = 1;
        item.0[20] = 30;
        item.0[2..4].copy_from_slice(&2_i16.to_be_bytes());
        item.0[4..6].copy_from_slice(&ammo.to_be_bytes());
        state.warriors[actor].bytes[0x132..0x148].copy_from_slice(&item.0);
        state.warriors[actor].bytes[0xe4..0xe6].copy_from_slice(&weapon.to_be_bytes());
        state.warriors[actor].bytes[0x126..0x128].copy_from_slice(&ammo.to_be_bytes());
        source_item_refresh(&mut state, &content);
        let target = state.combat.as_mut().unwrap();
        // Assisted initiative isolates the next warrior's original item loader.
        for body in &mut target.order {
            if *body == 0 {
                *body = -1;
            }
        }
        target.order[target.cursor + 1] = 0;
        target.enemies.records[0].bytes[0x110..0x112].copy_from_slice(&1_i16.to_be_bytes());
        target.enemies.records[0].bytes[0xb3] = 1;
        target.enemies.records[0].bytes[0x6a..0x6c]
            .copy_from_slice(&case.n::<i16>("record_sound").to_be_bytes());
        let events = support::saved_command(Command::ShootEnemy { body: 10 }, &mut state, &content);
        assert!(
            events.iter().any(|event| matches!(
                event,
                Event::AttackResolved {
                    target: 10,
                    defeated: true,
                    ..
                }
            )),
            "weapon {weapon}: {events:?}"
        );
        let cue = case.array("sounds")[0].as_i64().unwrap() as i16;
        assert_eq!(
            sounds(&events)
                .iter()
                .filter(|sound| **sound == cue)
                .count(),
            1,
            "original showresults death cue for weapon {weapon}"
        );
        let next = refresh
            .array("cases")
            .iter()
            .find(|case| case["preceding_weapon"] == weapon)
            .unwrap();
        assert_eq!(
            current_combat(&state).active_body(),
            Some(next.n("next_body"))
        );
        let mut expected = content.items[next.n::<usize>("selected_item")].clone();
        let category = word(&expected, 26).unwrap().wrapping_abs();
        expected[26..28].copy_from_slice(&category.to_be_bytes());
        assert_eq!(
            state.native_work.ammunition_work.item.as_ref(),
            Some(&expected),
            "next native item after weapon {weapon}"
        );
        checkpoint(&state, &content, "assisted-firearm-after");
    }
}
