use crate::prelude::*;

fn input(fixture: &Value, case: &Value) -> Session {
    let mut state: Session = serde_json::from_value(
        fixture["foundations"][case.n::<usize>("foundation")]["session"].clone(),
    )
    .unwrap();
    if case["kind"] == "normalized_diagonal_attempt" {
        // Native display normalization reaches Getup's full item refresh before this command.
        state.ammunition_work.item = Some(case["before_command"].bytes("item_work"));
    }
    state
}
fn project(state: &mut Session, snapshot: &Value, baseline: &Value) {
    crate::snapshots::earned_projection(state, snapshot, baseline);
    if snapshot["seed"] != baseline["seed"] || snapshot["ammunition"] != baseline["ammunition"] {
        state.ammunition_work.item = Some(snapshot.bytes("item_work"));
        state.ammunition_work.weapon = Some(snapshot.bytes("weapon_work"));
        state.ammunition_work.ammo = Some(snapshot.bytes("ammo_work"));
    }
}

use crate::support::same_session as same;

use crate::support::checkpoint;

fn replay_historical_prefix(fixture: &Value, case: &Value, content: &Content) -> Option<Session> {
    let index = case.n::<usize>("foundation");
    let receipt = crate::native_common::earned_receipt("earned-combat-reload", index);
    let commands = receipt.array("commands");
    let ordered_events = receipt.array("ordered_events");
    assert_eq!(ordered_events.len(), commands.len());
    let mut state: Session = serde_json::from_value(receipt["starting_session"].clone()).unwrap();
    state.validate(content).unwrap();
    state = checkpoint(&state, content, &format!("prefix-start-{index}"));
    for (step, (command, recorded)) in commands.iter().zip(ordered_events).enumerate() {
        let command: Command = serde_json::from_value(command.clone()).unwrap();
        let expected = crate::native_common::shield_result_events(
            serde_json::from_value(recorded.clone()).unwrap(),
        );
        let before = state.clone();
        let result = apply(command.clone(), &mut state, content);
        if let Ok(events) = &result
            && events.iter().any(|e| {
                matches!(
                    e,
                    Event::Feedback(centurions_core::feedback::NativeFeedback::Combat { .. })
                )
            })
        {
            assert_eq!((index, step), (1, 149));
            assert!(matches!(&command, Command::ShootEnemy { body: 15 }));
            let feedback = vec![centurions_core::feedback::NativeFeedback::Combat {
                kind: centurions_core::feedback::CombatRefusal::OutOfRange,
            }];
            crate::support::refuse_command(&before, content, command.clone(), feedback);
            assert_eq!(state.warriors, before.warriors);
            assert_eq!(state.random, before.random);
            let combat = state.combat.as_ref().unwrap();
            let start = combat.squad.positions[1];
            let end = combat.enemies.positions[5];
            let dx = f32::from(start.0 - end.0);
            let dy = f32::from(start.1 - end.1);
            assert_eq!((dx * dx + dy * dy).sqrt().trunc() as i16, 9);
            assert_eq!(word(&state.warriors[1].bytes, 0xe4).unwrap(), 1);
            let mission = content
                .map(state.exploration.as_ref().unwrap().map)
                .unwrap();
            let prepared = firearm::prepare(
                &state.warriors[1],
                &content.items[1],
                &content.items[200],
                &mission.combat.as_ref().unwrap().attacks,
            )
            .unwrap();
            assert_eq!(
                centurions_core::combat::player_psi::range(&prepared.packet, prepared.shots)
                    .unwrap(),
                8
            );
            return None;
        }
        let events = result.unwrap();
        assert_eq!(events, expected, "foundation {index} command {step}");
        if state.medical_continuation.is_some() {
            state = checkpoint(&state, content, &format!("prefix-medical-{index}"));
        }
    }
    let mut expected = input(fixture, case);
    // This earned battle is still inside timeclick's regional newland call.
    expected.movement_continuation =
        Some(centurions_core::MovementContinuation::RegionalReturn { header: false });
    // The recorded prefix defeats body16; native killbody retains that body.
    expected.health_context.as_mut().unwrap().last_body = 16;
    // The seed12 receipt's last full pair writer is its real Shoot at command145.
    let mut weapon = content.items[165].clone();
    let ammo = content.items[280].clone();
    for offset in [12, 14] {
        let value = word(&ammo, offset).unwrap();
        if value != 0 {
            weapon[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
        }
    }
    expected.ammunition_work.weapon = Some(weapon);
    expected.ammunition_work.ammo = Some(ammo);
    expected.inventory_record = crate::native_common::earned_inventory_work(index);
    expected.combat.as_mut().unwrap().turn_origin =
        crate::snapshots::recorded_turn_origin(&receipt, expected.combat.as_ref().unwrap());
    same(&state, &expected);
    Some(state)
}

#[test]
fn historical_extra_cell_shot_rejects_atomically_and_valid_prefix_still_replays() {
    let content = crate::support::content();
    let fixture: Value = crate::support::fixture("earned-combat-reload.json");
    for case in fixture
        .array("cases")
        .iter()
        .filter(|case| case["kind"] == "normalized_diagonal_attempt")
    {
        let result = replay_historical_prefix(&fixture, case, &content);
        assert_eq!(result.is_some(), case["foundation"] == 0);
    }
}

#[test]
fn historical_occupied_cell_refill_matches_complete_native_caller_and_save_resume() {
    let content = crate::support::content();
    let fixture: Value = crate::support::fixture("earned-combat-reload.json");
    for (index, case) in fixture
        .array("cases")
        .iter()
        .filter(|case| case["kind"] == "normalized_diagonal_attempt")
        .enumerate()
    {
        let mut state = input(&fixture, case);
        let initial = state.clone();
        let mut before = initial.clone();
        project(&mut before, &case["before"], &case["before"]);
        same(&initial, &before);
        let mut resumed = checkpoint(&state, &content, &format!("refill-before-{index}"));
        let command = Command::Move { dx: 1, dy: 1 };
        let events = replay_command(command.clone(), &mut state, &mut resumed, &content);
        let mut expected = initial.clone();
        project(&mut expected, &case["after"], &case["before"]);
        same(&state, &expected);
        assert!(events.is_empty());

        checkpoint(&state, &content, &format!("refill-after-{index}"));
    }
}

#[test]
fn last_stock_control_matches_complete_native_caller_and_save_resume() {
    let content = crate::support::content();
    let original_fixture: Value = crate::support::fixture("earned-combat-reload.json");
    let bounds: Value = crate::support::fixture("earned-combat-reload-bounds.json");
    let case = bounds
        .array("cases")
        .iter()
        .find(|case| case["id"] == "normalized_seed12_last_stock_control")
        .unwrap();
    let original_case = original_fixture
        .array("cases")
        .iter()
        .find(|case| case["foundation"] == 0 && case["kind"] == "normalized_diagonal_attempt")
        .unwrap();
    let foundation = input(&bounds, case);
    same(&foundation, &input(&original_fixture, original_case));
    assert_eq!(foundation.ammunition[80], 3);

    let mut state = foundation;
    state.ammunition[80] = 1;
    let initial = state.clone();
    let mut before = initial.clone();
    project(&mut before, &case["before"], &case["before"]);
    same(&initial, &before);
    let mut resumed = checkpoint(&state, &content, "last-stock-before");
    let command = Command::Move { dx: 1, dy: 1 };
    let events = replay_command(command, &mut state, &mut resumed, &content);
    let mut expected = initial.clone();
    project(&mut expected, &case["after"], &case["before"]);
    same(&state, &expected);
    assert!(events.is_empty());

    checkpoint(&state, &content, "last-stock-after");
    replay_command(
        Command::Move { dx: 1, dy: 1 },
        &mut state,
        &mut resumed,
        &content,
    );
    assert_eq!(
        word(&state.warriors[4].inventory().unwrap()[0].0, 2).unwrap(),
        0
    );
}

#[test]
fn reserve_two_uses_the_source_supported_full_load_interval() {
    let content = crate::support::content();
    let fixture: Value = crate::support::fixture("earned-combat-reload.json");
    for case in fixture
        .array("cases")
        .iter()
        .filter(|case| case["kind"] == "normalized_diagonal_attempt")
    {
        let mut state = input(&fixture, case);
        state.ammunition[80] = 2;
        let initial = state.clone();
        let mut resumed = checkpoint(&state, &content, "stock-two-before");
        let command = Command::Move { dx: 1, dy: 1 };
        let events = replay_command(command, &mut state, &mut resumed, &content);
        let mut expected = initial.clone();
        project(&mut expected, &case["after"], &case["before"]);
        expected.ammunition[80] = 1;
        same(&state, &expected);
        assert!(events.is_empty());
        checkpoint(&state, &content, "stock-two-after");
    }
}
