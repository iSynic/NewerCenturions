use crate::{Content, Event, Phase, RuleError, Session, combat, story_spell};
use centurions_content::word;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Selection {
    pub actor: usize,
    pub spell: i16,
    pub power: i16,
    pub remaining: i16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item: Option<i16>,
}

/// Items12ae48 charges its item before getchoice, including cancellation.
pub(crate) fn begin_item(
    state: &mut Session,
    content: &Content,
    actor: usize,
    slot: usize,
) -> Result<Vec<Event>, RuleError> {
    if state.phase != Phase::Exploration
        || state.combat.is_some()
        || state.exploration.as_ref().is_none_or(|p| {
            (p.presentation.is_some() || p.script.is_some())
                && !matches!(p.presentation, Some(crate::Presentation::Typed { .. }))
        })
    {
        return Err(RuleError::Invalid(
            "item use requires free exploration".into(),
        ));
    }
    let selected = state
        .warriors
        .get(actor)
        .ok_or("item warrior missing")?
        .inventory()?
        .get(slot)
        .cloned()
        .ok_or("inventory item missing")?;
    let definition = crate::ammunition::definition(content, selected.id())?;
    if word(&definition, 26)? != 28 || selected.0[9] != 0 {
        return Err(RuleError::Unsupported {
            action: "item.use",
            dependency: "other outdoor item context",
        });
    }
    let map = crate::exploration::active_map(state, content)?;
    let encounter = matches!(
        state.exploration.as_ref().unwrap().presentation,
        Some(crate::Presentation::Typed { .. })
    );
    if !encounter && !crate::equipment::can_use(&state.warriors[actor], &definition, map)? {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    let spell = word(&definition, 44)?;
    let power = word(&definition, 42)?.wrapping_abs();
    let packet = combat::spells::load_packet(
        &map.combat
            .as_ref()
            .ok_or("medical attack content missing")?
            .attacks,
        spell,
    )?;
    if !(spell == 1401 && packet[1] == 1)
        && !(packet[1] == 7 && matches!(word(&packet, 40)?, 0..=9 | 110))
    {
        return Err(RuleError::Unsupported {
            action: "item.use",
            dependency: "other outdoor item targeting context",
        });
    }
    if !encounter && !combat::player_psi::mode_allowed(&packet, false)? {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    let quantity = word(&selected.0, 2)?;
    if quantity <= 0 {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    crate::items::compact(&mut state.warriors[actor])?;
    if state.warriors[actor].inventory()?.get(slot) != Some(&selected) {
        return Err(RuleError::Unsupported {
            action: "item.use",
            dependency: "inventory selection moved during item pane initialization",
        });
    }
    crate::equipment::calculate_weight(&mut state.warriors[actor], content)?;
    for warrior in &mut state.warriors {
        warrior.bytes[0x60..0x62].fill(0);
    }
    let warrior = &mut state.warriors[actor];
    warrior.bytes[0x132 + slot * 22 + 2..0x132 + slot * 22 + 4]
        .copy_from_slice(&quantity.wrapping_sub(1).to_be_bytes());
    let weight = word(&warrior.bytes, 0x9d6)?.wrapping_sub(word(&definition, 36)?);
    warrior.bytes[0x9d6..0x9d8].copy_from_slice(&weight.to_be_bytes());
    let mut events = vec![];
    if quantity == 1 && word(&definition, 38)? != 0 {
        crate::items::remove_inventory_slot(warrior, slot)?;
        events.push(Event::Sound { resource_id: 655 });
    }
    state.selected_warrior = actor;
    state.load_item_work(content, selected.id())?;
    let global = packet[1] == 7;
    state.attack_packet = Some(packet);
    state.attack_identity = Some([spell / 1000 - 1, spell % 1000 / 100 - 1, spell % 100 - 1]);
    state.attack_count = power;
    state.party_mask.fill(0);
    state.outdoor_psi = Some(Selection {
        actor,
        spell,
        power,
        remaining: 1,
        item: Some(selected.id()),
    });
    if global || state.warriors.len() == 1 {
        state.party_mask[..if global { state.warriors.len() } else { 1 }].fill(1);
        events.extend(finish(state, content)?);
    } else {
        events.push(Event::Sound { resource_id: 6001 });
    }
    Ok(events)
}

pub(crate) fn validate(state: &Session, content: &Content) -> Result<(), RuleError> {
    let Some(pending) = &state.outdoor_psi else {
        return Ok(());
    };
    let packet = state
        .attack_packet
        .as_ref()
        .ok_or("outdoor Psi packet missing")?;
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
                word(&definition, 26).ok() == Some(28)
                    && word(&definition, 44).ok() == Some(pending.spell)
                    && word(&definition, 42).map(i16::wrapping_abs).ok() == Some(pending.power)
            })
        })
        || !(1..=7).contains(&pending.power)
        || !combat::player_psi::OUTDOOR_TARGET_SPELLS.contains(&pending.spell)
        || !matches!(packet[1], 0 | 1)
        || packet != &original
        || state.attack_count != pending.power
        || state.attack_identity
            != Some([
                pending.spell / 1000 - 1,
                pending.spell % 1000 / 100 - 1,
                pending.spell % 100 - 1,
            ])
    {
        return Err(RuleError::Invalid(
            "invalid outdoor Psi continuation".into(),
        ));
    }
    let limit = if packet[1] == 0 {
        pending.power.min(state.warriors.len() as i16)
    } else {
        1
    };
    if limit >= state.warriors.len() as i16
        || !(1..=limit).contains(&pending.remaining)
        || state.party_mask[state.warriors.len()..]
            .iter()
            .any(|rank| *rank != 0)
        || state
            .party_mask
            .iter()
            .any(|rank| *rank < 0 || *rank > limit)
        || (pending.remaining + 1..=limit)
            .any(|rank| state.party_mask.iter().filter(|r| **r == rank).count() != 1)
        || state
            .party_mask
            .iter()
            .any(|rank| *rank > 0 && *rank <= pending.remaining)
    {
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
    let old = state.party_mask[warrior];
    let sound = if old == 0 {
        state.party_mask[warrior] = pending.remaining;
        pending.remaining -= 1;
        141
    } else {
        state.party_mask[warrior] = 0;
        for rank in &mut state.party_mask {
            if *rank > 0 && *rank < old {
                *rank += 1;
            }
        }
        pending.remaining += 1;
        144
    };
    let mut events = vec![Event::Sound { resource_id: sound }];
    if pending.remaining == 0 {
        events.extend(finish(state, content)?);
    }
    Ok(events)
}

pub(crate) fn finish(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    if !state.party_mask.iter().any(|rank| *rank > 0)
        && state
            .outdoor_psi
            .as_ref()
            .is_none_or(|pending| pending.item.is_none())
    {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    let pending = state
        .outdoor_psi
        .take()
        .ok_or("outdoor Psi selection missing")?;
    let packet = state
        .attack_packet
        .clone()
        .ok_or("outdoor Psi packet missing")?;
    let identity = state
        .attack_identity
        .ok_or("outdoor Psi identity missing")?;
    let mut events = vec![];
    if pending.item.is_none() {
        let memory = state
            .psi_memory
            .as_mut()
            .ok_or("outdoor Psi memory missing")?;
        for (index, value) in identity.into_iter().chain([pending.power]).enumerate() {
            let offset = pending.actor * 80 + 72 + index * 2;
            memory[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
        }
        events.push(Event::Sound {
            resource_id: word(&packet, 36)?.wrapping_neg(),
        });
    }
    events.extend(story_spell::resolve_outdoors(
        state,
        &packet,
        pending.spell,
        pending.power,
        pending.actor,
        false,
    )?);
    if state
        .health_context
        .as_ref()
        .is_some_and(|h| h.unavailable >= state.warriors.len() as i8)
    {
        events.extend(crate::mission::defeat(state, content));
    }
    if let Some(item) = pending.item
        && state
            .exploration
            .as_ref()
            .is_some_and(|p| matches!(p.presentation, Some(crate::Presentation::Typed { .. })))
    {
        // Items12c422 adds its owner after resolving the item; encounter2 matches the ID.
        state.party_mask[pending.actor] = 1;
        let selected = crate::typed_encounter::select_item(
            crate::exploration::typed_record(state, content)?,
            item,
        )?;
        events.extend(crate::exploration::finish_typed_selection(
            state, content, selected,
        )?);
    }
    Ok(events)
}

/// buttonchoice10ad22 returns after the charge, without the combat cancellation refund.
pub(crate) fn cancel(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    if state
        .outdoor_psi
        .as_ref()
        .is_some_and(|pending| pending.item.is_some())
    {
        state.party_mask.fill(0);
        return finish(state, content);
    }
    state
        .outdoor_psi
        .take()
        .ok_or("outdoor Psi selection missing")?;
    state.party_mask.fill(0);
    Ok(vec![])
}
