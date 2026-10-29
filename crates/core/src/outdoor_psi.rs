use crate::{Content, Event, Phase, RuleError, Session, combat, story_spell};
use centurions_content::records::{AttackPacket, ItemDefinition};
use serde::{Deserialize, Serialize};
mod ranks;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Selection {
    pub actor: usize,
    pub spell: i16,
    pub power: i16,
    pub remaining: i16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item: Option<i16>,
}

mod item;
pub(crate) use item::begin_item;
pub(crate) use item::begin_selected as begin_item_selected;

// Items12b0e4..12b0f6 compares the wrapped choice argument with partyLast.
fn choice_limit(mode: u8, power: i16, members: usize) -> i16 {
    let last = members as i16 - 1;
    let argument = if mode == 0 { power.wrapping_sub(1) } else { 0 };
    if argument >= last {
        last + 1
    } else {
        argument.wrapping_add(1)
    }
}

pub(crate) fn validate(state: &Session, content: &Content) -> Result<(), RuleError> {
    let Some(pending) = &state.outdoor_psi else {
        return Ok(());
    };
    let packet = &state.native_work.attack_packet;
    centurions_content::span(packet, 0, 58)?;
    let map = state
        .map(
            content,
            state
                .exploration
                .as_ref()
                .ok_or("outdoor Psi position missing")?
                .map,
        )
        .ok_or("outdoor Psi content missing")?;
    let original = combat::spells::load_packet(
        &map.combat
            .as_ref()
            .ok_or("outdoor Psi tables missing")?
            .attacks,
        pending.spell,
    )?;
    if state.phase != Phase::Exploration
        || state.combat.is_some()
        || state.exploration.as_ref().is_none_or(|p| {
            (p.presentation.is_some() || p.script.is_some())
                && !(pending.item.is_some()
                    && matches!(p.presentation, Some(crate::Presentation::Typed { .. })))
        })
        || state.medical_continuation.is_some()
        || state.psi_selection.is_some()
        || state.promotion.is_some()
        || pending.actor >= state.warriors.len()
        || pending.item.is_some_and(|item| {
            !crate::ammunition::definition(content, item).is_ok_and(|definition| {
                ItemDefinition(&definition).spell().ok() == Some(pending.spell)
                    && ItemDefinition(&definition)
                        .effect_selector()
                        .map(i16::wrapping_abs)
                        .ok()
                        .is_some_and(|value| {
                            if value == 8 {
                                (1..=7).contains(&pending.power)
                            } else {
                                value == pending.power
                            }
                        })
            })
        })
        || (pending.item.is_none() && !(1..=7).contains(&pending.power))
        || (pending.item.is_none()
            && !combat::player_psi::OUTDOOR_TARGET_SPELLS.contains(&pending.spell))
        || !(if pending.item.is_some() {
            (AttackPacket(packet).targeting_mode() as i8) <= 2
        } else {
            matches!(AttackPacket(packet).targeting_mode(), 0 | 1)
        })
        || packet != &original
        || state.native_work.attack_count != pending.power
        || state.native_work.attack_identity
            != [
                pending.spell / 1000 - 1,
                pending.spell % 1000 / 100 - 1,
                pending.spell % 100 - 1,
            ]
    {
        return Err(RuleError::Invalid(
            "invalid outdoor Psi continuation".into(),
        ));
    }
    let limit = choice_limit(
        AttackPacket(packet).targeting_mode(),
        pending.power,
        state.warriors.len(),
    );
    if !ranks::valid(
        &state.native_work.party_mask,
        state.warriors.len(),
        limit,
        pending.remaining,
    ) {
        return Err(RuleError::Invalid(
            "invalid outdoor Psi selection ranks".into(),
        ));
    }
    Ok(())
}

/// getchoice12535e assigns descending ranks; deselection closes the rank gap.
pub(crate) fn select(
    state: &mut Session,
    content: &Content,
    warrior: usize,
) -> Result<Vec<Event>, RuleError> {
    validate(state, content)?;
    if warrior >= state.warriors.len() {
        return Err(RuleError::Invalid("outdoor Psi target missing".into()));
    }
    let pending = state
        .outdoor_psi
        .as_mut()
        .ok_or("outdoor Psi selection missing")?;
    let sound = ranks::toggle(
        &mut state.native_work.party_mask[..state.warriors.len()],
        &mut pending.remaining,
        warrior,
    );
    let mut events = vec![Event::Sound { resource_id: sound }];
    if pending.remaining == 0 {
        events.extend(resolve(state, content)?);
    }
    Ok(events)
}

/// getchoice125264 accepts Enter/Space only while the shared Ready WORD is nonzero.
pub fn can_finish(state: &Session) -> bool {
    state.outdoor_psi.is_some() && state.native_work.ready != 0
}

pub(crate) fn finish(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    if state.native_work.ready == 0 {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    resolve(state, content)
}

fn resolve(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let pending = state
        .outdoor_psi
        .take()
        .ok_or("outdoor Psi selection missing")?;
    let packet = state.native_work.attack_packet.clone();
    let identity = state.native_work.attack_identity;
    let mut events = vec![];
    if pending.item.is_none() {
        let memory = &mut state.native_work.psi_memory;
        for (index, value) in identity.into_iter().chain([pending.power]).enumerate() {
            let offset = pending.actor * 80 + 72 + index * 2;
            memory[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
        }
        return Ok(story_spell::resolve_party_outdoors(
            state,
            content,
            &packet,
            pending.spell,
            pending.power,
        )?
        .events);
    }
    let outcome =
        story_spell::resolve_outdoors(state, content, &packet, pending.spell, pending.power)?;
    events.extend(outcome.events);
    if outcome.flow == combat::runtime::TurnFlow::Terminated {
        return Ok(events);
    }
    events.extend(item::finish_return(state, content, &pending)?);
    Ok(events)
}

/// buttonchoice10ad22 returns after the charge, without the combat cancellation refund.
pub(crate) fn cancel(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    // buttonchoice10ae52 and getchoice12524c.
    state.set_ready(0);
    if state
        .outdoor_psi
        .as_ref()
        .is_some_and(|pending| pending.item.is_some())
    {
        state.native_work.party_mask.fill(0);
        return resolve(state, content);
    }
    state
        .outdoor_psi
        .take()
        .ok_or("outdoor Psi selection missing")?;
    state.native_work.party_mask.fill(0);
    Ok(vec![])
}
