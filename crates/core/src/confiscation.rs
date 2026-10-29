use crate::{Event, InventoryRecord, RuleError, Session, equipment, feedback::NativeFeedback};
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
        let mut globals = vec![];
        for (member, warrior) in state.warriors.iter_mut().enumerate() {
            saved.money = saved
                .money
                .wrapping_add(i32::from(warrior.record().carried_credits()? as u16));
            warrior.record_mut().set_carried_credits(0);
            let records = warrior.inventory()?;
            saved.items[member * 100..member * 100 + records.len()].clone_from_slice(&records);
            for slot in 0..100 {
                globals.extend(equipment::force_drop(warrior, slot, content, mission)?);
            }
            InventorySlotsMut(&mut warrior.bytes).set_count(0);
        }
        state.apply_equipment_globals(&globals)?;
        state.load_item_work(content, 0)?;
        state.native_work.recovery_work.held_money = saved.money;
        state.confiscated = Some(saved);
        return Ok(vec![flash(0x401c02)]);
    }
    let mut saved = state.confiscated.take().unwrap();
    let mut displaced = Vec::new();
    let mut globals = vec![];
    for (member, warrior) in state.warriors.iter_mut().enumerate() {
        InventorySlotsMut(&mut warrior.bytes).set_count(0);
        for slot in 0..100 {
            let record = &saved.items[member * 100 + slot];
            if record.id() == 0 {
                continue;
            }
            let id = InventorySlots(&warrior.bytes).item_id(slot)?;
            state.native_work.recovery_work.displaced_items[displaced.len()] = id;
            displaced.push(id);
            let count = InventorySlots(&warrior.bytes).count()?.wrapping_add(1);
            InventorySlotsMut(&mut warrior.bytes).set_count(count);
            warrior.record_mut().set_inventory_slot(slot, record);
            if record.equipment_flag() != 0 {
                let (restored, _, writes) =
                    crate::editor_items::equipment_record_restored(warrior, content, slot)?;
                *warrior = restored;
                globals.extend(writes);
            }
            saved.items[member * 100 + slot] = InventoryRecord([0; 22]);
        }
        equipment::calculate_weight(warrior, content)?;
        Warrior::read(&warrior.bytes)?;
    }
    state.apply_equipment_globals(&globals)?;
    saved.active = false;
    state.native_work.recovery_work.held_money = saved.money;
    state.confiscated = Some(saved);
    let mut events = vec![flash(0x401bdc)];
    if !displaced.is_empty() {
        events.push(flash(0x401bac));
        let caller = crate::loot_recovery::RecoveryReturn::Confiscation {
            detached: state.combat.is_some(),
        };
        events.extend(crate::loot_recovery::begin(state, content, caller)?);
    }
    Ok(events)
}
