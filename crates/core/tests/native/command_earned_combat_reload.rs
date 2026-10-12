use crate::prelude::*;

fn fixture() -> (Content, Value) {
    (
        crate::support::content(),
        crate::support::fixture("earned-combat-reload.json"),
    )
}
use crate::native_common::restore_party as party;

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

use crate::support::reject_command as atomic;
use crate::support::same_session as same;

use crate::support::checkpoint;
use crate::support::event_sounds as sounds;

fn receipt(index: usize) -> Value {
    let receipts = [
        include_str!("../../../../evidence/fixtures/earned-combat-reload-receipt-12.json"),
        include_str!("../../../../evidence/fixtures/earned-combat-reload-receipt-1.json"),
    ];
    serde_json::from_str(receipts[index]).unwrap()
}

fn replay_historical_prefix(fixture: &Value, case: &Value, content: &Content) -> Option<Session> {
    let index = case.n::<usize>("foundation");
    let receipt = receipt(index);
    assert_eq!(
        receipt["complete_state_and_every_ordered_event_replay"],
        true
    );
    assert_eq!(receipt["full_input_save_reload"], true);
    assert_eq!(receipt["field_grants_or_random_trials"], false);
    let commands = receipt.array("commands");
    let ordered_events = receipt.array("ordered_events");
    assert_eq!(commands.len(), if index == 0 { 148 } else { 168 });
    assert_eq!(ordered_events.len(), commands.len());
    let mut state: Session = serde_json::from_value(receipt["starting_session"].clone()).unwrap();
    state.validate(content).unwrap();
    let mut resumed = checkpoint(&state, content, &format!("prefix-start-{index}"));
    for (step, (command, recorded)) in commands.iter().zip(ordered_events).enumerate() {
        let command: Command = serde_json::from_value(command.clone()).unwrap();
        let expected: Vec<Event> = serde_json::from_value(recorded.clone()).unwrap();
        let before = state.clone();
        let result = apply(command.clone(), &mut state, content);
        if let Err(error) = result {
            assert_eq!(index, 1);
            assert_eq!(step, 149);
            assert!(matches!(&command, Command::ShootEnemy { body: 15 }));
            assert_eq!(
                error.to_string(),
                "invalid session: enemy is outside firearm range or line of sight"
            );
            same(&state, &before);
            same(&resumed, &before);
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
            assert_eq!(
                apply(command, &mut resumed, content)
                    .unwrap_err()
                    .to_string(),
                error.to_string()
            );
            same(&resumed, &before);
            return None;
        }
        let events = result.unwrap();
        assert_eq!(events, expected, "foundation {index} command {step}");
        assert_eq!(
            events,
            apply(command, &mut resumed, content).unwrap(),
            "resumed foundation {index} command {step}"
        );
        same(&state, &resumed);
        assert_eq!(state.random, resumed.random);
        resumed = checkpoint(&state, content, &format!("prefix-{index}-{step}"));
    }
    let mut expected = input(fixture, case);
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
    same(&state, &expected);
    same(&resumed, &state);
    let effect = state.health_context.as_ref().unwrap().effect;
    assert_eq!(state.attack_packet.as_ref().unwrap()[18] as i8, effect);
    assert_eq!(
        state
            .combat
            .as_ref()
            .unwrap()
            .attack_packet
            .as_ref()
            .unwrap()[18] as i8,
        effect
    );
    Some(state)
}

#[test]
fn historical_extra_cell_shot_rejects_atomically_and_valid_prefix_still_replays() {
    let (content, fixture) = fixture();
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
fn silent_first_refit_matches_native_warrior_stock_and_cache_scalars() {
    let (content, fixture) = fixture();
    for case in fixture
        .array("cases")
        .iter()
        .filter(|case| case["kind"] == "normalized_diagonal_attempt")
    {
        let mut state = input(&fixture, case);
        party(&mut state, &case["before_command"]);
        let original = state.clone();
        let result =
            firearm::refit_earned_secondary(&state.warriors[4], &content, state.ammunition[80])
                .unwrap();
        same(&state, &original);
        state.warriors[4] = result.warrior;
        state.ammunition[80] = result.reserve;
        let mut expected = original;
        party(&mut expected, &case["after"]);
        expected.ammunition = crate::support::longs(&case["after"].bytes("ammunition"));
        same(&state, &expected);
        assert_eq!(result.loaded_item_field32, 16);
        assert_eq!(result.weapon_work, WeaponWork::zero());
        assert_eq!(result.reserve, 2);
        assert_eq!(
            word(&state.warriors[4].inventory().unwrap()[0].0, 2).unwrap(),
            1
        );
    }
}

#[test]
fn historical_occupied_cell_refill_matches_complete_native_caller_and_save_resume() {
    let (content, fixture) = fixture();
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
        let events = apply(command.clone(), &mut state, &content).unwrap();
        assert_eq!(
            events,
            apply(command.clone(), &mut resumed, &content).unwrap()
        );
        same(&state, &resumed);
        let mut expected = initial.clone();
        project(&mut expected, &case["after"], &case["before"]);
        same(&state, &expected);
        assert!(events.is_empty());
        assert!(sounds(&events).is_empty());

        assert_eq!(state.random, initial.random);
        assert_eq!(
            state.combat.as_ref().unwrap().attack_packet,
            initial.combat.as_ref().unwrap().attack_packet
        );
        assert_eq!(
            state.combat.as_ref().unwrap().attack_identity,
            initial.combat.as_ref().unwrap().attack_identity
        );
        assert_eq!(state.attack_count, 0);
        assert_eq!(state.loaded_item_field32, Some(16));
        assert_eq!(state.ammunition[80], 2);
        let restored = checkpoint(&state, &content, &format!("refill-after-{index}"));
        same(&state, &restored);
        atomic(&state, &content, command);
        atomic(&state, &content, Command::Reload { warrior: 4 });
    }
}

#[test]
fn last_stock_control_matches_complete_native_caller_and_save_resume() {
    let (content, original_fixture) = fixture();
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
    let events = apply(command.clone(), &mut state, &content).unwrap();
    assert_eq!(events, apply(command, &mut resumed, &content).unwrap());
    same(&state, &resumed);
    let mut expected = initial.clone();
    project(&mut expected, &case["after"], &case["before"]);
    same(&state, &expected);
    assert!(events.is_empty());

    assert_eq!(state.random, initial.random);
    assert_eq!(state.ammunition[80], 0);
    assert_eq!(
        word(&state.warriors[4].inventory().unwrap()[0].0, 2).unwrap(),
        1
    );
    assert_eq!(state.attack_count, 0);
    assert_eq!(state.loaded_item_field32, Some(16));
    assert_eq!(state.weapon_work, WeaponWork::zero());
    assert_eq!(state.attack_packet, initial.attack_packet);
    assert_eq!(state.attack_identity, initial.attack_identity);
    for (key, id) in [("item_work", 165), ("weapon_work", 165), ("ammo_work", 280)] {
        assert_eq!(bytes(&case["after"][key]), *content.items.get(id).unwrap());
    }
    let restored = checkpoint(&state, &content, "last-stock-after");
    same(&state, &restored);
    atomic(&state, &content, Command::Move { dx: 1, dy: 1 });
    atomic(&state, &content, Command::Reload { warrior: 4 });
}

#[test]
fn reserve_two_uses_the_source_supported_full_load_interval() {
    let (content, fixture) = fixture();
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
        let events = apply(command.clone(), &mut state, &content).unwrap();
        assert_eq!(events, apply(command, &mut resumed, &content).unwrap());
        same(&state, &resumed);
        let mut expected = initial.clone();
        project(&mut expected, &case["after"], &case["before"]);
        expected.ammunition[80] = 1;
        same(&state, &expected);
        assert!(events.is_empty());
        assert_eq!(state.random, initial.random);
        let restored = checkpoint(&state, &content, "stock-two-after");
        same(&state, &restored);
    }
}
