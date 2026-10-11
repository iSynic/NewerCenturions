use centurions_content::{combat::MonsterRecord, mission::OpeningMission};
use centurions_core::{
    RandomState,
    combat::{Combat, EnemySetup, SquadPlacement, TacticalMap, enemy::move_enemy},
};
fn decode(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}
#[test]
fn enemy_movement_matches_original_routine() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../evidence/fixtures/enemy-move.json")).unwrap();
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
        let dx = case["dx"].as_i64().unwrap() as i16;
        let dy = case["dy"].as_i64().unwrap() as i16;
        let layout = case["layout"].as_i64().unwrap();
        let mut tiles = vec![1155; 8100];
        if layout == 1 || layout == 3 {
            tiles[((25 + dx) * 90 + 25 + dy) as usize] = if layout == 3 { 1000 } else { 1061 };
        }
        if layout == 2 {
            for x in 24..=26 {
                for y in 24..=26 {
                    tiles[x * 90 + y] = 1061;
                }
            }
        }
        tiles[25 * 90 + 25] = 10;
        tiles[((25 + dx * 4) * 90 + 25 + dy * 4) as usize] = 0;
        let mut combat = Combat {
            battle: 2,
            map: TacticalMap {
                tiles,
                shift: (0, 0),
                edge_word: 0,
            },
            squad: SquadPlacement {
                positions: vec![(20 + dx * 4, 20 + dy * 4); 6],
                under_tiles: vec![1155; 6],
            },
            enemies: EnemySetup {
                records: vec![MonsterRecord {
                    bytes: decode(case["input"].as_str().unwrap()),
                }],
                positions: vec![(20, 20)],
            },
            order: vec![-1, 10],
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
        };
        let mut random = RandomState::new(case["initial_seed"].as_u64().unwrap() as u32);
        let result = move_enemy(&mut combat, &mission, 0, &mut random).unwrap();
        assert_eq!(
            combat.enemies.records[0].bytes,
            decode(case["after"].as_str().unwrap()),
            "enemy case {index}"
        );
        assert_eq!(
            combat
                .map
                .tiles
                .iter()
                .flat_map(|v| v.to_be_bytes())
                .collect::<Vec<_>>(),
            decode(case["tiles"].as_str().unwrap()),
            "map case {index}"
        );
        assert_eq!(
            combat.enemies.positions[0],
            (
                case["x"].as_i64().unwrap() as i16,
                case["y"].as_i64().unwrap() as i16
            ),
            "position case {index}"
        );
        assert_eq!(
            random.seed,
            case["seed"].as_u64().unwrap() as u32,
            "seed case {index}"
        );
        assert_eq!(
            random.draws,
            case["draws"].as_u64().unwrap(),
            "draws case {index}"
        );
        assert_eq!(
            result.moved,
            case["moved"].as_bool().unwrap(),
            "moved case {index}"
        );
        assert_eq!(
            result.sounds,
            case["sounds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|s| s.as_i64().unwrap() as i16)
                .collect::<Vec<_>>(),
            "sounds case {index}"
        );
    }
}
