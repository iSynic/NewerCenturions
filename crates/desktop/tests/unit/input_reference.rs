use super::*;
use centurions_content::{Content, Warrior};

#[test]
fn held_navigation_repeats_without_backlog_and_release_stops_it() {
    let mut input = MovementInput::default();
    for (key, direction) in [
        (KeyCode::Right, (1, 0)),
        (KeyCode::Kp1, (-1, 1)),
        (KeyCode::Kp8, (0, -1)),
        (KeyCode::Kp9, (1, -1)),
    ] {
        input.reset();
        let actual = movement_direction(|pressed| pressed == key);
        assert_eq!(actual, direction);
        assert!(matches!(input.step(actual, 1.), Some(Command::Move { .. })));
        assert!(input.step(actual, 1.05).is_none());
        assert!(input.step(actual, 1.11).is_some());
        assert!(input.step(actual, 10.).is_some());
        assert!(input.step(actual, 10.).is_none());
        assert!(input.step((0, 0), 10.).is_none());
        assert!(input.step(actual, 10.).is_some());
    }
}

fn state() -> Session {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../evidence/fixtures/player-psi.json"
    ))
    .unwrap();
    serde_json::from_value(fixture["route_foundation"].clone()).unwrap()
}

fn command(action: Action) -> Command {
    let Action::Command(command) = action else {
        panic!("refill must use the production command boundary");
    };
    command
}

#[test]
fn combat_refill_selects_equipped_weapon_and_ordinary_ammunition() {
    let mut state = state();
    let actor: &mut Warrior = &mut state.warriors[0];
    actor.bytes[0x1a..0x1c].copy_from_slice(&2_i16.to_be_bytes());
    actor.bytes[0x132..0x15e].fill(0);
    actor.bytes[0x132..0x134].copy_from_slice(&165_i16.to_be_bytes());
    // An unequipped earlier row must not steal the Load button or R key.
    for (weapon, ammunition) in [(1_i16, 200), (2, 201), (38, 203), (165, 280)] {
        state.warriors[0].bytes[0x148..0x14a].copy_from_slice(&weapon.to_be_bytes());
        state.warriors[0].bytes[0x151] = 1;
        let Command::FitAmmunition {
            warrior,
            slot,
            ammunition: selected,
        } = command(refill_action(&state, 0))
        else {
            panic!("combat refill did not select ammunition");
        };
        assert_eq!((warrior, slot, selected), (0, 1, ammunition));
    }
}

#[test]
fn exploration_refill_uses_manual_command_and_no_weapon_fallback_is_bounded() {
    let mut state = state();
    state.phase = Phase::Exploration;
    assert!(matches!(
        command(refill_action(&state, 0)),
        Command::Reload { warrior: 0 }
    ));
    state.phase = Phase::Combat;
    state.warriors[0].bytes[0x1a..0x1c].copy_from_slice(&0_i16.to_be_bytes());
    assert!(matches!(
        command(refill_action(&state, 0)),
        Command::Reload { warrior: 0 }
    ));
    // The rule boundary, rather than desktop input, reports unsupported use.
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let content = Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap();
    let before = state.clone();
    assert!(
        centurions_core::apply(command(refill_action(&state, 0)), &mut state, &content).is_err()
    );
    assert_eq!(state, before);
}
