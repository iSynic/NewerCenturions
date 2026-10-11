use centurions_content::Content;
use centurions_core::{Command, Event, Phase, RandomState, Session, apply, load, save};
use serde_json::Value;

fn fixture(name: &str) -> Value {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    serde_json::from_str(
        &std::fs::read_to_string(root.join("evidence/fixtures").join(name)).unwrap(),
    )
    .unwrap()
}
fn content() -> Content {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap()
}
fn bytes(value: &Value) -> Vec<u8> {
    value
        .as_str()
        .unwrap()
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
fn words(value: &Value) -> Vec<i16> {
    bytes(value)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| i16::from_be_bytes(*p))
        .collect()
}
fn positions(value: &Value) -> Vec<(i16, i16)> {
    words(value)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| (p[0], p[1]))
        .collect()
}
fn copy_party(state: &mut Session, snapshot: &Value) {
    for (w, record) in state
        .warriors
        .iter_mut()
        .zip(bytes(&snapshot["party"]).as_chunks::<3138>().0)
    {
        w.bytes = record.to_vec();
    }
}
fn copy_stock(state: &mut Session, snapshot: &Value) {
    state.ammunition = bytes(&snapshot["ammunition"])
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| i32::from_be_bytes(*p))
        .collect();
    state.loaded_item_field32 = Some(snapshot["item_context"].as_i64().unwrap() as i16);
    let weapon = bytes(&snapshot["weapon"]);
    let ammo = bytes(&snapshot["ammo"]);
    state.weapon_work = centurions_core::combat::weapon_work::WeaponWork {
        sound: Some(centurions_content::word(&weapon, 0x0e).unwrap()),
        weapon_armor_filter: Some(centurions_content::word(&weapon, 0x88).unwrap()),
        ammunition_armor_filter: Some(centurions_content::word(&ammo, 0x88).unwrap()),
    };
    if let Some(combat) = state.combat.as_mut() {
        combat.weapon_work = state.weapon_work;
    }
    state.random.seed = snapshot["rng_seed"].as_u64().unwrap() as u32;
    state.random.draws = snapshot["rng_draws"].as_u64().unwrap();
}
fn sync_health(state: &mut Session) {
    if let Some(c) = state.combat.as_ref() {
        let mut health = centurions_core::story_health::context(
            &state.warriors,
            state.selected_warrior,
            state.health_context.as_ref(),
        );
        health.unavailable = c.unavailable;
        state.health_context = Some(health);
    }
}
fn assert_session(actual: &Session, expected: &Session) {
    let a = serde_json::to_value(actual).unwrap();
    let e = serde_json::to_value(expected).unwrap();
    for (key, value) in e.as_object().unwrap() {
        assert!(a[key] == *value, "Session field {key} differs");
    }
}
fn manual_state(f: &Value, case: &Value) -> Session {
    let mut state: Session = serde_json::from_value(
        f["foundations"][case["source"].as_u64().unwrap() as usize]["session"].clone(),
    )
    .unwrap();
    copy_party(&mut state, &case["operation_before"]);
    copy_stock(&mut state, &case["operation_before"]);
    state.attack_packet = Some(bytes(&case["operation_before"]["packet"]));
    state.attack_identity = Some(
        words(&case["operation_before"]["identity"])
            .try_into()
            .unwrap(),
    );
    state.attack_count = case["operation_before"]["attack_count"].as_i64().unwrap() as i16;
    state.random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
    if case["combat"] == 1 {
        let p = fixture("player-psi.json");
        let mut combat: centurions_core::combat::Combat =
            serde_json::from_value(p["route_foundation"]["combat"].clone()).unwrap();
        combat.order[1] = 0;
        combat.cursor = 1;
        combat.attack_packet = state.attack_packet.clone();
        combat.attack_identity = state.attack_identity;
        combat.weapon_work = state.weapon_work;
        state.combat = Some(combat);
        state.phase = Phase::Combat;
        sync_health(&mut state);
    }
    state
}
fn command(case: &Value) -> Command {
    Command::FitAmmunition {
        warrior: 0,
        slot: case["inventory_slot"].as_u64().unwrap() as usize,
        ammunition: 203,
    }
}
fn sound_events(snapshot: &Value, previous: &Value) -> Vec<Event> {
    snapshot["sounds"]
        .as_array()
        .unwrap()
        .iter()
        .skip(previous["sounds"].as_array().unwrap().len())
        .map(|s| Event::Sound {
            resource_id: s.as_i64().unwrap() as i16,
        })
        .collect()
}

#[test]
fn actual_empty_crate_rifle_is_collected_equipped_and_fitted_without_changing_other_state() {
    let content = content();
    let f = fixture("empty-rifle-ammunition.json");
    for case in f["wear_cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["combat"] == 0)
    {
        let mut state: Session =
            serde_json::from_value(f["foundations"][0]["session"].clone()).unwrap();
        copy_party(&mut state, &case["before"]);
        copy_stock(&mut state, &case["before"]);
        let before = state.clone();
        let slot = state
            .crate_items
            .iter()
            .position(|item| item.id() == 38)
            .unwrap();
        let count = state.warriors[0].inventory().unwrap().len();
        let events = apply(
            Command::TakeCrate { slot, warrior: 0 },
            &mut state,
            &content,
        )
        .unwrap();
        let mut expected = before;
        copy_party(&mut expected, &case["picked"]);
        copy_stock(&mut expected, &case["picked"]);
        expected.ammunition_work.item = Some(bytes(&case["picked"]["item_work"]));
        expected.crate_items.remove(slot);
        expected.crate_items.retain(|r| r.id() != 0);
        expected.crate_items.resize(
            state.crate_items.len(),
            centurions_content::InventoryRecord([0; 22]),
        );
        assert_session(&state, &expected);
        assert_eq!(events, sound_events(&case["picked"], &case["before"]));
        let before_wear = state.clone();
        let events = apply(
            Command::EquipItem {
                warrior: 0,
                slot: count,
            },
            &mut state,
            &content,
        )
        .unwrap();
        let mut expected = before_wear;
        copy_party(&mut expected, &case["compacted"]);
        copy_stock(&mut expected, &case["compacted"]);
        expected.ammunition_work.item = Some(bytes(&case["compacted"]["item_work"]));
        let mut health = centurions_core::story_health::context(
            &expected.warriors,
            expected.selected_warrior,
            expected.health_context.as_ref(),
        );
        health.affected = words(&case["compacted"]["markers"])[..6]
            .try_into()
            .unwrap();
        expected.health_context = Some(health);
        assert_session(&state, &expected);
        assert_eq!(events, sound_events(&case["compacted"], &case["picked"]));
        let manual = f["manual_cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| {
                c["source"] == 0
                    && c["combat"] == 0
                    && c["seed_before"] == case["seed_before"]
                    && c["reserve203"] == 12
            })
            .unwrap();
        // Reserve12 is the controlled native manual-case input, independent of the real pickup stock.
        state.ammunition = bytes(&manual["operation_before"]["ammunition"])
            .as_chunks::<4>()
            .0
            .iter()
            .map(|b| i32::from_be_bytes(*b))
            .collect();
        let before_fit = state.clone();
        let events = apply(command(manual), &mut state, &content).unwrap();
        let mut expected = before_fit;
        copy_party(&mut expected, &manual["after"]);
        copy_stock(&mut expected, &manual["after"]);
        expected.ammunition_work.item = Some(bytes(&manual["after"]["item_work"]));
        expected.ammunition_work.ammo = Some(bytes(&manual["after"]["ammo"]));
        assert_session(&state, &expected);
        assert_eq!(
            events,
            sound_events(&manual["after"], &manual["operation_before"])
        );
    }
}

#[test]
fn manual_selection_matches_native_whole_session_order_and_resume() {
    let content = content();
    let f = fixture("empty-rifle-ammunition.json");
    for (index, case) in f["manual_cases"].as_array().unwrap().iter().enumerate() {
        let mut state = manual_state(&f, case);
        let before = state.clone();
        if case["reserve203"] == 0 {
            assert!(apply(command(case), &mut state, &content).is_err());
            assert_session(&state, &before);
            continue;
        }
        let path = std::env::temp_dir().join(format!(
            "centurions-fit-{}-{index}.json",
            std::process::id()
        ));
        save(&path, &state, &content).unwrap();
        let mut resumed = load(&path, &content).unwrap();
        let events = apply(command(case), &mut state, &content).unwrap();
        assert_eq!(
            events,
            apply(command(case), &mut resumed, &content).unwrap()
        );
        assert_session(&state, &resumed);
        let mut expected = before;
        copy_party(&mut expected, &case["after"]);
        copy_stock(&mut expected, &case["after"]);
        expected.ammunition_work.item = Some(bytes(&case["after"]["item_work"]));
        expected.ammunition_work.ammo = Some(bytes(&case["after"]["ammo"]));
        sync_health(&mut expected);
        assert_session(&state, &expected);
        assert_eq!(
            events,
            sound_events(&case["after"], &case["operation_before"])
        );
        save(&path, &state, &content).unwrap();
        assert_session(&state, &load(&path, &content).unwrap());
        std::fs::remove_file(path).unwrap();
    }
}

#[test]
fn fitting_establishes_only_ammunition_context_and_preserves_unknown_weapon_history() {
    use centurions_core::combat::weapon_work::WeaponWork;
    let content = content();
    let f = fixture("empty-rifle-ammunition.json");
    let case = f["manual_cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["combat"] == 0 && case["reserve203"] == 12)
        .unwrap();
    let mut state = manual_state(&f, case);
    state.weapon_work = WeaponWork::default();
    apply(command(case), &mut state, &content).unwrap();
    assert_eq!(
        state.weapon_work,
        WeaponWork {
            sound: None,
            weapon_armor_filter: None,
            ammunition_armor_filter: Some(0),
        }
    );
    let path = std::env::temp_dir().join(format!(
        "centurions-partial-work-{}.json",
        std::process::id()
    ));
    save(&path, &state, &content).unwrap();
    assert_session(&state, &load(&path, &content).unwrap());
    std::fs::remove_file(path).unwrap();
}

fn advance_state(f: &Value, case: &Value) -> Session {
    let mut state: Session = serde_json::from_value(f["foundation"]["session"].clone()).unwrap();
    copy_party(&mut state, &case["before"]);
    copy_stock(&mut state, &case["before"]);
    state.psi_memory = Some(bytes(&case["before"]["memory"]));
    state.party_buffs = bytes(&case["before"]["party_buffs"]);
    let p = fixture("player-psi.json");
    state.combat = Some(serde_json::from_value(p["route_foundation"]["combat"].clone()).unwrap());
    state.phase = Phase::Combat;
    copy_arena(&mut state, &case["before"]);
    sync_health(&mut state);
    state
}
fn copy_arena(state: &mut Session, snapshot: &Value) {
    let c = state.combat.as_mut().unwrap();
    c.map.shift = (0, 0);
    c.map.tiles = words(&snapshot["map"]);
    c.squad.positions = positions(&snapshot["party_positions"]);
    c.squad.under_tiles = vec![1155; 6];
    c.enemies.records = bytes(&snapshot["target"])
        .as_chunks::<288>()
        .0
        .iter()
        .map(|p| centurions_content::combat::MonsterRecord { bytes: p.to_vec() })
        .collect();
    c.enemies.positions = positions(&snapshot["monster_positions"]);
    c.order = words(&snapshot["order"]);
    c.cursor = snapshot["cursor"].as_u64().unwrap() as usize;
    c.round = snapshot["round"].as_i64().unwrap() as i16;
    c.unavailable = snapshot["unavailable"].as_i64().unwrap() as i8;
    c.spell_fields = bytes(&snapshot["fields"]);
    c.field_used = bytes(&snapshot["used"]);
    c.attack_packet = Some(bytes(&snapshot["packet"]));
    c.attack_identity = Some(words(&snapshot["identity"]).try_into().unwrap());
    c.weapon_work = state.weapon_work;
    state.attack_packet = c.attack_packet.clone();
    state.attack_identity = c.attack_identity;
    state.attack_count = snapshot["attack_count"].as_i64().unwrap() as i16;
}

#[test]
fn low_ap_selection_matches_complete_native_automatic_initiative_and_reload() {
    let content = content();
    let f = fixture("empty-rifle-advance.json");
    for (index, case) in f["cases"].as_array().unwrap().iter().enumerate() {
        let mut state = advance_state(&f, case);
        let before = state.clone();
        let path = std::env::temp_dir().join(format!(
            "centurions-fit-advance-{}-{index}.json",
            std::process::id()
        ));
        save(&path, &state, &content).unwrap();
        let mut resumed = load(&path, &content).unwrap();
        let events = apply(command(case), &mut state, &content).unwrap();
        assert_eq!(
            apply(command(case), &mut resumed, &content).unwrap(),
            events
        );
        assert_session(&state, &resumed);
        let mut expected = before;
        copy_party(&mut expected, &case["after"]);
        copy_stock(&mut expected, &case["after"]);
        expected.ammunition_work.item = Some(bytes(&case["after"]["item_work"]));
        expected.ammunition_work.ammo = Some(bytes(&case["after"]["ammo"]));
        copy_arena(&mut expected, &case["after"]);
        expected.bandage_ready = Some(1);
        sync_health(&mut expected);
        assert_session(&state, &expected);
        let mut expected_events = sound_events(&case["after"], &case["before"]);
        expected_events.push(Event::TurnStarted { body: 10, round: 1 });
        assert_eq!(events, expected_events);
        save(&path, &state, &content).unwrap();
        assert_session(&state, &load(&path, &content).unwrap());
        std::fs::remove_file(path).unwrap();
    }
}

#[test]
fn unsupported_later_turn_and_invalid_choices_preserve_every_session_field() {
    let content = content();
    let f = fixture("empty-rifle-advance.json");
    for case in f["cases"].as_array().unwrap() {
        let mut state = advance_state(&f, case);
        let monster = &mut state.combat.as_mut().unwrap().enemies.records[0].bytes;
        monster[0x5e..0x60].copy_from_slice(&32767_i16.to_be_bytes());
        monster[0x5c..0x5e].copy_from_slice(&100_i16.to_be_bytes());
        let before = state.clone();
        assert!(apply(command(case), &mut state, &content).is_err());
        assert_session(&state, &before);
    }
    for command in [
        Command::FitAmmunition {
            warrior: 1,
            slot: 0,
            ammunition: 203,
        },
        Command::FitAmmunition {
            warrior: 0,
            slot: 99,
            ammunition: 203,
        },
        Command::FitAmmunition {
            warrior: 0,
            slot: 0,
            ammunition: 200,
        },
    ] {
        let mut state = advance_state(&f, &f["cases"][0]);
        let before = state.clone();
        assert!(apply(command, &mut state, &content).is_err());
        assert_session(&state, &before);
    }
}
