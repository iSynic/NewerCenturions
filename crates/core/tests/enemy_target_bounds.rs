use centurions_content::{Warrior, combat::MonsterRecord, mission::OpeningMission};
use centurions_core::{
    RandomState,
    combat::{Combat, EnemySetup, SquadPlacement, TacticalMap, runtime::select_enemy_target},
};
fn decode(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn words(bytes: &[u8]) -> Vec<i16> {
    bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|v| i16::from_be_bytes(*v))
        .collect()
}
fn positions(bytes: &[u8]) -> Vec<(i16, i16)> {
    bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|v| {
            (
                i16::from_be_bytes([v[0], v[1]]),
                i16::from_be_bytes([v[2], v[3]]),
            )
        })
        .collect()
}
#[test]
fn invisible_target_scan_stays_in_bounds_without_extra_random_draws() {
    let f: serde_json::Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/opening-enemy-prefix.json"
    ))
    .unwrap();
    let get = |key: &str| decode(f[key].as_str().unwrap());
    let warriors = get("warriors")
        .as_chunks::<3138>()
        .0
        .iter()
        .map(|w| Warrior::read(w).unwrap())
        .collect::<Vec<_>>();
    let records = get("enemies")
        .as_chunks::<288>()
        .0
        .iter()
        .map(|m| MonsterRecord { bytes: m.to_vec() })
        .collect();
    let mut combat = Combat {
        battle: 3,
        map: TacticalMap {
            tiles: words(&get("map")),
            shift: (-30, -36),
            edge_word: 0,
        },
        squad: SquadPlacement {
            positions: positions(&get("squad")),
            under_tiles: vec![1155; 6],
        },
        enemies: EnemySetup {
            records,
            positions: positions(&get("positions")),
        },
        order: words(&get("order")),
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
    let mission = OpeningMission {
        name: "fixture".into(),
        start: (2, 2),
        header: vec![],
        regions: vec![],
        floor: vec![],
        objects: vec![],
        events: vec![],
        tile_properties: get("properties")[64..12064].to_vec(),
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
    let mut random = RandomState::new(f["initial_seed"].as_u64().unwrap() as u32);
    random.bound(100);
    random.bound(100);
    let target = select_enemy_target(&mut combat, &warriors, &mission, &mut random, 13).unwrap();
    assert_eq!(target, f["selected_target"].as_i64().unwrap() as i16);
    assert_eq!(random.seed, f["seed"].as_u64().unwrap() as u32);
    assert_eq!(random.draws, f["draws"].as_u64().unwrap());
    assert_eq!(f["unsafe_target"], 197);
    assert!(target < 6 || (10..16).contains(&target));
}
