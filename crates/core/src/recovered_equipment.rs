//! Complete equipment, dropping and refill control bodies from CODE 1.

use crate::recovered_equipment_ui::{self, EquipmentUiEffect, EquipmentUiServices};
use crate::recovered_feedback::{self, FeedbackEffect};
use crate::recovered_files::FileObservation;
use crate::recovered_install::InstallMemory;
use crate::recovered_inventory_rules::{self, InventoryEffect, InventoryServices};
use crate::recovered_navigation;
use crate::recovered_records::{self, ITEM_TABLE_ADDRESSES, ItemTables, RecordWrite};
use crate::recovered_rendering::RenderingMemory;
use crate::recovered_runtime::{self, RuntimeContext, RuntimeEffect, RuntimeResult};
use crate::{RandomState, RuleError};
use centurions_content::{ITEM_BYTES, WARRIOR_BYTES};

pub trait EquipmentServices: InventoryServices + EquipmentUiServices {
    /// Actual stdio/numeric receipts for the recovered race/caste file children.
    fn profile_observations(
        &mut self,
        race: i16,
        caste: i16,
        memory: &RenderingMemory,
    ) -> Result<Vec<FileObservation>, RuleError>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EquipmentLimits {
    pub maximum_steps: usize,
    pub maximum_polls: usize,
    pub filename_capacity: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EquipmentReturn {
    pub low_word: u16,
    /// None means a typed imported/void child has not established that register half.
    pub upper_word: Option<u16>,
}
impl EquipmentReturn {
    fn long(value: u32) -> Self {
        Self {
            low_word: value as u16,
            upper_word: Some((value >> 16) as u16),
        }
    }
    pub fn known_long(self) -> Option<u32> {
        self.upper_word
            .map(|upper| (u32::from(upper) << 16) | u32::from(self.low_word))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum EquipmentEffect {
    Write(RecordWrite),
    Inventory(InventoryEffect),
    Feedback(FeedbackEffect),
    Ui(EquipmentUiEffect),
    Runtime(RuntimeEffect),
}

#[derive(Clone, Debug, PartialEq)]
pub struct EquipmentPlan {
    pub memory: RenderingMemory,
    pub random: RandomState,
    pub effects: Vec<EquipmentEffect>,
    pub native_return: EquipmentReturn,
}
fn plan(memory: &RenderingMemory, random: &RandomState) -> EquipmentPlan {
    EquipmentPlan {
        memory: memory.clone(),
        random: random.clone(),
        effects: vec![],
        native_return: EquipmentReturn::long(0),
    }
}
pub fn read(memory: &RenderingMemory, address: u32, length: usize) -> Result<Vec<u8>, RuleError> {
    (0..length)
        .map(|i| memory.byte(address.wrapping_add(i as u32)))
        .collect()
}
pub fn sw(memory: &RenderingMemory, address: u32) -> Result<i16, RuleError> {
    Ok(memory.word(address)? as i16)
}
pub fn sb(memory: &RenderingMemory, address: u32) -> Result<i16, RuleError> {
    Ok(i16::from(memory.byte(address)? as i8))
}
pub fn member_address(member: u16) -> u32 {
    0x448aa0_u32.wrapping_add(u32::from(member) * WARRIOR_BYTES as u32)
}
pub fn node_address(member: u16, slot: u16) -> u32 {
    member_address(member)
        .wrapping_add(u32::from(slot) * 22)
        .wrapping_add(0x132)
}
pub fn indexed(base: u32, index: i16) -> u32 {
    base.wrapping_add((i32::from(index) * 2) as u32)
}
fn step(remaining: &mut usize) -> Result<(), RuleError> {
    *remaining = remaining.checked_sub(1).ok_or(RuleError::Unsupported {
        action: "equipment.processing",
        dependency: "the caller's bounded instruction/loop budget is exhausted",
    })?;
    Ok(())
}
fn write(result: &mut EquipmentPlan, address: u32, bytes: &[u8]) -> Result<(), RuleError> {
    read(&result.memory, address, bytes.len())?;
    result.memory.insert_region(address, bytes);
    result.effects.push(EquipmentEffect::Write(RecordWrite {
        address,
        bytes: bytes.to_vec(),
    }));
    Ok(())
}
fn word(result: &mut EquipmentPlan, address: u32, value: i16) -> Result<(), RuleError> {
    write(result, address, &value.to_be_bytes())
}
fn add_word(result: &mut EquipmentPlan, address: u32, value: i16) -> Result<(), RuleError> {
    let value = sw(&result.memory, address)?.wrapping_add(value);
    word(result, address, value)
}
fn subtract_word(result: &mut EquipmentPlan, address: u32, value: i16) -> Result<(), RuleError> {
    let value = sw(&result.memory, address)?.wrapping_sub(value);
    word(result, address, value)
}
fn clamp_negative(result: &mut EquipmentPlan, address: u32) -> Result<(), RuleError> {
    if sw(&result.memory, address)? < 0 {
        word(result, address, 0)?;
    }
    Ok(())
}
pub fn item_tables(memory: &RenderingMemory, id: i16) -> Result<[Vec<u8>; 5], RuleError> {
    let mut backing: [Vec<u8>; 5] = std::array::from_fn(|_| vec![]);
    let absolute = id.wrapping_abs();
    if let Ok(family) = usize::try_from(absolute / 200)
        && family < 5
    {
        let offset = (absolute % 200) as usize * ITEM_BYTES;
        let bytes = read(
            memory,
            ITEM_TABLE_ADDRESSES[family].wrapping_add(offset as u32),
            ITEM_BYTES,
        )?;
        backing[family].resize(offset + ITEM_BYTES, 0);
        backing[family][offset..].copy_from_slice(&bytes);
    }
    Ok(backing)
}
fn load_item(result: &mut EquipmentPlan, id: i16, ammo: bool) -> Result<(), RuleError> {
    let destination = if ammo { 0x4536d2 } else { 0x45363c };
    let current = read(&result.memory, destination, ITEM_BYTES)?
        .try_into()
        .expect("item record");
    let backing = item_tables(&result.memory, id)?;
    let tables = ItemTables {
        families: std::array::from_fn(|i| backing[i].as_slice()),
    };
    let child = if ammo {
        recovered_records::loadammo(&tables, &current, id)?
    } else {
        recovered_records::loaditem(&tables, &current, id)?
    };
    for effect in child.writes {
        write(result, effect.address, &effect.bytes)?;
    }
    Ok(())
}
fn copy_record(
    result: &mut EquipmentPlan,
    source: u32,
    destination: u32,
    length: usize,
) -> Result<(), RuleError> {
    for offset in (0..length / 4 * 4).step_by(4) {
        let bytes = read(&result.memory, source.wrapping_add(offset as u32), 4)?;
        write(result, destination.wrapping_add(offset as u32), &bytes)?;
    }
    let offset = length / 4 * 4;
    if offset < length {
        let bytes = read(
            &result.memory,
            source.wrapping_add(offset as u32),
            length - offset,
        )?;
        write(result, destination.wrapping_add(offset as u32), &bytes)?;
    }
    Ok(())
}
fn sound(
    result: &mut EquipmentPlan,
    id: i16,
    services: &(impl EquipmentServices + ?Sized),
) -> Result<(), RuleError> {
    let child = recovered_feedback::sound(&result.memory, id, services)?;
    result.memory = child.memory;
    result
        .effects
        .extend(child.effects.into_iter().map(EquipmentEffect::Feedback));
    Ok(())
}
fn warn(
    result: &mut EquipmentPlan,
    frame: u32,
    message: i16,
    limits: EquipmentLimits,
    services: &(impl EquipmentServices + ?Sized),
) -> Result<(), RuleError> {
    let child = recovered_feedback::warn(
        &result.memory,
        frame,
        message,
        limits.maximum_polls,
        services,
    )?;
    result.memory = child.memory;
    result
        .effects
        .extend(child.effects.into_iter().map(EquipmentEffect::Feedback));
    Ok(())
}
fn remove(
    result: &mut EquipmentPlan,
    frame: u32,
    member: u16,
    slot: u16,
    forced: bool,
    limits: EquipmentLimits,
    services: &(impl EquipmentServices + ?Sized),
) -> Result<u32, RuleError> {
    write(
        result,
        frame.wrapping_add(12),
        &u32::from(forced).to_be_bytes(),
    )?;
    word(result, frame.wrapping_add(10), slot as i16)?;
    word(result, frame.wrapping_add(8), member as i16)?;
    let child = recovered_inventory_rules::removeitem(
        &result.memory,
        frame,
        limits.maximum_polls,
        services,
    )?;
    let value = child.native_return;
    result.memory = child.memory;
    result
        .effects
        .extend(child.effects.into_iter().map(EquipmentEffect::Inventory));
    Ok(value)
}
fn movement(result: &mut EquipmentPlan, member: u16) -> Result<(), RuleError> {
    let warrior = match member {
        0xfffe => 0x447e5e,
        0xffff => 0x44721c,
        _ => member_address(member),
    };
    let race = sw(&result.memory, warrior.wrapping_add(0x32))?.wrapping_sub(1);
    let race_address = 0x435664_u32.wrapping_add(i32::from(race) as u32);
    let start = warrior.min(race_address);
    let end = warrior
        .checked_add(WARRIOR_BYTES as u32)
        .ok_or_else(|| RuleError::Invalid("equipment movement projection wraps".into()))?
        .max(
            race_address
                .checked_add(1)
                .ok_or_else(|| RuleError::Invalid("equipment race projection wraps".into()))?,
        );
    let length = (end - start) as usize;
    if length > 1024 * 1024 {
        return Err(RuleError::Invalid(
            "equipment movement projection exceeds one MiB".into(),
        ));
    }
    let memory = InstallMemory {
        base_address: start,
        bytes: read(&result.memory, start, length)?,
    };
    let child = recovered_navigation::movecalc(&memory, member)?;
    for effect in child.writes {
        write(result, effect.address, &effect.bytes)?;
    }
    Ok(())
}
fn update_items(
    result: &mut EquipmentPlan,
    frame: u32,
    first: i16,
    last: i16,
    services: &mut (impl EquipmentServices + ?Sized),
) -> Result<(), RuleError> {
    word(result, frame.wrapping_add(10), last)?;
    word(result, frame.wrapping_add(8), first)?;
    let child = recovered_equipment_ui::update_items(&result.memory, frame, services)?;
    result.memory = child.memory;
    result
        .effects
        .extend(child.effects.into_iter().map(EquipmentEffect::Ui));
    Ok(())
}

/// dropitem, 0x121152. Item, member, slot and sound gate remain raw caller WORDs.
pub fn dropitem(
    memory: &RenderingMemory,
    random: &RandomState,
    frame: u32,
    limits: EquipmentLimits,
    services: &mut (impl EquipmentServices + ?Sized),
) -> Result<EquipmentPlan, RuleError> {
    drop_body(memory, random, frame, limits, services, false)
}
/// forcedropitem, 0x1212a8. The removeitem result is discarded by this caller.
pub fn forcedropitem(
    memory: &RenderingMemory,
    random: &RandomState,
    frame: u32,
    limits: EquipmentLimits,
    services: &mut (impl EquipmentServices + ?Sized),
) -> Result<EquipmentPlan, RuleError> {
    drop_body(memory, random, frame, limits, services, true)
}
fn drop_body(
    memory: &RenderingMemory,
    random: &RandomState,
    frame: u32,
    limits: EquipmentLimits,
    services: &mut (impl EquipmentServices + ?Sized),
    forced: bool,
) -> Result<EquipmentPlan, RuleError> {
    let mut result = plan(memory, random);
    let mut remaining = limits.maximum_steps;
    let member = result.memory.word(frame.wrapping_add(8))?;
    let slot = result.memory.word(frame.wrapping_add(12))?;
    let item = sw(&result.memory, frame.wrapping_add(10))?;
    load_item(&mut result, item, false)?;
    if result.memory.word(frame.wrapping_add(14))? != 0 {
        sound(&mut result, 655, services)?;
    }
    let node = node_address(member, slot);
    if result.memory.byte(node.wrapping_add(9))? != 0 {
        let child_frame = frame.wrapping_sub(if forced { 28 } else { 32 });
        let removed = remove(
            &mut result,
            child_frame,
            member,
            slot,
            forced,
            limits,
            services,
        )?;
        if !forced && removed as u16 == 0 {
            return Ok(result);
        }
    }
    let weight = sw(&result.memory, 0x453660)?
        .wrapping_mul(sw(&result.memory, node.wrapping_add(2))?)
        .wrapping_add(sw(&result.memory, 0x45364c)?);
    subtract_word(
        &mut result,
        member_address(member).wrapping_add(0x9d6),
        weight,
    )?;
    let mut index = slot as i16;
    while index < 29 {
        step(&mut remaining)?;
        let source = node_address(member, index.wrapping_add(1) as u16);
        let destination = node_address(member, index as u16);
        copy_record(&mut result, source, destination, 22)?;
        index = index.wrapping_add(1);
    }
    subtract_word(&mut result, member_address(member).wrapping_add(0x1a), 1)?;
    index = sw(&result.memory, member_address(member).wrapping_add(0x1a))?;
    while index < 100 {
        step(&mut remaining)?;
        let destination = node_address(member, index as u16);
        copy_record(&mut result, 0x41fc32, destination, 22)?;
        index = index.wrapping_add(1);
    }
    movement(&mut result, member)?;
    Ok(result)
}

fn can_use(
    result: &mut EquipmentPlan,
    frame: u32,
    item: i16,
    race: i16,
    caste: i16,
    limits: EquipmentLimits,
    services: &mut (impl EquipmentServices + ?Sized),
) -> Result<u32, RuleError> {
    word(result, frame.wrapping_add(12), caste)?;
    word(result, frame.wrapping_add(10), race)?;
    word(result, frame.wrapping_add(8), item)?;
    let observations = if race != 0 || caste != 0 {
        services.profile_observations(race, caste, &result.memory)?
    } else {
        vec![]
    };
    let child = recovered_inventory_rules::canuse(
        &result.memory,
        frame,
        limits.filename_capacity,
        &observations,
    )?;
    let value = child.native_return;
    result.memory = child.memory;
    result
        .effects
        .extend(child.effects.into_iter().map(EquipmentEffect::Inventory));
    Ok(value)
}
fn c_string(memory: &RenderingMemory, address: u32) -> Result<Vec<u8>, RuleError> {
    let mut bytes = vec![];
    for offset in 0..=memory.bytes.len() {
        let byte = memory.byte(address.wrapping_add(offset as u32))?;
        bytes.push(byte);
        if byte == 0 {
            return Ok(bytes);
        }
    }
    Err(RuleError::Invalid("unterminated equipment C string".into()))
}
fn strength_message(
    result: &mut EquipmentPlan,
    frame: u32,
    required: i16,
    emit: i16,
    limits: EquipmentLimits,
    services: &mut (impl EquipmentServices + ?Sized),
) -> Result<(), RuleError> {
    let text = frame.wrapping_sub(0x108);
    let source = c_string(&result.memory, 0x400548)?;
    write(result, text, &source)?;
    let converted = services.number_to_string(i32::from(required))?;
    result
        .effects
        .push(EquipmentEffect::Ui(EquipmentUiEffect::ItemUi(
            crate::recovered_item_ui::ItemUiEffect::NumberToString {
                value: i32::from(required),
                destination: 0x479ed8,
                result: converted.clone(),
            },
        )));
    write(result, 0x479ed8, converted.raw())?;
    for (offset, byte) in converted.payload().iter().enumerate() {
        write(result, 0x479ed8_u32.wrapping_add(offset as u32), &[*byte])?;
    }
    write(
        result,
        0x479ed8_u32.wrapping_add(converted.payload().len() as u32),
        &[0],
    )?;
    let length = c_string(&result.memory, text)?.len() - 1;
    let suffix = c_string(&result.memory, 0x479ed8)?;
    write(result, text.wrapping_add(length as u32), &suffix)?;
    if emit != 0 {
        let child_frame = frame.wrapping_sub(0x138);
        write(
            result,
            child_frame.wrapping_add(16),
            &6000_u32.to_be_bytes(),
        )?;
        write(
            result,
            child_frame.wrapping_add(12),
            &0x00320032_u32.to_be_bytes(),
        )?;
        write(result, child_frame.wrapping_add(8), &text.to_be_bytes())?;
        let child = recovered_feedback::flashmessage(
            &result.memory,
            child_frame,
            text,
            50,
            50,
            0,
            6000,
            limits.maximum_polls,
            services,
        )?;
        result.memory = child.memory;
        result
            .effects
            .extend(child.effects.into_iter().map(EquipmentEffect::Feedback));
    }
    Ok(())
}
fn wear_return(result: &mut EquipmentPlan, upper: Option<u16>) -> Result<(), RuleError> {
    let upper = if result.memory.word(0x438776)? != 0 {
        let active = result.memory.word(0x4435d2)?;
        let product = u32::from(active) * WARRIOR_BYTES as u32;
        let amount = sw(&result.memory, 0x4471ea)?;
        subtract_word(result, member_address(active).wrapping_add(0x40), amount)?;
        Some((product >> 16) as u16)
    } else {
        upper
    };
    result.native_return = EquipmentReturn {
        low_word: result.memory.word(0x4471ea)?,
        upper_word: upper,
    };
    Ok(())
}
fn wear_warning(
    result: &mut EquipmentPlan,
    frame: u32,
    emit: i16,
    message: i16,
    upper: Option<u16>,
    limits: EquipmentLimits,
    services: &(impl EquipmentServices + ?Sized),
) -> Result<Option<u16>, RuleError> {
    if emit != 0 {
        warn(result, frame.wrapping_sub(0x126), message, limits, services)?;
        Ok(Some(0))
    } else {
        Ok(upper)
    }
}
fn wear_attribute(
    result: &mut EquipmentPlan,
    warrior: u32,
    selector_address: u32,
    upper: &mut Option<u16>,
) -> Result<(), RuleError> {
    let selector = sw(&result.memory, selector_address)?;
    if selector == 0 {
        return Ok(());
    }
    if selector < 0 {
        let index = selector.wrapping_abs().wrapping_sub(1);
        let address = indexed(warrior.wrapping_add(0xae), index);
        *upper = Some(((i32::from(index) * 2) as u32 >> 16) as u16);
        let value = sw(&result.memory, 0x45366e)?;
        add_word(result, address, value)?;
    } else if selector < 16 {
        let index = selector.wrapping_sub(1);
        *upper = Some(0);
        let address = indexed(warrior.wrapping_add(0xf4), index);
        let value = sw(&result.memory, 0x45366e)?;
        add_word(result, address, value)?;
    } else {
        // abs_short retains the prior upper WORD; only its low WORD participates.
        let amount = sw(&result.memory, 0x45366e)?.wrapping_abs();
        subtract_word(result, indexed(0x442562, selector.wrapping_sub(30)), amount)?;
    }
    Ok(())
}

/// wear, 0x164a0e. Both success and native partial-warning writes remain in the plan.
pub fn wear(
    memory: &RenderingMemory,
    random: &RandomState,
    frame: u32,
    limits: EquipmentLimits,
    services: &mut (impl EquipmentServices + ?Sized),
) -> Result<EquipmentPlan, RuleError> {
    let mut result = plan(memory, random);
    let mut remaining = limits.maximum_steps;
    wear_body(&mut result, frame, limits, &mut remaining, services)?;
    Ok(result)
}
fn wear_body(
    result: &mut EquipmentPlan,
    frame: u32,
    limits: EquipmentLimits,
    remaining: &mut usize,
    services: &mut (impl EquipmentServices + ?Sized),
) -> Result<(), RuleError> {
    let member = result.memory.word(frame.wrapping_add(8))?;
    let slot = result.memory.word(frame.wrapping_add(10))?;
    let mut emit = sw(&result.memory, frame.wrapping_add(12))?;
    let warrior = member_address(member);
    let node = node_address(member, slot);
    let member_upper = Some(((u32::from(member) * WARRIOR_BYTES as u32) >> 16) as u16);
    loop {
        step(remaining)?;
        if result.memory.word(0x438776)? != 0 {
            let active = sw(&result.memory, 0x447194)?;
            if result.memory.word(indexed(0x4470b8, active))? != member && emit != 0 {
                warn(result, frame.wrapping_sub(0x126), 10, limits, services)?;
                result.native_return = EquipmentReturn::long(0);
                return Ok(());
            }
        }
        word(result, frame.wrapping_sub(2), 640)?;
        word(result, frame.wrapping_sub(4), 460)?;
        let width = sw(&result.memory, frame.wrapping_sub(2))?.wrapping_sub(356);
        word(result, frame.wrapping_sub(6), width)?;
        let height = sw(&result.memory, frame.wrapping_sub(4))?.wrapping_sub(119);
        word(result, frame.wrapping_sub(8), height)?;
        word(result, 0x4471ea, 0)?;
        let item = sw(&result.memory, node)?;
        load_item(result, item, false)?;
        let original_type = sw(&result.memory, 0x453656)?.wrapping_abs();
        word(result, 0x453656, original_type)?;
        if result.memory.word(0x438776)? != 0
            && result.memory.word(warrior.wrapping_add(0x90))? != 0
        {
            emit = 0;
        }
        let caste = sw(&result.memory, warrior.wrapping_add(0x34))?;
        let race = sw(&result.memory, warrior.wrapping_add(0x32))?;
        let item = sw(&result.memory, node)?;
        if can_use(
            result,
            frame.wrapping_sub(0x12a),
            item,
            race,
            caste,
            limits,
            services,
        )? as u16
            == 0
        {
            let upper = wear_warning(result, frame, emit, 29, Some(0), limits, services)?;
            wear_return(result, upper)?;
            return Ok(());
        }
        if sw(&result.memory, 0x453656)? > 23 {
            let upper = wear_warning(result, frame, emit, 48, Some(0), limits, services)?;
            wear_return(result, upper)?;
            return Ok(());
        }
        if sb(&result.memory, node.wrapping_add(13))?
            > sw(&result.memory, warrior.wrapping_add(0x3a))?
            && result.memory.word(0x4471ec)? == 0
        {
            let required = sb(&result.memory, node.wrapping_add(13))?;
            strength_message(result, frame, required, emit, limits, services)?;
            wear_return(result, None)?;
            return Ok(());
        }
        let mut upper = member_upper;
        if sw(&result.memory, 0x453656)? > 1 {
            let kind = sw(&result.memory, 0x453656)?;
            upper = Some(((i32::from(kind) * 2) as u32 >> 16) as u16);
            if result
                .memory
                .word(indexed(warrior.wrapping_add(0xc6), kind))?
                != 0
            {
                let mut search = 0_i16;
                let mut replaced = false;
                while search <= sw(&result.memory, warrior.wrapping_add(0x1a))? {
                    step(remaining)?;
                    let old_node = node_address(member, search as u16);
                    let id = sw(&result.memory, old_node)?;
                    load_item(result, id, false)?;
                    let kind = sw(&result.memory, 0x453656)?.wrapping_abs();
                    if original_type == kind && result.memory.byte(old_node.wrapping_add(9))? != 0 {
                        copy_record(result, old_node, 0x41fc48, 22)?;
                        if remove(
                            result,
                            frame.wrapping_sub(0x12c),
                            member,
                            search as u16,
                            false,
                            limits,
                            services,
                        )? as u16
                            == 0
                        {
                            result.native_return = EquipmentReturn::long(0);
                            return Ok(());
                        }
                        copy_record(result, node, 0x41fc48, 22)?;
                        if result.memory.byte(0x442750)? != 0 {
                            let last = search.wrapping_sub(sw(&result.memory, 0x44719c)?);
                            let first = search.wrapping_sub(sw(&result.memory, 0x44719c)?);
                            update_items(result, frame.wrapping_sub(0x128), first, last, services)?;
                            copy_record(result, node, 0x41fc48, 22)?;
                        }
                        replaced = true;
                        break;
                    }
                    search = search.wrapping_add(1);
                }
                if replaced {
                    continue;
                }
                let id = sw(&result.memory, node)?;
                load_item(result, id, false)?;
                let kind = sw(&result.memory, 0x453656)?.wrapping_abs();
                word(result, 0x453656, kind)?;
                upper = Some(((i32::from(kind) * 2) as u32 >> 16) as u16);
                word(result, indexed(warrior.wrapping_add(0xc6), kind), 0)?;
                if result.memory.byte(0x453644)? != 0 {
                    upper = member_upper;
                    word(result, warrior.wrapping_add(0x5c), 0)?;
                }
            }
        }
        let kind = sw(&result.memory, 0x453656)?;
        match kind {
            0 => {
                upper = member_upper;
                if result.memory.word(warrior.wrapping_add(0xc6))? != 0
                    && result.memory.word(warrior.wrapping_add(0xc8))? != 0
                {
                    upper = wear_warning(result, frame, emit, 9, upper, limits, services)?;
                    wear_return(result, upper)?;
                    return Ok(());
                }
                word(result, 0x4471ea, 1)?;
                let destination = if result.memory.word(warrior.wrapping_add(0xc6))? == 0 {
                    0xc6
                } else {
                    0xc8
                };
                let id = sw(&result.memory, 0x4536a2)?;
                word(result, warrior.wrapping_add(destination), id)?;
                if result.memory.word(warrior.wrapping_add(0xc6))?
                    == result.memory.word(warrior.wrapping_add(0xc8))?
                {
                    word(result, warrior.wrapping_add(0xc8), 0)?;
                    word(result, 0x4471ea, 0)?;
                    upper = wear_warning(result, frame, emit, 31, member_upper, limits, services)?;
                    wear_return(result, upper)?;
                    return Ok(());
                }
            }
            2 => {
                upper = member_upper;
                if sw(&result.memory, warrior.wrapping_add(0x5c))? < 2
                    && result.memory.word(warrior.wrapping_add(0xca))? == 0
                {
                    if sb(&result.memory, 0x453644)?
                        .wrapping_add(sw(&result.memory, warrior.wrapping_add(0x5c))?)
                        > 2
                    {
                        upper = wear_warning(result, frame, emit, 8, upper, limits, services)?;
                        wear_return(result, upper)?;
                        return Ok(());
                    }
                    let id = sw(&result.memory, 0x4536a2)?;
                    word(result, warrior.wrapping_add(0xca), id)?;
                    let sound = sw(&result.memory, 0x45364a)?;
                    word(result, warrior.wrapping_add(0x128), sound)?;
                    let hands = sb(&result.memory, 0x453644)?;
                    add_word(result, warrior.wrapping_add(0x5c), hands)?;
                    let id = sw(&result.memory, 0x4536a2)?;
                    word(result, warrior.wrapping_add(0x12e), id)?;
                    word(result, 0x4471ea, 1)?;
                    write(result, warrior.wrapping_add(0x9dd), &[0])?;
                }
            }
            3 => {
                upper = member_upper;
                if sb(&result.memory, 0x453644)?
                    .wrapping_add(sw(&result.memory, warrior.wrapping_add(0x5c))?)
                    > 2
                {
                    upper = wear_warning(result, frame, emit, 8, upper, limits, services)?;
                    wear_return(result, upper)?;
                    return Ok(());
                }
                let id = sw(&result.memory, 0x4536a2)?;
                word(result, warrior.wrapping_add(0xcc), id)?;
                add_word(result, warrior.wrapping_add(0x5c), 1)?;
                word(result, 0x4471ea, 1)?;
            }
            13 | 14 => {
                upper = member_upper;
                let other = if kind == 13 { 0xe2 } else { 0xe0 };
                if result.memory.word(warrior.wrapping_add(0xea))? == 0
                    && result.memory.word(warrior.wrapping_add(other))? != 0
                {
                    upper = wear_warning(result, frame, emit, 146, upper, limits, services)?;
                    wear_return(result, upper)?;
                    return Ok(());
                }
                word(result, 0x4471ea, 1)?;
            }
            15 => {
                upper = member_upper;
                if result.memory.word(warrior.wrapping_add(0xe4))? == 0 {
                    let id = sw(&result.memory, 0x4536a2)?;
                    word(result, warrior.wrapping_add(0x12c), id)?;
                    let id = sw(&result.memory, 0x4536a2)?;
                    word(result, warrior.wrapping_add(0xe4), id)?;
                    let ammo = sw(&result.memory, node.wrapping_add(4))?;
                    word(result, warrior.wrapping_add(0x126), ammo)?;
                    word(result, 0x4471ea, 1)?;
                    write(result, warrior.wrapping_add(0x9dd), &[1])?;
                    upper = Some(0);
                }
            }
            _ => {
                if kind < 23 {
                    upper = Some(((i32::from(kind) * 2) as u32 >> 16) as u16);
                    let id = sw(&result.memory, 0x4536a2)?;
                    word(result, indexed(warrior.wrapping_add(0xc6), kind), id)?;
                }
                word(result, 0x4471ea, 1)?;
            }
        }
        if result.memory.word(0x4471ea)? != 0 {
            if sw(&result.memory, 0x4536ce)? == 1 && sw(&result.memory, node.wrapping_add(2))? < 1 {
                wear_warning(result, frame, emit, 7, upper, limits, services)?;
                result.native_return = EquipmentReturn::long(0);
                return Ok(());
            }
            if sw(&result.memory, 0x4536ce)? == 2
                && sw(&result.memory, 0x453698)?
                    .wrapping_add(sw(&result.memory, warrior.wrapping_add(0x2a))?)
                    > sw(&result.memory, warrior.wrapping_add(0x26))?
            {
                wear_warning(result, frame, emit, 144, member_upper, limits, services)?;
                result.native_return = EquipmentReturn::long(0);
                return Ok(());
            }
            let value = sb(&result.memory, 0x41fc56)?.wrapping_add(sw(&result.memory, 0x453694)?);
            add_word(result, warrior.wrapping_add(0x24), value)?;
            clamp_negative(result, warrior.wrapping_add(0x24))?;
            let value = sb(&result.memory, 0x41fc5d)?.wrapping_add(sw(&result.memory, 0x453698)?);
            add_word(result, warrior.wrapping_add(0x2a), value)?;
            clamp_negative(result, warrior.wrapping_add(0x2a))?;
            if result.memory.word(0x453696)? != 0 {
                let value = sw(&result.memory, 0x4536d0)?
                    .wrapping_add(sw(&result.memory, node.wrapping_add(6))?);
                add_word(result, warrior.wrapping_add(0x28), value)?;
                clamp_negative(result, warrior.wrapping_add(0x28))?;
                let value = sw(&result.memory, node.wrapping_add(2))?;
                add_word(result, warrior.wrapping_add(0x26), value)?;
                clamp_negative(result, warrior.wrapping_add(0x26))?;
            }
            let kind = sw(&result.memory, 0x453656)?;
            let id = sw(&result.memory, 0x4536a2)?;
            word(result, indexed(warrior.wrapping_add(0xc6), kind), id)?;
            write(result, node.wrapping_add(9), &[1])?;
            copy_record(result, warrior, 0x447e5e, WARRIOR_BYTES)?;
            let (source, offset) = if sw(&result.memory, 0x453656)? == 2 {
                (0x41fc53, 6)
            } else {
                (0x41fc57, 4)
            };
            let value = sb(&result.memory, source)?;
            add_word(result, warrior.wrapping_add(offset), value)?;
            for (source, offset) in [(0x45363c, 0x9e2), (0x45363e, 0x9e8)] {
                let value = sb(&result.memory, source)?;
                add_word(result, warrior.wrapping_add(offset), value)?;
            }
            for (first, second, offset) in [(0x41fc58, 0x45363f, 8), (0x41fc59, 0x453640, 10)] {
                let value = sb(&result.memory, first)?.wrapping_add(sb(&result.memory, second)?);
                add_word(result, warrior.wrapping_add(offset), value)?;
            }
            for (source, offset) in [(0x453641, 0x30), (0x453642, 0x22), (0x453643, 0x5a)] {
                let value = sb(&result.memory, source)?;
                add_word(result, warrior.wrapping_add(offset), value)?;
            }
            for (first, second, offset) in [(0x41fc5c, 0x453678, 0x130), (0x41fc5b, 0x45369a, 0x2c)]
            {
                let value = sb(&result.memory, first)?.wrapping_add(sw(&result.memory, second)?);
                add_word(result, warrior.wrapping_add(offset), value)?;
            }
            for (source, offset) in [
                (0x4536c8, 0x114),
                (0x45367c, 0x116),
                (0x45367e, 0x118),
                (0x453680, 0x11a),
                (0x453682, 0x11c),
                (0x453684, 0x11e),
                (0x453686, 0x120),
            ] {
                let value = sw(&result.memory, source)?;
                if value < 0 {
                    subtract_word(result, warrior.wrapping_add(offset), value)?;
                }
            }
            movement(result, member)?;
            upper = Some(0);
            if sw(&result.memory, 0x453666)? > 59 && sw(&result.memory, 0x453666)? < 100 {
                let index = sw(&result.memory, 0x453668)?;
                word(result, indexed(warrior.wrapping_add(0x5e), index), 0)?;
                upper = member_upper;
            }
            if sw(&result.memory, 0x453666)? == 122 {
                let value = sw(&result.memory, 0x453668)?;
                add_word(result, warrior.wrapping_add(0x1e), value)?;
                upper = member_upper;
            }
            if sw(&result.memory, 0x453666)? > 19 && sw(&result.memory, 0x453666)? < 60 {
                let index = sw(&result.memory, 0x453666)?.wrapping_sub(20);
                let address = indexed(warrior.wrapping_add(0x5e), index);
                if sw(&result.memory, address)? > -1 {
                    word(result, address, 0)?;
                }
                let index = sw(&result.memory, 0x453666)?.wrapping_sub(20);
                let value = sw(&result.memory, 0x453668)?;
                add_word(result, indexed(warrior.wrapping_add(0x5e), index), value)?;
                upper = Some(0xffff);
            }
            wear_attribute(result, warrior, 0x45366a, &mut upper)?;
            wear_attribute(result, warrior, 0x45366c, &mut upper)?;
        }
        if result.memory.word(0x4471ea)? != 0 {
            word(result, indexed(0x442798, member as i16), 1)?;
            let id = if result.memory.word(0x453648)? != 0 && emit != 0 {
                Some(sw(&result.memory, 0x453648)?)
            } else if result.memory.word(0x45364a)? != 0 && emit != 0 {
                Some(sw(&result.memory, 0x45364a)?)
            } else {
                None
            };
            if let Some(id) = id {
                sound(result, id, services)?;
                upper = Some(0);
            }
        }
        wear_return(result, upper)?;
        return Ok(());
    }
}

fn call_wear(
    result: &mut EquipmentPlan,
    frame: u32,
    member: u16,
    slot: u16,
    limits: EquipmentLimits,
    remaining: &mut usize,
    services: &mut (impl EquipmentServices + ?Sized),
) -> Result<(), RuleError> {
    word(result, frame.wrapping_add(12), 1)?;
    word(result, frame.wrapping_add(10), slot as i16)?;
    word(result, frame.wrapping_add(8), member as i16)?;
    wear_body(result, frame, limits, remaining, services)
}
fn refill_sound(
    result: &mut EquipmentPlan,
    silent: i16,
    services: &(impl EquipmentServices + ?Sized),
) -> Result<(), RuleError> {
    if silent == 0 {
        let mut context = RuntimeContext::new(result.random.clone(), 1);
        let child = recovered_runtime::rand(&mut context, 2)?;
        let value = match child.result {
            RuntimeResult::Word { value, .. } => value,
            _ => return Err(RuleError::Invalid("unexpected Rand result".into())),
        };
        result.random = context.random;
        result
            .effects
            .extend(child.effects.into_iter().map(EquipmentEffect::Runtime));
        sound(result, value.wrapping_add(403), services)?;
    }
    Ok(())
}
fn ammunition_address(index: i16) -> u32 {
    0x42dbe4_u32.wrapping_add((i32::from(index) * 4) as u32)
}
fn subtract_ammunition(
    result: &mut EquipmentPlan,
    index: i16,
    amount: i16,
) -> Result<(), RuleError> {
    let address = ammunition_address(index);
    let value = result
        .memory
        .long(address)?
        .wrapping_sub(i32::from(amount) as u32);
    write(result, address, &value.to_be_bytes())
}
fn restricted_refill(result: &mut EquipmentPlan, frame: u32, node: u32) -> Result<bool, RuleError> {
    if result.memory.word(frame.wrapping_add(14))? == 0 && result.memory.word(0x438776)? == 0 {
        return Ok(true);
    }
    let item = sw(&result.memory, node)?;
    load_item(result, item, true)?;
    Ok(sw(&result.memory, 0x453656)? == 2 || sw(&result.memory, 0x453656)? == 15)
}
#[allow(clippy::too_many_arguments)]
fn refill_energy(
    result: &mut EquipmentPlan,
    frame: u32,
    member: u16,
    slot: u16,
    amount: i16,
    remove_first: bool,
    limits: EquipmentLimits,
    remaining: &mut usize,
    services: &mut (impl EquipmentServices + ?Sized),
) -> Result<(), RuleError> {
    let node = node_address(member, slot);
    if result.memory.byte(node.wrapping_add(10))? == 20 {
        if sw(&result.memory, node.wrapping_add(2))? > 0 {
            if remove_first {
                remove(
                    result,
                    frame.wrapping_sub(40),
                    member,
                    slot,
                    true,
                    limits,
                    services,
                )?;
            }
            let kind = sb(&result.memory, node.wrapping_add(10))?;
            word(
                result,
                indexed(member_address(member).wrapping_add(0xc6), kind),
                0,
            )?;
            write(result, node.wrapping_add(9), &[0])?;
            call_wear(
                result,
                frame.wrapping_sub(if remove_first { 46 } else { 38 }),
                member,
                slot,
                limits,
                remaining,
                services,
            )?;
        } else {
            add_word(result, member_address(member).wrapping_add(0x26), amount)?;
        }
    }
    Ok(())
}
fn refresh_refill(
    result: &mut EquipmentPlan,
    frame: u32,
    slot: i16,
    services: &mut (impl EquipmentServices + ?Sized),
) -> Result<(), RuleError> {
    if result.memory.byte(0x442750)? != 0 {
        update_items(result, frame.wrapping_sub(36), slot, slot, services)?;
    }
    Ok(())
}

/// refit, 0x14e5e4. Both inventory passes and their distinct resource writes are retained.
pub fn refit(
    memory: &RenderingMemory,
    random: &RandomState,
    frame: u32,
    limits: EquipmentLimits,
    services: &mut (impl EquipmentServices + ?Sized),
) -> Result<EquipmentPlan, RuleError> {
    let mut result = plan(memory, random);
    let member = result.memory.word(frame.wrapping_add(8))?;
    let silent = sw(&result.memory, frame.wrapping_add(10))?;
    if (member as i16) < 0 {
        return Ok(result);
    }
    let warrior = member_address(member);
    let mut remaining = limits.maximum_steps;
    word(&mut result, frame.wrapping_sub(4), 0)?;
    let mut slot = 0_i16;
    while slot < sw(&result.memory, warrior.wrapping_add(0x1a))? {
        step(&mut remaining)?;
        let node = node_address(member, slot as u16);
        if result.memory.byte(node.wrapping_add(9))? != 0
            && sw(&result.memory, node.wrapping_add(2))? < sw(&result.memory, node.wrapping_add(6))?
            && result.memory.word(node.wrapping_add(4))? != 0
            && restricted_refill(&mut result, frame, node)?
        {
            let ammo = sw(&result.memory, node.wrapping_add(4))?;
            load_item(&mut result, ammo, true)?;
            word(&mut result, frame.wrapping_sub(4), 0)?;
            let index = sw(&result.memory, node.wrapping_add(4))?.wrapping_sub(200);
            if result.memory.long(ammunition_address(index))? != 0 {
                let index = sw(&result.memory, node.wrapping_add(4))?.wrapping_sub(200);
                word(&mut result, frame.wrapping_sub(4), index)?;
                if result.memory.word(node.wrapping_add(4))? != 0 {
                    let needed = sw(&result.memory, node.wrapping_add(6))?
                        .wrapping_sub(sw(&result.memory, node.wrapping_add(2))?);
                    let index = sw(&result.memory, frame.wrapping_sub(4))?;
                    let available = result.memory.long(ammunition_address(index))? as i16;
                    if available < needed {
                        refill_sound(&mut result, silent, services)?;
                        add_word(&mut result, node.wrapping_add(2), available)?;
                        refill_energy(
                            &mut result,
                            frame,
                            member,
                            slot as u16,
                            available,
                            false,
                            limits,
                            &mut remaining,
                            services,
                        )?;
                        refresh_refill(&mut result, frame, slot, services)?;
                        if result.memory.word(frame.wrapping_add(12))? == 0 {
                            warn(&mut result, frame.wrapping_sub(34), 49, limits, services)?;
                        }
                    } else {
                        add_word(&mut result, node.wrapping_add(2), needed)?;
                        refill_energy(
                            &mut result,
                            frame,
                            member,
                            slot as u16,
                            needed,
                            false,
                            limits,
                            &mut remaining,
                            services,
                        )?;
                        let index = sw(&result.memory, frame.wrapping_sub(4))?;
                        subtract_ammunition(&mut result, index, needed)?;
                        refill_sound(&mut result, silent, services)?;
                        refresh_refill(&mut result, frame, slot, services)?;
                        if result.memory.word(frame.wrapping_add(14))? != 0
                            || result.memory.word(0x438776)? != 0
                        {
                            result.native_return = EquipmentReturn::long(1);
                            return Ok(result);
                        }
                    }
                }
            }
        }
        slot = slot.wrapping_add(1);
    }
    slot = 0;
    while slot < sw(&result.memory, warrior.wrapping_add(0x1a))? {
        step(&mut remaining)?;
        let node = node_address(member, slot as u16);
        if result.memory.byte(node.wrapping_add(9))? != 0
            && sw(&result.memory, node.wrapping_add(2))? <= 0
            && sw(&result.memory, node.wrapping_add(6))? > 0
            && restricted_refill(&mut result, frame, node)?
        {
            let item = sw(&result.memory, node)?;
            load_item(&mut result, item, false)?;
            word(&mut result, frame.wrapping_sub(4), 0)?;
            let kind = sw(&result.memory, 0x45365c)?;
            word(&mut result, frame.wrapping_sub(2), kind)?;
            word(&mut result, 0x4434ac, 0)?;
            while sw(&result.memory, 0x4434ac)? < 200 {
                step(&mut remaining)?;
                let index = sw(&result.memory, 0x4434ac)?;
                if result.memory.long(ammunition_address(index))? != 0 {
                    let ammo = sw(&result.memory, 0x4434ac)?.wrapping_add(200);
                    load_item(&mut result, ammo, true)?;
                    if sw(&result.memory, 0x4536f4)? == sw(&result.memory, frame.wrapping_sub(2))? {
                        let needed = sw(&result.memory, node.wrapping_add(6))?
                            .wrapping_sub(sw(&result.memory, node.wrapping_add(2))?);
                        let index = sw(&result.memory, 0x4434ac)?;
                        let available = result.memory.long(ammunition_address(index))? as i16;
                        if available <= needed {
                            refill_sound(&mut result, silent, services)?;
                            add_word(&mut result, node.wrapping_add(2), available)?;
                            // Native 0x14eb42 uses the inventory slot, not the matched ammo index.
                            word(&mut result, node.wrapping_add(4), slot.wrapping_add(200))?;
                            refill_energy(
                                &mut result,
                                frame,
                                member,
                                slot as u16,
                                available,
                                true,
                                limits,
                                &mut remaining,
                                services,
                            )?;
                            refresh_refill(&mut result, frame, slot, services)?;
                            if result.memory.word(frame.wrapping_add(12))? == 0 {
                                warn(&mut result, frame.wrapping_sub(34), 49, limits, services)?;
                            }
                            result.native_return = EquipmentReturn::long(0);
                            return Ok(result);
                        }
                        add_word(&mut result, node.wrapping_add(2), needed)?;
                        let ammo = sw(&result.memory, 0x4434ac)?.wrapping_add(200);
                        word(&mut result, node.wrapping_add(4), ammo)?;
                        refill_energy(
                            &mut result,
                            frame,
                            member,
                            slot as u16,
                            needed,
                            true,
                            limits,
                            &mut remaining,
                            services,
                        )?;
                        let index = sw(&result.memory, 0x4434ac)?;
                        subtract_ammunition(&mut result, index, needed)?;
                        refill_sound(&mut result, silent, services)?;
                        refresh_refill(&mut result, frame, slot, services)?;
                        result.native_return = EquipmentReturn::long(u32::from(
                            result.memory.word(frame.wrapping_add(14))? != 0
                                || result.memory.word(0x438776)? != 0,
                        ));
                        return Ok(result);
                    }
                }
                let value = sw(&result.memory, 0x4434ac)?.wrapping_add(1);
                word(&mut result, 0x4434ac, value)?;
            }
        }
        slot = slot.wrapping_add(1);
    }
    result.native_return = EquipmentReturn::long(0);
    Ok(result)
}
