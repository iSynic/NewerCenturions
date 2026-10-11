use centurions_content::{Warrior, combat::MonsterRecord, mission::OpeningMission};
use centurions_core::{
    RandomState,
    combat::{
        Combat, EnemySetup, SquadPlacement, TacticalMap,
        allies::{place_allies, synchronize},
        runtime::neighbors,
    },
};

fn decode(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}
fn bytes(case: &serde_json::Value, field: &str) -> Vec<u8> {
    decode(case[field].as_str().unwrap())
}
fn words(bytes: &[u8]) -> Vec<i16> {
    bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|v| i16::from_be_bytes(*v))
        .collect()
}
fn records(bytes: &[u8]) -> Vec<MonsterRecord> {
    bytes
        .as_chunks::<288>()
        .0
        .iter()
        .map(|v| MonsterRecord { bytes: v.to_vec() })
        .collect()
}
fn fixture() -> serde_json::Value {
    serde_json::from_str(include_str!("../../../evidence/fixtures/ally-combat.json")).unwrap()
}

#[test]
fn ally_placement_matches_original_records_positions_and_map() {
    let fixture = fixture();
    let properties = bytes(&fixture, "properties");
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
        shared: Default::default(),
    };
    for case in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| v["kind"] == "placement")
    {
        let mut map = TacticalMap {
            tiles: words(&bytes(case, "map_before")),
            shift: (-39, -15),
            edge_word: 0,
        };
        let menu = bytes(case, "menu_before");
        let mut random = RandomState::new(12);
        let result = place_allies(
            &menu,
            case["count"].as_u64().unwrap() as usize,
            &mut map,
            &mission,
            (39, 39),
            (5, 5, 85, 85),
            &mut random,
        )
        .unwrap();
        assert_eq!(
            result
                .records
                .iter()
                .flat_map(|m| &m.bytes)
                .copied()
                .collect::<Vec<_>>(),
            bytes(case, "records_after")
        );
        assert_eq!(
            result.positions,
            words(&bytes(case, "positions"))
                .as_chunks::<2>()
                .0
                .iter()
                .map(|p| (p[0], p[1]))
                .collect::<Vec<_>>()
        );
        assert_eq!(map.tiles, words(&bytes(case, "map_after")));
        assert_eq!(menu, bytes(case, "menu_after"));
        assert_eq!(
            result.records.len(),
            case["enemy_count"].as_u64().unwrap() as usize
        );
        assert_eq!(random, RandomState::new(12));
    }
}

#[test]
fn original_bodycount_preserves_selection_order_rng_and_empty_survivor_quirk() {
    let fixture = fixture();
    for case in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| v["kind"] == "synchronization")
    {
        let mut records = records(&bytes(case, "records_before"));
        let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        let result = synchronize(
            &bytes(case, "menu_before"),
            case["count"].as_u64().unwrap() as usize,
            &mut records,
            &mut random,
        )
        .unwrap();
        assert_eq!(
            result.menu,
            bytes(case, "menu_after"),
            "count {} dead {} seed {}",
            case["count"],
            case["dead"],
            case["seed_before"]
        );
        assert_eq!(result.count, case["count_after"].as_u64().unwrap() as usize);
        assert_eq!(result.modal, case["dialogs"].as_u64().unwrap() > 0);
        assert_eq!(
            result.sounds,
            case["sounds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_i64().unwrap() as i16)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            records
                .iter()
                .flat_map(|m| &m.bytes)
                .copied()
                .collect::<Vec<_>>(),
            bytes(case, "records_after")
        );
        assert_eq!(random.seed, case["seed_after"].as_u64().unwrap() as u32);
        assert_eq!(random.draws, case["rng_draws"].as_u64().unwrap());
    }
}

#[test]
fn adjacent_faction_targets_match_original_warrior_scientist_and_hostile_lists() {
    let fixture = fixture();
    for case in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| v["kind"] == "targets")
    {
        let combat = Combat {
            battle: 0,
            map: TacticalMap {
                tiles: words(&bytes(case, "map")),
                shift: (-5, -5),
                edge_word: 0,
            },
            squad: SquadPlacement {
                positions: vec![(25, 25)],
                under_tiles: vec![1155],
            },
            enemies: EnemySetup {
                records: records(&bytes(case, "records_before")),
                positions: vec![(24, 25), (25, 24)],
            },
            order: vec![],
            cursor: 0,
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
        let warriors = vec![Warrior {
            name: "fixture".into(),
            original_sha256: String::new(),
            bytes: bytes(case, "warrior_before"),
        }];
        let actual = neighbors(&combat, &warriors, case["actor"].as_i64().unwrap() as i16).unwrap();
        let expected = words(&bytes(case, "targets"))
            [..case["target_count"].as_u64().unwrap() as usize]
            .to_vec();
        assert_eq!(
            actual, expected,
            "actor {} dead {}",
            case["actor"], case["dead"]
        );
        assert_eq!(bytes(case, "records_before"), bytes(case, "records_after"));
        assert_eq!(bytes(case, "warrior_before"), bytes(case, "warrior_after"));
    }
}
