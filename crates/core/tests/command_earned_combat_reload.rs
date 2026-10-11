use centurions_content::{Content, Warrior, word};
use centurions_core::{
    Command, Event, Session, apply,
    combat::{firearm, weapon_work::WeaponWork},
    load, save,
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
            "../../../evidence/fixtures/earned-combat-reload.json"
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
    if case["kind"] == "normalized_diagonal_attempt" {
        // Native display normalization reaches Getup's full item refresh before this command.
        state.ammunition_work.item = Some(bytes(&case["before_command"]["item_work"]));
    }
    state
}

fn project(state: &mut Session, snapshot: &Value, baseline: &Value) {
    party(state, snapshot);
    state.random.seed = snapshot["seed"].as_u64().unwrap() as u32;
    state.random.draws = snapshot["draws"].as_u64().unwrap();
    state.selected_warrior = snapshot["selected_member"].as_u64().unwrap() as usize;
    state.loaded_item_field32 = Some(snapshot["item_context"].as_i64().unwrap() as i16);
    if snapshot["seed"] != baseline["seed"] || snapshot["ammunition"] != baseline["ammunition"] {
        state.ammunition_work.item = Some(bytes(&snapshot["item_work"]));
        state.ammunition_work.weapon = Some(bytes(&snapshot["weapon_work"]));
        state.ammunition_work.ammo = Some(bytes(&snapshot["ammo_work"]));
    }
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
        "centurions-earned-combat-reload-{}-{label}.json",
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
        include_str!("../../../evidence/fixtures/earned-combat-reload-receipt-12.json"),
        include_str!("../../../evidence/fixtures/earned-combat-reload-receipt-1.json"),
    ];
    serde_json::from_str(receipts[index]).unwrap()
}

fn replay_historical_prefix(fixture: &Value, case: &Value, content: &Content) -> Option<Session> {
    let index = case["foundation"].as_u64().unwrap() as usize;
    let receipt = receipt(index);
    assert_eq!(
        receipt["complete_state_and_every_ordered_event_replay"],
        true
    );
    assert_eq!(receipt["full_input_save_reload"], true);
    assert_eq!(receipt["field_grants_or_random_trials"], false);
    let commands = receipt["commands"].as_array().unwrap();
    let ordered_events = receipt["ordered_events"].as_array().unwrap();
    assert_eq!(commands.len(), if index == 0 { 148 } else { 168 });
    assert_eq!(ordered_events.len(), commands.len());
    let mut state: Session = serde_json::from_value(receipt["starting_session"].clone()).unwrap();
    state.validate(content).unwrap();
    let mut resumed = checkpoint(&state, content, &format!("prefix-start-{index}"));
    for (step, (command, recorded)) in commands.iter().zip(ordered_events).enumerate() {
        let command: Command = serde_json::from_value(command.clone()).unwrap();
        let expected: Vec<Event> = serde_json::from_value(recorded.clone()).unwrap();
        let before = state.clone();
        let result = apply(command.clone(), &mut state, content);
        if let Err(error) = result {
            assert_eq!(index, 1);
            assert_eq!(step, 149);
            assert!(matches!(&command, Command::ShootEnemy { body: 15 }));
            assert_eq!(
                error.to_string(),
                "invalid session: enemy is outside firearm range or line of sight"
            );
            same(&state, &before);
            same(&resumed, &before);
            let combat = state.combat.as_ref().unwrap();
            let start = combat.squad.positions[1];
            let end = combat.enemies.positions[5];
            let dx = f32::from(start.0 - end.0);
            let dy = f32::from(start.1 - end.1);
            assert_eq!((dx * dx + dy * dy).sqrt().trunc() as i16, 9);
            assert_eq!(word(&state.warriors[1].bytes, 0xe4).unwrap(), 1);
            let mission = content
                .map(state.exploration.as_ref().unwrap().map)
                .unwrap();
            let prepared = firearm::prepare(
                &state.warriors[1],
                &content.items[1],
                &content.items[200],
                &mission.combat.as_ref().unwrap().attacks,
            )
            .unwrap();
            assert_eq!(
                centurions_core::combat::player_psi::range(&prepared.packet, prepared.shots)
                    .unwrap(),
                8
            );
            assert_eq!(
                apply(command, &mut resumed, content)
                    .unwrap_err()
                    .to_string(),
                error.to_string()
            );
            same(&resumed, &before);
            return None;
        }
        let events = result.unwrap();
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
    let mut expected = input(fixture, case);
    // The seed12 receipt's last full pair writer is its real Shoot at command145.
    let mut weapon = content.items[165].clone();
    let ammo = content.items[280].clone();
    for offset in [12, 14] {
        let value = word(&ammo, offset).unwrap();
        if value != 0 {
            weapon[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
        }
    }
    expected.ammunition_work.weapon = Some(weapon);
    expected.ammunition_work.ammo = Some(ammo);
    same(&state, &expected);
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
    Some(state)
}

#[test]
fn historical_extra_cell_shot_rejects_atomically_and_valid_prefix_still_replays() {
    let (content, fixture) = fixture();
    for case in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["kind"] == "normalized_diagonal_attempt")
    {
        let result = replay_historical_prefix(&fixture, case, &content);
        assert_eq!(result.is_some(), case["foundation"] == 0);
    }
}

#[test]
fn silent_first_refit_matches_native_warrior_stock_and_cache_scalars() {
    let (content, fixture) = fixture();
    for case in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["kind"] == "normalized_diagonal_attempt")
    {
        let mut state = input(&fixture, case);
        party(&mut state, &case["before_command"]);
        let original = state.clone();
        let result =
            firearm::refit_earned_secondary(&state.warriors[4], &content, state.ammunition[80])
                .unwrap();
        same(&state, &original);
        state.warriors[4] = result.warrior;
        state.ammunition[80] = result.reserve;
        let mut expected = original;
        party(&mut expected, &case["after"]);
        expected.ammunition = bytes(&case["after"]["ammunition"])
            .as_chunks::<4>()
            .0
            .iter()
            .map(|v| i32::from_be_bytes(*v))
            .collect();
        same(&state, &expected);
        assert_eq!(result.loaded_item_field32, 16);
        assert_eq!(result.weapon_work, WeaponWork::zero());
        assert_eq!(result.reserve, 2);
        assert_eq!(
            word(&state.warriors[4].inventory().unwrap()[0].0, 2).unwrap(),
            1
        );
    }
}

#[test]
fn historical_occupied_cell_refill_matches_complete_native_caller_and_save_resume() {
    let (content, fixture) = fixture();
    for (index, case) in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["kind"] == "normalized_diagonal_attempt")
        .enumerate()
    {
        let mut state = input(&fixture, case);
        let initial = state.clone();
        let mut before = initial.clone();
        project(&mut before, &case["before"], &case["before"]);
        same(&initial, &before);
        let mut resumed = checkpoint(&state, &content, &format!("refill-before-{index}"));
        let command = Command::Move { dx: 1, dy: 1 };
        let events = apply(command.clone(), &mut state, &content).unwrap();
        assert_eq!(
            events,
            apply(command.clone(), &mut resumed, &content).unwrap()
        );
        same(&state, &resumed);
        let mut expected = initial.clone();
        project(&mut expected, &case["after"], &case["before"]);
        same(&state, &expected);
        assert!(events.is_empty());
        assert!(sounds(&events).is_empty());
        assert!(case["sounds"].as_array().unwrap().is_empty());
        assert!(case["rng"].as_array().unwrap().is_empty());
        assert_eq!(state.random, initial.random);
        assert_eq!(
            state.combat.as_ref().unwrap().attack_packet,
            initial.combat.as_ref().unwrap().attack_packet
        );
        assert_eq!(
            state.combat.as_ref().unwrap().attack_identity,
            initial.combat.as_ref().unwrap().attack_identity
        );
        assert_eq!(state.attack_count, 0);
        assert_eq!(state.loaded_item_field32, Some(16));
        assert_eq!(state.ammunition[80], 2);
        let restored = checkpoint(&state, &content, &format!("refill-after-{index}"));
        same(&state, &restored);
        atomic(&state, &content, command);
        atomic(&state, &content, Command::Reload { warrior: 4 });
    }
}

#[test]
fn last_stock_control_matches_complete_native_caller_and_save_resume() {
    let (content, original_fixture) = fixture();
    let bounds: Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/earned-combat-reload-bounds.json"
    ))
    .unwrap();
    let case = bounds["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "normalized_seed12_last_stock_control")
        .unwrap();
    let original_case = original_fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["foundation"] == 0 && case["kind"] == "normalized_diagonal_attempt")
        .unwrap();
    let foundation = input(&bounds, case);
    same(&foundation, &input(&original_fixture, original_case));
    assert_eq!(foundation.ammunition[80], 3);
    assert_eq!(case["quantity_control"]["controlled_quantity"], 1);
    assert_eq!(case["quantity_control"]["width_bits"], 32);
    assert_eq!(case["quantity_control"]["index"], 80);
    assert_eq!(
        case["quantity_control"]["all_other_whole_input_bytes_unchanged"],
        true
    );
    let mut state = foundation;
    state.ammunition[80] = 1;
    let initial = state.clone();
    let mut before = initial.clone();
    project(&mut before, &case["before"], &case["before"]);
    same(&initial, &before);
    let mut resumed = checkpoint(&state, &content, "last-stock-before");
    let command = Command::Move { dx: 1, dy: 1 };
    let events = apply(command.clone(), &mut state, &content).unwrap();
    assert_eq!(events, apply(command, &mut resumed, &content).unwrap());
    same(&state, &resumed);
    let mut expected = initial.clone();
    project(&mut expected, &case["after"], &case["before"]);
    same(&state, &expected);
    assert!(events.is_empty());
    assert!(case["sounds"].as_array().unwrap().is_empty());
    assert!(case["rng"].as_array().unwrap().is_empty());
    assert_eq!(state.random, initial.random);
    assert_eq!(state.ammunition[80], 0);
    assert_eq!(
        word(&state.warriors[4].inventory().unwrap()[0].0, 2).unwrap(),
        1
    );
    assert_eq!(state.attack_count, 0);
    assert_eq!(state.loaded_item_field32, Some(16));
    assert_eq!(state.weapon_work, WeaponWork::zero());
    assert_eq!(state.attack_packet, initial.attack_packet);
    assert_eq!(state.attack_identity, initial.attack_identity);
    for (key, id) in [("item_work", 165), ("weapon_work", 165), ("ammo_work", 280)] {
        assert_eq!(bytes(&case["after"][key]), *content.items.get(id).unwrap());
    }
    let restored = checkpoint(&state, &content, "last-stock-after");
    same(&state, &restored);
    atomic(&state, &content, Command::Move { dx: 1, dy: 1 });
    atomic(&state, &content, Command::Reload { warrior: 4 });
}

#[test]
fn reserve_two_uses_the_source_supported_full_load_interval() {
    let (content, fixture) = fixture();
    for case in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["kind"] == "normalized_diagonal_attempt")
    {
        let mut state = input(&fixture, case);
        state.ammunition[80] = 2;
        let initial = state.clone();
        let mut resumed = checkpoint(&state, &content, "stock-two-before");
        let command = Command::Move { dx: 1, dy: 1 };
        let events = apply(command.clone(), &mut state, &content).unwrap();
        assert_eq!(events, apply(command, &mut resumed, &content).unwrap());
        same(&state, &resumed);
        let mut expected = initial.clone();
        project(&mut expected, &case["after"], &case["before"]);
        expected.ammunition[80] = 1;
        same(&state, &expected);
        assert!(events.is_empty());
        assert_eq!(state.random, initial.random);
        let restored = checkpoint(&state, &content, "stock-two-after");
        same(&state, &restored);
    }
}

#[test]
fn unsupported_refill_instances_and_occupied_cell_attempts_preserve_whole_session() {
    let (content, fixture) = fixture();
    for case in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["kind"] == "normalized_diagonal_attempt")
    {
        let original = input(&fixture, case);
        let command = Command::Move { dx: 1, dy: 1 };
        let actor = 4;
        let combat = original.combat.as_ref().unwrap();
        let position = combat.squad.positions[actor];
        let destination = centurions_content::mission::OpeningMission::index(
            i32::from(position.0 + combat.map.shift.0 + 6),
            i32::from(position.1 + combat.map.shift.1 + 6),
        )
        .unwrap();
        let target = combat.map.tiles[destination].wrapping_abs() as usize - 10;
        for (offset, value) in [
            (0x132 + 3, 1),
            (0x132 + 5, 25),
            (0x132 + 7, 2),
            (0x132 + 9, 0),
            (0x132 + 10, 24),
            (0x132 + 13, 4),
            (0x132 + 18, 11),
            (0x132 + 20, 1),
            (0x9dd, 0),
        ] {
            let mut invalid = original.clone();
            invalid.warriors[actor].bytes[offset] = value;
            let before = invalid.clone();
            assert!(
                firearm::refit_earned_secondary(&invalid.warriors[actor], &content, 3).is_err()
            );
            same(&invalid, &before);
            if matches!(offset, 0x13c | 0x9dd) {
                atomic(&invalid, &content, command.clone());
            }
        }
        for reserve in [-1, 0, 4, 32768] {
            let mut invalid = original.clone();
            invalid.ammunition[80] = reserve;
            assert!(
                firearm::refit_earned_secondary(&invalid.warriors[actor], &content, reserve)
                    .is_err()
            );
            if reserve < 0 {
                atomic(&invalid, &content, command.clone());
            }
        }
        for (offset, value) in [(0x34, 3_i16), (0x40, 1), (0x90, 1), (0x50, 0)] {
            let mut invalid = original.clone();
            invalid.warriors[actor].bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
            atomic(&invalid, &content, command.clone());
        }
        let mut friendly = original.clone();
        friendly.combat.as_mut().unwrap().enemies.records[target].bytes[0xe..0x10]
            .copy_from_slice(&0_i16.to_be_bytes());
        atomic(&friendly, &content, command.clone());
        // HP0 is admitted by the original occupied-cell refill path; the four
        // controlled command-ammunition cases preserve its native zero HP.
        let mut inconsistent = original.clone();
        inconsistent.combat.as_mut().unwrap().enemies.positions[target].0 += 1;
        atomic(&inconsistent, &content, command.clone());
        let mut inactive = original.clone();
        let cursor = inactive.combat.as_ref().unwrap().cursor;
        inactive.combat.as_mut().unwrap().order[cursor] = target as i16 + 10;
        atomic(&inactive, &content, command);
        atomic(&original, &content, Command::Reload { warrior: 4 });
    }
}

#[test]
fn raw_offscreen_native_control_is_separate_from_normalized_public_movement() {
    let (_, fixture) = fixture();
    let case = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["kind"] == "raw_offscreen_scroll")
        .unwrap();
    for key in [
        "party",
        "records",
        "all_records",
        "map",
        "order",
        "fields",
        "used",
        "ammunition",
    ] {
        assert_eq!(case["before"][key], case["after"][key], "raw scroll {key}");
    }
    assert!(case["rng"].as_array().unwrap().is_empty());
    assert!(case["sounds"].as_array().unwrap().is_empty());
}
