mod outdoors;
mod targets;
use crate::combat::spells::spell_identity;
use crate::{Content, Event, RuleError, Session, combat};
use centurions_content::records::{ItemDefinition, ScriptedSpellExtra, SpellInfoRecordMut};
pub(crate) use outdoors::{party as resolve_party_outdoors, resolve as resolve_outdoors};

/// Selected Spell keeps the existing picks; Party Spell replaces them.
///
/// Original: newland
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
    SpellInfoRecordMut(&mut packet).add_save_bonus(extra.save_bonus()?);
    if extra.overrides_saving_throw()? {
        SpellInfoRecordMut(&mut packet).set_saving_throw_mode(3);
    }
    // Movement and summon spells use the target cell. Selected Spell sets it to
    // its script target; Party Spell keeps whatever cell was set before.
    let arm = combat::spells::out_of_combat::arm_of(&packet)?;
    if state.combat.is_some() && arm.is_some() && state.char_select_new >= state.warriors.len() {
        // The session admits only a selected warrior inside the squad.
        return Err(
            "scripted combat movement or summon arm for an inactive selected warrior".into(),
        );
    }
    if all {
        state.clear_spell_targets();
        state.globals.party_track[..state.warriors.len()].fill(1);
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
            combat.spell_queue = selected.fields;
            combat.target_picks.cell = target;
        }
        state.globals.party_track.copy_from_slice(&mask[..6]);
        combat.track_tail = mask[6..].to_vec();
        combat.spell_info = packet.clone();
        combat.cast_identity = spell_identity(id);
        state.globals.attack_count = count;
        combat.in_spell = 1;
        if let Some(arm) = arm {
            combat.mark_monster_spell_pick(&packet, &mask);
            combat.blank_round = 0;
            let result = combat::runtime::live_arm(
                combat,
                &mut state.warriors,
                (map, &mut state.random),
                (arm, state.char_select_new),
                (id, &packet, count),
                &mut state.bandage_ready,
                (Some(caster), None),
                Some((&content.items, &mut state.globals.loaded_items)),
            )?;
            combat.in_spell = 0;
            if !all {
                // Selected Spell clears its target after resolvespell.
                combat.target_picks.resolved(0);
            }
            return Ok(result);
        }
        let memory = combat::spells::RestrictionMemoryView::ordinary_startup(
            &map.item_cache,
            state
                .globals
                .loaded_items
                .item
                .as_ref()
                .map(|item| ItemDefinition(item).restriction_alias())
                .transpose()?,
        )
        .with_tables(map)
        .with_definitions(&content.items)
        .with_char_up(state.char_select_new as i16)
        .with_alias_owners(crate::combat::warrior_alias::backup_owners!(
            state,
            state.globals.loaded_items.item.as_deref()
        ))
        // resist reads the battle order entry at `up` and `char_up` or
        // `monster_up`.
        .with_live_caster(combat, &state.warriors)
        .with_live_resist_caster(
            combat,
            &state.warriors,
            (Some(state.char_select_new as i16), Some(combat.monster_up)),
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
        combat.in_spell = 0;
        if !all {
            combat.target_picks.resolved(0);
        }
        state.commit_alias_writes()?;
        return Ok(result);
    }
    state.globals.cast_identity = spell_identity(id);
    state.globals.attack_count = count;
    state.set_in_spell(1);
    let outcome = resolve_outdoors(state, content, &packet, id, count)?;
    if outcome.flow == combat::runtime::TurnFlow::Terminated {
        return Ok(outcome.events);
    }
    // `in_spell` is set only while the outdoor spell resolves.
    state.set_in_spell(0);
    Ok(outcome.events)
}
