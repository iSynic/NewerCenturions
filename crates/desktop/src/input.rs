use crate::{
    layout,
    presentation::{Action, Control, Inspection, UiState, tactical_view},
};
use centurions_core::{Command, Phase, Presentation, Session};
use macroquad::prelude::*;

pub fn refill_action(state: &Session, warrior: usize) -> Action {
    if state.phase == Phase::Combat
        && let Some((slot, item)) = state
            .warriors
            .get(warrior)
            .and_then(|actor| actor.inventory().ok())
            .and_then(|inventory| {
                inventory
                    .into_iter()
                    .enumerate()
                    .find(|(_, item)| item.0[9] != 0 && matches!(item.id(), 1 | 2 | 38 | 165))
            })
    {
        let ammunition = match item.id() {
            1 => 200,
            2 => 201,
            38 => 203,
            165 => 280,
            _ => unreachable!(),
        };
        return Action::Command(Command::FitAmmunition {
            warrior,
            slot,
            ammunition,
        });
    }
    Action::Command(Command::Reload { warrior })
}

pub fn action(
    state: &Session,
    ui: &mut UiState,
    controls: &[Control],
    pointer: Option<Vec2>,
) -> Option<Action> {
    if state.medical_continuation.is_some() {
        if is_mouse_button_pressed(MouseButton::Left)
            && let Some(p) = pointer
            && let Some(c) = controls.iter().rev().find(|c| c.rect.contains(p))
        {
            return c.enabled.then(|| c.action.clone());
        }
        if is_key_pressed(KeyCode::Escape) {
            return Some(if ui.medical_cancel {
                Action::KeepMedicalTarget
            } else {
                Action::MedicalCancel
            });
        }
        return None;
    }
    if let Some((spell, power)) = ui.psi_pending {
        if is_key_pressed(KeyCode::Escape) {
            return Some(Action::Close);
        }
        if is_mouse_button_pressed(MouseButton::Left)
            && let Some(p) = pointer
            && layout::MAP.contains(p)
            && let Some((vx, vy)) = tactical_view(state)
        {
            return Some(Action::Command(Command::CastPsi {
                spell,
                power,
                target: Some((
                    vx + ((p.x - layout::MAP.x) / 32.) as i16,
                    vy + ((p.y - layout::MAP.y) / 32.) as i16,
                )),
            }));
        }
        return None;
    }
    if !state
        .exploration
        .as_ref()
        .and_then(|position| position.presentation.as_ref())
        .is_some_and(|presentation| matches!(presentation, Presentation::Typed { .. }))
    {
        while get_char_pressed().is_some() {}
    }
    if is_mouse_button_pressed(MouseButton::Left)
        && let Some(p) = pointer
        && let Some(c) = controls.iter().rev().find(|c| c.rect.contains(p))
    {
        return c.enabled.then(|| c.action.clone());
    }
    if ui.inspection.is_some() {
        return is_key_pressed(KeyCode::Escape).then_some(
            if ui.inspection == Some(Inspection::Bandage) {
                Action::Command(Command::StopBleeding { warrior: None })
            } else {
                Action::Close
            },
        );
    }
    if let Some(p) = state
        .exploration
        .as_ref()
        .and_then(|p| p.presentation.as_ref())
    {
        if matches!(p, Presentation::WarriorChoice { .. }) {
            for (index, key) in [
                KeyCode::Key1,
                KeyCode::Key2,
                KeyCode::Key3,
                KeyCode::Key4,
                KeyCode::Key5,
                KeyCode::Key6,
            ]
            .into_iter()
            .enumerate()
            {
                if is_key_pressed(key) {
                    return Some(Action::Command(Command::ChooseWarrior { index }));
                }
            }
            return None;
        }
        if matches!(
            p,
            Presentation::Question { .. } | Presentation::StageChange { .. }
        ) {
            return if is_key_pressed(KeyCode::Y) {
                Some(Action::Command(Command::Choose { answer: true }))
            } else if is_key_pressed(KeyCode::N) {
                Some(Action::Command(Command::Choose { answer: false }))
            } else {
                None
            };
        }
        if let Presentation::Menu { labels, .. } = p {
            for (index, key) in [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4]
                .into_iter()
                .enumerate()
            {
                if index < labels.len() && is_key_pressed(key) {
                    return Some(Action::Command(Command::SelectMenuEntry { index }));
                }
            }
            return is_key_pressed(KeyCode::Escape).then_some(Action::Command(Command::CancelMenu));
        }
        if matches!(p, Presentation::Typed { .. }) {
            while let Some(character) = get_char_pressed() {
                if !character.is_control() && ui.typed_text.len() < 40 {
                    ui.typed_text.push(character);
                }
            }
            if is_key_pressed(KeyCode::Backspace) {
                ui.typed_text.pop();
            }
            if is_key_pressed(KeyCode::Escape) {
                ui.typed_text.clear();
                return Some(Action::Command(Command::CancelMenu));
            }
            return is_key_pressed(KeyCode::Enter).then(|| {
                Action::Command(Command::SubmitText {
                    text: std::mem::take(&mut ui.typed_text),
                })
            });
        }
        return (is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space)).then_some(
            Action::Command(
                if matches!(p, Presentation::Loot)
                    && centurions_core::needs_spell_confirmation(state)
                {
                    Command::KeepCurrentSpells
                } else {
                    Command::Continue
                },
            ),
        );
    }
    if state.phase == Phase::SquadSelection {
        return None;
    }
    let active = state.combat.as_ref().and_then(|c| c.active_body());
    if active.is_some_and(|b| b >= 10) {
        return None;
    }
    let selected = active.map(|b| b as usize).unwrap_or(ui.selected);
    if state.phase == Phase::Exploration && is_key_pressed(KeyCode::E) {
        return Some(Action::Command(Command::Encounter));
    }
    if is_key_pressed(KeyCode::R) {
        return Some(refill_action(state, selected));
    }
    if is_key_pressed(KeyCode::P) {
        return Some(Action::Open(Inspection::Psi));
    }
    if state.phase == Phase::Combat {
        if is_key_pressed(KeyCode::B) {
            return Some(if state.bandage_ready.is_some_and(|ready| ready != 0) {
                Action::Open(Inspection::Bandage)
            } else {
                Action::Command(Command::StopBleeding { warrior: None })
            });
        }
        if is_key_pressed(KeyCode::F) {
            return Some(Action::Command(Command::SetFirearmMode {
                warrior: selected,
                enabled: state.warriors[selected].bytes[0x9dd] == 0,
            }));
        }
        if is_key_pressed(KeyCode::G) {
            return Some(Action::Command(Command::Guard));
        }
        if is_key_pressed(KeyCode::Space) {
            return Some(Action::Command(Command::EndTurn));
        }
    }
    let dx = i8::from(is_key_pressed(KeyCode::Right)) - i8::from(is_key_pressed(KeyCode::Left));
    let dy = i8::from(is_key_pressed(KeyCode::Down)) - i8::from(is_key_pressed(KeyCode::Up));
    if dx != 0 || dy != 0 {
        return Some(Action::Command(Command::Move { dx, dy }));
    }
    if is_mouse_button_pressed(MouseButton::Left)
        && let Some(p) = pointer
        && layout::MAP.contains(p)
    {
        let x = ((p.x - layout::MAP.x) / 32.) as i32;
        let y = ((p.y - layout::MAP.y) / 32.) as i32;
        if let Some(c) = &state.combat {
            let (vx, vy) = tactical_view(state).unwrap();
            let x = vx + x as i16;
            let y = vy + y as i16;
            let index = (x + c.map.shift.0 + 5) as usize * 90 + (y + c.map.shift.1 + 5) as usize;
            let body = c.map.tiles[index].wrapping_abs();
            if body >= 10 && body < 10 + c.enemies.records.len() as i16 {
                return Some(Action::Command(
                    if state.warriors[selected].bytes[0x9dd] != 0 {
                        Command::ShootEnemy { body }
                    } else {
                        Command::AttackEnemy { body }
                    },
                ));
            }
            let origin = c.squad.positions[selected];
            let dx = (x - origin.0).signum() as i8;
            let dy = (y - origin.1).signum() as i8;
            if dx != 0 || dy != 0 {
                return Some(Action::Command(Command::Move { dx, dy }));
            }
        } else if let Some(position) = &state.exploration {
            let x = x + (position.x - 7).clamp(0, 75);
            let y = y + (position.y - 6).clamp(0, 77);
            let dx = (x - position.x).signum() as i8;
            let dy = (y - position.y).signum() as i8;
            if dx != 0 || dy != 0 {
                return Some(Action::Command(Command::Move { dx, dy }));
            }
        }
    }
    None
}

#[cfg(all(test, feature = "original-fixtures"))]
mod tests {
    use super::*;
    use centurions_content::{Content, Warrior};

    fn state() -> Session {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../../../evidence/fixtures/player-psi.json"))
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
            centurions_core::apply(command(refill_action(&state, 0)), &mut state, &content)
                .is_err()
        );
        assert_eq!(state, before);
    }
}
