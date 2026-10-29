//! Transactions for equipment.
use super::{
    collect_item, drop, item_definition, race_movement, remove_retained, unsupported, wear_retained,
};
use crate::{Event, Phase, RuleError, Session, combat, items};
use centurions_content::mission::OpeningMission;
use centurions_content::records::ItemDefinition;
use centurions_content::{Content, InventoryRecord};

pub(super) fn mission<'a>(
    state: &Session,
    content: &'a Content,
) -> Result<&'a OpeningMission, RuleError> {
    state
        .land(content)
        .ok_or_else(|| RuleError::Invalid("equipment mission missing".into()))
}

pub(super) fn admission(state: &Session, warrior: usize, needs_ap: bool) -> Result<(), RuleError> {
    if warrior >= state.warriors.len() {
        return Err("inventory warrior outside squad".into());
    }
    if !matches!(state.phase, Phase::Exploration | Phase::Combat)
        || state
            .exploration
            .as_ref()
            .is_some_and(|p| p.presentation.is_some())
    {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    if state.phase == Phase::Combat
        && state.combat.as_ref().and_then(combat::Combat::active_body) != Some(warrior as i16)
    {
        return Err(RuleError::Refused {
            feedback: vec![crate::feedback::NativeFeedback::Warning { index: 10 }],
        });
    }
    if state.phase == Phase::Combat
        && needs_ap
        && state.warriors[warrior].record().action_points()? < 1
    {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    Ok(())
}

pub(super) fn events(sounds: Vec<i16>, warnings: Vec<i16>) -> Vec<Event> {
    sounds
        .into_iter()
        .map(|resource_id| Event::Sound { resource_id })
        .chain(
            warnings
                .into_iter()
                .map(|warning| Event::CollectionRejected { warning }),
        )
        .collect()
}

/// moveicon's owned-item route collects into the destination before dropping the source.
pub fn transfer(
    state: &mut Session,
    content: &Content,
    from: usize,
    to: usize,
    slot: usize,
) -> Result<Vec<Event>, RuleError> {
    if state.phase == Phase::Combat
        && state.combat.as_ref().and_then(combat::Combat::active_body) != Some(from as i16)
    {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    admission(state, from, false)?;
    if from == to || to >= state.warriors.len() {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    let mission = mission(state, content)?;
    let item = state.warriors[from]
        .inventory()?
        .get(slot)
        .cloned()
        .ok_or("transfer slot missing")?;
    state.native_work.inventory_record = item.clone();
    state.load_item_work(content, item.id())?;
    if state.warriors[to].inventory()?.len() >= 30 {
        return Ok(vec![Event::Feedback(
            crate::feedback::NativeFeedback::Warning { index: 19 },
        )]);
    }
    let collected = if ItemDefinition(item_definition(content, item.id())?).category()? == 23 {
        crate::story_collection::collect_story_item(
            &state.warriors[to],
            &item,
            content,
            race_movement(&state.warriors[to], mission)?,
        )?
    } else {
        collect_item(&state.warriors[to], &item, content, mission)?
    };
    let mut effects = events(collected.sounds, collected.warnings);
    state.warriors[to] = collected.warrior;
    if collected.collected {
        drop(&mut state.warriors[from], slot, content, mission)?;
        effects.push(Event::ItemTransferred {
            from,
            to,
            item: item.id(),
        });
    }
    Ok(effects)
}

pub fn take_crate(
    state: &mut Session,
    content: &Content,
    slot: usize,
    warrior: usize,
) -> Result<Vec<Event>, RuleError> {
    admission(state, warrior, true)?;
    if state.phase != Phase::Exploration {
        return Err(unsupported("crate access in combat"));
    }
    let item = state
        .crate_items
        .get(slot)
        .ok_or_else(|| RuleError::Invalid("crate slot missing".into()))?;
    let item_id = item.id();
    let retained = item.clone();
    let prior_count = state.warriors[warrior].inventory()?.len();
    let result = collect_item(
        &state.warriors[warrior],
        item,
        content,
        mission(state, content)?,
    )?;
    state.warriors[warrior] = result.warrior;
    if result.collected {
        if state.warriors[warrior].inventory()?.len() > prior_count {
            state.native_work.inventory_record = retained;
        }
        state.crate_items[slot] = result.item;
    }
    let mut occupied: Vec<_> = state
        .crate_items
        .iter()
        .filter(|record| record.id() != 0)
        .cloned()
        .collect();
    occupied.resize(state.crate_items.len(), InventoryRecord([0; 22]));
    state.crate_items = occupied;
    state.load_item_work(content, item_id)?;
    Ok(events(result.sounds, result.warnings))
}

pub fn equip(
    state: &mut Session,
    content: &Content,
    warrior: usize,
    slot: usize,
) -> Result<Vec<Event>, RuleError> {
    admission(state, warrior, true)?;
    let item = state.warriors[warrior]
        .inventory()?
        .get(slot)
        .cloned()
        .ok_or_else(|| RuleError::Invalid("equipment slot missing".into()))?;
    let item_id = item.id();
    state.native_work.inventory_record = item.clone();
    let mut result = wear_retained(
        &state.warriors[warrior],
        content,
        mission(state, content)?,
        slot,
        state.phase == Phase::Combat,
        Some(&state.native_work.inventory_record),
    )?;
    state.apply_equipment_globals(&result.global_writes)?;
    if let Some(record) = result.record.take() {
        state.native_work.inventory_record = record;
    }
    if !result.changed {
        state.warriors[warrior] = result.warrior;
        state.load_item_work(content, item_id)?;
        return Ok(events(result.sounds, result.warnings));
    }
    items::compact(&mut result.warrior)?;
    state.warriors[warrior] = result.warrior;
    state.load_item_work(content, item_id)?;
    let selected = state.native_work.health_context.selected_warrior;
    let mut health =
        crate::story_health::retained_context(selected, &state.native_work.health_context);
    health.affected[warrior] = 1;
    state.native_work.health_context = health;
    Ok(events(result.sounds, result.warnings))
}

pub fn unequip(
    state: &mut Session,
    content: &Content,
    warrior: usize,
    slot: usize,
) -> Result<Vec<Event>, RuleError> {
    admission(state, warrior, false)?;
    let item = state.warriors[warrior]
        .inventory()?
        .get(slot)
        .cloned()
        .ok_or_else(|| RuleError::Invalid("equipment slot missing".into()))?;
    let item_id = item.id();
    state.native_work.inventory_record = item.clone();
    let mut result = remove_retained(
        &state.warriors[warrior],
        content,
        mission(state, content)?,
        slot,
        Some(&state.native_work.inventory_record),
    )?;
    state.apply_equipment_globals(&result.global_writes)?;
    if let Some(record) = result.record.take() {
        state.native_work.inventory_record = record;
    }
    if !result.changed {
        state.warriors[warrior] = result.warrior;
        state.load_item_work(content, item_id)?;
        return Ok(events(result.sounds, result.warnings));
    }
    items::compact(&mut result.warrior)?;
    state.warriors[warrior] = result.warrior;
    state.load_item_work(content, item_id)?;
    let health = &mut state.native_work.health_context;
    health.affected[warrior] = 1;
    Ok(events(result.sounds, result.warnings))
}
