use crate::{Command, Event, RuleError, Session, combat, feedback, outdoor_psi, reference_views};
use centurions_content::Content;

pub(super) fn execute(
    command: Command,
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    match command {
        Command::SwapCombatants { body } => combat::interaction::exchange(state, body),
        Command::AutomaticBandage => {
            let member = state
                .combat
                .as_ref()
                .and_then(combat::Combat::active_body)
                .filter(|n| (0..6).contains(n))
                .ok_or(RuleError::Refused { feedback: vec![] })? as usize;
            let record = state.warriors[member].record();
            if record.automatic_mode()? <= 0
                || record.faction()? != 0
                || record.fleeing_duration()? != 0
            {
                return Err(RuleError::Refused { feedback: vec![] });
            }
            let target = state
                .warriors
                .iter()
                .position(|w| w.record().bleeding_flag() != 0)
                .ok_or(RuleError::Refused { feedback: vec![] })?;
            state.warriors[target].record_mut().set_bleeding_flag(0);
            state.warriors[member].record_mut().set_action_points(0);
            Ok(vec![Event::Feedback(feedback::NativeFeedback::Flash {
                address: 0x403cd6,
                x: 30,
                y: 100,
                wait: 120,
                sound: 10105,
            })])
        }
        Command::BeginAutomaticTurn => combat::controls::automatic(state),
        Command::DelayTurn => combat::controls::delay(state, content),
        Command::UndoCombatMove => combat::controls::undo(state, content),
        Command::ToggleCombatAuto { warrior } => combat::controls::toggle_auto(state, warrior),
        Command::RequestCombatEscape => {
            combat::controls::escape_check(state)?;
            Ok(vec![
                Event::Sound { resource_id: 20005 },
                Event::CombatEscapeOffered,
            ])
        }
        Command::EscapeCombat => combat::controls::escape(state, content),
        Command::SelectCombatWeapon { slot } => {
            combat::controls::select_weapon(state, content, slot)
        }
        Command::ViewCombatBody { body } => reference_views::combat_body(state, content, body),
        Command::Guard => combat::runtime::player_guard(state, content),
        Command::SelectSurvivor { row } => {
            let combat = state.combat.as_ref().ok_or("survivor roster missing")?;
            state
                .survivors
                .as_mut()
                .ok_or("survivor selection missing")?
                .toggle(row, &combat.enemies.records, &mut state.random)
        }
        Command::EndTurn => {
            if let Some(body) = state.combat.as_ref().and_then(|arena| arena.active_body())
                && (0..6).contains(&body)
            {
                state.warriors[body as usize]
                    .record_mut()
                    .finish_turn_movement();
            }
            combat::runtime::end_turn(state, content)
        }
        Command::RunEnemyTurn => combat::runtime::enemy_turn(state, content),
        Command::ResumeCombatSelection => combat::runtime::resume_selection(state, content),
        Command::RunControlledTurn => combat::runtime::controlled_turn(state, content),
        Command::BeginCapture => combat::capture::begin(state),
        Command::SelectCaptureTarget { x, y } => combat::capture::select(state, content, x, y),
        Command::CancelCapture => combat::capture::cancel(state),
        Command::AttackEnemy { body } => combat::runtime::player_attack(state, content, body),
        Command::AttackFriend { body } => {
            combat::runtime::player_attack_friend(state, content, body)
        }
        Command::ShootEnemy { body } => combat::runtime::player_shoot(state, content, body),
        Command::ShootAt { x, y } => combat::shooting::fire_at(state, content, (x, y)),
        Command::FireSelectedTargets => combat::shooting::fire_selected(state, content),
        Command::CancelFireTargets => combat::shooting::cancel(state),
        Command::CastPsi {
            rotation,
            spell,
            power,
            target,
        } => combat::runtime::player_psi(state, content, spell, power, target, rotation),
        Command::CastPartyPsi {
            warrior,
            spell,
            power,
        } => combat::runtime::party_psi(state, content, warrior, spell, power),
        Command::CastPsiOnWarrior {
            warrior,
            spell,
            power,
        } => combat::runtime::player_psi_warrior(state, content, warrior, spell, power),
        Command::BeginMedicalItem { warrior, slot } => {
            combat::runtime::begin_medical_item(state, content, warrior, slot)
        }
        Command::BeginLearnedHeal { power } => {
            combat::runtime::begin_learned_heal(state, content, power)
        }
        Command::BeginPsiSelection { spell, power } => {
            combat::psi_selection::begin(state, content, spell, power)
        }
        Command::SelectPsiTarget { x, y } => combat::psi_selection::select(state, content, (x, y)),
        Command::FirePsiSelection => combat::psi_selection::fire(state, content),
        Command::BeginPsiTargeting { spell, power } => {
            combat::psi_selection::begin_targeting(state, content, spell, power)
        }
        Command::FirePreparedPsi { x, y, rotation } => {
            combat::psi_selection::fire_at(state, content, (x, y), rotation)
        }
        Command::FirePreparedPsiOnWarrior { warrior } => {
            combat::psi_selection::fire_warrior(state, content, warrior)
        }
        Command::BeginFireTargets => combat::targeting::begin_firearm(state, content),
        Command::AdjustTargetQuantity { increase } => {
            combat::targeting::adjust(state, content, increase)
        }
        Command::ClearPsiTargets => combat::psi_selection::clear(state),
        Command::CancelPsiSelection => combat::psi_selection::cancel(state, content),
        Command::SelectOutdoorPsiTarget { warrior } => outdoor_psi::select(state, content, warrior),
        Command::FinishOutdoorPsi => outdoor_psi::finish(state, content),
        Command::CancelOutdoorPsi => outdoor_psi::cancel(state, content),
        Command::SelectMedicalTarget { selection } => {
            combat::runtime::select_medical_target(state, content, selection)
        }
        Command::StopBleeding { warrior } => {
            combat::runtime::stop_bleeding(state, content, warrior)
        }
        Command::SetFirearmMode { warrior, enabled } => {
            combat::controls::firearm_mode(state, content, warrior, enabled)
        }
        _ => unreachable!("command routed to wrong subsystem"),
    }
}
