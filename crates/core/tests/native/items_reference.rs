//! Independent native stockpile and pack-removal comparisons.
use super::*;

pub(super) fn compare_collection() {
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

pub(super) fn compare_drop() {
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
fn stockpile_collection_matches_original_instruction_slice() {
    compare_collection();
}

#[test]
fn carried_pack_removal_matches_original_drop_and_movement_recalculation() {
    compare_drop();
}
