use centurions_content::{Content, InventoryRecord, Warrior, word};
use centurions_core::{
    Command, Event, MedicalContinuation, MedicalTarget, Session, apply,
    combat::weapon_work::WeaponWork, load, save,
};
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
        .map(|v| i16::from_be_bytes(*v))
        .collect()
}

fn fixture() -> (Content, Value) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    (
        Content::load(
            &root.join("NewCenturions"),
            &root.join("analysis/extracted"),
        )
        .unwrap(),
        serde_json::from_str(include_str!("../../../evidence/fixtures/combat-aid.json")).unwrap(),
    )
}

fn project(state: &mut Session, snapshot: &Value) {
    for (warrior, record) in state
        .warriors
        .iter_mut()
        .zip(bytes(&snapshot["party"]).as_chunks::<3138>().0)
    {
        warrior.bytes = Warrior::read(record).unwrap().bytes;
    }
    for (target, key) in [
        (&mut state.crate_items, "crate"),
        (&mut state.booty_items, "loot"),
    ] {
        *target = bytes(&snapshot[key])
            .as_chunks::<22>()
            .0
            .iter()
            .map(|v| InventoryRecord(*v))
            .collect();
    }
    state.ammunition = bytes(&snapshot["ammunition"])
        .as_chunks::<4>()
        .0
        .iter()
        .map(|v| i32::from_be_bytes(*v))
        .collect();
    state.random.seed = snapshot["seed"].as_u64().unwrap() as u32;
    state.random.draws = snapshot["draws"].as_u64().unwrap();
    state.selected_warrior = snapshot["selected_member"].as_u64().unwrap() as usize;
    state.loaded_item_field32 = Some(snapshot["item_context"].as_i64().unwrap() as i16);
    state.attack_count = snapshot["count"].as_i64().unwrap() as i16;
    state.attack_packet = Some(bytes(&snapshot["packet"]));
    state.attack_identity = Some(words(&snapshot["identity"]).try_into().unwrap());
    state.psi_memory = Some(bytes(&snapshot["memory"]));
    state.party_buffs = bytes(&snapshot["party_buffs"]);
    state
        .party_mask
        .copy_from_slice(&words(&snapshot["mask"])[..6]);
    state.bandage_ready = Some(snapshot["aid_gate"].as_i64().unwrap() as i16);
    let weapon = bytes(&snapshot["weapon_work"]);
    let ammo = bytes(&snapshot["ammo_work"]);
    state.weapon_work = WeaponWork {
        sound: Some(word(&weapon, 14).unwrap()),
        weapon_armor_filter: Some(word(&weapon, 136).unwrap()),
        ammunition_armor_filter: Some(word(&ammo, 136).unwrap()),
    };
    let arena = state.combat.as_mut().unwrap();
    for (record, native) in arena
        .enemies
        .records
        .iter_mut()
        .zip(bytes(&snapshot["records"]).as_chunks::<288>().0)
    {
        record.bytes = native.to_vec();
    }
    for (target, key) in [
        (&mut arena.squad.positions, "party_positions"),
        (&mut arena.enemies.positions, "monster_positions"),
    ] {
        *target = words(&snapshot[key])
            .as_chunks::<2>()
            .0
            .iter()
            .map(|p| (p[0], p[1]))
            .collect();
    }
    arena.squad.under_tiles = words(&snapshot["under_tiles"]);
    arena.map.tiles = words(&snapshot["map"]);
    arena.order = words(&snapshot["order"]);
    arena.cursor = snapshot["cursor"].as_u64().unwrap() as usize;
    arena.round = snapshot["round"].as_i64().unwrap() as i16;
    arena.round_scripts.counter = arena.round as i8;
    arena.round_scripts.slots = words(&snapshot["round_slots"]).try_into().unwrap();
    arena.spell_fields = bytes(&snapshot["fields"]);
    arena.field_used = bytes(&snapshot["used"]);
    arena.unavailable = snapshot["unavailable"].as_u64().unwrap() as i8;
    arena.attack_packet = state.attack_packet.clone();
    arena.attack_identity = state.attack_identity;
    arena.weapon_work = state.weapon_work;
    let mut health = centurions_core::story_health::context(
        &state.warriors,
        state.selected_warrior,
        state.health_context.as_ref(),
    );
    health.unavailable = arena.unavailable;
    state.health_context = Some(health);
}

fn input(fixture: &Value, snapshot: &Value) -> Session {
    let foundation = fixture["foundations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["session"]["random"]["seed"] == snapshot["seed"])
        .unwrap();
    let mut state = serde_json::from_value(foundation["session"].clone()).unwrap();
    project(&mut state, snapshot);
    state
}

fn source_item_refresh(state: &mut Session, content: &Content) {
    // combatupdate2 loads NPC WORD4e, or party ca/e4 selected by BYTE9dd.
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
    // Original item-use prepattack loads WEAPON700 and AMMO0. Zero ammo
    // sound WORDs skip both raw weapon overrides at 14c944/14c960.
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

fn native_sounds(case: &Value) -> Vec<i16> {
    case["sounds"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_i64().unwrap() as i16)
        .collect()
}

fn checkpoint(state: &Session, content: &Content, label: &str) -> Session {
    let path = std::env::temp_dir().join(format!(
        "centurions-medical-{}-{label}.json",
        std::process::id()
    ));
    save(&path, state, content).unwrap();
    let restored = load(&path, content).unwrap();
    same(&restored, state);
    std::fs::remove_file(path).unwrap();
    restored
}

#[test]
fn stop_bleeding_and_cancel_match_native_full_session_rounds_and_ordered_sounds() {
    let (content, fixture) = fixture();
    for (index, case) in fixture["stop_cases"].as_array().unwrap().iter().enumerate() {
        let mut state = input(&fixture, &case["before"]);
        state.validate(&content).unwrap();
        let mut resumed = checkpoint(&state, &content, &format!("stop-before-{index}"));
        let target = case["selected_target"].as_i64().unwrap();
        let command = Command::StopBleeding {
            warrior: usize::try_from(target).ok(),
        };
        if case["kind"] == "gate_zero" {
            atomic(&state, &content, command);
            continue;
        }
        let before = state.clone();
        let events = apply(command.clone(), &mut state, &content).unwrap();
        assert_eq!(events, apply(command, &mut resumed, &content).unwrap());
        same(&state, &resumed);
        let mut expected = before;
        project(&mut expected, &case["after"]);
        source_item_refresh(&mut expected, &content);
        let effect = bytes(&case["after"]["packet"])[18] as i8;
        expected.health_context.as_mut().unwrap().effect = effect;
        assert_eq!(state.health_context.as_ref().unwrap().effect, effect);
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
        same(&state, &expected);
        assert_eq!(sounds(&events), native_sounds(case), "stop case {index}");
        checkpoint(&state, &content, &format!("stop-after-{index}"));
    }
}

#[test]
fn consumed_needle_pending_targets_and_cancellation_match_native_and_reload_without_reconsumption()
{
    let (content, fixture) = fixture();
    for (index, case) in fixture["stim_cases"].as_array().unwrap().iter().enumerate() {
        let mut state = input(&fixture, &case["before"]);
        let initial = state.clone();
        let mut resumed = checkpoint(&state, &content, &format!("needle-before-{index}"));
        let begin = Command::BeginMedicalItem {
            warrior: 4,
            slot: case["slot"].as_u64().unwrap() as usize,
        };
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
        expected.health_context.as_mut().unwrap().effect =
            expected.attack_packet.as_ref().unwrap()[18] as i8;
        expected.medical_continuation = Some(MedicalContinuation {
            origin: centurions_core::MedicalOrigin::OwnedNeedle,
            actor: 4,
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
                selection: MedicalTarget::Portrait { warrior: 6 },
            },
        );
        atomic(
            &state,
            &content,
            Command::SelectMedicalTarget {
                selection: MedicalTarget::Portrait { warrior: 4 },
            },
        );
        atomic(
            &state,
            &content,
            Command::SelectMedicalTarget {
                selection: MedicalTarget::LivingPoint { x: -90, y: -90 },
            },
        );
        resumed = checkpoint(&state, &content, &format!("needle-pending-{index}"));
        let target = case["target"].as_i64().unwrap() as usize;
        let selection = if case["kind"] == "actual_stim_needle_living" {
            let (x, y) = state.combat.as_ref().unwrap().squad.positions[target];
            MedicalTarget::LivingPoint { x, y }
        } else if case["kind"] == "actual_stim_needle_confirmed_cancel" {
            MedicalTarget::ConfirmedCancel
        } else {
            MedicalTarget::Portrait { warrior: target }
        };
        let command = Command::SelectMedicalTarget { selection };
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
        expected.health_context.as_mut().unwrap().effect =
            expected.attack_packet.as_ref().unwrap()[18] as i8;
        same(&state, &expected);
        assert_eq!(sounds(&events), native_sounds(case), "needle case {index}");
        atomic(&state, &content, command);
        checkpoint(&state, &content, &format!("needle-after-{index}"));
    }
}

#[test]
fn medical_admission_and_unknown_legacy_marker_leave_complete_session_and_rng_unchanged() {
    let (content, fixture) = fixture();
    for case in fixture["admission_cases"].as_array().unwrap() {
        let state = input(&fixture, &case["before"]);
        atomic(
            &state,
            &content,
            Command::BeginMedicalItem {
                warrior: case["member"].as_u64().unwrap() as usize,
                slot: case["slot"].as_u64().unwrap() as usize,
            },
        );
    }
    let mut state = input(&fixture, &fixture["stop_cases"][0]["before"]);
    state.bandage_ready = None;
    let restored = checkpoint(&state, &content, "legacy-unknown");
    atomic(
        &restored,
        &content,
        Command::StopBleeding { warrior: Some(3) },
    );
    let mut ready = restored;
    apply(Command::EndTurn, &mut ready, &content).unwrap();
    assert_eq!(ready.bandage_ready, Some(1));
}

#[test]
fn existing_v3_prepared_needles_load_with_their_original_origin_and_costs() {
    let (content, fixture) = fixture();
    let case = &fixture["stim_cases"][0];
    let mut state = input(&fixture, &case["before"]);
    apply(
        Command::BeginMedicalItem {
            warrior: 4,
            slot: case["slot"].as_u64().unwrap() as usize,
        },
        &mut state,
        &content,
    )
    .unwrap();
    let path = std::env::temp_dir().join(format!(
        "centurions-existing-medical-{}.json",
        std::process::id()
    ));
    save(&path, &state, &content).unwrap();
    let mut old: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    let session = old["session"].as_object_mut().unwrap();
    session.remove("psi_menu_choices");
    session["medical_continuation"]
        .as_object_mut()
        .unwrap()
        .remove("origin");
    std::fs::write(&path, serde_json::to_vec(&old).unwrap()).unwrap();
    let restored = load(&path, &content).unwrap();
    same(&restored, &state);
    std::fs::remove_file(path).unwrap();
}
