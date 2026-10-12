use crate::prelude::*;

fn fixture() -> (Content, Value) {
    (
        crate::support::content(),
        crate::support::fixture("earned-loaded-shot-replay.json"),
    )
}

use crate::native_common::foundation_session as input;
fn project(state: &mut Session, snapshot: &Value, baseline: &Value, prepared_pair: bool) {
    crate::snapshots::earned_projection(state, snapshot, baseline);
    if prepared_pair {
        state.ammunition_work.weapon = Some(snapshot.bytes("weapon_work"));
        state.ammunition_work.ammo = Some(snapshot.bytes("ammo_work"));
    }
}

fn source_prefix_pair(state: &mut Session, content: &Content, index: usize) {
    // Seed12's last writer is NPC10 fire at command40, after Shoot34.
    // Seed1's last writer is owned needle700/0 at command40. Old neutral
    // tails do not capture these complete reached original loader outputs.
    let receipt = receipt(index);
    let (weapon, ammo) = if index == 0 {
        assert_eq!(receipt["commands"][40], "RunEnemyTurn");
        let events: Vec<Event> =
            serde_json::from_value(receipt["ordered_events"][40].clone()).unwrap();
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, Event::AttackResolved { attacker: 10, .. }))
                .count(),
            2
        );
        for step in 41..47 {
            let command: Command =
                serde_json::from_value(receipt["commands"][step].clone()).unwrap();
            assert!(matches!(command, Command::Move { .. }));
            let events: Vec<Event> =
                serde_json::from_value(receipt["ordered_events"][step].clone()).unwrap();
            assert!(events.iter().all(|event| matches!(
                event,
                Event::TacticalMoved { body: 4, .. } | Event::Sound { .. }
            )));
        }
        let record = &state.combat.as_ref().unwrap().enemies.records[0].bytes;
        let weapon = word(record, 0x26).unwrap();
        let selected = word(record, 0x4e).unwrap();
        assert_eq!((weapon, selected), (37, 37));
        // Getup reloads NPC WORD4e before 114728/11472e tests ITEM WORD32.
        // Its nonzero context selects this actual NPC's ammunition WORD4c.
        assert_ne!(word(&content.items[selected as usize], 32).unwrap(), 0);
        let ammo = word(record, 0x4c).unwrap();
        assert_eq!(ammo, 200);
        (weapon as usize, ammo as usize)
    } else {
        let command: Command = serde_json::from_value(receipt["commands"][40].clone()).unwrap();
        assert!(matches!(
            command,
            Command::BeginMedicalItem { warrior: 4, .. }
        ));
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
    assert_eq!(
        receipt["complete_state_and_every_ordered_event_replay"],
        true
    );
    assert_eq!(receipt["every_command_save_resume"], true);
    assert_eq!(receipt["full_input_save_reload"], true);
    assert_eq!(receipt["field_grants_or_random_trials"], false);
    let commands = receipt.array("commands");
    let ordered_events = receipt.array("ordered_events");
    assert_eq!(commands.len(), 47);
    assert_eq!(ordered_events.len(), commands.len());
    let mut state: Session = serde_json::from_value(receipt["starting_session"].clone()).unwrap();
    state.validate(content).unwrap();
    let mut resumed = checkpoint(&state, content, &format!("prefix-start-{index}"));
    for (step, (command, recorded)) in commands.iter().zip(ordered_events).enumerate() {
        let command: Command = serde_json::from_value(command.clone()).unwrap();
        let expected: Vec<Event> = serde_json::from_value(recorded.clone()).unwrap();
        let events = apply(command.clone(), &mut state, content).unwrap();
        assert_eq!(events, expected, "foundation {index} command {step}");
        assert_eq!(
            events,
            apply(command, &mut resumed, content).unwrap(),
            "resumed foundation {index} command {step}"
        );
        same(&state, &resumed);
        assert_eq!(state.random, resumed.random);
        resumed = checkpoint(&state, content, &format!("prefix-{index}-{step}"));
    }
    let mut expected_input = input(fixture, case);
    source_item_refresh(&mut expected_input, content);
    source_prefix_pair(&mut expected_input, content, index);
    same(&state, &expected_input);
    same(&resumed, &state);
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
    let (content, _) = fixture();
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
    let (content, fixture) = fixture();

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
        let events = apply(command.clone(), &mut state, &content).unwrap();
        assert_eq!(
            events,
            apply(command.clone(), &mut resumed, &content).unwrap()
        );
        same(&state, &resumed);
        let mut expected = initial.clone();
        project(&mut expected, &case["after"], &case["before"], true);
        source_item_refresh(&mut expected, &content);
        same(&state, &expected);
        assert_eq!(
            events,
            native_events(&initial, &expected, case),
            "ordered native effects case {index}"
        );
        crate::support::expect_array(&(sounds(&events)), &case["sounds"]);
        let mut native_random = initial.random.clone();
        for draw in case.array("rng") {
            crate::support::expect(native_random.bound(draw.n::<i16>("bound")), &draw["result"]);
        }
        assert_eq!(state.random, native_random);
        let loaded = state.warriors[4]
            .inventory()
            .unwrap()
            .into_iter()
            .find(|item| item.id() == 165)
            .unwrap();
        assert_eq!(word(&loaded.0, 2).unwrap(), 0);
        assert_eq!(state.ammunition, initial.ammunition);
        atomic(&state, &content, command);
        atomic(&state, &content, Command::Reload { warrior: 4 });
        atomic(&initial, &content, Command::ShootEnemy { body: 4 });
        atomic(&initial, &content, Command::ShootEnemy { body: 109 });
        let mut invalid = initial.clone();
        invalid.warriors[4].bytes[0x40..0x42].copy_from_slice(&1_i16.to_be_bytes());
        atomic(&invalid, &content, Command::ShootEnemy { body: 10 });
        let restored = checkpoint(&state, &content, &format!("shot-after-{index}"));
        same(&state, &restored);
    }
}
