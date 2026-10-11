use centurions_content::{Content, InventoryRecord, Warrior, word};
use centurions_core::{
    Command, Event, Session, apply, combat::weapon_work::WeaponWork, load, save,
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
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}

fn project(state: &mut Session, snapshot: &Value) {
    for (warrior, native) in state
        .warriors
        .iter_mut()
        .zip(bytes(&snapshot["party"]).as_chunks::<3138>().0)
    {
        warrior.bytes = Warrior::read(native).unwrap().bytes;
    }
    state.crate_items = bytes(&snapshot["crate"])
        .as_chunks::<22>()
        .0
        .iter()
        .map(|p| InventoryRecord(*p))
        .collect();
    state.ammunition = bytes(&snapshot["ammunition"])
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| i32::from_be_bytes(*p))
        .collect();
    state.random.seed = snapshot["rng_seed"].as_u64().unwrap() as u32;
    state.random.draws = snapshot["rng_draws"].as_u64().unwrap();
    state.loaded_item_field32 = Some(snapshot["item_context"].as_i64().unwrap() as i16);
    let mut health = centurions_core::story_health::context(
        &state.warriors,
        state.selected_warrior,
        state.health_context.as_ref(),
    );
    health.affected = bytes(&snapshot["markers"])
        .as_chunks::<2>()
        .0
        .iter()
        .take(6)
        .map(|value| i16::from_be_bytes(*value))
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    state.health_context = Some(health);
    let weapon = bytes(&snapshot["weapon"]);
    let ammo = bytes(&snapshot["ammo"]);
    state.weapon_work = WeaponWork {
        sound: Some(word(&weapon, 14).unwrap()),
        weapon_armor_filter: Some(word(&weapon, 136).unwrap()),
        ammunition_armor_filter: Some(word(&ammo, 136).unwrap()),
    };
}

fn slot(state: &Session, warrior: usize, id: i16) -> usize {
    state.warriors[warrior]
        .inventory()
        .unwrap()
        .iter()
        .position(|p| p.id() == id)
        .unwrap()
}

fn original_cache(content: &Content, id: usize, captured: &Value) -> Vec<u8> {
    let mut record = content.items[id].clone();
    let class = word(&record, 26).unwrap().wrapping_abs();
    record[26..28].copy_from_slice(&class.to_be_bytes());
    assert_eq!(record, bytes(captured));
    record
}

fn atomic(state: &Session, content: &Content, command: Command) {
    let mut candidate = state.clone();
    assert!(apply(command, &mut candidate, content).is_err());
    assert_eq!(&candidate, state);
}

#[test]
fn earned_shield_charge_equipment_and_every_saved_continuation_match_native() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let content = Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap();
    let fixture: Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("evidence/fixtures/personal-shield.json")).unwrap(),
    )
    .unwrap();
    for case in fixture["equipment_cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["kind"] == "manual")
    {
        let source = case["source"].as_i64().unwrap();
        let foundation = fixture["foundations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|f| {
                f["input"]
                    .as_str()
                    .unwrap()
                    .ends_with(&format!("laboratory-{source}.json"))
            })
            .unwrap();
        let mut state: Session = serde_json::from_value(foundation["session"].clone()).unwrap();
        project(&mut state, &case["before"]);
        state.validate(&content).unwrap();
        let initial = state.clone();
        let member = case["member"].as_u64().unwrap() as usize;
        let path = std::env::temp_dir().join(format!(
            "personal-shield-command-{}-{source}.json",
            std::process::id()
        ));
        let mut resumed = state.clone();
        let mut effects = vec![];
        for (id, equip) in [
            (600, false),
            (637, false),
            (637, true),
            (307, true),
            (600, true),
        ] {
            let command = if !equip {
                Command::TakeCrate {
                    slot: state.crate_items.iter().position(|p| p.id() == id).unwrap(),
                    warrior: member,
                }
            } else if id == 307 {
                Command::FitAmmunition {
                    warrior: member,
                    slot: slot(&state, member, 637),
                    ammunition: 307,
                }
            } else {
                Command::EquipItem {
                    warrior: member,
                    slot: slot(&state, member, id),
                }
            };
            let events = apply(command.clone(), &mut state, &content).unwrap();
            assert_eq!(apply(command, &mut resumed, &content).unwrap(), events);
            assert_eq!(resumed, state);
            effects.extend(events);
            if id == 307 {
                // Complete loadammoproc's independently captured writers.
                assert_eq!(
                    state.ammunition_work.item,
                    Some(original_cache(
                        &content,
                        637,
                        &case["refitted"]["item_work"]
                    ))
                );
                assert_eq!(
                    state.ammunition_work.ammo,
                    Some(original_cache(&content, 307, &case["refitted"]["ammo"]))
                );
            }
            if id == 600 && !equip {
                atomic(
                    &state,
                    &content,
                    Command::EquipItem {
                        warrior: member,
                        slot: slot(&state, member, 600),
                    },
                );
            }
            save(&path, &state, &content).unwrap();
            resumed = load(&path, &content).unwrap();
            assert_eq!(resumed, state);
        }
        let mut expected = initial;
        project(&mut expected, &case["compacted"]);
        expected.ammunition_work.item = Some(original_cache(
            &content,
            600,
            &case["compacted"]["item_work"],
        ));
        // The earlier accepted battery Fit loads AMMO307; later wear/compact
        // operations retain that cache while loaditem selects shield600.
        expected.ammunition_work.ammo =
            Some(original_cache(&content, 307, &case["compacted"]["ammo"]));
        assert_eq!(state, expected, "source {source}");
        let sounds: Vec<_> = effects
            .iter()
            .filter_map(|e| match e {
                Event::Sound { resource_id } => Some(*resource_id),
                _ => None,
            })
            .collect();
        let expected_sounds: Vec<_> = case["compacted"]["sounds"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p.as_i64().unwrap() as i16)
            .collect();
        assert_eq!(sounds, expected_sounds);
        assert!(effects.iter().all(|e| matches!(e, Event::Sound { .. })));
        assert_eq!(state.ammunition[107], 0);
        atomic(
            &state,
            &content,
            Command::UnequipItem {
                warrior: member,
                slot: slot(&state, member, 637),
            },
        );
        atomic(
            &state,
            &content,
            Command::FitAmmunition {
                warrior: member,
                slot: slot(&state, member, 637),
                ammunition: 307,
            },
        );
        atomic(
            &state,
            &content,
            Command::FitAmmunition {
                warrior: member,
                slot: slot(&state, member, 637),
                ammunition: 203,
            },
        );
        for (id, equip) in [(600, false), (637, false), (637, true), (600, true)] {
            let command = if equip {
                Command::EquipItem {
                    warrior: member,
                    slot: slot(&state, member, id),
                }
            } else {
                Command::UnequipItem {
                    warrior: member,
                    slot: slot(&state, member, id),
                }
            };
            let events = apply(command.clone(), &mut state, &content).unwrap();
            assert_eq!(apply(command, &mut resumed, &content).unwrap(), events);
            assert_eq!(resumed, state);
            effects.extend(events);
            save(&path, &state, &content).unwrap();
            resumed = load(&path, &content).unwrap();
            assert_eq!(resumed, state);
        }
        project(&mut expected, &case["after"]);
        expected.ammunition_work.item =
            Some(original_cache(&content, 600, &case["after"]["item_work"]));
        expected.ammunition_work.ammo = Some(original_cache(&content, 307, &case["after"]["ammo"]));
        assert_eq!(state, expected);
        let sounds: Vec<_> = effects
            .iter()
            .filter_map(|event| match event {
                Event::Sound { resource_id } => Some(*resource_id),
                _ => None,
            })
            .collect();
        assert_eq!(
            sounds,
            case["after"]["sounds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| value.as_i64().unwrap() as i16)
                .collect::<Vec<_>>()
        );
        std::fs::remove_file(&path).unwrap();
    }
}

#[test]
fn ordinary_pistol_fit_after_battery_fit_matches_native_complete_session() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let content = Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap();
    let fixture: Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("evidence/fixtures/shield-ranged.json")).unwrap(),
    )
    .unwrap();
    let case = &fixture["restore_cases"][0];
    let mut state: Session =
        serde_json::from_value(fixture["foundations"][0]["session"].clone()).unwrap();
    project(&mut state, &case["before"]);
    let initial = state.clone();
    let path = std::env::temp_dir().join(format!(
        "personal-shield-fit-restore-{}.json",
        std::process::id()
    ));
    save(&path, &state, &content).unwrap();
    let mut resumed = load(&path, &content).unwrap();
    let command = Command::FitAmmunition {
        warrior: 3,
        slot: slot(&state, 3, 2),
        ammunition: 201,
    };
    let events = apply(command.clone(), &mut state, &content).unwrap();
    assert_eq!(apply(command, &mut resumed, &content).unwrap(), events);
    assert_eq!(resumed, state);
    let mut expected = initial;
    project(&mut expected, &case["after"]);
    // The accepted ordinary Fit reloads its actual item and ammunition cache.
    expected.ammunition_work.item = Some(original_cache(&content, 2, &case["after"]["item_work"]));
    expected.ammunition_work.ammo = Some(original_cache(&content, 201, &case["after"]["ammo"]));
    assert_eq!(state, expected);
    let before_sounds = case["before"]["sounds"].as_array().unwrap().len();
    assert_eq!(
        events,
        case["after"]["sounds"].as_array().unwrap()[before_sounds..]
            .iter()
            .map(|s| Event::Sound {
                resource_id: s.as_i64().unwrap() as i16
            })
            .collect::<Vec<_>>()
    );
    save(&path, &state, &content).unwrap();
    assert_eq!(load(&path, &content).unwrap(), state);
    std::fs::remove_file(&path).unwrap();
}
