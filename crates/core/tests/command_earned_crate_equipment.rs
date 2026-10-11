use centurions_content::{Content, InventoryRecord, Warrior, word};
use centurions_core::{Command, Event, Session, apply, load, save};
use serde_json::Value;

fn data() -> (Value, Content) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let fixture = serde_json::from_slice(
        &std::fs::read(root.join("evidence/fixtures/earned-crate-equipment.json")).unwrap(),
    )
    .unwrap();
    let content = Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap();
    (fixture, content)
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

fn set_party(state: &mut Session, snapshot: &Value) {
    for (warrior, native) in state
        .warriors
        .iter_mut()
        .zip(bytes(&snapshot["party"]).as_chunks::<3138>().0)
    {
        warrior.bytes = Warrior::read(native).unwrap().bytes;
    }
}

fn records(value: &Value) -> Vec<InventoryRecord> {
    bytes(value)
        .as_chunks::<22>()
        .0
        .iter()
        .map(|record| InventoryRecord(*record))
        .collect()
}

fn initial(fixture: &Value, case: &Value) -> Session {
    let mut state: Session = serde_json::from_value(
        fixture["foundations"][case["foundation"].as_u64().unwrap() as usize]["session"].clone(),
    )
    .unwrap();
    // The named boundary controls supply independent native capacity/count/quantity inputs.
    set_party(&mut state, &case["before"]);
    state
}

fn expected(before: &Session, native: &Value, crate_view: bool) -> Session {
    let mut state = before.clone();
    set_party(&mut state, native);
    if crate_view {
        state.crate_items = records(&native["crate"]);
    } else {
        state.booty_items = records(&native["loot"]);
    }
    state.loaded_item_field32 = Some(native["item_context"].as_i64().unwrap() as i16);
    state.ammunition_work.item = Some(bytes(&native["item_work"]));
    state.health_context.as_mut().unwrap().affected = bytes(&native["markers"])
        .as_chunks::<2>()
        .0
        .iter()
        .take(6)
        .map(|value| i16::from_be_bytes(*value))
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    assert_eq!(
        state.health_context.as_ref().unwrap().marks_affected,
        native["heal_marks_affected"].as_u64().unwrap() != 0
    );
    assert_eq!(
        state.random.seed,
        native["rng_seed"].as_u64().unwrap() as u32
    );
    assert_eq!(state.random.draws, native["rng_draws"].as_u64().unwrap());
    state
}

fn events(after: &Value, before: &Value) -> Vec<Event> {
    after["sounds"]
        .as_array()
        .unwrap()
        .iter()
        .skip(before["sounds"].as_array().unwrap().len())
        .map(|sound| Event::Sound {
            resource_id: sound.as_i64().unwrap() as i16,
        })
        .chain(
            after["warnings"]
                .as_array()
                .unwrap()
                .iter()
                .skip(before["warnings"].as_array().unwrap().len())
                .map(|warning| Event::CollectionRejected {
                    warning: warning.as_i64().unwrap() as i16,
                }),
        )
        .collect()
}

fn slot(state: &Session, member: usize, id: i16) -> usize {
    state.warriors[member]
        .inventory()
        .unwrap()
        .iter()
        .position(|item| item.id() == id)
        .unwrap()
}

fn atomic(state: &Session, content: &Content, command: Command) {
    let mut actual = state.clone();
    assert!(
        apply(command.clone(), &mut actual, content).is_err(),
        "{command:?}"
    );
    assert_eq!(&actual, state);
}

fn checkpoint(state: &Session, content: &Content, path: &std::path::Path) -> Session {
    save(path, state, content).unwrap();
    let resumed = load(path, content).unwrap();
    assert_eq!(&resumed, state);
    resumed
}

fn assert_state(actual: &Session, expected: &Session, label: &str) {
    let actual = serde_json::to_value(actual).unwrap();
    let expected = serde_json::to_value(expected).unwrap();
    let changed: Vec<_> = actual
        .as_object()
        .unwrap()
        .iter()
        .filter_map(|(key, value)| (expected[key] != *value).then_some(key))
        .collect();
    assert!(changed.is_empty(), "{label}: Session fields {changed:?}");
}

#[test]
fn earned_pickup_and_outdoor_wear_match_native_complete_session_and_resume() {
    let (fixture, content) = data();
    let path = std::env::temp_dir().join(format!(
        "centurions-earned-crate-public-{}.json",
        std::process::id()
    ));
    let mut checked = 0;
    for case in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["kind"] != "actual_composition")
    {
        let member = case["receiver"].as_u64().unwrap() as usize;
        let crate_view = case["crate_mode"].as_bool().unwrap();
        let mut state = initial(&fixture, case);
        state.validate(&content).unwrap();
        let before = state.clone();
        let mut resumed = checkpoint(&state, &content, &path);
        let command = if crate_view {
            Command::TakeCrate {
                slot: case["slot"].as_u64().unwrap() as usize,
                warrior: member,
            }
        } else {
            Command::TakeLoot {
                slot: case["slot"].as_u64().unwrap() as usize,
                warrior: member,
            }
        };
        let actual_events = apply(command.clone(), &mut state, &content).unwrap();
        assert_eq!(
            apply(command, &mut resumed, &content).unwrap(),
            actual_events
        );
        assert_eq!(resumed, state);
        let native = if crate_view {
            &case["closed"]
        } else {
            &case["picked"]
        };
        assert_eq!(actual_events, events(native, &case["before"]));
        assert_state(
            &state,
            &expected(&before, native, crate_view),
            case["kind"].as_str().unwrap(),
        );
        resumed = checkpoint(&state, &content, &path);
        if case.get("wear_return").is_some() {
            if case["kind"] == "level" {
                set_party(&mut state, &case["wear_before"]);
                resumed = checkpoint(&state, &content, &path);
            }
            let command = Command::EquipItem {
                warrior: member,
                slot: slot(&state, member, 165),
            };
            if case["wear_return"] == 1 {
                let before = state.clone();
                let actual_events = apply(command.clone(), &mut state, &content).unwrap();
                assert_eq!(
                    apply(command, &mut resumed, &content).unwrap(),
                    actual_events
                );
                assert_eq!(resumed, state);
                assert_eq!(actual_events, events(&case["compacted"], native));
                assert_state(
                    &state,
                    &expected(&before, &case["compacted"], true),
                    "native wear and compaction",
                );
                assert_eq!(slot(&state, member, 165), 0);
                let mut unrepresented = state.clone();
                unrepresented.warriors[member].bytes[0x132 + 19] = 1;
                atomic(
                    &unrepresented,
                    &content,
                    Command::UnequipItem {
                        warrior: member,
                        slot: 0,
                    },
                );
                atomic(
                    &state,
                    &content,
                    Command::FitAmmunition {
                        warrior: member,
                        slot: 0,
                        ammunition: 203,
                    },
                );
            } else {
                atomic(&state, &content, command);
                assert_eq!(state, resumed);
            }
        }
        checkpoint(&state, &content, &path);
        checked += 1;
    }
    assert_eq!(checked, 13);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn continuous_medical_stack_loaded_weapon_and_wear_match_native_saved_steps() {
    let (fixture, content) = data();
    let case = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["kind"] == "actual_composition")
        .unwrap();
    let mut state = initial(&fixture, case);
    let path = std::env::temp_dir().join(format!(
        "centurions-earned-crate-composition-{}.json",
        std::process::id()
    ));
    let mut resumed = checkpoint(&state, &content, &path);
    let mut native_before = &case["before"];
    for (id, native_after) in [
        (700, "needle_closed"),
        (165, "weapon_closed"),
        (0, "compacted"),
    ] {
        let before = state.clone();
        let command = if id == 0 {
            Command::EquipItem {
                warrior: 4,
                slot: slot(&state, 4, 165),
            }
        } else {
            Command::TakeCrate {
                warrior: 4,
                slot: state
                    .crate_items
                    .iter()
                    .position(|item| item.id() == id)
                    .unwrap(),
            }
        };
        let actual_events = apply(command.clone(), &mut state, &content).unwrap();
        assert_eq!(
            apply(command, &mut resumed, &content).unwrap(),
            actual_events
        );
        assert_eq!(resumed, state);
        assert_eq!(actual_events, events(&case[native_after], native_before));
        assert_state(
            &state,
            &expected(&before, &case[native_after], true),
            native_after,
        );
        resumed = checkpoint(&state, &content, &path);
        native_before = &case[native_after];
    }
    let inventory = state.warriors[4].inventory().unwrap();
    assert_eq!(
        inventory
            .iter()
            .map(InventoryRecord::id)
            .collect::<Vec<_>>(),
        [165, 403, 460, 1, 700]
    );
    assert_eq!(word(&inventory[4].0, 2).unwrap(), 2);
    assert_eq!(inventory[4].0[10], 24);
    assert_eq!(word(&state.warriors[4].bytes, 0x9d6).unwrap(), 970);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn unsupported_instances_and_modal_crate_access_are_atomic() {
    let (fixture, content) = data();
    let mut state: Session =
        serde_json::from_value(fixture["foundations"][1]["session"].clone()).unwrap();
    atomic(
        &state,
        &content,
        Command::TakeCrate {
            slot: 1000,
            warrior: 4,
        },
    );
    atomic(
        &state,
        &content,
        Command::TakeCrate {
            slot: 10,
            warrior: 6,
        },
    );
    state.crate_items[10].0[19] = 1;
    atomic(
        &state,
        &content,
        Command::TakeCrate {
            slot: 10,
            warrior: 4,
        },
    );
    let state: Session =
        serde_json::from_value(fixture["foundations"][2]["session"].clone()).unwrap();
    atomic(
        &state,
        &content,
        Command::TakeCrate {
            slot: 0,
            warrior: 4,
        },
    );
    atomic(
        &state,
        &content,
        Command::TakeLoot {
            slot: 0,
            warrior: 6,
        },
    );
}
