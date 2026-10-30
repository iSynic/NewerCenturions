//! The item information dialog (DLOG 149), opened from the Items info button
//! and retargeted by row clicks while it is open.
//!
//! It loads the item, its ammunition, the instance's own values and the item's
//! spell. An item that shows damage but has no attack count shows one attack.
//! With the quick-info preference the Items detail follows. No random numbers
//! are drawn.
use super::inventory_opening::{admit, check_scroll, redraw};
use crate::{Event, ItemInformationList, RuleError, Session};
use centurions_content::records::{InventorySlots, ItemDefinition, ItemPanelMut};
use centurions_content::{Content, InventoryRecord};

/// The info button: sound 30005, the shown row redrawn, then the dialog for
/// that row. An owner without items ignores the button.
///
/// Original: items, showiteminfo
pub(crate) fn show(
    state: &mut Session,
    content: &Content,
    owner: usize,
    scroll: usize,
    row: usize,
    quick_info: bool,
) -> Result<Vec<Event>, RuleError> {
    admit_dialog(state, owner)?;
    check_scroll(scroll)?;
    if row > 10 {
        return Err("inventory row outside the eleven display rows".into());
    }
    if InventorySlots(&state.warriors[owner].bytes).count()? == 0 {
        return Ok(vec![]);
    }
    state.globals.inventory_scroll = scroll as u16;
    redraw(state, content, owner, scroll, row, row)?;
    load(
        state,
        content,
        owner,
        Shown::Slot(owner, scroll + row),
        quick_info,
    )?;
    Ok(vec![Event::Sound { resource_id: 30005 }])
}

/// A click behind the open dialog shows another item without sound or redraw.
/// An Items row needs a slot below the owner's count and keeps the earlier
/// member; the second owner's list takes its own member with no count test; a
/// shop cell only moves the shop cursor, so a member's slot reloads unchanged.
///
/// Original: showiteminfo
pub(crate) fn retarget(
    state: &mut Session,
    content: &Content,
    owner: usize,
    (member, slot, list, item): (usize, usize, ItemInformationList, Option<i16>),
    quick_info: bool,
) -> Result<Vec<Event>, RuleError> {
    admit_dialog(state, owner)?;
    if member >= state.warriors.len() || slot > 40 {
        return Err("item information row outside the shown lists".into());
    }
    let count = InventorySlots(&state.warriors[owner].bytes).count()?;
    if list == ItemInformationList::Items && slot as i16 >= count {
        return Ok(vec![]);
    }
    // A dialog showing a bare item ID keeps doing so for Items rows and shop
    // cells, which then pass the clicked record or stock ID.
    let shown = match (item, list) {
        (Some(_), ItemInformationList::Items) => {
            let id = state.warriors[owner].record().inventory_slot(slot).id();
            Shown::Item(id, Some(slot))
        }
        (Some(id), ItemInformationList::Shop) => Shown::Item(id, Some(slot)),
        _ => Shown::Slot(member, slot),
    };
    load(state, content, owner, shown, quick_info)?;
    Ok(vec![])
}

/// The dialog's user and MOS popups play 609, then 607 on close. The
/// user list checks each member with canuse, which reloads the item from its
/// description each time; the MOS list loads only the profiles.
///
/// Original: showcondition
pub(crate) fn list_users(
    state: &mut Session,
    content: &Content,
    owner: usize,
    professions: bool,
    widened: bool,
) -> Result<Vec<Event>, RuleError> {
    admit_dialog(state, owner)?;
    if !professions {
        let passes = if widened { state.warriors.len() } else { 1 };
        for _ in 0..passes * state.warriors.len() {
            let item = state.globals.loaded_items.item.as_deref();
            let id =
                ItemDefinition(item.ok_or("item information work missing")?).description_id()?;
            if id > 0 {
                state.load_item(content, id)?;
            }
        }
    }
    Ok(vec![
        Event::Sound { resource_id: 609 },
        Event::Sound { resource_id: 607 },
    ])
}

/// The dialog runs over the Items window, the shop or booty's loot grid.
fn admit_dialog(state: &Session, owner: usize) -> Result<(), RuleError> {
    let shop = state.exploration.as_ref().is_some_and(|p| {
        matches!(
            p.presentation,
            Some(crate::Presentation::Shop | crate::Presentation::Loot)
        )
    });
    if !shop {
        admit(state, owner)?;
    } else if owner >= state.warriors.len() {
        return Err("inventory entry requires an available owner and caller".into());
    }
    if state.globals.health_context.char_select_new != owner {
        return Err("item information owner is not selected".into());
    }
    Ok(())
}

/// What the dialog shows: a member's slot, or a bare item ID with the slot the
/// caller passes, if any.
#[derive(Clone, Copy)]
pub(super) enum Shown {
    Slot(usize, usize),
    Item(i16, Option<usize>),
}

/// Loads what the item information dialog shows: the item, its ammunition, the instance's own
/// values and the item's spell.
pub(super) fn load(
    state: &mut Session,
    content: &Content,
    owner: usize,
    shown: Shown,
    quick_info: bool,
) -> Result<(), RuleError> {
    let (record, slot) = match shown {
        Shown::Slot(member, slot) => (
            state.warriors[member].record().inventory_slot(slot),
            Some(slot),
        ),
        Shown::Item(id, slot) => {
            let mut record = InventoryRecord([0; 22]);
            record.set_id(id);
            (record, slot)
        }
    };
    let merged = matches!(shown, Shown::Slot(..));
    state.load_item(content, record.id())?;
    let loaded = merged && record.ammunition_id()? != 0;
    let ammo_id = if merged { record.ammunition_id()? } else { 0 };
    let ammo = crate::ammunition::definition(content, ammo_id)?;
    state.globals.weapon_traits.load_ammunition(&ammo)?;
    if let Some(combat) = state.combat.as_mut() {
        combat.weapon_traits.load_ammunition(&ammo)?;
    }
    let item = state.globals.loaded_items.item.as_mut().unwrap();
    // A bare item ID loads no ammunition and has no instance values.
    if merged {
        ItemPanelMut(item).merge_item_information(&record.0, &ammo)?;
    }
    let spell = ItemDefinition(item).spell()?;
    let damage = shown_damage(item, &ammo, loaded, record.damage_enhancement())?;
    if damage.total != 0 && ItemDefinition(item).attack_count()? == 0 {
        ItemPanelMut(item).set_attack_count(1)?;
    }
    state.globals.loaded_items.ammo = Some(ammo);
    super::inventory_detail::load_spell(state, content, spell)?;
    // The detail follows for the selected owner in every window; without a
    // slot it loads nothing.
    if quick_info && let Some(slot) = slot {
        super::quick_info(state, content, owner, slot)?;
    }
    Ok(())
}

/// Option-clicking loot picks up reserve ammunition (200..400), or else plays
/// sound 30005 and shows the item's information.
///
/// Original: booty, showiteminfo
pub(crate) fn show_loot(
    state: &mut Session,
    content: &Content,
    slot: usize,
) -> Result<Vec<Event>, RuleError> {
    let collecting = state
        .exploration
        .as_ref()
        .is_some_and(|p| p.presentation == Some(crate::Presentation::Loot));
    if !collecting {
        return Err("no loot choice active".into());
    }
    let id = state
        .booty_list
        .get(slot)
        .map(|item| item.id().wrapping_abs())
        .filter(|id| *id != 0)
        .ok_or("loot slot is empty")?;
    state.load_item(content, id)?;
    let reserve = ItemDefinition(state.globals.loaded_items.item.as_deref().unwrap())
        .reserve_ammunition_kind()?;
    if (200..=400).contains(&id) && reserve != 0 {
        return crate::exploration::take_loot(state, content, slot, None);
    }
    let owner = state.globals.health_context.char_select_new;
    load(state, content, owner, Shown::Item(id, None), false)?;
    Ok(vec![Event::Sound { resource_id: 30005 }])
}

/// Option-dragging in the shop window shows an item's information instead of
/// moving it. A member's row selects that member and shows the row; a shop cell
/// shows the stock item's description.
///
/// Original: moveicon, showiteminfo
pub(crate) fn show_dragged(
    state: &mut Session,
    content: &Content,
    member: usize,
    (slot, stock): (usize, bool),
    quick_info: bool,
) -> Result<Vec<Event>, RuleError> {
    let shop = state
        .exploration
        .as_ref()
        .is_some_and(|p| p.presentation == Some(crate::Presentation::Shop));
    if !shop || member >= state.warriors.len() {
        return Err("item drag requires the shop window and a squad member".into());
    }
    let shown = if stock {
        let shop = &state.globals.the_shop;
        if shop.quantity(slot)? == 0 {
            return Err("shop cell is empty".into());
        }
        let definition = crate::ammunition::definition(content, shop.item(slot)?)?;
        Shown::Item(ItemDefinition(&definition).description_id()?, None)
    } else {
        if slot as i16 >= InventorySlots(&state.warriors[member].bytes).count()? {
            return Err("dragged row is empty".into());
        }
        Shown::Slot(member, slot)
    };
    state.set_warrior_selector(member);
    load(state, content, member, shown, quick_info)?;
    Ok(vec![])
}

/// The dialog's damage rows: each value scaled by (100 + the instance's damage
/// enhancement) / 100, rounded to single precision, multiplied in extended
/// precision and truncated.
pub struct ShownDamage {
    pub base: i16,
    pub elements: [i16; 7],
    pub total: i16,
}

/// The damage the information dialog shows for an item: its missile and melee damage (with the
/// loaded ammunition's), multiplied by the rounds per shot when that is nonzero. Each element is
/// the item's plus the ammunition's, multiplied by the rounds per shot. All are scaled by the
/// enhancement percentage, and only positive elements add to the total.
pub fn shown_damage(
    item: &[u8],
    ammo: &[u8],
    loaded: bool,
    enhancement: i8,
) -> Result<ShownDamage, RuleError> {
    let factor = (100_i32 + i32::from(enhancement)) as f32 / 100.0;
    let scale = |value: i16| (f64::from(value) * f64::from(factor)).trunc() as i32 as i16;
    let (item, ammo) = (ItemDefinition(item), ItemDefinition(ammo));
    let mut base = item.missile_damage()?.wrapping_add(item.melee_damage()?);
    if loaded {
        base = base.wrapping_add(ammo.missile_damage()?.wrapping_add(ammo.melee_damage()?));
    }
    let rounds = item.rounds_per_shot()?;
    if rounds != 0 {
        base = base.wrapping_mul(rounds);
    }
    let base = scale(base);
    let mut total = base;
    let mut elements = [0; 7];
    for (element, shown) in elements.iter_mut().enumerate() {
        let value = ammo
            .elemental_damage(element)?
            .wrapping_add(item.elemental_damage(element)?)
            .wrapping_mul(rounds);
        *shown = scale(value);
        if *shown > 0 {
            total = total.wrapping_add(*shown);
        }
    }
    Ok(ShownDamage {
        base,
        elements,
        total,
    })
}
