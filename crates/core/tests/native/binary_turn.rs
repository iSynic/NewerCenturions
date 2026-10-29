use crate::{prelude::*, support};

#[test]
fn first_turn_matches_whole_records_and_random_draws() {
    let fixture: serde_json::Value = support::fixture("turn.json");
    for case in fixture.array("cases") {
        let mut warriors: Vec<_> = support::warriors(&case["warriors"]);
        let mut enemies = vec![MonsterRecord {
            bytes: case.bytes("enemy"),
        }];
        let mut random = RandomState::new(1);
        initialize_turn(
            case.n::<i16>("body"),
            &mut warriors,
            &mut enemies,
            &[12, 14, 12, 14, 10, 12, 10, 10, 14, 16, 0, 12],
            &mut random,
        )
        .unwrap();
        support::native_expect! {
            records &warriors => &case["after_warriors"], format!("body {}",case["body"]);
            bytes enemies[0].bytes => &case["after_enemy"], "binary_turn";
            rng random => (&case["seed"], &case["draws"]), "binary_turn";
        }
    }
    let content = content();
    for case in support::fixture("turn-caller-contexts.json").array("cases") {
        let mut state = support::mission_session(&content);
        state.warriors = support::warriors(&case["warriors"]);
        state.random = RandomState::new(case.n("seed_before"));
        let position = state.exploration.as_mut().unwrap();
        position.presentation = None;
        position.script = None;
        let mut c = support::battle();
        // VerifyTurnContexts writes total6/defeated0 before the captured entry.
        c.victory_work = centurions_core::combat::victory_work::VictoryWork {
            total: 6,
            defeated: 0,
        };
        c.enemies.records = vec![MonsterRecord {
            bytes: case.bytes("enemy"),
        }];
        c.enemies.positions = vec![(20, 20)];
        c.order[1..4].copy_from_slice(&[1, 0, 2]);
        state.combat = Some(c);
        state.phase = Phase::Combat;
        let events = support::saved_command(Command::EndTurn, &mut state, &content);
        support::native_expect! {
            records &state.warriors => &case["after_warriors"], "conditioned Getup party";
            records &current_combat(&state).enemies.records => &case["after_enemy"], "conditioned Getup NPC";
            rng state.random => (&case["seed"], &case["draws"]), "conditioned Getup RNG";
            scalar current_combat(&state).cursor => &case["cursor"], "conditioned Getup skip";
            scalar current_combat(&state).unavailable => &case["unavailable"], "conditioned Getup counter";
            scalar state.selected_warrior => &case["selected_warrior"], "conditioned Getup selection";
            scalar support::event_sounds(&events) => &case["sounds"], "conditioned Getup sounds";
        }
    }
}

#[test]
fn map_edge_turn_exit_matches_original_caller_and_cursor() {
    let content = content();
    let fixture = support::fixture("enemy-turn-exit.json");
    for (index, case) in fixture.array("cases").iter().enumerate() {
        let mut state: Session = serde_json::from_value(
            fixture[if index == 0 {
                "command_input_session"
            } else {
                "input_session"
            }]
            .clone(),
        )
        .unwrap();
        // VerifyEnemyTurnExit's input control writes total7 and the captured
        // defeated prefix. These are harness writes, not a live roster recount.
        state.combat.as_mut().unwrap().victory_work =
            centurions_core::combat::victory_work::VictoryWork {
                total: 7,
                defeated: case.n("removed_before"),
            };
        // VerifyEnemyTurnExit's loaded A5 image has zero438780/43877e.
        crate::native_common::npc_command_input(&mut state, fixture.n("enemy_index"), [0, 0]);
        let queue = current_combat(&state).death_queue.clone();
        if index != 0 {
            state.combat.as_mut().unwrap().enemies.records[fixture.n::<usize>("enemy_index")]
                .bytes = case.bytes("enemy_before");
        }
        let events = if index == 0 {
            support::saved_command(Command::RunEnemyTurn, &mut state, &content)
        } else {
            // The guest controls start inside the caller, after public EndTurn's
            // movement/guard reset. Enter that same runtime boundary directly.
            runtime::end_turn(&mut state, &content).unwrap()
        };
        // The native capture begins after the command's movement prefix.
        let start = events
            .iter()
            .position(|event| {
                matches!(
                    event,
                    Event::Feedback(centurions_core::feedback::NativeFeedback::CombatResult {
                        code: -5,
                        ..
                    })
                )
            })
            .unwrap();
        let events = &events[start..];
        let combat = current_combat(&state);
        assert_eq!(combat.victory_work.defeated, case.n::<i16>("removed_after"));
        assert_eq!(combat.victory_work.total, 7);
        support::native_expect! {
            records &state.warriors => &case["warriors_after"], "Getup exit party";
            records &combat.enemies.records => &case["records_after"], "Getup exit enemies";
            bytes support::word_bytes(&combat.map.tiles) => &case["map_after"], "Getup exit footprint";
            bytes support::word_bytes(&combat.order) => &case["order_after"], "Getup exit order";
            scalar combat.cursor => &case["cursor"], "Getup exit cursor";
            rng state.random => (&case["seed"], &case["total_draws_after"]), "Getup exit RNG";
            scalar support::event_sounds(events) => &case["sounds"], "Getup exit sounds";
        }
        let results: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                Event::Feedback(centurions_core::feedback::NativeFeedback::CombatResult {
                    body,
                    code,
                    ..
                }) => Some(json!({"body": body,"code": code})),
                _ => None,
            })
            .collect();
        assert_eq!(json!(results), case["results"]);
        assert_eq!(combat.death_queue, queue);
    }
}
