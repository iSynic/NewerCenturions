use centurions_content::{Content, Warrior, combat::MonsterRecord, word};
use centurions_core::{
    Command, Event, Phase, Session, apply,
    combat::{Combat, EnemySetup, SquadPlacement, TacticalMap},
    load, save,
};
use serde_json::Value;

fn fixture() -> Value {
    serde_json::from_str(
        &std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../evidence/fixtures/required-player-gear.json"),
        )
        .unwrap(),
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

fn bytes(hex: &str) -> Vec<u8> {
    hex.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

fn campaign_input(fixture: &Value, source: usize) -> Session {
    let mut state: Session =
        serde_json::from_value(fixture["foundations"][source]["session"].clone()).unwrap();
    state.psi_memory = Some(
        state
            .warriors
            .iter()
            .flat_map(|warrior| warrior.bytes[0xa76..0xac6].iter().copied())
            .collect(),
    );
    state
}

fn crate_slot(state: &Session, item: i16) -> usize {
    state
        .crate_items
        .iter()
        .position(|record| record.id() == item)
        .unwrap()
}

fn inventory_slot(state: &Session, warrior: usize, item: i16) -> usize {
    state.warriors[warrior]
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

fn assert_party(state: &Session, snapshot: &Value, context: &str) {
    let actual: Vec<_> = state
        .warriors
        .iter()
        .flat_map(|warrior| warrior.bytes.iter().copied())
        .collect();
    let expected = bytes(snapshot["party"].as_str().unwrap());
    if let Some(offset) = actual.iter().zip(&expected).position(|(a, b)| a != b) {
        panic!(
            "{context}: byte {offset:#x}: {} != {}",
            actual[offset], expected[offset]
        );
    }
    assert_eq!(actual.len(), expected.len());
}

fn take(state: &mut Session, content: &Content, item: i16, warrior: usize) -> Vec<Event> {
    apply(
        Command::TakeCrate {
            slot: crate_slot(state, item),
            warrior,
        },
        state,
        content,
    )
    .unwrap()
}

#[test]
fn actual_campaign_gear_commands_preserve_instances_events_and_save_resume() {
    let fixture = fixture();
    let content = content();
    let mut uninterrupted = campaign_input(&fixture, 2);
    let before = uninterrupted.clone();
    assert_eq!(word(&before.warriors[3].bytes, 0x3a).unwrap(), 4);
    let source_items: Vec<_> = [420, 38]
        .into_iter()
        .map(|id| before.crate_items[crate_slot(&before, id)].clone())
        .collect();
    let path = std::env::temp_dir().join(format!(
        "centurions-required-player-gear-sequence-{}.json",
        std::process::id()
    ));
    let mut resumed = checkpoint(&uninterrupted, &content, &path);
    for (step, item, operation) in [
        (0, 420, 0),
        (1, 420, 1),
        (2, 38, 0),
        (3, 38, 1),
        (4, 38, 2),
        (5, 38, 1),
        (6, 420, 2),
        (7, 420, 1),
    ] {
        let command = match operation {
            0 => Command::TakeCrate {
                slot: crate_slot(&uninterrupted, item),
                warrior: 3,
            },
            1 => Command::EquipItem {
                warrior: 3,
                slot: inventory_slot(&uninterrupted, 3, item),
            },
            _ => Command::UnequipItem {
                warrior: 3,
                slot: inventory_slot(&uninterrupted, 3, item),
            },
        };
        let events = apply(command.clone(), &mut uninterrupted, &content).unwrap();
        assert_eq!(
            events,
            vec![Event::Sound {
                resource_id: if item == 38 { 397 } else { 450 }
            }],
            "step {step}"
        );
        assert_eq!(
            apply(command, &mut resumed, &content).unwrap(),
            events,
            "resumed step {step}"
        );
        assert_eq!(resumed, uninterrupted, "complete session step {step}");
        assert_eq!(
            uninterrupted.loaded_item_field32,
            Some(if item == 38 { 4 } else { 0 })
        );
        if step == 2 || step == 3 || step == 7 {
            resumed = checkpoint(&resumed, &content, &path);
        }
    }
    assert_eq!(uninterrupted.random, before.random);
    assert_eq!(word(&uninterrupted.warriors[3].bytes, 0x3a).unwrap(), 4);
    assert_eq!(word(&uninterrupted.warriors[3].bytes, 0xe4).unwrap(), 38);
    assert_eq!(word(&uninterrupted.warriors[3].bytes, 0xd2).unwrap(), 420);
    for source in source_items {
        assert!(
            uninterrupted
                .crate_items
                .iter()
                .all(|record| record.id() != source.id())
        );
        let inventory = uninterrupted.warriors[3].inventory().unwrap();
        let matching: Vec<_> = inventory
            .iter()
            .filter(|record| record.id() == source.id())
            .collect();
        assert_eq!(matching.len(), 1);
        assert_eq!(&matching[0].0[..8], &source.0[..8]);
        assert_eq!(&matching[0].0[13..], &source.0[13..]);
        assert_eq!(matching[0].0[9], 1);
    }
    assert_eq!(uninterrupted.ammunition, before.ammunition);
    assert_eq!(uninterrupted.warriors[0], before.warriors[0]);
    assert_eq!(uninterrupted.warriors[1], before.warriors[1]);
    assert_eq!(uninterrupted.warriors[2], before.warriors[2]);
    assert_eq!(uninterrupted.warriors[4], before.warriors[4]);
    assert_eq!(uninterrupted.warriors[5], before.warriors[5]);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn public_equipment_commands_match_native_phase_snapshots_and_resume() {
    let fixture = fixture();
    let content = content();
    let path = std::env::temp_dir().join(format!(
        "centurions-required-player-gear-native-{}.json",
        std::process::id()
    ));
    let mut checked = 0;
    for case in fixture["equipment_cases"].as_array().unwrap() {
        if case["source"].as_u64().unwrap() != 2
            || !matches!(case["member"].as_u64().unwrap(), 0 | 3)
        {
            continue;
        }
        checked += 1;
        let member = case["member"].as_u64().unwrap() as usize;
        let id = case["item_id"].as_i64().unwrap() as i16;
        let mut state = if case["combat"].as_u64().unwrap() != 0 {
            combat_input(&fixture, member)
        } else {
            campaign_input(&fixture, 2)
        };
        state.warriors = bytes(case["before"]["party"].as_str().unwrap())
            .as_chunks::<3138>()
            .0
            .iter()
            .map(|record| Warrior::read(record).unwrap())
            .collect();
        state.loaded_item_field32 =
            Some(word(&bytes(case["before"]["item_work"].as_str().unwrap()), 32).unwrap());
        let initial = state.clone();
        let mut resumed = checkpoint(&state, &content, &path);
        let command = Command::EquipItem {
            warrior: member,
            slot: case["inventory_slot"].as_u64().unwrap() as usize,
        };
        let events = apply(command.clone(), &mut state, &content).unwrap();
        assert_eq!(apply(command, &mut resumed, &content).unwrap(), events);
        assert_eq!(state, resumed);
        assert_party(&state, &case["compacted"], "public wear+compact");
        assert_eq!(
            state.health_context.as_ref().unwrap().affected.to_vec(),
            bytes(case["compacted"]["markers"].as_str().unwrap())
                .as_chunks::<2>()
                .0
                .iter()
                .take(6)
                .map(|value| i16::from_be_bytes(*value))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            state.loaded_item_field32,
            Some(word(&bytes(case["compacted"]["item_work"].as_str().unwrap()), 32).unwrap())
        );
        assert_eq!(
            events,
            vec![Event::Sound {
                resource_id: if id == 38 { 397 } else { 450 }
            }]
        );
        resumed = checkpoint(&resumed, &content, &path);
        let command = Command::UnequipItem {
            warrior: member,
            slot: inventory_slot(&state, member, id),
        };
        let events = apply(command.clone(), &mut state, &content).unwrap();
        assert_eq!(apply(command, &mut resumed, &content).unwrap(), events);
        assert_eq!(state, resumed);
        assert_party(&state, &case["removed_state"], "public remove");
        assert_eq!(
            state.loaded_item_field32,
            Some(
                word(
                    &bytes(case["removed_state"]["item_work"].as_str().unwrap()),
                    32
                )
                .unwrap()
            )
        );
        let command = Command::EquipItem {
            warrior: member,
            slot: inventory_slot(&state, member, id),
        };
        let events = apply(command.clone(), &mut state, &content).unwrap();
        assert_eq!(apply(command, &mut resumed, &content).unwrap(), events);
        assert_eq!(state, resumed);
        assert_party(&state, &case["reworn_state"], "public rewear");
        assert_eq!(
            state.loaded_item_field32,
            Some(
                word(
                    &bytes(case["reworn_state"]["item_work"].as_str().unwrap()),
                    32
                )
                .unwrap()
            )
        );
        assert_eq!(state.random, initial.random);
        let initial_inventory = initial.warriors[member].inventory().unwrap();
        let original_item = &initial_inventory[case["inventory_slot"].as_u64().unwrap() as usize];
        let inventory = state.warriors[member].inventory().unwrap();
        let reworn_item = &inventory[inventory_slot(&state, member, id)];
        assert_eq!(&reworn_item.0[..8], &original_item.0[..8]);
        assert_eq!(&reworn_item.0[13..], &original_item.0[13..]);
        assert_eq!(reworn_item.0[9], 1);
        let mut unaffected = state.clone();
        unaffected.warriors.clone_from(&initial.warriors);
        assert_eq!(
            state.ammunition_work.item,
            Some(bytes(case["reworn_state"]["item_work"].as_str().unwrap()))
        );
        unaffected
            .ammunition_work
            .item
            .clone_from(&initial.ammunition_work.item);
        unaffected.loaded_item_field32 = initial.loaded_item_field32;
        unaffected
            .health_context
            .clone_from(&initial.health_context);
        let actual = serde_json::to_value(&unaffected).unwrap();
        let expected = serde_json::to_value(&initial).unwrap();
        let changed: Vec<_> = actual
            .as_object()
            .unwrap()
            .iter()
            .filter_map(|(key, value)| (expected[key] != *value).then_some(key))
            .collect();
        assert!(
            changed.is_empty(),
            "equipment changed session fields {changed:?}"
        );
        checkpoint(&state, &content, &path);
    }
    assert_eq!(checked, 8);
    std::fs::remove_file(path).unwrap();
}

fn assert_atomic(state: &Session, content: &Content, command: Command, context: &str) {
    let mut candidate = state.clone();
    let before = candidate.clone();
    assert!(
        apply(command, &mut candidate, content).is_err(),
        "{context}"
    );
    assert_eq!(candidate, before, "{context} complete session");
}

fn combat_input(fixture: &Value, member: usize) -> Session {
    let case = fixture["equipment_cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| {
            case["source"].as_u64() == Some(2)
                && case["member"].as_u64() == Some(member as u64)
                && case["item_id"].as_u64() == Some(38)
                && case["combat"].as_u64() == Some(1)
        })
        .unwrap();
    let mut state = campaign_input(fixture, 2);
    state.warriors = bytes(case["before"]["party"].as_str().unwrap())
        .as_chunks::<3138>()
        .0
        .iter()
        .map(|record| Warrior::read(record).unwrap())
        .collect();
    let mut order = vec![-1; 110];
    order[1] = member as i16;
    order[2] = 10;
    let mut tiles = vec![1155; 8100];
    tiles[25 * 90 + 25] = 10;
    state.phase = Phase::Combat;
    state.combat = Some(Combat {
        battle: 85,
        map: TacticalMap {
            tiles,
            shift: (0, 0),
            edge_word: 0,
        },
        squad: SquadPlacement {
            positions: vec![(20, 10); 6],
            under_tiles: vec![1155; 6],
        },
        enemies: EnemySetup {
            records: vec![MonsterRecord {
                bytes: bytes(case["before"]["target"].as_str().unwrap()),
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
        attack_packet: Some(vec![0; 58]),
        attack_identity: Some([0; 3]),
        weapon_work: state.weapon_work,
        unavailable: 0,
    });
    state.attack_packet = state.combat.as_ref().unwrap().attack_packet.clone();
    state.attack_identity = state.combat.as_ref().unwrap().attack_identity;
    state
}

#[test]
fn invalid_and_unsupported_equipment_commands_leave_complete_state_unchanged() {
    let fixture = fixture();
    let content = content();
    let foundation = campaign_input(&fixture, 2);
    let other = foundation
        .crate_items
        .iter()
        .position(|item| item.id() != 0 && !matches!(item.id(), 38 | 420))
        .unwrap();
    let empty = crate_slot(&foundation, 0);
    for (command, context) in [
        (
            Command::TakeCrate {
                slot: other,
                warrior: 3,
            },
            "unsupported crate item",
        ),
        (
            Command::TakeCrate {
                slot: empty,
                warrior: 3,
            },
            "empty crate slot",
        ),
        (
            Command::TakeCrate {
                slot: 1000,
                warrior: 3,
            },
            "missing crate slot",
        ),
        (
            Command::TakeCrate {
                slot: crate_slot(&foundation, 38),
                warrior: 6,
            },
            "missing recipient",
        ),
        (
            Command::EquipItem {
                warrior: 3,
                slot: 100,
            },
            "missing inventory slot",
        ),
        (
            Command::UnequipItem {
                warrior: 3,
                slot: 100,
            },
            "missing removal slot",
        ),
    ] {
        assert_atomic(&foundation, &content, command, context);
    }
    let mut mos = foundation.clone();
    take(&mut mos, &content, 38, 1);
    assert_atomic(
        &mos,
        &content,
        Command::EquipItem {
            warrior: 1,
            slot: inventory_slot(&mos, 1, 38),
        },
        "MOS rejection",
    );
    let mut low_level = campaign_input(&fixture, 0);
    take(&mut low_level, &content, 38, 3);
    assert_eq!(word(&low_level.warriors[3].bytes, 0x3a).unwrap(), 3);
    assert_atomic(
        &low_level,
        &content,
        Command::EquipItem {
            warrior: 3,
            slot: inventory_slot(&low_level, 3, 38),
        },
        "actual level three rejection",
    );
    let mut combat = combat_input(&fixture, 3);
    let slot = inventory_slot(&combat, 3, 38);
    combat.combat.as_mut().unwrap().order[1] = 0;
    assert_atomic(
        &combat,
        &content,
        Command::EquipItem { warrior: 3, slot },
        "inactive combat warrior",
    );
    combat.combat.as_mut().unwrap().order[1] = 3;
    combat.warriors[3].bytes[0x40..0x42].copy_from_slice(&0_i16.to_be_bytes());
    assert_atomic(
        &combat,
        &content,
        Command::EquipItem { warrior: 3, slot },
        "zero AP equipment",
    );
    let mut equipped = combat_input(&fixture, 3);
    apply(
        Command::EquipItem {
            warrior: 3,
            slot: inventory_slot(&equipped, 3, 38),
        },
        &mut equipped,
        &content,
    )
    .unwrap();
    let slot = inventory_slot(&equipped, 3, 38);
    equipped.warriors[3].bytes[0x40..0x42].copy_from_slice(&0_i16.to_be_bytes());
    let mut at_zero_ap = equipped.clone();
    let removal = apply(
        Command::UnequipItem { warrior: 3, slot },
        &mut at_zero_ap,
        &content,
    )
    .unwrap();
    assert_eq!(word(&at_zero_ap.warriors[3].bytes, 0x40).unwrap(), 0);
    assert_eq!(removal, vec![Event::Sound { resource_id: 397 }]);
    equipped.warriors[3].bytes[0x40..0x42].copy_from_slice(&5_i16.to_be_bytes());
    equipped.combat.as_mut().unwrap().order[1] = 0;
    assert_atomic(
        &equipped,
        &content,
        Command::UnequipItem { warrior: 3, slot },
        "inactive removal",
    );
}
