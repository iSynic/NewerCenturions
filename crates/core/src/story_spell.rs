mod outdoors;
mod targets;
use crate::{Content, Event, RuleError, Session, combat};
use centurions_content::records::{AttackPacketMut, ItemDefinition, ScriptedSpellExtra};
pub(crate) use outdoors::{party as resolve_party_outdoors, resolve as resolve_outdoors};

/// newland 17/18: the existing selection mask is retained by 17 and replaced by 18.
pub(crate) fn apply(
    state: &mut Session,
    content: &Content,
    extra: &[u8],
    all: bool,
) -> Result<Vec<Event>, RuleError> {
    let caster = targets::caster(state);
    let extra = ScriptedSpellExtra::new(extra)?;
    let id = extra.spell()?;
    let count = extra.attack_count()?;
    let map = content
        .map_at(
            state.mission.as_deref(),
            state.scenario_level,
            state.exploration.as_ref().unwrap().map,
        )
        .ok_or_else(|| RuleError::Invalid("script spell map missing".into()))?;
    let mut packet = combat::spells::load_packet(&map.combat.as_ref().unwrap().attacks, id)?;
    AttackPacketMut(&mut packet).add_save_bonus(extra.save_bonus()?);
    if extra.overrides_saving_throw()? {
        AttackPacketMut(&mut packet).set_saving_throw_mode(3);
    }
    // resolvespell's movement and summon arms read WORD443722/443724. newland17 writes
    // its script target there (143a82..143ae2); newland18 leaves an earlier writer's cell.
    let arm = combat::spells::detached::arm_of(&packet)?;
    if state.combat.is_some() && arm.is_some() && state.selected_warrior >= state.warriors.len() {
        // The session admits only a selected warrior inside the squad.
        return Err(
            "scripted combat movement or summon arm for an inactive selected warrior".into(),
        );
    }
    if all {
        state.clear_spell_targets();
        state.native_work.party_mask[..state.warriors.len()].fill(1);
    }
    let selected = if !all && state.combat.is_some() {
        Some(targets::select(state, content, map, &packet, id, count)?)
    } else {
        None
    };
    if let Some(combat) = state.combat.as_mut() {
        let mut mask = [0; 110];
        if all {
            mask[..state.warriors.len()].fill(1);
        } else {
            let (selected, target) = selected.ok_or("scripted tactical selection missing")?;
            mask = selected.mask;
            combat.spell_fields = selected.fields;
            combat.target_work.cell = target;
        }
        state.native_work.party_mask.copy_from_slice(&mask[..6]);
        combat.target_tail = mask[6..].to_vec();
        combat.attack_packet = packet.clone();
        combat.attack_identity = [id / 1000 - 1, id % 1000 / 100 - 1, id % 100 - 1];
        state.native_work.attack_count = count;
        combat.ready = 1;
        if let Some(arm) = arm {
            combat.mark_monster_spell_work(&packet, &mask);
            combat.inactive_rounds = 0;
            let result = combat::runtime::live_arm(
                combat,
                &mut state.warriors,
                (map, &mut state.random),
                (arm, state.selected_warrior),
                (id, &packet, count),
                &mut state.bandage_ready,
                (Some(caster), None),
                Some((&content.items, &mut state.native_work.ammunition_work)),
            )?;
            combat.ready = 0;
            if !all {
                // newland17's spelltargets (143af2, triple argument 0) writes -1 after resolvespell.
                combat.target_work.resolved(0);
            }
            return Ok(result);
        }
        let memory = combat::spells::RestrictionMemoryView::ordinary_startup(
            &map.item_cache,
            state
                .native_work
                .ammunition_work
                .item
                .as_ref()
                .map(|item| ItemDefinition(item).restriction_alias())
                .transpose()?,
        )
        .with_tables(map)
        .with_definitions(&content.items)
        .with_selected_warrior(state.selected_warrior as i16)
        .with_alias_owners(
            combat::warrior_alias::Owners::new(
                state.native_work.ammunition_work.item.as_deref(),
                &state.native_work.shop_work,
                state
                    .exploration
                    .as_ref()
                    .map(|position| position.event_headers.as_slice()),
            )
            .with_backup(&state.native_work.warrior_table),
        )
        // resist reads the body at 4470b8[447194] and WORD4435d2 or WORD447196.
        .with_live_caster(combat, &state.warriors)
        .with_live_resist_caster(
            combat,
            &state.warriors,
            (
                Some(state.selected_warrior as i16),
                Some(combat.selected_enemy),
            ),
        );
        let result = combat::runtime::resolve_spell_mask(
            combat,
            &mut state.warriors,
            &mut state.random,
            (id, &packet, count),
            &mask,
            caster,
            memory,
        )?;
        combat.ready = 0;
        if !all {
            combat.target_work.resolved(0);
        }
        state.commit_alias_writes()?;
        return Ok(result);
    }
    state.native_work.attack_identity = [id / 1000 - 1, id % 1000 / 100 - 1, id % 100 - 1];
    state.native_work.attack_count = count;
    state.set_ready(1);
    let outcome = resolve_outdoors(state, content, &packet, id, count)?;
    if outcome.flow == combat::runtime::TurnFlow::Terminated {
        return Ok(outcome.events);
    }
    // newland143afe..143b0c surrounds the outdoor resolver with Ready1/0.
    state.set_ready(0);
    Ok(outcome.events)
}
