use centurions_content::{Content, Warrior, word};
use centurions_core::{
    Command, Event, Session, apply, combat::weapon_work::WeaponWork, load, save,
};
use serde_json::Value;

fn fixture() -> (Content, Value) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    (
        Content::load(
            &root.join("NewCenturions"),
            &root.join("analysis/extracted"),
        )
        .unwrap(),
        serde_json::from_str(include_str!(
            "../../../evidence/fixtures/earned-loaded-shot-replay.json"
        ))
        .unwrap(),
    )
}

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
        .map(|v| i16::from_be_bytes(*v))
        .collect()
}
fn party(state: &mut Session, snapshot: &Value) {
    for (warrior, record) in state
        .warriors
        .iter_mut()
        .zip(bytes(&snapshot["party"]).as_chunks::<3138>().0)
    {
        warrior.bytes = Warrior::read(record).unwrap().bytes;
    }
}
fn input(fixture: &Value, case: &Value) -> Session {
    serde_json::from_value(
        fixture["foundations"][case["foundation"].as_u64().unwrap() as usize]["session"].clone(),
    )
    .unwrap()
}

fn project(state: &mut Session, snapshot: &Value, baseline: &Value, prepared_pair: bool) {
    if prepared_pair {
        state.ammunition_work.weapon = Some(bytes(&snapshot["weapon_work"]));
        state.ammunition_work.ammo = Some(bytes(&snapshot["ammo_work"]));
    }
    party(state, snapshot);
    state.random.seed = snapshot["seed"].as_u64().unwrap() as u32;
    state.random.draws = snapshot["draws"].as_u64().unwrap();
    state.selected_warrior = snapshot["selected_member"].as_u64().unwrap() as usize;
    state.loaded_item_field32 = Some(snapshot["item_context"].as_i64().unwrap() as i16);
    state.attack_count = snapshot["count"].as_i64().unwrap() as i16;
    state
        .party_mask
        .copy_from_slice(&words(&snapshot["mask"])[..6]);
    state.bandage_ready = Some(snapshot["aid_gate"].as_i64().unwrap() as i16);
    state.party_currency = snapshot["currency"].as_i64().unwrap() as i32;
    state.flags = bytes(&snapshot["flags"]);
    state.ammunition = bytes(&snapshot["ammunition"])
        .as_chunks::<4>()
        .0
        .iter()
        .map(|v| i32::from_be_bytes(*v))
        .collect();
    state.psi_memory = Some(bytes(&snapshot["memory"]));
    state.party_buffs = bytes(&snapshot["party_buffs"]);
    let combat = state.combat.as_mut().unwrap();
    for (record, native) in combat
        .enemies
        .records
        .iter_mut()
        .zip(bytes(&snapshot["records"]).as_chunks::<288>().0)
    {
        record.bytes = native.to_vec();
    }
    for (target, key) in [
        (&mut combat.squad.positions, "party_positions"),
        (&mut combat.enemies.positions, "monster_positions"),
    ] {
        *target = words(&snapshot[key])
            .as_chunks::<2>()
            .0
            .iter()
            .map(|p| {
                (
                    p[0].wrapping_add(
                        (snapshot["shift_x"].as_i64().unwrap()
                            - baseline["shift_x"].as_i64().unwrap()) as i16,
                    ),
                    p[1].wrapping_add(
                        (snapshot["shift_y"].as_i64().unwrap()
                            - baseline["shift_y"].as_i64().unwrap()) as i16,
                    ),
                )
            })
            .collect();
    }
    combat.squad.under_tiles = words(&snapshot["under_tiles"]);
    combat.map.tiles = words(&snapshot["map"]);
    combat.order = words(&snapshot["order"]);
    combat.cursor = snapshot["cursor"].as_u64().unwrap() as usize;
    combat.round +=
        (snapshot["round"].as_i64().unwrap() - baseline["round"].as_i64().unwrap()) as i16;
    combat.round_scripts.counter = snapshot["round_script_counter"].as_i64().unwrap() as i8;
    combat.round_scripts.slots = words(&snapshot["round_slots"]).try_into().unwrap();
    combat.spell_fields = bytes(&snapshot["fields"]);
    combat.field_used = bytes(&snapshot["used"]);
    combat.unavailable = snapshot["unavailable"].as_u64().unwrap() as i8;
    combat.attack_packet = Some(bytes(&snapshot["packet"]));
    combat.attack_identity = Some(words(&snapshot["identity"]).try_into().unwrap());
    let weapon = bytes(&snapshot["weapon_work"]);
    let ammo = bytes(&snapshot["ammo_work"]);
    combat.weapon_work = WeaponWork {
        sound: Some(word(&weapon, 14).unwrap()),
        weapon_armor_filter: Some(word(&weapon, 136).unwrap()),
        ammunition_armor_filter: Some(word(&ammo, 136).unwrap()),
    };
    state.attack_packet = combat.attack_packet.clone();
    state.attack_identity = combat.attack_identity;
    state.weapon_work = combat.weapon_work;
    let health = state.health_context.as_mut().unwrap();
    health.selected_warrior = state.selected_warrior;
    health.unavailable = combat.unavailable;
    health.effect = snapshot["health_effect"].as_u64().unwrap() as i8;
    health.last_body = snapshot["health_last_body"].as_i64().unwrap() as i16;
    health.marks_affected = snapshot["heal_marks_affected"].as_u64().unwrap() != 0;
    health
        .affected
        .copy_from_slice(&words(&snapshot["markers"])[..6]);
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

fn source_item_refresh(state: &mut Session, content: &Content) {
    // Compose the omitted combatupdate2 writer after the native Getup/prefix:
    // NPC WORD4e or party ca/e4 (BYTE9dd) selects the complete loaditem record.
    let body = state.combat.as_ref().unwrap().active_body().unwrap();
    let id = if body >= 10 {
        word(
            &state.combat.as_ref().unwrap().enemies.records[(body - 10) as usize].bytes,
            0x4e,
        )
        .unwrap()
    } else {
        let warrior = &state.warriors[body as usize].bytes;
        word(warrior, if warrior[0x9dd] == 0 { 0xca } else { 0xe4 }).unwrap()
    };
    if body >= 10 && id == 0 {
        return;
    }
    let mut record = content.items[usize::from(id.unsigned_abs())].clone();
    let class = word(&record, 26).unwrap().wrapping_abs();
    record[26..28].copy_from_slice(&class.to_be_bytes());
    state.loaded_item_field32 = Some(word(&record, 32).unwrap());
    state.ammunition_work.item = Some(record);
}
fn first_difference(actual: &Value, expected: &Value, path: &str) -> Option<String> {
    if actual == expected {
        return None;
    }
    match (actual, expected) {
        (Value::Object(a), Value::Object(b)) if a.len() == b.len() => {
            for (key, value) in a {
                if let Some(other) = b.get(key) {
                    if let Some(error) = first_difference(value, other, &format!("{path}.{key}")) {
                        return Some(error);
                    }
                } else {
                    return Some(format!("{path}.{key}: unexpected field"));
                }
            }
        }
        (Value::Array(a), Value::Array(b)) if a.len() == b.len() => {
            for (index, (v, w)) in a.iter().zip(b).enumerate() {
                if let Some(error) = first_difference(v, w, &format!("{path}[{index}]")) {
                    return Some(error);
                }
            }
        }
        (Value::Array(a), Value::Array(b)) => {
            return Some(format!("{path}: lengths {} != {}", a.len(), b.len()));
        }
        _ => return Some(format!("{path}: {actual} != {expected}")),
    }
    Some(format!("{path}: fields differ"))
}

fn same(actual: &Session, expected: &Session) {
    if let Some(error) = first_difference(
        &serde_json::to_value(actual).unwrap(),
        &serde_json::to_value(expected).unwrap(),
        "session",
    ) {
        panic!("{error}");
    }
}

fn atomic(state: &Session, content: &Content, command: Command) {
    let mut candidate = state.clone();
    assert!(apply(command, &mut candidate, content).is_err());
    same(&candidate, state);
}

fn sounds(events: &[Event]) -> Vec<i16> {
    events
        .iter()
        .filter_map(|e| {
            if let Event::Sound { resource_id } = e {
                Some(*resource_id)
            } else {
                None
            }
        })
        .collect()
}

fn checkpoint(state: &Session, content: &Content, label: &str) -> Session {
    let path = std::env::temp_dir().join(format!(
        "centurions-earned-loaded-shot-replay-{}-{label}.json",
        std::process::id()
    ));
    save(&path, state, content).unwrap();
    let restored = load(&path, content).unwrap();
    same(&restored, state);
    std::fs::remove_file(path).unwrap();
    restored
}

fn receipt(index: usize) -> Value {
    let receipts = [
        include_str!("../../../evidence/fixtures/earned-loaded-shot-replay-receipt-12.json"),
        include_str!("../../../evidence/fixtures/earned-loaded-shot-replay-receipt-1.json"),
    ];
    serde_json::from_str(receipts[index]).unwrap()
}

fn replay(fixture: &Value, case: &Value, content: &Content) -> Session {
    let index = case["foundation"].as_u64().unwrap() as usize;
    let receipt = receipt(index);
    assert_eq!(
        receipt["complete_state_and_every_ordered_event_replay"],
        true
    );
    assert_eq!(receipt["every_command_save_resume"], true);
    assert_eq!(receipt["full_input_save_reload"], true);
    assert_eq!(receipt["field_grants_or_random_trials"], false);
    let commands = receipt["commands"].as_array().unwrap();
    let ordered_events = receipt["ordered_events"].as_array().unwrap();
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
    let native_sounds: Vec<i16> = case["sounds"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_i64().unwrap() as i16)
        .collect();
    let native_damage = case["phase_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["address"] == "15075a" && p["spell_id"] == 4101)
        .map_or(0, |p| p["d0_return_word"].as_i64().unwrap() as i16);
    let mut events = Vec::new();
    let cast_end = case["phase_receipts"]
        .as_array()
        .unwrap()
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
        for r in case["resolutions"].as_array().unwrap().iter().filter(|r| {
            r["spell_id"] == 2204
                && r["sound_count_after"].as_u64().unwrap() as usize == sound_index + 1
        }) {
            for (target, flag) in words(&r["applied_mask"])
                .iter()
                .enumerate()
                .filter(|(_, flag)| **flag > 0)
            {
                assert_eq!(*flag, 1);
                let damage = case["phase_receipts"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|p| {
                        p["address"] == "15075a"
                            && p["spell_id"] == 2204
                            && p["arguments"][0].as_u64().unwrap() as usize == target
                    })
                    .map_or(0, |p| p["d0_return_word"].as_i64().unwrap() as i16);
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
    for (step, (command, recorded)) in receipt["commands"]
        .as_array()
        .unwrap()
        .iter()
        .zip(receipt["ordered_events"].as_array().unwrap())
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
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 2);
    for (index, case) in fixture["cases"].as_array().unwrap().iter().enumerate() {
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
            body: case["target"].as_i64().unwrap() as i16,
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
        assert_eq!(
            sounds(&events),
            case["sounds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_i64().unwrap() as i16)
                .collect::<Vec<_>>()
        );
        let mut native_random = initial.random.clone();
        for draw in case["rng"].as_array().unwrap() {
            assert_eq!(
                native_random.bound(draw["bound"].as_i64().unwrap() as i16),
                draw["result"].as_i64().unwrap() as i16
            );
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
