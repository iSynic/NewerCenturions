//! Party psi in the original combat execution order.
use crate::{Event, Phase, RuleError, Session};
use centurions_content::Content;

pub fn party_psi(
    state: &mut Session,
    content: &Content,
    warrior: usize,
    spell: i16,
    power: i16,
) -> Result<Vec<Event>, RuleError> {
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
    let mission = state
        .land(content)
        .ok_or_else(|| RuleError::Invalid("mission missing".into()))?;
    let result = crate::combat::player_psi::prepare_party(
        &state.warriors,
        &state.native_work.psi_memory,
        &state.party_buffs,
        mission,
        state.psi_disabled,
        crate::combat::player_psi::PartyCastRequest {
            warrior,
            spell,
            power,
        },
    )?;
    // castspell10b9bc; mode-seven resolvespell1512b8/151316 clears it.
    state.native_work.ready = 1;
    if let Some(duration) = result.scenario_duration {
        crate::combat::party_buffs::extend_scenario_tail(state, duration)?;
    }
    state.warriors = result.warriors;
    let memory = result.memory;
    state.party_buffs = result.buffs;
    let packet = result.packet;
    state.native_work.attack_packet = packet.clone();
    // buttonchoice 10ac9a calls cleartarget after castspell returns.
    state.clear_targets();

    state.native_work.attack_identity = result.identity;
    state.native_work.attack_count = result.count;
    // castspell 0x10d328 stores the cast power in WORD 0x437a7a + 2 * member.
    state.native_work.psi_powers[warrior] = None;
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
    state.native_work.party_mask[..state.warriors.len()].fill(1);
    state.native_work.psi_memory = memory;
    let outcome =
        crate::story_spell::resolve_party_outdoors(state, content, &packet, spell, power)?;
    Ok(outcome.events)
}
