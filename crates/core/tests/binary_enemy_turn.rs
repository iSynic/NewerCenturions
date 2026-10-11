use centurions_content::{combat::MonsterRecord, mission::OpeningMission};
use centurions_core::{
    RandomState,
    combat::{Combat, EnemySetup, SquadPlacement, TacticalMap, runtime::enemy_action},
};
fn decode(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}
#[test]
fn enemy_turn_matches_original_decision_loop() {
    verify(false);
}

#[test]
fn chemical_enemy_turn_matches_original_decision_loop() {
    verify(true);
}

fn verify(ranged: bool) {
    let fixture: serde_json::Value = serde_json::from_str(if ranged {
        include_str!("../../../evidence/fixtures/enemy-ranged-turn.json")
    } else {
        include_str!("../../../evidence/fixtures/enemy-turn.json")
    })
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
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let content = centurions_content::Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap();
    let mut mission = mission;
    std::sync::Arc::make_mut(&mut mission.shared).combat = content.map(0).unwrap().combat.clone();
    std::sync::Arc::make_mut(&mut mission.shared).areas = content.map(0).unwrap().areas.clone();
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
        let bytes = decode(case["warriors"].as_str().unwrap());
        let mut warriors: Vec<centurions_content::Warrior> = bytes
            .as_chunks::<3138>()
            .0
            .iter()
            .map(|w| centurions_content::Warrior::read(w).unwrap())
            .collect();
        for i in 1..6 {
            combat.squad.positions[i] = if ranged {
                (20 + dx * 4 + i as i16, 20 + dy * 4)
            } else {
                (40 + i as i16, 40)
            };
            if ranged {
                combat.map.tiles[((25 + dx * 4 + i as i16) * 90 + 25 + dy * 4) as usize] = i as i16;
            }
        }
        let _result = enemy_action(
            &mut combat,
            &mut warriors,
            &mission,
            &mut random,
            &content.items,
        )
        .unwrap();
        let actual: Vec<_> = warriors.iter().flat_map(|w| w.bytes.clone()).collect();
        let expected = decode(case["after_warriors"].as_str().unwrap());
        let differences: Vec<_> = actual
            .iter()
            .zip(&expected)
            .enumerate()
            .filter(|(_, (a, b))| a != b)
            .map(|(i, (a, b))| (i / 3138, format!("{:x}", i % 3138), a, b))
            .collect();
        assert!(
            differences.is_empty(),
            "warrior differences case {index}: {differences:?} seed {:?} draws {}",
            random.seed,
            random.draws
        );
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
    }
}
