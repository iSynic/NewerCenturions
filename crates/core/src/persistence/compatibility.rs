//! Ordered compatibility pipeline for schema-13 saves and versions 1-12.
//! Some legacy fields must be rejected, others restored; these steps must not
//! be reordered into a blanket defaults pass followed by validation.
use super::Save;
use crate::{
    MedicalOrigin, MenuContinuation, MovementContinuation, Presentation, RuleError, clinic,
    initial_psi_memory,
};
use centurions_content::Content;

pub(super) fn restore(saved: &mut Save, content: &Content) -> Result<(), RuleError> {
    read_version_and_schedule(saved, content)?;
    check_control_contexts(saved)?;
    check_story_contexts(saved)?;
    restore_actor_defaults(saved)?;
    check_campaign_contexts(saved)?;
    restore_map_defaults(saved, content)?;
    if saved.session.psi_memory.is_none() {
        saved.session.psi_memory = Some(initial_psi_memory(&saved.session.warriors)?);
    }
    Ok(())
}

fn read_version_and_schedule(saved: &mut Save, content: &Content) -> Result<(), RuleError> {
    if saved.schema_version == 1 {
        if let Some(position) = saved.session.exploration.as_mut() {
            if position.day != 1 {
                return Err(RuleError::Invalid(
                    "legacy save after day one has unknown schedule history".into(),
                ));
            }
            position.schedule = content
                .map(0)
                .ok_or_else(|| RuleError::Invalid("opening schedule unavailable".into()))?
                .schedule
                .clone();
            position.region_state = content.map(0).unwrap().regions.clone();
        }
    } else if !matches!(saved.schema_version, 2..=13) {
        return Err(RuleError::Invalid(format!(
            "unsupported save version {}",
            saved.schema_version
        )));
    }
    Ok(())
}

fn check_control_contexts(saved: &mut Save) -> Result<(), RuleError> {
    if saved.schema_version < 13
        && (saved.session.weapon_work.natural_attack_index.is_some()
            || saved
                .session
                .combat
                .as_ref()
                .is_some_and(|combat| combat.weapon_work.natural_attack_index.is_some()))
    {
        return Err("retained natural-attack context requires save version 13".into());
    }
    if saved.schema_version < 12
        && saved
            .session
            .combat
            .as_ref()
            .is_some_and(|c| c.turn_origin.is_some() || c.automatic.iter().any(|v| *v))
    {
        return Err("combat controls require save version 12".into());
    }
    if saved.schema_version < 11
        && saved
            .session
            .combat
            .as_ref()
            .is_some_and(|c| c.capture_actor.is_some())
    {
        return Err("mechanical targeting requires save version 11".into());
    }
    if saved.schema_version < 10 && saved.session.field_controls.is_some() {
        return Err("outdoor controls require save version 10".into());
    }
    Ok(())
}

fn check_story_contexts(saved: &mut Save) -> Result<(), RuleError> {
    if saved.schema_version < 9 {
        let mut page = saved
            .session
            .exploration
            .as_ref()
            .and_then(|p| p.presentation.as_ref());
        while let Some(current) = page {
            match current {
                Presentation::ScriptPause => {
                    return Err("script pause requires save version 9".into());
                }
                Presentation::Messages { next, .. } => page = next.as_deref(),
                _ => break,
            }
        }
        if saved.session.retained_death_queue.is_some() {
            return Err("retained death queue requires save version 9".into());
        }
        let mut story = saved.session.combat_story.as_ref();
        while let Some(current) = story {
            if current.queue_cursor.is_some() {
                return Err("queued caller cursor requires save version 9".into());
            }
            story = current.parent.as_deref();
        }
    }
    if saved.schema_version < 8
        && saved.session.round_dispatch.as_ref().is_some_and(|frame| {
            frame.child_battle
                || frame.parent.is_some()
                || frame.outer_battle.is_some()
                || frame.death_depth.is_some()
        })
    {
        return Err(RuleError::Invalid(
            "round child context requires save version 8".into(),
        ));
    }
    Ok(())
}

fn restore_actor_defaults(saved: &mut Save) -> Result<(), RuleError> {
    if saved.schema_version < 7 {
        saved.session.npc_backing = Default::default();
        let mut story = saved.session.combat_story.as_mut();
        while let Some(current) = story {
            if current.child_returned {
                return Err(RuleError::Invalid(
                    "returned child context requires save version 7".into(),
                ));
            }
            current.direct = false;
            story = current.parent.as_deref_mut();
        }
        if let Some(combat) = saved.session.combat.as_mut() {
            combat.retained_npcs = Default::default();
            combat.direct_deaths = false;
            combat.direct_death = None;
        }
    }
    Ok(())
}

fn check_campaign_contexts(saved: &mut Save) -> Result<(), RuleError> {
    if saved.schema_version < 6
        && (saved.session.time_continuation.is_some()
            || saved.session.scenario_duration_tail.is_some()
            || saved.session.energy_recovery_disabled != 0
            || saved.session.survivors.is_some()
            || saved.session.round_dispatch.is_some()
            || saved.session.scripted_cleanup.is_some()
            || saved.session.actor_replacement.is_some()
            || saved.session.inventory_record.is_some()
            || clinic::pending(&saved.session).is_some()
            || saved.session.psi_selection.is_some()
            || saved.session.outdoor_psi.is_some()
            || saved.session.encounter_actions.is_some()
            || saved.session.exploration.as_ref().is_some_and(|position| {
                position
                    .script
                    .as_ref()
                    .is_some_and(|script| script.item_return)
            })
            || saved
                .session
                .medical_continuation
                .as_ref()
                .is_some_and(|pending| {
                    pending.origin == MedicalOrigin::OwnedNeedle && pending.item != 700
                })
            || saved.session.combat.as_ref().is_some_and(|c| {
                c.round_scripts.pending.is_some()
                    || c.pending_last_defeated.is_some()
                    || c.inactive_rounds != 0
                    || c.selection_pending
                    || c.script_enemy != 0
            })
            || saved.session.promotion.is_some()
            || matches!(
                saved.session.movement_continuation,
                Some(
                    MovementContinuation::Region { .. }
                        | MovementContinuation::RegionalReturn { .. }
                )
            ))
    {
        return Err(RuleError::Invalid(
            "pending runtime context requires save version 6".into(),
        ));
    }
    if saved.schema_version < 5
        && (saved.session.scenario_level != 0
            || !saved.session.visited_levels.is_empty()
            || saved.session.native_import.is_some()
            || !saved.session.shops.is_empty()
            || saved.session.shop_access.is_some()
            || saved.session.confiscated.is_some()
            || saved.session.suppress_allies != 0
            || saved.session.loot_limit.is_some()
            || !saved.session.typed_encounters.is_empty()
            || saved.session.notes.iter().any(|value| *value != 0))
    {
        return Err(RuleError::Invalid(
            "level state requires save version 5".into(),
        ));
    }
    if saved.schema_version < 4 && saved.session.difficulty != 0 {
        return Err(RuleError::Invalid(
            "non-Normal difficulty requires save version 4".into(),
        ));
    }
    Ok(())
}

fn restore_map_defaults(saved: &mut Save, content: &Content) -> Result<(), RuleError> {
    if saved.schema_version < 3
        && let Some(position) = saved.session.exploration.as_mut()
    {
        if position.map != 0 || !saved.session.visited_maps.is_empty() {
            return Err(RuleError::Invalid(
                "legacy save has unreconstructable map history".into(),
            ));
        }
        let map = content
            .map(0)
            .ok_or_else(|| RuleError::Invalid("legacy map is unavailable".into()))?;
        position.visibility = vec![0; 8100];
        position.floor = map.floor.clone();
        position.objects = map.objects.clone();
        position.events = map.events.clone();
        if let Some(Presentation::Menu { id, .. }) = position.presentation.as_ref() {
            let script = position
                .script
                .as_mut()
                .ok_or_else(|| RuleError::Invalid("legacy menu script missing".into()))?;
            if *id != 0 || !script.scheduled {
                return Err(RuleError::Invalid(
                    "legacy menu history is unavailable".into(),
                ));
            }
            script.menu = Some(MenuContinuation {
                id: *id,
                remaining: i16::from(map.menus[101] as i8).wrapping_sub(1),
                original: script.record.clone(),
                typed: false,
            });
        }
    }
    Ok(())
}
