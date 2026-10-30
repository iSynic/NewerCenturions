use crate::combat::spells::spell_identity;
use crate::{Content, Event, Phase, RuleError, Session, combat, story_spell};
use centurions_content::records::{ItemDefinition, SpellInfoRecord};
use serde::{Deserialize, Serialize};
mod ranks;

/// A psi spell or item being cast outdoors, waiting for its targets: who casts, which spell, its
/// power and how many targets remain, and the item when one casts it.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Selection {
    /// The warrior casting.
    pub actor: usize,
    /// The spell being cast.
    pub spell: i16,
    /// The spell's power.
    pub power: i16,
    /// Targets still to choose.
    pub remaining: i16,
    /// The item casting the spell, if an item does.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item: Option<i16>,
}

mod item;
pub(crate) use item::begin_item;
pub(crate) use item::begin_selected as begin_item_selected;

// The choice argument wraps before it is compared with the last party member.
/// The most targets the choice allows: with mode 0, the spell's power capped at the number of
/// members, otherwise one; the argument wraps before it is compared with the last member.
fn choice_limit(mode: u8, power: i16, members: usize) -> i16 {
    let last = members as i16 - 1;
    let argument = if mode == 0 { power.wrapping_sub(1) } else { 0 };
    if argument >= last {
        last + 1
    } else {
        argument.wrapping_add(1)
    }
}

/// Checks that a pending outdoor cast is consistent with the spell and party.
pub(crate) fn validate(state: &Session, content: &Content) -> Result<(), RuleError> {
    let Some(pending) = &state.outdoor_psi else {
        return Ok(());
    };
    let packet = &state.globals.spell_info;
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
            (SpellInfoRecord(packet).target_type() as i8) <= 2
        } else {
            matches!(SpellInfoRecord(packet).target_type(), 0 | 1)
        })
        || packet != &original
        || state.globals.attack_count != pending.power
        || state.globals.cast_identity != spell_identity(pending.spell)
    {
        return Err(RuleError::Invalid(
            "invalid outdoor Psi continuation".into(),
        ));
    }
    let limit = choice_limit(
        SpellInfoRecord(packet).target_type(),
        pending.power,
        state.warriors.len(),
    );
    if !ranks::valid(
        &state.globals.party_track,
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

/// Choosing a target gives it the next lower rank; deselecting closes the gap.
///
/// Original: getchoice
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
        &mut state.globals.party_track[..state.warriors.len()],
        &mut pending.remaining,
        warrior,
    );
    let mut events = vec![Event::Sound { resource_id: sound }];
    if pending.remaining == 0 {
        events.extend(resolve(state, content)?);
    }
    Ok(events)
}

/// Enter or Space finishes the choice only while `in_spell` is set.
///
/// Original: getchoice
pub fn can_finish(state: &Session) -> bool {
    state.outdoor_psi.is_some() && state.globals.in_spell != 0
}

/// Finishes the choice of targets and resolves the cast, refusing while no spell is in effect.
pub(crate) fn finish(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    if state.globals.in_spell == 0 {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    resolve(state, content)
}

/// Resolves the outdoor cast on the chosen targets and clears the pending selection.
fn resolve(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let pending = state
        .outdoor_psi
        .take()
        .ok_or("outdoor Psi selection missing")?;
    let packet = state.globals.spell_info.clone();
    let identity = state.globals.cast_identity;
    let mut events = vec![];
    if pending.item.is_none() {
        let memory = &mut state.globals.define_spells;
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

/// Cancelling outside combat keeps the Psi already spent; only combat refunds it.
///
/// Original: buttonchoice, getchoice
pub(crate) fn cancel(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    state.set_in_spell(0);
    if state
        .outdoor_psi
        .as_ref()
        .is_some_and(|pending| pending.item.is_some())
    {
        state.globals.party_track.fill(0);
        return resolve(state, content);
    }
    state
        .outdoor_psi
        .take()
        .ok_or("outdoor Psi selection missing")?;
    state.globals.party_track.fill(0);
    Ok(vec![])
}
