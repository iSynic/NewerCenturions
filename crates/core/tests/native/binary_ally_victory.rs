use crate::{prelude::*, support};

use centurions_core::combat::allies::synchronize;
use centurions_core::combat::runtime::victory_loot;

#[test]
fn survivor_cleanup_and_rewards_match_original_composite_execution() {
    let content = support::content();
    let fixture: serde_json::Value = support::fixture("ally-victory.json");
    for case in fixture.array("cases") {
        let mut warriors: Vec<_> = support::warriors(&case["warriors_before"]);
        let count = case.n::<usize>("count");
        let mut enemies: Vec<_> = support::monsters(&case.bytes("enemies_before"))
            .into_iter()
            .take(count)
            .collect();
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let result = synchronize(
            &case.bytes("menu_before"),
            case.n::<usize>("ally_count"),
            &mut enemies,
            &mut random,
        )
        .unwrap();
        support::native_expect! {
            bytes result.menu => &case["menu_after"], "binary_ally_victory";
            scalar result.count => &case["ally_count_after"], "binary_ally_victory";
        }
        let (share, loot) =
            victory_loot(&mut warriors, &mut enemies, &mut random, Some(&content)).unwrap();
        support::native_expect! {
            scalar share => &case["share"], "binary_ally_victory";
            records &warriors => &case["warriors_after"], "binary_ally_victory";
        }
        assert_eq!(
            support::record_bytes(&enemies),
            case.bytes("enemies_after")[..count * 288]
        );
        assert_eq!(
            loot.iter().flat_map(|i| i.0).collect::<Vec<_>>(),
            case.bytes("loot_after")[..loot.len() * 22]
        );
        support::assert_rng(&random, &case["seed_after"], &case["rng_draws"]);
    }
}

#[test]
fn survivor_controls_match_native_capacity_warnings_copyback_and_saved_commands() {
    let content = content();
    for case in support::fixture("survivor-selection.json").array("cases") {
        let mut state = support::mission_session(&content);
        state.phase = Phase::Combat;
        state.random = RandomState::new(12);
        for warrior in &mut state.warriors {
            warrior.bytes[0x9df] = 1;
        }
        let mut arena = support::battle();
        arena.enemies.records = support::monsters(&case.bytes("records_before"));
        arena.enemies.positions = (0..7).map(|i| (40 + i, 40)).collect();
        arena.squad.positions = (0..6).map(|i| (20 + i, 20)).collect();
        arena.order[1] = 0;
        state.combat = Some(arena);
        let position = state.exploration.as_mut().unwrap();
        position.presentation = None;
        position.script = None;
        let mut events = apply(Command::EndTurn, &mut state, &content).unwrap();
        let mut resumed = support::checkpoint(&state, &content, "survivors");
        for step in case.array("steps") {
            let selection = state.survivors.as_ref().unwrap();
            let native = words(&step.bytes("rows"));
            let rows: Vec<_> = selection
                .eligible
                .iter()
                .zip(&selection.selected)
                .map(|(index, selected)| {
                    if *selected {
                        -(*index as i16 + 1)
                    } else {
                        *index as i16 + 1
                    }
                })
                .collect();
            assert_eq!(rows, native[..rows.len()]);
            assert_eq!(selection.limit, step.n::<usize>("limit"));
            support::assert_rng(&state.random, &step["seed"], &step["draws"]);
            let control = step.n::<usize>("control");
            if control == 2 {
                break;
            }
            let command = Command::SelectSurvivor { row: control - 3 };
            let changed = replay_command(command, &mut state, &mut resumed, &content);
            events.extend(changed);
            resumed = support::checkpoint(&state, &content, "survivors");
        }
        let observed: Vec<Value> = events
            .iter()
            .filter_map(|event| match event {
                Event::Sound { resource_id } => Some(serde_json::json!(["sound", resource_id])),
                Event::Feedback(centurions_core::feedback::NativeFeedback::DialogText {
                    item,
                    address,
                    ..
                }) => Some(serde_json::json!(["text", item, address])),
                Event::Delay { ticks } => Some(serde_json::json!(["delay", ticks])),
                _ => None,
            })
            .collect();
        let expected: Vec<_> = case
            .array("events")
            .iter()
            .filter(|event| event[0] != "text" || event[2] != 0x404000)
            .cloned()
            .collect();
        assert_eq!(observed, expected);
        let mut records = current_combat(&state).enemies.records.clone();
        let copied = centurions_core::combat::allies::commit_survivors(
            &state.allies,
            state.ally_count,
            &mut records,
            state.survivors.as_ref(),
        )
        .unwrap();
        assert_eq!(support::record_bytes(&records), case.bytes("records_after"));
        assert_eq!(copied.menu, case.bytes("menu_after"));
        replay_command(Command::Continue, &mut state, &mut resumed, &content);
        assert_eq!(state.allies, copied.menu);
        assert_eq!(state.ally_count, case.n::<usize>("count_after"));
        assert!(state.survivors.is_none());
    }
}
