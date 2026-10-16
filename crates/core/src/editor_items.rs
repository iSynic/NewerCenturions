use crate::recovered_files::{FileBuffer, FileObservation, FileReply, FileRequest};
use crate::recovered_rendering::RenderingMemory;
use crate::recovered_text::PascalString;
use crate::{
    Event, Phase, RuleError, Session, recovered_equipment as equipment,
    recovered_inventory_rules as inventory, rules,
};
use centurions_content::{Content, InventoryRecord, Warrior, span, word};

fn unavailable() -> RuleError {
    RuleError::Unsupported {
        action: "editor.item",
        dependency: "the reached presentation or arithmetic service is not part of item editing",
    }
}

/// edititems 0x134034: wrapped BYTE edits precede signed pin calls.
pub fn change_record(record: &mut InventoryRecord, control: u16) -> Result<(), RuleError> {
    const OFFSETS: [usize; 10] = [13, 15, 16, 11, 17, 18, 19, 14, 21, 20];
    let mut candidate = record.clone();
    let bytes = &mut candidate.0;
    match control {
        40..=49 => {
            let i = OFFSETS[usize::from(control - 40)];
            bytes[i] = bytes[i].wrapping_sub(1);
        }
        52..=61 => {
            let i = OFFSETS[usize::from(control - 52)];
            bytes[i] = bytes[i].wrapping_add(1);
        }
        50 => bytes[2..4].copy_from_slice(&0_i16.to_be_bytes()),
        62 => {
            let capacity = word(bytes, 6)?;
            bytes[2..4].copy_from_slice(&capacity.to_be_bytes());
        }
        51 | 63 => {
            let capacity = word(bytes, 6)?.wrapping_add(if control == 51 { -1 } else { 1 });
            bytes[6..8].copy_from_slice(&capacity.to_be_bytes());
        }
        _ => {
            return Err(RuleError::Invalid(
                "item editor control is outside the original modifier controls".into(),
            ));
        }
    }
    for (offset, minimum, maximum) in [
        (13, 1, 200),
        (15, 0, 200),
        (16, 0, 100),
        (11, 0, 100),
        (17, 0, 100),
        (18, 0, 200),
        (19, 0, 50),
        (14, 0, 50),
        (21, -10, 10),
        (20, 0, 50),
    ] {
        bytes[offset] = rules::pin(i16::from(bytes[offset] as i8), minimum, maximum) as u8;
    }
    let quantity = rules::pin(word(bytes, 2)?, 0, word(bytes, 6)?);
    bytes[2..4].copy_from_slice(&quantity.to_be_bytes());
    *record = candidate;
    Ok(())
}

struct Services<'a> {
    content: &'a Content,
}
macro_rules! unsupported_services {
    ($trait:path { $(fn $name:ident($($args:tt)*) -> $result:ty;)* }) => {
        impl $trait for Services<'_> { $(fn $name($($args)*) -> $result { Err(unavailable()) })* }
    };
}
unsupported_services! { crate::recovered_feedback::FeedbackServices {
    fn observe(&self, _: &crate::recovered_feedback::FeedbackRequest, _: &RenderingMemory) -> Result<crate::recovered_feedback::FeedbackObservation, RuleError>;
    fn sound_observations(&self, _: i16, _: &crate::recovered_audio::AudioContext, _: &RenderingMemory) -> Result<Vec<crate::recovered_audio::AudioObservation>, RuleError>;
}}
impl inventory::InventoryServices for Services<'_> {
    fn defer_warnings(&self) -> bool {
        true
    }
    fn sane(
        &self,
        _: &inventory::SaneRequest,
        _: &RenderingMemory,
    ) -> Result<Vec<crate::recovered_records::RecordWrite>, RuleError> {
        Err(unavailable())
    }
}
unsupported_services! { crate::recovered_drawing::DrawingServices {
    fn color_icon(&mut self, _: i16) -> Result<u32, RuleError>;
    fn indexed_string(&mut self, _: i16, _: i16) -> Result<PascalString, RuleError>;
    fn current_world(&mut self) -> Result<(u32, u32), RuleError>;
    fn picture(&mut self, _: i16) -> Result<crate::recovered_drawing::PictureObservation, RuleError>;
    fn new_world(&mut self, _: i16, _: crate::recovered_geometry::BodyRect) -> Result<(i16, crate::recovered_drawing::BuildingWorld), RuleError>;
}}
unsupported_services! { crate::recovered_item_ui::ItemUiServices {
    fn observe(&mut self, _: &crate::recovered_item_ui::ItemUiRequest, _: &RenderingMemory) -> Result<crate::recovered_item_ui::ItemUiReply, RuleError>;
}}
unsupported_services! { crate::recovered_equipment_ui::EquipmentUiServices {
    fn pen_position(&mut self, _: u32, _: &RenderingMemory) -> Result<Vec<crate::recovered_records::RecordWrite>, RuleError>;
}}
impl crate::recovered_text::NumericServices for Services<'_> {
    fn number_to_string(&mut self, value: i32) -> Result<PascalString, RuleError> {
        PascalString::from_payload(value.to_string().as_bytes())
    }
    fn string_to_number(&mut self, _: &PascalString) -> Result<i32, RuleError> {
        Err(unavailable())
    }
}
impl equipment::EquipmentServices for Services<'_> {
    fn profile_observations(
        &mut self,
        race: i16,
        caste: i16,
        _: &RenderingMemory,
    ) -> Result<Vec<FileObservation>, RuleError> {
        let map = self.content.map(0).ok_or_else(unavailable)?;
        let mut observations = Vec::new();
        for (selector, size, name, data, destination) in [
            (race, 408, "Data Race", &map.races, FileBuffer::RaceProfile),
            (
                caste,
                576,
                "Data MOS",
                &map.profiles,
                FileBuffer::CasteProfile,
            ),
        ] {
            if selector == 0 {
                continue;
            }
            let offset = u32::from(selector.wrapping_sub(1) as u16) * size;
            let bytes = span(data, offset as usize, size as usize)?.to_vec();
            for (request, reply) in [
                (
                    FileRequest::NumberToString(0),
                    FileReply::Pascal(PascalString::from_payload(b"0")?),
                ),
                (
                    FileRequest::Open {
                        name: format!(":0:{name}\0").into_bytes(),
                        mode: b"rb\0".to_vec(),
                    },
                    FileReply::Handle(1),
                ),
                (
                    FileRequest::Seek {
                        handle: 1,
                        offset,
                        origin: 0,
                    },
                    FileReply::Ignored,
                ),
                (
                    FileRequest::Read {
                        handle: 1,
                        size,
                        count: 1,
                        destination,
                    },
                    FileReply::Read {
                        bytes_written: bytes,
                        elements_read: 1,
                    },
                ),
                (FileRequest::Close(1), FileReply::Ignored),
            ] {
                observations.push(FileObservation { request, reply });
            }
        }
        Ok(observations)
    }
}

fn projection(
    warrior: &Warrior,
    content: &Content,
    difficulty: i16,
    slot: usize,
    full_catalog: bool,
) -> Result<RenderingMemory, RuleError> {
    let mut memory = RenderingMemory::default();
    if full_catalog {
        memory.insert_region(0x435500, &vec![0; 0x44a000 - 0x435500]);
    }
    for (address, length) in [
        (0x435500, 4096),
        (0x438700, 1024),
        (0x439400, 1024),
        (0x442500, 1024),
        (0x443300, 2048),
        (0x447000, 8192),
    ] {
        memory.insert_region(address, &vec![0; length]);
    }
    for (address, length) in [
        (0x41f800, 4096),
        (0x45363c, 450),
        (0x4781ee, 150),
        (0x479ed8, 512),
        (0x6fc000, 24576),
    ] {
        memory.insert_region(address, &vec![0; length]);
    }
    memory.insert_region(0x448aa0, &warrior.bytes);
    memory.insert_region(0x442576, &difficulty.to_be_bytes());
    let map = content.map(0).ok_or_else(unavailable)?;
    for (index, movement) in map
        .combat
        .as_ref()
        .ok_or_else(unavailable)?
        .race_moves
        .iter()
        .enumerate()
    {
        memory.insert_region(0x435664 + index as u32, &[*movement as u8]);
    }
    let mut required = std::collections::BTreeSet::from([0]);
    if !full_catalog {
        for item in warrior.inventory()? {
            required.insert(item.id().unsigned_abs() as usize);
            required.insert(word(&item.0, 4)?.unsigned_abs() as usize);
        }
        for offset in (0xc6..0x132).step_by(2) {
            let id = word(&warrior.bytes, offset)?;
            if (0..1000).contains(&id) {
                required.insert(id as usize);
            }
        }
    }
    for (id, definition) in content
        .items
        .iter()
        .enumerate()
        .take(1000)
        .filter(|(id, _)| full_catalog || required.contains(id))
    {
        memory.insert_region(
            crate::recovered_records::ITEM_TABLE_ADDRESSES[id / 200] + (id % 200 * 150) as u32,
            definition,
        );
    }
    memory.insert_region(0x6ffe08, &0_u16.to_be_bytes());
    memory.insert_region(0x6ffe0a, &(slot as u16).to_be_bytes());
    Ok(memory)
}

/// Execute the recovered record mutation silently; the command caller owns feedback and AP.
pub(crate) fn equipment_record(
    warrior: &Warrior,
    content: &Content,
    slot: usize,
    wearing: bool,
) -> Result<(Warrior, Vec<i16>), RuleError> {
    equipment_record_inner(warrior, content, slot, wearing, false)
}

pub(crate) fn equipment_record_forced(
    warrior: &Warrior,
    content: &Content,
    slot: usize,
) -> Result<(Warrior, Vec<i16>), RuleError> {
    equipment_record_inner(warrior, content, slot, false, true)
}

pub(crate) fn equipment_record_restored(
    warrior: &Warrior,
    content: &Content,
    slot: usize,
) -> Result<(Warrior, Vec<i16>), RuleError> {
    equipment_record_inner(warrior, content, slot, true, true)
}

fn equipment_record_inner(
    warrior: &Warrior,
    content: &Content,
    slot: usize,
    wearing: bool,
    forced: bool,
) -> Result<(Warrior, Vec<i16>), RuleError> {
    let item = warrior
        .inventory()?
        .get(slot)
        .cloned()
        .ok_or("equipment slot missing")?;
    let retained = if forced {
        InventoryRecord([0; 22])
    } else {
        item
    };
    let result = retained_equipment(warrior, content, slot, wearing, forced, &retained)?;
    Ok((result.warrior, result.warnings))
}

pub(crate) struct RetainedEquipment {
    pub warrior: Warrior,
    pub record: InventoryRecord,
    pub definition: Vec<u8>,
    pub warnings: Vec<i16>,
    pub native_warnings: Vec<i16>,
    pub native_return: u32,
}

pub(crate) fn retained_equipment(
    warrior: &Warrior,
    content: &Content,
    slot: usize,
    wearing: bool,
    forced: bool,
    retained: &InventoryRecord,
) -> Result<RetainedEquipment, RuleError> {
    let mut memory = projection(warrior, content, 0, slot, false)?;
    memory.insert_region(0x6ffe0e, &u16::from(forced).to_be_bytes());
    memory.insert_region(0x41fc48, &retained.0);
    let mut services = Services { content };
    let warnings;
    let native_return;
    let native_warnings;
    memory = if wearing {
        let result = equipment::wear(
            &memory,
            &crate::RandomState::new(12),
            0x6ffe00,
            equipment::EquipmentLimits {
                maximum_steps: 10000,
                maximum_polls: 100,
                filename_capacity: 256,
            },
            &mut services,
        )?;
        warnings = result.refusal_warnings;
        native_warnings = result.deferred_warnings;
        native_return = u32::from(result.native_return.low_word);
        result.memory
    } else {
        let result = inventory::removeitem(&memory, 0x6ffe00, 100, &services)?;
        warnings = result.warnings;
        native_warnings = warnings.clone();
        native_return = result.native_return;
        result.memory
    };
    if (0x442562..0x442598)
        .filter(|p| !(0x442576..0x442578).contains(p))
        .any(|p| memory.byte(p).unwrap_or(0) != 0)
    {
        return Err(RuleError::Unsupported {
            action: "equipment",
            dependency: "item changes global rules outside the warrior record",
        });
    }
    let mut result = warrior.clone();
    result.bytes = equipment::read(&memory, 0x448aa0, centurions_content::WARRIOR_BYTES)?;
    Warrior::read(&result.bytes)?;
    Ok(RetainedEquipment {
        warrior: result,
        record: InventoryRecord(equipment::read(&memory, 0x41fc48, 22)?.try_into().unwrap()),
        definition: equipment::read(&memory, 0x45363c, 150)?,
        warnings,
        native_warnings,
        native_return,
    })
}

pub fn edit(
    state: &mut Session,
    content: &Content,
    member: usize,
    slot: usize,
    control: u16,
) -> Result<Vec<Event>, RuleError> {
    if state.phase != Phase::Exploration {
        return Err(RuleError::Invalid(
            "item editing requires exploration".into(),
        ));
    }
    let warrior = state.warriors.get(member).ok_or_else(unavailable)?;
    if slot > warrior.inventory()?.len() || slot >= 100 {
        return Err(unavailable());
    }
    let mut item = InventoryRecord(
        span(&warrior.bytes, 0x132 + slot * 22, 22)?
            .try_into()
            .unwrap(),
    );
    let equipped = item.0[9] != 0;
    let mut memory = projection(warrior, content, state.difficulty, slot, true)?;
    memory.insert_region(0x4471ec, &1_u16.to_be_bytes());
    memory.insert_region(0x41fc48, &item.0);
    let frame = 0x6ffe00;
    memory.insert_region(frame + 8, &0_u16.to_be_bytes());
    memory.insert_region(frame + 10, &(slot as u16).to_be_bytes());
    // edititems 0x13442c passes a LONG1: silent removal with the force WORD set.
    memory.insert_region(frame + 14, &1_u16.to_be_bytes());
    let mut services = Services { content };
    if equipped {
        memory = inventory::removeitem(&memory, frame, 100, &services)?.memory;
    }
    change_record(&mut item, control)?;
    let node = equipment::node_address(0, slot as u16);
    if equipped {
        item.0[9] = memory.byte(node + 9)?;
    }
    memory.insert_region(node, &item.0);
    memory.insert_region(0x41fc48, &item.0);
    if equipped {
        memory = equipment::wear(
            &memory,
            &state.random,
            frame,
            equipment::EquipmentLimits {
                maximum_steps: 10000,
                maximum_polls: 100,
                filename_capacity: 256,
            },
            &mut services,
        )?
        .memory;
    }
    let original_flags = (0x442562..0x442598).filter(|p| !(0x442576..0x442578).contains(p));
    if original_flags
        .into_iter()
        .any(|p| memory.byte(p).unwrap_or(0) != 0)
    {
        return Err(unavailable());
    }
    let bytes = equipment::read(&memory, 0x448aa0, centurions_content::WARRIOR_BYTES)?;
    Warrior::read(&bytes)?;
    state.warriors[member].bytes = bytes;
    if state.inventory_record.is_some() {
        state.inventory_record = Some(InventoryRecord(
            equipment::read(&memory, 0x41fc48, 22)?.try_into().unwrap(),
        ));
    }
    if item.id() != 0 {
        state.loaded_item_field32 = content
            .items
            .get(usize::from(item.id().unsigned_abs()))
            .map(|definition| word(definition, 32))
            .transpose()?;
    }
    state.difficulty = memory.word(0x442576)? as i16;
    Ok(vec![Event::Sound { resource_id: 141 }])
}
