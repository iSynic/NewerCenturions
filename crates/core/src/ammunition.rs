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

fn ordinary_ammo(id: i16) -> Option<i16> {
    match id.unsigned_abs() {
        1 => Some(200),
        2 => Some(201),
        38 => Some(203),
        165 => Some(280),
        _ => None,
    }
}

fn validate_instance(record: &InventoryRecord, content: &Content) -> Result<i16, RuleError> {
    let ammo = ordinary_ammo(record.id())
        .ok_or_else(|| unsupported("other equipped item refill effects"))?;
    let weapon = definition(content, record.id())?;
    let ammo_definition = definition(content, ammo)?;
    if record.0[10] != 15
        || word(&weapon, 26)? != 15
        || word(&ammo_definition, 26)? != 26
        || word(&ammo_definition, 18)? != 0
        || word(&weapon, 32)? != word(&ammo_definition, 34)?
    {
        return Err(unsupported("other weapon or ammunition definition"));
    }
    let loaded_ammo = word(&record.0, 4)?;
    if loaded_ammo != 0 && loaded_ammo != ammo {
        return Err(unsupported("other owned ammunition pair"));
    }
    Ok(ammo)
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
    let inventory = warrior.inventory()?;
    for record in inventory.iter().filter(|record| record.0[9] != 0) {
        let loaded = word(&record.0, 2)?;
        let capacity = word(&record.0, 6)?;
        if (loaded < capacity && word(&record.0, 4)? != 0) || (loaded <= 0 && capacity > 0) {
            validate_instance(record, content)?;
        }
    }
    let mut result = initial(warrior, reserves, work)?;
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
            partial_warning(&mut result, flags.warnings);
        } else {
            put_word(
                &mut result.warrior.bytes,
                offset + 2,
                loaded.wrapping_add(needed),
            );
            result.reserves[index] = stock.wrapping_sub(i32::from(needed));
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
            if Some(ammo) != ordinary_ammo(record.id()) {
                return Err(unsupported("other compatible alternate ammunition"));
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
                partial_warning(&mut result, flags.warnings);
            } else {
                put_word(
                    &mut result.warrior.bytes,
                    offset + 2,
                    loaded.wrapping_add(needed),
                );
                put_word(&mut result.warrior.bytes, offset + 4, ammo);
                result.reserves[index] = stock.wrapping_sub(i32::from(needed));
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
    let ordinary = validate_instance(record, content)?;
    if request.reserve_index >= 200 || (request.reserve_index + 200) as i16 != ordinary {
        return Err(unsupported("other selected ammunition candidate"));
    }
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
    if old_ammo != 0 && loaded >= 1 && old_ammo != request.reserve_index as i16 {
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
    if request.reserve_index != 0 || result.changed_ammo {
        result.sounds.push(403 + next_random.bound(2));
    }
    result.native_return = request.reserve_index as i16;
    *random = next_random;
    Ok(result)
}

#[cfg(all(test, feature = "original-fixtures"))]
mod work_tests {
    use super::*;
    use serde_json::Value;

    fn content() -> Content {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        Content::load(
            &root.join("NewCenturions"),
            &root.join("analysis/extracted"),
        )
        .unwrap()
    }

    fn bytes(value: &Value) -> Vec<u8> {
        let text = value.as_str().unwrap();
        (0..text.len())
            .step_by(2)
            .map(|offset| u8::from_str_radix(&text[offset..offset + 2], 16).unwrap())
            .collect()
    }

    #[test]
    fn complete_npc_preparation_records_match_original_instructions() {
        let content = content();
        let fixture: Value = serde_json::from_str(include_str!(
            "../../../evidence/fixtures/final-enemy-weapons.json"
        ))
        .unwrap();
        let cases = fixture["preparation_cases"].as_array().unwrap();
        assert_eq!(cases.len(), 324);
        for (index, case) in cases.iter().enumerate() {
            let input = bytes(&case["enemy_before"]);
            let weapon = word(&input, 0x26).unwrap();
            let ammo = if case["context"].as_i64().unwrap() == 0 {
                0
            } else {
                word(&input, 0x4c).unwrap()
            };
            let retained = definition(&content, 700).unwrap();
            let mut work = WorkBuffers {
                item: Some(retained.clone()),
                ..Default::default()
            };
            work.prepare_pair(
                &content.items[weapon.unsigned_abs() as usize],
                &content.items[ammo.unsigned_abs() as usize],
            )
            .unwrap();
            assert_eq!(
                work.weapon,
                Some(bytes(&case["weapon"])),
                "weapon case {index}"
            );
            assert_eq!(work.ammo, Some(bytes(&case["ammo"])), "ammo case {index}");
            assert_eq!(work.item, Some(retained), "retained item case {index}");
        }
    }

    #[test]
    fn natural_and_melee_loaders_replace_only_their_original_work_records() {
        let content = content();
        let fixture: Value = serde_json::from_str(include_str!(
            "../../../evidence/fixtures/player-guard-reaction.json"
        ))
        .unwrap();
        let mut work = WorkBuffers::default();
        work.load_natural(&content.items).unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            for snapshot in std::iter::once(&case["after"]).chain(
                case["attack_returns"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|entry| &entry["state"]),
            ) {
                assert_eq!(work.item, Some(bytes(&snapshot["item_work"])));
                assert_eq!(work.weapon, Some(bytes(&snapshot["weapon_work"])));
                assert_eq!(work.ammo, Some(bytes(&snapshot["ammo_work"])));
            }
        }
        let retained = work.clone();
        work.load_weapon(&content.items, 187).unwrap();
        assert_eq!(work.weapon, Some(definition(&content, 187).unwrap()));
        assert_eq!(work.item, retained.item);
        assert_eq!(work.ammo, retained.ammo);
        let mut unknown = WorkBuffers::default();
        unknown.load_weapon(&content.items, 187).unwrap();
        assert!(unknown.item.is_none() && unknown.ammo.is_none());
    }
}
