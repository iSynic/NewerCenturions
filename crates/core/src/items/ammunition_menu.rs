//! The ammunition popup (MENU 1000) from the Items ammunition button.
//!
//! An item without a caliber warns 3. Otherwise the first entry unloads the
//! item or says why it cannot, followed by every reserve with rounds that
//! matches the caliber and, outside combat, a divider, three headings and every
//! reserve with rounds, to show its information. The chosen entry unloads,
//! loads through the FitAmmunition rules, or shows the reserve's description.
//! Loading or unloading redraws the Items rows.
use super::inventory_opening::{admit, check_scroll, redraw};
use super::item_information::{Shown, load};
use crate::ammunition::definition;
use crate::{Event, Phase, RuleError, Session};
use centurions_content::Content;
use centurions_content::records::{InventorySlots, ItemDefinition};

/// One line of the ammunition popup.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AmmunitionEntry {
    /// Item 1, enabled: "Unload your " and the item name.
    Unload,
    /// Item 1, disabled: the item "is not loaded."
    NotLoaded,
    /// Item 1, disabled: the item "cannot be unloaded."; every load is checked.
    Fixed,
    /// A compatible reserve, fitted through FitAmmunition.
    Load {
        reserve: usize,
        description: i16,
        stock: i32,
        units: bool,
        checked: bool,
    },
    /// An AppendMenu "-" divider.
    Divider,
    /// A disabled heading: the menu string stored at this DATA address.
    Heading(u32),
    /// A reserve with rounds, chosen to show its information.
    Information {
        description: i16,
        stock: i32,
        units: bool,
    },
}

impl AmmunitionEntry {
    /// Whether the player can choose the entry.
    pub fn enabled(&self) -> bool {
        matches!(
            self,
            Self::Unload | Self::Load { .. } | Self::Information { .. }
        )
    }
}

/// The popup for an owner's inventory slot: the item's description ID (the
/// name used by item 1) and the entries in menu order from item 1.
pub fn ammunition_menu(
    state: &Session,
    content: &Content,
    owner: usize,
    slot: usize,
) -> Result<(i16, Vec<AmmunitionEntry>), RuleError> {
    let warrior = state.warriors.get(owner).ok_or("warrior outside squad")?;
    if slot as i16 >= InventorySlots(&warrior.bytes).count()? {
        return Err("ammunition row is empty".into());
    }
    let record = warrior.record().inventory_slot(slot);
    let item = definition(content, record.id())?;
    let caliber = ItemDefinition(&item).ammunition_caliber()?;
    if caliber == 0 {
        return Err("the item takes no ammunition".into());
    }
    let loaded = record.ammunition_id()?;
    let first = if loaded == 0 {
        AmmunitionEntry::NotLoaded
    } else if !ItemDefinition(&definition(content, loaded)?).supplies_shield_energy()?
        && record.charge()? >= 1
    {
        AmmunitionEntry::Unload
    } else {
        AmmunitionEntry::Fixed
    };
    let fixed = first == AmmunitionEntry::Fixed;
    let mut entries = vec![first];
    let mut reserves = vec![];
    for reserve in 0..=200 {
        // Reserve 200 lies past the table and shares the first loot-quality
        // bytes; the game reads it as a reserve all the same.
        let stock = match reserve {
            200 => match state.globals.loot_quality.as_deref() {
                Some(quality) => i32::from_be_bytes(quality[..4].try_into().unwrap()),
                None => continue,
            },
            _ => state.ammunition[reserve],
        };
        if stock <= 0 {
            continue;
        }
        let ammo = definition(content, reserve as i16 + 200)?;
        let ammo = ItemDefinition(&ammo);
        let (description, units) = (ammo.description_id()?, ammo.supplies_shield_energy()?);
        reserves.push((reserve, description, stock, units));
        if ammo.reserve_ammunition_kind()? == caliber {
            entries.push(AmmunitionEntry::Load {
                reserve,
                description,
                stock,
                units,
                // The check mark compares the reserve index with the description ID.
                checked: reserve as i16 == description || fixed,
            });
        }
    }
    if state.phase != Phase::Combat {
        entries.extend([
            AmmunitionEntry::Divider,
            AmmunitionEntry::Heading(0x403140),
            AmmunitionEntry::Heading(0x403124),
            AmmunitionEntry::Divider,
            AmmunitionEntry::Heading(0x40310c),
        ]);
        entries.extend(reserves.into_iter().map(|(_, description, stock, units)| {
            AmmunitionEntry::Information {
                description,
                stock,
                units,
            }
        }));
    }
    Ok((ItemDefinition(&item).description_id()?, entries))
}

/// The Items ammunition button and its popup choice. `item` is the chosen
/// MENU 1000 item; zero stands for the warning path of an item without a
/// caliber.
pub(crate) fn choose(
    state: &mut Session,
    content: &Content,
    owner: usize,
    (scroll, row): (usize, usize),
    item: u16,
) -> Result<Vec<Event>, RuleError> {
    admit(state, owner)?;
    check_scroll(scroll)?;
    if state.globals.health_context.char_select_new != owner {
        return Err("ammunition owner is not selected".into());
    }
    if row > 10 {
        return Err("inventory row outside the eleven display rows".into());
    }
    // In combat the owner must be the warrior whose turn it is (else warning 10)
    // with an action point (else warning 17). The combat display update this
    // needs cannot run behind the Items window.
    if state.phase == Phase::Combat {
        let warning = if state
            .combat
            .as_ref()
            .and_then(crate::combat::Combat::active_body)
            != Some(owner as i16)
        {
            Some(10)
        } else if state.warriors[owner].record().action_points()? < 1 {
            Some(17)
        } else {
            None
        };
        if let Some(index) = warning {
            return Err(RuleError::Refused {
                feedback: vec![crate::feedback::GameFeedback::Warning { index }],
            });
        }
    }
    let slot = scroll + row;
    let record = state.warriors[owner].record().inventory_slot(slot);
    if slot as i16 >= InventorySlots(&state.warriors[owner].bytes).count()? {
        return Err("ammunition row is empty".into());
    }
    if ItemDefinition(&definition(content, record.id())?).ammunition_caliber()? == 0 {
        state.globals.inventory_scroll = scroll as u16;
        state.load_item(content, record.id())?;
        return Ok(vec![Event::Feedback(
            crate::feedback::GameFeedback::Warning { index: 3 },
        )]);
    }
    let (_, entries) = ammunition_menu(state, content, owner, slot)?;
    let entry = usize::from(item)
        .checked_sub(1)
        .and_then(|index| entries.get(index))
        .filter(|entry| entry.enabled())
        .ok_or_else(|| RuleError::Invalid("ammunition menu item is not an enabled entry".into()))?
        .clone();
    let loaded = record.ammunition_id()?;
    if entry == AmmunitionEntry::Unload && !(200..400).contains(&loaded) {
        return Err("loaded ammunition outside the reserve table".into());
    }
    state.globals.inventory_scroll = scroll as u16;
    let (events, update) = match entry {
        AmmunitionEntry::Unload => {
            // The reserve gains the rounds, then the magazine and the loaded
            // ammunition clear.
            let ammo = definition(content, loaded)?;
            let projected = match state.combat.as_mut() {
                Some(combat) => &mut combat.weapon_traits,
                None => &mut state.globals.weapon_traits,
            };
            projected.load_ammunition(&ammo)?;
            state.globals.loaded_items.ammo = Some(ammo);
            let reserve = (loaded - 200) as usize;
            state.ammunition[reserve] =
                state.ammunition[reserve].wrapping_add(i32::from(record.charge()?));
            let mut warrior = state.warriors[owner].record_mut();
            warrior.set_inventory_quantity(slot, 0);
            warrior.set_loaded_ammunition(0);
            warrior.set_inventory_ammunition(slot, 0);
            let sound = 403 + state.random.bound(2);
            (vec![Event::Sound { resource_id: sound }], true)
        }
        AmmunitionEntry::Load { reserve, units, .. } => {
            // A different loaded reserve is unloaded first.
            let unloads = !units && loaded != 0 && record.charge()? > 0 && loaded != reserve as i16;
            // The FitAmmunition rules: one action point in combat when rounds move.
            let loot_quality = state
                .globals
                .loot_quality
                .as_deref()
                .map(|quality| quality[..4].try_into().unwrap());
            let result = crate::ammunition::fit(
                &state.warriors[owner],
                &state.ammunition,
                content,
                crate::ammunition::FitRequest {
                    slot,
                    reserve_index: reserve,
                    in_combat: state.phase == Phase::Combat,
                    loot_quality,
                },
                &state.globals.loaded_items,
                &mut state.random,
            )?;
            let events = crate::ammunition::commit_result(state, owner, result)?;
            (events, reserve != 0 || unloads)
        }
        AmmunitionEntry::Information { description, .. } => {
            load(state, content, owner, Shown::Item(description, None), false)?;
            (vec![], false)
        }
        _ => unreachable!("disabled entries are refused above"),
    };
    if update {
        redraw(state, content, owner, scroll, 0, 10)?;
    }
    Ok(events)
}
