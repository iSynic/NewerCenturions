use crate::prelude::*;
use centurions_core::{
    combat::click_command,
    feedback::{CombatRefusal, NativeFeedback},
};

// Protects ordinary map-click routing and distinct native range, obstruction and ammunition refusals.
#[test]
fn ordinary_clicks_and_refusals_use_the_shared_command_boundary() {
    let content = content();
    let fixture = crate::support::fixture("combat-refusals.json");
    for (case, refusal) in fixture.array("cases").iter().zip([
        CombatRefusal::OutOfRange,
        CombatRefusal::Blocked,
        CombatRefusal::NotEnoughAmmo,
    ]) {
        let mut state = crate::support::mission_session(&content);
        state.phase = Phase::Combat;
        state.warriors = vec![Warrior::read(&case.bytes("before")).unwrap(); 6];
        let mut enemy =
            content.map(0).unwrap().combat.as_ref().unwrap().monsters[13 * 288..14 * 288].to_vec();
        enemy[14..16].copy_from_slice(
            &word(&state.warriors[0].bytes, 0x10)
                .unwrap()
                .wrapping_add(1)
                .to_be_bytes(),
        );
        enemy[0x110..0x112].copy_from_slice(&100_i16.to_be_bytes());
        enemy[0xb0] = 0;
        let target = (case.n::<i16>("target_x"), 20);
        state.combat = Some(Combat {
            map: TacticalMap {
                tiles: case.words("map_before"),
                shift: (0, 0),
                edge_word: 0,
            },
            squad: SquadPlacement {
                positions: vec![(20, 20); 6],
                under_tiles: vec![1155; 6],
            },
            enemies: EnemySetup {
                records: vec![MonsterRecord { bytes: enemy }],
                positions: vec![target],
            },
            ..crate::support::battle()
        });
        state.combat.as_mut().unwrap().order[1..3].copy_from_slice(&[0, 10]);
        let random = state.random.clone();
        assert!(click_command(&state, 20, 20).unwrap().is_none());
        assert!(matches!(
            click_command(&state, 19, 20).unwrap(),
            Some(Command::Move { dx: -1, dy: 0 })
        ));
        state.warriors[0].bytes[0x9dd] = 0;
        assert!(matches!(
            click_command(&state, target.0, target.1).unwrap(),
            Some(Command::Move { dx: 1, dy: 0 })
        ));
        state.warriors[0].bytes[0x9dd] = 1;
        let command = Command::CombatClick {
            x: target.0,
            y: target.1,
        };
        let replay = serde_json::from_str(&serde_json::to_string(&command).unwrap()).unwrap();
        let events = apply(replay, &mut state, &content).unwrap();
        let mut expected = vec![];
        if refusal == CombatRefusal::NotEnoughAmmo {
            expected.push(Event::Feedback(NativeFeedback::Tutorial {
                first: 78,
                last: 78,
            }));
        }
        expected.push(Event::Feedback(NativeFeedback::Combat { kind: refusal }));
        assert_eq!(
            serde_json::to_value(&events).unwrap(),
            serde_json::to_value(expected).unwrap(),
            "{}",
            case.text("case")
        );
        assert_eq!(state.warriors[0].bytes, case.bytes("after"));
        assert_eq!(
            state.combat.as_ref().unwrap().map.tiles,
            case.words("map_after")
        );
        assert_eq!(state.combat.as_ref().unwrap().cursor, 1);
        assert_eq!(state.random, random);
    }
    // Protects native burst and area shots, ammunition costs and party damage.
    let multishot = crate::support::fixture("multishot.json");
    for case in multishot.array("cases") {
        let mut state = crate::support::mission_session(&content);
        state.phase = Phase::Combat;
        state.warriors = crate::support::warriors(&case.bytes("before"));
        let mut arena = crate::support::battle();
        arena.order[1] = 0;
        arena.squad.positions.fill((20, 20));
        arena.squad.positions[1] = (21, 20);
        arena.squad.positions[2] = (21, 21);
        arena.map.tiles.fill(1155);
        let mut enemy = MonsterRecord::read(
            &content.map(0).unwrap().combat.as_ref().unwrap().monsters,
            13,
        )
        .unwrap();
        enemy.bytes[0x110..0x112].copy_from_slice(&1000_i16.to_be_bytes());
        enemy.bytes[14..16].copy_from_slice(&1_i16.to_be_bytes());
        arena.enemies.records.push(enemy);
        arena.enemies.positions.push((40, 40));
        state.combat = Some(arena);
        let targets = case.n::<Vec<i16>>("targets");
        let mut expected = crate::support::warriors(&case.bytes("after"));
        let weapon = &content.items[case.n::<usize>("weapon")];
        for (offset, cost) in [
            (0x40, 2),
            (0x3c, 3),
            (0x134, word(weapon, 22).unwrap() * targets.len() as i16),
            (
                0x9d6,
                word(weapon, 22).unwrap() * word(weapon, 36).unwrap() * targets.len() as i16,
            ),
        ] {
            let value = word(&expected[0].bytes, offset).unwrap().wrapping_sub(cost);
            let value = if offset == 0x3c { value.max(0) } else { value };
            expected[0].bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
        }
        state.random = RandomState::new(case.n("seed_before"));
        if let Some(location) = case.get("location").and_then(|value| value.as_array()) {
            let c = state.combat.as_mut().unwrap();
            c.map.tiles[25 * 90 + 25] = 0;
            c.map.tiles[26 * 90 + 25] = 1;
            c.map.tiles[26 * 90 + 26] = 2;
            apply(
                Command::ShootAt {
                    x: location[0].as_i64().unwrap() as i16,
                    y: location[1].as_i64().unwrap() as i16,
                },
                &mut state,
                &content,
            )
            .unwrap();
        } else {
            for &body in &targets {
                apply(Command::ShootEnemy { body }, &mut state, &content).unwrap();
            }
        }
        assert_eq!(
            state
                .combat
                .as_ref()
                .unwrap()
                .attack_packet
                .as_ref()
                .unwrap(),
            &case.bytes("packet")
        );
        for (actual, expected) in state.warriors.iter().zip(&expected) {
            assert_eq!(
                actual.bytes,
                expected.bytes,
                "weapon {}",
                case.n::<i16>("weapon")
            );
        }
        assert_eq!(state.random.seed, case.n::<u32>("seed"));
        assert_eq!(state.random.draws, case.n::<u64>("draws"));
    }
    // Native confirmed exchanges protect both party and monster terrain ownership.
    let friendly = crate::support::fixture("friendly-attack.json");
    let melee = crate::support::fixture("warrior-melee.json");
    let firearms = crate::support::fixture("friendly-fire.json");
    let refusals = crate::support::fixture("melee-refusals.json");
    for case in friendly
        .array("cases")
        .iter()
        .chain(melee.array("cases").iter())
        .chain(firearms.array("cases").iter())
        .chain(refusals.array("cases").iter())
    {
        let mut state = crate::support::mission_session(&content);
        state.phase = Phase::Combat;
        state.warriors = crate::support::warriors(&case.bytes("before"));
        // Leave another action available so this comparison isolates the captured attack.
        let actor = case.get("actor").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        state.warriors[actor].bytes[0x40..0x42].copy_from_slice(&5_i16.to_be_bytes());
        state.random = RandomState::new(case.n("seed_before"));
        let mut battle = crate::support::battle();
        battle.order[1] = actor as i16;
        battle.squad.positions[0] = (20, 20);
        battle.squad.positions[1] = (21, 20);
        let target = case.get("target").and_then(|v| v.as_i64()).unwrap_or(1) as i16;
        battle.squad.positions[actor] = (20, 20);
        if target < 10 {
            battle.squad.positions[target as usize] = (21, 20);
        }
        if case.get("enemy_before").is_some() {
            battle.enemies.records = vec![MonsterRecord {
                bytes: case.bytes("enemy_before"),
            }];
            battle.enemies.positions = vec![(21, 20)];
            battle.map.tiles[26 * 90 + 25] = 10;
        }
        state.combat = Some(battle);
        let events = apply(
            if case.get("actor").is_some() {
                Command::ShootEnemy { body: target }
            } else if target < 10 {
                Command::AttackFriend { body: target }
            } else {
                Command::AttackEnemy { body: target }
            },
            &mut state,
            &content,
        )
        .unwrap();
        let mut expected = case.bytes("after");
        expected[actor * 3138 + 0x40..actor * 3138 + 0x42].copy_from_slice(&5_i16.to_be_bytes());
        for (offset, cost) in [(0x3c, 3), (0x40, 2)] {
            let offset = actor * 3138 + offset;
            let value = word(&expected, offset).unwrap().wrapping_sub(cost).max(0);
            expected[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
        }
        if case.get("actor").is_some() {
            let weapon = word(&expected, actor * 3138 + 0xe4).unwrap();
            let offset = actor * 3138
                + 0x132
                + state.warriors[actor]
                    .inventory()
                    .unwrap()
                    .iter()
                    .position(|i| i.id() == weapon && i.0[9] == 1)
                    .unwrap()
                    * 22
                + 2;
            let rounds = word(&content.items[weapon as usize], 22).unwrap();
            let left = word(&expected, offset).unwrap().wrapping_sub(rounds);
            expected[offset..offset + 2].copy_from_slice(&left.to_be_bytes());
            let offset = actor * 3138 + 0x9d6;
            let weight = word(&expected, offset)
                .unwrap()
                .wrapping_sub(rounds * word(&content.items[weapon as usize], 36).unwrap());
            expected[offset..offset + 2].copy_from_slice(&weight.to_be_bytes());
        }
        assert_eq!(
            state
                .warriors
                .iter()
                .flat_map(|w| w.bytes.clone())
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(
            (state.random.seed, state.random.draws),
            (case.n::<u32>("seed"), case.n::<u64>("draws"))
        );
        if target >= 10 {
            assert_eq!(
                state.combat.as_ref().unwrap().enemies.records[0].bytes,
                case.bytes("enemy_after")
            );
        }
        assert!(
            events
                .iter()
                .any(|event| matches!(event, Event::AttackResolved { attacker, .. } if *attacker == actor as i16))
        );
    }
    let receipt = crate::support::fixture("combat-exchange.json");
    // Controlled and fleeing warriors commit the original automatic step, without manual input.
    for case in crate::support::fixture("controlled-move.json").array("cases") {
        let mut state = crate::support::mission_session(&content);
        state.phase = Phase::Combat;
        state.random = RandomState::new(12);
        state.warriors = crate::support::warriors(&case.bytes("before"));
        let mut combat = crate::support::battle();
        combat.map.tiles = case.words("map_before");
        combat.squad.positions = (0..6)
            .map(|i| if i == 0 { (20, 20) } else { (40 + i, 40) })
            .collect();
        combat.enemies.records = crate::support::monsters(&case.bytes("enemy"));
        combat.enemies.positions = vec![(30, 20)];
        combat.order[1] = 0;
        state.combat = Some(combat);
        let before = state.clone();
        assert!(apply(Command::Move { dx: 1, dy: 0 }, &mut state, &content).is_err());
        assert_eq!(state, before);
        apply(Command::RunControlledTurn, &mut state, &content).unwrap();
        assert_eq!(
            state
                .warriors
                .iter()
                .flat_map(|w| w.bytes.clone())
                .collect::<Vec<_>>(),
            case.bytes("after")
        );
        assert_eq!(
            state.combat.as_ref().unwrap().map.tiles,
            case.words("map_after")
        );
        assert_eq!(
            state.combat.as_ref().unwrap().squad.positions[0],
            crate::support::positions(&case.bytes("position"))[0]
        );
        assert_eq!(
            (state.random.seed, state.random.draws),
            (case.n::<u32>("seed"), case.n::<u64>("draws"))
        );
    }
    for case in crate::support::fixture("fleeing-edges.json").array("cases") {
        let mut state = crate::command_support_psi::combat_state(&content, case);
        let events = apply(Command::RunControlledTurn, &mut state, &content).unwrap();
        assert_eq!(
            state
                .warriors
                .iter()
                .flat_map(|w| w.bytes.clone())
                .collect::<Vec<_>>(),
            case["after"].bytes("party")
        );
        assert_eq!(
            (state.random.seed, state.random.draws),
            (
                case["after"].n::<u32>("seed"),
                case["after"].n::<u64>("draws")
            )
        );
        assert!(matches!(
            events.first(),
            Some(Event::Feedback(
                centurions_core::feedback::NativeFeedback::Flash {
                    address: 0x403c3a,
                    sound: 6000,
                    ..
                }
            ))
        ));
        if case["all_fled"] == true {
            assert!(state.combat.is_none());
            assert_eq!(state.phase, Phase::Exploration);
            assert!(events.contains(&Event::BattleEnded { victory: false }));
        } else {
            let c = state.combat.as_ref().unwrap();
            assert_eq!(c.map.tiles, case["after"].words("map"));
            assert_eq!(
                c.squad.positions,
                crate::support::positions(&case["after"].bytes("party_positions"))
            );
            assert_eq!(c.order, case["after"].words("order"));
            assert_eq!(c.cursor, case["after"].n::<usize>("cursor"));
        }
        let path =
            std::env::temp_dir().join(format!("centurions-edge-{}.json", std::process::id()));
        save(&path, &state, &content).unwrap();
        assert_eq!(load(&path, &content).unwrap(), state);
        std::fs::remove_file(path).unwrap();
    }
    let edges = crate::support::fixture("fleeing-edges.json");
    for kind in 0..3 {
        let mut state = crate::command_support_psi::combat_state(
            &content,
            edges.array("cases").last().unwrap(),
        );
        let position = state.exploration.as_mut().unwrap();
        position.x = 10;
        position.y = 10;
        position.last_move = [1, 0];
        position.script = Some(crate::support::script_state(
            content.map(0).unwrap().script(0).unwrap().to_vec(),
            0,
        ));
        if kind == 0 {
            state.battle_group_script = Some(0);
        } else {
            let mut extra = [0; 10];
            if kind == 2 {
                extra[4..6].copy_from_slice(&(-1_i16).to_be_bytes());
            }
            state.battle_npc = Some(
                centurions_core::story_npc_battle::NpcBattleStart::capture(&extra, &[0; 42], 0)
                    .unwrap(),
            );
        }
        let rng = state.random.clone();
        apply(Command::RunControlledTurn, &mut state, &content).unwrap();
        assert!(state.combat.is_none());
        assert_eq!(state.phase, Phase::Exploration);
        assert_eq!(state.random, rng);
        assert!(state.battle_group_script.is_none() && state.battle_npc.is_none());
        assert_eq!(
            state.exploration.as_ref().unwrap().x,
            if kind == 2 { 9 } else { 10 }
        );
    }
    for case in receipt.array("cases") {
        let mut state = crate::support::mission_session(&content);
        state.phase = Phase::Combat;
        state.warriors = crate::support::warriors(&case.bytes("party_before"));
        let positions = |key: &str| {
            case.words(key)
                .as_chunks::<2>()
                .0
                .iter()
                .map(|xy| (xy[0], xy[1]))
                .collect::<Vec<_>>()
        };
        let mut combat = crate::support::battle();
        combat.map.tiles = case.words("tiles_before");
        combat.squad.positions = positions("positions_before");
        combat.squad.under_tiles = case.words("terrain_before");
        combat.enemies.records = vec![MonsterRecord {
            bytes: case.bytes("monster_before"),
        }];
        combat.enemies.positions = vec![(21, 20)];
        combat.order[1] = 0;
        state.combat = Some(combat);
        let before = state.random.clone();
        let target = case.n::<i16>("target");
        let events = apply(
            Command::SwapCombatants { body: target },
            &mut state,
            &content,
        )
        .unwrap();
        assert_eq!(
            state
                .warriors
                .iter()
                .flat_map(|warrior| warrior.bytes.iter().copied())
                .collect::<Vec<_>>(),
            case.bytes("party_after")
        );
        let combat = state.combat.as_ref().unwrap();
        assert_eq!(combat.map.tiles, case.words("tiles_after"));
        assert_eq!(combat.squad.positions, positions("positions_after"));
        assert_eq!(combat.squad.under_tiles, case.words("terrain_after"));
        assert_eq!(combat.enemies.records[0].bytes, case.bytes("monster_after"));
        assert_eq!(
            combat.enemies.positions,
            positions("monster_position_after")
        );
        assert_eq!(combat.cursor, 1);
        assert_eq!(state.random, before);
        let sounds = crate::support::event_sounds(&events);
        crate::support::expect_array(&sounds, &case["sounds"]);
        assert!(
            matches!(events.as_slice(), [Event::Sound { .. }, Event::TacticalMoved { body: 0, .. }, Event::TacticalMoved { body, .. }] if *body == target)
        );
    }
}
