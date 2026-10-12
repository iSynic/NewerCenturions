use centurions_content::{Content, InventoryRecord, Warrior, word};
use centurions_core::{
    Command, Event, MedicalContinuation, MedicalOrigin, MedicalTarget, Session, apply,
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
        serde_json::from_str(include_str!(
            "../../../evidence/fixtures/learned-portrait-heal.json"
        ))
        .unwrap(),
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
    health.marks_affected = snapshot["heal_marks_affected"].as_u64().unwrap() != 0;
    health
        .affected
        .copy_from_slice(&words(&snapshot["markers"])[..6]);
    health.effect = state.attack_packet.as_ref().unwrap()[18] as i8;
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
        "centurions-learned-heal-{}-{label}.json",
        std::process::id()
    ));
    save(&path, state, content).unwrap();
    let restored = load(&path, content).unwrap();
    same(&restored, state);
    std::fs::remove_file(path).unwrap();
    restored
}

#[test]
fn native_learned_portrait_continuations_match_full_session_and_saved_replay() {
    let (content, fixture) = fixture();
    for (index, case) in fixture["cases"].as_array().unwrap().iter().enumerate() {
        let mut state = input(&fixture, &case["before"]);
        let initial = state.clone();
        let power = case["power"].as_i64().unwrap() as i16;
        let begin = Command::BeginLearnedHeal { power };
        if !case["prepared"].as_bool().unwrap() {
            atomic(&state, &content, begin);
            continue;
        }
        let mut restored = checkpoint(&state, &content, &format!("before-{index}"));
        let mut events = apply(begin.clone(), &mut state, &content).unwrap();
        assert_eq!(
            events,
            apply(begin.clone(), &mut restored, &content).unwrap()
        );
        same(&state, &restored);
        let mut expected = initial.clone();
        project(&mut expected, &case["after_preparation"]);
        source_item_refresh(&mut expected, &content);
        let actor = case["actor"].as_u64().unwrap() as usize;
        let choice = centurions_core::PsiMenuChoice {
            row: 0,
            column: 0,
            power,
        };
        expected.psi_menu_choices[actor] = Some(choice.clone());
        expected.medical_continuation = Some(MedicalContinuation {
            origin: MedicalOrigin::LearnedHeal,
            actor: case["actor"].as_u64().unwrap() as usize,
            item: 0,
            spell: 1101,
            power,
        });
        same(&state, &expected);
        atomic(&state, &content, begin);
        atomic(&state, &content, Command::EndTurn);
        atomic(
            &state,
            &content,
            Command::SelectMedicalTarget {
                selection: MedicalTarget::LivingPoint { x: 1, y: 1 },
            },
        );
        restored = checkpoint(&state, &content, &format!("pending-{index}"));
        let selection = if case["kind"] == "confirmed_cancel" {
            MedicalTarget::ConfirmedCancel
        } else {
            MedicalTarget::Portrait {
                warrior: case["target"].as_u64().unwrap() as usize,
            }
        };
        let command = Command::SelectMedicalTarget { selection };
        if matches!(
            case["kind"].as_str().unwrap(),
            "living_portrait" | "terminal_portrait"
        ) {
            atomic(&state, &content, command.clone());
            atomic(&restored, &content, command);
            same(&state, &restored);
            continue;
        }
        let result = apply(command.clone(), &mut state, &content).unwrap();
        assert_eq!(
            result,
            apply(command.clone(), &mut restored, &content).unwrap()
        );
        same(&state, &restored);
        events.extend(result);
        let mut expected = initial;
        project(&mut expected, &case["after"]);
        source_item_refresh(&mut expected, &content);
        expected.psi_menu_choices[actor] = Some(choice);
        same(&state, &expected);
        assert_eq!(sounds(&events), native_sounds(case), "learned case {index}");
        atomic(&state, &content, command);
        checkpoint(&state, &content, &format!("after-{index}"));
    }
}

#[test]
fn learned_preparation_rejects_unknown_powers_and_preserves_inventory() {
    let (content, fixture) = fixture();
    let state = input(&fixture, &fixture["cases"][0]["before"]);
    for power in [-1, 0, 5, i16::MAX] {
        atomic(&state, &content, Command::BeginLearnedHeal { power });
    }
    let mut marking = state.clone();
    marking.health_context.as_mut().unwrap().marks_affected = true;
    atomic(&marking, &content, Command::BeginLearnedHeal { power: 1 });
    let mut other_menu = state.clone();
    other_menu.psi_menu_choices[5] = Some(centurions_core::PsiMenuChoice {
        row: 1,
        column: 0,
        power: 5,
    });
    atomic(
        &other_menu,
        &content,
        Command::BeginLearnedHeal { power: 1 },
    );
    for power in [1, 2] {
        let mut prepared = state.clone();
        apply(Command::BeginLearnedHeal { power }, &mut prepared, &content).unwrap();
        for (before, after) in state.warriors.iter().zip(&prepared.warriors) {
            assert_eq!(before.inventory().unwrap(), after.inventory().unwrap());
        }
        assert_eq!(state.random, prepared.random);
    }
}

#[test]
fn consecutive_native_heals_restore_saved_menu_power_without_repeating_costs() {
    let (content, _) = fixture();
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/learned-heal-memory.json"
    ))
    .unwrap();
    for (index, case) in fixture["cases"].as_array().unwrap().iter().enumerate() {
        let mut state: Session =
            serde_json::from_value(fixture["foundation"]["session"].clone()).unwrap();
        project(&mut state, &case["before"]);
        let initial = state.clone();
        let second_power = case["second_power"].as_i64().unwrap() as i16;
        let mut events = vec![];
        for (step, (command, snapshot, pending, remembered_power)) in [
            (
                Command::BeginLearnedHeal { power: 2 },
                "first_prepared",
                true,
                2,
            ),
            (
                Command::SelectMedicalTarget {
                    selection: MedicalTarget::ConfirmedCancel,
                },
                "after_cancel",
                false,
                2,
            ),
            (
                Command::BeginLearnedHeal {
                    power: second_power,
                },
                "second_prepared",
                true,
                second_power,
            ),
            (
                Command::SelectMedicalTarget {
                    selection: MedicalTarget::Portrait { warrior: 1 },
                },
                "after",
                false,
                second_power,
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let mut restored = checkpoint(&state, &content, &format!("memory-{index}-{step}"));
            let result = apply(command.clone(), &mut state, &content).unwrap();
            assert_eq!(result, apply(command, &mut restored, &content).unwrap());
            same(&state, &restored);
            events.extend(result);
            let mut expected = initial.clone();
            project(&mut expected, &case[snapshot]);
            source_item_refresh(&mut expected, &content);
            expected.psi_menu_choices[5] = Some(centurions_core::PsiMenuChoice {
                row: 0,
                column: 0,
                power: remembered_power,
            });
            if pending {
                expected.medical_continuation = Some(MedicalContinuation {
                    origin: MedicalOrigin::LearnedHeal,
                    actor: 5,
                    item: 0,
                    spell: 1101,
                    power: remembered_power,
                });
            }
            same(&state, &expected);
        }
        assert_eq!(sounds(&events), native_sounds(case), "memory case {index}");
    }
}

#[test]
fn owned_prior_spell_row_matches_native_heal_menu_and_saved_continuation() {
    let (content, _) = fixture();
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/learned-heal-row.json"
    ))
    .unwrap();
    for (index, case) in fixture["cases"].as_array().unwrap().iter().enumerate() {
        let mut state: Session =
            serde_json::from_value(fixture["foundation"]["session"].clone()).unwrap();
        project(&mut state, &case["before"]);
        let selected = bytes(&case["before"]["selected"]);
        state.psi_menu_choices[5] = Some(centurions_core::PsiMenuChoice {
            row: selected[10],
            column: selected[11],
            power: words(&case["before"]["powers"])[5],
        });
        let initial = state.clone();
        let power = case["power"].as_i64().unwrap() as i16;
        let mut events = vec![];
        for (step, (command, snapshot, pending)) in [
            (Command::BeginLearnedHeal { power }, "prepared", true),
            (
                Command::SelectMedicalTarget {
                    selection: MedicalTarget::Portrait { warrior: 1 },
                },
                "after",
                false,
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let mut restored = checkpoint(&state, &content, &format!("row-{index}-{step}"));
            let result = apply(command.clone(), &mut state, &content).unwrap();
            assert_eq!(result, apply(command, &mut restored, &content).unwrap());
            same(&state, &restored);
            events.extend(result);
            let mut expected = initial.clone();
            project(&mut expected, &case[snapshot]);
            source_item_refresh(&mut expected, &content);
            expected.psi_menu_choices[5] = Some(centurions_core::PsiMenuChoice {
                row: 0,
                column: 0,
                power,
            });
            if pending {
                expected.medical_continuation = Some(MedicalContinuation {
                    origin: MedicalOrigin::LearnedHeal,
                    actor: 5,
                    item: 0,
                    spell: 1101,
                    power,
                });
            }
            same(&state, &expected);
        }
        assert_eq!(sounds(&events), native_sounds(case), "row case {index}");
    }
}

#[test]
fn higher_portrait_powers_match_native_full_session_and_saved_continuations() {
    let (content, _) = fixture();
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/learned-portrait-power.json"
    ))
    .unwrap();
    for (index, case) in fixture["cases"].as_array().unwrap().iter().enumerate() {
        let mut state = input(&fixture, &case["before"]);
        let initial = state.clone();
        let power = case["power"].as_i64().unwrap() as i16;
        let actor = state.combat.as_ref().unwrap().active_body().unwrap() as usize;
        let kind = case["kind"].as_str().unwrap();
        for snapshot in ["before", "prepared", "after"] {
            assert_eq!(
                case[snapshot]["health_effect"].as_u64().unwrap(),
                bytes(&case[snapshot]["packet"])[18] as u64
            );
            assert_eq!(
                case[snapshot]["health_last_body"].as_i64().unwrap(),
                initial.health_context.as_ref().unwrap().last_body as i64
            );
        }
        let begin = Command::BeginLearnedHeal { power };
        if kind == "psi_below_cost" {
            assert_eq!(initial.warriors[actor].psi_points().unwrap(), 6 * power - 1);
            atomic(&initial, &content, begin);
            continue;
        }
        let mut restored = checkpoint(&state, &content, &format!("power-before-{index}"));
        let mut events = apply(begin.clone(), &mut state, &content).unwrap();
        assert_eq!(events, apply(begin, &mut restored, &content).unwrap());
        same(&state, &restored);
        let choice = centurions_core::PsiMenuChoice {
            row: 0,
            column: 0,
            power,
        };
        let mut expected = initial.clone();
        project(&mut expected, &case["prepared"]);
        source_item_refresh(&mut expected, &content);
        expected.psi_menu_choices[actor] = Some(choice.clone());
        expected.medical_continuation = Some(MedicalContinuation {
            origin: MedicalOrigin::LearnedHeal,
            actor,
            item: 0,
            spell: 1101,
            power,
        });
        same(&state, &expected);
        assert_eq!(case["prepared"]["required_target_count"], 1);
        assert_eq!(state.attack_count, power);
        atomic(&state, &content, Command::EndTurn);
        atomic(
            &state,
            &content,
            Command::SelectMedicalTarget {
                selection: MedicalTarget::Portrait { warrior: 0 },
            },
        );
        restored = checkpoint(&state, &content, &format!("power-pending-{index}"));
        if kind == "psi_equal_cost" {
            assert_eq!(state.warriors[actor].psi_points().unwrap(), 0);
            assert_eq!(sounds(&events), native_sounds(case));
            continue;
        }
        let selection = if kind == "confirmed_cancel" {
            MedicalTarget::ConfirmedCancel
        } else {
            MedicalTarget::Portrait {
                warrior: case["target"].as_u64().unwrap() as usize,
            }
        };
        let command = Command::SelectMedicalTarget { selection };
        let result = apply(command.clone(), &mut state, &content).unwrap();
        assert_eq!(
            result,
            apply(command.clone(), &mut restored, &content).unwrap()
        );
        same(&state, &restored);
        events.extend(result);
        let mut expected = initial;
        project(&mut expected, &case["after"]);
        source_item_refresh(&mut expected, &content);
        expected.psi_menu_choices[actor] = Some(choice);
        same(&state, &expected);
        assert_eq!(sounds(&events), native_sounds(case), "power case {index}");
        atomic(&state, &content, command);
        checkpoint(&state, &content, &format!("power-after-{index}"));
    }
}
