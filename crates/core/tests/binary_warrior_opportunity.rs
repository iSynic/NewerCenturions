use centurions_content::{Content, Warrior, combat::MonsterRecord};
use centurions_core::{
    Event, RandomState, Session,
    combat::{Combat, EnemySetup, SquadPlacement, TacticalMap, move_warrior},
};

fn bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|at| u8::from_str_radix(&hex[at..at + 2], 16).unwrap())
        .collect()
}
fn words(raw: &[u8]) -> Vec<i16> {
    raw.as_chunks::<2>()
        .0
        .iter()
        .map(|word| i16::from_be_bytes(*word))
        .collect()
}
fn assert_bytes(actual: &[u8], expected: &[u8], context: &str) {
    assert_eq!(actual.len(), expected.len(), "{context} size");
    if let Some(at) = actual.iter().zip(expected).position(|(a, b)| a != b) {
        panic!("{context} byte {at:x}: {} != {}", actual[at], expected[at]);
    }
}

#[test]
fn warrior_walk_matches_native_factions_guard_departure_order_and_cost() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/warrior-opportunity.json"
    ))
    .unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut content = Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap();
    content.scenario.as_mut().unwrap().maps[0].tile_properties =
        bytes(fixture["properties"].as_str().unwrap())[64..12064].to_vec();
    let mut checked = 0;
    for case in fixture["cases"].as_array().unwrap() {
        if case["forced_kill_return"].as_bool().unwrap() {
            continue;
        }
        let context = format!("layout{} seed{}", case["layout"], case["seed_before"]);
        let mut state = Session::new(&content);
        state.random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        state.warriors = bytes(case["warriors_before"].as_str().unwrap())
            .as_chunks::<3138>()
            .0
            .iter()
            .map(|record| Warrior::read(record).unwrap())
            .collect();
        let mut order = vec![-1; 110];
        order[1] = 0;
        state.combat = Some(Combat {
            battle: 1,
            map: TacticalMap {
                tiles: words(&bytes(case["map_before"].as_str().unwrap())),
                shift: (0, 0),
                edge_word: 0,
            },
            squad: SquadPlacement {
                positions: (0..6)
                    .map(|member| {
                        if member == 0 {
                            (20, 20)
                        } else {
                            (40 + member, 40)
                        }
                    })
                    .collect(),
                under_tiles: vec![1155; 6],
            },
            enemies: EnemySetup {
                records: bytes(case["records_before"].as_str().unwrap())
                    .as_chunks::<288>()
                    .0
                    .iter()
                    .map(|record| MonsterRecord {
                        bytes: record.to_vec(),
                    })
                    .collect(),
                positions: case["enemy_positions"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|point| {
                        (
                            point[0].as_i64().unwrap() as i16,
                            point[1].as_i64().unwrap() as i16,
                        )
                    })
                    .collect(),
            },
            order,
            cursor: 1,
            round: 0,
            round_scripts: Default::default(),
            death_queue: Default::default(),
            spell_fields: vec![0; 1200],
            field_used: vec![0; 128],
            attack_packet: Some(vec![0; 58]),
            attack_identity: Some([0; 3]),
            weapon_work: centurions_core::combat::weapon_work::WeaponWork::zero(),
            unavailable: 0,
        });
        let events = move_warrior(
            &mut state,
            &content,
            case["dx"].as_i64().unwrap() as i8,
            case["dy"].as_i64().unwrap() as i8,
        )
        .unwrap_or_else(|error| panic!("{context}: {error}"));
        let combat = state.combat.as_ref().unwrap();
        assert_bytes(
            &state
                .warriors
                .iter()
                .flat_map(|w| w.bytes.iter().copied())
                .collect::<Vec<_>>(),
            &bytes(case["warriors_after"].as_str().unwrap()),
            &format!("{context} party"),
        );
        assert_bytes(
            &combat
                .enemies
                .records
                .iter()
                .flat_map(|r| r.bytes.iter().copied())
                .collect::<Vec<_>>(),
            &bytes(case["records_after"].as_str().unwrap()),
            &format!("{context} monsters"),
        );
        assert_eq!(
            combat.map.tiles,
            words(&bytes(case["map_after"].as_str().unwrap())),
            "{context} map"
        );
        assert_eq!(
            combat.squad.positions[0],
            (
                case["x"].as_i64().unwrap() as i16,
                case["y"].as_i64().unwrap() as i16
            ),
            "{context} position"
        );
        assert_eq!(
            combat.squad.under_tiles[0],
            case["under"].as_i64().unwrap() as i16,
            "{context} under tile"
        );
        assert_bytes(
            combat.attack_packet.as_ref().unwrap(),
            &bytes(case["packet"].as_str().unwrap()),
            &format!("{context} packet"),
        );
        assert_eq!(
            state.random.seed,
            case["seed_after"].as_u64().unwrap() as u32,
            "{context} seed"
        );
        assert_eq!(
            state.random.draws,
            case["rng_draws"].as_u64().unwrap(),
            "{context} draws"
        );
        let sounds: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                Event::Sound { resource_id } => Some(i64::from(*resource_id)),
                _ => None,
            })
            .collect();
        assert_eq!(
            sounds,
            case["sounds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_i64().unwrap())
                .collect::<Vec<_>>(),
            "{context} sounds"
        );
        let calls: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                Event::AttackResolved {
                    attacker, target, ..
                } => Some((i64::from(*attacker), i64::from(*target))),
                _ => None,
            })
            .collect();
        assert_eq!(
            calls,
            case["calls"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| (v[0].as_i64().unwrap(), v[1].as_i64().unwrap()))
                .collect::<Vec<_>>(),
            "{context} reaction order"
        );
        assert_eq!(combat.cursor, 1);
        assert_eq!(combat.order[1], 0);
        assert_eq!(combat.round, 0);
        checked += 1;
    }
    assert_eq!(checked, 38);
}

#[test]
fn native_nonzero_reaction_return_advances_before_movement_cost_or_map_writes() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/warrior-opportunity.json"
    ))
    .unwrap();
    let cases: Vec<_> = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["forced_kill_return"].as_bool().unwrap())
        .collect();
    assert_eq!(cases.len(), 3);
    for case in cases {
        assert_eq!(case["phases"], serde_json::json!([0, 1, 2]));
        assert_eq!(case["advance_calls"], 1);
        assert_eq!(
            (case["x"].as_i64().unwrap(), case["y"].as_i64().unwrap()),
            (20, 20)
        );
        assert_eq!(case["map_after"], case["map_before"]);
        let mut expected = bytes(case["warriors_before"].as_str().unwrap());
        expected[0x9e0] = 0;
        assert_bytes(
            &bytes(case["warriors_after"].as_str().unwrap()),
            &expected,
            "aborted walk facing",
        );
        assert_eq!(case["rng_draws"], 0);
        assert_eq!(
            case["calls"].as_array().unwrap().last().unwrap(),
            &serde_json::json!([10, 0, 0, 2])
        );
    }
}
