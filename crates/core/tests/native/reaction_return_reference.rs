use super::*;
#[path = "combat_encounter_item_reference.rs"]
mod combat_encounter_items;
#[path = "detached_attack_reference.rs"]
mod detached_attacks;
#[path = "detached_warrior_reference.rs"]
mod detached_warriors;
#[path = "field_terminal_reference.rs"]
mod field_terminal;
#[path = "getup_booty_reference.rs"]
mod getup_booty;
#[path = "occupied_walking_reference.rs"]
mod occupied_walking;
#[path = "outdoor_allied_reference.rs"]
mod outdoor_allied;
#[path = "outdoor_death_reference.rs"]
mod outdoor_deaths;
#[path = "outdoor_item_reference.rs"]
mod outdoor_items;
#[path = "selected_reward_reference.rs"]
mod selected_rewards;
#[path = "shop_work_reference.rs"]
mod shop_work;
use crate::Session;
use crate::test_support::{bytes, fixture};
use centurions_content::combat::MonsterRecord;
use serde_json::Value;

fn input(case: &Value) -> Session {
    let mut state: Session =
        serde_json::from_value(fixture("player-psi.json")["route_foundation"].clone()).unwrap();
    state.warriors = bytes(&case["warriors_before"])
        .as_chunks::<3138>()
        .0
        .iter()
        .map(|bytes| Warrior::read(bytes).unwrap())
        .collect();
    let combat = state.combat.as_mut().unwrap();
    combat.enemies.records = bytes(&case["records_before"])
        .as_chunks::<288>()
        .0
        .iter()
        .map(|bytes| MonsterRecord {
            bytes: bytes.to_vec(),
        })
        .collect();
    combat.enemies.positions = case["enemy_positions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|point| {
            (
                point[0].as_i64().unwrap() as i16,
                point[1].as_i64().unwrap() as i16,
            )
        })
        .collect();
    combat.map.tiles = bytes(&case["map_before"])
        .as_chunks::<2>()
        .0
        .iter()
        .map(|bytes| i16::from_be_bytes(*bytes))
        .collect();
    combat.map.shift = (0, 0);
    combat.squad.positions = (0..6)
        .map(|member| {
            if member == 0 {
                (20, 20)
            } else {
                (40 + member, 40)
            }
        })
        .collect();
    combat.cursor = 1;
    combat.order[1] = 0;
    // A declared admission control; these receipts omit the literal pair.
    combat.victory_work.total = 1;
    combat.victory_work.defeated = 0;
    state.random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
    state
}

#[test]
fn native_forced_attack_return_interrupts_with_the_moving_warrior_still_alive() {
    let fixture = fixture("warrior-opportunity.json");
    for case in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["forced_kill_return"] == true)
    {
        let mut state = input(case);
        let combat = state.combat.as_mut().unwrap();
        let before = state.random.clone();
        let mut mask = [0; 6];
        let mut native = NativeNeighbors {
            selected_warrior: 0,
            party_mask: &mut mask,
            aliases: Default::default(),
            table: None,
        };
        crate::combat::neighbor_work::begin(combat);
        crate::combat::neighbor_work::scan(combat, &state.warriors, 1, (0, 0), &mut native)
            .unwrap();
        let list = crate::combat::neighbor_work::scan(
            combat,
            &state.warriors,
            2,
            (
                case["dx"].as_i64().unwrap() as i8,
                case["dy"].as_i64().unwrap() as i8,
            ),
            &mut native,
        )
        .unwrap();
        let mut buffers = Default::default();
        let mut context = ReactionContext {
            items: None,
            attacks: None,
            buffs: &[0; 20],
            buffers: &mut buffers,
            display_writers: false,
            native: Some(native),
        };
        let mut calls = vec![];
        let mut changed = false;
        // The original receipt hooks attack2 to return one without killing a body.
        let result = guards_with(
            combat,
            &mut state.warriors,
            &mut state.random,
            0,
            true,
            &list,
            &mut changed,
            &mut context,
            &mut |_, _, _, call, _| {
                calls.push(serde_json::json!([
                    call.attacker,
                    call.recipient,
                    call.index,
                    2
                ]));
                Ok(attacks::AttackOutcome {
                    events: vec![],
                    returned: true,
                    terminated: false,
                })
            },
        )
        .unwrap();
        assert_eq!(
            calls,
            case["calls"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|call| call[3] == 2)
                .cloned()
                .collect::<Vec<_>>()
        );
        assert!(result.interrupted && !changed);
        assert!(state.warriors[0].stamina().unwrap() > 0);
        assert_eq!(state.random, before);
    }
}

#[test]
fn source_callback_list_changes_do_not_replace_the_callers_local_count() {
    let fixture = fixture("warrior-opportunity.json");
    let mut state = input(&fixture["cases"][0]);
    let combat = state.combat.as_mut().unwrap();
    for record in &mut combat.enemies.records {
        record.bytes[0xb3] = 0;
    }
    combat.neighbor_work.bodies[..3].copy_from_slice(&[10, 11, 12]);
    let mut mask = [0; 6];
    let mut buffers = Default::default();
    let mut context = ReactionContext {
        items: None,
        attacks: None,
        buffs: &[0; 20],
        buffers: &mut buffers,
        display_writers: false,
        native: Some(NativeNeighbors {
            selected_warrior: 0,
            party_mask: &mut mask,
            aliases: Default::default(),
            table: None,
        }),
    };
    let mut calls = vec![];
    let mut frame = GuardFrame::new(0, true, 2, false).unwrap();
    let returned = loop {
        match frame
            .advance(combat, &mut state.warriors, &[], &mut context)
            .unwrap()
        {
            GuardStep::Attack(call) => {
                calls.push(call.attacker);
                combat.neighbor_work.bodies[1] = 12;
                frame.finish_attack(false).unwrap();
                // A paused callback saves locals, while the next WORD stays shared.
                frame = serde_json::from_slice(&serde_json::to_vec(&frame).unwrap()).unwrap();
                assert!(frame.returned_attack());
            }
            GuardStep::Done(returned) => break returned,
            GuardStep::Refresh(_) => unreachable!(),
        }
    };
    assert_eq!(calls, [10, 12]);
    assert!(!returned);
}
