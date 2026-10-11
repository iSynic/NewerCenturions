use centurions_content::{Content, Warrior, mission::OpeningMission};
use centurions_core::{
    Event, Session,
    combat::{Combat, EnemySetup, SquadPlacement, TacticalMap, move_warrior},
};
fn decode(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}
#[test]
fn walking_matches_instruction_state_and_effects() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../evidence/fixtures/walk.json")).unwrap();
    let properties = decode(fixture["properties"].as_str().unwrap());
    let content = Content {
        identity: "walking".into(),
        warriors: vec![],
        items: vec![],
        resources: vec![],
        original_services: None,
        scenario: Some(centurions_content::mission::ScenarioContent::single(
            OpeningMission {
                name: "fixture".into(),
                start: (2, 2),
                header: vec![],
                regions: vec![],
                floor: vec![],
                objects: vec![],
                events: vec![],
                tile_properties: properties[64..12064].to_vec(),
                terrain: 2,
                default_tile: 0,
                minutes_per_tick: 5,
                shared: std::sync::Arc::new(centurions_content::mission::StoryTables {
                    item_cache: vec![],
                    areas: vec![],
                    races: vec![],
                    profiles: vec![],
                    door_scripts: vec![],
                    dialogue: vec![],
                    treasure: vec![],
                    prologue: String::new(),
                    item_descriptions: vec![],
                    schedule: vec![],
                    scheduled_scripts: vec![],
                    menus: vec![],
                    typed_encounters: vec![],
                    movies: Default::default(),
                    combat: None,
                }),
            },
        )),
    };
    for case in fixture["cases"].as_array().unwrap() {
        let dx = case["dx"].as_i64().unwrap() as i8;
        let dy = case["dy"].as_i64().unwrap() as i8;
        let mut state = Session::new(&content);
        state.warriors = vec![Warrior::read(&decode(case["input"].as_str().unwrap())).unwrap(); 6];
        let mut tiles = vec![1155; 8100];
        tiles[25 * 90 + 25] = 0;
        tiles[(25 + i32::from(dx)) as usize * 90 + (25 + i32::from(dy)) as usize] =
            case["tile"].as_i64().unwrap() as i16;
        state.combat = Some(Combat {
            battle: 2,
            map: TacticalMap {
                tiles,
                shift: (0, 0),
                edge_word: 0,
            },
            squad: SquadPlacement {
                positions: vec![(20, 20); 6],
                under_tiles: vec![1155; 6],
            },
            enemies: EnemySetup {
                records: vec![],
                positions: vec![],
            },
            order: vec![-1, 0],
            cursor: 1,
            round: 0,
            death_queue: Default::default(),
            spell_fields: vec![0; 1200],
            field_used: vec![0; 128],
            attack_packet: Some(vec![0; 58]),
            attack_identity: Some([0; 3]),
            weapon_work: centurions_core::combat::weapon_work::WeaponWork::zero(),
            unavailable: 0,
            round_scripts: Default::default(),
        });
        let events = move_warrior(&mut state, &content, dx, dy).unwrap();
        assert_eq!(
            state.warriors[0].bytes,
            decode(case["after"].as_str().unwrap())
        );
        let combat = state.combat.unwrap();
        assert_eq!(
            combat
                .map
                .tiles
                .iter()
                .flat_map(|v| v.to_be_bytes())
                .collect::<Vec<_>>(),
            decode(case["tiles"].as_str().unwrap())
        );
        assert_eq!(
            combat.squad.positions[0],
            (
                case["x"].as_i64().unwrap() as i16,
                case["y"].as_i64().unwrap() as i16
            )
        );
        assert_eq!(
            combat.squad.under_tiles[0],
            case["under"].as_i64().unwrap() as i16
        );
        let sounds: Vec<_> = events
            .iter()
            .filter_map(|e| {
                if let Event::Sound { resource_id } = e {
                    Some(i64::from(*resource_id))
                } else {
                    None
                }
            })
            .collect();
        let mut expected_sounds = case["sounds"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_i64().unwrap())
            .collect::<Vec<_>>();
        // This probe stops at the failure label before the caller's final sound.
        // The complete caller is covered by binary_warrior_opportunity.
        if !events
            .iter()
            .any(|event| matches!(event, Event::TacticalMoved { .. }))
        {
            expected_sounds.push(-130);
        }
        assert_eq!(sounds, expected_sounds);
        assert_eq!(state.random.draws, 0);
    }
}
