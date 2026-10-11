use centurions_content::{Content, Warrior, word};
use centurions_core::{
    Command, Event, MedicalContinuation, MedicalTarget, Session, apply,
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
            "../../../evidence/fixtures/earned-combat-medical.json"
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
    let mut state: Session = serde_json::from_value(
        fixture["foundations"][case["foundation"].as_u64().unwrap() as usize]["session"].clone(),
    )
    .unwrap();
    if case["qualified"].as_bool().unwrap() {
        party(&mut state, &case["before"]);
    } else {
        assert_eq!(
            state
                .warriors
                .iter()
                .flat_map(|w| w.bytes.iter().copied())
                .collect::<Vec<_>>(),
            bytes(&case["before"]["party"])
        );
    }
    state
}

fn project(state: &mut Session, snapshot: &Value) {
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
            .map(|p| (p[0], p[1]))
            .collect();
    }
    combat.squad.under_tiles = words(&snapshot["under_tiles"]);
    combat.map.tiles = words(&snapshot["map"]);
    combat.order = words(&snapshot["order"]);
    combat.cursor = snapshot["cursor"].as_u64().unwrap() as usize;
    combat.round = snapshot["round"].as_i64().unwrap() as i16;
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

fn source_item_refresh(state: &mut Session, content: &Content) {
    // The actual refresh after preparation or Getup reloads selected equipment.
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
    assert_eq!(state.loaded_item_field32, Some(word(&record, 32).unwrap()));
    state.ammunition_work.item = Some(record);
}

fn source_medical_preparation(state: &mut Session, content: &Content, snapshot: &Value) {
    // Full item-use prepattack loads these two definitions; AMMO0 does not
    // override the original WEAPON700 sound WORDs at 14c944/14c960.
    for (id, key, target) in [
        (700, "weapon_work", &mut state.ammunition_work.weapon),
        (0, "ammo_work", &mut state.ammunition_work.ammo),
    ] {
        let mut record = content.items[id].clone();
        let class = word(&record, 26).unwrap().wrapping_abs();
        record[26..28].copy_from_slice(&class.to_be_bytes());
        assert_eq!(record, bytes(&snapshot[key]));
        *target = Some(record);
    }
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
        "centurions-earned-medical-{}-{label}.json",
        std::process::id()
    ));
    save(&path, state, content).unwrap();
    let restored = load(&path, content).unwrap();
    same(&restored, state);
    std::fs::remove_file(path).unwrap();
    restored
}

#[test]
fn earned_medical_full_sessions_match_native_and_resume_without_reconsumption() {
    let (content, fixture) = fixture();
    for (index, case) in fixture["cases"].as_array().unwrap().iter().enumerate() {
        let mut state = input(&fixture, case);
        state.validate(&content).unwrap();
        let initial = state.clone();
        let mut resumed = checkpoint(&state, &content, &format!("before-{index}"));
        let actor = case["actor"].as_u64().unwrap() as usize;
        let begin = Command::BeginMedicalItem {
            warrior: actor,
            slot: case["slot"].as_u64().unwrap() as usize,
        };
        if case["qualified"].as_bool().unwrap() {
            atomic(&state, &content, begin);
            continue;
        }
        let mut events = apply(begin.clone(), &mut state, &content).unwrap();
        assert_eq!(
            events,
            apply(begin.clone(), &mut resumed, &content).unwrap()
        );
        same(&state, &resumed);
        let mut expected = initial.clone();
        project(&mut expected, &case["prepared"]);
        source_item_refresh(&mut expected, &content);
        source_medical_preparation(&mut expected, &content, &case["prepared"]);
        expected.medical_continuation = Some(MedicalContinuation {
            origin: centurions_core::MedicalOrigin::OwnedNeedle,
            actor,
            item: 700,
            spell: 1401,
            power: 1,
        });
        same(&state, &expected);
        atomic(&state, &content, begin);
        atomic(&state, &content, Command::EndTurn);
        atomic(
            &state,
            &content,
            Command::SelectMedicalTarget {
                selection: MedicalTarget::Portrait { warrior: actor },
            },
        );
        resumed = checkpoint(&state, &content, &format!("pending-{index}"));
        let selection = if case["kind"] == "cancel" {
            MedicalTarget::ConfirmedCancel
        } else {
            MedicalTarget::LivingPoint {
                x: case["target_x"].as_i64().unwrap() as i16,
                y: case["target_y"].as_i64().unwrap() as i16,
            }
        };
        let command = Command::SelectMedicalTarget { selection };
        if case["kind"] == "range_reject" {
            atomic(&state, &content, command);
            project(&mut expected, &case["after"]);
            source_item_refresh(&mut expected, &content);
            source_medical_preparation(&mut expected, &content, &case["after"]);
            same(&state, &expected);
            continue;
        }
        let result = apply(command.clone(), &mut state, &content).unwrap();
        assert_eq!(
            result,
            apply(command.clone(), &mut resumed, &content).unwrap()
        );
        same(&state, &resumed);
        events.extend(result);
        let mut expected = initial;
        project(&mut expected, &case["after"]);
        source_item_refresh(&mut expected, &content);
        source_medical_preparation(&mut expected, &content, &case["after"]);
        expected.medical_continuation = None;
        same(&state, &expected);
        let native: Vec<i16> = case["use_sounds"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_i64().unwrap() as i16)
            .collect();
        assert_eq!(sounds(&events), native, "case {index}");
        let mut native_events: Vec<Event> = native
            .into_iter()
            .map(|resource_id| Event::Sound { resource_id })
            .collect();
        if let Some(damage) = case["effect_events"]
            .as_array()
            .unwrap()
            .iter()
            .find(|event| event["address"] == "1518d4")
        {
            native_events.push(Event::SpellResolved {
                caster: Some(actor as i16),
                spell: 1401,
                target: case["target"].as_i64().unwrap() as i16,
                damage: damage["d0"].as_i64().unwrap() as i16,
                defeated: false,
            });
        }
        assert_eq!(events, native_events, "ordered effects case {index}");
        atomic(&state, &content, command);
        checkpoint(&state, &content, &format!("after-{index}"));
        if index == 1 || index == 2 {
            assert!(
                state.warriors[actor]
                    .inventory()
                    .unwrap()
                    .iter()
                    .all(|item| item.id() != 700)
            );
            atomic(
                &state,
                &content,
                Command::BeginMedicalItem {
                    warrior: actor,
                    slot: 3,
                },
            );
        }
    }
}

#[test]
fn unmatched_medical_records_remain_atomic() {
    let (content, fixture) = fixture();
    let start = input(&fixture, &fixture["cases"][0]);
    for (offset, value) in [(2, 3), (10, 28), (13, 2), (14, 1)] {
        let mut state = start.clone();
        let base = 0x132 + 3 * 22;
        if offset == 2 {
            state.warriors[4].bytes[base + offset..base + offset + 2]
                .copy_from_slice(&(value as i16).to_be_bytes());
        } else {
            state.warriors[4].bytes[base + offset] = value;
        }
        atomic(
            &state,
            &content,
            Command::BeginMedicalItem {
                warrior: 4,
                slot: 3,
            },
        );
    }
    let mut unsorted = start;
    let bytes = &mut unsorted.warriors[4].bytes;
    for offset in 0..22 {
        bytes.swap(0x132 + offset, 0x132 + 3 * 22 + offset);
    }
    atomic(
        &unsorted,
        &content,
        Command::BeginMedicalItem {
            warrior: 4,
            slot: 3,
        },
    );
}
