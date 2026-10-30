use crate::{prelude::*, support};
use centurions_core::{
    combat::click_command,
    feedback::{CombatRefusal, GameFeedback},
};

// Protects ordinary map-click routing and distinct native range, obstruction and ammunition refusals.
#[test]
fn ordinary_clicks_and_refusals_use_the_shared_command_boundary() {
    let content = content();
    let fixture = support::fixture("combat-refusals.json");
    for (case, refusal) in fixture.array("cases").iter().zip([
        CombatRefusal::OutOfRange,
        CombatRefusal::Blocked,
        CombatRefusal::NotEnoughAmmo,
    ]) {
        let mut state = support::mission_session(&content);
        state.phase = Phase::Combat;
        state.warriors = vec![Warrior::read(&case.bytes("before")).unwrap(); 6];
        let mut monster = content.map(0).unwrap().combat.as_ref().unwrap().monsters
            [13 * MONSTER_BYTES..14 * MONSTER_BYTES]
            .to_vec();
        monster[14..16].copy_from_slice(
            &word(&state.warriors[0].bytes, 0x10)
                .unwrap()
                .wrapping_add(1)
                .to_be_bytes(),
        );
        monster[0x110..0x112].copy_from_slice(&100_i16.to_be_bytes());
        monster[0xb0] = 0;
        let target = (case.n::<i16>("target_x"), 20);
        let mut combat = support::battle();
        combat.field.tiles = case.words("map_before");
        combat.squad.pos = vec![(20, 20); 6];
        combat.roster.records = vec![MonsterRecord { bytes: monster }];
        combat.roster.mon_pos = vec![target];
        combat.battle_order[1..3].copy_from_slice(&[0, 10]);
        state.combat = Some(combat);
        crate::native_common::source_victory_context(&mut state);
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
        // The target key's firstt(76, 77) precedes its refusals (combatchoice 11b026).
        let mut expected = vec![Event::Feedback(GameFeedback::Tutorial {
            first: 76,
            last: 77,
        })];
        if refusal == CombatRefusal::NotEnoughAmmo {
            expected.push(Event::Feedback(GameFeedback::Tutorial {
                first: 78,
                last: 78,
            }));
        }
        expected.push(Event::Feedback(GameFeedback::Combat { kind: refusal }));
        assert_eq!(events, expected, "{}", case.text("case"));
        assert_eq!(state.warriors[0].bytes, case.bytes("after"));
        assert_eq!(current_combat(&state).field.tiles, case.words("map_after"));
        assert_eq!(current_combat(&state).battle_order_index, 1);
        assert_eq!(state.random, random);
    }
    // Protects native burst and area shots, ammunition costs and party damage.
    let multishot = support::fixture("multishot.json");
    for case in multishot.array("cases") {
        let mut state = support::mission_session(&content);
        state.phase = Phase::Combat;
        state.warriors = support::warriors(&case.bytes("before"));
        let mut arena = support::battle();
        arena.battle_order[1] = 0;
        arena.squad.pos.fill((20, 20));
        arena.squad.pos[1] = (21, 20);
        arena.squad.pos[2] = (21, 21);
        arena.field.tiles.fill(1155);
        let mut monster = MonsterRecord::read(
            &content.map(0).unwrap().combat.as_ref().unwrap().monsters,
            13,
        )
        .unwrap();
        monster.bytes[0x110..0x112].copy_from_slice(&1000_i16.to_be_bytes());
        monster.bytes[14..16].copy_from_slice(&1_i16.to_be_bytes());
        arena.roster.records.push(monster);
        arena.roster.mon_pos.push((40, 40));
        state.combat = Some(arena);
        crate::native_common::source_victory_context(&mut state);
        let targets = case.n::<Vec<i16>>("targets");
        let mut expected = support::warriors(&case.bytes("after"));
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
            c.field.tiles[25 * 90 + 25] = 0;
            c.field.tiles[26 * 90 + 25] = 1;
            c.field.tiles[26 * 90 + 26] = 2;
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
        // A triple list below WORD4435be waits for the fire key (combatchoice 11ae08).
        if current_combat(&state).firearm_selection.is_some() {
            apply(Command::FireSelectedTargets, &mut state, &content).unwrap();
        }
        assert_eq!(
            current_combat(&state).spell_info,
            case.bytes("packet")
        );
        support::compare_record_bytes(&state.warriors, &expected);
        assert_eq!(state.random.seed, case.n::<u32>("seed"));
        assert_eq!(state.random.draws, case.n::<u64>("draws"));
    }
    // Native confirmed exchanges protect both party and monster terrain ownership.
    let friendly = support::fixture("friendly-attack.json");
    let melee = support::fixture("warrior-melee.json");
    let firearms = support::fixture("friendly-fire.json");
    let refusals = support::fixture("melee-refusals.json");
    for case in friendly
        .array("cases")
        .iter()
        .chain(melee.array("cases").iter())
        .chain(firearms.array("cases").iter())
        .chain(refusals.array("cases").iter())
    {
        let mut state = support::mission_session(&content);
        state.phase = Phase::Combat;
        state.warriors = support::warriors(&case.bytes("before"));
        // Leave another action available so this comparison isolates the captured attack.
        let actor = case.get("actor").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        state.warriors[actor].bytes[0x40..0x42].copy_from_slice(&5_i16.to_be_bytes());
        state.random = RandomState::new(case.n("seed_before"));
        let mut battle = support::battle();
        battle.battle_order[1] = actor as i16;
        battle.squad.pos[0] = (20, 20);
        battle.squad.pos[1] = (21, 20);
        let target = case.get("target").and_then(|v| v.as_i64()).unwrap_or(1) as i16;
        battle.squad.pos[actor] = (20, 20);
        if target < 10 {
            battle.squad.pos[target as usize] = (21, 20);
        }
        if case.get("enemy_before").is_some() {
            battle.roster.records = vec![MonsterRecord {
                bytes: case.bytes("enemy_before"),
            }];
            battle.roster.mon_pos = vec![(21, 20)];
            battle.field.tiles[26 * 90 + 25] = 10;
        }
        state.combat = Some(battle);
        crate::native_common::source_victory_context(&mut state);
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
        expected[actor * WARRIOR_BYTES + 0x40..actor * WARRIOR_BYTES + 0x42]
            .copy_from_slice(&5_i16.to_be_bytes());
        for (offset, cost) in [(0x3c, 3), (0x40, 2)] {
            let offset = actor * WARRIOR_BYTES + offset;
            let value = word(&expected, offset).unwrap().wrapping_sub(cost).max(0);
            expected[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
        }
        if case.get("actor").is_some() {
            let weapon = word(&expected, actor * WARRIOR_BYTES + 0xe4).unwrap();
            let offset = actor * WARRIOR_BYTES
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
            let offset = actor * WARRIOR_BYTES + 0x9d6;
            let weight = word(&expected, offset)
                .unwrap()
                .wrapping_sub(rounds * word(&content.items[weapon as usize], 36).unwrap());
            expected[offset..offset + 2].copy_from_slice(&weight.to_be_bytes());
        }
        assert_eq!(support::record_bytes(&state.warriors), expected);
        support::assert_rng(&state.random, &case["seed"], &case["draws"]);
        if target >= 10 {
            assert_eq!(
                current_combat(&state).roster.records[0].bytes,
                case.bytes("enemy_after")
            );
        }
        assert!(
            events
                .iter()
                .any(|event| matches!(event, Event::AttackResolved { attacker, .. } if *attacker == actor as i16))
        );
    }
    let receipt = support::fixture("combat-exchange.json");
    // Controlled and fleeing warriors commit the original automatic step, without manual input.
    for case in support::fixture("controlled-move.json").array("cases") {
        let mut state = support::mission_session(&content);
        state.phase = Phase::Combat;
        state.random = RandomState::new(12);
        state.warriors = support::warriors(&case.bytes("before"));
        let mut combat = support::battle();
        combat.field.tiles = case.words("map_before");
        combat.squad.pos = (0..6)
            .map(|i| if i == 0 { (20, 20) } else { (40 + i, 40) })
            .collect();
        combat.roster.records = support::monsters(&case.bytes("enemy"));
        combat.roster.mon_pos = vec![(30, 20)];
        combat.battle_order[1] = 0;
        state.combat = Some(combat);
        crate::native_common::source_victory_context(&mut state);
        let before = state.clone();
        assert!(apply(Command::Move { dx: 1, dy: 0 }, &mut state, &content).is_err());
        assert_eq!(state, before);
        apply(Command::RunControlledTurn, &mut state, &content).unwrap();
        assert_eq!(support::record_bytes(&state.warriors), case.bytes("after"));
        assert_eq!(current_combat(&state).field.tiles, case.words("map_after"));
        assert_eq!(
            current_combat(&state).squad.pos[0],
            support::positions(&case.bytes("position"))[0]
        );
        support::assert_rng(&state.random, &case["seed"], &case["draws"]);
    }
    for case in support::fixture("fleeing-edges.json").array("cases") {
        let mut state = crate::command_support_psi::combat_state(&content, case);
        let events = apply(Command::RunControlledTurn, &mut state, &content).unwrap();
        assert_eq!(
            support::record_bytes(&state.warriors),
            case["after"].bytes("party")
        );
        support::assert_rng(
            &state.random,
            &case["after"]["seed"],
            &case["after"]["draws"],
        );
        assert!(matches!(
            events.first(),
            Some(Event::Feedback(
                centurions_core::feedback::GameFeedback::Flash {
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
            let c = current_combat(&state);
            assert_eq!(c.field.tiles, case["after"].words("map"));
            support::native_expect! {
                positions c.squad.pos => &case["after"].bytes("party_positions"), "native positions";
            };
            assert_eq!(c.battle_order, case["after"].words("order"));
            assert_eq!(c.battle_order_index, case["after"].n::<usize>("cursor"));
        }
        let path =
            std::env::temp_dir().join(format!("centurions-edge-{}.json", std::process::id()));
        save(&path, &state, &content).unwrap();
        assert_eq!(load(&path, &content).unwrap(), state);
        std::fs::remove_file(path).unwrap();
    }
    let edges = support::fixture("fleeing-edges.json");
    for kind in 0..3 {
        let mut state = crate::command_support_psi::combat_state(
            &content,
            edges.array("cases").last().unwrap(),
        );
        let position = state.exploration.as_mut().unwrap();
        position.x = 10;
        position.y = 10;
        position.last_move = [1, 0];
        position.script = Some(centurions_core::ScriptState::scheduled(
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
        let mut state = support::mission_session(&content);
        state.phase = Phase::Combat;
        state.warriors = support::warriors(&case.bytes("party_before"));
        let positions = |key: &str| support::positions(&case[key]);
        let mut combat = support::battle();
        combat.field.tiles = case.words("tiles_before");
        combat.squad.pos = positions("positions_before");
        combat.squad.char_under = case.words("terrain_before");
        combat.roster.records = vec![MonsterRecord {
            bytes: case.bytes("monster_before"),
        }];
        combat.roster.mon_pos = vec![(21, 20)];
        combat.battle_order[1] = 0;
        state.combat = Some(combat);
        crate::native_common::source_victory_context(&mut state);
        let before = state.random.clone();
        let target = case.n::<i16>("target");
        let events = apply(
            Command::SwapCombatants { body: target },
            &mut state,
            &content,
        )
        .unwrap();
        assert_eq!(
            support::record_bytes(&state.warriors),
            case.bytes("party_after")
        );
        let combat = current_combat(&state);
        assert_eq!(combat.field.tiles, case.words("tiles_after"));
        assert_eq!(combat.squad.pos, positions("positions_after"));
        assert_eq!(combat.squad.char_under, case.words("terrain_after"));
        assert_eq!(combat.roster.records[0].bytes, case.bytes("monster_after"));
        assert_eq!(
            combat.roster.mon_pos,
            positions("monster_position_after")
        );
        assert_eq!(combat.battle_order_index, 1);
        assert_eq!(state.random, before);
        let sounds = support::event_sounds(&events);
        support::expect_array(&sounds, &case["sounds"]);
        assert!(
            matches!(events.as_slice(), [Event::Sound { .. }, Event::TacticalMoved { body: 0, .. }, Event::TacticalMoved { body, .. }] if *body == target)
        );
    }
}
