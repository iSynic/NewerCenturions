use crate::combat::Combat;
use crate::feedback::CLIP_SOUND_BASE;
use crate::{Event, Phase, RandomState, RuleError, Session};
use centurions_content::records::ItemDefinition;
use centurions_content::{Content, WARRIOR_BYTES, Warrior, word};

/// Data ID item of the worn battery that `fit_battery` refills.
const BATTERY: i16 = 637;
/// Data ID item of the battery supply loaded into it.
const BATTERY_SUPPLY: i16 = 307;
/// Party ammunition reserve holding the battery supply.
const BATTERY_SUPPLY_RESERVE: usize = 107;
/// Charge held by a full battery.
const BATTERY_CAPACITY: i16 = 250;

/// The player's reload command for a warrior. Out of combat it refits his ammunition; in combat
/// only the active warrior may reload and the refit is forced. A warrior whose weapon cannot be
/// filled is warned that he is out of ammunition and loses automatic control.
pub fn reload_command(
    state: &mut Session,
    content: &Content,
    index: usize,
) -> Result<Vec<Event>, RuleError> {
    if state.phase == Phase::SquadSelection
        || state
            .exploration
            .as_ref()
            .is_some_and(|p| p.presentation.is_some())
    {
        return Err(RuleError::Invalid(
            "reload requires an unobstructed mission".into(),
        ));
    }
    if state.phase == Phase::Combat
        && state.combat.as_ref().and_then(Combat::active_body) != Some(index as i16)
    {
        return Err(RuleError::Invalid(
            "reload requires the active warrior".into(),
        ));
    }
    if state.phase == Phase::Exploration {
        let warrior = state
            .warriors
            .get(index)
            .ok_or_else(|| RuleError::Invalid("warrior outside squad".into()))?;
        let result = crate::ammunition::refit_with_record(
            warrior,
            &state.ammunition,
            content,
            crate::ammunition::RefitFlags {
                in_combat: false,
                force: false,
                silent: false,
                warnings: true,
            },
            &state.globals.loaded_items,
            &mut state.random,
            Some(&state.globals.inventory_record),
        )?;
        return crate::ammunition::commit_result(state, index, result);
    }
    let warrior = state.warriors.get(index).ok_or("warrior outside squad")?;
    let id = if warrior.record().toggle() {
        warrior.record().equipped_weapon()?
    } else {
        warrior.record().melee_weapon()?
    };
    let mut loaded = state.globals.loaded_items.clone();
    loaded.load_weapon(&content.items, id)?;
    if loaded.item.is_none() {
        loaded.item = Some(crate::ammunition::definition(content, 0)?);
    }
    let rounds = centurions_content::records::ItemDefinition(loaded.weapon.as_deref().unwrap())
        .rounds_per_shot()?;
    let insufficient = warrior.inventory()?.iter().any(|record| {
        record.id() == id
            && record.equipped() != 0
            && record.charge().is_ok_and(|quantity| quantity < rounds)
    });
    let result = crate::ammunition::refit_with_record(
        warrior,
        &state.ammunition,
        content,
        crate::ammunition::RefitFlags {
            in_combat: true,
            force: true,
            silent: false,
            warnings: true,
        },
        &loaded,
        &mut state.random,
        Some(&state.globals.inventory_record),
    )?;
    let complete = result.completed_refit();
    let mut events = crate::ammunition::commit_result(state, index, result)?;
    if !complete {
        // Written only when the load is short and the refit failed.
        if insufficient {
            state.set_in_spell(0);
        }
        state.warriors[index].record_mut().set_computer_control(0);
        if !events.iter().any(|event| {
            matches!(
                event,
                Event::CollectionRejected {
                    warning: crate::feedback::OUT_OF_AMMUNITION_WARNING,
                }
            )
        }) {
            events.push(Event::CollectionRejected {
                warning: crate::feedback::OUT_OF_AMMUNITION_WARNING,
            });
        }
    }
    Ok(events)
}

/// The outcome of fitting ammunition: the warrior as changed, the party's ammunition reserves, the
/// sounds to play, and whether anything was loaded.
pub struct FitAmmunitionResult {
    pub warrior: Warrior,
    pub ammunition: Vec<i32>,
    pub loaded_item_field32: i16,
    pub sounds: Vec<i16>,
    pub loaded: bool,
}

/// Charges the worn shield battery from the battery supply reserve, taking up to its capacity, and
/// plays a clip sound. Not supported in combat.
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
    let loaded = item.charge()?;
    let ammunition = item.ammunition_id()?;
    let battery = ItemDefinition(
        content
            .items
            .get(BATTERY as usize)
            .ok_or_else(|| RuleError::Invalid("battery definition missing".into()))?,
    );
    let supply = ItemDefinition(
        content
            .items
            .get(BATTERY_SUPPLY as usize)
            .ok_or_else(|| RuleError::Invalid("battery supply definition missing".into()))?,
    );
    if in_combat
        || item.equipped() != 1
        || item.category_code() != 20
        || item.reserve_kind_code() != 0
        || item.unidentified_bytes().iter().any(|value| *value != 0)
        || item.required_level_code() != 2
        || item.modifier_bytes().iter().any(|value| *value != 0)
        || item.capacity()? != BATTERY_CAPACITY
        || !((loaded == 0 && ammunition == 0)
            || ((0..=BATTERY_CAPACITY).contains(&loaded) && ammunition == BATTERY_SUPPLY))
        || warrior.record().equipped_battery()? != BATTERY
        || warrior.record().shield_points()? != loaded
        || warrior.record().shield_capacity()? != BATTERY_CAPACITY
        || battery.item_type()? != 20
        || battery.ammunition_caliber()? != 24
        || battery.battery_capacity()? != BATTERY_CAPACITY
        || battery.capacity_bonus()? != 0
        || supply.item_type()? != 26
        || supply.reserve_ammunition_kind()? != 24
        || (loaded != 0 && reserves[BATTERY_SUPPLY_RESERVE] > 0)
    {
        return Err(RuleError::Unsupported {
            action: "equipment.fit_ammunition",
            dependency: "other battery instance, repeated fill or combat fitting",
        });
    }
    if reserves[BATTERY_SUPPLY_RESERVE] < 0 {
        return Err(RuleError::Invalid("negative battery supply reserve".into()));
    }
    let mut result = FitAmmunitionResult {
        warrior: warrior.clone(),
        ammunition: reserves.to_vec(),
        loaded_item_field32: 24,
        sounds: vec![],
        loaded: false,
    };
    if reserves[BATTERY_SUPPLY_RESERVE] == 0 {
        if loaded != 0 {
            result
                .sounds
                .push(random.bound(2).wrapping_add(CLIP_SOUND_BASE));
        }
        return Ok(result);
    }
    let amount = reserves[BATTERY_SUPPLY_RESERVE].min(i32::from(BATTERY_CAPACITY)) as i16;
    let mut record = result.warrior.record_mut();
    record.set_inventory_quantity(slot, amount);
    record.set_loaded_ammunition(BATTERY_SUPPLY);
    record.set_inventory_ammunition(slot, BATTERY_SUPPLY);
    record.set_shield_points(loaded.wrapping_add(amount));
    result.ammunition[BATTERY_SUPPLY_RESERVE] =
        reserves[BATTERY_SUPPLY_RESERVE].wrapping_sub(i32::from(amount));
    result
        .sounds
        .push(random.bound(2).wrapping_add(CLIP_SOUND_BASE));
    result.loaded = true;
    Ok(result)
}

/// Loads a weapon's ammunition from the party's reserves, as much as the weapon holds. Any
/// ammunition already in the weapon goes back to the reserves first. In combat the load costs the
/// warrior one attack.
pub fn fit_ammunition(
    warrior: &Warrior,
    slot: usize,
    content: &centurions_content::Content,
    reserves: &[i32],
    in_combat: bool,
    random: &mut RandomState,
) -> Result<FitAmmunitionResult, RuleError> {
    centurions_content::span(&warrior.bytes, 0, WARRIOR_BYTES)?;
    if reserves.len() != 200 {
        return Err(RuleError::Invalid("ammunition reserve size".into()));
    }
    let inventory = warrior.inventory()?;
    let item = inventory
        .get(slot)
        .ok_or_else(|| RuleError::Invalid("ammunition fitting slot missing".into()))?;
    if item.id() == BATTERY {
        return fit_battery(warrior, slot, content, reserves, in_combat, random);
    }
    let loaded = item.charge()?;
    let ammunition = item.ammunition_id()?;
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
        || warrior.record().toggle_flag() != 1
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

/// Refills the warrior's equipped weapons from one ammunition reserve, stopping after the first
/// that is filled completely. Returns whether one was, and the sounds to play.
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
                || warrior.record().toggle_flag() != 1
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
        // A partial refill leaves the stockpile unchanged.
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
