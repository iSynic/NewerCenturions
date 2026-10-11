use centurions_content::{combat::MonsterRecord, mission::OpeningMission};
use centurions_core::{
    RandomState,
    combat::{
        TacticalMap,
        footprint::{find_placement, restore},
    },
};
fn decode(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}
#[test]
fn all_footprint_placement_and_restore_match_original() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/large-placement.json"
    ))
    .unwrap();
    let properties = decode(fixture["properties"].as_str().unwrap());
    let mission = OpeningMission {
        name: "fixture".into(),
        start: (2, 2),
        header: vec![],
        regions: vec![],
        floor: vec![],
        objects: vec![],
        events: vec![],
        tile_properties: properties[64..12064].to_vec(),
        terrain: 2,
        default_tile: 155,
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
    };
    for (index, case) in fixture["cases"].as_array().unwrap().iter().enumerate() {
        let map = TacticalMap {
            tiles: decode(case["map"].as_str().unwrap())
                .as_chunks::<2>()
                .0
                .iter()
                .map(|v| i16::from_be_bytes(*v))
                .collect(),
            shift: (0, 0),
            edge_word: 0,
        };
        let monster = MonsterRecord {
            bytes: decode(case["monster"].as_str().unwrap()),
        };
        let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        let placed =
            find_placement(&map, &monster, &mission, 10, (0, 0), (40, 40), &mut random).unwrap();
        assert_eq!(
            placed.monster.bytes,
            decode(case["after"].as_str().unwrap()),
            "monster case {index}"
        );
        assert_eq!(
            placed
                .map
                .tiles
                .iter()
                .flat_map(|v| v.to_be_bytes())
                .collect::<Vec<_>>(),
            decode(case["tiles"].as_str().unwrap()),
            "map case {index}"
        );
        assert_eq!(
            placed.position,
            (
                case["x"].as_i64().unwrap() as i16,
                case["y"].as_i64().unwrap() as i16
            ),
            "position case {index}"
        );
        assert_eq!(
            random.seed,
            case["seed_after"].as_u64().unwrap() as u32,
            "seed case {index}"
        );
        assert_eq!(
            random.draws,
            case["draws"].as_u64().unwrap(),
            "draws case {index}"
        );
        let restored = restore(&placed.map, &placed.monster, placed.position).unwrap();
        assert_eq!(
            restored
                .tiles
                .iter()
                .flat_map(|v| v.to_be_bytes())
                .collect::<Vec<_>>(),
            decode(case["restored"].as_str().unwrap()),
            "restore case {index}"
        );
    }
}
