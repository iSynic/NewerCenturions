use centurions_content::{Warrior, combat::MonsterRecord, mission::OpeningMission};
use centurions_core::{
    Event, RandomState,
    combat::{
        Combat, EnemySetup, SquadPlacement, TacticalMap,
        ally_turns::{
            item_context_after_load, item_context_after_weight_calc, natural_attack_enemy,
            result_audio, scientist_turn_with_item_context,
        },
    },
};
fn decode(h: &str) -> Vec<u8> {
    (0..h.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
        .collect()
}
fn bytes(c: &serde_json::Value, k: &str) -> Vec<u8> {
    decode(c[k].as_str().unwrap())
}
fn words(b: &[u8]) -> Vec<i16> {
    b.as_chunks::<2>()
        .0
        .iter()
        .map(|x| i16::from_be_bytes(*x))
        .collect()
}
fn records(b: &[u8]) -> Vec<MonsterRecord> {
    b.as_chunks::<288>()
        .0
        .iter()
        .map(|x| MonsterRecord { bytes: x.to_vec() })
        .collect()
}
fn fixture() -> serde_json::Value {
    serde_json::from_str(include_str!("../../../evidence/fixtures/ally-turns.json")).unwrap()
}
fn content() -> centurions_content::Content {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    centurions_content::Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap()
}
fn mission(f: &serde_json::Value, content: &centurions_content::Content) -> OpeningMission {
    let p = bytes(f, "properties");
    let mut m = content.map(0).unwrap().clone();
    m.tile_properties = p[64..12064].to_vec();
    m.terrain = 2;
    m.default_tile = 155;
    m
}
fn combat(case: &serde_json::Value, distance: i16) -> Combat {
    Combat {
        battle: 8,
        map: TacticalMap {
            tiles: if case["map_before"].is_string() {
                words(&bytes(case, "map_before"))
            } else {
                vec![1155; 8100]
            },
            shift: (0, 0),
            edge_word: 0,
        },
        squad: SquadPlacement {
            positions: (0..6).map(|i| (40 + i, 40)).collect(),
            under_tiles: vec![1155; 6],
        },
        enemies: EnemySetup {
            records: records(&bytes(case, "records_before")),
            positions: vec![(20, 20), (20 + distance, 20)],
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
    }
}
fn record_bytes(c: &Combat) -> Vec<u8> {
    c.enemies
        .records
        .iter()
        .flat_map(|m| m.bytes.clone())
        .collect()
}
fn sounds(events: &[Event]) -> Vec<i16> {
    events
        .iter()
        .filter_map(|e| {
            if let Event::Sound { resource_id } = e {
                Some(*resource_id)
            } else {
                None
            }
        })
        .collect()
}
#[test]
fn scientist_turn_matches_original_records_map_and_rng() {
    let f = fixture();
    let content = content();
    let mission = mission(&f, &content);
    for (index, case) in f["turn_cases"].as_array().unwrap().iter().enumerate() {
        let mut c = combat(case, case["distance"].as_i64().unwrap() as i16);
        let mut w: Vec<_> = bytes(case, "warriors_before")
            .as_chunks::<3138>()
            .0
            .iter()
            .map(|x| Warrior::read(x).unwrap())
            .collect();
        let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        let events = scientist_turn_with_item_context(
            &mut c,
            &mut w,
            &mission,
            &mut random,
            &content.items,
            case["item_context"].as_i64().unwrap_or(0) as i16,
        )
        .unwrap();
        let expected = bytes(case, "records_after");
        let actual = record_bytes(&c);
        let diff: Vec<_> = actual
            .iter()
            .zip(&expected)
            .enumerate()
            .filter(|(_, (a, b))| a != b)
            .map(|(i, (a, b))| (i, a, b))
            .collect();
        assert!(
            diff.is_empty(),
            "case{index} differences{diff:?} draws{} seed{} sounds{:?}",
            random.draws,
            random.seed,
            sounds(&events)
        );
        assert_eq!(
            w.iter().flat_map(|w| w.bytes.clone()).collect::<Vec<_>>(),
            bytes(case, "warriors_after"),
            "party{index}"
        );
        assert_eq!(c.map.tiles, words(&bytes(case, "map_after")), "map{index}");
        let p = words(&bytes(case, "positions"));
        assert_eq!(
            c.enemies.positions,
            vec![(p[0], p[1]), (p[2], p[3])],
            "positions{index}"
        );
        assert_eq!(
            random.draws,
            case["rng_draws"].as_u64().unwrap(),
            "draws{index}"
        );
        assert_eq!(
            random.seed,
            u32::try_from(case["seed_after"].as_u64().unwrap()).unwrap(),
            "seed{index}"
        );
        assert_eq!(
            sounds(&events),
            case["sounds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_i64().unwrap() as i16)
                .collect::<Vec<_>>(),
            "sounds{index}"
        );
    }
}
#[test]
fn monster_natural_attack_matches_original_scientist_damage() {
    let f = fixture();
    for (index, case) in f["attack_cases"].as_array().unwrap().iter().enumerate() {
        let mut c = combat(case, 0);
        c.enemies.positions = vec![(0, 0); 2];
        let mut w: Vec<Warrior> = bytes(case, "warriors_before")
            .as_chunks::<3138>()
            .0
            .iter()
            .map(|x| Warrior::read(x).unwrap())
            .collect();
        let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        let result = natural_attack_enemy(
            &mut c,
            &mut w,
            &mut random,
            10,
            11,
            case["attack"].as_u64().unwrap() as usize,
        )
        .unwrap();
        assert_eq!(
            record_bytes(&c),
            bytes(case, "records_after"),
            "records{index} draws{}",
            random.draws
        );
        assert_eq!(
            i16::from(result.killed),
            case["result"].as_i64().unwrap() as i16,
            "result{index}"
        );
        assert_eq!(
            random.draws,
            case["rng_draws"].as_u64().unwrap(),
            "draws{index}"
        );
        assert_eq!(
            random.seed,
            case["seed_after"].as_u64().unwrap() as u32,
            "seed{index}"
        );
        assert_eq!(c.map.tiles, words(&bytes(case, "map_after")), "map{index}");
        assert_eq!(
            w.iter().flat_map(|w| w.bytes.clone()).collect::<Vec<_>>(),
            bytes(case, "warriors_after"),
            "party{index}"
        );
        let p = words(&bytes(case, "positions"));
        assert_eq!(
            c.enemies.positions,
            vec![(p[0], p[1]), (p[2], p[3])],
            "positions{index}"
        );
        let mut observed_sounds = result.sounds;
        // The direct native probe leaves the result-audio index word at zero.
        observed_sounds.push(result_audio(&c.enemies.records[0], 0, result.damage, 0).unwrap());
        assert_eq!(
            observed_sounds,
            case["sounds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_i64().unwrap() as i16)
                .collect::<Vec<_>>(),
            "sounds{index}"
        );
    }
}

#[test]
fn item_work_context_matches_original_load_and_weight_calculation() {
    let f = fixture();
    let content = content();
    for case in f["context_cases"].as_array().expect("context cases") {
        let result = if case["kind"] == "loaditem" {
            item_context_after_load(&content.items, case["item_id"].as_i64().unwrap() as i16)
                .unwrap()
        } else {
            let party = bytes(case, "warriors_before");
            let member = case["member"].as_u64().unwrap() as usize;
            let w = Warrior::read(&party[member * 3138..(member + 1) * 3138]).unwrap();
            item_context_after_weight_calc(&w, &content.items, 0x5a5a).unwrap()
        };
        assert_eq!(result, case["field32"].as_i64().unwrap() as i16);
        assert_eq!(case["rng_draws"].as_u64().unwrap(), 0);
    }
}
