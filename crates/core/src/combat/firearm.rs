mod preparation;
pub(super) use preparation::compose_weapon_attack;
pub(crate) use preparation::prepare_packet;
pub use preparation::{prepare, prepare_enemy};
mod reload;
mod resolution;
pub use reload::reload_command;
pub use resolution::{resolve, resolve_enemy, resolve_enemy_after_duration};
pub(crate) use resolution::{resolve_after_duration, resolve_friendly_after_duration};

/// Weapon `spell` descriptors for firearms: Cargo Pod `STR#` 4000 "Special 1st"
/// entries 1 "Point Target" and 2 "Area Target".
pub(crate) const POINT_TARGET_ATTACK: i16 = 4101;
pub(crate) const AREA_TARGET_ATTACK: i16 = 4102;
/// Data ID items whose `STR#` 601 "Devices Short" names are "Small Laser" and
/// "Medium Laser"; the only enemy primary weapons `prepare_enemy` accepts.
pub(crate) const SMALL_LASER: i16 = 680;
pub(crate) const MEDIUM_LASER: i16 = 681;

use crate::feedback::CLIP_SOUND_BASE;
use crate::{RandomState, RuleError};
use centurions_content::{Warrior, word};

pub struct PreparedFirearm {
    pub packet: Vec<u8>,
    pub slot: usize,
    pub shots: i16,
    pub rounds: i16,
    pub weight: i16,
    pub sound: i16,
    pub armor_filter: bool,
}

pub struct FitAmmunitionResult {
    pub warrior: Warrior,
    pub ammunition: Vec<i32>,
    pub loaded_item_field32: i16,
    pub sounds: Vec<i16>,
    pub loaded: bool,
}

fn fit_battery(
    warrior: &Warrior,
    slot: usize,
    content: &centurions_content::Content,
    reserves: &[i32],
    in_combat: bool,
    random: &mut RandomState,
) -> Result<FitAmmunitionResult, RuleError> {
    let inventory = warrior.inventory()?;
    let item = &inventory[slot];
    let loaded = word(&item.0, 2)?;
    let ammunition = word(&item.0, 4)?;
    let battery = content
        .items
        .get(637)
        .ok_or_else(|| RuleError::Invalid("battery definition missing".into()))?;
    let supply = content
        .items
        .get(307)
        .ok_or_else(|| RuleError::Invalid("battery supply definition missing".into()))?;
    if in_combat
        || item.0[9] != 1
        || item.0[10] != 20
        || item.0[8] != 0
        || item.0[11..13].iter().any(|value| *value != 0)
        || item.0[13] != 2
        || item.0[14..].iter().any(|value| *value != 0)
        || word(&item.0, 6)? != 250
        || !((loaded == 0 && ammunition == 0) || ((0..=250).contains(&loaded) && ammunition == 307))
        || warrior.record().equipped_battery()? != 637
        || warrior.record().shield_points()? != loaded
        || warrior.record().shield_capacity()? != 250
        || word(battery, 26)? != 20
        || word(battery, 32)? != 24
        || word(battery, 90)? != 250
        || word(battery, 148)? != 0
        || word(supply, 26)? != 26
        || word(supply, 34)? != 24
        || (loaded != 0 && reserves[107] > 0)
    {
        return Err(RuleError::Unsupported {
            action: "equipment.fit_ammunition",
            dependency: "other battery instance, repeated fill or combat fitting",
        });
    }
    if reserves[107] < 0 {
        return Err(RuleError::Invalid("negative battery supply reserve".into()));
    }
    let mut result = FitAmmunitionResult {
        warrior: warrior.clone(),
        ammunition: reserves.to_vec(),
        loaded_item_field32: 24,
        sounds: vec![],
        loaded: false,
    };
    if reserves[107] == 0 {
        if loaded != 0 {
            result
                .sounds
                .push(random.bound(2).wrapping_add(CLIP_SOUND_BASE));
        }
        return Ok(result);
    }
    let amount = reserves[107].min(250) as i16;
    let mut record = result.warrior.record_mut();
    record.set_inventory_quantity(slot, amount);
    record.set_loaded_ammunition(307);
    record.set_inventory_ammunition(slot, 307);
    record.set_shield_points(loaded.wrapping_add(amount));
    result.ammunition[107] = reserves[107].wrapping_sub(i32::from(amount));
    result
        .sounds
        .push(random.bound(2).wrapping_add(CLIP_SOUND_BASE));
    result.loaded = true;
    Ok(result)
}

pub fn fit_ammunition(
    warrior: &Warrior,
    slot: usize,
    content: &centurions_content::Content,
    reserves: &[i32],
    in_combat: bool,
    random: &mut RandomState,
) -> Result<FitAmmunitionResult, RuleError> {
    centurions_content::span(&warrior.bytes, 0, 3138)?;
    if reserves.len() != 200 {
        return Err(RuleError::Invalid("ammunition reserve size".into()));
    }
    let inventory = warrior.inventory()?;
    let item = inventory
        .get(slot)
        .ok_or_else(|| RuleError::Invalid("ammunition fitting slot missing".into()))?;
    if item.id() == 637 {
        return fit_battery(warrior, slot, content, reserves, in_combat, random);
    }
    let loaded = word(&item.0, 2)?;
    let ammunition = word(&item.0, 4)?;
    let (ammo_id, reserve_index, capacity, caliber, level, bonus) = match item.id() {
        2 => (201, 1, 12, 2, 1, 0),
        38 => (203, 3, 3, 4, 4, 35),
        _ => {
            return Err(RuleError::Unsupported {
                action: "equipment.fit_ammunition",
                dependency: "other firearm",
            });
        }
    };
    let loaded_supported = if item.id() == 2 {
        (ammunition == 0 && loaded == 0)
            || (ammunition == ammo_id && (0..=capacity).contains(&loaded))
    } else {
        matches!((loaded, ammunition), (0, 0) | (2, 203))
    };
    if item.0[9] != 1
        || item.0[10] != 15
        || item.0[13] != level
        || item.0[14] != 0
        || item.0[15] != bonus
        || item.0[16..].iter().any(|value| *value != 0)
        || word(&item.0, 6)? != capacity
        || !loaded_supported
        || warrior.record().equipped_weapon()? != item.id()
        || warrior.record().firing_mode_flag() != 1
        || (item.id() != 2 && warrior.record().loaded_ammunition()? != ammunition)
    {
        return Err(RuleError::Unsupported {
            action: "equipment.fit_ammunition",
            dependency: "other firearm, ammunition or loaded instance",
        });
    }
    let weapon = content
        .items
        .get(item.id() as usize)
        .ok_or_else(|| RuleError::Invalid("firearm definition missing".into()))?;
    let ammo = content
        .items
        .get(ammo_id as usize)
        .ok_or_else(|| RuleError::Invalid("ammunition definition missing".into()))?;
    if word(weapon, 32)? != caliber || word(ammo, 34)? != caliber || word(ammo, 26)? != 26 {
        return Err(RuleError::Unsupported {
            action: "equipment.fit_ammunition",
            dependency: "other ammunition definitions",
        });
    }
    if reserves[reserve_index] < 0 {
        return Err(RuleError::Invalid("negative ammunition reserve".into()));
    }
    let mut result = FitAmmunitionResult {
        warrior: warrior.clone(),
        ammunition: reserves.to_vec(),
        loaded_item_field32: caliber,
        sounds: if in_combat { vec![141] } else { vec![] },
        loaded: false,
    };
    if reserves[reserve_index] == 0 {
        return Ok(result);
    }
    let mut next_random = random.clone();
    // Stored ammunition IDs differ from popup reserve indices, so unload first.
    if ammunition != 0 {
        let returned = reserves[reserve_index]
            .checked_add(i32::from(loaded))
            .ok_or(RuleError::Unsupported {
                action: "equipment.fit_ammunition",
                dependency: "ammunition stock overflow",
            })?;
        result.ammunition[reserve_index] = returned;
        let mut record = result.warrior.record_mut();
        record.set_inventory_quantity(slot, 0);
        record.set_inventory_ammunition(slot, 0);
        record.set_loaded_ammunition(0);
    }
    let amount = result.ammunition[reserve_index].min(i32::from(capacity)) as i16;
    if in_combat && amount > 0 {
        let actions = result.warrior.record().action_points()?.wrapping_sub(1);
        result.warrior.record_mut().set_action_points(actions);
    }
    let mut record = result.warrior.record_mut();
    record.set_inventory_quantity(slot, amount);
    record.set_loaded_ammunition(ammo_id);
    record.set_inventory_ammunition(slot, ammo_id);
    result.ammunition[reserve_index] =
        result.ammunition[reserve_index].wrapping_sub(i32::from(amount));
    result
        .sounds
        .push(next_random.bound(2).wrapping_add(CLIP_SOUND_BASE));
    result.loaded = true;
    *random = next_random;
    Ok(result)
}

pub fn reload(
    warrior: &mut Warrior,
    reserve: &mut i32,
    silent: bool,
    random: &mut RandomState,
) -> Result<(bool, Vec<i16>), RuleError> {
    if warrior.record().equipped_weapon()? == 165 {
        return Err(RuleError::Unsupported {
            action: "combat.firearm.reload",
            dependency: "earned firearm reload",
        });
    }
    let inventory = warrior.inventory()?;
    let mut sounds = vec![];
    for (slot, item) in inventory.iter().enumerate() {
        if item.0[9] == 0 {
            continue;
        }
        let loaded = word(&item.0, 2)?;
        if item.id() == 2
            && (item.0[9] != 1
                || item.0[10] != 15
                || item.0[13] != 1
                || item.0[14..22].iter().any(|value| *value != 0)
                || word(&item.0, 6)? != 12
                || word(&item.0, 4)? != 201
                || loaded != 0
                || warrior.record().equipped_weapon()? != 2
                || warrior.record().loaded_ammunition()? != 201
                || warrior.record().firing_mode_flag() != 1
                || !(0..=i32::from(i16::MAX)).contains(reserve)
                || !silent)
        {
            return Err(RuleError::Unsupported {
                action: "combat.reload",
                dependency: "other ordinary firearm reload instance",
            });
        }
        let needed = word(&item.0, 6)?.wrapping_sub(loaded);
        if needed <= 0 || word(&item.0, 4)? == 0 {
            continue;
        }
        let expected_ammo = match item.id() {
            2 => 201,
            38 => 203,
            _ => 200,
        };
        if !matches!(item.id(), 1 | 2 | 38)
            || word(&item.0, 4)? != expected_ammo
            || item.0[10] != 15
            || (item.id() == 38 && (item.0[15] != 35 || word(&item.0, 6)? != 3))
        {
            return Err(RuleError::Unsupported {
                action: "combat.reload",
                dependency: "other active equipment",
            });
        }
        if needed <= 0 || *reserve == 0 {
            continue;
        }
        let amount = (*reserve as i16).min(needed);
        if item.id() == 38 && (*reserve as i16) < needed {
            return Err(RuleError::Unsupported {
                action: "combat.reload",
                dependency: "partial refill with ammunition 203",
            });
        }
        warrior
            .record_mut()
            .set_inventory_quantity(slot, loaded.wrapping_add(amount));
        let complete = (*reserve as i16) >= needed;
        // The original partial-refill branch leaves the stockpile unchanged.
        if complete {
            *reserve = reserve.wrapping_sub(i32::from(needed));
        }
        if !silent {
            sounds.push(random.bound(2).wrapping_add(CLIP_SOUND_BASE));
        }
        if complete {
            return Ok((true, sounds));
        }
    }
    Ok((false, sounds))
}

pub(in crate::combat) use preparation::compose_item_attack;
