use crate::{prelude::*, support};

use support::positions;
fn origin(c: &serde_json::Value) -> (i16, i16) {
    let b = c.bytes("origin");
    (
        i32::from_be_bytes(b[..4].try_into().unwrap()) as i16,
        i32::from_be_bytes(b[4..].try_into().unwrap()) as i16,
    )
}

#[test]
fn composed_shot_matches_live_original_records_rng_and_next_turn() {
    let f: serde_json::Value = support::fixture("live-firearm.json");

    let content = support::content();
    let before = &f["before"];
    let after = &f["after"];
    let c = &before["combat"];
    let hex = |key: &str| c[key].as_str().unwrap();
    let o = origin(c);
    let mut state = support::mission_session(&content);
    state.phase = Phase::Combat;
    state.warriors = before
        .array("warriors")
        .iter()
        .map(|v| Warrior::read(&decode(v.as_str().unwrap())).unwrap())
        .collect();
    state.random = RandomState::new(before.n::<u32>("random_seed"));
    let mut combat = support::battle();
    combat.battle = 2;
    combat.map.tiles = words(hex("map"));
    combat.map.shift = (o.0 - 5, o.1 - 5);
    combat.squad.positions = positions(hex("squad_positions"));
    combat.squad.under_tiles = words(hex("under_tiles"));
    combat.enemies.records = support::monsters(&decode(hex("enemies")));
    combat.enemies.positions = positions(hex("enemy_positions"));
    combat.order = words(hex("order"));
    combat.cursor = words(hex("cursor_enemy_index"))[0] as usize;
    state.combat = Some(combat);
    crate::native_common::source_victory_context(&mut state);
    let events = apply(Command::ShootEnemy { body: 12 }, &mut state, &content).unwrap();
    assert!(events.iter().any(|e| matches!(
        e,
        centurions_core::Event::AttackResolved {
            attacker: 3,
            target: 12,
            damage: 2,
            defeated: false,
            ..
        }
    )));
    support::expect(state.random.seed, &after["random_seed"]);
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
        words(expected.text("cursor_enemy_index"))[0] as usize
    );
    support::native_expect! {
        words &(c.order) => &expected["order"], "live_firearm";
        words &(c.map.tiles) => &expected["map"], "live_firearm";
        records &c.enemies.records => &expected["enemies"], "live_firearm";
        words &(c.squad.under_tiles) => &expected["under_tiles"], "live_firearm";
    }
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
    let f: serde_json::Value = support::fixture("live-firearm.json");
    let v = &f["victory"];
    let mut warriors: Vec<_> = v
        .array("warriors")
        .iter()
        .map(|w| Warrior::read(&decode(w.as_str().unwrap())).unwrap())
        .collect();
    let mut enemies: Vec<_> = support::monsters(&v["enemies"]);
    let share =
        runtime::victory_records(&mut warriors, &mut enemies, &mut RandomState::new(12)).unwrap();
    support::expect(share, &v["share"]);
    for (i, w) in warriors.iter().enumerate() {
        let expected = decode(v["after_warriors"][i].as_str().unwrap());
        assert_eq!(
            w.bytes[0x9ce..0x9d6],
            expected[0x9ce..0x9d6],
            "warrior {i} point fields"
        );
    }
}
