//! Reloading when a warrior clicks a cell another body holds.
use crate::combat::runtime;
use crate::feedback::{ERROR_SOUND, OUT_OF_AMMUNITION_WARNING};
use crate::{Event, RuleError, Session};
use centurions_content::Content;

/// Starts the active warrior's move against a cell held by a living foe, when the warrior has at
/// least two attacks left. A friend, a dead body or anything else is not supported.
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
    if member.record().in_battle() != 1 {
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
            target.record().stamina()? > 0 && target.record().in_battle() != 0,
        )
    } else if let Some(monster) = usize::try_from(target - 10)
        .ok()
        .and_then(|target| combat.roster.records.get(target))
    {
        (
            monster.record().faction()?,
            !monster.record().removed() && monster.record().stamina()? > 0,
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

/// Reloads or fires the equipped firearm against the occupied cell. A loaded weapon fires.
/// Otherwise the weapon is prepared, the ammunition is refitted, and a warrior who cannot refit
/// hears the error sound and is told the ammunition is out.
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
        .find(|item| item.id() == weapon_id && item.equipped() == 1)
        .cloned()
        .ok_or_else(|| refuse("no equipped weapon"))?;
    if !state.warriors[actor].record().toggle() {
        return Err(refuse("actor is not in firing mode"));
    }
    let weapon = crate::ammunition::definition(content, weapon_id)?;
    if equipped.charge()?
        >= centurions_content::records::ItemDefinition(&weapon).rounds_per_shot()?
    {
        let mut effects = effects;
        effects.extend(runtime::player_shoot(state, content, target)?);
        return Ok(effects);
    }
    state.globals.attack_count = 0;
    // Only an empty magazine stops before ammunition loads; a partial burst keeps
    // its prepared attack even when refused.
    if equipped.charge()? != 0 {
        let ammunition = crate::ammunition::definition(content, equipped.ammunition_id()?)?;
        let prepared =
            crate::combat::firearm::prepare(&state.warriors[actor], &weapon, &ammunition, attacks)?;
        state
            .globals
            .loaded_items
            .prepare_pair(&weapon, &ammunition)?;
        combat.weapon_traits.prepare(&weapon, &ammunition)?;
        state.globals.attack_count = prepared.shots;
        state.globals.spell_info = prepared.packet.clone();
        combat.spell_info = prepared.packet;
    } else {
        combat.weapon_traits.load_weapon(&weapon)?;
        state.globals.loaded_items.weapon = Some(weapon);
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
        &state.globals.loaded_items,
        &mut state.random,
        Some(&state.globals.inventory_record),
    )?;
    let completed = refit.completed_refit();
    effects.extend(crate::ammunition::commit_result(state, actor, refit)?);
    if !completed {
        state.set_in_spell(0);
        state.globals.possessed = 0;
        state.warriors[actor].record_mut().set_computer_control(0);
        effects.push(Event::Sound {
            resource_id: ERROR_SOUND,
        });
        effects.push(Event::CollectionRejected {
            warning: OUT_OF_AMMUNITION_WARNING,
        });
    }

    Ok(effects)
}
