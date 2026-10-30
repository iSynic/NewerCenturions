//! Original positive-count booty callers; combat and loot Close are controlled services.
use crate::{prelude::*, support};
use centurions_core::combat::globals::{Monsters, MonsterSlot};
use centurions_core::combat::round_stories::{RoundBattleReturn, RoundDispatch};
use centurions_core::combat::battlefield::Battlefield;

#[test]
fn retained_npc_cleanup_and_reward_returns_match_native() {
    for case in support::fixture("retained-rewards.json").array("cases") {
        let content = content();
        let mut state = support::mission_session(&content);
        let records = support::monsters(&case["npc_before"]);
        state.globals.monsters = Monsters {
            num_mon: case.n("npc_count_before"),
            slots: records.into_iter().map(|record| Some(MonsterSlot { record, position: Some((7, 9)) })).collect(),
            origin: (0, 0),
        };
        state.globals.battlefield = Battlefield {
            field: CombatField { tiles: vec![0; 8100], edge_word: 0, map_shift: (0, 0) },
            saved_land: None, pos: [(0, 0); 6], char_under: [0; 6],
            spell_queue: case.bytes("fields_before"), collide_check: vec![0; 128],
            battle: None,
        };
        state.warriors = support::warriors(&case["party_before"]);
        state.holdover = case.bytes("allies_before");
        state.held_over = 1;
        state.suspend_allies = case.n("suppress_allies");
        state.party_currency = 37;
        state.random = RandomState::new(12);
        state.flags.fill(0);
        state.globals.treasure = case.bytes("treasure_before");
        state.globals.extra_record = case.bytes("extra");
        state.globals.kill_counts.num_enemy = 2;
        state.globals.kill_counts.kill_mon = 1;
        if case.text("kind") == "ordinary" {
            // A saved child Close joins the independently captured outer newland2 prefix.
            support::pause_script(&mut state, case.bytes("outer"), 1);
            let suspended_script = state.exploration.as_mut().unwrap().script.take();
            support::pause_script(&mut state, vec![0; 42], 0);
            state.exploration.as_mut().unwrap().presentation = Some(Presentation::Loot);
            state.ordinary_battle_rewards = Some(serde_json::from_value(serde_json::json!({"mode":0,"loot":true})).unwrap());
            state.round_dispatch = Some(RoundDispatch {
                suspended_script, child_battle: true, death_depth: Some(0),
                outer_battle: Some(RoundBattleReturn {
                    after_message: 0, treasure: 0,
                    rewards: Some(serde_json::from_value(serde_json::json!({})).unwrap()),
                    selected: None,
                    group: None, npc: None,
                }),
                ..Default::default()
            });
        } else {
            support::pause_script(&mut state, case.bytes("inner"), 0);
        }
        let mut events = support::saved_command(Command::Continue, &mut state, &content);
        if case.n::<i16>("windows") != 0 {
            compare(&state, case, "prepared");
            assert_eq!(support::record_bytes(&state.booty_list), case.bytes("loot_prepared"));
            events.extend(support::saved_command(Command::Continue, &mut state, &content));
        }
        compare(&state, case, "after");
        assert_eq!(state.globals.treasure, case.bytes("treasure_after"));
        assert!(state.globals.monsters.slots.iter().flatten().all(|slot| slot.position == Some((7, 9))));
        assert!(state.combat.is_none() && state.survivors.is_none());
        assert!(state.exploration.as_ref().unwrap().script.is_none());
        assert!(state.exploration.as_ref().unwrap().presentation.is_none());
        let sounds: Vec<_> = events.iter().filter_map(|event| match event {
            Event::Sound { resource_id } => Some(*resource_id), _ => None,
        }).collect();
        let expected: Vec<i16> = case.array("calls").iter().filter_map(|call| {
            call.as_str().unwrap().strip_prefix("sound:").map(|id| id.parse().unwrap())
        }).collect();
        assert_eq!(sounds, expected);
    }
}

fn compare(state: &Session, case: &Value, suffix: &str) {
    assert_eq!(support::record_bytes(&state.warriors), case.bytes(&format!("party_{suffix}")));
    let npcs: Vec<_> = state.globals.monsters.slots.iter().flat_map(|slot| slot.as_ref().unwrap().record.bytes.iter().copied()).collect();
    assert_eq!(npcs, case.bytes(&format!("npc_{suffix}")));
    assert_eq!(state.holdover, case.bytes(&format!("allies_{suffix}")));
    assert_eq!(state.globals.battlefield.spell_queue, case.bytes(&format!("fields_{suffix}")));
    assert_eq!(state.flags[..100], case.bytes(&format!("flags_{suffix}")));
    support::expect(state.party_currency, &case[format!("currency_{suffix}")]);
    assert_eq!(state.globals.monsters.num_mon, case.n::<i16>(&format!("npc_count_{suffix}")));
    support::expect(state.globals.health_context.kill_party, &case[format!("unavailable_{suffix}")]);
    support::assert_rng(&state.random, &case[format!("seed_{suffix}")], &case[format!("draws_{suffix}")]);
    assert_eq!(state.globals.battle_rewards_suppressed, case.n::<i8>(&format!("suppression_{suffix}")));
}
