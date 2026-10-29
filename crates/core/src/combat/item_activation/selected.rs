//! Shared selected Items load, charge and redraw before caller-specific returns.
use super::{
    Activation, Caller, check_selected_slot, refresh_inventory_info_work, refusal,
    retain_drop_redraw,
};
use crate::{Content, Event, RuleError, Session, equipment};
use centurions_content::records::ItemDefinition;

/// Items 12aa80 leaves the selected ID in WORD44719a before loaditem. A refusal
/// returns to the event loop, whose 129e4a clears it.
pub(crate) fn prepare_selected(
    state: &mut Session,
    content: &Content,
    owner: usize,
    slot: usize,
    caller: Caller,
) -> Result<Activation, RuleError> {
    let result = prepare(state, content, owner, slot, caller);
    if matches!(result, Err(RuleError::RefusedAfterEffects { .. })) {
        state.native_work.fire_item = 0;
    }
    result
}

fn prepare(
    state: &mut Session,
    content: &Content,
    owner: usize,
    slot: usize,
    caller: Caller,
) -> Result<Activation, RuleError> {
    // This raw suffix still requires a row selectable by its original Items UI.
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
    state.native_work.fire_item = item;
    let definition = crate::ammunition::definition(content, item)?;
    let definition = ItemDefinition(&definition);
    state.load_item_work(content, item)?;
    if definition.category()? == 27 {
        return refusal(24);
    }
    if definition.category()?.wrapping_abs() == 23 {
        return refusal(52);
    }
    let mut quantity = selected.quantity()?;
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
    state.native_work.attack_identity = [spell / 1000 - 1, spell % 1000 / 100 - 1, spell % 100 - 1];
    let mut count = definition.effect_selector()?.wrapping_abs();
    if count == 8 {
        count = state.random.bound(7);
    }
    state.native_work.attack_count = count;
    state.native_work.attack_packet = packet.clone();
    let health = &mut state.native_work.health_context;
    health.effect = centurions_content::records::AttackPacket(&packet).health_effect();
    if let Some(combat) = state.combat.as_mut() {
        combat.attack_packet = packet.clone();
        combat.attack_identity = state.native_work.attack_identity;
    }
    // The two packet flags follow load/count/Rand, before any charge.
    if state.combat.is_some() {
        if !crate::combat::player_psi::mode_allowed(&packet, true)? {
            return refusal(23);
        }
    } else if caller != Caller::Encounter
        && !crate::combat::player_psi::mode_allowed(&packet, false)?
    {
        // 12aeec returns to the inventory loop without an original warning.
        return Err(RuleError::RefusedAfterEffects { feedback: vec![] });
    }
    let mut events = vec![];
    // 12af32..12af68 includes the uncharged -1 row in the redraw block.
    if positive || quantity == -1 {
        if positive {
            quantity = quantity.wrapping_sub(1);
            state.warriors[owner]
                .record_mut()
                .set_inventory_quantity(slot, quantity);
            let weight = state.warriors[owner]
                .record()
                .carried_weight()?
                .wrapping_sub(definition.unit_weight()?);
            state.warriors[owner]
                .record_mut()
                .set_carried_weight(weight);
        }
        if quantity != 0 || definition.discarded_when_empty()? == 0 {
            // updateitems160d08 first copies this owner to character work44721c.
            state.native_work.character_bare_hand_bonus =
                state.warriors[owner].record().bare_hand_bonus()?;
            state.native_work.inventory_record =
                state.warriors[owner].record().inventory_slot(slot);
        }
        if quantity == 0 && definition.discarded_when_empty()? != 0 {
            let map = state.land(content).ok_or("item activation map missing")?;
            let globals = equipment::drop(&mut state.warriors[owner], slot, content, map)?;
            state.apply_equipment_globals(&globals)?;
            events.push(Event::Sound { resource_id: 655 });
            retain_drop_redraw(state, owner, slot)?;
            refresh_inventory_info_work(state, content, owner)?;
        }
    }
    // The original ID survives a final-unit drop and its fixed-slot compaction.
    state.load_item_work(content, item)?;
    state.clear_targets();
    Ok(Activation {
        item,
        spell,
        events,
    })
}
