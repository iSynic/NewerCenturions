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
    let mission = content
        .map_at(
            state.mission.as_deref(),
            state.scenario_level,
            state.exploration.as_ref().unwrap().map,
        )
        .ok_or_else(|| RuleError::Invalid("mission missing".into()))?;
    if state.psi_memory.is_none() {
        state.psi_memory = Some(crate::initial_psi_memory(&state.warriors)?);
    }
    let result = crate::combat::player_psi::cast_party(
        &state.warriors,
        state.psi_memory.as_ref().unwrap(),
        &state.party_buffs,
        mission,
        &mut state.random,
        state.psi_disabled,
        crate::combat::player_psi::PartyCastRequest {
            warrior,
            spell,
            power,
        },
    )?;
    if let Some(duration) = result.scenario_duration {
        crate::combat::party_buffs::extend_scenario_tail(state, duration)?;
    }
    state.warriors = result.warriors;
    let memory = result.memory;
    state.party_buffs = result.buffs;
    let packet = result.packet;
    state.attack_packet = Some(packet.clone());
    state.party_mask.fill(0);

    state.attack_identity = Some(result.identity);
    state.attack_count = result.count;
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
    state.party_mask[..state.warriors.len()].fill(1);
    state.psi_memory = Some(memory);
    let mut events = result.events;
    if packet[1] != 7 {
        events.extend(crate::story_spell::resolve_outdoors(
            state,
            &packet,
            spell,
            power,
            warrior,
            matches!(packet[1], 3 | 9) || (packet[1] == 0 && power > state.warriors.len() as i16),
        )?);
    }
    if packet[1] == 11 {
        events.push(Event::Sound {
            resource_id: centurions_content::records::AttackPacket(&packet).impact_sound()?,
        });
    }
    Ok(events)
}
