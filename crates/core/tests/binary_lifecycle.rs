use centurions_content::{Warrior, combat::MonsterRecord};
use centurions_core::combat::{
    Combat, EnemySetup, SquadPlacement, TacticalMap, lifecycle::kill_body,
};
fn decode(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}
#[test]
fn defeat_cleanup_matches_original_routine() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../evidence/fixtures/lifecycle.json")).unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let mut warriors: Vec<Warrior> = decode(case["warriors"].as_str().unwrap())
            .as_chunks::<3138>()
            .0
            .iter()
            .map(|b| Warrior::read(b).unwrap())
            .collect();
        let mut tiles = vec![1155; 8100];
        for i in 0..6 {
            tiles[(25 + i) * 90 + 25] = i as i16;
        }
        tiles[35 * 90 + 35] = 10;
        let mut order = vec![-1; 110];
        for i in 0..6 {
            order[i + 1] = i as i16;
        }
        order[7] = 10;
        let mut combat = Combat {
            battle: 2,
            map: TacticalMap {
                tiles,
                shift: (0, 0),
                edge_word: 0,
            },
            squad: SquadPlacement {
                positions: (0..6).map(|i| (20 + i, 20)).collect(),
                under_tiles: vec![1155; 6],
            },
            enemies: EnemySetup {
                records: vec![MonsterRecord {
                    bytes: decode(case["enemy"].as_str().unwrap()),
                }],
                positions: vec![(30, 30)],
            },
            order,
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
        kill_body(
            &mut combat,
            &mut warriors,
            case["body"].as_i64().unwrap() as i16,
        )
        .unwrap();
        assert_eq!(
            warriors
                .iter()
                .flat_map(|w| w.bytes.clone())
                .collect::<Vec<_>>(),
            decode(case["after_warriors"].as_str().unwrap()),
            "warriors body {}",
            case["body"]
        );
        assert_eq!(
            combat.enemies.records[0].bytes,
            decode(case["after_enemy"].as_str().unwrap())
        );
        assert_eq!(
            combat
                .order
                .iter()
                .flat_map(|v| v.to_be_bytes())
                .collect::<Vec<_>>(),
            decode(case["order"].as_str().unwrap())
        );
        assert_eq!(
            combat
                .map
                .tiles
                .iter()
                .flat_map(|v| v.to_be_bytes())
                .collect::<Vec<_>>(),
            decode(case["map"].as_str().unwrap())
        );
    }
}
