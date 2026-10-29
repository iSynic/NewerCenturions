//! showiteminfo 0x1555f2, reached from the Items info button (CNTL 140,
//! 12b5a6..12b6ec) and from row clicks while its DLOG 149 is open.
//!
//! loaditem -> loadammo(record WORD+4) -> signed instance merge -> loadspell2
//! for 1101..3999 -> WORD 76 = 1 when the shown damage is nonzero and the
//! count is zero -> quickinfo132b0c under the preference. No RNG is drawn;
//! the race and MOS profiles loaded by canuse are not retained work.
use super::inventory_opening::{admit, check_scroll, redraw};
use crate::{Event, ItemInformationList, RuleError, Session};
use centurions_content::records::{InventorySlots, ItemDefinition, ItemPanelWorkMut};
use centurions_content::{Content, InventoryRecord};

/// Items 12b64e: sound 30005, updateitems on the shown row, then showiteminfo
/// on row+scroll. An owner without items skips the control (12b5ca).
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
    state.native_work.inventory_scroll = scroll as u16;
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

/// A click inside DLOG 149 restarts showiteminfo without sound or redraw.
/// Items rows (156988) need a slot below the selected owner's count and keep
/// the earlier member; the second owner's list (156a00) sets the member from
/// BYTE 44273d with no count test; a shop cell (156a6a) only moves the shop
/// cursor WORDs 41f70a/0c, so a member's slot reloads unchanged.
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
    // A dialog opened with member 0xffff keeps it for Items rows and shop
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

/// DLOG 149 items 1 and 2 (1568e2, 156932): showcondition154a46 plays 609, then 607 on
/// close. Mode 4 runs canuse(ITEM WORD 102) per member, and each canuse
/// reloads ITEM from that WORD; mode 3 loads only profiles.
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
            let item = state.native_work.ammunition_work.item.as_deref();
            let id =
                ItemDefinition(item.ok_or("item information work missing")?).description_id()?;
            if id > 0 {
                state.load_item_work(content, id)?;
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
    if state.native_work.health_context.selected_warrior != owner {
        return Err("item information owner is not selected".into());
    }
    Ok(())
}

/// showiteminfo's parameters: a member's slot, or member 0xffff with an item
/// ID and the slot WORD (0xffff when the caller passes none).
#[derive(Clone, Copy)]
pub(super) enum Shown {
    Slot(usize, usize),
    Item(i16, Option<usize>),
}

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
    state.load_item_work(content, record.id())?;
    let loaded = merged && record.ammunition_id()? != 0;
    let ammo_id = if merged { record.ammunition_id()? } else { 0 };
    let ammo = crate::ammunition::definition(content, ammo_id)?;
    state.native_work.weapon_work.load_ammunition(&ammo)?;
    if let Some(combat) = state.combat.as_mut() {
        combat.weapon_work.load_ammunition(&ammo)?;
    }
    let item = state.native_work.ammunition_work.item.as_mut().unwrap();
    // 1556a6: member 0xffff loads ammunition 0 and merges no instance.
    if merged {
        ItemPanelWorkMut(item).merge_item_information(&record.0, &ammo)?;
    }
    let spell = ItemDefinition(item).spell()?;
    let damage = shown_damage(item, &ammo, loaded, record.damage_enhancement())?;
    if damage.total != 0 && ItemDefinition(item).attack_count()? == 0 {
        ItemPanelWorkMut(item).set_attack_count(1)?;
    }
    state.native_work.ammunition_work.ammo = Some(ammo);
    super::inventory_detail::load_spell(state, content, spell)?;
    // 1567ae: quickinfo runs on BYTE44273f and the slot WORD in each window
    // mode; a negative slot returns before any load.
    if quick_info && let Some(slot) = slot {
        super::quick_info(state, content, owner, slot)?;
    }
    Ok(())
}

/// booty105748 option-click: loaditem(|ID|), the ammunition
/// pickup when that is a 200..400 reserve item, else sound 30005 and
/// showiteminfo(0xffff, 0xffff, ID).
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
        .booty_items
        .get(slot)
        .map(|item| item.id().wrapping_abs())
        .filter(|id| *id != 0)
        .ok_or("loot slot is empty")?;
    state.load_item_work(content, id)?;
    let reserve = ItemDefinition(state.native_work.ammunition_work.item.as_deref().unwrap())
        .reserve_ammunition_kind()?;
    if (200..=400).contains(&id) && reserve != 0 {
        return crate::exploration::take_loot(state, content, slot, None);
    }
    let owner = state.native_work.health_context.selected_warrior;
    load(state, content, owner, Shown::Item(id, None), false)?;
    Ok(vec![Event::Sound { resource_id: 30005 }])
}

/// moveicon13c1ee's option-drag in the shop window. BYTE44273f takes the
/// dragged list's member, then showiteminfo runs on that row (13c856 for the
/// second member, 13c88e for the first) or, on a shop cell, with member and
/// slot 0xffff and the stock's ITEM WORD 102 description ID (13c7fe). The
/// prefix's ITEM, cost and canuse loads are replaced by showiteminfo's
/// loaditem; Showitems and updateshopwings only redraw. No drag follows.
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
        let work = &state.native_work.shop_work;
        if work.quantity(slot)? == 0 {
            return Err("shop cell is empty".into());
        }
        let definition = crate::ammunition::definition(content, work.item(slot)?)?;
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

/// The DLOG 149 damage rows: each value scaled by the Single factor
/// (100 + instance BYTE 18) / 100 in extended precision and truncated.
pub struct ShownDamage {
    pub base: i16,
    pub elements: [i16; 7],
    pub total: i16,
}

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
