use centurions_content::{Content, Warrior, combat::MonsterRecord};
use centurions_core::{Command, Phase, RandomState, Session, apply, combat::*};

fn decode(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}
fn words(hex: &str) -> Vec<i16> {
    decode(hex)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|v| i16::from_be_bytes(*v))
        .collect()
}
fn positions(hex: &str) -> Vec<(i16, i16)> {
    words(hex)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|v| (v[0], v[1]))
        .collect()
}
fn origin(c: &serde_json::Value) -> (i16, i16) {
    let b = decode(c["origin"].as_str().unwrap());
    (
        i32::from_be_bytes(b[..4].try_into().unwrap()) as i16,
        i32::from_be_bytes(b[4..].try_into().unwrap()) as i16,
    )
}

#[test]
fn composed_shot_matches_live_original_records_rng_and_next_turn() {
    let f: serde_json::Value =
        serde_json::from_str(include_str!("../../../evidence/fixtures/live-firearm.json")).unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let content = Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap();
    let before = &f["before"];
    let after = &f["after"];
    let c = &before["combat"];
    let hex = |key: &str| c[key].as_str().unwrap();
    let o = origin(c);
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
    state.phase = Phase::Combat;
    state.warriors = before["warriors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| Warrior::read(&decode(v.as_str().unwrap())).unwrap())
        .collect();
    state.random = RandomState::new(before["random_seed"].as_u64().unwrap() as u32);
    state.combat = Some(Combat {
        battle: 2,
        map: TacticalMap {
            tiles: words(hex("map")),
            shift: (o.0 - 5, o.1 - 5),
            edge_word: 0,
        },
        squad: SquadPlacement {
            positions: positions(hex("squad_positions")),
            under_tiles: words(hex("under_tiles")),
        },
        enemies: EnemySetup {
            records: decode(hex("enemies"))
                .as_chunks::<288>()
                .0
                .iter()
                .map(|m| MonsterRecord { bytes: m.to_vec() })
                .collect(),
            positions: positions(hex("enemy_positions")),
        },
        order: words(hex("order")),
        cursor: words(hex("cursor_enemy_index"))[0] as usize,
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
    let events = apply(Command::ShootEnemy { body: 12 }, &mut state, &content).unwrap();
    assert!(events.iter().any(|e| matches!(
        e,
        centurions_core::Event::AttackResolved {
            attacker: 3,
            target: 12,
            damage: 2,
            defeated: false
        }
    )));
    assert_eq!(
        state.random.seed,
        after["random_seed"].as_u64().unwrap() as u32
    );
    for (i, warrior) in state.warriors.iter().enumerate() {
        assert_eq!(
            warrior.bytes,
            decode(after["warriors"][i].as_str().unwrap()),
            "warrior {i}"
        );
    }
    let expected = &after["combat"];
    let c = state.combat.unwrap();
    assert_eq!(
        c.cursor,
        words(expected["cursor_enemy_index"].as_str().unwrap())[0] as usize
    );
    assert_eq!(c.order, words(expected["order"].as_str().unwrap()));
    assert_eq!(c.map.tiles, words(expected["map"].as_str().unwrap()));
    assert_eq!(
        c.enemies
            .records
            .iter()
            .flat_map(|m| &m.bytes)
            .copied()
            .collect::<Vec<_>>(),
        decode(expected["enemies"].as_str().unwrap())
    );
    assert_eq!(
        c.squad.under_tiles,
        words(expected["under_tiles"].as_str().unwrap())
    );
    let after_origin = origin(expected);
    for (actual, key) in [&c.squad.positions, &c.enemies.positions]
        .into_iter()
        .zip(["squad_positions", "enemy_positions"])
    {
        let absolute: Vec<_> = actual.iter().map(|p| (p.0 + o.0, p.1 + o.1)).collect();
        let expected: Vec<_> = positions(expected[key].as_str().unwrap())
            .iter()
            .map(|p| (p.0 + after_origin.0, p.1 + after_origin.1))
            .collect();
        assert_eq!(absolute, expected);
    }
}

#[test]
fn victory_point_fields_match_live_original_five_wasp_rewards() {
    let f: serde_json::Value =
        serde_json::from_str(include_str!("../../../evidence/fixtures/live-firearm.json")).unwrap();
    let v = &f["victory"];
    let mut warriors: Vec<_> = v["warriors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| Warrior::read(&decode(w.as_str().unwrap())).unwrap())
        .collect();
    let mut enemies: Vec<_> = decode(v["enemies"].as_str().unwrap())
        .as_chunks::<288>()
        .0
        .iter()
        .map(|m| MonsterRecord { bytes: m.to_vec() })
        .collect();
    let share =
        runtime::victory_records(&mut warriors, &mut enemies, &mut RandomState::new(12)).unwrap();
    assert_eq!(share, v["share"].as_i64().unwrap() as i32);
    for (i, w) in warriors.iter().enumerate() {
        let expected = decode(v["after_warriors"][i].as_str().unwrap());
        assert_eq!(
            w.bytes[0x9ce..0x9d6],
            expected[0x9ce..0x9d6],
            "warrior {i} point fields"
        );
    }
}
