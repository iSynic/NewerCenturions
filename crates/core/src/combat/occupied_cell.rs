//! Occupied-cell reload continuation from combatchoice.
use crate::combat::runtime;
use crate::{Event, RuleError, Session};
use centurions_content::Content;

pub(super) fn reload_at_occupied_cell(
    state: &mut Session,
    content: &Content,
    mission: &centurions_content::mission::OpeningMission,
    actor: usize,
    target: i16,
    dx: i8,
) -> Result<Vec<Event>, RuleError> {
    let unsupported = || RuleError::Unsupported {
        action: "combat.move",
        dependency: "other occupied-cell attack, exchange or reload continuation",
    };
    let combat = state.combat.as_mut().ok_or_else(unsupported)?;
    let member = &state.warriors[actor];
    if member.record().stamina()? <= 0
        || member.record().active_flag() != 1
        || member.record().action_points()? < 2
    {
        return Err(unsupported());
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
        return Err(unsupported());
    };
    if !living || faction == member.record().faction()? {
        return Err(unsupported());
    }
    let attacks = mission
        .combat
        .as_ref()
        .map_or(&[][..], |tables| &tables.attacks);
    let mut effects = runtime::check_neighbors_with_work(
        combat,
        &mut state.warriors,
        &mut state.random,
        actor as i16,
        &content.items,
        attacks,
        &state.party_buffs,
        &mut state.ammunition_work,
    )?;
    if state.warriors[actor].record().active_flag() == 0 {
        effects.extend(runtime::end_turn(state, content)?);
        return Ok(effects);
    }
    state.warriors[actor].record_mut().face_horizontal_step(dx);
    let weapon_id = state.warriors[actor].record().equipped_weapon()?;
    let equipped = state.warriors[actor]
        .inventory()?
        .iter()
        .find(|item| item.id() == weapon_id && item.equipment_flag() == 1)
        .cloned()
        .ok_or_else(unsupported)?;
    if !state.warriors[actor].record().firing_mode() {
        return Err(unsupported());
    }
    let weapon = crate::ammunition::definition(content, weapon_id)?;
    if equipped.quantity()?
        >= centurions_content::records::ItemDefinition(&weapon).rounds_per_shot()?
    {
        let mut effects = effects;
        effects.extend(runtime::player_shoot(state, content, target)?);
        return Ok(effects);
    }
    state.attack_count = 0;
    // prepattack14c7be returns before loading ammunition only for an empty
    // magazine. A partial burst retains its prepared packet even on refusal.
    if equipped.quantity()? != 0 {
        let ammunition = crate::ammunition::definition(content, equipped.ammunition_id()?)?;
        let prepared =
            crate::combat::firearm::prepare(&state.warriors[actor], &weapon, &ammunition, attacks)?;
        state.ammunition_work.prepare_pair(&weapon, &ammunition)?;
        combat.weapon_work.prepare(&weapon, &ammunition)?;
        state.attack_count = prepared.shots;
        state.attack_packet = Some(prepared.packet.clone());
        combat.attack_packet = Some(prepared.packet);
    } else {
        combat.weapon_work.load_weapon(&weapon)?;
        state.ammunition_work.weapon = Some(weapon);
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
        &state.ammunition_work,
        &mut state.random,
        state.inventory_record.as_ref(),
    )?;
    let completed = refit.native_return != 0;
    effects.extend(crate::ammunition::commit_result(state, actor, refit)?);
    if !completed {
        state.warriors[actor].record_mut().set_automatic_mode(0);
        effects.push(Event::Sound { resource_id: 6000 });
        effects.push(Event::CollectionRejected { warning: 25 });
    }

    Ok(effects)
}
