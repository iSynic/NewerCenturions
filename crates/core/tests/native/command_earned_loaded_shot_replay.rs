use crate::prelude::*;

use crate::native_common::foundation_session as input;
fn project(state: &mut Session, snapshot: &Value, baseline: &Value, prepared_pair: bool) {
    let retained_body = state.health_context.as_ref().unwrap().last_body;
    crate::snapshots::earned_projection(state, snapshot, baseline);
    // Frozen probes start with old body0; an unchanged native field retains the earned origin.
    if snapshot["health_last_body"] == baseline["health_last_body"] {
        state.health_context.as_mut().unwrap().last_body = retained_body;
    }
    if prepared_pair {
        state.ammunition_work.weapon = Some(snapshot.bytes("weapon_work"));
        state.ammunition_work.ammo = Some(snapshot.bytes("ammo_work"));
    }
}

fn source_prefix_pair(state: &mut Session, content: &Content, index: usize) {
    // Seed12's last writer is NPC10 fire at command40, after Shoot34.
    // Seed1's last writer is owned needle700/0 at command40. Old neutral
    // tails do not capture these complete reached original loader outputs.
    let (weapon, ammo) = if index == 0 {
        let record = &state.combat.as_ref().unwrap().enemies.records[0].bytes;
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
    assert_eq!(state.weapon_work.sound, Some(word(&weapon, 14).unwrap()));
    assert_eq!(
        state.weapon_work.weapon_armor_filter,
        Some(word(&weapon, 136).unwrap())
    );
    assert_eq!(
        state.weapon_work.ammunition_armor_filter,
        Some(word(&ammo, 136).unwrap())
    );
    state.ammunition_work.weapon = Some(weapon);
    state.ammunition_work.ammo = Some(ammo);
}
use crate::native_common::refresh_item_work as source_item_refresh;

use crate::support::reject_command as atomic;
use crate::support::same_session as same;

use crate::support::checkpoint;
use crate::support::event_sounds as sounds;

fn receipt(index: usize) -> Value {
    let receipts = [
        include_str!("../../../../evidence/fixtures/earned-loaded-shot-replay-receipt-12.json"),
        include_str!("../../../../evidence/fixtures/earned-loaded-shot-replay-receipt-1.json"),
    ];
    serde_json::from_str(receipts[index]).unwrap()
}

fn replay(fixture: &Value, case: &Value, content: &Content) -> Session {
    let index = case.n::<usize>("foundation");
    let receipt = receipt(index);
    let commands = receipt.array("commands");
    let ordered_events = receipt.array("ordered_events");
    assert_eq!(ordered_events.len(), commands.len());
    let mut state: Session = serde_json::from_value(receipt["starting_session"].clone()).unwrap();
    state.validate(content).unwrap();
    state = checkpoint(&state, content, &format!("prefix-start-{index}"));
    for (step, (command, recorded)) in commands.iter().zip(ordered_events).enumerate() {
        let command: Command = serde_json::from_value(command.clone()).unwrap();
        let command = match command {
            Command::BeginMedicalItem { warrior, slot } => Command::UseItem { warrior, slot },
            other => other,
        };
        let expected: Vec<Event> = serde_json::from_value(recorded.clone()).unwrap();
        let events = apply(command.clone(), &mut state, content).unwrap();
        assert_eq!(events, expected, "foundation {index} command {step}");
        if state.medical_continuation.is_some() {
            state = checkpoint(&state, content, &format!("prefix-medical-{index}"));
        }
    }
    let mut expected_input = input(fixture, case);
    // This earned battle is still inside timeclick's regional newland call.
    expected_input.movement_continuation =
        Some(centurions_core::MovementContinuation::RegionalReturn { header: false });
    // The recorded prefix defeats body16; native killbody retains that body.
    expected_input.health_context.as_mut().unwrap().last_body = 16;
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
    crate::snapshots::actor_work(&mut expected_input, &case["before"]);
    source_item_refresh(&mut expected_input, content);
    source_prefix_pair(&mut expected_input, content, index);
    same(&state, &expected_input);
    let effect = state.health_context.as_ref().unwrap().effect;
    assert_eq!(state.attack_packet.as_ref().unwrap()[18] as i8, effect);
    assert_eq!(
        state
            .combat
            .as_ref()
            .unwrap()
            .attack_packet
            .as_ref()
            .unwrap()[18] as i8,
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
                });
            }
        }
    }
    if expected.combat.as_ref().unwrap().round != initial.combat.as_ref().unwrap().round {
        events.push(Event::TurnStarted {
            body: expected.combat.as_ref().unwrap().active_body().unwrap(),
            round: expected.combat.as_ref().unwrap().round,
        });
    }
    events
}

#[test]
fn party_range_membership_retains_the_original_truncated_boundary() {
    let content = crate::support::content();
    let receipt = receipt(1);
    let mut state: Session = serde_json::from_value(receipt["starting_session"].clone()).unwrap();
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
    let ammo =
        centurions_core::combat::enemy_weapons::ammo_id(actor, state.loaded_item_field32.unwrap())
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
    let content = crate::support::content();
    let fixture: Value = crate::support::fixture("earned-loaded-shot-replay.json");

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
        same(&initial, &projected_before);
        let mut resumed = checkpoint(&state, &content, &format!("shot-before-{index}"));
        let command = Command::ShootEnemy {
            body: case.n::<i16>("target"),
        };
        let events = replay_command(command.clone(), &mut state, &mut resumed, &content);
        let mut expected = initial.clone();
        project(&mut expected, &case["after"], &case["before"], true);
        expected.combat.as_mut().unwrap().inactive_rounds = case["after"].n("inactivity_counter");
        source_item_refresh(&mut expected, &content);
        same(&state, &expected);
        assert_eq!(
            events,
            native_events(&initial, &expected, case),
            "ordered native effects case {index}"
        );
        crate::support::expect_array(&(sounds(&events)), &case["sounds"]);
        let mut native_random = initial.random.clone();
        crate::support::replay_rng(&mut native_random, case.array("rng"));
        assert_eq!(state.random, native_random);
        let loaded = state.warriors[4]
            .inventory()
            .unwrap()
            .into_iter()
            .find(|item| item.id() == 165)
            .unwrap();
        assert_eq!(word(&loaded.0, 2).unwrap(), 0);
        assert_eq!(state.ammunition, initial.ammunition);
        if state.combat.as_ref().unwrap().enemies.records[0]
            .stamina()
            .unwrap()
            > 0
        {
            crate::support::refuse_command(
                &state,
                &content,
                command,
                vec![
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
        crate::support::refuse_command(
            &invalid,
            &content,
            Command::ShootEnemy { body: 10 },
            vec![centurions_core::feedback::NativeFeedback::Silent],
        );
        let restored = checkpoint(&state, &content, &format!("shot-after-{index}"));
        same(&state, &restored);
    }
}
