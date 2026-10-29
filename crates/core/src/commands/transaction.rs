use super::apply_inner;
use crate::{
    Command, Event, Presentation, RuleError, Session, combat, exploration, feedback, field_controls,
};
use centurions_content::Content;

pub fn apply(
    command: Command,
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    let selection = if let Command::EnterMission { name } = &command {
        Some(name.as_str())
    } else {
        state.mission.as_deref()
    };
    let context = content.context(selection, state.scenario_level)?;
    let content = context.as_ref();
    if let Command::WithPreferences {
        command,
        preferences,
    } = command
    {
        if matches!(*command, Command::WithPreferences { .. }) {
            return Err("nested command preferences".into());
        }
        if !(0..=50).contains(&preferences.animation_delay) || preferences.animation_delay % 5 != 0
        {
            return Err("invalid original animation speed".into());
        }
        let mut candidate = state.clone();
        let previous = candidate.command_preferences.clone();
        let _trace = crate::trace::Scope::enable(preferences.trace);
        candidate.command_preferences = preferences;
        let result = apply(*command, &mut candidate, content);
        candidate.command_preferences = previous;
        if result.is_ok() || matches!(result, Err(RuleError::RefusedAfterEffects { .. })) {
            *state = candidate;
        }
        return result;
    }
    state.validate(content)?;
    let Some(command) = normalize(command, state)? else {
        return Ok(vec![]);
    };
    let kind = ExecutionKind::of(&command, state);
    let mut candidate = state.clone();
    candidate.command_return = false;
    if let Some(combat) = candidate.combat.as_mut() {
        combat.direct_deaths = kind.melee_input;
    }
    candidate.scope_monster_template();
    let mut events = match apply_inner(command, &mut candidate, content) {
        Ok(events) => events,
        Err(error @ RuleError::RefusedAfterEffects { .. }) => {
            candidate.commit_alias_writes()?;
            candidate.store_spell_fatigue();
            candidate.command_return = false;
            if let Some(combat) = candidate.combat.as_mut() {
                combat.direct_deaths = false;
            }
            candidate.retain_npc_work();
            candidate.retain_monster_work();
            candidate.release_monster_template();
            candidate.validate(content)?;
            *state = candidate;
            return Err(error);
        }
        Err(error) => return Err(error),
    };
    events.extend(combat::runtime::resume_deferred_getup(
        &mut candidate,
        content,
    )?);
    events.extend(combat::runtime::resume_npc_action(&mut candidate, content)?);
    // Every live caster commits its writes at once; this keeps none past the boundary.
    candidate.commit_alias_writes()?;
    candidate.store_spell_fatigue();
    candidate.retain_npc_work();
    candidate.retain_monster_work();
    candidate.release_monster_template();
    let direct_return = std::mem::take(&mut candidate.command_return);
    if kind.targeting_only || kind.retained_getup_return || direct_return {
        candidate.validate(content)?;
        *state = candidate;
        return Ok(events);
    }
    finish_action(&mut candidate, content, &mut events, &kind)?;
    candidate.commit_alias_writes()?;
    candidate.retain_npc_work();
    candidate.validate(content)?;
    *state = candidate;
    Ok(events)
}

fn normalize(mut command: Command, state: &Session) -> Result<Option<Command>, RuleError> {
    if let Command::CombatClick { x, y } = command {
        let Some(resolved) = combat::click_command(state, x, y)? else {
            return Ok(None);
        };
        command = resolved;
    }
    if let Command::Move { dx, dy } = command
        && let Some(combat) = state.combat.as_ref().filter(|c| c.capture_actor.is_some())
    {
        let origin = combat.squad.positions[combat.capture_actor.unwrap() as usize];
        command = Command::SelectCaptureTarget {
            x: origin.0.wrapping_add(i16::from(dx)),
            y: origin.1.wrapping_add(i16::from(dy)),
        };
    }
    Ok(Some(command))
}

struct ExecutionKind {
    targeting_only: bool,
    retained_getup_return: bool,
    editing: bool,
    melee_input: bool,
    physical_work_input: bool,
}
impl ExecutionKind {
    fn of(command: &Command, state: &Session) -> Self {
        let retained_getup_return =
            matches!(command, Command::Continue | Command::SelectSurvivor { .. })
                && matches!(
                    state.scripted_cleanup,
                    Some(
                        crate::story_cleanup::Stage::RetainedGetupSurvivors { .. }
                            | crate::story_cleanup::Stage::RetainedSelectedSurvivors { .. }
                    )
                );
        let targeting_only = matches!(
            command,
            Command::BeginCapture
                | Command::CancelCapture
                | Command::BeginAutomaticTurn
                | Command::ToggleCombatAuto { .. }
                | Command::RequestCombatEscape
                | Command::ViewCombatBody { .. }
                | Command::BeginFireTargets
                | Command::BeginPsiTargeting { .. }
                | Command::AdjustTargetQuantity { .. }
                | Command::ClearPsiTargets
        );
        let editing = matches!(
            command,
            Command::EditWarriorUpgrade { .. }
                | Command::CreationDialog
                | Command::CreationEquipment
                | Command::CreationScans
                | Command::GenerateWarrior { .. }
                | Command::ReplaceSquad { .. }
                | Command::ReorderSquad { .. }
                | Command::EndMission
                | Command::RefitParty
                | Command::DropItem { .. }
                | Command::TransferItem { .. }
                | Command::OpenShop
                | Command::CloseShop
                | Command::BuyItem { .. }
                | Command::SellItem { .. }
                | Command::SetWarriorAppearance { .. }
                | Command::ViewWarrior { .. }
                | Command::InspectInventory { .. }
                | Command::OpenInventory { .. }
                | Command::SwitchInventoryOwner { .. }
                | Command::ScrollInventory { .. }
                | Command::HighlightInventoryRow { .. }
                | Command::ShowItemInformation { .. }
                | Command::RetargetItemInformation { .. }
                | Command::ListItemUsers { .. }
                | Command::ShowLootItemInformation { .. }
                | Command::ShowDraggedItemInformation { .. }
                | Command::ChooseAmmunition { .. }
                | Command::ViewCreature { .. }
                | Command::ViewNote { .. }
                | Command::UpgradeWarrior { .. }
                | Command::EnterWarriorUpgrade { .. }
                | Command::EditCarriedItem { .. }
                | Command::EditWarriorLevel { .. }
                | Command::EditWarriorScans { .. }
        );
        let melee_input = matches!(
            command,
            Command::AttackEnemy { .. }
                | Command::AttackFriend { .. }
                | Command::Move { .. }
                | Command::AnswerOccupiedChoice {
                    exchange: Some(false)
                }
        );
        let physical_work_input = matches!(
            command,
            Command::AttackEnemy { .. }
                | Command::AttackFriend { .. }
                | Command::Move { .. }
                | Command::RunEnemyTurn
                | Command::RunControlledTurn
                | Command::AnswerOccupiedChoice {
                    exchange: Some(false)
                }
        );
        Self {
            targeting_only,
            retained_getup_return,
            editing,
            melee_input,
            physical_work_input,
        }
    }
}

fn finish_action(
    candidate: &mut Session,
    content: &Content,
    events: &mut Vec<Event>,
    kind: &ExecutionKind,
) -> Result<(), RuleError> {
    for event in events.iter() {
        match event {
            Event::Feedback(feedback::NativeFeedback::CombatResult { .. }) => {
                candidate.bandage_ready = 0;
                if let Some(combat) = candidate.combat.as_mut() {
                    combat.neighbor_work.departure = 0;
                } else {
                    candidate.native_work.neighbor_work.departure = 0;
                }
            }
            Event::AttackResolved { damage, .. } if kind.melee_input || *damage > 0 => {
                candidate.bandage_ready = 0;
            }
            Event::SpellResolved { spell, damage, .. } if *damage > 0 || *spell == 1305 => {
                candidate.bandage_ready = 0;
            }
            Event::TurnStarted { body, .. } => {
                candidate.bandage_ready = 1;
                if (0..6).contains(body) {
                    if let Some(combat) = candidate.combat.as_mut() {
                        combat.neighbor_work.departure = 0;
                    } else {
                        candidate.native_work.neighbor_work.departure = 0;
                    }
                }
            }
            _ => {}
        }
    }
    if candidate.retained_walk_choice.is_none()
        && !kind.editing
        && !events
            .iter()
            .any(|event| matches!(event, Event::ItemUpgraded { .. }))
        && !events
            .iter()
            .any(|event| matches!(event, Event::Message { .. }))
    {
        events.extend(crate::time::resume_owned(candidate, content)?);
        events.extend(exploration::resume_movement(candidate, content)?);
        events.extend(field_controls::resume(candidate, content)?);
    }
    if kind.physical_work_input
        && let Some(item) = candidate.native_work.ammunition_work.item.as_ref()
    {
        candidate.native_work.loaded_item_field32 =
            centurions_content::records::ItemDefinition(item).retained_field32()?;
    }
    combat::runtime::retain_last_defeated(candidate)?;
    candidate.retain_combat_work();
    let pages: Vec<_> = events
        .iter()
        .filter_map(|event| {
            if let Event::Message { id, text } = event {
                Some((*id, text.clone()))
            } else {
                None
            }
        })
        .collect();
    if !pages.is_empty()
        && let Some(position) = candidate.exploration.as_mut()
    {
        let next = position.presentation.take().map(Box::new);
        position.presentation = Some(Presentation::Messages { pages, next });
    }
    if let Some(combat) = candidate.combat.as_mut() {
        combat.direct_deaths = false;
    }
    Ok(())
}
