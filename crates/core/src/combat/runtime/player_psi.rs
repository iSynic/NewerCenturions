//! Psi cast by a warrior in combat.
use super::{begin_learned_heal, finish_player_action, select_medical_target};
use crate::combat::Combat;
use crate::combat::player_psi::CastContext;
use crate::combat::player_psi::CastRequest;
use crate::combat::spells::spell_level;
use crate::{Event, RuleError, Session};
use centurions_content::Content;

/// The active warrior casts a psi power at a chosen cell.
pub fn player_psi(
    state: &mut Session,
    content: &Content,
    spell: i16,
    power: i16,
    target: Option<(i16, i16)>,
    rotation: u8,
) -> Result<Vec<Event>, RuleError> {
    player_psi_target(state, content, spell, power, target, rotation, None)
}

/// The active warrior casts a psi power on a warrior of the party. A heal on a fallen warrior is
/// carried out as a learned heal.
pub fn player_psi_warrior(
    state: &mut Session,
    content: &Content,
    warrior: usize,
    spell: i16,
    power: i16,
) -> Result<Vec<Event>, RuleError> {
    if spell == 1101
        && state
            .warriors
            .get(warrior)
            .is_some_and(|target| (-9..=0).contains(&target.stamina().unwrap_or(-10)))
    {
        let mut events = begin_learned_heal(state, content, power)?;
        events.extend(select_medical_target(
            state,
            content,
            crate::MedicalTarget::Portrait { warrior },
        )?);
        return Ok(events);
    }
    let target = *state
        .combat
        .as_ref()
        .ok_or("Psi combat missing")?
        .squad
        .pos
        .get(warrior)
        .ok_or("Psi warrior target missing")?;
    player_psi_target(
        state,
        content,
        spell,
        power,
        Some(target),
        0,
        Some(warrior as i16),
    )
}

/// The active warrior casts a psi power: the cast is resolved with the party buffs, the choice is
/// remembered in the psi menu and the action ends.
///
/// Original: castspell
pub(super) fn player_psi_target(
    state: &mut Session,
    content: &Content,
    spell: i16,
    power: i16,
    target: Option<(i16, i16)>,
    rotation: u8,
    body: Option<i16>,
) -> Result<Vec<Event>, RuleError> {
    let actor = state
        .combat
        .as_ref()
        .and_then(Combat::active_body)
        .filter(|body| (0..6).contains(body))
        .ok_or_else(|| RuleError::Invalid("psi casting requires an active warrior".into()))?;
    let mission = state
        .land(content)
        .ok_or_else(|| RuleError::Invalid("mission missing".into()))?;
    crate::combat::player_psi::select_combat_caster(state);
    if let Some(combat) = state.combat.as_mut() {
        combat.target_picks.enter(0, 1);
    }
    let mut events = crate::combat::player_psi::cast_with_buffs(
        state.combat.as_mut().unwrap(),
        &mut state.warriors,
        mission,
        &mut state.random,
        &mut state.globals.define_spells,
        &mut state.party_condition,
        state.psi_disabled,
        CastRequest {
            rotation,
            spell,
            power,
            target,
        },
        CastContext {
            movement_gate: state.bandage_ready,
            body,
            animation_delay: state.command_preferences.animation_delay,
            item_restriction: crate::combat::spells::loaded_item_restriction(
                &state.globals.loaded_items,
            )?,
            owners: crate::combat::warrior_alias::backup_owners!(state),
            land: crate::combat::map_alias::session_map_aliases!(state),
            refresh: Some((&content.items, &mut state.globals.loaded_items)),
        },
    )?;
    state.commit_alias_writes()?;
    // The target marks clear when the cast starts and again when it ends.
    if let Some(combat) = state.combat.as_mut() {
        combat.clear_track_tail();
        combat.target_picks.leave();
    }
    if matches!(
        centurions_content::records::SpellInfoRecord(&state.combat.as_ref().unwrap().spell_info)
            .effect()?,
        55 | 56
    ) {
        state.bandage_ready = 0;
    }
    state.globals.attack_count = power;
    // The warrior's remembered spell power is forgotten.
    state.globals.old_power_level[actor as usize] = None;
    state.psi_menu_choices[actor as usize] = Some(crate::PsiMenuChoice {
        row: spell_level(spell) as u8,
        column: (spell % 100 - 1) as u8,
        power,
    });
    events.extend(finish_player_action(state, content, actor)?);
    Ok(events)
}
