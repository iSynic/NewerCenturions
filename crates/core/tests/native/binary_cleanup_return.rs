//! Original newland100/booty/newland2; combat and modal services are controlled.
use crate::{prelude::*, support};
use centurions_core::combat::round_stories::{RoundBattleReturn, RoundDispatch, RoundProgram};

#[test]
fn scripted_cleanup_records_rewards_and_outer_suppression_match_native() {
    for case in support::fixture("scripted-cleanup-return.json").array("cases") {
        let mut content = content();
        let tables = Arc::make_mut(&mut content.scenario.as_mut().unwrap().maps[0].shared);
        tables.door_scripts.resize(901 * 42, 0);
        tables.door_scripts[900 * 42..901 * 42].copy_from_slice(&case.bytes("inner"));
        let mut state = support::mission_session(&content);
        support::pause_script(&mut state, case.bytes("outer"), 0);
        support::saved_command(Command::Continue, &mut state, &content);
        // The probe supplies this combat-service boundary; round dispatch is composed.
        let outer = state.exploration.as_mut().unwrap().script.take();
        state.round_dispatch = Some(RoundDispatch {
            suspended_script: outer,
            ..Default::default()
        });
        support::pause_script(&mut state, case.bytes("inner"), 0);
        let arena = state.combat.as_mut().unwrap();
        arena.roster.records = support::monsters(&case["npc_before"]);
        arena.roster.mon_pos = vec![(40, 40)];
        arena.battle_order.fill(-1);
        arena.macro_monster = 0;
        arena.monster_up = 0;
        arena.battle_order_index = 110;
        arena.turn_origin = None;
        arena.kill_counts.num_enemy = 2;
        arena.kill_counts.kill_mon = 1;
        arena.spell_queue = case.bytes("fields_before");
        arena.round_macros.slots = [900, 0, 0, 0, 0];
        arena.round_macros.pending = Some(RoundProgram {
            slot: 0,
            script: Some(900),
            cursor: 1,
            text: None,
            delegated: true,
        });
        state.warriors = support::warriors(&case["party_before"]);
        state.random = RandomState::new(12);
        state.difficulty = 0;
        state.party_currency = 37;
        state.suspend_allies = 1;
        state.held_over = 0;
        state.flags.fill(0);
        state.globals.treasure = case.bytes("treasure_before");
        state.globals.extra_record = case.bytes("extra");
        let mut events = support::saved_command(Command::Continue, &mut state, &content);
        if case.n::<usize>("windows") != 0 {
            assert_eq!(
                state.exploration.as_ref().unwrap().presentation,
                Some(Presentation::Loot)
            );
            compare_records(&state, case, "prepared");
            assert_eq!(
                support::record_bytes(&state.booty_list),
                case.bytes("loot_prepared")
            );
            assert_eq!(
                state.globals.battle_rewards_suppressed,
                case.n::<i8>("suppression_prepared")
            );
            assert_eq!(state.flags[..100], case.bytes("flags_prepared"));
            events.extend(support::saved_command(
                Command::Continue,
                &mut state,
                &content,
            ));
        }
        let sounds: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                Event::Sound { resource_id } => Some(*resource_id),
                _ => None,
            })
            .collect();
        let expected: Vec<i16> = case
            .array("calls")
            .iter()
            .filter_map(|call| {
                call.as_str()
                    .unwrap()
                    .strip_prefix("sound:")
                    .map(|id| id.parse().unwrap())
            })
            .collect();
        assert_eq!(sounds, expected);
        compare_records(&state, case, "after");
        assert_eq!(state.flags[..100], case.bytes("flags_after"));
        assert_eq!(
            state.globals.treasure,
            case.bytes("treasure_after")
        );
        assert_eq!(
            state
                .globals
                .battlefield
                .spell_queue,
            case.bytes("fields_after")
        );
        assert_eq!(
            state.globals.battle_rewards_suppressed,
            case.n::<i8>("suppression_after")
        );
        assert!(state.combat.is_none() && state.ordinary_battle_rewards.is_none());
        assert!(state.exploration.as_ref().unwrap().script.is_none());
    }
}

fn compare_records(state: &Session, case: &Value, suffix: &str) {
    assert_eq!(
        support::record_bytes(&state.warriors),
        case.bytes(&format!("party_{suffix}"))
    );
    assert_eq!(
        state.globals.monsters.slots[0]
            .as_ref()
            .unwrap()
            .record
            .bytes,
        case.bytes(&format!("npc_{suffix}"))
    );
    support::expect(state.party_currency, &case[format!("currency_{suffix}")]);
    support::assert_rng(
        &state.random,
        &case[format!("seed_{suffix}")],
        &case[format!("draws_{suffix}")],
    );
}

#[test]
fn outer_early_cleanup_retains_fixed_arrays_after_zero_count_child_return() {
    let content = content();
    let capture = support::fixture("outer-early-cleanup.json");
    let case = &capture.array("cases")[0];
    let mut state = support::mission_session(&content);
    support::pause_script(&mut state, case.bytes("outer"), 0);
    support::saved_command(Command::Continue, &mut state, &content);
    // Acknowledge the child Close; original outer booty2 runs without a roster.
    state.round_dispatch = Some(RoundDispatch {
        suspended_script: state.exploration.as_mut().unwrap().script.take(),
        child_battle: true,
        outer_battle: Some(RoundBattleReturn {
            after_message: 0,
            treasure: 0,
            rewards: state.ordinary_battle_rewards.take(),
            selected: None,
            group: None,
            npc: None,
        }),
        death_depth: Some(0),
        ..Default::default()
    });
    state.ordinary_battle_rewards = Some(
        serde_json::from_value(json!({
            "mode": 0, "loot": true
        }))
        .unwrap(),
    );
    support::pause_script(&mut state, support::script_record(&[]), 0);
    state.exploration.as_mut().unwrap().presentation = Some(Presentation::Loot);
    state.combat = None;
    state.globals.monsters.num_mon = 0;
    state.phase = Phase::Exploration;
    state.warriors = support::warriors(&case["party_before"]);
    state.booty_list.fill(InventoryRecord([0; 22]));
    state.random = RandomState::new(12);
    state.party_currency = 37;
    state.flags.fill(0);
    state.holdover = case.bytes("allies_before");
    state.held_over = 1;
    state.suspend_allies = 1;
    state
        .globals
        .health_context
        .kill_party = 6;
    state.globals.extra_record = case.bytes("extra");
    state.globals.treasure = case.bytes("treasure_before");
    state
        .globals
        .battlefield
        .spell_queue = case.bytes("fields_before");
    for (slot, record) in state
        .globals
        .monsters
        .slots
        .iter_mut()
        .zip(support::monsters(&case["npc_before"]))
    {
        *slot = Some(centurions_core::combat::globals::MonsterSlot {
            record,
            position: Some((0, 0)),
        });
    }
    support::saved_command(Command::Continue, &mut state, &content);
    assert_eq!(
        support::record_bytes(&state.warriors),
        case.bytes("party_after")
    );
    let records: Vec<_> = state
        .globals
        .monsters
        .slots
        .iter()
        .flat_map(|slot| slot.as_ref().unwrap().record.bytes.iter().copied())
        .collect();
    assert_eq!(records, case.bytes("npc_after"));
    assert_eq!(state.holdover, case.bytes("allies_after"));
    assert_eq!(
        state
            .globals
            .health_context
            .kill_party,
        case.n::<i8>("unavailable_after")
    );
    assert_eq!(
        state
            .globals
            .battlefield
            .spell_queue,
        case.bytes("fields_after")
    );
    assert_eq!(
        state.globals.treasure,
        case.bytes("treasure_after")
    );
    assert_eq!(state.flags[..100], case.bytes("flags_after"));
    support::expect(state.party_currency, &case["currency_after"]);
    support::assert_rng(&state.random, &case["seed_after"], &case["draws_after"]);
    assert_eq!(
        state.globals.battle_rewards_suppressed,
        case.n::<i8>("suppression_after")
    );
    assert!(state.combat.is_none() && state.ordinary_battle_rewards.is_none());
    assert!(state.exploration.as_ref().unwrap().script.is_none());
}
