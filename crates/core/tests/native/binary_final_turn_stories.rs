use crate::{native_common, prelude::*, support};

use centurions_core::combat::final_turn_stories;
use centurions_core::combat::final_turn_stories::TurnEffect;

use native_common::script_mission as mission;
fn input(case: &serde_json::Value) -> (Combat, Vec<Warrior>, RandomState) {
    let mut combat = support::battle();
    combat.battle = 134;
    combat.map.tiles = case.words("map_before");
    combat.squad.positions = (0..6).map(|i| (40 + i, 40)).collect();
    combat.enemies.records = support::monsters(&case["records_before"]);
    combat.enemies.positions = support::positions(&case["positions_before"]);
    combat.order[1] = 10 + case["active_index"].as_i64().unwrap_or(0) as i16;
    combat.script_enemy = case["actor_before"].as_u64().unwrap_or(0) as usize;
    (
        combat,
        support::warriors(&case["warriors_before"]),
        RandomState::new(case.n::<u32>("seed_before")),
    )
}

#[test]
fn original_final_turn_stories_match_all_body_map_order_party_and_rng_bytes() {
    let fixture = support::fixture("final-turn-stories.json");
    let mission = mission(&fixture);
    let definitions = fixture.bytes("monsters");
    let extras = fixture.bytes("extras");
    let earlier = support::fixture("earlier-turn-stories.json");
    let retained = support::fixture("retained-turn-origin.json");
    let conditions = support::fixture("npc-turn-conditions.json");
    let replacement = support::fixture("body-replacement.json");
    let selected = support::fixture("retained-body-replacement.json");
    let builder = support::fixture("difficulty-summons.json");
    let builder = &builder["cases"][replacement.n::<usize>("builder_case")];
    for case in replacement
        .array("cases")
        .iter()
        .chain(selected.array("cases"))
    {
        let mut definition = builder.bytes("before");
        definition[0x5c..0x5e].copy_from_slice(&23_i16.to_be_bytes());
        let selected = case["selected"].as_u64().unwrap_or(0) as usize;
        let stopped = case.n::<bool>("stopped_script");
        let mut content = content();
        let tables =
            std::sync::Arc::make_mut(&mut content.scenario.as_mut().unwrap().maps[0].shared)
                .combat
                .as_mut()
                .unwrap();
        tables.monsters[MONSTER_BYTES..576].copy_from_slice(&definition);
        let extra = [1_i16, 0, 655, 469, 470];
        tables.encounters[2470..2480].copy_from_slice(&support::word_bytes(&extra));
        let mut state = support::mission_session(&content);
        let mut arena = support::battle();
        arena.difficulty = builder.n("difficulty");
        arena.selected_enemy = selected;
        arena.enemies.records = support::monsters(&case.bytes("before"));
        arena.enemies.positions = vec![(10, 10); selected + 1];
        arena.order[1] = 0;
        state.phase = Phase::Combat;
        state.combat = Some(arena);
        state.random = RandomState::new(builder.n::<u32>("seed_before"));
        let mut script = vec![0; 42];
        script[8..10].copy_from_slice(&120_i16.to_be_bytes());
        script[24..26].copy_from_slice(&247_i16.to_be_bytes());
        let position = state.exploration.as_mut().unwrap();
        position.script = Some(centurions_core::ScriptState::scheduled(script, 0));
        position.presentation = Some(Presentation::Dialogue {
            id: 469,
            text: String::new(),
        });
        let mut resumed = support::checkpoint(&state, &content, "actor-before");
        support::replay_command(Command::Continue, &mut state, &mut resumed, &content);
        assert_eq!(
            support::record_bytes(&current_combat(&state).enemies.records),
            case.bytes("before")
        );
        state = support::checkpoint(&state, &content, "actor-introduction");
        resumed = state.clone();
        let events = support::replay_command(Command::Continue, &mut state, &mut resumed, &content);
        assert_eq!(
            support::record_bytes(&current_combat(&state).enemies.records),
            case.bytes("after")
        );
        support::assert_rng(&state.random, &builder["seed_after"], &builder["draws"]);
        if !stopped {
            assert_eq!(support::event_sounds(&events), vec![655]);
            state = support::checkpoint(&state, &content, "actor-conclusion");
            resumed = state.clone();
            support::replay_command(Command::Continue, &mut state, &mut resumed, &content);
        }
        assert!(state.actor_replacement.is_none());
    }
    for (index, case) in fixture
        .array("cases")
        .iter()
        .chain(earlier.array("cases"))
        .chain(retained.array("cases"))
        .chain(conditions.array("cases"))
        .enumerate()
    {
        let (mut combat, mut warriors, mut random) = input(case);
        let outcome = final_turn_stories::initialize(
            &mut combat,
            &mut warriors,
            &mission,
            &definitions,
            &extras,
            &mut random,
        )
        .unwrap_or_else(|error| panic!("case{index}: {error}"));
        let records: Vec<u8> = support::record_bytes(&combat.enemies.records);
        assert_eq!(records, case.bytes("records_after"), "records case{index}");
        assert_eq!(
            combat.map.tiles,
            words(&case.bytes("map_after")),
            "map case{index}"
        );
        assert_eq!(
            combat.order,
            words(&case.bytes("order_after")),
            "order case{index}"
        );
        support::native_expect! {
            positions combat.enemies.positions => &case["positions_after"], "positions case{index}";
        };
        let party: Vec<u8> = support::record_bytes(&warriors);
        assert_eq!(party, case.bytes("warriors_after"), "party case{index}");
        support::native_expect! {
            scalar random.seed as u64 => &case["seed_after"], "binary_final_turn_stories";
            scalar random.draws => &case["rng_draws"], "binary_final_turn_stories";
            scalar outcome.skipped_turn => &case["skipped_turn"], "binary_final_turn_stories";
            scalar i64::from(outcome.increment_kills) => &case["kills_after"], "binary_final_turn_stories";
            scalar combat.script_enemy => &case["actor_after"], "binary_final_turn_stories";
        }
        let initial_hostile = case["hostile_count_before"]
            .as_i64()
            .unwrap_or_else(|| i64::from(case.n::<i64>("faction") != 0));
        support::expect(
            initial_hostile + i64::from(outcome.hostile_additions),
            &case["hostile_count_after"],
        );
        let effects: Vec<serde_json::Value> = outcome
            .effects
            .iter()
            .filter_map(|effect| match effect {
                TurnEffect::Sound(id) => Some(serde_json::json!(["sound", id])),
                TurnEffect::Text(id) => Some(serde_json::json!(["text", id])),
                TurnEffect::SelfRemoval { body } => Some(serde_json::json!(["kill", body])),
                TurnEffect::Spawn { .. } => None,
            })
            .collect();
        let native: Vec<_> = case
            .array("events")
            .iter()
            .filter(|event| ["sound", "text", "kill"].contains(&event[0].as_str().unwrap()))
            .cloned()
            .collect();
        assert_eq!(effects, native, "effect order case{index}");
        // Friendly-only final controls have already won before the getup boundary.
        if case.n::<i16>("monster_id") < 107 || case.n::<i16>("faction") != 0 {
            let content = content();
            let mut state = support::mission_session(&content);
            let (mut arena, _, start_random) = input(case);
            arena.cursor = 0;
            let conditioned = arena.enemies.records[0].bytes[0xcf] != 0
                || arena.enemies.records[0].bytes[0xc9] != 0;
            if conditioned {
                // Keep combat active when the captured actor starts friendly or changes sides.
                let mut sentinel = arena.enemies.records[0].clone();
                sentinel.bytes[0xe..0x10].copy_from_slice(&1_i16.to_be_bytes());
                arena.enemies.records.push(sentinel);
                arena.enemies.positions.push((25, 25));
                arena.order[2] = 0;
            }
            state.warriors = support::warriors(&case.bytes("warriors_before"));
            state.phase = Phase::Combat;
            state.combat = Some(arena);
            state.random = start_random;
            let events = if conditioned {
                let mut resumed = support::checkpoint(&state, &content, "npc-condition-before");
                support::replay_command(Command::EndTurn, &mut state, &mut resumed, &content)
            } else {
                apply(Command::EndTurn, &mut state, &content).unwrap()
            };
            let Some(arena) = state.combat.as_ref() else {
                assert!(outcome.skipped_turn);
                assert!(
                    combat
                        .enemies
                        .records
                        .iter()
                        .all(|record| record.stamina().unwrap() <= 0
                            || word(&record.bytes, 14).unwrap() == 0)
                );
                assert!(events.contains(&Event::BattleEnded { victory: true }));
                support::checkpoint(&state, &content, "final-story-victory");
                continue;
            };
            assert_eq!(arena.script_enemy, combat.script_enemy);
            if !outcome.skipped_turn || conditioned {
                assert_eq!(
                    support::record_bytes(&arena.enemies.records[..combat.enemies.records.len()]),
                    records
                );
                assert_eq!(state.random, random);
            } else {
                let active = case["active_index"].as_u64().unwrap_or(0) as usize;
                assert!(arena.enemies.records[active].stamina().unwrap() <= 0);
                assert_eq!(arena.enemies.records.len(), combat.enemies.records.len());
            }
            assert_eq!(arena.map.tiles, combat.map.tiles);
            assert!(matches!(
                events.last(),
                Some(Event::TurnStarted { body, .. }) if if conditioned && outcome.skipped_turn { *body == 0 } else { *body >= 10 }
            ));
            if case.get("active_index").is_some() || conditioned {
                assert_eq!(
                    state,
                    support::checkpoint(&state, &content, "retained-origin")
                );
            }
            if conditioned && !outcome.skipped_turn {
                let mut resumed = support::checkpoint(&state, &content, "npc-condition-action");
                support::replay_command(Command::RunEnemyTurn, &mut state, &mut resumed, &content);
            }
        }
    }
}
