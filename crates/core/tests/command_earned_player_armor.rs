use centurions_content::{Content, InventoryRecord, Warrior, combat::MonsterRecord};
use centurions_core::{
    Command, Event, Phase, Session, apply,
    combat::{Combat, EnemySetup, SquadPlacement, TacticalMap},
    load, save,
};
use serde_json::Value;

fn fixture() -> Value {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    serde_json::from_str(
        &std::fs::read_to_string(root.join("evidence/fixtures/earned-player-armor.json")).unwrap(),
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
    let hex = value.as_str().unwrap();
    (0..hex.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&hex[index..index + 2], 16).unwrap())
        .collect()
}
fn party(snapshot: &Value) -> Vec<Warrior> {
    bytes(&snapshot["party"])
        .as_chunks::<3138>()
        .0
        .iter()
        .map(|record| Warrior::read(record).unwrap())
        .collect()
}
fn crate_records(snapshot: &Value) -> Vec<InventoryRecord> {
    bytes(&snapshot["crate"])
        .as_chunks::<22>()
        .0
        .iter()
        .map(|record| InventoryRecord(*record))
        .collect()
}
fn inventory_slot(state: &Session, member: usize, item: i16) -> usize {
    state.warriors[member]
        .inventory()
        .unwrap()
        .iter()
        .position(|record| record.id() == item)
        .unwrap()
}
fn checkpoint(state: &Session, content: &Content, path: &std::path::Path) -> Session {
    save(path, state, content).unwrap();
    let resumed = load(path, content).unwrap();
    assert_eq!(&resumed, state);
    resumed
}
fn native_expected(before: &Session, snapshot: &Value) -> Session {
    let mut expected = before.clone();
    set_party(&mut expected, snapshot);
    expected.crate_items = crate_records(snapshot);
    expected.loaded_item_field32 = Some(snapshot["item_context"].as_i64().unwrap() as i16);
    // The full native loaditem writer is captured for pickup, wear and removal,
    // including collection rejection before the destination record changes.
    expected.ammunition_work.item = Some(bytes(&snapshot["item_work"]));
    let affected: [i16; 6] = bytes(&snapshot["markers"])
        .as_chunks::<2>()
        .0
        .iter()
        .take(6)
        .map(|value| i16::from_be_bytes(*value))
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    if expected.health_context.is_some() || affected.iter().any(|value| *value != 0) {
        let mut health = centurions_core::story_health::context(
            &expected.warriors,
            expected.selected_warrior,
            expected.health_context.as_ref(),
        );
        health.affected = affected;
        if before.combat.is_some() {
            health.effect = before.attack_packet.as_ref().unwrap()[18] as i8;
        }
        expected.health_context = Some(health);
    }
    expected
}
fn set_party(state: &mut Session, snapshot: &Value) {
    for (warrior, native) in state.warriors.iter_mut().zip(party(snapshot)) {
        warrior.bytes = native.bytes;
    }
}
fn assert_state(actual: &Session, expected: &Session, context: &str) {
    if actual == expected {
        return;
    }
    fn difference(actual: &Value, expected: &Value, path: String) -> String {
        match (actual, expected) {
            (Value::Array(a), Value::Array(b)) if a.len() == b.len() => {
                let (index, (a, b)) = a
                    .iter()
                    .zip(b)
                    .enumerate()
                    .find(|(_, (a, b))| a != b)
                    .unwrap();
                difference(a, b, format!("{path}[{index}]"))
            }
            (Value::Object(a), Value::Object(b)) => {
                let (key, value) = a
                    .iter()
                    .find(|(key, value)| b.get(*key) != Some(*value))
                    .unwrap();
                difference(value, &b[key], format!("{path}.{key}"))
            }
            _ => format!("{path}: {actual} != {expected}"),
        }
    }
    panic!(
        "{context}: {}",
        difference(
            &serde_json::to_value(actual).unwrap(),
            &serde_json::to_value(expected).unwrap(),
            "Session".into()
        )
    );
}
fn controlled_combat(state: &mut Session, snapshot: &Value, member: usize) {
    let mut order = vec![-1; 110];
    order[1] = member as i16;
    order[2] = 10;
    state.phase = Phase::Combat;
    state.combat = Some(Combat {
        battle: 85,
        map: TacticalMap {
            tiles: vec![1155; 8100],
            shift: (0, 0),
            edge_word: 0,
        },
        squad: SquadPlacement {
            positions: vec![(20, 10); 6],
            under_tiles: vec![1155; 6],
        },
        enemies: EnemySetup {
            records: vec![MonsterRecord {
                bytes: bytes(&snapshot["target"]),
            }],
            positions: vec![(20, 20)],
        },
        order,
        cursor: 1,
        round: 0,
        round_scripts: Default::default(),
        death_queue: Default::default(),
        spell_fields: vec![0; 1200],
        field_used: vec![0; 128],
        attack_packet: state.attack_packet.clone(),
        attack_identity: state.attack_identity,
        weapon_work: state.weapon_work,
        unavailable: 0,
    });
    let mut health = centurions_core::story_health::context(
        &state.warriors,
        state.selected_warrior,
        state.health_context.as_ref(),
    );
    health.unavailable = 0;
    state.health_context = Some(health);
}

#[test]
fn earned_armor_public_commands_match_native_phases_complete_session_and_resume() {
    let fixture = fixture();
    let content = content();
    let path = std::env::temp_dir().join(format!(
        "centurions-earned-armor-{}.json",
        std::process::id()
    ));
    let mut checked = 0;
    for case in fixture["equipment_cases"].as_array().unwrap() {
        let member = case["member"].as_u64().unwrap() as usize;
        let id = case["item_id"].as_i64().unwrap() as i16;
        let mut state: Session =
            serde_json::from_value(fixture["foundation"]["session"].clone()).unwrap();
        set_party(&mut state, &case["before"]);
        state.crate_items = crate_records(&case["before"]);
        let before = state.clone();
        let source = state.crate_items[case["crate_slot"].as_u64().unwrap() as usize].clone();
        let mut resumed = checkpoint(&state, &content, &path);
        let command = Command::TakeCrate {
            warrior: member,
            slot: case["crate_slot"].as_u64().unwrap() as usize,
        };
        let events = apply(command.clone(), &mut state, &content).unwrap();
        assert_eq!(events, vec![Event::Sound { resource_id: 450 }]);
        assert_eq!(apply(command, &mut resumed, &content).unwrap(), events);
        assert_state(&state, &resumed, "resumed pickup");
        assert_state(
            &state,
            &native_expected(&before, &case["picked"]),
            "native pickup",
        );
        if case["combat"].as_bool().unwrap() {
            // The native family supplies only an active-combat mode/AP control for wear.
            controlled_combat(&mut state, &case["picked"], member);
        }
        resumed = checkpoint(&state, &content, &path);
        for (operation, snapshot) in [(0, "compacted"), (1, "removed"), (0, "after")] {
            let before = state.clone();
            let slot = inventory_slot(&state, member, id);
            let command = if operation == 0 {
                Command::EquipItem {
                    warrior: member,
                    slot,
                }
            } else {
                Command::UnequipItem {
                    warrior: member,
                    slot,
                }
            };
            let events = apply(command.clone(), &mut state, &content).unwrap();
            assert_eq!(events, vec![Event::Sound { resource_id: 450 }]);
            assert_eq!(apply(command, &mut resumed, &content).unwrap(), events);
            assert_state(&state, &resumed, "resumed equipment");
            assert_state(
                &state,
                &native_expected(&before, &case[snapshot]),
                &format!("{id}/{member}/{snapshot}"),
            );
            resumed = checkpoint(&resumed, &content, &path);
        }
        let inventory = state.warriors[member].inventory().unwrap();
        let worn = &inventory[inventory_slot(&state, member, id)];
        assert_eq!(&worn.0[..8], &source.0[..8]);
        assert_eq!(&worn.0[13..], &source.0[13..]);
        assert_eq!(worn.0[9], 1);
        assert_eq!(state.random, before.random);
        checked += 1;
    }
    assert_eq!(checked, 16);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn earned_armor_public_capacity_and_count_rejections_match_complete_native_state() {
    let fixture = fixture();
    let content = content();
    let mut checked = 0;
    for case in fixture["transfer_cases"].as_array().unwrap() {
        if case["variant"].as_u64().unwrap() == 0 {
            continue;
        }
        let mut state: Session =
            serde_json::from_value(fixture["foundation"]["session"].clone()).unwrap();
        set_party(&mut state, &case["before"]);
        state.crate_items = crate_records(&case["before"]);
        let before = state.clone();
        let events = apply(
            Command::TakeCrate {
                warrior: case["member"].as_u64().unwrap() as usize,
                slot: case["crate_slot"].as_u64().unwrap() as usize,
            },
            &mut state,
            &content,
        )
        .unwrap();
        let warning = if case["variant"].as_u64().unwrap() == 1 {
            19
        } else {
            147
        };
        assert_eq!(
            events,
            vec![
                Event::Sound { resource_id: 450 },
                Event::CollectionRejected { warning }
            ]
        );
        assert_state(
            &state,
            &native_expected(&before, &case["after"]),
            "native rejected pickup",
        );
        assert_eq!(state.warriors, before.warriors);
        assert_eq!(state.crate_items, before.crate_items);
        assert_eq!(state.random, before.random);
        checked += 1;
    }
    assert_eq!(checked, 8);
}

#[test]
fn invalid_armor_instance_and_occupied_slot_are_atomic() {
    let fixture = fixture();
    let content = content();
    let mut state: Session =
        serde_json::from_value(fixture["foundation"]["session"].clone()).unwrap();
    let slot = state
        .crate_items
        .iter()
        .position(|item| item.id() == 400)
        .unwrap();
    let mut changed = state.clone();
    changed.crate_items[slot].0[14] = 1;
    let before = changed.clone();
    assert!(
        apply(
            Command::TakeCrate { warrior: 1, slot },
            &mut changed,
            &content
        )
        .is_err()
    );
    assert_eq!(changed, before);
    apply(
        Command::TakeCrate { warrior: 1, slot },
        &mut state,
        &content,
    )
    .unwrap();
    let slot = inventory_slot(&state, 1, 400);
    state.warriors[1].bytes[0xd4..0xd6].copy_from_slice(&403_i16.to_be_bytes());
    let before = state.clone();
    assert!(
        apply(
            Command::EquipItem { warrior: 1, slot },
            &mut state,
            &content
        )
        .is_err()
    );
    assert_eq!(state, before);
}
