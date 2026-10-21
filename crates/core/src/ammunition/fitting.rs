use super::selection::validate_instance;
use super::{
    AmmunitionOutcome, AmmunitionResult, FitRequest, WorkBuffers, definition, initial, unsupported,
};
use crate::{RandomState, RuleError};
use centurions_content::records::ItemDefinition;
use centurions_content::{Content, Warrior};

pub fn fit(
    warrior: &Warrior,
    reserves: &[i32],
    content: &Content,
    request: FitRequest,
    work: &WorkBuffers,
    random: &mut RandomState,
) -> Result<AmmunitionResult, RuleError> {
    let inventory = warrior.inventory()?;
    let record = inventory
        .get(request.slot)
        .ok_or_else(|| RuleError::Invalid("ammunition inventory slot".into()))?;
    validate_instance(record, content)?;
    if request.reserve_index >= 200 {
        return Err("ammunition candidate outside reserve table".into());
    }
    let ordinary = (request.reserve_index + 200) as i16;
    let mut result = initial(warrior, reserves, work)?;
    let mut next_random = random.clone();
    let old_ammo = record.ammunition_id()?;
    if old_ammo != 0 {
        result.work.ammo = Some(definition(content, old_ammo)?);
    }
    result.work.item = Some(definition(content, record.id())?);
    let caliber = ItemDefinition(result.work.item.as_deref().unwrap()).ammunition_caliber()?;
    let mut candidates = Vec::new();
    for index in 0..=200 {
        let stock = if index == 200 {
            // ID400 cannot match these calibers; accepted ammo replaces its cache write.
            let Some(quality) = request.loot_quality else {
                continue;
            };
            i32::from_be_bytes(quality)
        } else {
            result.reserves[index]
        };
        if stock <= 0 {
            continue;
        }
        result.work.ammo = Some(definition(content, (index + 200) as i16)?);
        if ItemDefinition(result.work.ammo.as_deref().unwrap()).reserve_ammunition_kind()?
            == caliber
        {
            candidates.push(index);
        }
    }
    if !candidates.contains(&request.reserve_index) {
        return Err(unsupported(
            "selected ammunition is not a positive compatible candidate",
        ));
    }
    result.work.ammo = Some(definition(content, ordinary)?);
    let mut loaded = record.quantity()?;
    if !ItemDefinition(result.work.ammo.as_deref().unwrap()).supplies_shield_energy()?
        && old_ammo != 0
        && loaded >= 1
        && old_ammo != request.reserve_index as i16
    {
        result.changed_ammo = true;
        result.work.ammo = Some(definition(content, old_ammo)?);
        let old_index = (old_ammo - 200) as usize;
        result.reserves[old_index] = result.reserves[old_index].wrapping_add(i32::from(loaded));
        result
            .warrior
            .record_mut()
            .set_inventory_quantity(request.slot, 0);
        result
            .warrior
            .record_mut()
            .set_inventory_ammunition(request.slot, 0);
        result.warrior.record_mut().set_loaded_ammunition(0);
        loaded = 0;
    }
    let mut needed = record.capacity()?.wrapping_sub(loaded);
    let stock = result.reserves[request.reserve_index];
    if i32::from(needed) >= stock {
        needed = stock as i16;
    }
    if request.in_combat && needed > 0 {
        let ap = result.warrior.record().action_points()?.wrapping_sub(1);
        result.warrior.record_mut().set_action_points(ap);
    }
    result
        .warrior
        .record_mut()
        .set_inventory_quantity(request.slot, loaded.wrapping_add(needed));
    result.reserves[request.reserve_index] = stock.wrapping_sub(i32::from(needed));
    result
        .warrior
        .record_mut()
        .set_inventory_ammunition(request.slot, ordinary);
    result.warrior.record_mut().set_loaded_ammunition(ordinary);
    if ItemDefinition(result.work.ammo.as_deref().unwrap()).supplies_shield_energy()?
        && record.equipment_flag() != 0
    {
        let energy = result
            .warrior
            .record()
            .shield_points()?
            .wrapping_add(needed);
        result.warrior.record_mut().set_shield_points(energy);
    }
    if request.reserve_index != 0 || result.changed_ammo {
        result.sounds.push(403 + next_random.bound(2));
    }
    result.outcome = AmmunitionOutcome::Fitted {
        reserve_index: request.reserve_index,
    };
    Ok(result.finish(random, next_random))
}
