//! Outdoor Items choice and its typed-return or inventory-loop continuation.
use super::{Selection, choice_limit, resolve};
use crate::combat::item_activation::{self, Caller};
use crate::{Content, Event, Phase, Presentation, RuleError, Session};
use centurions_content::records::AttackPacket;

pub(crate) fn begin_item(
    state: &mut Session,
    content: &Content,
    owner: usize,
    slot: usize,
) -> Result<Vec<Event>, RuleError> {
    if state.phase != Phase::Exploration
        || state.combat.is_some()
        || state.exploration.as_ref().is_none_or(|position| {
            (position.presentation.is_some() || position.script.is_some())
                && !matches!(position.presentation, Some(Presentation::Typed { .. }))
        })
    {
        return Err("item use requires free exploration".into());
    }
    crate::exploration::check_item_actor(state, owner)?;
    // The explicit inventory chooser owns BYTE44273f, independently of WORD4435d2.
    state.set_warrior_selector(owner);
    super::begin_item_selected(state, content, owner, slot)
}

/// Items12ae48 with a literal selected inventory owner and post-UI work.
pub(crate) fn begin_selected(
    state: &mut Session,
    content: &Content,
    owner: usize,
    slot: usize,
) -> Result<Vec<Event>, RuleError> {
    let caller =
        if state.exploration.as_ref().is_some_and(|position| {
            matches!(position.presentation, Some(Presentation::Typed { .. }))
        }) {
            Caller::Encounter
        } else {
            Caller::Ordinary
        };
    let activation = item_activation::prepare_selected(state, content, owner, slot, caller)?;
    if activation.spell == 0 {
        return Err(RuleError::RefusedAfterEffects { feedback: vec![] });
    }
    let packet = &state.native_work.attack_packet;
    let mode = AttackPacket(packet).targeting_mode();
    let power = state.native_work.attack_count;
    let targets = choice_limit(mode, power, state.warriors.len());
    state.outdoor_psi = Some(Selection {
        actor: owner,
        spell: activation.spell,
        power,
        remaining: targets,
        item: Some(activation.item),
    });
    let mut events = activation.events;
    if mode == 5 {
        state.native_work.party_mask[owner] = 1;
        events.extend(resolve(state, content)?);
    } else if (mode as i8) > 2 || targets >= state.warriors.len() as i16 {
        state.native_work.party_mask[..state.warriors.len()].fill(1);
        events.extend(resolve(state, content)?);
    } else {
        events.push(Event::Sound { resource_id: 6001 });
    }
    Ok(events)
}

pub(super) fn finish_return(
    state: &mut Session,
    content: &Content,
    pending: &Selection,
) -> Result<Vec<Event>, RuleError> {
    if state
        .exploration
        .as_ref()
        .is_some_and(|position| matches!(position.presentation, Some(Presentation::Typed { .. })))
    {
        item_activation::finish_return(state, pending.actor, Caller::Encounter);
        let mut events = vec![];
        if let Some(selected) = item_activation::encounter_return(
            state,
            content,
            pending.item.ok_or("outdoor item return missing")?,
        )? {
            events.extend(crate::exploration::finish_typed_selection(
                state, content, selected,
            )?);
        }
        Ok(events)
    } else {
        // 12b1ee refreshes the owner's information before Ready0/UI reentry.
        // This ordinary arm does not reach common RTS's marks reset at12c452.
        item_activation::refresh_inventory_info_work(state, content, pending.actor)?;
        // 12b202 clears WORD44719a with Ready.
        state.native_work.fire_item = 0;
        state.set_ready(0);
        Ok(vec![])
    }
}
