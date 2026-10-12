use super::*;
fn decode(value: &str) -> Vec<u8> {
    (0..value.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&value[i..i + 2], 16).unwrap())
        .collect()
}
#[test]
fn stockpile_collection_matches_original_instruction_slice() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../evidence/fixtures/ammunition-collection.json"
    ))
    .unwrap();
    let mut items = vec![vec![0; 150]; 1000];
    items[200] = decode(fixture["definition"].as_str().unwrap());
    let content = Content {
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
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../evidence/fixtures/ammunition-drop.json"
    ))
    .unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let mut warrior = Warrior::read(&decode(case["input"].as_str().unwrap())).unwrap();
        remove_inventory_slot(&mut warrior, 1).unwrap();
        let expected = decode(case["after"].as_str().unwrap());
        let differences: Vec<_> = warrior
            .bytes
            .iter()
            .zip(&expected)
            .enumerate()
            .filter(|(_, (a, b))| a != b)
            .map(|(i, (a, b))| (format!("{i:x}"), *a, *b))
            .collect();
        assert!(differences.is_empty(), "{} {differences:?}", case["name"]);
    }
}

#[test]
fn pack_command_is_atomic_and_resumes_with_the_same_events() {
    let calendar: serde_json::Value =
        serde_json::from_str(include_str!("../../../../evidence/fixtures/calendar.json")).unwrap();
    let ammo: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../evidence/fixtures/ammunition-init.json"
    ))
    .unwrap();
    let warriors = decode(calendar["cases"][0]["input"].as_str().unwrap())
        .as_chunks::<3138>()
        .0
        .iter()
        .map(|b| Warrior::read(b).unwrap())
        .collect();
    let mut definitions = vec![vec![0; 150]; 1000];
    definitions[200] = decode(ammo["definition"].as_str().unwrap());
    let map = centurions_content::mission::OpeningMission {
        name: "The Thunder of Guns".into(),
        start: (2, 2),
        header: vec![0; 4200],
        regions: vec![0; 644],
        floor: vec![1; 8100],
        objects: vec![0; 8100],
        events: vec![0; 8100],
        tile_properties: vec![0; 12000],
        terrain: 2,
        default_tile: 1,
        minutes_per_tick: 5,
        shared: std::sync::Arc::new(centurions_content::mission::StoryTables {
            areas: vec![],
            item_cache: vec![],
            races: vec![],
            profiles: vec![],
            door_scripts: vec![],
            dialogue: vec![],
            treasure: vec![],
            prologue: String::new(),
            item_descriptions: vec![],
            schedule: vec![0; 40],
            scheduled_scripts: vec![],
            menus: vec![],
            typed_encounters: vec![],
            movies: Default::default(),
            combat: None,
        }),
    };
    let content = Content {
        identity: "fixture".into(),
        warriors,
        items: definitions,
        resources: vec![],
        original_services: None,
        scenario: Some(centurions_content::mission::ScenarioContent::single(map)),
    };
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
    for (id, quantity, equipped) in [(201, 32, 0), (200, 0, 0), (200, 32, 1), (1, 32, 0)] {
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
    assert!(
        apply(
            Command::UseAmmunitionPack {
                warrior: 6,
                slot: 0
            },
            &mut state,
            &content
        )
        .is_err()
    );
    let mut modal = resumed.clone();
    modal.exploration.as_mut().unwrap().script = Some(ScriptState {
        record: vec![0; 42],
        cursor: 0,
        map_slot: 0,
        scheduled: false,
        menu: None,
        persist: false,
        origin: None,
    });
    modal.exploration.as_mut().unwrap().presentation = Some(Presentation::Loot);
    let before = modal.clone();
    assert!(
        apply(
            Command::UseAmmunitionPack {
                warrior: 0,
                slot: 0
            },
            &mut modal,
            &content
        )
        .is_err()
    );
    assert_eq!(modal, before);
    fs::remove_file(path).unwrap();
}
