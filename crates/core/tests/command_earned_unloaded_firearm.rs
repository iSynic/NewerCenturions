use centurions_content::{Content, InventoryRecord, digest, word};
use centurions_core::{Command, Event, RuleError, Session, apply, load, save};
use serde_json::Value;

fn fixture() -> (Content, Value) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let content = Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap();
    let fixture = serde_json::from_slice(
        &std::fs::read(root.join("evidence/fixtures/earned-unloaded-firearm.json")).unwrap(),
    )
    .unwrap();
    (content, fixture)
}

fn bytes(f: &Value, value: &Value) -> Vec<u8> {
    let text = value.as_str().unwrap_or_else(|| {
        f["shared_rule_buffers"][value["shared_buffer_sha256"].as_str().unwrap()]
            .as_str()
            .unwrap()
    });
    assert_eq!(text.len() % 2, 0);
    let result: Vec<_> = text
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect();
    if value.is_object() {
        assert_eq!(result.len() as u64, value["byte_length"].as_u64().unwrap());
        assert_eq!(
            digest(&result),
            value["shared_buffer_sha256"].as_str().unwrap()
        );
    }
    result
}

fn words(f: &Value, value: &Value) -> Vec<i16> {
    bytes(f, value)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|value| i16::from_be_bytes(*value))
        .collect()
}

fn project(state: &mut Session, f: &Value, native: &Value) {
    for (warrior, record) in state
        .warriors
        .iter_mut()
        .zip(bytes(f, &native["party"]).as_chunks::<3138>().0)
    {
        warrior.bytes = record.to_vec();
    }
    state.crate_items = bytes(f, &native["crate"])
        .as_chunks::<22>()
        .0
        .iter()
        .map(|record| InventoryRecord(*record))
        .collect();
    state.ammunition = bytes(f, &native["reserves"])[..800]
        .as_chunks::<4>()
        .0
        .iter()
        .map(|record| i32::from_be_bytes(*record))
        .collect();
    state.ammunition_work.item = Some(bytes(f, &native["item_work"]));
    state.ammunition_work.weapon = Some(bytes(f, &native["weapon_work"]));
    state.ammunition_work.ammo = Some(bytes(f, &native["ammo_work"]));
    state.loaded_item_field32 = Some(native["item_context"].as_i64().unwrap() as i16);
    state.party_buffs = bytes(f, &native["party_buffs"]);
    state.flags = bytes(f, &native["flags"]);
    state.psi_memory = Some(bytes(f, &native["memory"]));
    state.allies = bytes(f, &native["references"]);
    state.ally_count = native["ally_count"].as_u64().unwrap() as usize;
    state.party_currency = native["currency"].as_i64().unwrap() as i32;
    state.attack_packet = Some(bytes(f, &native["packet"]));
    state.attack_identity = Some(words(f, &native["identity"]).try_into().unwrap());
    state.attack_count = native["count"].as_i64().unwrap() as i16;
    // The six-byte automatic mask is separate from the 220-byte spell target mask.
    assert_eq!(
        bytes(f, &native["auto_mask"]),
        state.party_mask.map(|value| value as u8)
    );
    state.selected_warrior = native["selected_member"].as_u64().unwrap() as usize;
    state.random.seed = native["seed"].as_u64().unwrap() as u32;
    state.random.draws = native["draws"].as_u64().unwrap();
    state.bandage_ready = Some(native["aid_gate"].as_i64().unwrap() as i16);
    let health = state.health_context.as_mut().unwrap();
    health.unavailable = native["unavailable"].as_i64().unwrap() as i8;
    health.selected_warrior = state.selected_warrior;
    health.effect = native["health_effect"].as_i64().unwrap() as i8;
    health.last_body = native["health_last_body"].as_i64().unwrap() as i16;
    health.marks_affected = native["heal_marks_affected"].as_i64().unwrap() != 0;
    health
        .affected
        .copy_from_slice(&words(f, &native["markers"])[..6]);
    let exploration = state.exploration.as_mut().unwrap();
    exploration.floor = words(f, &native["map"]);
    exploration.objects = words(f, &native["objects"]);
    exploration.events = bytes(f, &native["outdoor_events"])
        .as_chunks::<4>()
        .0
        .iter()
        .map(|value| i32::from_be_bytes(*value))
        .collect();
    exploration.visibility = bytes(f, &native["visibility"]);
    exploration.event_headers = bytes(f, &native["event_headers"]);
    exploration.region_state = bytes(f, &native["active_region"]);
    exploration.map = native["map_id"].as_i64().unwrap() as i32;
    exploration.x = native["world_x"].as_i64().unwrap() as i32;
    exploration.y = native["world_y"].as_i64().unwrap() as i32;
    exploration.day = native["day"].as_i64().unwrap() as i16;
    exploration.hour = native["hour"].as_i64().unwrap() as i16;
    exploration.minute = native["minute"].as_i64().unwrap() as i16;
    exploration.fatigue = native["fatigue"].as_i64().unwrap() as i16;
    exploration.facing = native["facing"].as_u64().unwrap() as u8;
    exploration.last_move = bytes(f, &native["last_move"])
        .into_iter()
        .map(|value| value as i8)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    // Crate UI loot, selected inventory/profile copies and unused tactical buffers
    // are declared native workspace. They do not replace saved mission/booty state.
    assert_eq!(native["combat"], 0);
    assert!(state.combat.is_none());
}

fn initial(f: &Value, case: &Value, content: &Content) -> Session {
    let mut state: Session = serde_json::from_value(f["foundation"]["session"].clone()).unwrap();
    match case["kind"].as_str().unwrap() {
        "capacity964" => {
            state.warriors[4].bytes[0x9d8..0x9da].copy_from_slice(&964_i16.to_be_bytes())
        }
        "capacity965" => {
            state.warriors[4].bytes[0x9d8..0x9da].copy_from_slice(&965_i16.to_be_bytes())
        }
        "count99" => state.warriors[4].bytes[0x1a..0x1c].copy_from_slice(&99_i16.to_be_bytes()),
        "actual" | "level3" => {}
        other => panic!("unqualified earned case {other}"),
    }
    let controlled = state.clone();
    project(&mut state, f, &case["before"]);
    assert_eq!(
        state, controlled,
        "native input projection {}",
        case["kind"]
    );
    state.validate(content).unwrap();
    state
}

fn expected(before: &Session, f: &Value, native: &Value) -> Session {
    let mut state = before.clone();
    project(&mut state, f, native);
    state
}

fn events(case: &Value, before: &Value, after: &Value, pickup: bool) -> Vec<Event> {
    let mut result: Vec<_> =
        case["sounds"].as_array().unwrap()[before["sound_count"].as_u64().unwrap() as usize
            ..after["sound_count"].as_u64().unwrap() as usize]
            .iter()
            .map(|sound| Event::Sound {
                resource_id: sound.as_i64().unwrap() as i16,
            })
            .collect();
    if pickup {
        result.extend(case["warnings"].as_array().unwrap().iter().map(|warning| {
            Event::CollectionRejected {
                warning: warning.as_i64().unwrap() as i16,
            }
        }));
    }
    result
}

fn checkpoint(state: &Session, content: &Content, path: &std::path::Path) -> Session {
    save(path, state, content).unwrap();
    let document: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(document["schema_version"], 3);
    let resumed = load(path, content).unwrap();
    assert_eq!(&resumed, state);
    resumed
}

#[test]
fn earned_unloaded_pickup_and_outdoor_replacement_match_full_saved_native_projection() {
    let (content, f) = fixture();
    let path = std::env::temp_dir().join(format!(
        "centurions-earned-unloaded-{}.json",
        std::process::id()
    ));
    let items = content.items.clone();
    let cache = content.map(3).unwrap().item_cache.clone();
    let attacks = content
        .map(3)
        .unwrap()
        .combat
        .as_ref()
        .unwrap()
        .attacks
        .clone();
    let mut checked = 0;
    for case in f["cases"].as_array().unwrap() {
        let mut state = initial(&f, case, &content);
        let before = state.clone();
        let mut resumed = checkpoint(&state, &content, &path);
        let command = Command::TakeCrate {
            slot: 9,
            warrior: 4,
        };
        let actual_events = apply(command.clone(), &mut state, &content).unwrap();
        assert_eq!(
            apply(command, &mut resumed, &content).unwrap(),
            actual_events
        );
        assert_eq!(
            actual_events,
            events(case, &case["before"], &case["closed"], true)
        );
        assert_eq!(
            state,
            expected(&before, &f, &case["closed"]),
            "pickup {}",
            case["kind"]
        );
        assert_eq!(state, resumed);
        resumed = checkpoint(&state, &content, &path);
        if case.get("wear_return").is_some() {
            if case["kind"] == "level3" {
                // This control changes only level after actual successful collection.
                state.warriors[4].bytes[0x3a..0x3c].copy_from_slice(&3_i16.to_be_bytes());
                assert_eq!(state, expected(&resumed, &f, &case["wear_before"]));
                resumed = checkpoint(&state, &content, &path);
            }
            let before = state.clone();
            let command = Command::EquipItem {
                warrior: 4,
                slot: 4,
            };
            if case["wear_return"] == 1 {
                let actual_events = apply(command.clone(), &mut state, &content).unwrap();
                assert_eq!(
                    apply(command, &mut resumed, &content).unwrap(),
                    actual_events
                );
                assert_eq!(
                    actual_events,
                    events(case, &case["wear_before"], &case["compacted"], false)
                );
                assert_eq!(state, expected(&before, &f, &case["compacted"]));
                assert_eq!(state, resumed);
                let inventory = state.warriors[4].inventory().unwrap();
                assert_eq!(
                    inventory
                        .iter()
                        .map(InventoryRecord::id)
                        .collect::<Vec<_>>(),
                    [165, 403, 460, 1, 700]
                );
                assert_eq!(word(&inventory[0].0, 2).unwrap(), 0);
                assert_eq!(word(&inventory[0].0, 4).unwrap(), 0);
                assert_eq!(word(&state.warriors[4].bytes, 0x126).unwrap(), 0);
                assert_eq!(state.ammunition[80], before.ammunition[80]);
                assert_eq!(state.random, before.random);
            } else {
                // The helper's native sound6000 is compared separately; the public
                // invalid-action boundary commits no work, events or warrior change.
                assert!(matches!(
                    apply(command.clone(), &mut state, &content),
                    Err(RuleError::Invalid(_))
                ));
                assert!(matches!(
                    apply(command, &mut resumed, &content),
                    Err(RuleError::Invalid(_))
                ));
                assert_eq!(state, before);
                assert_eq!(state, resumed);
            }
        }
        checkpoint(&state, &content, &path);
        assert_eq!(content.items, items);
        assert_eq!(content.map(3).unwrap().item_cache, cache);
        assert_eq!(
            content.map(3).unwrap().combat.as_ref().unwrap().attacks,
            attacks
        );
        checked += 1;
    }
    assert_eq!(checked, 5);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn other_instance_modifiers_new_removal_and_later_movement_failure_are_atomic() {
    let (content, f) = fixture();
    let original = initial(&f, &f["cases"][0], &content);
    for offset in [9, 13, 15, 18] {
        let mut state = original.clone();
        state.crate_items[9].0[offset] = state.crate_items[9].0[offset].wrapping_add(1);
        let before = state.clone();
        assert!(matches!(
            apply(
                Command::TakeCrate {
                    slot: 9,
                    warrior: 4
                },
                &mut state,
                &content
            ),
            Err(RuleError::Unsupported { .. })
        ));
        assert_eq!(state, before, "unqualified instance byte {offset}");
    }
    let mut collected = original.clone();
    apply(
        Command::TakeCrate {
            slot: 9,
            warrior: 4,
        },
        &mut collected,
        &content,
    )
    .unwrap();
    assert_eq!(collected, expected(&original, &f, &f["cases"][0]["closed"]));
    let mut later_failure = collected.clone();
    // The old1 removal reaches movement recalculation before installing new165.
    // This declared zero divisor must reject after that local removal prefix.
    later_failure.warriors[4].bytes[0x9e2..0x9e4].copy_from_slice(&(-10_i16).to_be_bytes());
    later_failure.warriors[4].bytes[0xace..0xad0].fill(0);
    later_failure.validate(&content).unwrap();
    let before = later_failure.clone();
    assert!(matches!(
        apply(
            Command::EquipItem {
                warrior: 4,
                slot: 4
            },
            &mut later_failure,
            &content
        ),
        Err(RuleError::Unsupported {
            action: "combat.movement",
            dependency: "SANE division by zero"
        })
    ));
    assert_eq!(later_failure, before);
    let mut worn = collected.clone();
    apply(
        Command::EquipItem {
            warrior: 4,
            slot: 4,
        },
        &mut worn,
        &content,
    )
    .unwrap();
    assert_eq!(worn, expected(&collected, &f, &f["cases"][0]["compacted"]));
    let before = worn.clone();
    assert!(matches!(
        apply(
            Command::UnequipItem {
                warrior: 4,
                slot: 0
            },
            &mut worn,
            &content
        ),
        Err(RuleError::Unsupported { .. })
    ));
    assert_eq!(worn, before);
}
