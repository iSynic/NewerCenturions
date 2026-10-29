//! Booty's retained array writers before refit and before bodycount.
use crate::{RuleError, Session, story_inventory};
use centurions_content::{InventoryRecord, records::SpellFieldMut};

/// Booty tests BYTE4386fc after its mode10 branch and before normal awards.
pub(crate) fn effective_mode(state: &Session, requested: i16) -> Result<i16, RuleError> {
    if matches!(requested, 2 | 10) {
        return Ok(requested);
    }
    Ok(if state.native_work.early_booty == 0 {
        requested
    } else {
        2
    })
}

/// killbody tests this predicate after its immediate story returns.
pub(crate) fn after_death(state: &mut Session) -> Result<(), RuleError> {
    let mut unavailable = 0;
    for warrior in &state.warriors {
        let record = warrior.record();
        if record.defeated_marker()? == -1
            || record.stamina()? < 1
            || record.faction()? != 0
            || record.automatic_mode()? < 0
        {
            unavailable += 1;
        }
    }
    if unavailable == state.warriors.len() {
        state.native_work.early_booty = 1;
    }
    Ok(())
}

pub(crate) fn entry(state: &mut Session) -> Result<(), RuleError> {
    if state.combat.is_some() {
        return Err(unavailable("retained booty entry while combat is active"));
    }
    if state.booty_items.len() != 1000 {
        return Err("retained booty record array length".into());
    }
    let health = &mut state.native_work.health_context;
    health.unavailable = 0;
    state.native_work.round_work.inactive_rounds = Some(0);
    // DATA41fc32 is the read-only zero record copied into every41fc74 slot.
    state.booty_items.fill(InventoryRecord([0; 22]));
    state.native_work.loot_quality = Some(vec![0; 1000]);
    Ok(())
}

/// Follows entry and optional refit, preserving independent health-work fields.
pub(crate) fn normalize_retained(state: &mut Session) -> Result<(), RuleError> {
    let health = &mut state.native_work.health_context;
    for warrior in &mut state.warriors {
        let unavailable = warrior.stamina()? < 1 || warrior.record().automatic_mode()? < 0;
        story_inventory::normalize_warrior_after_battle(warrior)?;
        if unavailable {
            health.unavailable = health.unavailable.wrapping_add(1);
        }
    }
    let fields = &mut state.native_work.tactical_work.spell_fields;
    for record in fields.as_chunks_mut::<20>().0 {
        let mut field = SpellFieldMut(record);
        field.set_initiative(0);
        field.set_duration(0);
    }
    for item in &mut state.booty_items {
        item.clear_id();
    }
    Ok(())
}

pub(crate) fn early_retained(state: &mut Session) -> Result<(), RuleError> {
    normalize_retained(state)?;
    for warrior in &mut state.warriors {
        warrior.record_mut().set_active_flag(1);
    }
    // This loop is independent of the active roster and reads every marker.
    for index in 0..100 {
        let record = &mut state.native_work.npc_backing.slot_mut(index)?.record;
        if record.record().survivor_marker()? != -1 {
            record.record_mut().set_stamina(0);
        }
    }
    Ok(())
}

fn unavailable(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "combat.reward_work",
        dependency,
    }
}
