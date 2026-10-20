//! Script execution for maintained exploration.
use super::{
    active_map, finish_dispatch, finish_script, present_menu, present_typed, retain_script_stamp,
};
use crate::{Event, RuleError, Session};
use centurions_content::Content;

mod operation;
use centurions_content::records::StoryRecord;
use operation::Operation;

mod battles;
mod conditions;
mod party;
mod presentation;

#[derive(PartialEq, Eq)]
pub(super) enum ScriptStep {
    Continue,
    Pause,
}

pub(crate) fn run_script(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let mut effects = Vec::new();
    for _ in 0..128 {
        if let Some(stamp) = state
            .exploration
            .as_ref()
            .and_then(|p| p.script.as_ref())
            .and_then(|s| StoryRecord(&s.record).dispatch_stamp())
        {
            retain_script_stamp(state, stamp);
        }
        let map = active_map(state, content)?;
        let position = state.exploration.as_mut().unwrap();
        let Some(script) = position.script.as_mut() else {
            if let Some(closed) = finish_dispatch(state, content)? {
                effects.extend(closed);
            }
            return Ok(effects);
        };
        if usize::from(script.cursor) >= StoryRecord::ACTIONS {
            if let Some(menu) = script.menu.as_mut() {
                let before = menu.remaining;
                menu.remaining = before.wrapping_sub(1);
                if before > 1 {
                    let id = menu.id;
                    if menu.typed {
                        position.presentation = Some(present_typed(
                            map,
                            if state.typed_encounters.is_empty() {
                                &map.typed_encounters
                            } else {
                                &state.typed_encounters
                            },
                            script,
                            id,
                        )?);
                        effects.push(Event::StoryPresented);
                        return Ok(effects);
                    }
                    let table = if state.menus.is_empty() {
                        &map.menus
                    } else {
                        &state.menus
                    };
                    effects.push(Event::Sound { resource_id: 20005 });
                    position.presentation = present_menu(map, table, script, id)?;
                    if position.presentation.is_some() {
                        effects.push(Event::StoryPresented);
                        return Ok(effects);
                    }
                    continue;
                }
            }
            effects.extend(finish_script(state, content, true)?);
            return Ok(effects);
        }
        let cursor = usize::from(script.cursor);
        let (opcode, parameter) = StoryRecord(&script.record).instruction(cursor)?;
        let operation = Operation::from_native(opcode);
        script.cursor += 1;
        let step = match operation {
            Operation::CombatCleanup
            | Operation::UndoLastStep
            | Operation::ReplaceActor
            | Operation::EndCombat
            | Operation::NeutralizeActors
            | Operation::SetFactionCondition
            | Operation::SummonActors
            | Operation::RemoveActorClass
            | Operation::ReplaceRoundScript
            | Operation::SelectedSpell
            | Operation::PartySpell
            | Operation::SelectPartyBySave
            | Operation::EditSchedule
            | Operation::GroupBattle
            | Operation::SelectedBattle
            | Operation::NpcBattle
            | Operation::StageTransition => battles::execute(
                operation,
                parameter,
                state,
                content,
                map,
                cursor,
                &mut effects,
            )?,
            Operation::Clinic
            | Operation::AdjustStatistics
            | Operation::Confiscate
            | Operation::Shop
            | Operation::RevealNote
            | Operation::SelectedHealth
            | Operation::PartyHealth
            | Operation::FilterParty
            | Operation::SetRecoveryRestrictions
            | Operation::ChangeFatigue
            | Operation::SetMissionFlag
            | Operation::AddMissionScore
            | Operation::AddAlly
            | Operation::RemoveAllies
            | Operation::CopyMapEvent
            | Operation::ReplaceScript
            | Operation::LimitedShopOrTreasure
            | Operation::Treasure
            | Operation::Experience => party::execute(
                operation,
                parameter,
                state,
                content,
                map,
                cursor,
                &mut effects,
            )?,
            Operation::ClearObjects
            | Operation::ChangeRegion
            | Operation::ReplaceMapEvent
            | Operation::ChangeTiles
            | Operation::SetEventChances
            | Operation::Pause
            | Operation::MapEffect
            | Operation::ConsumePartyItems
            | Operation::ChooseWarrior
            | Operation::Travel
            | Operation::Menu
            | Operation::TypedMenu
            | Operation::PartyItemBranch
            | Operation::UpdateVariable
            | Operation::VariableBranch
            | Operation::RandomBranch
            | Operation::AllyClassBranch
            | Operation::DisableMenuEntry
            | Operation::PaymentBranch
            | Operation::ItemPrerequisiteBranch
            | Operation::ChanceBranch
            | Operation::FlagBranch => conditions::execute(
                operation,
                parameter,
                state,
                content,
                map,
                cursor,
                &mut effects,
            )?,
            _ => presentation::execute(
                operation,
                parameter,
                state,
                content,
                map,
                cursor,
                &mut effects,
            )?,
        };
        if step == ScriptStep::Pause {
            return Ok(effects);
        }
    }
    Err(RuleError::Invalid(
        "story exceeds instruction budget".into(),
    ))
}
