use crate::{native_common, prelude::*, support};

#[test]
fn summon_capacity_matches_native_boundaries_and_rejects_position_aliasing_atomically() {
    let fixture = support::fixture("summon-capacity-boundary.json");
    let content = content();
    let mut mission = content.map(0).unwrap().clone();
    let tables = std::sync::Arc::make_mut(&mut mission.shared);
    tables.door_scripts = vec![0; 84];
    tables.door_scripts[42 + 8..42 + 10].copy_from_slice(&124_i16.to_be_bytes());
    tables.door_scripts[42 + 24..42 + 26].copy_from_slice(&73_i16.to_be_bytes());
    let mut extras = vec![0; 740];
    extras[730..740].copy_from_slice(&fixture.bytes("extra73"));
    for case in fixture.array("cases") {
        let before = &case["before"];
        let after = &case["after"];
        let count = case.n::<usize>("count_before");
        let mut combat = support::battle();
        combat.roster.records = support::monsters(&before.bytes("records"))[..count].to_vec();
        combat.roster.mon_pos =
            support::positions(&before.bytes("npc_positions"))[..count].to_vec();
        combat.squad.pos = support::positions(&before.bytes("party_positions"));
        combat.field.tiles = before.words("map");
        combat.field.map_shift = (-5, -5);
        combat.battle_order = before.words("order");
        let mut scripts = RoundMacros {
            slots: [1, 0, 0, 0, 0],
            ..Default::default()
        };
        let mut random = RandomState::new(case.n("seed_before"));
        let original = (combat.clone(), scripts.clone(), random.clone());
        let result = round_stories::advance(
            &mut combat,
            &mut scripts,
            &mission,
            &mission.combat.as_ref().unwrap().monsters,
            &extras,
            round_stories::RoundCaller {
                active_enemy: 0,
                last_defeated: None,
                party_positions: [(0, 0); 6],
            },
            &mut random,
        );
        if after.n::<usize>("allocated") > 100 {
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("summon count exceeds the native roster capacity")
            );
            assert_eq!((combat, scripts, random), original);
            assert_ne!(after["party_positions"], before["party_positions"]);
        } else {
            let result = result.unwrap();
            let size = after.n::<usize>("allocated");
            support::assert_bytes(
                &support::record_bytes(&combat.roster.records),
                &after.bytes("records")[..size * 288],
                "summon boundary records",
            );
            assert_eq!(
                combat.roster.mon_pos,
                support::positions(&after.bytes("npc_positions"))[..size]
            );
            assert_eq!(combat.field.tiles, after.words("map"));
            assert_eq!(combat.battle_order, after.words("order"));
            assert_eq!(combat.squad.pos, original.0.squad.pos);
            support::assert_rng(&random, &after["seed"], &after["draws"]);
            assert_eq!(
                result.hostile_additions,
                after.n::<i16>("hostile_count") - before.n::<i16>("hostile_count")
            );
        }
    }
}

use centurions_core::combat::round_stories;
use centurions_core::combat::round_stories::RoundEffect;

#[test]
fn shared_round_operations_match_native_flags_pages_slots_and_saved_returns() {
    let fixture = support::fixture("shared-round-operations.json");
    let foundation = support::fixture("round-stories.json");
    for case in fixture.array("cases") {
        let mut content = content();
        let tables = Arc::make_mut(&mut content.scenario.as_mut().unwrap().maps[0].shared);
        tables.door_scripts[42..84].copy_from_slice(&fixture.bytes("first_record"));
        tables.door_scripts[84..126].copy_from_slice(&fixture.bytes("second_record"));
        tables.combat.as_mut().unwrap().encounters.resize(9020, 0);
        tables.combat.as_mut().unwrap().encounters[9000..9010]
            .copy_from_slice(&case.bytes("extra"));
        let mut state = support::mission_session(&content);
        let (mut arena, warriors, random) = input(&foundation, &foundation.array("cases")[0]);
        state.warriors = warriors;
        state.random = random;
        let initial_random = state.random.clone();
        state.flags[..100].copy_from_slice(&case.bytes("flags_before"));
        state.phase = Phase::Combat;
        state.held_over = 0;
        state.holdover.fill(0);
        arena.battle_order_index = 109;
        arena.battle_order[109] = 0;
        arena.round_macros = RoundMacros {
            counter: case.n("counter_before"),
            slots: [1, 2, 0, 0, 0],
            pending: None,
        };
        state.combat = Some(arena);
        crate::native_common::source_victory_context(&mut state);
        let position = state.exploration.as_mut().unwrap();
        position.script = None;
        position.presentation = None;
        let mut sounds =
            support::event_sounds(&apply(Command::EndTurn, &mut state, &content).unwrap());
        for (page_index, page) in case.array("pages").iter().enumerate() {
            assert!(matches!(state.exploration.as_ref().unwrap().presentation,
                Some(Presentation::Dialogue { id, .. }) if id == page.n::<i16>("id")));
            assert_eq!(&state.flags[..100], page.bytes("flags"));
            assert_eq!(state.random, initial_random);
            let program = state
                .combat
                .as_ref()
                .unwrap()
                .round_macros
                .pending
                .as_ref()
                .unwrap();
            assert_eq!(program.slot, page.n::<u8>("slot"));
            assert!(program.delegated);
            assert_eq!(
                state.combat.as_ref().unwrap().round_macros.counter,
                page.n::<i8>("counter")
            );
            let restored = support::checkpoint(&state, &content, "shared-round-operations");
            if page_index + 1 < case.array("pages").len() {
                let mut restored = restored;
                sounds.extend(support::event_sounds(&replay_command(
                    Command::Continue,
                    &mut state,
                    &mut restored,
                    &content,
                )));
            }
        }
        sounds.retain(|id| (5..=7).contains(id));
        assert_eq!(sounds, [5, 6, 7]);
        support::saved_command(Command::Continue, &mut state, &content);
        assert!(state.round_dispatch.is_none());
        assert!(state.exploration.as_ref().unwrap().script.is_none());
        assert_eq!(&state.flags[..100], case.bytes("flags_after"));
        assert_eq!(
            state.combat.as_ref().unwrap().round_macros.counter,
            case.n::<i8>("counter_after")
        );
        assert_eq!(
            state
                .combat
                .as_ref()
                .unwrap()
                .round_macros
                .slots
                .as_slice(),
            words(&case.bytes("slots_after"))
        );
    }
}

#[test]
fn summoned_difficulty_matches_original_record_widths_rounding_and_rng() {
    let fixture = support::fixture("difficulty-summons.json");
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
        support::assert_rng(&random, &case["seed_after"], &case["draws"]);
    }
}

use native_common::script_mission as mission;
/// Only the literal captured counter fields; source contexts construct the rest.
fn recorded_victory(case: &Value) -> (Option<i16>, Option<i16>) {
    let word = |key| case.get(key).and_then(Value::as_i64).map(|n| n as i16);
    (word("hostile_count_before"), word("dead_before"))
}

pub(crate) fn input(
    fixture: &serde_json::Value,
    case: &serde_json::Value,
) -> (Combat, Vec<Warrior>, RandomState) {
    let mut combat = support::battle();
    // These controlled round inputs start with cold NPC backing. The active
    // native record comparisons below do not qualify inactive slot behavior.
    combat.monsters = Session::new(&support::fixture_content("round backing"))
        .globals
        .monsters;
    combat.battle = 146;
    combat.field.tiles = case.words("map_before");
    combat.squad.pos = (0..6).map(|i| (40 + i, 40)).collect();
    combat.roster.records = support::monsters(&case["records_before"]);
    combat.roster.mon_pos = support::positions(&case["positions_before"]);
    combat.battle_order[1] = 10;
    if case.get("order_before").is_some() {
        combat.battle_order = case.words("order_before");
    }
    combat.macro_monster = case["actor_before"].as_u64().unwrap_or(0) as usize;
    let (total, defeated) = recorded_victory(case);
    combat.kill_counts = centurions_core::combat::kill_counts::KillCounts {
        num_enemy: total.unwrap_or(0),
        kill_mon: defeated.unwrap_or(0),
    };
    (
        combat,
        support::warriors(&fixture["warriors_before"]),
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
        let fixture = support::fixture(fixture_name);
        let mission = mission(&fixture);
        let definitions = fixture.bytes("monsters");
        let extras = fixture.bytes("extras");
        for (index, case) in fixture.array("cases").iter().enumerate() {
            let (mut combat, warriors, mut random) = input(&fixture, case);
            let initial_hostiles = case["hostile_count_before"].as_i64().unwrap_or_else(|| {
                combat
                    .roster
                    .records
                    .iter()
                    .filter(|m| {
                        m.stamina().unwrap() > 0
                            && centurions_content::word(&m.bytes, 14).unwrap() != 0
                    })
                    .count() as i64
            });
            let slot_words = words(&case.bytes("slots_before"));
            let mut state = RoundMacros {
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
                        active_enemy: combat.macro_monster,
                        last_defeated: case["last_defeated"].as_i64().map(|body| body as i16),
                        party_positions: [(0, 0); 6],
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
            if let Some(dead) = case.get("dead_after").and_then(Value::as_i64) {
                assert_eq!(
                    combat.kill_counts.kill_mon,
                    dead as i16,
                    "defeated case{index}"
                );
            }
            if case.get("hostile_count_before").is_some() {
                assert_eq!(
                    combat.kill_counts.num_enemy,
                    case.n::<i16>("hostile_count_after")
                );
            }
            let records: Vec<u8> = support::record_bytes(&combat.roster.records);
            assert_eq!(records, case.bytes("records_after"), "records case{index}");
            assert_eq!(
                combat.field.tiles,
                words(&case.bytes("map_after")),
                "map case{index}"
            );
            assert_eq!(
                combat.battle_order,
                words(&case.bytes("order_after")),
                "order case{index}"
            );
            let positions: Vec<i16> = combat
                .roster
                .mon_pos
                .iter()
                .flat_map(|&(x, y)| [x, y])
                .collect();
            assert_eq!(
                positions,
                words(&case.bytes("positions_after")),
                "positions case{index}"
            );
            let party: Vec<u8> = support::record_bytes(&warriors);
            assert_eq!(party, case.bytes("warriors_after"), "party case{index}");
            support::expect(state.counter as i64, &case["round_after"]);
            assert_eq!(
                state.slots.to_vec(),
                words(&case.bytes("slots_after")),
                "slots case{index}"
            );
            support::native_expect! {
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
                let mut session = support::mission_session(&content);
                let (mut arena, party, start_random) = input(&fixture, case);
                session.warriors = party;
                session.random = start_random;
                session.phase = Phase::Combat;
                arena.battle_order_index = 110;
                arena.round_macros = RoundMacros {
                    counter: state.counter,
                    slots: words(&case.bytes("slots_before")).try_into().unwrap(),
                    pending: Some(round_stories::RoundProgram {
                        slot: 0,
                        script: Some(11),
                        cursor: 3,
                        text: None,
                        delegated: true,
                    }),
                };
                arena.round_macros.slots[1] = 1;
                session.combat = Some(arena);
                session.round_dispatch = Some(round_stories::RoundDispatch {
                    suspended_script: None,
                    ..Default::default()
                });
                let position = session.exploration.as_mut().unwrap();
                position.restore_map(
                    4,
                    centurions_core::MapState::authored(content.map(4).unwrap()),
                );
                position.script = Some(centurions_core::ScriptState::scheduled(
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
                let mut resumed = support::checkpoint(&session, &content, "shared-round-before");
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
                    session = support::checkpoint(&session, &content, "shared-round-paused");
                    resumed = session.clone();
                }
                let actual = current_combat(&session);
                assert!(session.round_dispatch.is_none());
                let mut expected_effects = native.clone();
                expected_effects.push(RoundEffect::Text(155));
                assert_eq!(observed, expected_effects, "shared round case{index}");
                support::native_expect! {
                    bytes &support::record_bytes(&actual.roster.records) => &case.bytes("records_after"), &format!("shared round monsters case{index}");
                };
                assert_eq!(actual.roster.mon_pos, combat.roster.mon_pos);
                assert_eq!(actual.field, combat.field);
                assert_eq!(actual.battle_order, combat.battle_order);
                assert_eq!(actual.round_macros.slots[0], state.slots[0]);
                assert_eq!(session.random, random);
                support::native_expect! {
                    bytes &support::record_bytes(&session.warriors) => &case.bytes("warriors_after"), &format!("shared round warriors case{index}");
                };
                support::checkpoint(&session, &content, "shared-round-after");
            }
            let carried = fixture_name == "retained-round-origin.json";
            if carried
                || fixture_name == "removal-rounds.json" && case.n::<u32>("seed_before") == 12
            {
                let content = content();
                let mut state = support::mission_session(&content);
                let (mut arena, warriors, random) = input(&fixture, case);
                state.warriors = warriors;
                for warrior in &mut state.warriors {
                    warrior.bytes[0x9df] = 1;
                }
                state.random = random;
                state.phase = Phase::Combat;
                let mut health = centurions_core::story_health::context(&state.warriors, 0, None);
                health.last_body = case["last_defeated"].as_i64().unwrap_or(-1) as i16;
                state.globals.health_context = health;
                if !carried {
                    arena.battle_order[109] = 0;
                }
                arena.battle_order_index = 109;
                arena.combat_round = case.n("round_before");
                arena.round_macros = RoundMacros {
                    counter: case.n("round_before"),
                    slots: words(&case.bytes("slots_before")).try_into().unwrap(),
                    pending: None,
                };
                if carried {
                    arena.round_macros.slots[1] = 35;
                }
                state.combat = Some(arena);
                let (total, defeated) = recorded_victory(case);
                crate::native_common::source_victory_context_recorded(&mut state, total, defeated);
                let mut events = apply(Command::EndTurn, &mut state, &content).unwrap();
                if carried {
                    let mut resumed = support::checkpoint(&state, &content, "retained-summon");
                    let next =
                        replay_command(Command::Continue, &mut state, &mut resumed, &content);
                    events.extend(next);
                }
                assert_eq!(
                    support::record_bytes(&current_combat(&state).roster.records),
                    case.bytes("records_after")
                );
                support::assert_rng(&state.random, &case["seed_after"], &case["rng_draws"]);
                assert!(matches!(events.last(), Some(Event::StoryPresented)));
                for effect in &native {
                    if let RoundEffect::Removed(body) = effect {
                        assert!(events.contains(&Event::CombatantRemoved { body: *body }));
                    }
                }
                support::saved_command(Command::Continue, &mut state, &content);
                let scripts = &current_combat(&state).round_macros;
                assert!(scripts.pending.is_none());
            }
        }
    }
}

#[test]
fn original_round_surrender_enters_question_then_declines_or_cleans_without_rewards() {
    let content = content();
    let fixture = support::fixture("round-surrender.json");
    for (case, stop) in fixture.array("cases").iter().zip(fixture.array("stops")) {
        let mut state = support::mission_session(&content);
        let (mut arena, warriors, random) = input(&fixture, case);
        state.warriors = warriors;
        state.random = random;
        state.phase = Phase::Combat;
        state.held_over = 0;
        state.holdover.fill(0);
        arena.battle_order_index = 109;
        arena.battle_order[109] = 0;
        arena.round_macros.counter = 2;
        arena.round_macros.slots = [10, 0, 0, 0, 0];
        state.combat = Some(arena);
        let (total, defeated) = recorded_victory(case);
        crate::native_common::source_victory_context_recorded(&mut state, total, defeated);
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
        let mut resumed = support::checkpoint(&state, &content, "round-surrender");
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
            assert!(current_combat(&state).round_macros.pending.is_none());
            assert_eq!(current_combat(&state).round_macros.counter, 3);
        } else {
            assert!(state.combat.is_none());
            assert_eq!(
                support::record_bytes(&state.warriors),
                case.bytes("warriors_after")
            );
            assert_eq!(state.holdover, stop.bytes("allies"));
            support::assert_rng(&state.random, &case["seed_after"], &case["rng_draws"]);
            assert!(matches!(
                position.presentation,
                Some(Presentation::Dialogue { id: 93, .. })
            ));
            // Authored alternative after surrender: destroy the containers, then return from BD6.
            for _ in 0..40 {
                if matches!(
                    state.exploration.as_ref().unwrap().presentation,
                    Some(Presentation::Menu { id: 0, .. })
                ) {
                    break;
                }
                replay_command(Command::Continue, &mut state, &mut resumed, &content);
            }
            replay_command(
                Command::SelectMenuEntry { index: 3 },
                &mut state,
                &mut resumed,
                &content,
            );
            for _ in 0..8 {
                if state.combat.is_some() {
                    break;
                }
                replay_command(Command::Continue, &mut state, &mut resumed, &content);
            }
            assert_eq!(current_combat(&state).battle, 6);
            state = support::checkpoint(&state, &content, "round-child-battle");
            for candidate in [&mut state, &mut resumed] {
                let arena = candidate.combat.as_mut().unwrap();
                for body in 0..arena.roster.records.len() {
                    arena.roster.records[body].bytes[0x110..0x112].fill(0);
                    centurions_core::combat::lifecycle::kill_body(
                        arena,
                        &mut candidate.warriors,
                        body as i16 + 10,
                    )
                    .unwrap();
                }
            }
            replay_command(Command::EndTurn, &mut state, &mut resumed, &content);
            advance_dispatch(&mut state, &mut resumed, &content, 30, None);
            assert!(state.round_dispatch.is_none() && state.combat.is_none());
        }
    }
}

#[test]
fn surrender_question_cleanup_and_saved_script_return_match_native_early_exit() {
    let content = content();
    for case in support::fixture("story-cleanup.json").array("cases") {
        let before = &case["before"];
        let mut state = support::mission_session(&content);
        state.phase = Phase::Combat;
        state.warriors = support::warriors(&before["party"]);
        state.holdover = before.bytes("allies");
        state.held_over = 2;
        state.random = RandomState::new(before.n("seed"));
        let mut arena = support::battle();
        arena.roster.records = support::monsters(&before["records"]);
        arena.roster.mon_pos = vec![(40, 40); 2];
        arena.squad.pos = support::positions(&before["party_positions"]);
        arena.battle_order_index = 110;
        arena.round_macros.pending = Some(round_stories::RoundProgram {
            slot: 0,
            script: Some(11),
            cursor: 3,
            text: None,
            delegated: true,
        });
        state.combat = Some(arena);
        crate::native_common::source_victory_context(&mut state);
        state.round_dispatch = Some(round_stories::RoundDispatch {
            suspended_script: None,
            ..Default::default()
        });
        let position = state.exploration.as_mut().unwrap();
        position.restore_map(
            4,
            centurions_core::MapState::authored(content.map(4).unwrap()),
        );
        position.script = Some(centurions_core::ScriptState::scheduled(
            content.map(4).unwrap().script(11).unwrap().to_vec(),
            3,
        ));
        position.presentation = Some(Presentation::Question {
            extra: 50,
            text: String::new(),
        });
        let map = position.map_state();
        let mut resumed = support::checkpoint(&state, &content, "surrender-question");
        assert_eq!(
            apply(Command::Choose { answer: true }, &mut state, &content).unwrap(),
            apply(Command::Choose { answer: true }, &mut resumed, &content).unwrap()
        );
        assert_eq!(state, resumed);
        let after = &case["after"];
        assert_eq!(support::record_bytes(&state.warriors), after.bytes("party"));
        assert_eq!(state.holdover, after.bytes("allies"));
        assert_eq!(
            support::record_bytes(&state.booty_list),
            after.bytes("booty")
        );
        support::assert_rng(&state.random, &after["seed"], &after["draws"]);
        assert_eq!(
            state
                .globals
                .health_context
                .kill_party,
            after.n::<i8>("unavailable")
        );
        assert!(state.combat.is_none());
        assert_eq!(state.exploration.as_ref().unwrap().map_state(), map);
        assert!(matches!(
            state.exploration.as_ref().unwrap().presentation,
            Some(Presentation::Dialogue { id: 93, .. })
        ));
        resumed = support::checkpoint(&state, &content, "surrender-cleaned");
        advance_dispatch(
            &mut state,
            &mut resumed,
            &content,
            40,
            Some("surrender-script"),
        );
        assert!(state.round_dispatch.is_none());
        assert!(state.scripted_cleanup.is_none());
    }
}

/// Advances a round dispatch as the desktop does: dismiss a menu, otherwise Continue.
fn advance_dispatch(
    state: &mut Session,
    resumed: &mut Session,
    content: &Content,
    limit: usize,
    checkpoint: Option<&str>,
) {
    for _ in 0..limit {
        if state.round_dispatch.is_none() {
            break;
        }
        let menu = matches!(
            state.exploration.as_ref().unwrap().presentation,
            Some(Presentation::Menu { .. })
        );
        let command = if menu {
            Command::CancelMenu
        } else {
            Command::Continue
        };
        replay_command(command, state, resumed, content);
        if let Some(label) = checkpoint {
            *resumed = support::checkpoint(state, content, label);
        }
    }
}
