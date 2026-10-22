use crate::{
    Event, InventoryRecord, Presentation, RuleError, Session, equipment, feedback::NativeFeedback,
};
use centurions_content::records::{InventorySlots, InventorySlotsMut};
use centurions_content::{Content, Warrior};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfiscatedEquipment {
    pub active: bool,
    pub money: i32,
    pub items: Vec<InventoryRecord>,
}
impl ConfiscatedEquipment {
    pub(crate) fn validate(&self) -> Result<(), RuleError> {
        if self.items.len() != 600
            || self
                .items
                .iter()
                .any(|item| item.id().unsigned_abs() >= 1000)
        {
            return Err("invalid confiscated equipment records".into());
        }
        Ok(())
    }
}

fn flash(address: u32) -> Event {
    Event::Feedback(NativeFeedback::Flash {
        address,
        x: 50,
        y: 50,
        wait: 0,
        sound: 6000,
    })
}

/// newland opcode 36, 0x144ce8..0x144f90. The forward force-drop loop is intentional.
pub(crate) fn apply(
    state: &mut Session,
    content: &Content,
    parameter: i16,
) -> Result<Vec<Event>, RuleError> {
    let active = state.confiscated.as_ref().is_some_and(|s| s.active);
    if parameter != 0 && active || parameter == 0 && !active {
        return Ok(vec![]);
    }
    let mission = content.map(0).ok_or("confiscation scenario unavailable")?;
    if parameter != 0 {
        let mut saved = state.confiscated.take().unwrap_or(ConfiscatedEquipment {
            active: false,
            money: 0,
            items: vec![InventoryRecord([0; 22]); 600],
        });
        saved.active = true;
        saved.money = state.party_currency;
        state.party_currency = 0;
        for (member, warrior) in state.warriors.iter_mut().enumerate() {
            saved.money = saved
                .money
                .wrapping_add(i32::from(warrior.record().carried_credits()? as u16));
            warrior.record_mut().set_carried_credits(0);
            let records = warrior.inventory()?;
            saved.items[member * 100..member * 100 + records.len()].clone_from_slice(&records);
            for slot in 0..100 {
                equipment::force_drop(warrior, slot, content, mission)?;
            }
            InventorySlotsMut(&mut warrior.bytes).set_count(0);
        }
        state.load_item_work(content, 0)?;
        state.confiscated = Some(saved);
        return Ok(vec![flash(0x401c02)]);
    }
    let mut saved = state.confiscated.take().unwrap();
    let mut displaced = Vec::new();
    for (member, warrior) in state.warriors.iter_mut().enumerate() {
        InventorySlotsMut(&mut warrior.bytes).set_count(0);
        for slot in 0..100 {
            let record = &saved.items[member * 100 + slot];
            if record.id() == 0 {
                continue;
            }
            displaced.push(InventorySlots(&warrior.bytes).item_id(slot)?);
            let count = InventorySlots(&warrior.bytes).count()?.wrapping_add(1);
            InventorySlotsMut(&mut warrior.bytes).set_count(count);
            warrior.record_mut().set_inventory_slot(slot, record);
            if record.equipment_flag() != 0 {
                *warrior =
                    crate::editor_items::equipment_record_restored(warrior, content, slot)?.0;
            }
            saved.items[member * 100 + slot] = InventoryRecord([0; 22]);
        }
        equipment::calculate_weight(warrior, content)?;
        Warrior::read(&warrior.bytes)?;
    }
    saved.active = false;
    let mut events = vec![flash(0x401bdc)];
    if !displaced.is_empty() {
        events.push(flash(0x401bac));
        state.party_currency = state.party_currency.wrapping_add(saved.money);
        saved.money = 0;
        state.booty_items.fill(InventoryRecord([0; 22]));
        for (slot, id) in displaced.into_iter().take(180).enumerate() {
            if id != 0 {
                state.booty_items[slot] = crate::shop::preview(content, id)?;
            }
        }
        state
            .exploration
            .as_mut()
            .ok_or("no active confiscation story")?
            .presentation = Some(Presentation::Loot);
        events.push(Event::StoryPresented);
    }
    state.confiscated = Some(saved);
    Ok(events)
}
