//! Native booty10 and detached booty4; combat, modal and service boundaries are controlled.
use crate::{prelude::*, support};
use centurions_core::combat::retained::{NpcBacking, NpcSlot};
use centurions_core::combat::round_stories::{RoundDispatch, RoundProgram};
use centurions_core::combat::tactical_work::TacticalWork;

#[test]
fn storage_and_zero_roster_script_returns_match_native_records_and_work() {
    for case in support::fixture("storage-recovery.json").array("cases") {
        let mut content = content();
        let tables = Arc::make_mut(&mut content.scenario.as_mut().unwrap().maps[0].shared);
        tables.door_scripts.resize(901 * 42, 0);
        tables.door_scripts[900 * 42..901 * 42].copy_from_slice(&case.bytes("inner"));
        let kind = case.text("kind");
        let mut state = support::mission_session(&content);
        let records = support::monsters(&case["npc_before"]);
        let backing = NpcBacking {
            slots: records
                .iter()
                .map(|record| {
                    Some(NpcSlot {
                        record: record.clone(),
                        position: Some((0, 0)),
                    })
                })
                .collect(),
            active_count: case.n("npc_count_before"),
            origin: (0, 0),
        };
        if kind != "detached" {
            support::pause_script(&mut state, case.bytes("outer"), 0);
            support::saved_command(Command::Continue, &mut state, &content);
            let arena = state.combat.as_mut().unwrap();
            arena.enemies.records = records[..1].to_vec();
            arena.enemies.positions = vec![(40, 40)];
            arena.retained_npcs = backing.clone();
            arena.spell_fields = case.bytes("fields_before");
            arena.order.fill(-1);
            arena.cursor = 0;
            arena.turn_origin = None;
            arena.script_enemy = 0;
            arena.selected_enemy = 0;
            arena.victory_work.total = 2;
            arena.victory_work.defeated = if kind == "ordinary" { 2 } else { 1 };
            if kind == "scripted" {
                arena.cursor = 110;
                arena.round_scripts.slots = [900, 0, 0, 0, 0];
                arena.round_scripts.pending = Some(RoundProgram {
                    slot: 0,
                    script: Some(900),
                    cursor: 1,
                    text: None,
                    delegated: true,
                });
                state.round_dispatch = Some(RoundDispatch {
                    suspended_script: state.exploration.as_mut().unwrap().script.take(),
                    ..Default::default()
                });
                support::pause_script(&mut state, case.bytes("inner"), 0);
            }
        } else {
            state.native_work.tactical_work = TacticalWork {
                map: TacticalMap {
                    tiles: vec![0; 8100],
                    edge_word: 0,
                    shift: (0, 0),
                },
                saved_land: None,
                positions: [(0, 0); 6],
                under_tiles: [0; 6],
                spell_fields: case.bytes("fields_before"),
                field_used: vec![0; 128],
                battle: None,
            };
            state.native_work.victory_work.total = 2;
            state.native_work.victory_work.defeated = 1;
            support::pause_script(&mut state, case.bytes("inner"), 0);
        }
        state.native_work.npc_backing = backing;
        state.warriors = support::warriors(&case["party_before"]);
        state.random = RandomState::new(12);
        state.suppress_allies = 1;
        state.party_currency = 37;
        state.flags.fill(0);
        state.native_work.recovery_work.displaced_items = scratch_ids(&case["scratch_before"]);
        state.native_work.recovery_work.held_money = case.n("held_before");
        state.native_work.extra_record = case.bytes("extra");
        state.native_work.treasure_record = case.bytes("treasure_before");
        let command = if kind == "ordinary" {
            Command::EndTurn
        } else {
            Command::Continue
        };
        let mut events = support::saved_command(command, &mut state, &content);
        compare(&state, case, "prepared");
        assert_eq!(
            support::record_bytes(&state.booty_items),
            case.bytes("loot_prepared")
        );
        assert_eq!(
            state.exploration.as_ref().unwrap().presentation,
            Some(Presentation::Loot)
        );
        // Collection/promotion are separate services in this probe.
        state.booty_items.fill(InventoryRecord([0; 22]));
        events.extend(support::saved_command(
            Command::Continue,
            &mut state,
            &content,
        ));
        compare(&state, case, "after");
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
        assert_eq!(
            state.native_work.recovery_work.displaced_items,
            scratch_ids(&case["scratch_after"])
        );
        assert_eq!(
            state.native_work.treasure_record,
            case.bytes("treasure_after")
        );
        assert!(state.recovery_return.is_none() && state.ordinary_battle_rewards.is_none());
        assert!(state.combat.is_none() && state.exploration.as_ref().unwrap().script.is_none());
    }
}

fn scratch_ids(value: &Value) -> Vec<i16> {
    support::bytes(value)
        .as_chunks::<22>()
        .0
        .iter()
        .map(|record| InventoryRecord(*record).id())
        .collect()
}

fn compare(state: &Session, case: &Value, suffix: &str) {
    assert_eq!(
        support::record_bytes(&state.warriors),
        case.bytes(&format!("party_{suffix}"))
    );
    let npcs: Vec<u8> = state
        .native_work
        .npc_backing
        .slots
        .iter()
        .flat_map(|slot| slot.as_ref().unwrap().record.bytes.iter().copied())
        .collect();
    assert_eq!(npcs, case.bytes(&format!("npc_{suffix}")));
    assert_eq!(
        state
            .native_work
            .tactical_work
            .spell_fields,
        case.bytes(&format!("fields_{suffix}"))
    );
    assert_eq!(state.flags[..100], case.bytes(&format!("flags_{suffix}")));
    support::expect(state.party_currency, &case[format!("currency_{suffix}")]);
    assert_eq!(
        state.native_work.recovery_work.held_money,
        case.n::<i32>(&format!("held_{suffix}"))
    );
    assert_eq!(
        state.native_work.npc_backing.active_count,
        case.n::<i16>(&format!("npc_count_{suffix}"))
    );
    support::expect(
        state
            .native_work
            .health_context
            .unavailable,
        &case[format!("unavailable_{suffix}")],
    );
    support::assert_rng(
        &state.random,
        &case[format!("seed_{suffix}")],
        &case[format!("draws_{suffix}")],
    );
    assert_eq!(
        state.native_work.battle_rewards_suppressed,
        case.n::<i8>(&format!("suppression_{suffix}"))
    );
}
