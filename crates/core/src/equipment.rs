use crate::{Event, Phase, RuleError, Session, combat, items, story_collection::Collection};
use centurions_content::{Content, InventoryRecord, Warrior, mission::OpeningMission, span, word};

pub struct EquipmentResult {
    pub warrior: Warrior,
    pub changed: bool,
    pub sounds: Vec<i16>,
    pub warnings: Vec<i16>,
    pub record: Option<InventoryRecord>,
}

fn unsupported(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "equipment",
        dependency,
    }
}

/// Display copies all nine visible rows; updateshop20 then copies a selected owned row.
pub(crate) fn inspect(
    state: &mut Session,
    members: [usize; 2],
    scroll: [usize; 2],
    selected: Option<(usize, usize)>,
) -> Result<Vec<Event>, RuleError> {
    if state.exploration.as_ref().is_some_and(|position| {
        position.presentation.as_ref().is_some_and(|presentation| {
            !matches!(
                presentation,
                crate::Presentation::Shop | crate::Presentation::Typed { .. }
            )
        })
    }) {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    let shop = state
        .exploration
        .as_ref()
        .is_some_and(|position| position.presentation == Some(crate::Presentation::Shop));
    for side in 0..2 {
        if shop && side == 1 {
            // Shop rows partially overwrite the previous temporary record.
            state.inventory_record = None;
            continue;
        }
        let warrior = state
            .warriors
            .get(members[side])
            .ok_or("inventory member missing")?;
        let row = scroll[side]
            .checked_add(8)
            .filter(|row| *row < 100)
            .ok_or("inventory scroll outside native rows")?;
        state.inventory_record = Some(InventoryRecord(
            span(&warrior.bytes, 0x132 + row * 22, 22)?
                .try_into()
                .unwrap(),
        ));
    }
    if let Some((side, slot)) = selected {
        if side > 1 || slot >= if shop && side == 1 { 1000 } else { 100 } {
            return Err("inventory selection outside native rows".into());
        }
        if !(shop && side == 1) {
            state.inventory_record = Some(
                state.warriors[members[side]]
                    .inventory()?
                    .get(slot)
                    .cloned()
                    .ok_or("inventory selection missing")?,
            );
        }
    }
    Ok(vec![])
}

fn item_definition(content: &Content, id: i16) -> Result<&[u8], RuleError> {
    content
        .items
        .get(usize::from(id.unsigned_abs()))
        .map(Vec::as_slice)
        .ok_or_else(|| RuleError::Invalid("equipment definition missing".into()))
}

/// removeitem checks the battery dependency after its charge-minus-one shortcut.
pub(crate) fn removal_blocked(
    warrior: &Warrior,
    slot: usize,
    definition: &[u8],
) -> Result<bool, RuleError> {
    Ok(word(definition, 26)?.wrapping_abs() == 20
        && word(&warrior.bytes, 0x134 + slot * 22)? != -1
        && (word(&warrior.bytes, 0xe0)? != 0 || word(&warrior.bytes, 0xe2)? != 0))
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

pub fn collect_item(
    warrior: &Warrior,
    item: &InventoryRecord,
    content: &Content,
    mission: &OpeningMission,
) -> Result<Collection, RuleError> {
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

pub(crate) fn remove_inner(
    warrior: &mut Warrior,
    slot: usize,
    content: &Content,
    mission: &OpeningMission,
) -> Result<(), RuleError> {
    let shortcut = word(&warrior.bytes, 0x134 + slot * 22)? == -1;
    *warrior = crate::editor_items::equipment_record(warrior, content, slot, false)?.0;
    if !shortcut {
        combat::calculate_movement(warrior, race_movement(warrior, mission)?)?;
    }
    Ok(())
}

/// dropitem 0x121152 retains the original 29-slot shift and empty-record fill.
pub(crate) fn drop(
    warrior: &mut Warrior,
    slot: usize,
    content: &Content,
    mission: &OpeningMission,
) -> Result<(), RuleError> {
    let item = warrior
        .inventory()?
        .get(slot)
        .cloned()
        .ok_or("drop slot missing")?;
    drop_record(warrior, slot, &item, content, mission, false)
}

pub(crate) fn force_drop(
    warrior: &mut Warrior,
    slot: usize,
    content: &Content,
    mission: &OpeningMission,
) -> Result<(), RuleError> {
    let item = InventoryRecord(
        span(&warrior.bytes, 0x132 + slot * 22, 22)?
            .try_into()
            .unwrap(),
    );
    drop_record(warrior, slot, &item, content, mission, true)
}

fn drop_record(
    warrior: &mut Warrior,
    slot: usize,
    item: &InventoryRecord,
    content: &Content,
    mission: &OpeningMission,
    forced: bool,
) -> Result<(), RuleError> {
    if item.0[9] != 0 {
        if forced {
            *warrior = crate::editor_items::equipment_record_forced(warrior, content, slot)?.0;
        } else {
            remove_inner(warrior, slot, content, mission)?;
        }
    }
    let definition = item_definition(content, item.id())?;
    let weight = word(definition, 36)?
        .wrapping_mul(word(&item.0, 2)?)
        .wrapping_add(word(definition, 16)?);
    let remaining = word(&warrior.bytes, 0x1a)?.wrapping_sub(1);
    let total = word(&warrior.bytes, 0x9d6)?.wrapping_sub(weight);
    for index in slot..29 {
        warrior.bytes.copy_within(
            0x132 + (index + 1) * 22..0x132 + (index + 2) * 22,
            0x132 + index * 22,
        );
    }
    // Original DATA at 0x41fc32 is the 22-byte zero record.
    let empty = &[0; 22];
    for index in remaining as usize..100 {
        warrior.bytes[0x132 + index * 22..0x132 + (index + 1) * 22].copy_from_slice(empty);
    }
    put_word(&mut warrior.bytes, 0x1a, remaining);
    put_word(&mut warrior.bytes, 0x9d6, total);
    combat::calculate_movement(warrior, race_movement(warrior, mission)?)
}

pub fn wear(
    warrior: &Warrior,
    content: &Content,
    mission: &OpeningMission,
    slot: usize,
    in_combat: bool,
) -> Result<EquipmentResult, RuleError> {
    wear_retained(warrior, content, mission, slot, in_combat, None)
}

fn wear_retained(
    warrior: &Warrior,
    content: &Content,
    mission: &OpeningMission,
    slot: usize,
    in_combat: bool,
    retained: Option<&InventoryRecord>,
) -> Result<EquipmentResult, RuleError> {
    let item = warrior
        .inventory()?
        .get(slot)
        .cloned()
        .ok_or_else(|| RuleError::Invalid("equipment slot missing".into()))?;
    if item.0[9] != 0 {
        return Ok(EquipmentResult {
            warrior: warrior.clone(),
            changed: false,
            sounds: vec![],
            warnings: vec![],
            record: retained.cloned(),
        });
    }
    let definition = item_definition(content, item.id())?;
    let mut result = EquipmentResult {
        warrior: warrior.clone(),
        changed: false,
        sounds: vec![],
        warnings: vec![],
        record: retained.cloned(),
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
    let worn = crate::editor_items::retained_equipment(
        warrior,
        content,
        slot,
        true,
        false,
        retained.unwrap_or(&item),
    )?;
    result.warrior = worn.warrior;
    result.warnings = worn.warnings;
    result.record = retained.map(|_| worn.record);
    if result.warrior.bytes[0x132 + slot * 22 + 9] == 0 {
        result.changed = result.warrior != *warrior;
        return Ok(result);
    }
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
    remove_retained(warrior, content, mission, slot, None)
}

fn remove_retained(
    warrior: &Warrior,
    content: &Content,
    mission: &OpeningMission,
    slot: usize,
    retained: Option<&InventoryRecord>,
) -> Result<EquipmentResult, RuleError> {
    let item = warrior
        .inventory()?
        .get(slot)
        .cloned()
        .ok_or_else(|| RuleError::Invalid("equipment slot missing".into()))?;
    if item.0[9] == 0 {
        return Err(unsupported("unequipped selection"));
    }
    if removal_blocked(warrior, slot, item_definition(content, item.id())?)? {
        return Ok(EquipmentResult {
            warrior: warrior.clone(),
            changed: false,
            sounds: vec![],
            warnings: vec![145],
            record: retained.cloned(),
        });
    }
    let mut result = EquipmentResult {
        warrior: warrior.clone(),
        changed: true,
        sounds: vec![word(item_definition(content, item.id())?, 12)?],
        warnings: vec![],
        record: retained.cloned(),
    };
    if let Some(record) = retained {
        let removed =
            crate::editor_items::retained_equipment(warrior, content, slot, false, false, record)?;
        result.warrior = removed.warrior;
        result.record = Some(removed.record);
    } else {
        remove_inner(&mut result.warrior, slot, content, mission)?;
    }
    Ok(result)
}

fn mission<'a>(state: &Session, content: &'a Content) -> Result<&'a OpeningMission, RuleError> {
    content
        .map(state.exploration.as_ref().map_or(0, |p| p.map))
        .ok_or_else(|| RuleError::Invalid("equipment mission missing".into()))
}

fn admission(state: &Session, warrior: usize, needs_ap: bool) -> Result<(), RuleError> {
    if warrior >= state.warriors.len() {
        return Err("inventory warrior outside squad".into());
    }
    if !matches!(state.phase, Phase::Exploration | Phase::Combat)
        || state
            .exploration
            .as_ref()
            .is_some_and(|p| p.presentation.is_some())
    {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    if state.phase == Phase::Combat
        && state.combat.as_ref().and_then(combat::Combat::active_body) != Some(warrior as i16)
    {
        return Err(RuleError::Refused {
            feedback: vec![crate::feedback::NativeFeedback::Warning { index: 10 }],
        });
    }
    if state.phase == Phase::Combat && needs_ap && word(&state.warriors[warrior].bytes, 0x40)? < 1 {
        return Err(RuleError::Refused { feedback: vec![] });
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

/// moveicon's owned-item route collects into the destination before dropping the source.
pub fn transfer(
    state: &mut Session,
    content: &Content,
    from: usize,
    to: usize,
    slot: usize,
) -> Result<Vec<Event>, RuleError> {
    if state.phase == Phase::Combat
        && state.combat.as_ref().and_then(combat::Combat::active_body) != Some(from as i16)
    {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    admission(state, from, false)?;
    if from == to || to >= state.warriors.len() {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    let mission = mission(state, content)?;
    let item = state.warriors[from]
        .inventory()?
        .get(slot)
        .cloned()
        .ok_or("transfer slot missing")?;
    if state.inventory_record.is_some() {
        state.inventory_record = Some(item.clone());
    }
    state.load_item_work(content, item.id())?;
    if state.warriors[to].inventory()?.len() >= 30 {
        return Ok(vec![Event::Feedback(
            crate::feedback::NativeFeedback::Warning { index: 19 },
        )]);
    }
    let collected = if word(item_definition(content, item.id())?, 26)? == 23 {
        crate::story_collection::collect_story_item(
            &state.warriors[to],
            &item,
            content,
            race_movement(&state.warriors[to], mission)?,
        )?
    } else {
        collect_item(&state.warriors[to], &item, content, mission)?
    };
    let mut effects = events(collected.sounds, collected.warnings);
    state.warriors[to] = collected.warrior;
    if collected.collected {
        drop(&mut state.warriors[from], slot, content, mission)?;
        effects.push(Event::ItemTransferred {
            from,
            to,
            item: item.id(),
        });
    }
    Ok(effects)
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
    if state.inventory_record.is_some() {
        state.inventory_record = Some(item.clone());
    }
    let mut result = wear_retained(
        &state.warriors[warrior],
        content,
        mission(state, content)?,
        slot,
        state.phase == Phase::Combat,
        state.inventory_record.as_ref(),
    )?;
    state.inventory_record = result.record.take();
    if !result.changed {
        state.warriors[warrior] = result.warrior;
        state.load_item_work(content, item_id)?;
        return Ok(events(result.sounds, result.warnings));
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
    if state.inventory_record.is_some() {
        state.inventory_record = Some(item.clone());
    }
    let mut result = remove_retained(
        &state.warriors[warrior],
        content,
        mission(state, content)?,
        slot,
        state.inventory_record.as_ref(),
    )?;
    state.inventory_record = result.record.take();
    if !result.changed {
        state.warriors[warrior] = result.warrior;
        state.load_item_work(content, item_id)?;
        return Ok(events(result.sounds, result.warnings));
    }
    items::compact(&mut result.warrior)?;
    state.warriors[warrior] = result.warrior;
    state.load_item_work(content, item_id)?;
    if let Some(health) = state.health_context.as_mut() {
        health.affected[warrior] = 1;
    }
    Ok(events(result.sounds, result.warnings))
}
