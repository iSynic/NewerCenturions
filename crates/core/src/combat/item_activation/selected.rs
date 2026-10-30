//! Loading, charging and redrawing the chosen item, before returning to whoever
//! opened the Items screen.
use super::{
    Activation, Caller, check_selected_slot, redraw_after_drop, refresh_inventory_info_items,
    refusal,
};
use crate::combat::player_psi;
use crate::combat::spells::spell_identity;
use crate::{Content, Event, RuleError, Session, equipment};
use centurions_content::records::ItemDefinition;

/// Loads the chosen item, which becomes the item in use; a refusal returns to the
/// screen, which clears it.
///
/// Original: Items
pub(crate) fn prepare_selected(
    state: &mut Session,
    content: &Content,
    owner: usize,
    slot: usize,
    caller: Caller,
) -> Result<Activation, RuleError> {
    let result = prepare(state, content, owner, slot, caller);
    if matches!(result, Err(RuleError::RefusedAfterEffects { .. })) {
        state.globals.item_used = 0;
    }
    result
}

/// Prepares the chosen inventory item for use. A type-27 or type-23 item is refused with its
/// warning; an item with no spell or no charge left does nothing. Otherwise the warrior must
/// be able to use it (except at an encounter) and its spell mode must be allowed here. A
/// charged item then loses one charge and its weight, and is dropped when emptied if it is
/// discarded when empty. Otherwise the warrior must
/// be able to use it (except at an encounter) and its spell mode must be allowed here. A
/// charged item then loses one charge and its weight, and is dropped when emptied if it is
/// discarded when empty.
fn prepare(
    state: &mut Session,
    content: &Content,
    owner: usize,
    slot: usize,
    caller: Caller,
) -> Result<Activation, RuleError> {
    // The row must be one the Items screen lets the player choose.
    check_selected_slot(slot)?;
    let selected = state
        .warriors
        .get(owner)
        .ok_or("item inventory owner missing")?
        .inventory()?
        .get(slot)
        .cloned()
        .ok_or("selected item slot missing")?;
    let item = selected.id();
    state.globals.item_used = item;
    let definition = crate::ammunition::definition(content, item)?;
    let definition = ItemDefinition(&definition);
    state.load_item(content, item)?;
    if definition.item_type()? == 27 {
        return refusal(24);
    }
    if definition.item_type()?.wrapping_abs() == 23 {
        return refusal(52);
    }
    let mut quantity = selected.charge()?;
    let positive = quantity > 0;
    let activating =
        definition.spell()? > 1100 && (positive || definition.supplied_quantity()? < 0);
    if !activating {
        state.clear_targets();
        return Ok(Activation {
            item,
            spell: 0,
            events: vec![],
        });
    }
    let map = state.land(content).ok_or("item activation map missing")?;
    if caller != Caller::Encounter
        && !equipment::can_use(&state.warriors[owner], definition.0, map)?
    {
        return refusal(29);
    }
    let spell = definition.spell()?;
    let packet = crate::combat::spells::load_packet(
        &map.combat
            .as_ref()
            .ok_or("item activation attack tables missing")?
            .attacks,
        spell,
    )?;
    state.globals.cast_identity = spell_identity(spell);
    let mut count = definition.effect_selector()?.wrapping_abs();
    if count == 8 {
        count = state.random.bound(7);
    }
    state.globals.attack_count = count;
    state.globals.spell_info = packet.clone();
    let health = &mut state.globals.health_context;
    health.effect = centurions_content::records::SpellInfoRecord(&packet).health_effect();
    if let Some(combat) = state.combat.as_mut() {
        combat.spell_info = packet.clone();
        combat.cast_identity = state.globals.cast_identity;
    }
    // The two packet flags follow loading, counting and the draw, before any charge.
    if state.combat.is_some() {
        if !player_psi::mode_allowed(&packet, true)? {
            return refusal(23);
        }
    } else if caller != Caller::Encounter && !player_psi::mode_allowed(&packet, false)? {
        // Back to the inventory with no warning.
        return Err(RuleError::RefusedAfterEffects { feedback: vec![] });
    }
    let mut events = vec![];
    // An item with quantity -1 is not charged but still redraws.
    if positive || quantity == -1 {
        if positive {
            quantity = quantity.wrapping_sub(1);
            state.warriors[owner]
                .record_mut()
                .set_inventory_quantity(slot, quantity);
            let weight = state.warriors[owner]
                .record()
                .load()?
                .wrapping_sub(definition.charge_weight()?);
            state.warriors[owner].record_mut().set_load(weight);
        }
        if quantity != 0 || definition.discarded_when_empty()? == 0 {
            // Redrawing the inventory makes this warrior the panel's character.
            state.globals.character_bare_hand_bonus =
                state.warriors[owner].record().bare_hand_bonus()?;
            state.globals.inventory_record = state.warriors[owner].record().inventory_slot(slot);
        }
        if quantity == 0 && definition.discarded_when_empty()? != 0 {
            let map = state.land(content).ok_or("item activation map missing")?;
            let globals = equipment::drop(&mut state.warriors[owner], slot, content, map)?;
            state.apply_equipment_globals(&globals)?;
            events.push(Event::Sound { resource_id: 655 });
            redraw_after_drop(state, owner, slot)?;
            refresh_inventory_info_items(state, content, owner)?;
        }
    }
    // The item stays in use even after its last unit is dropped.
    state.load_item(content, item)?;
    state.clear_targets();
    Ok(Activation {
        item,
        spell,
        events,
    })
}
