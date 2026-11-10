//! Psi cast by the party outside the tactical map.
use crate::combat::player_psi::PartyCastRequest;
use crate::{Event, Phase, RuleError, Session};
use centurions_content::Content;

/// A warrior casts a psi power of the party on the land map outside a battle. The power is
/// prepared, the warrior's menu choice is remembered and, when targets must still be chosen, the
/// cast waits for them; otherwise it is resolved on the party.
///
/// Original: castspell
pub fn party_psi(
    state: &mut Session,
    content: &Content,
    warrior: usize,
    spell: i16,
    power: i16,
) -> Result<Vec<Event>, RuleError> {
    require_field_turn(state)?;
    let mission = state
        .land(content)
        .ok_or_else(|| RuleError::Invalid("mission missing".into()))?;
    let result = crate::combat::player_psi::prepare_party(
        &state.warriors,
        &state.globals.define_spells,
        &state.party_condition,
        mission,
        state.psi_disabled,
        PartyCastRequest {
            warrior,
            spell,
            power,
        },
    )?;
    // The menu cast marks a spell in effect; a global (mode 7) spell clears it again.
    state.globals.in_spell = 1;
    begin_cast(state, content, warrior, spell, power, result)
}

/// Fails unless the party stands on the land map with nothing else under way.
pub(super) fn require_field_turn(state: &Session) -> Result<(), RuleError> {
    if state.phase != Phase::Exploration
        || state.combat.is_some()
        || state
            .exploration
            .as_ref()
            .is_none_or(|position| position.presentation.is_some() || position.script.is_some())
    {
        return Err(RuleError::Invalid(
            "outdoor psi requires an unobstructed exploration turn".into(),
        ));
    }
    Ok(())
}

/// Starts a paid cast: the warrior's choice is remembered and, when targets must still be chosen,
/// the cast waits for them; otherwise it is resolved on the party, which ends the marks of a memory
/// spell and a paid fast spell.
///
/// Original: buttonchoice
pub(super) fn begin_cast(
    state: &mut Session,
    content: &Content,
    warrior: usize,
    spell: i16,
    power: i16,
    result: crate::combat::player_psi::PartyCastResult,
) -> Result<Vec<Event>, RuleError> {
    if let Some(duration) = result.scenario_duration {
        crate::combat::party_buffs::extend_scenario_tail(state, duration)?;
    }
    state.warriors = result.warriors;
    let memory = result.memory;
    state.party_condition = result.buffs;
    let packet = result.packet;
    state.globals.spell_info = packet.clone();
    state.clear_targets();

    state.globals.cast_identity = result.identity;
    state.globals.attack_count = result.count;
    // The warrior's remembered spell power is forgotten.
    state.globals.old_power_level[warrior] = None;
    state.psi_menu_choices[warrior] = Some(crate::PsiMenuChoice {
        row: result.identity[1] as u8,
        column: result.identity[2] as u8,
        power,
    });
    let selection_limit = if packet[1] == 0 { power } else { 1 };
    if matches!(packet[1], 0 | 1) && selection_limit < state.warriors.len() as i16 {
        state.outdoor_psi = Some(crate::outdoor_psi::Selection {
            actor: warrior,
            spell,
            power,
            item: None,
            remaining: if packet[1] == 0 {
                power.min(state.warriors.len() as i16)
            } else {
                1
            },
        });
        return Ok(vec![]);
    }
    state.globals.party_track[..state.warriors.len()].fill(1);
    state.globals.define_spells = memory;
    let outcome =
        crate::story::spell::resolve_party_outdoors(state, content, &packet, spell, power)?;
    state.globals.memory_spell = 0;
    state.globals.fast_spell = 0;
    Ok(outcome.events)
}
