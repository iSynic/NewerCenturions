//! Occupied-cell reload continuation from combatchoice.
use crate::combat::runtime;
use crate::feedback::{ERROR_SOUND, OUT_OF_AMMUNITION_WARNING};
use crate::{Event, RuleError, Session};
use centurions_content::Content;

pub(super) fn reload_at_occupied_cell(
    state: &mut Session,
    content: &Content,
    actor: usize,
    target: i16,
    dx: i8,
    dy: i8,
) -> Result<Vec<Event>, RuleError> {
    let refuse = |predicate: &'static str| {
        RuleError::Unsupported {
            action: "combat.move",
            dependency: "other occupied-cell attack, exchange or reload continuation",
        }
        .with_context(|| format!("occupied cell {target}: {predicate}"))
    };
    let combat = state
        .combat
        .as_mut()
        .ok_or_else(|| refuse("no active combat"))?;
    let member = &state.warriors[actor];
    if member.record().stamina()? <= 0 {
        return Err(refuse("actor has no stamina"));
    }
    if member.record().active_flag() != 1 {
        return Err(refuse("actor is not active"));
    }
    if member.record().action_points()? < 2 {
        return Err(refuse("actor has fewer than two action points"));
    }
    let (faction, living) = if let Some(target) = usize::try_from(target)
        .ok()
        .and_then(|target| state.warriors.get(target))
    {
        (
            target.record().faction()?,
            target.record().stamina()? > 0 && target.record().active_flag() != 0,
        )
    } else if let Some(enemy) = usize::try_from(target - 10)
        .ok()
        .and_then(|target| combat.enemies.records.get(target))
    {
        (
            enemy.record().faction()?,
            !enemy.record().removed() && enemy.record().stamina()? > 0,
        )
    } else {
        return Err(refuse("no body at target"));
    };
    if !living {
        return Err(refuse("target is not living"));
    }
    if faction == member.record().faction()? {
        return Err(refuse("target is friendly"));
    }
    runtime::begin_move(
        state,
        content,
        actor,
        (dx, dy),
        runtime::MovePlan::Occupied { tile: target },
    )
}

pub(super) fn resume_reload(
    state: &mut Session,
    content: &Content,
    mission: &centurions_content::mission::OpeningMission,
    actor: usize,
    target: i16,
    dx: i8,
) -> Result<Vec<Event>, RuleError> {
    let refuse = |predicate: &'static str| {
        RuleError::Unsupported {
            action: "combat.move",
            dependency: "other occupied-cell attack, exchange or reload continuation",
        }
        .with_context(|| format!("occupied cell {target}: {predicate}"))
    };
    let attacks = mission
        .combat
        .as_ref()
        .map_or(&[][..], |tables| &tables.attacks);
    let combat = state
        .combat
        .as_mut()
        .ok_or_else(|| refuse("no active combat"))?;
    let mut effects = vec![];
    state.warriors[actor].record_mut().face_horizontal_step(dx);
    let weapon_id = state.warriors[actor].record().equipped_weapon()?;
    let equipped = state.warriors[actor]
        .inventory()?
        .iter()
        .find(|item| item.id() == weapon_id && item.equipment_flag() == 1)
        .cloned()
        .ok_or_else(|| refuse("no equipped weapon"))?;
    if !state.warriors[actor].record().firing_mode() {
        return Err(refuse("actor is not in firing mode"));
    }
    let weapon = crate::ammunition::definition(content, weapon_id)?;
    if equipped.quantity()?
        >= centurions_content::records::ItemDefinition(&weapon).rounds_per_shot()?
    {
        let mut effects = effects;
        effects.extend(runtime::player_shoot(state, content, target)?);
        return Ok(effects);
    }
    state.native_work.attack_count = 0;
    // prepattack14c7be returns before loading ammunition only for an empty
    // magazine. A partial burst retains its prepared packet even on refusal.
    if equipped.quantity()? != 0 {
        let ammunition = crate::ammunition::definition(content, equipped.ammunition_id()?)?;
        let prepared =
            crate::combat::firearm::prepare(&state.warriors[actor], &weapon, &ammunition, attacks)?;
        state
            .native_work
            .ammunition_work
            .prepare_pair(&weapon, &ammunition)?;
        combat.weapon_work.prepare(&weapon, &ammunition)?;
        state.native_work.attack_count = prepared.shots;
        state.native_work.attack_packet = prepared.packet.clone();
        combat.attack_packet = prepared.packet;
    } else {
        combat.weapon_work.load_weapon(&weapon)?;
        state.native_work.ammunition_work.weapon = Some(weapon);
    }
    let refit = crate::ammunition::refit_with_record(
        &state.warriors[actor],
        &state.ammunition,
        content,
        crate::ammunition::RefitFlags {
            in_combat: true,
            force: true,
            silent: true,
            warnings: false,
        },
        &state.native_work.ammunition_work,
        &mut state.random,
        Some(&state.native_work.inventory_record),
    )?;
    let completed = refit.completed_refit();
    effects.extend(crate::ammunition::commit_result(state, actor, refit)?);
    if !completed {
        state.set_ready(0);
        state.native_work.walking_confusion = 0;
        state.warriors[actor].record_mut().set_automatic_mode(0);
        effects.push(Event::Sound {
            resource_id: ERROR_SOUND,
        });
        effects.push(Event::CollectionRejected {
            warning: OUT_OF_AMMUNITION_WARNING,
        });
    }

    Ok(effects)
}
