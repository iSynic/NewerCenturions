use crate::{Event, Phase, RuleError, Session, combat, items, story_collection::Collection};
use centurions_content::{Content, InventoryRecord, Warrior, mission::OpeningMission, span, word};

pub struct EquipmentResult {
    pub warrior: Warrior,
    pub changed: bool,
    pub sounds: Vec<i16>,
    pub warnings: Vec<i16>,
}

fn unsupported(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "equipment",
        dependency,
    }
}

fn item_definition(content: &Content, id: i16) -> Result<&[u8], RuleError> {
    content
        .items
        .get(usize::from(id.unsigned_abs()))
        .map(Vec::as_slice)
        .ok_or_else(|| RuleError::Invalid("equipment definition missing".into()))
}

pub(crate) fn calculate_weight(warrior: &mut Warrior, content: &Content) -> Result<(), RuleError> {
    let mut weight = 0_i16;
    for record in warrior.inventory()? {
        let definition = item_definition(content, record.id())?;
        weight = weight
            .wrapping_add(word(definition, 36)?.wrapping_mul(word(&record.0, 2)?))
            .wrapping_add(word(definition, 16)?);
    }
    put_word(&mut warrior.bytes, 0x9d6, weight);
    Ok(())
}

fn bit(bytes: &[u8], offset: usize, index: usize) -> Result<bool, RuleError> {
    Ok(span(bytes, offset + index / 8, 1)?[0] & (0x80 >> (index % 8)) != 0)
}

pub fn can_use(
    warrior: &Warrior,
    definition: &[u8],
    mission: &OpeningMission,
) -> Result<bool, RuleError> {
    span(definition, 0, 150)?;
    let race_id = word(&warrior.bytes, 0x32)?;
    let mos_id = word(&warrior.bytes, 0x34)?;
    if race_id <= 0 || mos_id <= 0 {
        return Err(unsupported("unset Race or MOS profile"));
    }
    let race = span(&mission.races, (race_id as usize - 1) * 408, 408)?;
    let mos = span(&mission.profiles, (mos_id as usize - 1) * 576, 576)?;
    if (word(definition, 108)? != 0 && word(definition, 108)? != race_id)
        || (word(definition, 110)? != 0 && word(definition, 110)? != mos_id)
    {
        return Ok(false);
    }
    let mut ability = 0;
    while ability < 58 && !bit(definition, 128, ability)? {
        ability += 1;
    }
    if !bit(mos, 442, ability)? || !bit(race, 336, ability)? {
        return Ok(false);
    }
    for index in 0..9 {
        let race_trait = bit(race, 344, index)?;
        if (bit(definition, 104, index)? && race_trait)
            || (bit(definition, 112, index)? && !race_trait)
        {
            return Ok(false);
        }
    }
    let school = word(mos, 254)?;
    if (0..7).contains(&school.wrapping_sub(1)) {
        let index = (school - 1) as usize;
        if bit(definition, 106, index)?
            || (word(definition, 114)? != 0 && !bit(definition, 114, index)?)
        {
            return Ok(false);
        }
    } else if word(definition, 114)? != 0 {
        return Ok(false);
    }
    Ok(true)
}

fn race_movement(warrior: &Warrior, mission: &OpeningMission) -> Result<i8, RuleError> {
    let race = word(&warrior.bytes, 0x32)?;
    mission
        .combat
        .as_ref()
        .and_then(|tables| tables.race_moves.get(race.wrapping_sub(1) as usize))
        .copied()
        .ok_or_else(|| RuleError::Invalid("equipment movement profile missing".into()))
}

fn earned_unloaded_firearm(item: &InventoryRecord) -> bool {
    item.0
        == [
            0, 165, 0, 0, 0, 0, 0, 1, 0, 0, 15, 0, 0, 4, 0, 6, 0, 0, 0, 0, 0, 0,
        ]
}

fn supported_firearm_instance(item: &InventoryRecord) -> Result<(), RuleError> {
    if earned_unloaded_firearm(item) {
        return Ok(());
    }
    let (capacity, ammunition, accuracy) = match item.id() {
        1 => (12, 200, 0),
        2 => (12, 201, 0),
        38 => (3, 203, 35),
        165 => (1, 280, 0),
        _ => return Err(unsupported("other secondary firearm definition")),
    };
    if item.0[10] != 15
        || word(&item.0, 6)? != capacity
        || word(&item.0, 2)? < 0
        || !matches!(word(&item.0, 4)?, 0) && word(&item.0, 4)? != ammunition
        || item.0[9] > 1
        || item.0[14] != 0
        || item.0[15] != accuracy
        || item.0[16..18].iter().any(|value| *value != 0)
        || item.0[19..22].iter().any(|value| *value != 0)
        || (item.0[18] != 0 && !(item.id() == 165 && item.0[18] == 10))
    {
        return Err(unsupported("other secondary firearm instance modifiers"));
    }
    Ok(())
}

fn supported_instance(item: &InventoryRecord) -> Result<(), RuleError> {
    if earned_unloaded_firearm(item) {
        return Ok(());
    }
    if matches!(item.id(), 165 | 700) {
        let supported = item.0[8] == 0
            && item.0[9] <= 1
            && item.0[11..13].iter().all(|value| *value == 0)
            && word(&item.0, 2)? == 1
            && word(&item.0, 6)? == 1
            && if item.id() == 165 {
                item.0[10] == 15
                    && word(&item.0, 4)? == 280
                    && matches!((item.0[13], item.0[18]), (2, 0) | (3, 10))
                    && item.0[14..18].iter().all(|value| *value == 0)
                    && item.0[19..].iter().all(|value| *value == 0)
            } else {
                item.0[10] == 28
                    && word(&item.0, 4)? == 0
                    && item.0[13] == 1
                    && item.0[14..].iter().all(|value| *value == 0)
            };
        return if supported {
            Ok(())
        } else {
            Err(unsupported(
                "other earned equipment instance or enhancement",
            ))
        };
    }
    let expected = match item.id() {
        2 => (15, 1, 0),
        38 => (15, 4, 35),
        400 | 403 => (7, 1, 0),
        420 => (6, 3, 0),
        440 => (4, 1, 0),
        460 => (10, 1, 0),
        600 => (13, 1, 0),
        637 => (20, 2, 0),
        _ => return Err(unsupported("other item definition")),
    };
    if item.0[10] != expected.0
        || item.0[13] != expected.1
        || item.0[15] != expected.2
        || item.0[14] != 0
        || item.0[16..22].iter().any(|value| *value != 0)
        || (matches!(item.id(), 600 | 637)
            && (item.0[8] != 0 || item.0[9] > 1 || item.0[11..13].iter().any(|value| *value != 0)))
        || (item.id() == 2
            && (word(&item.0, 6)? != 12
                || !((word(&item.0, 4)? == 0 && word(&item.0, 2)? == 0)
                    || (word(&item.0, 4)? == 201 && (0..=12).contains(&word(&item.0, 2)?)))))
        || (item.id() == 38
            && (word(&item.0, 6)? != 3
                || (word(&item.0, 4)? != 203
                    && !(word(&item.0, 4)? == 0 && word(&item.0, 2)? == 0))))
        || (item.id() == 637
            && (word(&item.0, 6)? != 250
                || !((word(&item.0, 4)? == 0 && word(&item.0, 2)? == 0)
                    || (word(&item.0, 4)? == 307 && (0..=250).contains(&word(&item.0, 2)?)))))
        || (matches!(item.id(), 400 | 403 | 440 | 460 | 600)
            && item.0[2..8].iter().any(|value| *value != 0))
    {
        return Err(unsupported("other equipment instance or enhancement"));
    }
    Ok(())
}

pub fn collect_item(
    warrior: &Warrior,
    item: &InventoryRecord,
    content: &Content,
    mission: &OpeningMission,
) -> Result<Collection, RuleError> {
    supported_instance(item)?;
    if item.id() == 2 && (word(&item.0, 2)? != 0 || word(&item.0, 4)? != 0) {
        return Err(unsupported("collection of loaded ordinary firearm"));
    }
    if item.id() == 637 && (word(&item.0, 2)? != 0 || word(&item.0, 4)? != 0) {
        return Err(unsupported("collection of charged battery"));
    }
    let definition = item_definition(content, item.id())?;
    let mut result = Collection {
        warrior: warrior.clone(),
        item: item.clone(),
        collected: false,
        sounds: vec![word(definition, 12)?],
        warnings: vec![],
    };
    let count = warrior.inventory()?.len();
    let stacked = if item.id() == 700 {
        warrior
            .inventory()?
            .iter()
            .position(|record| record.id() == 700)
    } else {
        None
    };
    if let Some(slot) = stacked {
        // Native stacking writes quantity and weight before the rejection gates.
        let offset = 0x132 + slot * 22 + 2;
        let quantity = word(&item.0, 2)?;
        let total = word(&result.warrior.bytes, offset)?.wrapping_add(quantity);
        put_word(&mut result.warrior.bytes, offset, total);
        let weight = word(&result.warrior.bytes, 0x9d6)?.wrapping_add(quantity);
        put_word(&mut result.warrior.bytes, 0x9d6, weight);
    }
    if count >= 99 {
        if earned_unloaded_firearm(item) {
            result.sounds.push(6000);
        }
        result.warnings.push(147);
        return Ok(result);
    }
    let needed = word(definition, 20)?
        .wrapping_mul(word(definition, 36)?)
        .wrapping_add(word(&result.warrior.bytes, 0x9d6)?)
        .wrapping_add(word(definition, 16)?) as u16;
    if needed > word(&warrior.bytes, 0x9d8)? as u16 {
        if earned_unloaded_firearm(item) {
            result.sounds.push(6000);
        }
        result.warnings.push(19);
        return Ok(result);
    }
    if stacked.is_some() {
        combat::calculate_movement(&mut result.warrior, race_movement(warrior, mission)?)?;
        result.item.0[..2].fill(0);
        result.collected = true;
        return Ok(result);
    }
    let destination = &mut result.warrior.bytes[0x132 + count * 22..0x132 + (count + 1) * 22];
    destination[..8].copy_from_slice(&item.0[..8]);
    destination[..2].copy_from_slice(&word(definition, 102)?.to_be_bytes());
    destination[8] = word(definition, 34)? as u8;
    destination[9] = 0;
    destination[10] = word(definition, 26)? as u8;
    destination[13..].copy_from_slice(&item.0[13..]);
    put_word(&mut result.warrior.bytes, 0x1a, (count + 1) as i16);
    calculate_weight(&mut result.warrior, content)?;
    combat::calculate_movement(&mut result.warrior, race_movement(warrior, mission)?)?;
    result.item.0[..2].fill(0);
    result.collected = true;
    Ok(result)
}

fn put_word(bytes: &mut [u8], offset: usize, value: i16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
}

fn armor_slot(definition: &[u8]) -> Result<usize, RuleError> {
    let class = word(definition, 26)?;
    if !matches!(class, 4 | 6 | 7 | 10 | 13 | 20) {
        return Err(unsupported("other armor class"));
    }
    Ok(0xc6 + class as usize * 2)
}

fn adjust_armor(
    warrior: &mut [u8],
    definition: &[u8],
    item: &InventoryRecord,
    equipped: bool,
) -> Result<(), RuleError> {
    let armor = word(warrior, 0x2c)?;
    let increment = word(definition, 94)?;
    let enhancement = i16::from(item.0[19] as i8);
    let armor = if equipped {
        armor.wrapping_add(increment).wrapping_add(enhancement)
    } else {
        armor.wrapping_sub(increment).wrapping_sub(enhancement)
    };
    put_word(warrior, 0x2c, armor.max(0));
    let movement = word(warrior, 0x30)?;
    let increment = i16::from(definition[5] as i8);
    put_word(
        warrior,
        0x30,
        if equipped {
            movement.wrapping_add(increment)
        } else {
            movement.wrapping_sub(increment)
        },
    );
    for (source, destination) in [(140, 0x114), (64, 0x116), (66, 0x118)] {
        let increment = word(definition, source)?;
        if increment < 0 {
            let previous = word(warrior, destination)?;
            put_word(
                warrior,
                destination,
                if equipped {
                    previous.wrapping_sub(increment)
                } else {
                    previous.wrapping_add(increment)
                },
            );
        }
    }
    Ok(())
}

fn adjust_shield(
    warrior: &mut [u8],
    definition: &[u8],
    item: &InventoryRecord,
    equipped: bool,
) -> Result<(), RuleError> {
    let chance = word(definition, 88)?.wrapping_add(i16::from(item.0[14] as i8));
    let cost = word(definition, 92)?.wrapping_add(i16::from(item.0[21] as i8));
    for (offset, increment) in [(0x24, chance), (0x2a, cost)] {
        let previous = word(warrior, offset)?;
        let next = if equipped {
            previous.wrapping_add(increment)
        } else {
            previous.wrapping_sub(increment)
        };
        put_word(warrior, offset, next.max(0));
    }
    if word(definition, 90)? != 0 {
        for (offset, increment) in [
            (0x28, word(definition, 148)?.wrapping_add(word(&item.0, 6)?)),
            (0x26, word(&item.0, 2)?),
        ] {
            let previous = word(warrior, offset)?;
            let next = if equipped {
                previous.wrapping_add(increment)
            } else {
                previous.wrapping_sub(increment)
            };
            put_word(warrior, offset, next.max(0));
        }
    }
    adjust_armor(warrior, definition, item, equipped)?;
    let defense = i16::from(definition[3] as i8).wrapping_add(i16::from(item.0[16] as i8));
    let previous = word(warrior, 8)?;
    let next = if equipped {
        previous.wrapping_add(defense)
    } else {
        previous.wrapping_sub(defense).max(0)
    };
    put_word(warrior, 8, next);
    for (source, destination) in [(68, 0x11a), (70, 0x11c), (72, 0x11e)] {
        let increment = word(definition, source)?;
        if increment < 0 {
            let previous = word(warrior, destination)?;
            put_word(
                warrior,
                destination,
                if equipped {
                    previous.wrapping_sub(increment)
                } else {
                    previous.wrapping_add(increment)
                },
            );
        }
    }
    Ok(())
}

fn remove_inner(
    warrior: &mut Warrior,
    slot: usize,
    content: &Content,
    mission: &OpeningMission,
) -> Result<(), RuleError> {
    let item = warrior
        .inventory()?
        .get(slot)
        .cloned()
        .ok_or_else(|| RuleError::Invalid("equipment slot missing".into()))?;
    if matches!(item.id(), 1 | 2 | 38 | 165) {
        supported_firearm_instance(&item)?;
    } else {
        supported_instance(&item)?;
    }
    let definition = item_definition(content, item.id())?;
    let w = &mut warrior.bytes;
    if matches!(item.id(), 1 | 2 | 38 | 165) {
        for offset in [0xe4, 0x126, 0x12c] {
            put_word(w, offset, 0);
        }
        w[0x9dd] = 0;
        let value = word(w, 4)?.wrapping_sub(i16::from(item.0[15] as i8));
        put_word(w, 4, value);
        let range = word(w, 0x130)?
            .wrapping_sub(i16::from(item.0[20] as i8).wrapping_add(word(definition, 60)?));
        put_word(w, 0x130, range);
    } else {
        put_word(w, armor_slot(definition)?, 0);
        if matches!(item.id(), 600 | 637) {
            adjust_shield(w, definition, &item, false)?;
            if word(w, 0xe0)? == 0 && word(w, 0xe2)? == 0 {
                put_word(w, 0x24, 0);
                put_word(w, 0x2a, 0);
            }
            if word(w, 0xee)? == 0 {
                put_word(w, 0x24, 0);
            }
        } else {
            adjust_armor(w, definition, &item, false)?;
        }
    }
    w[0x132 + slot * 22 + 9] = 0;
    let movement = race_movement(warrior, mission)?;
    combat::calculate_movement(warrior, movement)
}

pub fn wear(
    warrior: &Warrior,
    content: &Content,
    mission: &OpeningMission,
    slot: usize,
    in_combat: bool,
) -> Result<EquipmentResult, RuleError> {
    let item = warrior
        .inventory()?
        .get(slot)
        .cloned()
        .ok_or_else(|| RuleError::Invalid("equipment slot missing".into()))?;
    let firearm = matches!(item.id(), 1 | 2 | 38 | 165);
    if firearm {
        supported_firearm_instance(&item)?;
    } else {
        supported_instance(&item)?;
    }
    if item.0[9] != 0 {
        return Err(unsupported("already equipped selection"));
    }
    let definition = item_definition(content, item.id())?;
    let mut result = EquipmentResult {
        warrior: warrior.clone(),
        changed: false,
        sounds: vec![],
        warnings: vec![],
    };
    if !can_use(warrior, definition, mission)? {
        result.warnings.push(29);
        result.sounds.push(6000);
        return Ok(result);
    }
    if word(&warrior.bytes, 0x3a)? < i16::from(item.0[13] as i8) {
        result.sounds.push(6000);
        return Ok(result);
    }
    if !firearm && word(&warrior.bytes, armor_slot(definition)?)? != 0 {
        return Err(unsupported("replacement armor"));
    }
    if item.id() == 600 {
        if word(&warrior.bytes, 0xea)? == 0 && word(&warrior.bytes, 0xe2)? != 0 {
            result.warnings.push(146);
            return Ok(result);
        }
        if word(&warrior.bytes, 0x26)?
            < word(&warrior.bytes, 0x2a)?.wrapping_add(word(definition, 92)?)
        {
            result.warnings.push(144);
            return Ok(result);
        }
    }
    if firearm && word(&warrior.bytes, 0xe4)? != 0 {
        let old_slot = warrior
            .inventory()?
            .iter()
            .position(|old| old.0[10] == 15 && old.0[9] != 0)
            .ok_or_else(|| RuleError::Invalid("equipped secondary record missing".into()))?;
        remove_inner(&mut result.warrior, old_slot, content, mission)?;
    }
    let w = &mut result.warrior.bytes;
    if firearm {
        for (offset, value) in [
            (0xe4, item.id()),
            (0x12c, item.id()),
            (0x126, word(&item.0, 4)?),
        ] {
            put_word(w, offset, value);
        }
        let range = word(w, 0x130)?
            .wrapping_add(i16::from(item.0[20] as i8).wrapping_add(word(definition, 60)?));
        put_word(w, 0x130, range);
        w[0x9dd] = 1;
        let value = word(w, 4)?.wrapping_add(i16::from(item.0[15] as i8));
        put_word(w, 4, value);
    } else {
        put_word(w, armor_slot(definition)?, word(definition, 102)?);
        if matches!(item.id(), 600 | 637) {
            adjust_shield(w, definition, &item, true)?;
        } else {
            adjust_armor(w, definition, &item, true)?;
        }
    }
    w[0x132 + slot * 22 + 9] = 1;
    let movement = race_movement(warrior, mission)?;
    combat::calculate_movement(&mut result.warrior, movement)?;
    if in_combat {
        let ap = word(&result.warrior.bytes, 0x40)?.wrapping_sub(1);
        put_word(&mut result.warrior.bytes, 0x40, ap);
    }
    result.changed = true;
    result.sounds.push(word(definition, 12)?);
    Ok(result)
}

pub fn remove(
    warrior: &Warrior,
    content: &Content,
    mission: &OpeningMission,
    slot: usize,
) -> Result<EquipmentResult, RuleError> {
    let item = warrior
        .inventory()?
        .get(slot)
        .cloned()
        .ok_or_else(|| RuleError::Invalid("equipment slot missing".into()))?;
    if matches!(item.id(), 1 | 2 | 38 | 165) {
        supported_firearm_instance(&item)?;
    } else {
        supported_instance(&item)?;
    }
    if item.0[9] == 0 {
        return Err(unsupported("unequipped selection"));
    }
    if item.id() == 637 && (word(&warrior.bytes, 0xe0)? != 0 || word(&warrior.bytes, 0xe2)? != 0) {
        return Ok(EquipmentResult {
            warrior: warrior.clone(),
            changed: false,
            sounds: vec![],
            warnings: vec![145],
        });
    }
    let mut result = EquipmentResult {
        warrior: warrior.clone(),
        changed: true,
        sounds: vec![word(item_definition(content, item.id())?, 12)?],
        warnings: vec![],
    };
    remove_inner(&mut result.warrior, slot, content, mission)?;
    Ok(result)
}

fn mission<'a>(state: &Session, content: &'a Content) -> Result<&'a OpeningMission, RuleError> {
    content
        .map(state.exploration.as_ref().map_or(0, |p| p.map))
        .ok_or_else(|| RuleError::Invalid("equipment mission missing".into()))
}

fn admission(state: &Session, warrior: usize, needs_ap: bool) -> Result<(), RuleError> {
    if warrior >= state.warriors.len()
        || !matches!(state.phase, Phase::Exploration | Phase::Combat)
        || state
            .exploration
            .as_ref()
            .is_some_and(|p| p.presentation.is_some())
    {
        return Err(RuleError::Invalid(
            "items require unobstructed mission".into(),
        ));
    }
    if state.phase == Phase::Combat
        && (state.combat.as_ref().and_then(combat::Combat::active_body) != Some(warrior as i16)
            || (needs_ap && word(&state.warriors[warrior].bytes, 0x40)? < 1))
    {
        return Err(RuleError::Invalid(
            "items require active warrior with an action point".into(),
        ));
    }
    Ok(())
}

fn events(sounds: Vec<i16>, warnings: Vec<i16>) -> Vec<Event> {
    sounds
        .into_iter()
        .map(|resource_id| Event::Sound { resource_id })
        .chain(
            warnings
                .into_iter()
                .map(|warning| Event::CollectionRejected { warning }),
        )
        .collect()
}

pub fn take_crate(
    state: &mut Session,
    content: &Content,
    slot: usize,
    warrior: usize,
) -> Result<Vec<Event>, RuleError> {
    admission(state, warrior, true)?;
    if state.phase != Phase::Exploration {
        return Err(unsupported("crate access in combat"));
    }
    let item = state
        .crate_items
        .get(slot)
        .ok_or_else(|| RuleError::Invalid("crate slot missing".into()))?;
    let item_id = item.id();
    let result = collect_item(
        &state.warriors[warrior],
        item,
        content,
        mission(state, content)?,
    )?;
    state.warriors[warrior] = result.warrior;
    if result.collected {
        state.crate_items[slot] = result.item;
    }
    let mut occupied: Vec<_> = state
        .crate_items
        .iter()
        .filter(|record| record.id() != 0)
        .cloned()
        .collect();
    occupied.resize(state.crate_items.len(), InventoryRecord([0; 22]));
    state.crate_items = occupied;
    state.load_item_work(content, item_id)?;
    Ok(events(result.sounds, result.warnings))
}

pub fn equip(
    state: &mut Session,
    content: &Content,
    warrior: usize,
    slot: usize,
) -> Result<Vec<Event>, RuleError> {
    admission(state, warrior, true)?;
    let item = state.warriors[warrior]
        .inventory()?
        .get(slot)
        .cloned()
        .ok_or_else(|| RuleError::Invalid("equipment slot missing".into()))?;
    let item_id = item.id();
    let mut result = wear(
        &state.warriors[warrior],
        content,
        mission(state, content)?,
        slot,
        state.phase == Phase::Combat,
    )?;
    if !result.changed {
        return Err(RuleError::Invalid(
            "warrior cannot equip selected item".into(),
        ));
    }
    items::compact(&mut result.warrior)?;
    state.warriors[warrior] = result.warrior;
    state.load_item_work(content, item_id)?;
    let selected = state
        .health_context
        .as_ref()
        .map_or(state.selected_warrior, |context| context.selected_warrior);
    let mut health =
        crate::story_health::context(&state.warriors, selected, state.health_context.as_ref());
    health.affected[warrior] = 1;
    state.health_context = Some(health);
    Ok(events(result.sounds, result.warnings))
}

pub fn unequip(
    state: &mut Session,
    content: &Content,
    warrior: usize,
    slot: usize,
) -> Result<Vec<Event>, RuleError> {
    admission(state, warrior, false)?;
    let item = state.warriors[warrior]
        .inventory()?
        .get(slot)
        .cloned()
        .ok_or_else(|| RuleError::Invalid("equipment slot missing".into()))?;
    let item_id = item.id();
    let mut result = remove(
        &state.warriors[warrior],
        content,
        mission(state, content)?,
        slot,
    )?;
    if !result.changed {
        return Err(RuleError::Invalid(
            "warrior cannot remove selected item".into(),
        ));
    }
    items::compact(&mut result.warrior)?;
    state.warriors[warrior] = result.warrior;
    state.load_item_work(content, item_id)?;
    if let Some(health) = state.health_context.as_mut() {
        health.affected[warrior] = 1;
    }
    Ok(events(result.sounds, result.warnings))
}
