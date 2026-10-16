use crate::prelude::*;

use crate::support::field_bytes as bytes;
use centurions_core::combat::round_stories;
use centurions_core::combat::round_stories::RoundEffect;

#[test]
fn summoned_difficulty_matches_original_record_widths_rounding_and_rng() {
    let fixture = crate::support::fixture("difficulty-summons.json");
    for case in fixture.array("cases") {
        let before = MonsterRecord {
            bytes: case.bytes("before"),
        };
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let after = centurions_core::combat::monster::summon(
            before,
            case.n::<i16>("faction"),
            case.n::<i16>("difficulty"),
            &mut random,
        )
        .unwrap();
        assert_eq!(after.bytes, case.bytes("after"), "{}", case["case"]);
        assert_eq!(
            (random.seed, random.draws),
            (case.n::<u32>("seed_after"), case.n::<u64>("draws"))
        );
    }
}

use crate::native_common::script_mission as mission;
fn input(
    fixture: &serde_json::Value,
    case: &serde_json::Value,
) -> (Combat, Vec<Warrior>, RandomState) {
    let mut order = vec![-1; 110];
    order[1] = 10;
    if case.get("order_before").is_some() {
        order = words(&bytes(case, "order_before"));
    }
    (
        Combat {
            battle: 146,
            map: TacticalMap {
                tiles: words(&bytes(case, "map_before")),
                edge_word: 0,
                shift: (0, 0),
            },
            squad: SquadPlacement {
                positions: (0..6).map(|i| (40 + i, 40)).collect(),
                under_tiles: vec![1155; 6],
            },
            enemies: EnemySetup {
                records: crate::support::monsters(&bytes(case, "records_before")),
                positions: crate::support::positions(&case["positions_before"]),
            },
            order,
            script_enemy: case["actor_before"].as_u64().unwrap_or(0) as usize,
            ..crate::support::battle()
        },
        crate::support::warriors(&bytes(fixture, "warriors_before")),
        RandomState::new(case.n::<u32>("seed_before")),
    )
}

#[test]
fn original_round_callers_match_records_map_order_slots_party_and_rng() {
    for fixture_name in [
        "round-stories.json",
        "additional-rounds.json",
        "removal-rounds.json",
        "retained-round-origin.json",
    ] {
        let fixture = crate::support::fixture(fixture_name);
        let mission = mission(&fixture);
        let definitions = bytes(&fixture, "monsters");
        let extras = bytes(&fixture, "extras");
        for (index, case) in fixture.array("cases").iter().enumerate() {
            let (mut combat, warriors, mut random) = input(&fixture, case);
            let initial_hostiles = case["hostile_count_before"].as_i64().unwrap_or_else(|| {
                combat
                    .enemies
                    .records
                    .iter()
                    .filter(|m| {
                        m.stamina().unwrap() > 0
                            && centurions_content::word(&m.bytes, 14).unwrap() != 0
                    })
                    .count() as i64
            });
            let slot_words = words(&bytes(case, "slots_before"));
            let mut state = RoundScripts {
                counter: case.n::<i8>("round_before"),
                slots: slot_words.try_into().unwrap(),
                pending: None,
            };
            let mut effects = Vec::new();
            let mut hostile_additions = 0i16;
            for _ in 0..case.n::<u64>("calls") {
                loop {
                    if let Some(program) = state.pending.as_mut() {
                        program.text = None;
                    }
                    let caller = round_stories::RoundCaller {
                        active_enemy: combat.script_enemy,
                        last_defeated: case["last_defeated"].as_i64().map(|body| body as i16),
                    };
                    let result = round_stories::advance(
                        &mut combat,
                        &mut state,
                        &mission,
                        &definitions,
                        &extras,
                        caller,
                        &mut random,
                    )
                    .unwrap_or_else(|error| panic!("case{index}: {error}"));
                    hostile_additions = hostile_additions.wrapping_add(result.hostile_additions);
                    effects.extend(result.effects);
                    if state.pending.is_none() {
                        break;
                    }
                }
            }
            let records: Vec<u8> = crate::support::record_bytes(&combat.enemies.records);
            assert_eq!(records, bytes(case, "records_after"), "records case{index}");
            assert_eq!(
                combat.map.tiles,
                words(&bytes(case, "map_after")),
                "map case{index}"
            );
            assert_eq!(
                combat.order,
                words(&bytes(case, "order_after")),
                "order case{index}"
            );
            let positions: Vec<i16> = combat
                .enemies
                .positions
                .iter()
                .flat_map(|&(x, y)| [x, y])
                .collect();
            assert_eq!(
                positions,
                words(&bytes(case, "positions_after")),
                "positions case{index}"
            );
            let party: Vec<u8> = crate::support::record_bytes(&warriors);
            assert_eq!(party, bytes(case, "warriors_after"), "party case{index}");
            crate::support::expect(state.counter as i64, &case["round_after"]);
            assert_eq!(
                state.slots.to_vec(),
                words(&bytes(case, "slots_after")),
                "slots case{index}"
            );
            crate::support::native_expect! {
            scalar random.seed as u64 => &case["seed_after"], "binary_round_stories";
            scalar random.draws => &case["rng_draws"], "binary_round_stories";
            scalar hostile_additions as i64 + initial_hostiles => &case["hostile_count_after"], "binary_round_stories";
            }
            let native: Vec<RoundEffect> = case
                .array("events")
                .iter()
                .filter_map(|event| {
                    let id = event[1].as_i64()? as i16;
                    match event[0].as_str()? {
                        "sound" => Some(RoundEffect::Sound(id)),
                        "text" => Some(RoundEffect::Text(id)),
                        "remove" => Some(RoundEffect::Removed(id + 10)),
                        "spawn" => Some(RoundEffect::Spawn {
                            body: id,
                            monster: event[2].as_i64()? as i16,
                        }),
                        _ => None,
                    }
                })
                .collect();
            assert_eq!(effects, native, "ordered effects case{index}");
            if fixture_name == "additional-rounds.json" && index < 4 {
                let mut content = content();
                let map = &mut content.scenario.as_mut().unwrap().maps[4];
                // Stop the parent at its next slot, before reduction/initiative adds unrelated RNG.
                let sentinel = &mut std::sync::Arc::make_mut(&mut map.shared).door_scripts[42..84];
                sentinel.fill(0);
                sentinel[8..10].copy_from_slice(&1_i16.to_be_bytes());
                sentinel[24..26].copy_from_slice(&155_i16.to_be_bytes());
                let mut session = crate::support::mission_session(&content);
                let (mut arena, party, start_random) = input(&fixture, case);
                session.warriors = party;
                session.random = start_random;
                session.phase = Phase::Combat;
                arena.cursor = 110;
                arena.round_scripts = RoundScripts {
                    counter: state.counter,
                    slots: words(&bytes(case, "slots_before")).try_into().unwrap(),
                    pending: Some(round_stories::RoundProgram {
                        slot: 0,
                        script: Some(11),
                        cursor: 3,
                        text: None,
                        delegated: true,
                    }),
                };
                arena.round_scripts.slots[1] = 1;
                session.combat = Some(arena);
                session.round_dispatch = Some(round_stories::RoundDispatch {
                    suspended_script: None,
                });
                let position = session.exploration.as_mut().unwrap();
                position.restore_map(
                    4,
                    centurions_core::MapState::authored(content.map(4).unwrap()),
                );
                position.script = Some(crate::support::script_state(
                    mission
                        .script(case.words("slots_before")[0])
                        .unwrap()
                        .to_vec(),
                    0,
                ));
                position.presentation = Some(Presentation::Dialogue {
                    id: 0,
                    text: String::new(),
                });
                let mut resumed =
                    crate::support::checkpoint(&session, &content, "shared-round-before");
                let mut observed = vec![];
                for _ in 0..8 {
                    let events =
                        replay_command(Command::Continue, &mut session, &mut resumed, &content);
                    for event in events {
                        match event {
                            Event::Sound { resource_id } => {
                                observed.push(RoundEffect::Sound(resource_id))
                            }
                            Event::CombatantAdded { body, definition } => {
                                observed.push(RoundEffect::Spawn {
                                    body,
                                    monster: definition,
                                })
                            }
                            Event::StoryPresented => {
                                let Some(Presentation::Dialogue { id, .. }) =
                                    session.exploration.as_ref().unwrap().presentation
                                else {
                                    panic!("round dialogue missing")
                                };
                                observed.push(RoundEffect::Text(id));
                            }
                            _ => {}
                        }
                    }
                    if session.round_dispatch.is_none() {
                        break;
                    }
                    session = crate::support::checkpoint(&session, &content, "shared-round-paused");
                    resumed = session.clone();
                }
                let actual = session.combat.as_ref().unwrap();
                assert!(session.round_dispatch.is_none());
                let mut expected_effects = native.clone();
                expected_effects.push(RoundEffect::Text(155));
                assert_eq!(observed, expected_effects, "shared round case{index}");
                crate::support::assert_bytes(
                    &crate::support::record_bytes(&actual.enemies.records),
                    &bytes(case, "records_after"),
                    &format!("shared round monsters case{index}"),
                );
                assert_eq!(actual.enemies.positions, combat.enemies.positions);
                assert_eq!(actual.map, combat.map);
                assert_eq!(actual.order, combat.order);
                assert_eq!(actual.round_scripts.slots[0], state.slots[0]);
                assert_eq!(session.random, random);
                crate::support::assert_bytes(
                    &crate::support::record_bytes(&session.warriors),
                    &bytes(case, "warriors_after"),
                    &format!("shared round warriors case{index}"),
                );
                crate::support::checkpoint(&session, &content, "shared-round-after");
            }
            let carried = fixture_name == "retained-round-origin.json";
            if carried
                || fixture_name == "removal-rounds.json" && case.n::<u32>("seed_before") == 12
            {
                let content = content();
                let mut state = crate::support::mission_session(&content);
                let (mut arena, warriors, random) = input(&fixture, case);
                state.warriors = warriors;
                for warrior in &mut state.warriors {
                    warrior.bytes[0x9df] = 1;
                }
                state.random = random;
                state.phase = Phase::Combat;
                let mut health = centurions_core::story_health::context(&state.warriors, 0, None);
                health.last_body = case["last_defeated"].as_i64().unwrap_or(-1) as i16;
                state.health_context = Some(health);
                if !carried {
                    arena.order[109] = 0;
                }
                arena.cursor = 109;
                arena.round = case.n("round_before");
                arena.round_scripts = RoundScripts {
                    counter: case.n("round_before"),
                    slots: words(&bytes(case, "slots_before")).try_into().unwrap(),
                    pending: None,
                };
                if carried {
                    arena.round_scripts.slots[1] = 35;
                }
                state.combat = Some(arena);
                let mut events = apply(Command::EndTurn, &mut state, &content).unwrap();
                if carried {
                    let mut resumed =
                        crate::support::checkpoint(&state, &content, "retained-summon");
                    let next =
                        replay_command(Command::Continue, &mut state, &mut resumed, &content);
                    events.extend(next);
                }
                assert_eq!(
                    crate::support::record_bytes(&state.combat.as_ref().unwrap().enemies.records),
                    bytes(case, "records_after")
                );
                assert_eq!(
                    (state.random.seed, state.random.draws),
                    (case.n("seed_after"), case.n("rng_draws"))
                );
                assert!(matches!(events.last(), Some(Event::StoryPresented)));
                for effect in &native {
                    if let RoundEffect::Removed(body) = effect {
                        assert!(events.contains(&Event::CombatantRemoved { body: *body }));
                    }
                }
                let mut resumed = crate::support::checkpoint(&state, &content, "round");
                replay_command(Command::Continue, &mut state, &mut resumed, &content);
                let scripts = &state.combat.as_ref().unwrap().round_scripts;
                assert!(scripts.pending.is_none());
            }
        }
    }
}

#[test]
fn original_round_surrender_enters_question_then_declines_or_cleans_without_rewards() {
    let content = content();
    let fixture = crate::support::fixture("round-surrender.json");
    for (case, stop) in fixture.array("cases").iter().zip(fixture.array("stops")) {
        let mut state = crate::support::mission_session(&content);
        let (mut arena, warriors, random) = input(&fixture, case);
        state.warriors = warriors;
        state.random = random;
        state.phase = Phase::Combat;
        state.ally_count = 0;
        state.allies.fill(0);
        arena.cursor = 109;
        arena.order[109] = 0;
        arena.round_scripts.counter = 2;
        arena.round_scripts.slots = [10, 0, 0, 0, 0];
        state.combat = Some(arena);
        let position = state.exploration.as_mut().unwrap();
        position.restore_map(
            4,
            centurions_core::MapState::authored(content.map(4).unwrap()),
        );
        position.x = 50;
        position.y = 50;
        position.last_move = [1, 0];
        position.script = None;
        position.presentation = None;
        let events = apply(Command::EndTurn, &mut state, &content).unwrap();
        assert!(events.contains(&Event::Sound { resource_id: 10136 }));
        assert!(matches!(
            state.exploration.as_ref().unwrap().presentation,
            Some(Presentation::Dialogue { id: -92, .. })
        ));
        apply(Command::Continue, &mut state, &content).unwrap();
        assert!(matches!(
            state.exploration.as_ref().unwrap().presentation,
            Some(Presentation::Question { extra: 50, .. })
        ));
        let mut resumed = crate::support::checkpoint(&state, &content, "round-surrender");
        let command = Command::Choose {
            answer: stop.n::<u8>("answer") != 0,
        };
        assert_eq!(
            apply(command.clone(), &mut state, &content).unwrap(),
            apply(command, &mut resumed, &content).unwrap()
        );
        assert_eq!(state, resumed);
        let position = state.exploration.as_ref().unwrap();
        assert_eq!((position.x, position.y), (stop.n("x"), stop.n("y")));
        if stop.n::<u8>("answer") == 0 {
            assert!(state.round_dispatch.is_none());
            assert!(
                state
                    .combat
                    .as_ref()
                    .unwrap()
                    .round_scripts
                    .pending
                    .is_none()
            );
            assert_eq!(state.combat.as_ref().unwrap().round_scripts.counter, 3);
        } else {
            assert!(state.combat.is_none());
            assert_eq!(
                crate::support::record_bytes(&state.warriors),
                case.bytes("warriors_after")
            );
            assert_eq!(state.allies, stop.bytes("allies"));
            assert_eq!(
                (state.random.seed, state.random.draws),
                (case.n("seed_after"), case.n("rng_draws"))
            );
            assert!(matches!(
                position.presentation,
                Some(Presentation::Dialogue { id: 93, .. })
            ));
        }
    }
}

#[test]
fn surrender_question_cleanup_and_saved_script_return_match_native_early_exit() {
    let content = content();
    for case in crate::support::fixture("story-cleanup.json").array("cases") {
        let before = &case["before"];
        let mut state = crate::support::mission_session(&content);
        state.phase = Phase::Combat;
        state.warriors = crate::support::warriors(&before["party"]);
        state.allies = before.bytes("allies");
        state.ally_count = 2;
        state.random = RandomState::new(before.n("seed"));
        let mut arena = crate::support::battle();
        arena.enemies.records = crate::support::monsters(&before["records"]);
        arena.enemies.positions = vec![(40, 40); 2];
        arena.squad.positions = crate::support::positions(&before["party_positions"]);
        arena.cursor = 110;
        arena.round_scripts.pending = Some(round_stories::RoundProgram {
            slot: 0,
            script: Some(11),
            cursor: 3,
            text: None,
            delegated: true,
        });
        state.combat = Some(arena);
        state.round_dispatch = Some(round_stories::RoundDispatch {
            suspended_script: None,
        });
        let position = state.exploration.as_mut().unwrap();
        position.restore_map(
            4,
            centurions_core::MapState::authored(content.map(4).unwrap()),
        );
        position.script = Some(crate::support::script_state(
            content.map(4).unwrap().script(11).unwrap().to_vec(),
            3,
        ));
        position.presentation = Some(Presentation::Question {
            extra: 50,
            text: String::new(),
        });
        let map = position.map_state();
        let mut resumed = crate::support::checkpoint(&state, &content, "surrender-question");
        assert_eq!(
            apply(Command::Choose { answer: true }, &mut state, &content).unwrap(),
            apply(Command::Choose { answer: true }, &mut resumed, &content).unwrap()
        );
        assert_eq!(state, resumed);
        let after = &case["after"];
        assert_eq!(
            crate::support::record_bytes(&state.warriors),
            after.bytes("party")
        );
        assert_eq!(state.allies, after.bytes("allies"));
        assert_eq!(
            state
                .booty_items
                .iter()
                .flat_map(|record| record.0)
                .collect::<Vec<_>>(),
            after.bytes("booty")
        );
        assert_eq!(
            (state.random.seed, state.random.draws),
            (after.n("seed"), after.n("draws"))
        );
        assert_eq!(
            state.health_context.as_ref().unwrap().unavailable,
            after.n::<i8>("unavailable")
        );
        assert!(state.combat.is_none());
        assert_eq!(state.exploration.as_ref().unwrap().map_state(), map);
        assert!(matches!(
            state.exploration.as_ref().unwrap().presentation,
            Some(Presentation::Dialogue { id: 93, .. })
        ));
        resumed = crate::support::checkpoint(&state, &content, "surrender-cleaned");
        for _ in 0..40 {
            if state.round_dispatch.is_none() {
                break;
            }
            let command = if matches!(
                state.exploration.as_ref().unwrap().presentation,
                Some(Presentation::Menu { .. })
            ) {
                Command::CancelMenu
            } else {
                Command::Continue
            };
            replay_command(command, &mut state, &mut resumed, &content);
            resumed = crate::support::checkpoint(&state, &content, "surrender-script");
        }
        assert!(state.round_dispatch.is_none());
        assert!(state.scripted_cleanup.is_none());
    }
}
