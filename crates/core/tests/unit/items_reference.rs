use super::*;
use crate::test_support::decode;
#[test]
fn stockpile_collection_matches_original_instruction_slice() {
    let fixture = crate::test_support::fixture("ammunition-collection.json");
    let mut items = vec![vec![0; 150]; 1000];
    items[200] = decode(fixture["definition"].as_str().unwrap());
    let content = Content {
        tutorials: vec![],
        identity: "fixture".into(),
        warriors: vec![],
        items,
        resources: vec![],
        original_services: None,
        scenario: None,
    };
    for case in fixture["cases"].as_array().unwrap() {
        let mut state = Session::new(&content);
        state.random = RandomState::new(case["initial_seed"].as_u64().unwrap() as u32);
        state.ammunition[0] = case["reserve"].as_i64().unwrap() as i32;
        let item = InventoryRecord::read(&decode(case["input"].as_str().unwrap())).unwrap();
        let events = add_ammunition(&mut state, &content, &item).unwrap();
        assert_eq!(
            state.ammunition[0],
            case["after_reserve"].as_i64().unwrap() as i32
        );
        assert_eq!(state.random.seed, case["seed"].as_u64().unwrap() as u32);
        assert_eq!(state.random.draws, case["draws"].as_u64().unwrap());
        assert_eq!(
            events
                .iter()
                .filter_map(|e| if let Event::Sound { resource_id } = e {
                    Some(*resource_id)
                } else {
                    None
                })
                .collect::<Vec<_>>(),
            case["sounds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_i64().unwrap() as i16)
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn carried_pack_removal_matches_original_drop_and_movement_recalculation() {
    let content = crate::test_support::content();
    let fixture = crate::test_support::fixture("ammunition-drop.json");
    for case in fixture["cases"].as_array().unwrap() {
        let warrior = Warrior::read(&decode(case["input"].as_str().unwrap())).unwrap();
        let mut state = Session::new(&content);
        state.warriors = vec![warrior.clone()];
        apply(
            Command::EnterMission {
                name: "The Thunder of Guns".into(),
            },
            &mut state,
            &content,
        )
        .unwrap();
        apply(
            Command::DropItem {
                warrior: 0,
                slot: 1,
            },
            &mut state,
            &content,
        )
        .unwrap();
        let expected = decode(case["after"].as_str().unwrap());
        assert_eq!(state.warriors[0].bytes, expected);
    }
}

#[test]
fn pack_command_is_atomic_and_resumes_with_the_same_events() {
    let content = crate::test_support::content();
    let ammo = crate::test_support::fixture("ammunition-init.json");
    let mut state = Session::new(&content);
    apply(Command::ImportStarters, &mut state, &content).unwrap();
    apply(
        Command::EnterMission {
            name: "The Thunder of Guns".into(),
        },
        &mut state,
        &content,
    )
    .unwrap();
    state.exploration.as_mut().unwrap().presentation = None;
    state.exploration.as_mut().unwrap().script = None;
    state.movement_continuation = None;
    let pack = decode(ammo["cases"][0]["record"].as_str().unwrap());
    state.warriors[0].bytes[0x132 + 66..0x132 + 88].copy_from_slice(&pack);
    state.warriors[0].bytes[0x1a..0x1c].copy_from_slice(&4i16.to_be_bytes());
    let path = std::env::temp_dir().join(format!("centurions-pack-{}.json", std::process::id()));
    save(&path, &state, &content).unwrap();
    let mut resumed = load(&path, &content).unwrap();
    let command = Command::UseAmmunitionPack {
        warrior: 0,
        slot: 3,
    };
    let expected = apply(command.clone(), &mut state, &content).unwrap();
    assert_eq!(apply(command, &mut resumed, &content).unwrap(), expected);
    assert_eq!(state, resumed);
    assert_eq!(state.ammunition[0], 32);
    assert_eq!(state.warriors[0].inventory().unwrap().len(), 3);
    for (id, quantity, equipped) in [(200, 0, 0), (200, 32, 1), (1, 32, 0)] {
        let mut rejected = resumed.clone();
        let w = &mut rejected.warriors[0];
        w.bytes[0x132 + 66..0x132 + 88].copy_from_slice(&pack);
        w.bytes[0x132 + 66..0x134 + 66].copy_from_slice(&(id as i16).to_be_bytes());
        w.bytes[0x134 + 66..0x136 + 66].copy_from_slice(&(quantity as i16).to_be_bytes());
        w.bytes[0x132 + 66 + 9] = equipped;
        w.bytes[0x1a..0x1c].copy_from_slice(&4i16.to_be_bytes());
        let before = rejected.clone();
        assert!(
            apply(
                Command::UseAmmunitionPack {
                    warrior: 0,
                    slot: 3
                },
                &mut rejected,
                &content
            )
            .is_err()
        );
        assert_eq!(rejected, before);
    }
    fs::remove_file(path).unwrap();
}
