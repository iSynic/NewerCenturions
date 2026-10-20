//! Loot for maintained exploration.
use super::{continue_with_spells, needs_spell_confirmation};
use crate::{Event, Presentation, RuleError, Session, insert, items, story_collection};
use centurions_content::{Content, InventoryRecord, span};

pub fn take_loot(
    state: &mut Session,
    content: &Content,
    slot: usize,
    warrior: Option<usize>,
) -> Result<Vec<Event>, RuleError> {
    if !state
        .exploration
        .as_ref()
        .is_some_and(|position| matches!(position.presentation, Some(Presentation::Loot)))
    {
        return Err(RuleError::Invalid("no loot choice active".into()));
    }
    let item = state
        .booty_items
        .get(slot)
        .filter(|item| item.id() != 0)
        .ok_or_else(|| RuleError::Invalid("loot slot is empty".into()))?
        .clone();
    state.load_item_work(content, item.id())?;
    let mut effects = Vec::new();
    if (200..400).contains(&item.id()) && item.0[10] == 26 {
        effects = items::add_ammunition(state, content, &item)?;
    } else if let Some(index) = warrior {
        let recipient = state
            .warriors
            .get_mut(index)
            .ok_or_else(|| RuleError::Invalid("recipient outside squad".into()))?;
        let stacked =
            item.id() == 700 && recipient.inventory()?.iter().any(|owned| owned.id() == 700);
        let race = recipient.record().race()?.wrapping_sub(1);
        let movement = *content
            .map(state.exploration.as_ref().unwrap().map)
            .unwrap()
            .combat
            .as_ref()
            .unwrap()
            .race_moves
            .get(race as usize)
            .ok_or_else(|| RuleError::Invalid("recipient race unavailable".into()))?;
        let collected = if item.0[10] != 23 {
            crate::equipment::collect_item(
                recipient,
                &item,
                content,
                content
                    .map(state.exploration.as_ref().unwrap().map)
                    .unwrap(),
            )?
        } else {
            story_collection::collect_story_item(recipient, &item, content, movement)?
        };
        effects.extend(
            collected
                .sounds
                .into_iter()
                .map(|resource_id| Event::Sound { resource_id }),
        );
        effects.extend(
            collected
                .warnings
                .into_iter()
                .map(|warning| Event::CollectionRejected { warning }),
        );
        *recipient = collected.warrior;
        state.booty_items[slot] = collected.item;
        if !collected.collected {
            return Ok(effects);
        }
        if !stacked && let Some(last) = recipient.inventory()?.last() {
            let id = last.id();
            state.inventory_record = Some(item.clone());
            state.load_item_work(content, id)?;
        }
    } else {
        effects = insert(item, &mut state.crate_items, true)?;
        if effects.contains(&Event::StorageFull) {
            return Ok(effects);
        }
    }
    state.booty_items[slot].0[..2].fill(0);
    if let Some(limit) = &mut state.loot_limit
        && *limit != 0
    {
        *limit = limit.wrapping_sub(1);
        if *limit == 0 {
            state.booty_items.fill(InventoryRecord([0; 22]));
            if !needs_spell_confirmation(state) {
                effects.extend(continue_with_spells(state, content, false)?);
            }
        }
    }
    Ok(effects)
}

pub(super) fn present_treasure(
    state: &mut Session,
    content: &Content,
    id: i16,
) -> Result<Vec<Event>, RuleError> {
    if id < 0 {
        return Err("negative treasure record".into());
    }
    let position = state
        .exploration
        .as_ref()
        .ok_or("treasure mission unavailable")?;
    let map = state
        .map(content, position.map)
        .ok_or("treasure map unavailable")?;
    let record = span(&map.treasure, id as usize * 88, 88)?;
    let prepared = crate::story_treasure::prepare(
        &state.warriors,
        record,
        content,
        &mut state.random,
        state.difficulty,
    )?;
    state.warriors = prepared.warriors;
    state.party_currency = state.party_currency.wrapping_add(prepared.credits);
    if let Some(record) = prepared.inventory_record {
        state.inventory_record = Some(record);
    }
    state.booty_items.fill(InventoryRecord([0; 22]));
    for (slot, item) in prepared.items.into_iter().enumerate() {
        state.loaded_item_field32 = Some(state.ammunition_work.load_item(content, item.id())?);
        state.booty_items[slot] = item;
    }
    state.exploration.as_mut().unwrap().presentation = Some(Presentation::Loot);
    Ok(vec![Event::StoryPresented])
}
