use centurions_content::{Content, Warrior, word};
use centurions_core::{
    Command, Event, MedicalContinuation, MedicalOrigin, MedicalTarget, Session, apply,
    combat::weapon_work::WeaponWork, load, save,
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
            "../../../evidence/fixtures/earned-loaded-shot.json"
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

fn project(state: &mut Session, snapshot: &Value, baseline: &Value) {
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

fn original_definition(content: &Content, id: usize) -> Vec<u8> {
    let mut record = content.items[id].clone();
    let class = word(&record, 26).unwrap().wrapping_abs();
    record[26..28].copy_from_slice(&class.to_be_bytes());
    record
}

fn source_prepared_pair(
    state: &mut Session,
    content: &Content,
    snapshot: &Value,
    weapon: usize,
    ammo: usize,
) {
    // Actual prepattack establishes both full records. Its spell-derived
    // branch applies nonzero ammo sound WORDs at 14c944/14c960.
    let mut weapon = original_definition(content, weapon);
    let ammo = original_definition(content, ammo);
    assert!(word(&weapon, 44).unwrap() >= 1101);
    for offset in [12, 14] {
        let sound = word(&ammo, offset).unwrap();
        if sound != 0 {
            weapon[offset..offset + 2].copy_from_slice(&sound.to_be_bytes());
        }
    }
    assert_eq!(weapon, bytes(&snapshot["weapon_work"]));
    assert_eq!(ammo, bytes(&snapshot["ammo_work"]));
    state.ammunition_work.weapon = Some(weapon);
    state.ammunition_work.ammo = Some(ammo);
}

fn source_item_refresh(state: &mut Session, content: &Content) {
    // The represented public caller/combatupdate2 refresh selects actual
    // equipped work, including the successor reached by Getup.
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
    let item = original_definition(content, usize::from(id.unsigned_abs()));
    assert_eq!(state.loaded_item_field32, Some(word(&item, 32).unwrap()));
    state.ammunition_work.item = Some(item);
}

fn same_initial_projection(initial: &Session, projected: &mut Session) {
    // Before reloading, retained health effect owns the native packet alias.
    assert_eq!(
        projected.attack_packet.as_ref().unwrap()[18] as i8,
        initial.health_context.as_ref().unwrap().effect
    );
    projected.attack_packet.as_mut().unwrap()[18] = initial.attack_packet.as_ref().unwrap()[18];
    projected
        .combat
        .as_mut()
        .unwrap()
        .attack_packet
        .as_mut()
        .unwrap()[18] = initial
        .combat
        .as_ref()
        .unwrap()
        .attack_packet
        .as_ref()
        .unwrap()[18];
    same(initial, projected);
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
        "centurions-earned-loaded-shot-{}-{label}.json",
        std::process::id()
    ));
    save(&path, state, content).unwrap();
    let restored = load(&path, content).unwrap();
    same(&restored, state);
    std::fs::remove_file(path).unwrap();
    restored
}

#[test]
fn earned_loaded_shots_match_complete_native_state_and_resume() {
    let (content, fixture) = fixture();
    for (index, case) in fixture["cases"].as_array().unwrap().iter().enumerate() {
        let mut state = input(&fixture, case);
        state.validate(&content).unwrap();
        let initial = state.clone();
        let mut projected_before = initial.clone();
        project(&mut projected_before, &case["before"], &case["before"]);
        same_initial_projection(&initial, &mut projected_before);
        let mut resumed = checkpoint(&state, &content, &format!("before-{index}"));
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
        project(&mut expected, &case["after"], &case["before"]);
        source_prepared_pair(&mut expected, &content, &case["after"], 165, 280);
        source_item_refresh(&mut expected, &content);
        same(&state, &expected);
        let native_sounds: Vec<i16> = case["sounds"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_i64().unwrap() as i16)
            .collect();
        assert_eq!(sounds(&events), native_sounds, "case {index}");
        let native_damage = case["phase_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["address"] == "15075a" && p["spell_id"] == 4101)
            .map_or(0, |p| p["d0_return_word"].as_i64().unwrap() as i16);
        let mut native_events = Vec::new();
        let cast_end = case["phase_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["address"] == "10d4fe")
            .unwrap()["sound_count_after"]
            .as_u64()
            .unwrap() as usize;
        for (sound_index, resource_id) in native_sounds.iter().enumerate() {
            native_events.push(Event::Sound {
                resource_id: *resource_id,
            });
            if sound_index + 1 == cast_end {
                native_events.push(Event::AttackResolved {
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
                    native_events.push(Event::SpellResolved {
                        caster: None,
                        spell: 2204,
                        target: target as i16,
                        damage,
                        defeated: false,
                    });
                }
            }
        }
        if state.combat.as_ref().unwrap().round != initial.combat.as_ref().unwrap().round {
            native_events.push(Event::TurnStarted {
                body: state.combat.as_ref().unwrap().active_body().unwrap(),
                round: state.combat.as_ref().unwrap().round,
            });
        }
        assert_eq!(events, native_events, "ordered native effects case {index}");
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
        let restored = checkpoint(&state, &content, &format!("after-{index}"));
        same(&state, &restored);
    }
}

#[test]
fn corrected_medical_round_byte_matches_native_normal_cancel_and_rejection() {
    let (content, fixture) = fixture();
    for (index, case) in fixture["medical_counter_cases"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        let mut state: Session =
            serde_json::from_value(
                fixture["medical_counter_foundations"]
                    [case["foundation"].as_u64().unwrap() as usize]["session"]
                    .clone(),
            )
            .unwrap();
        let initial = state.clone();
        let mut before = initial.clone();
        project(&mut before, &case["before"], &case["before"]);
        same_initial_projection(&initial, &mut before);
        let begin = Command::BeginMedicalItem {
            warrior: 4,
            slot: case["slot"].as_u64().unwrap() as usize,
        };
        let mut events = apply(begin.clone(), &mut state, &content).unwrap();
        let mut expected = initial.clone();
        project(&mut expected, &case["prepared"], &case["before"]);
        source_prepared_pair(&mut expected, &content, &case["prepared"], 700, 0);
        source_item_refresh(&mut expected, &content);
        expected.medical_continuation = Some(MedicalContinuation {
            origin: MedicalOrigin::OwnedNeedle,
            actor: 4,
            item: 700,
            spell: 1401,
            power: 1,
        });
        same(&state, &expected);
        atomic(&state, &content, begin);
        let mut resumed = checkpoint(&state, &content, &format!("medical-pending-{index}"));
        let selection = if case["kind"] == "cancel" {
            MedicalTarget::ConfirmedCancel
        } else {
            MedicalTarget::LivingPoint {
                x: (case["target_x"].as_i64().unwrap()
                    + case["prepared"]["shift_x"].as_i64().unwrap()
                    - case["before"]["shift_x"].as_i64().unwrap()) as i16,
                y: (case["target_y"].as_i64().unwrap()
                    + case["prepared"]["shift_y"].as_i64().unwrap()
                    - case["before"]["shift_y"].as_i64().unwrap()) as i16,
            }
        };
        let command = Command::SelectMedicalTarget { selection };
        if case["kind"] == "range_reject" {
            atomic(&state, &content, command);
            same(&state, &resumed);
        } else {
            let result = apply(command.clone(), &mut state, &content).unwrap();
            assert_eq!(result, apply(command, &mut resumed, &content).unwrap());
            same(&state, &resumed);
            events.extend(result);
            let mut expected = initial.clone();
            project(&mut expected, &case["after"], &case["before"]);
            source_prepared_pair(&mut expected, &content, &case["after"], 700, 0);
            source_item_refresh(&mut expected, &content);
            expected.medical_continuation = None;
            same(&state, &expected);
            assert_eq!(
                sounds(&events),
                case["use_sounds"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_i64().unwrap() as i16)
                    .collect::<Vec<_>>()
            );
        }
        assert_eq!(state.combat.as_ref().unwrap().round_scripts.counter, 1);
        assert_eq!(state.combat.as_ref().unwrap().round, 0);
        let restored = checkpoint(&state, &content, &format!("medical-after-{index}"));
        same(&state, &restored);
    }
}
