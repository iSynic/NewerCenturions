//! Transactions for shop.
use super::access::current_work;
use super::{active, enhanced_stock, preview, price, refused};
use crate::{Event, Presentation, RuleError, Session, equipment};
use centurions_content::Content;
use centurions_content::records::{ItemDefinition, SHOP_SLOTS};

pub(crate) fn buy(
    state: &mut Session,
    content: &Content,
    warrior: usize,
    slot: usize,
) -> Result<Vec<Event>, RuleError> {
    active(state)?;
    let access = state.shop_access.as_ref().unwrap();
    if access.enhancement != 0 && access.remaining == 0 {
        return Err(refused(159));
    }
    let work = current_work(state)?;
    if slot >= SHOP_SLOTS || work.quantity(slot)? == 0 {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    let id = work.item(slot)?;
    let cost = price(state, content, id, false)?;
    let item = preview(content, id)?;
    let recipient = state
        .warriors
        .get(warrior)
        .ok_or("shop recipient unavailable")?;
    let definition = &content.items[id.unsigned_abs() as usize];
    let weight = ItemDefinition(definition).base_weight()?.wrapping_add(
        ItemDefinition(definition)
            .unit_weight()?
            .wrapping_mul(item.quantity()?),
    );
    if recipient.inventory()?.len() >= 30
        || recipient.record().carried_weight()?.wrapping_add(weight) as u16
            > recipient.record().weight_capacity()? as u16
    {
        return Err(refused(19));
    }
    if cost > state.party_currency {
        return Err(refused(37));
    }
    state.load_item_work(content, id)?;
    let mut events = vec![Event::Sound { resource_id: 83 }];
    if ItemDefinition(definition).reserve_ammunition_kind()? != 0 {
        let reserve = id
            .checked_sub(200)
            .filter(|i| (0..200).contains(i))
            .ok_or("shop ammunition identifier outside reserves")? as usize;
        state.ammunition[reserve] = state.ammunition[reserve]
            .wrapping_add(i32::from(ItemDefinition(definition).supplied_quantity()?));
        let sound = if ItemDefinition(definition).pickup_sound()? != 0 {
            ItemDefinition(definition).pickup_sound()?
        } else {
            state
                .random
                .bound(2)
                .wrapping_add(crate::feedback::CLIP_SOUND_BASE)
        };
        events.push(Event::Sound { resource_id: sound });
        events.push(Event::Feedback(crate::feedback::NativeFeedback::Flash {
            address: 0x401d9e,
            x: 50,
            y: 50,
            wait: 30,
            sound: 0,
        }));
    } else {
        // moveicon calculates movement before installitem updates the load field.
        calculate_movement(state, content, warrior)?;
        let inventory = state.warriors[warrior].inventory()?;
        let count = inventory.len();
        let merged = if ItemDefinition(definition).unit_weight()? != 0 {
            inventory.iter().position(|i| i.id() == id)
        } else {
            None
        };
        let item = if merged.is_none() && state.shop_access.as_ref().unwrap().enhancement != 0 {
            enhanced_stock(state, content, id)?
        } else {
            item
        };
        let record = &mut state.warriors[warrior];
        if let Some(slot) = merged {
            let quantity = record
                .record()
                .inventory_quantity(slot)?
                .wrapping_add(ItemDefinition(definition).supplied_quantity()?);
            record.record_mut().set_inventory_quantity(slot, quantity);
        } else {
            record.record_mut().append_inventory(count, &item);
        }
        equipment::calculate_weight(&mut state.warriors[warrior], content)?;
    }
    state.party_currency = state.party_currency.wrapping_sub(cost);
    if let Some(access) = &mut state.shop_access
        && access.remaining != 0
    {
        access.remaining = access.remaining.wrapping_sub(1);
    }
    state.native_work.shop_work.buy_one(slot)?;
    Ok(events)
}

pub(super) fn calculate_movement(
    state: &mut Session,
    content: &Content,
    warrior: usize,
) -> Result<(), RuleError> {
    let record = &mut state.warriors[warrior];
    let race = record.record().race()?.wrapping_sub(1) as usize;
    let movement = content
        .map(0)
        .and_then(|m| m.combat.as_ref())
        .and_then(|c| c.race_moves.get(race))
        .copied()
        .ok_or("shop race movement unavailable")?;
    crate::combat::calculate_movement(record, movement)
}

pub(crate) fn sell(
    state: &mut Session,
    content: &Content,
    warrior: usize,
    slot: usize,
) -> Result<Vec<Event>, RuleError> {
    active(state)?;
    let access = state.shop_access.as_ref().unwrap();
    if access.enhancement != 0 || access.remaining != 0 {
        return Err(refused(158));
    }
    let item = state
        .warriors
        .get(warrior)
        .ok_or("shop warrior unavailable")?
        .inventory()?
        .get(slot)
        .cloned()
        .ok_or("shop item unavailable")?;
    let cost = price(state, content, item.id(), true)?;
    let mission = content.map(0).ok_or("shop scenario unavailable")?;
    let globals = equipment::drop(&mut state.warriors[warrior], slot, content, mission)?;
    state.apply_equipment_globals(&globals)?;
    state.native_work.shop_work.accept_sale(item.id())?;
    state.party_currency = state.party_currency.wrapping_add(cost);
    Ok(vec![Event::Sound { resource_id: 83 }])
}

pub(crate) fn reload(
    state: &mut Session,
    content: &Content,
    warrior: usize,
) -> Result<Vec<Event>, RuleError> {
    active(state)?;
    state.exploration.as_mut().unwrap().presentation = None;
    let result = crate::combat::firearm::reload_command(state, content, warrior);
    state.exploration.as_mut().unwrap().presentation = Some(Presentation::Shop);
    result
}
