//! Inventory mutations and eligibility from complete CODE 1 control bodies.

use crate::RuleError;
use crate::recovered_feedback::{self, FeedbackEffect, FeedbackServices};
use crate::recovered_files::{self, FileBuffer, FileContext, FileEffect, FileObservation};
use crate::recovered_install::InstallMemory;
use crate::recovered_navigation;
use crate::recovered_records::{self, ITEM_TABLE_ADDRESSES, ItemTables, RecordWrite};
use crate::recovered_rendering::RenderingMemory;
use centurions_content::{ITEM_BYTES, WARRIOR_BYTES};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SaneRequest {
    pub operation: u16,
    pub source: u32,
    pub destination: Option<u32>,
}

pub trait InventoryServices: FeedbackServices {
    /// Actual ordered external writes; this observer performs no host operation.
    fn sane(
        &self,
        request: &SaneRequest,
        memory: &RenderingMemory,
    ) -> Result<Vec<RecordWrite>, RuleError>;
}

#[derive(Clone, Debug, PartialEq)]
pub enum InventoryEffect {
    Write(RecordWrite),
    Sane {
        request: SaneRequest,
        writes: Vec<RecordWrite>,
    },
    BitTest {
        pointer: u32,
        index: i32,
        value: bool,
    },
    File(FileEffect),
    Feedback(FeedbackEffect),
}

#[derive(Clone, Debug, PartialEq)]
pub struct InventoryPlan {
    pub memory: RenderingMemory,
    pub effects: Vec<InventoryEffect>,
    pub native_return: u32,
}

fn plan(memory: &RenderingMemory) -> InventoryPlan {
    InventoryPlan {
        memory: memory.clone(),
        effects: vec![],
        native_return: 0,
    }
}

fn read(memory: &RenderingMemory, address: u32, length: usize) -> Result<Vec<u8>, RuleError> {
    (0..length)
        .map(|i| memory.byte(address.wrapping_add(i as u32)))
        .collect()
}

fn sw(memory: &RenderingMemory, address: u32) -> Result<i16, RuleError> {
    Ok(memory.word(address)? as i16)
}

fn sb(memory: &RenderingMemory, address: u32) -> Result<i16, RuleError> {
    Ok(i16::from(memory.byte(address)? as i8))
}

fn write(result: &mut InventoryPlan, address: u32, bytes: &[u8]) -> Result<(), RuleError> {
    read(&result.memory, address, bytes.len())?;
    result.memory.insert_region(address, bytes);
    result.effects.push(InventoryEffect::Write(RecordWrite {
        address,
        bytes: bytes.to_vec(),
    }));
    Ok(())
}

fn word(result: &mut InventoryPlan, address: u32, value: i16) -> Result<(), RuleError> {
    write(result, address, &value.to_be_bytes())
}

fn add_word(result: &mut InventoryPlan, address: u32, value: i16) -> Result<(), RuleError> {
    let next = sw(&result.memory, address)?.wrapping_add(value);
    word(result, address, next)
}

fn subtract_word(result: &mut InventoryPlan, address: u32, value: i16) -> Result<(), RuleError> {
    let next = sw(&result.memory, address)?.wrapping_sub(value);
    word(result, address, next)
}

fn add_byte(result: &mut InventoryPlan, address: u32, value: u8) -> Result<(), RuleError> {
    let next = result.memory.byte(address)?.wrapping_add(value);
    write(result, address, &[next])
}

fn clamp_negative(result: &mut InventoryPlan, address: u32) -> Result<(), RuleError> {
    if sw(&result.memory, address)? < 0 {
        word(result, address, 0)?;
    }
    Ok(())
}

fn member_address(member: u16) -> u32 {
    0x448aa0_u32.wrapping_add(u32::from(member) * WARRIOR_BYTES as u32)
}

fn indexed(address: u32, index: i16) -> u32 {
    address.wrapping_add((i32::from(index) * 2) as u32)
}

fn load_item(result: &mut InventoryPlan, id: i16, comparison: bool) -> Result<(), RuleError> {
    let destination = if comparison { 0x4781ee } else { 0x45363c };
    let current: [u8; ITEM_BYTES] = read(&result.memory, destination, ITEM_BYTES)?
        .try_into()
        .expect("item record");
    let absolute = id.wrapping_abs();
    let family = absolute / 200;
    let mut backing: [Vec<u8>; 5] = std::array::from_fn(|_| vec![]);
    if let Ok(family) = usize::try_from(family)
        && family < backing.len()
    {
        let offset = (absolute % 200) as usize * ITEM_BYTES;
        let bytes = read(
            &result.memory,
            ITEM_TABLE_ADDRESSES[family].wrapping_add(offset as u32),
            ITEM_BYTES,
        )?;
        // Only the selected record is read by the existing loader.
        backing[family].resize(offset + ITEM_BYTES, 0);
        backing[family][offset..].copy_from_slice(&bytes);
    }
    let tables = ItemTables {
        families: [
            &backing[0],
            &backing[1],
            &backing[2],
            &backing[3],
            &backing[4],
        ],
    };
    let child = if comparison {
        recovered_records::loaditemcompare(&tables, &current, id)?
    } else {
        recovered_records::loaditem(&tables, &current, id)?
    };
    for effect in child.writes {
        write(result, effect.address, &effect.bytes)?;
    }
    Ok(())
}

fn warn(
    result: &mut InventoryPlan,
    frame: u32,
    message: i16,
    maximum_polls: usize,
    services: &(impl InventoryServices + ?Sized),
) -> Result<(), RuleError> {
    let child = recovered_feedback::warn(&result.memory, frame, message, maximum_polls, services)?;
    result.memory = child.memory;
    result
        .effects
        .extend(child.effects.into_iter().map(InventoryEffect::Feedback));
    Ok(())
}

fn sane(
    result: &mut InventoryPlan,
    frame: u32,
    operation: u16,
    source_offset: u32,
    destination_offset: Option<u32>,
    services: &(impl InventoryServices + ?Sized),
) -> Result<(), RuleError> {
    let request = SaneRequest {
        operation,
        source: frame.wrapping_sub(source_offset),
        destination: destination_offset.map(|offset| frame.wrapping_sub(offset)),
    };
    let writes = services.sane(&request, &result.memory)?;
    result.effects.push(InventoryEffect::Sane {
        request,
        writes: writes.clone(),
    });
    for effect in writes {
        write(result, effect.address, &effect.bytes)?;
    }
    Ok(())
}

/// upgrade, 0x12c46e. Caller WORDs at A6+8/+10/+12 are member/slot/item.
pub fn upgrade(
    memory: &RenderingMemory,
    frame: u32,
    maximum_polls: usize,
    services: &(impl InventoryServices + ?Sized),
) -> Result<InventoryPlan, RuleError> {
    let mut result = plan(memory);
    let member = result.memory.word(frame.wrapping_add(8))?;
    let slot = result.memory.word(frame.wrapping_add(10))?;
    let warrior = member_address(member);
    let node = warrior.wrapping_add(u32::from(slot) * 22);
    write(&mut result, frame.wrapping_sub(8), &[0; 4])?;
    write(&mut result, frame.wrapping_sub(4), &[0; 4])?;
    let item = sw(&result.memory, frame.wrapping_add(12))?;
    load_item(&mut result, item, false)?;
    let cost = sw(&result.memory, 0x4536d0)?;
    write(
        &mut result,
        frame.wrapping_sub(0x16),
        &i32::from(cost).to_be_bytes(),
    )?;
    sane(&mut result, frame, 0x280e, 0x16, Some(0x16), services)?;
    sane(&mut result, frame, 0x1010, 0x16, Some(0x16), services)?;
    let single = read(&result.memory, frame.wrapping_sub(0x16), 4)?;
    write(&mut result, frame.wrapping_sub(4), &single)?;
    // At this call SP includes the retained item WORD, four saved LONGs and frame.
    sane(&mut result, frame, 0x101c, 4, Some(0x2a), services)?;
    if result.memory.word(frame.wrapping_sub(0x2a))? != 4 {
        write(
            &mut result,
            frame.wrapping_sub(0xc),
            &0x42c80000_u32.to_be_bytes(),
        )?;
        sane(&mut result, frame, 0x100e, 4, Some(0x16), services)?;
        sane(&mut result, frame, 0x1000, 0xc, Some(0x16), services)?;
        sane(&mut result, frame, 0x1010, 0x16, Some(4), services)?;
        write(
            &mut result,
            frame.wrapping_sub(0xc),
            &0x42c80000_u32.to_be_bytes(),
        )?;
        sane(&mut result, frame, 0x100e, 4, Some(0x16), services)?;
        sane(&mut result, frame, 0x1006, 0xc, Some(0x16), services)?;
        sane(&mut result, frame, 0x1010, 0x16, Some(4), services)?;
        let old = sw(&result.memory, node.wrapping_add(0x138))?;
        word(&mut result, frame.wrapping_sub(0x16), old)?;
        sane(&mut result, frame, 0x200e, 0x16, Some(0x16), services)?;
        sane(&mut result, frame, 0x1010, 0x16, Some(0x16), services)?;
        let single = read(&result.memory, frame.wrapping_sub(0x16), 4)?;
        write(&mut result, frame.wrapping_sub(8), &single)?;
        sane(&mut result, frame, 0x100e, 8, Some(0x16), services)?;
        sane(&mut result, frame, 0x1004, 4, Some(0x16), services)?;
        sane(&mut result, frame, 0x1010, 0x16, Some(8), services)?;
        sane(&mut result, frame, 0x100e, 8, Some(0x16), services)?;
        sane(&mut result, frame, 0x16, 0x16, None, services)?;
        sane(&mut result, frame, 0x2810, 0x16, Some(0x16), services)?;
        let value = result.memory.long(frame.wrapping_sub(0x16))? as i16;
        let previous = sw(&result.memory, node.wrapping_add(0x138))?;
        word(
            &mut result,
            node.wrapping_add(0x138),
            if value == previous {
                previous.wrapping_add(1)
            } else {
                value
            },
        )?;
        if result.memory.byte(node.wrapping_add(0x13c))? == 20 {
            word(&mut result, warrior.wrapping_add(0x28), value)?;
        }
    }
    for (source, member_offset, node_offset) in [
        (0x45368a, 0x24, 0x140),
        (0x45368d, 4, 0x141),
        (0x45368f, 8, 0x142),
    ] {
        let value = sb(&result.memory, source)?;
        add_word(&mut result, warrior.wrapping_add(member_offset), value)?;
        let value = result.memory.byte(source)?;
        add_byte(&mut result, node.wrapping_add(node_offset), value)?;
    }
    if matches!(result.memory.byte(0x453693)?, 13 | 14) {
        let value = result.memory.byte(0x45368c)?;
        add_byte(&mut result, node.wrapping_add(0x147), value)?;
        let selector = i16::from(result.memory.byte(0x453693)? as i8);
        let item = sw(
            &result.memory,
            indexed(warrior.wrapping_add(0xc6), selector),
        )?;
        load_item(&mut result, item, true)?;
        let effective = sb(&result.memory, node.wrapping_add(0x147))?
            .wrapping_add(sw(&result.memory, 0x47824a)?);
        if effective < 1 {
            add_byte(&mut result, node.wrapping_add(0x147), 1)?;
            warn(
                &mut result,
                frame.wrapping_sub(48),
                148,
                maximum_polls,
                services,
            )?;
            result.native_return = u32::MAX;
            return Ok(result);
        }
        let value = sb(&result.memory, 0x45368c)?;
        add_word(&mut result, warrior.wrapping_add(0x2a), value)?;
    }
    let value = result.memory.byte(0x45368b)?;
    add_byte(&mut result, node.wrapping_add(0x146), value)?;
    let value = sb(&result.memory, 0x453690)?;
    add_word(&mut result, warrior.wrapping_add(0xa), value)?;
    let value = result.memory.byte(0x453690)?;
    add_byte(&mut result, node.wrapping_add(0x143), value)?;
    let value = result.memory.byte(0x453691)?;
    add_byte(&mut result, node.wrapping_add(0x144), value)?;
    let value = sb(&result.memory, 0x453692)?;
    add_word(&mut result, warrior.wrapping_add(0x2c), value)?;
    let value = result.memory.byte(0x453692)?;
    add_byte(&mut result, node.wrapping_add(0x145), value)?;
    Ok(result)
}

fn bit(result: &mut InventoryPlan, pointer: u32, index: i32) -> Result<bool, RuleError> {
    let byte = result
        .memory
        .byte(pointer.wrapping_add((index >> 3) as u32))?;
    let value = byte & (0x80 >> (index as u32 & 7)) != 0;
    result.effects.push(InventoryEffect::BitTest {
        pointer,
        index,
        value,
    });
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
    Err(RuleError::Invalid(
        "unterminated inventory profile path".into(),
    ))
}

fn profile(
    result: &mut InventoryPlan,
    race: i16,
    caste: i16,
    filename_capacity: usize,
    observations: &[FileObservation],
) -> Result<(), RuleError> {
    if race == 0 && caste == 0 {
        return Ok(());
    }
    let mut context = FileContext {
        scenario_directory: c_string(&result.memory, 0x44260c)?,
        variant: result.memory.byte(0x44274c)?,
        filename: read(&result.memory, 0x4389a4, filename_capacity)?,
        file_handle: result.memory.long(0x4434a6)?,
        race_profile: if race != 0 {
            read(&result.memory, 0x41f85a, 408)?
        } else {
            vec![]
        },
        caste_profile: if caste != 0 {
            read(&result.memory, 0x41f9f2, 576)?
        } else {
            vec![]
        },
        ..FileContext::default()
    };
    let child = recovered_files::load_profile(&mut context, race, caste, observations)?;
    for effect in child.effects {
        match &effect {
            FileEffect::Write {
                destination,
                offset,
                bytes,
            } => {
                let base: u32 = match destination {
                    FileBuffer::Filename => 0x4389a4,
                    FileBuffer::RaceProfile => 0x41f85a,
                    FileBuffer::CasteProfile => 0x41f9f2,
                    _ => {
                        return Err(RuleError::Invalid(
                            "unexpected loadprofile output buffer".into(),
                        ));
                    }
                };
                write(result, base.wrapping_add(*offset as u32), bytes)?;
            }
            FileEffect::FileHandle(handle) => write(result, 0x4434a6, &handle.to_be_bytes())?,
            _ => result.effects.push(InventoryEffect::File(effect)),
        }
    }
    Ok(())
}

/// canuse, 0x157586. The two profile WORD arguments are both preserved.
pub fn canuse(
    memory: &RenderingMemory,
    frame: u32,
    filename_capacity: usize,
    file_observations: &[FileObservation],
) -> Result<InventoryPlan, RuleError> {
    let mut result = plan(memory);
    let item = sw(&result.memory, frame.wrapping_add(8))?;
    let race = sw(&result.memory, frame.wrapping_add(10))?;
    let caste = sw(&result.memory, frame.wrapping_add(12))?;
    profile(
        &mut result,
        race,
        caste,
        filename_capacity,
        file_observations,
    )?;
    if item > 0 {
        load_item(&mut result, item, false)?;
    }
    word(&mut result, 0x435506, 0)?;
    word(&mut result, 0x435504, 0)?;
    if result.memory.word(0x4536a8)? != 0 && race != sw(&result.memory, 0x4536a8)? {
        word(&mut result, 0x435504, 1)?;
    }
    if result.memory.word(0x4536aa)? != 0 && caste != sw(&result.memory, 0x4536aa)? {
        word(&mut result, 0x435506, 1)?;
    }
    if result.memory.word(0x435504)? != 0 {
        return Ok(result);
    }
    let mut index = 0_i32;
    while index < 58 {
        if bit(&mut result, 0x4536bc, index)? {
            break;
        }
        index += 1;
    }
    let enabled = bit(&mut result, 0x41fbac, index)?;
    if enabled && bit(&mut result, 0x41f9aa, index)? {
        for i in 0..9 {
            if bit(&mut result, 0x4536a4, i)? && bit(&mut result, 0x41f9b2, i)? {
                word(&mut result, 0x435504, 1)?;
            }
        }
        for i in 0..9 {
            if bit(&mut result, 0x4536ac, i)? && !bit(&mut result, 0x41f9b2, i)? {
                word(&mut result, 0x435504, 1)?;
            }
        }
        for i in 0..7 {
            if bit(&mut result, 0x4536a6, i)?
                && i as i16 == sw(&result.memory, 0x41faf0)?.wrapping_sub(1)
            {
                word(&mut result, 0x435506, 1)?;
            }
        }
        if result.memory.word(0x4536ae)? != 0 {
            let rank = sw(&result.memory, 0x41faf0)?.wrapping_sub(1);
            if !bit(&mut result, 0x4536ae, i32::from(rank))? {
                word(&mut result, 0x435506, 1)?;
            }
        }
        if result.memory.word(0x435504)? == 0 && result.memory.word(0x435506)? == 0 {
            result.native_return = 1;
        }
    } else if !bit(&mut result, 0x41fbac, index)? {
        word(&mut result, 0x435506, 1)?;
    } else {
        word(&mut result, 0x435504, 1)?;
    }
    Ok(result)
}

/// cancast, 0x13beea. Incoming D0 supplies the untouched upper WORD for members >= 6.
pub fn cancast(
    memory: &RenderingMemory,
    frame: u32,
    incoming_d0: u32,
    maximum_polls: usize,
    services: &(impl InventoryServices + ?Sized),
) -> Result<InventoryPlan, RuleError> {
    let mut result = plan(memory);
    let member = sw(&result.memory, frame.wrapping_add(8))?;
    if member < 0 {
        return Ok(result);
    }
    if member > sb(&result.memory, 0x44273a)? {
        return Ok(result);
    }
    if result.memory.byte(0x442749)? != 0 && result.memory.word(frame.wrapping_add(10))? == 0 {
        warn(
            &mut result,
            frame.wrapping_sub(18),
            113,
            maximum_polls,
            services,
        )?;
        return Ok(result);
    }
    let mut allowed = true;
    let high = if member < 6 {
        let offset = u32::from(member as u16) * WARRIOR_BYTES as u32;
        let warrior = 0x448aa0_u32.wrapping_add(offset);
        for at in [0x98, 0xac, 0x60, 0x68, 0x90] {
            if result.memory.word(warrior.wrapping_add(at))? != 0 {
                allowed = false;
                break;
            }
        }
        if allowed
            && (sw(&result.memory, warrior.wrapping_add(0x58))? < 1
                || sw(&result.memory, warrior.wrapping_add(0x50))? < 1)
        {
            allowed = false;
        }
        if result.memory.word(0x438776)? != 0 {
            if result.memory.word(warrior.wrapping_add(0x14))? != 0 {
                allowed = false;
            }
            let limit = sw(&result.memory, warrior.wrapping_add(0xac8))?;
            if limit >= sw(&result.memory, warrior.wrapping_add(0xac6))? {
                allowed = false;
            }
        }
        offset & 0xffff0000
    } else {
        incoming_d0 & 0xffff0000
    };
    result.native_return = high | u32::from(allowed);
    Ok(result)
}

fn copy_warrior(
    result: &mut InventoryPlan,
    source: u32,
    destination: u32,
) -> Result<(), RuleError> {
    for offset in (0..3136).step_by(4) {
        let bytes = read(&result.memory, source.wrapping_add(offset), 4)?;
        write(result, destination.wrapping_add(offset), &bytes)?;
    }
    let bytes = read(&result.memory, source.wrapping_add(3136), 2)?;
    write(result, destination.wrapping_add(3136), &bytes)
}

fn remove_attribute(
    result: &mut InventoryPlan,
    warrior: u32,
    selector_address: u32,
) -> Result<(), RuleError> {
    let selector = sw(&result.memory, selector_address)?;
    if selector == 0 {
        return Ok(());
    }
    if selector < 0 {
        let at = indexed(
            warrior.wrapping_add(0xae),
            selector.wrapping_abs().wrapping_sub(1),
        );
        let amount = sw(&result.memory, 0x45366e)?;
        subtract_word(result, at, amount)
    } else if selector < 16 {
        let at = indexed(warrior.wrapping_add(0xf4), selector.wrapping_sub(1));
        let amount = sw(&result.memory, 0x45366e)?;
        subtract_word(result, at, amount)
    } else {
        word(result, indexed(0x442562, selector.wrapping_sub(30)), 0)
    }
}

fn movecalc(result: &mut InventoryPlan, member: u16) -> Result<(), RuleError> {
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
        .ok_or_else(|| RuleError::Invalid("movecalc projection wraps address space".into()))?
        .max(race_address.checked_add(1).ok_or_else(|| {
            RuleError::Invalid("movecalc race projection wraps address space".into())
        })?);
    let length = (end - start) as usize;
    if length > 1024 * 1024 {
        return Err(RuleError::Invalid(
            "movecalc contiguous observation bridge exceeds one MiB".into(),
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

/// removeitem, 0x14ef76. Caller WORDs are member, slot, sound gate and forced removal.
pub fn removeitem(
    memory: &RenderingMemory,
    frame: u32,
    maximum_polls: usize,
    services: &(impl InventoryServices + ?Sized),
) -> Result<InventoryPlan, RuleError> {
    let mut result = plan(memory);
    let member = result.memory.word(frame.wrapping_add(8))?;
    let slot = result.memory.word(frame.wrapping_add(10))?;
    let sound_gate = result.memory.word(frame.wrapping_add(12))?;
    let warrior = member_address(member);
    let node = warrior.wrapping_add(u32::from(slot) * 22);
    if result.memory.word(0x438776)? != 0 {
        let active = sw(&result.memory, 0x447194)?;
        if result.memory.word(indexed(0x4470b8, active))? != member && sound_gate != 0 {
            warn(
                &mut result,
                frame.wrapping_sub(30),
                10,
                maximum_polls,
                services,
            )?;
            return Ok(result);
        }
    }
    word(&mut result, frame.wrapping_sub(2), 640)?;
    word(&mut result, frame.wrapping_sub(4), 460)?;
    let width = sw(&result.memory, frame.wrapping_sub(2))?.wrapping_sub(356);
    word(&mut result, frame.wrapping_sub(6), width)?;
    let height = sw(&result.memory, frame.wrapping_sub(4))?.wrapping_sub(119);
    word(&mut result, frame.wrapping_sub(8), height)?;
    let item = sw(&result.memory, node.wrapping_add(0x132))?;
    load_item(&mut result, item, false)?;
    let item_type = sw(&result.memory, 0x453656)?.wrapping_abs();
    word(&mut result, 0x453656, item_type)?;
    if sw(&result.memory, node.wrapping_add(0x134))? == -1 {
        word(
            &mut result,
            indexed(warrior.wrapping_add(0xc6), item_type),
            0,
        )?;
        write(&mut result, node.wrapping_add(0x13b), &[0])?;
    } else {
        match item_type {
            0 => {
                let id = sw(&result.memory, 0x4536a2)?;
                let target = if id == sw(&result.memory, warrior.wrapping_add(0xc6))? {
                    0xc6
                } else {
                    0xc8
                };
                word(&mut result, warrior.wrapping_add(target), 0)?;
            }
            2 => {
                word(&mut result, warrior.wrapping_add(0x12e), 0)?;
                word(&mut result, warrior.wrapping_add(0xca), 0)?;
                word(&mut result, warrior.wrapping_add(0x128), 38)?;
                let value = sb(&result.memory, 0x453644)?;
                subtract_word(&mut result, warrior.wrapping_add(0x5c), value)?;
                clamp_negative(&mut result, warrior.wrapping_add(0x5c))?;
                if result.memory.word(warrior.wrapping_add(0xe4))? != 0 {
                    write(&mut result, warrior.wrapping_add(0x9dd), &[1])?;
                }
            }
            3 => {
                subtract_word(&mut result, warrior.wrapping_add(0x5c), 1)?;
                clamp_negative(&mut result, warrior.wrapping_add(0x5c))?;
                word(&mut result, warrior.wrapping_add(0xcc), 0)?;
            }
            15 => {
                word(&mut result, warrior.wrapping_add(0x12c), 0)?;
                write(&mut result, warrior.wrapping_add(0x9dd), &[0])?;
                word(&mut result, warrior.wrapping_add(0xe4), 0)?;
                word(&mut result, warrior.wrapping_add(0x126), 0)?;
                let value = sb(&result.memory, 0x453644)?;
                subtract_word(&mut result, warrior.wrapping_add(0x5c), value)?;
                clamp_negative(&mut result, warrior.wrapping_add(0x5c))?;
            }
            20 => {
                if (result.memory.word(warrior.wrapping_add(0xe0))? != 0
                    || result.memory.word(warrior.wrapping_add(0xe2))? != 0)
                    && result.memory.word(frame.wrapping_add(14))? == 0
                {
                    warn(
                        &mut result,
                        frame.wrapping_sub(30),
                        145,
                        maximum_polls,
                        services,
                    )?;
                    return Ok(result);
                }
            }
            value if value < 24 => word(
                &mut result,
                indexed(warrior.wrapping_add(0xc6), value.wrapping_abs()),
                0,
            )?,
            _ => (),
        }
        if sw(&result.memory, 0x453666)? == 122 {
            let value = sw(&result.memory, 0x453668)?;
            subtract_word(&mut result, warrior.wrapping_add(0x1e), value)?;
        }
        remove_attribute(&mut result, warrior, 0x45366a)?;
        remove_attribute(&mut result, warrior, 0x45366c)?;
        let value = sb(&result.memory, 0x41fc56)?.wrapping_add(sw(&result.memory, 0x453694)?);
        subtract_word(&mut result, warrior.wrapping_add(0x24), value)?;
        if result.memory.word(0x453696)? != 0 {
            let value = sw(&result.memory, 0x4536d0)?
                .wrapping_add(sw(&result.memory, node.wrapping_add(0x138))?);
            subtract_word(&mut result, warrior.wrapping_add(0x28), value)?;
            let quantity = sw(&result.memory, node.wrapping_add(0x134))?;
            subtract_word(&mut result, warrior.wrapping_add(0x26), quantity)?;
            clamp_negative(&mut result, warrior.wrapping_add(0x28))?;
            clamp_negative(&mut result, warrior.wrapping_add(0x26))?;
        }
        let item_type = sw(&result.memory, 0x453656)?;
        word(
            &mut result,
            indexed(warrior.wrapping_add(0xc6), item_type),
            0,
        )?;
        write(&mut result, node.wrapping_add(0x13b), &[0])?;
        copy_warrior(&mut result, warrior, 0x447e5e)?;
        let power_at = if sw(&result.memory, 0x453656)? == 2 {
            6
        } else {
            4
        };
        let value = sb(&result.memory, 0x41fc57)?;
        subtract_word(&mut result, warrior.wrapping_add(power_at), value)?;
        for (source, destination) in [(0x45363c, 0x9e2), (0x45363e, 0x9e8)] {
            let value = sb(&result.memory, source)?;
            subtract_word(&mut result, warrior.wrapping_add(destination), value)?;
        }
        for (first, second, destination, clamp) in [
            (0x41fc58, 0x45363f, 8, true),
            (0x41fc59, 0x453640, 10, true),
        ] {
            let value = sb(&result.memory, first)?.wrapping_add(sb(&result.memory, second)?);
            subtract_word(&mut result, warrior.wrapping_add(destination), value)?;
            if clamp {
                clamp_negative(&mut result, warrior.wrapping_add(destination))?;
            }
        }
        for (source, destination) in [(0x453641, 0x30), (0x453642, 0x22), (0x453643, 0x5a)] {
            let value = sb(&result.memory, source)?;
            subtract_word(&mut result, warrior.wrapping_add(destination), value)?;
        }
        for (first, second, destination, clamp) in [
            (0x41fc5c, 0x453678, 0x130, false),
            (0x41fc5b, 0x45369a, 0x2c, true),
        ] {
            let value = sb(&result.memory, first)?.wrapping_add(sw(&result.memory, second)?);
            subtract_word(&mut result, warrior.wrapping_add(destination), value)?;
            if clamp {
                clamp_negative(&mut result, warrior.wrapping_add(destination))?;
            }
        }
        let value = sb(&result.memory, 0x41fc5d)?.wrapping_add(sb(&result.memory, 0x45368c)?);
        subtract_word(&mut result, warrior.wrapping_add(0x2a), value)?;
        clamp_negative(&mut result, warrior.wrapping_add(0x2a))?;
        for (source, destination) in [
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
                add_word(&mut result, warrior.wrapping_add(destination), value)?;
            }
        }
        if result.memory.word(warrior.wrapping_add(0xe0))? == 0
            && result.memory.word(warrior.wrapping_add(0xe2))? == 0
        {
            word(&mut result, warrior.wrapping_add(0x2a), 0)?;
            word(&mut result, warrior.wrapping_add(0x24), 0)?;
        }
        if result.memory.word(warrior.wrapping_add(0xee))? == 0 {
            word(&mut result, warrior.wrapping_add(0x24), 0)?;
        }
        movecalc(&mut result, member)?;
        let selector = sw(&result.memory, 0x453666)?;
        if selector > 59 && selector < 99 {
            let index = sw(&result.memory, 0x453668)?;
            word(&mut result, indexed(warrior.wrapping_add(0x5e), index), 0)?;
        }
        if selector > 19 && selector < 60 {
            let at = indexed(warrior.wrapping_add(0x5e), selector.wrapping_sub(20));
            if sw(&result.memory, at)? < 0 {
                let value = sw(&result.memory, 0x453668)?;
                subtract_word(&mut result, at, value)?;
            }
            if sw(&result.memory, at)? > 0 {
                word(&mut result, at, 0)?;
            }
        }
    }
    let sound = if result.memory.word(0x453648)? != 0 && sound_gate != 0 {
        Some(sw(&result.memory, 0x453648)?)
    } else if result.memory.word(0x45364a)? != 0 && sound_gate != 0 {
        Some(sw(&result.memory, 0x45364a)?)
    } else {
        None
    };
    if let Some(id) = sound {
        let child = recovered_feedback::sound(&result.memory, id, services)?;
        result.memory = child.memory;
        result
            .effects
            .extend(child.effects.into_iter().map(InventoryEffect::Feedback));
    }
    word(&mut result, indexed(0x442798, member as i16), 1)?;
    result.native_return = 1;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recovered_audio::{
        AudioContext, AudioObservation, AudioReply, AudioRequest, SoundCommand,
    };
    use crate::recovered_feedback::{
        FeedbackArg, FeedbackMemory, FeedbackObservation, FeedbackRequest,
    };
    use crate::recovered_files::{FileReply, FileRequest};
    use crate::recovered_text::PascalString;
    use std::cell::{Cell, RefCell};
    use std::collections::VecDeque;

    #[derive(Default)]
    struct Services {
        sane: RefCell<VecDeque<(SaneRequest, Vec<RecordWrite>)>>,
        sane_seen: RefCell<Vec<SaneRequest>>,
        warnings: RefCell<Vec<i16>>,
        sounds: RefCell<Vec<i16>>,
        ticks: Cell<u32>,
        fail: bool,
    }

    impl FeedbackServices for Services {
        fn observe(
            &self,
            request: &FeedbackRequest,
            memory: &FeedbackMemory,
        ) -> Result<FeedbackObservation, RuleError> {
            if self.fail {
                return Err(RuleError::Invalid("missing test feedback reply".into()));
            }
            let mut next = memory.clone();
            let value = match request {
                FeedbackRequest::TickCount => {
                    let value = self.ticks.get();
                    self.ticks.set(value.wrapping_add(100));
                    value
                }
                FeedbackRequest::Toolbox { opcode: 0xa860, .. } => {
                    next.insert_region(0x44720a, &1_u16.to_be_bytes());
                    1
                }
                FeedbackRequest::Toolbox {
                    opcode: 0xa874,
                    arguments,
                } => {
                    if let [FeedbackArg::Pointer(pointer)] = arguments.as_slice() {
                        next.insert_region(*pointer, &123_u32.to_be_bytes());
                    }
                    0
                }
                FeedbackRequest::GetIndexedString {
                    destination, index, ..
                } => {
                    self.warnings.borrow_mut().push(*index);
                    next.insert_region(*destination, &[1, b'W']);
                    0
                }
                _ => 0,
            };
            Ok(FeedbackObservation {
                value,
                d0: 0x12340000,
                memory: next,
            })
        }

        fn sound_observations(
            &self,
            id: i16,
            context: &AudioContext,
            _: &FeedbackMemory,
        ) -> Result<Vec<AudioObservation>, RuleError> {
            self.sounds.borrow_mut().push(id);
            let mut slot = context.queue_index.wrapping_add(1);
            if slot > context.last_channel {
                slot = 2;
            }
            let channel = context.channels[slot as usize];
            let old = context.resource_handles[slot as usize];
            let mut observations = vec![
                AudioObservation {
                    request: AudioRequest::ChannelStatus { channel, bytes: 24 },
                    reply: AudioReply::ChannelStatus {
                        bytes: [0; 24],
                        error: 0,
                    },
                },
                AudioObservation {
                    request: AudioRequest::UnlockHandle(old),
                    reply: AudioReply::Ignored,
                },
                AudioObservation {
                    request: AudioRequest::GetResource {
                        resource_type: *b"snd ",
                        id: id.wrapping_abs(),
                    },
                    reply: AudioReply::Handle(700),
                },
                AudioObservation {
                    request: AudioRequest::LockHandle(700),
                    reply: AudioReply::Ignored,
                },
            ];
            for code in [4, 3] {
                observations.push(AudioObservation {
                    request: AudioRequest::Immediate {
                        channel,
                        command: SoundCommand {
                            code,
                            param1: 0,
                            param2: 0,
                        },
                    },
                    reply: AudioReply::Error(0),
                });
            }
            observations.push(AudioObservation {
                request: AudioRequest::Play {
                    channel,
                    handle: 700,
                    asynchronous: id > 0,
                },
                reply: AudioReply::Ignored,
            });
            Ok(observations)
        }
    }

    impl InventoryServices for Services {
        fn sane(
            &self,
            request: &SaneRequest,
            _: &RenderingMemory,
        ) -> Result<Vec<RecordWrite>, RuleError> {
            self.sane_seen.borrow_mut().push(request.clone());
            match self.sane.borrow_mut().pop_front() {
                Some((expected, writes)) if &expected == request => Ok(writes),
                _ => Err(RuleError::Invalid("missing exact test SANE reply".into())),
            }
        }
    }

    fn common(frame: u32) -> RenderingMemory {
        let mut memory = RenderingMemory::default();
        for (address, length) in [
            (0x3fe200, 0x100),
            (0x402f00, 0x80),
            (0x41f850, 0x500),
            (0x435500, 0x500),
            (0x438500, 0x1200),
            (0x442500, 0x300),
            (0x443400, 0x400),
            (0x447000, 0x2000),
            (0x448aa0, WARRIOR_BYTES * 2),
            (0x45363c, ITEM_BYTES),
            (0x4781ee, ITEM_BYTES),
            (0x479ed8, 512),
        ] {
            memory.insert_region(address, &vec![0; length]);
        }
        memory.insert_region(frame - 128, &[0xa5; 144]);
        memory
    }

    fn put_word(memory: &mut RenderingMemory, address: u32, value: i16) {
        memory.insert_region(address, &value.to_be_bytes());
    }
    fn put_long(memory: &mut RenderingMemory, address: u32, value: u32) {
        memory.insert_region(address, &value.to_be_bytes());
    }
    fn record_word(record: &mut [u8; ITEM_BYTES], offset: usize, value: i16) {
        record[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
    }

    fn sane_replies(
        frame: u32,
        classification: u16,
        converted: i32,
    ) -> VecDeque<(SaneRequest, Vec<RecordWrite>)> {
        let mut calls = vec![
            (0x280e, 0x16, Some(0x16)),
            (0x1010, 0x16, Some(0x16)),
            (0x101c, 4, Some(0x2a)),
        ];
        if classification != 4 {
            calls.extend([
                (0x100e, 4, Some(0x16)),
                (0x1000, 0xc, Some(0x16)),
                (0x1010, 0x16, Some(4)),
                (0x100e, 4, Some(0x16)),
                (0x1006, 0xc, Some(0x16)),
                (0x1010, 0x16, Some(4)),
                (0x200e, 0x16, Some(0x16)),
                (0x1010, 0x16, Some(0x16)),
                (0x100e, 8, Some(0x16)),
                (0x1004, 4, Some(0x16)),
                (0x1010, 0x16, Some(8)),
                (0x100e, 8, Some(0x16)),
                (0x16, 0x16, None),
                (0x2810, 0x16, Some(0x16)),
            ]);
        }
        calls
            .into_iter()
            .map(|(operation, source, destination)| {
                let request = SaneRequest {
                    operation,
                    source: frame - source,
                    destination: destination.map(|at| frame - at),
                };
                let bytes = if operation == 0x101c {
                    classification.to_be_bytes().to_vec()
                } else if operation == 0x2810 {
                    converted.to_be_bytes().to_vec()
                } else if operation == 0x1010 {
                    0x3f800000_u32.to_be_bytes().to_vec()
                } else {
                    vec![0x11; 10]
                };
                let address = request.destination.unwrap_or(request.source);
                (request, vec![RecordWrite { address, bytes }])
            })
            .collect()
    }

    #[test]
    fn upgrade_preserves_sane_operands_wrapping_modifiers_and_warning_partial_success() {
        let frame = 0x800000;
        let mut memory = common(frame);
        put_word(&mut memory, frame + 8, 0);
        put_word(&mut memory, frame + 10, 1);
        put_word(&mut memory, frame + 12, 3);
        let mut item = [0; ITEM_BYTES];
        record_word(&mut item, 26, 4);
        record_word(&mut item, 148, 99);
        item[78] = 0xfe;
        item[81] = 0x80;
        item[83] = 3;
        item[84] = 4;
        item[85] = 5;
        item[86] = 6;
        memory.insert_region(ITEM_TABLE_ADDRESSES[0] + 3 * ITEM_BYTES as u32, &item);
        let node = 0x448aa0 + 22;
        put_word(&mut memory, node + 0x138, 42);
        memory.insert_region(node + 0x13c, &[20]);
        put_word(&mut memory, 0x448aa4, 32760);
        memory.insert_region(node + 0x141, &[127]);
        let original = memory.clone();
        let services = Services {
            sane: RefCell::new(sane_replies(frame, 0, 42)),
            ..Services::default()
        };
        let result = upgrade(&memory, frame, 4, &services).unwrap();
        assert_eq!(memory, original);
        assert_eq!(result.native_return, 0);
        assert_eq!(result.memory.word(node + 0x138).unwrap(), 43);
        assert_eq!(result.memory.word(0x448ac8).unwrap(), 42);
        assert_eq!(sw(&result.memory, 0x448aa4).unwrap(), 32632);
        assert_eq!(result.memory.byte(node + 0x141).unwrap(), 255);
        assert_eq!(sw(&result.memory, 0x448ac4).unwrap(), -2);
        assert_eq!(services.sane_seen.borrow().len(), 17);
        assert!(services.sane.borrow().is_empty());
        assert!(services.warnings.borrow().is_empty());
        assert!(upgrade(&memory, frame, 4, &Services::default()).is_err());
        assert_eq!(memory, original);

        item[87] = 13;
        item[80] = 0xfe;
        memory.insert_region(ITEM_TABLE_ADDRESSES[0] + 3 * ITEM_BYTES as u32, &item);
        memory.insert_region(ITEM_TABLE_ADDRESSES[0], &[0; ITEM_BYTES]);
        put_word(&mut memory, 0x448aa0 + 0xc6 + 26, 0);
        let services = Services {
            sane: RefCell::new(sane_replies(frame, 4, 0)),
            ..Services::default()
        };
        let result = upgrade(&memory, frame, 4, &services).unwrap();
        assert_eq!(result.native_return, u32::MAX);
        assert_eq!(services.warnings.borrow().as_slice(), [148]);
        assert_eq!(result.memory.byte(node + 0x147).unwrap(), 255);
        assert_eq!(result.memory.word(node + 0x138).unwrap(), 42);
        assert_eq!(result.memory.word(0x448aaa).unwrap(), 0);
        assert_eq!(services.sane_seen.borrow().len(), 3);
        assert!(result.effects.iter().any(|effect| matches!(effect, InventoryEffect::Feedback(FeedbackEffect::Write(RecordWrite { address, .. })) if *address == frame - 48 - 36)));
    }

    fn profile_observations() -> Vec<FileObservation> {
        let mut observations = vec![];
        for (name, size, destination, prefix) in [
            (
                b"Scenario:7:Data Race\0".as_slice(),
                408,
                FileBuffer::RaceProfile,
                0x11,
            ),
            (
                b"Scenario:7:Data MOS\0".as_slice(),
                576,
                FileBuffer::CasteProfile,
                0x22,
            ),
        ] {
            observations.extend([
                FileObservation {
                    request: FileRequest::NumberToString(7),
                    reply: FileReply::Pascal(PascalString::from_payload(b"7").unwrap()),
                },
                FileObservation {
                    request: FileRequest::Open {
                        name: name.to_vec(),
                        mode: b"rb\0".to_vec(),
                    },
                    reply: FileReply::Handle(77),
                },
                FileObservation {
                    request: FileRequest::Seek {
                        handle: 77,
                        offset: 0,
                        origin: 0,
                    },
                    reply: FileReply::Error(-1),
                },
                FileObservation {
                    request: FileRequest::Read {
                        handle: 77,
                        size,
                        count: 1,
                        destination,
                    },
                    reply: FileReply::Read {
                        bytes_written: vec![prefix],
                        elements_read: 0,
                    },
                },
                FileObservation {
                    request: FileRequest::Close(77),
                    reply: FileReply::Ignored,
                },
            ]);
        }
        observations
    }

    #[test]
    fn canuse_executes_both_profiles_keeps_short_tails_and_tests_signed_adjacent_bits() {
        let frame = 0x800000;
        let mut memory = common(frame);
        put_word(&mut memory, frame + 8, 1);
        put_word(&mut memory, frame + 10, 1);
        put_word(&mut memory, frame + 12, 1);
        memory.insert_region(0x44260c, b"Scenario\0");
        memory.insert_region(0x44274c, &[7]);
        memory.insert_region(0x4389a4, &[0xaa; 80]);
        memory.insert_region(0x41f9aa, &[0x20]);
        memory.insert_region(0x41fbac, &[0x20]);
        put_word(&mut memory, 0x41faf0, 3);
        let mut item = [0; ITEM_BYTES];
        record_word(&mut item, 108, 1);
        record_word(&mut item, 110, 1);
        record_word(&mut item, 114, 0x2000);
        item[128] = 0x20;
        memory.insert_region(ITEM_TABLE_ADDRESSES[0] + ITEM_BYTES as u32, &item);
        let original = memory.clone();
        let result = canuse(&memory, frame, 80, &profile_observations()).unwrap();
        assert_eq!(memory, original);
        assert_eq!(result.native_return, 1);
        assert_eq!(result.memory.byte(0x41f85a).unwrap(), 0x11);
        assert_eq!(result.memory.byte(0x41f9f2).unwrap(), 0x22);
        assert_eq!(result.memory.byte(0x41f9aa).unwrap(), 0x20);
        assert_eq!(result.memory.long(0x4434a6).unwrap(), 77);
        let prefix: Vec<_> = result
            .effects
            .iter()
            .filter_map(|effect| match effect {
                InventoryEffect::BitTest {
                    pointer: 0x4536bc,
                    index,
                    ..
                } => Some(*index),
                _ => None,
            })
            .collect();
        assert_eq!(prefix, [0, 1, 2]);
        assert!(canuse(&memory, frame, 80, &[]).is_err());
        assert_eq!(memory, original);

        let mut no_profile = result.memory.clone();
        put_word(&mut no_profile, frame + 8, 0);
        put_word(&mut no_profile, frame + 10, 0);
        put_word(&mut no_profile, frame + 12, 0);
        put_word(&mut no_profile, 0x4536a8, 0);
        put_word(&mut no_profile, 0x4536aa, 0);
        put_word(&mut no_profile, 0x41faf0, 0);
        let denied = canuse(&no_profile, frame, 0, &[]).unwrap();
        assert_eq!(denied.native_return, 0);
        assert_eq!(denied.memory.word(0x435506).unwrap(), 1);
        assert!(denied.effects.iter().any(|effect| matches!(
            effect,
            InventoryEffect::BitTest {
                pointer: 0x4536ae,
                index: -1,
                value: false
            }
        )));
        no_profile.insert_region(0x4536bc, &[0; 8]);
        no_profile.insert_region(0x41fbac + 7, &[0x20]);
        no_profile.insert_region(0x41f9aa + 7, &[0x20]);
        put_word(&mut no_profile, 0x4536ae, 0);
        let sentinel = canuse(&no_profile, frame, 0, &[]).unwrap();
        assert_eq!(sentinel.native_return, 1);
        assert!(sentinel.effects.iter().any(|effect| matches!(
            effect,
            InventoryEffect::BitTest {
                pointer: 0x41fbac,
                index: 58,
                value: true
            }
        )));
    }

    #[test]
    fn cancast_preserves_upper_d0_and_signed_limits_and_composes_warning() {
        let frame = 0x800000;
        let mut memory = common(frame);
        put_word(&mut memory, frame + 8, 6);
        put_word(&mut memory, frame + 10, 0);
        memory.insert_region(0x44273a, &[7]);
        let services = Services::default();
        assert_eq!(
            cancast(&memory, frame, 0xbeefaaaa, 4, &services)
                .unwrap()
                .native_return,
            0xbeef0001
        );
        memory.insert_region(0x44273a, &[0xff]);
        assert_eq!(
            cancast(&memory, frame, 0xbeefaaaa, 4, &services)
                .unwrap()
                .native_return,
            0
        );
        memory.insert_region(0x44273a, &[5]);
        put_word(&mut memory, frame + 8, 0);
        put_word(&mut memory, 0x448aa0 + 0x58, 1);
        put_word(&mut memory, 0x448aa0 + 0x50, 1);
        assert_eq!(
            cancast(&memory, frame, 0xbeefaaaa, 4, &services)
                .unwrap()
                .native_return,
            1
        );
        put_word(&mut memory, 0x438776, 1);
        put_word(&mut memory, 0x448aa0 + 0xac6, 2);
        put_word(&mut memory, 0x448aa0 + 0xac8, 2);
        assert_eq!(
            cancast(&memory, frame, 0xbeefaaaa, 4, &services)
                .unwrap()
                .native_return,
            0
        );
        put_word(&mut memory, 0x448aa0 + 0xac8, 1);
        assert_eq!(
            cancast(&memory, frame, 0xbeefaaaa, 4, &services)
                .unwrap()
                .native_return,
            1
        );
        memory.insert_region(0x442749, &[1]);
        let original = memory.clone();
        let result = cancast(&memory, frame, 0, 4, &services).unwrap();
        assert_eq!(result.native_return, 0);
        assert_eq!(services.warnings.borrow().as_slice(), [113]);
        assert!(result.effects.iter().any(|effect| matches!(effect, InventoryEffect::Feedback(FeedbackEffect::Write(RecordWrite { address, .. })) if *address == frame - 18 - 36)));
        assert!(
            cancast(
                &memory,
                frame,
                0,
                4,
                &Services {
                    fail: true,
                    ..Services::default()
                }
            )
            .is_err()
        );
        assert_eq!(memory, original);
    }

    #[test]
    fn removeitem_keeps_backup_order_real_movement_sound_and_minus_one_shortcut() {
        let frame = 0x800000;
        let mut memory = common(frame);
        // Supply actual contiguous bytes required by the existing movement adapter.
        memory.insert_region(
            0x435664,
            &vec![0; (0x448aa0 + WARRIOR_BYTES as u32 - 0x435664) as usize],
        );
        memory.insert_region(0x435664, &[10]);
        put_word(&mut memory, frame + 8, 0);
        put_word(&mut memory, frame + 10, 1);
        put_word(&mut memory, frame + 12, 1);
        put_word(&mut memory, frame + 14, 0);
        let node = 0x448aa0 + 22;
        put_word(&mut memory, node + 0x132, 3);
        put_word(&mut memory, node + 0x134, 2);
        for at in [
            4, 6, 8, 10, 0x1e, 0x22, 0x24, 0x26, 0x28, 0x2a, 0x2c, 0x30, 0x5a, 0x5c, 0x114, 0x130,
            0x9e2, 0x9e8,
        ] {
            put_word(&mut memory, 0x448aa0 + at, 10);
        }
        put_word(&mut memory, 0x448aa0 + 0x32, 1);
        for at in [0xe0, 0xe2, 0xe4, 0xee] {
            put_word(&mut memory, 0x448aa0 + at, 1);
        }
        put_word(&mut memory, 0x448aa0 + 0xae, 20);
        put_word(&mut memory, 0x448aa0 + 0xf8, 20);
        memory.insert_region(0x41fc56, &[0xfe, 0xfd, 1, 0xff, 0, 2, 0xff, 0xff]);
        let mut item = [0; ITEM_BYTES];
        item[0] = 5;
        item[2] = 3;
        item[3] = 0xfe;
        item[4] = 3;
        item[5] = 2;
        item[6] = 1;
        item[7] = 2;
        item[8] = 4;
        record_word(&mut item, 12, 77);
        record_word(&mut item, 14, 88);
        record_word(&mut item, 26, 2);
        record_word(&mut item, 42, 122);
        record_word(&mut item, 44, 4);
        record_word(&mut item, 46, -1);
        record_word(&mut item, 48, 3);
        record_word(&mut item, 50, 4);
        record_word(&mut item, 140, -2);
        memory.insert_region(ITEM_TABLE_ADDRESSES[0] + 3 * ITEM_BYTES as u32, &item);
        put_long(&mut memory, 0x439570, 1);
        put_word(&mut memory, 0x439576, 2);
        put_word(&mut memory, 0x402f3a, 1);
        let original = memory.clone();
        let services = Services::default();
        let result = removeitem(&memory, frame, 4, &services).unwrap();
        assert_eq!(memory, original);
        assert_eq!(result.native_return, 1);
        assert_eq!(result.memory.word(0x447e5e + 6).unwrap(), 10);
        assert_eq!(result.memory.word(0x448aa0 + 6).unwrap(), 13);
        assert_eq!(result.memory.word(0x447e5e + 0x9e2).unwrap(), 10);
        assert_eq!(result.memory.word(0x448aa0 + 0x9e2).unwrap(), 5);
        assert_eq!(result.memory.word(0x448aa0 + 0xae).unwrap(), 16);
        assert_eq!(result.memory.word(0x448aa0 + 0xf8).unwrap(), 16);
        assert_eq!(result.memory.word(0x448aa0 + 0x114).unwrap(), 8);
        assert_eq!(result.memory.word(0x448aa0 + 0x128).unwrap(), 38);
        assert_eq!(result.memory.word(0x448aa0 + 0x3e).unwrap(), 18);
        assert_eq!(result.memory.word(0x448aa0 + 0x9d8).unwrap(), 1500);
        assert_eq!(result.memory.word(0x442798).unwrap(), 1);
        assert_eq!(services.sounds.borrow().as_slice(), [77]);
        put_word(&mut memory, 0x448aa0 + 0x9e2, -5);
        // Removing five now leaves -10: maximum becomes zero, so no partial writes commit.
        let failing = memory.clone();
        assert!(removeitem(&memory, frame, 4, &Services::default()).is_err());
        assert_eq!(memory, failing);
        put_word(&mut memory, node + 0x134, -1);
        let services = Services::default();
        let shortcut = removeitem(&memory, frame, 4, &services).unwrap();
        assert_eq!(shortcut.native_return, 1);
        assert_eq!(
            shortcut.memory.word(0x448aa0 + 0x9e2).unwrap(),
            (-5_i16) as u16
        );
        assert_eq!(shortcut.memory.word(0x448aa0 + 0xca).unwrap(), 0);
        assert_eq!(services.sounds.borrow().as_slice(), [77]);
        put_word(&mut memory, 0x438776, 1);
        put_word(&mut memory, 0x447194, 0);
        put_word(&mut memory, 0x4470b8, 1);
        let services = Services::default();
        let rejected = removeitem(&memory, frame, 4, &services).unwrap();
        assert_eq!(rejected.native_return, 0);
        assert_eq!(services.warnings.borrow().as_slice(), [10]);
        assert!(rejected.effects.iter().any(|effect| matches!(effect, InventoryEffect::Feedback(FeedbackEffect::Write(RecordWrite { address, .. })) if *address == frame - 30 - 36)));
    }
}
