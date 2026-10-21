use crate::{RandomState, RuleError};
use centurions_content::{Content, InventoryRecord, Warrior, word};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkBuffers {
    pub item: Option<Vec<u8>>,
    pub ammo: Option<Vec<u8>>,
    pub weapon: Option<Vec<u8>>,
}

impl WorkBuffers {
    pub fn validate(&self) -> Result<(), RuleError> {
        if [&self.item, &self.ammo, &self.weapon]
            .into_iter()
            .flatten()
            .any(|record| record.len() != 150)
        {
            return Err(RuleError::Invalid("ammunition work record length".into()));
        }
        Ok(())
    }

    pub(crate) fn load_item(&mut self, content: &Content, id: i16) -> Result<i16, RuleError> {
        self.load_item_from(&content.items, id)
    }

    pub(crate) fn load_item_from(&mut self, items: &[Vec<u8>], id: i16) -> Result<i16, RuleError> {
        let record = item_definition(items, id)?;
        let context = word(&record, 32)?;
        self.item = Some(record);
        Ok(context)
    }

    pub(crate) fn load_natural(&mut self, items: &[Vec<u8>]) -> Result<(), RuleError> {
        let record = item_definition(items, 0)?;
        let mut weapon = record.clone();
        put_word(&mut weapon, 100, 0);
        self.weapon = Some(weapon);
        self.ammo = Some(record.clone());
        self.item = Some(record);
        Ok(())
    }

    pub(crate) fn load_weapon(&mut self, items: &[Vec<u8>], id: i16) -> Result<(), RuleError> {
        self.weapon = Some(item_definition(items, id)?);
        Ok(())
    }

    pub(crate) fn prepare_pair(&mut self, weapon: &[u8], ammo: &[u8]) -> Result<(), RuleError> {
        if weapon.len() != 150 || ammo.len() != 150 {
            return Err(RuleError::Invalid("ammunition work record length".into()));
        }
        let mut prepared = weapon.to_vec();
        put_word(&mut prepared, 26, word(weapon, 26)?.wrapping_abs());
        let mut ammunition = ammo.to_vec();
        put_word(&mut ammunition, 26, word(ammo, 26)?.wrapping_abs());
        if word(weapon, 44)? >= 1101 {
            for offset in [12, 14] {
                let value = word(ammo, offset)?;
                if value != 0 {
                    put_word(&mut prepared, offset, value);
                }
            }
            if word(ammo, 56)? != 0 {
                put_word(&mut prepared, 56, i16::from(ammo[57] as i8));
            }
        }
        self.weapon = Some(prepared);
        self.ammo = Some(ammunition);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RefitFlags {
    pub in_combat: bool,
    pub force: bool,
    pub silent: bool,
    pub warnings: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FitRequest {
    pub slot: usize,
    pub reserve_index: usize,
    pub in_combat: bool,
    pub loot_quality: Option<[u8; 4]>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct AmmunitionResult {
    pub warrior: Warrior,
    pub reserves: Vec<i32>,
    pub work: WorkBuffers,
    pub native_return: i16,
    pub sounds: Vec<i16>,
    pub warnings: Vec<i16>,
    pub changed_ammo: bool,
    pub(crate) retained_record: Option<InventoryRecord>,
}

fn unsupported(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "ammunition",
        dependency,
    }
}

fn put_word(bytes: &mut [u8], offset: usize, value: i16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
}

pub(crate) fn definition(content: &Content, id: i16) -> Result<Vec<u8>, RuleError> {
    item_definition(&content.items, id)
}

fn item_definition(items: &[Vec<u8>], id: i16) -> Result<Vec<u8>, RuleError> {
    let bytes = items
        .get(usize::from(id.unsigned_abs()))
        .ok_or_else(|| RuleError::Invalid("ammunition definition missing".into()))?;
    if bytes.len() != 150 {
        return Err(RuleError::Invalid("ammunition definition length".into()));
    }
    let mut result = bytes.clone();
    let class = word(&result, 26)?.wrapping_abs();
    put_word(&mut result, 26, class);
    Ok(result)
}

fn validate_instance(record: &InventoryRecord, content: &Content) -> Result<i16, RuleError> {
    let weapon = definition(content, record.id())?;
    let category = word(&weapon, 26)?;
    if !matches!(category, 2 | 15 | 20) {
        return Err(unsupported("refill effects for this equipment category"));
    }
    let loaded = word(&record.0, 4)?;
    let compatible = |id| -> Result<bool, RuleError> {
        let ammo = definition(content, id)?;
        // refit14e5e4 matches the caliber even when ammunition has a use effect.
        // fit handles those effects separately after its candidate is selected.
        Ok(word(&ammo, 26)? == 26 && word(&weapon, 32)? == word(&ammo, 34)?)
    };
    if loaded != 0 {
        // refit14e5e4 uses an existing magazine's reserve index directly. Its
        // empty-magazine branch can itself write 200 + inventory slot here.
        if !(200..400).contains(&loaded) {
            return Err("incompatible loaded ammunition".into());
        }
        definition(content, loaded)?;
        return Ok(loaded);
    }
    for id in 200..400 {
        if compatible(id)? {
            return Ok(id);
        }
    }
    Err(unsupported("no compatible ammunition definition"))
}

pub fn compatible_ammunition(
    content: &Content,
    record: &InventoryRecord,
) -> Result<Vec<i16>, RuleError> {
    let item = definition(content, record.id())?;
    let caliber = word(&item, 32)?;
    (200..400)
        .filter_map(|id| {
            let ammo = match definition(content, id) {
                Ok(ammo) => ammo,
                Err(error) => return Some(Err(error)),
            };
            match (word(&ammo, 26), word(&ammo, 34)) {
                (Ok(26), Ok(value)) if value == caliber => Some(Ok(id)),
                (Ok(_), Ok(_)) => None,
                (Err(error), _) | (_, Err(error)) => Some(Err(error.into())),
            }
        })
        .collect()
}

fn initial(
    warrior: &Warrior,
    reserves: &[i32],
    work: &WorkBuffers,
) -> Result<AmmunitionResult, RuleError> {
    if reserves.len() != 200 {
        return Err(RuleError::Invalid(
            "ammunition reserve vector length".into(),
        ));
    }
    warrior.inventory()?;
    work.validate()?;
    Ok(AmmunitionResult {
        warrior: warrior.clone(),
        reserves: reserves.to_vec(),
        work: work.clone(),
        native_return: 0,
        sounds: vec![],
        warnings: vec![],
        changed_ammo: false,
        retained_record: None,
    })
}

fn refill_sound(result: &mut AmmunitionResult, random: &mut RandomState, silent: bool) {
    if !silent {
        result.sounds.push(403 + random.bound(2));
    }
}

fn partial_warning(result: &mut AmmunitionResult, enabled: bool) {
    if enabled {
        result.warnings.push(49);
        result.sounds.push(6000);
    }
}

fn refill_energy(
    result: &mut AmmunitionResult,
    content: &Content,
    slot: usize,
    amount: i16,
    remove_first: bool,
) -> Result<(), RuleError> {
    let offset = 0x132 + slot * 22;
    if result.warrior.bytes[offset + 10] != 20 {
        return Ok(());
    }
    if word(&result.warrior.bytes, offset + 2)? > 0 {
        if remove_first {
            refit_equipment(result, content, slot, false)?;
        }
        put_word(&mut result.warrior.bytes, 0xee, 0);
        result.warrior.bytes[offset + 9] = 0;
        refit_equipment(result, content, slot, true)?;
        if result.warrior.bytes[offset + 9] != 0 {
            let id = word(&result.warrior.bytes, offset)?;
            result.sounds.push(word(&definition(content, id)?, 12)?);
        }
    } else {
        let energy = word(&result.warrior.bytes, 0x26)?.wrapping_add(amount);
        put_word(&mut result.warrior.bytes, 0x26, energy);
    }
    Ok(())
}

fn refit_equipment(
    result: &mut AmmunitionResult,
    content: &Content,
    slot: usize,
    wearing: bool,
) -> Result<(), RuleError> {
    let retained = result.retained_record.as_ref().ok_or_else(|| {
        unsupported("unrepresented selected inventory record for battery re-wear")
    })?;
    let equipped = crate::editor_items::retained_equipment(
        &result.warrior,
        content,
        slot,
        wearing,
        false,
        retained,
    )?;
    result.warrior = equipped.warrior;
    result.retained_record = Some(equipped.record);
    result.work.item = Some(equipped.definition);
    result.warnings.extend(equipped.warnings);
    Ok(())
}

fn class_gate(
    result: &mut AmmunitionResult,
    id: i16,
    content: &Content,
    flags: RefitFlags,
) -> Result<bool, RuleError> {
    if flags.force || flags.in_combat {
        result.work.ammo = Some(definition(content, id)?);
        return Ok(matches!(
            word(
                result
                    .work
                    .item
                    .as_deref()
                    .ok_or_else(|| unsupported("unrepresented retained item class writer"))?,
                26
            )?,
            2 | 15
        ));
    }
    Ok(true)
}

pub fn refit(
    warrior: &Warrior,
    reserves: &[i32],
    content: &Content,
    flags: RefitFlags,
    work: &WorkBuffers,
    random: &mut RandomState,
) -> Result<AmmunitionResult, RuleError> {
    // The historical primitive API supplies its selected battery row. Session
    // callers supply the actual retained record through refit_with_record.
    let retained = warrior.inventory()?.into_iter().find(|row| row.0[10] == 20);
    refit_with_record(
        warrior,
        reserves,
        content,
        flags,
        work,
        random,
        retained.as_ref(),
    )
}

pub(crate) fn refit_with_record(
    warrior: &Warrior,
    reserves: &[i32],
    content: &Content,
    flags: RefitFlags,
    work: &WorkBuffers,
    random: &mut RandomState,
    retained: Option<&InventoryRecord>,
) -> Result<AmmunitionResult, RuleError> {
    let inventory = warrior.inventory()?;
    for record in inventory.iter().filter(|record| record.0[9] != 0) {
        let loaded = word(&record.0, 2)?;
        let capacity = word(&record.0, 6)?;
        if (loaded < capacity && word(&record.0, 4)? != 0) || (loaded <= 0 && capacity > 0) {
            validate_instance(record, content)?;
        }
    }
    let mut result = initial(warrior, reserves, work)?;
    result.retained_record = retained.cloned();
    let mut next_random = random.clone();
    for (slot, record) in inventory.iter().enumerate() {
        let offset = 0x132 + slot * 22;
        let loaded = word(&result.warrior.bytes, offset + 2)?;
        let capacity = word(&record.0, 6)?;
        let ammo = word(&record.0, 4)?;
        if record.0[9] == 0 || loaded >= capacity || ammo == 0 {
            continue;
        }
        if !class_gate(&mut result, record.id(), content, flags)? {
            continue;
        }
        result.work.ammo = Some(definition(content, ammo)?);
        let index = (ammo - 200) as usize;
        let stock = result.reserves[index];
        if stock == 0 {
            continue;
        }
        let needed = capacity.wrapping_sub(loaded);
        if (stock as i16) < needed {
            refill_sound(&mut result, &mut next_random, flags.silent);
            put_word(
                &mut result.warrior.bytes,
                offset + 2,
                loaded.wrapping_add(stock as i16),
            );
            refill_energy(&mut result, content, slot, stock as i16, false)?;
            partial_warning(&mut result, flags.warnings);
        } else {
            put_word(
                &mut result.warrior.bytes,
                offset + 2,
                loaded.wrapping_add(needed),
            );
            result.reserves[index] = stock.wrapping_sub(i32::from(needed));
            refill_energy(&mut result, content, slot, needed, false)?;
            refill_sound(&mut result, &mut next_random, flags.silent);
            if flags.force || flags.in_combat {
                result.native_return = 1;
                *random = next_random;
                return Ok(result);
            }
        }
    }
    for (slot, record) in inventory.iter().enumerate() {
        let offset = 0x132 + slot * 22;
        let loaded = word(&result.warrior.bytes, offset + 2)?;
        let capacity = word(&record.0, 6)?;
        if record.0[9] == 0 || loaded > 0 || capacity <= 0 {
            continue;
        }
        if !class_gate(&mut result, record.id(), content, flags)? {
            continue;
        }
        result.work.item = Some(definition(content, record.id())?);
        let caliber = word(result.work.item.as_deref().unwrap(), 32)?;
        for index in 0..200 {
            let stock = result.reserves[index];
            if stock == 0 {
                continue;
            }
            let ammo = (index + 200) as i16;
            result.work.ammo = Some(definition(content, ammo)?);
            if word(result.work.ammo.as_deref().unwrap(), 34)? != caliber {
                continue;
            }
            let needed = capacity.wrapping_sub(loaded);
            if (stock as i16) <= needed {
                refill_sound(&mut result, &mut next_random, flags.silent);
                put_word(
                    &mut result.warrior.bytes,
                    offset + 2,
                    loaded.wrapping_add(stock as i16),
                );
                put_word(
                    &mut result.warrior.bytes,
                    offset + 4,
                    (slot as i16).wrapping_add(200),
                );
                refill_energy(&mut result, content, slot, stock as i16, true)?;
                partial_warning(&mut result, flags.warnings);
            } else {
                put_word(
                    &mut result.warrior.bytes,
                    offset + 2,
                    loaded.wrapping_add(needed),
                );
                put_word(&mut result.warrior.bytes, offset + 4, ammo);
                result.reserves[index] = stock.wrapping_sub(i32::from(needed));
                refill_energy(&mut result, content, slot, needed, true)?;
                refill_sound(&mut result, &mut next_random, flags.silent);
                result.native_return = i16::from(flags.force || flags.in_combat);
            }
            *random = next_random;
            return Ok(result);
        }
    }
    *random = next_random;
    Ok(result)
}

pub(crate) fn commit_result(
    state: &mut crate::Session,
    member: usize,
    result: AmmunitionResult,
) -> Result<Vec<crate::Event>, RuleError> {
    if let Some(item) = result.work.item.as_deref() {
        state.loaded_item_field32 = Some(word(item, 32)?);
    }
    let projected = if let Some(combat) = state.combat.as_mut() {
        &mut combat.weapon_work
    } else {
        &mut state.weapon_work
    };
    if let Some(weapon) = result.work.weapon.as_deref() {
        projected.load_weapon(weapon)?;
    }
    if let Some(ammo) = result.work.ammo.as_deref() {
        projected.load_ammunition(ammo)?;
    }
    state.warriors[member] = result.warrior;
    state.ammunition = result.reserves;
    state.ammunition_work = result.work;
    if let Some(record) = result.retained_record {
        state.inventory_record = Some(record);
    }
    Ok(result
        .sounds
        .into_iter()
        .map(|resource_id| crate::Event::Sound { resource_id })
        .chain(
            result
                .warnings
                .into_iter()
                .map(|warning| crate::Event::CollectionRejected { warning }),
        )
        .collect())
}

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
    let old_ammo = word(&record.0, 4)?;
    if old_ammo != 0 {
        result.work.ammo = Some(definition(content, old_ammo)?);
    }
    result.work.item = Some(definition(content, record.id())?);
    let caliber = word(result.work.item.as_deref().unwrap(), 32)?;
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
        if word(result.work.ammo.as_deref().unwrap(), 34)? == caliber {
            candidates.push(index);
        }
    }
    if !candidates.contains(&request.reserve_index) {
        return Err(unsupported(
            "selected ammunition is not a positive compatible candidate",
        ));
    }
    result.work.ammo = Some(definition(content, ordinary)?);
    let offset = 0x132 + request.slot * 22;
    let mut loaded = word(&record.0, 2)?;
    if word(result.work.ammo.as_deref().unwrap(), 18)? == 0
        && old_ammo != 0
        && loaded >= 1
        && old_ammo != request.reserve_index as i16
    {
        result.changed_ammo = true;
        result.work.ammo = Some(definition(content, old_ammo)?);
        let old_index = (old_ammo - 200) as usize;
        result.reserves[old_index] = result.reserves[old_index].wrapping_add(i32::from(loaded));
        put_word(&mut result.warrior.bytes, offset + 2, 0);
        put_word(&mut result.warrior.bytes, offset + 4, 0);
        put_word(&mut result.warrior.bytes, 0x126, 0);
        loaded = 0;
    }
    let mut needed = word(&record.0, 6)?.wrapping_sub(loaded);
    let stock = result.reserves[request.reserve_index];
    if i32::from(needed) >= stock {
        needed = stock as i16;
    }
    if request.in_combat && needed > 0 {
        let ap = word(&result.warrior.bytes, 0x40)?.wrapping_sub(1);
        put_word(&mut result.warrior.bytes, 0x40, ap);
    }
    put_word(
        &mut result.warrior.bytes,
        offset + 2,
        loaded.wrapping_add(needed),
    );
    result.reserves[request.reserve_index] = stock.wrapping_sub(i32::from(needed));
    put_word(&mut result.warrior.bytes, offset + 4, ordinary);
    put_word(&mut result.warrior.bytes, 0x126, ordinary);
    if word(result.work.ammo.as_deref().unwrap(), 18)? != 0 && record.0[9] != 0 {
        let energy = word(&result.warrior.bytes, 0x26)?.wrapping_add(needed);
        put_word(&mut result.warrior.bytes, 0x26, energy);
    }
    if request.reserve_index != 0 || result.changed_ammo {
        result.sounds.push(403 + next_random.bound(2));
    }
    result.native_return = request.reserve_index as i16;
    *random = next_random;
    Ok(result)
}
#[cfg(all(test, feature = "original-fixtures"))]
#[path = "../tests/unit/ammunition_work_tests.rs"]
mod work_tests;
