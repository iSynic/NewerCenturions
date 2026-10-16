//! Host dialog fields supplied by the recovered ShowStats instruction sequence.
use crate::{
    RuleError, Session,
    recovered_character_ui::{
        self, CharacterUiObservation, CharacterUiRequest, CharacterUiServices, CharacterUiState,
    },
    recovered_files::{FileContext, FileObservation},
    recovered_rendering::RenderingMemory,
};
use centurions_content::{native_text::NativeText, ui};
use std::collections::BTreeMap;

pub type Fields = BTreeMap<i16, (String, u16)>;

/// viewcharacter detailed record page, 0x164174..0x164448.
pub fn statistics(state: &Session, member: usize) -> Result<Fields, RuleError> {
    let warrior = state
        .warriors
        .get(member)
        .ok_or_else(|| RuleError::Invalid("statistics warrior is outside the squad".into()))?;
    let mut fields = Fields::new();
    let value = |offset| -> Result<i32, centurions_content::ContentError> {
        Ok(i32::from_be_bytes(
            centurions_content::span(&warrior.bytes, offset, 4)?
                .try_into()
                .unwrap(),
        ))
    };
    let mut score = value(0xa46)?.wrapping_sub(value(0xa42)?) / 20;
    for (offset, multiplier) in [
        (0xa4a, 1_i32),
        (0xa4e, -2),
        (0xa52, -2),
        (0xa56, 1),
        (0xa5a, 3),
        (0xa5e, -75),
        (0xa62, -35),
        (0xa66, -3),
        (0xa6a, 4),
        (0xa6e, 6),
        (0xa72, -1),
    ] {
        score = score.wrapping_add(value(offset)?.wrapping_mul(multiplier));
    }
    fields.insert(6, (score.to_string(), 12));
    fields.insert(37, (warrior.name.clone(), 12));
    for (item, offset) in [
        (26, 0xa46),
        (27, 0xa4a),
        (42, 0xa56),
        (28, 0xa5a),
        (30, 0xa6a),
        (35, 0xa6e),
        (33, 0xa66),
        (31, 0xa42),
        (32, 0xa4e),
        (43, 0xa52),
        (29, 0xa62),
        (34, 0xa5e),
    ] {
        fields.insert(item, (value(offset)?.to_string(), 12));
    }
    fields.insert(71, (value(0xa72)?.wrapping_neg().to_string(), 12));
    Ok(fields)
}

pub fn information(
    state: &Session,
    member: usize,
    native: &NativeText,
    strings: &BTreeMap<i16, Vec<u8>>,
) -> Result<Fields, RuleError> {
    let record = state
        .warriors
        .get(member)
        .ok_or_else(|| RuleError::Invalid("information warrior is outside the squad".into()))?;
    let mut memory = RenderingMemory::default();
    for (address, length) in [
        (0x4394b8, 8),
        (0x4358ce, 4),
        (0x443000, 0x1000),
        (0x442700, 0x100),
        (0x479ed8, 256),
        (0x700f00, 0x200),
    ] {
        memory.insert_region(address, &vec![0; length]);
    }
    for (address, length) in [(0x400af0, 0x50), (0x402f3c, 10)] {
        memory.insert_region(address, native.bytes(address, length)?);
    }
    memory.insert_region(0x44721c, &record.bytes);
    for (index, record) in state.warriors.iter().enumerate() {
        memory.insert_region(0x448aa0 + index as u32 * 3138, &record.bytes);
    }
    memory.insert_region(0x44273a, &[state.warriors.len().saturating_sub(1) as u8]);
    memory.insert_region(0x442788, &state.party_currency.to_be_bytes());
    memory.insert_region(0x701008, &1_i16.to_be_bytes());
    let mut services = DialogServices {
        strings,
        fields: Fields::new(),
        size: 12,
    };
    recovered_character_ui::show_stats(
        &CharacterUiState {
            memory,
            files: FileContext::default(),
        },
        1,
        0x701000,
        &mut services,
    )?;
    Ok(services.fields)
}

struct DialogServices<'a> {
    strings: &'a BTreeMap<i16, Vec<u8>>,
    fields: Fields,
    size: u16,
}

fn read(memory: &RenderingMemory, address: u32, length: usize) -> Result<Vec<u8>, RuleError> {
    (0..length)
        .map(|i| memory.byte(address + i as u32))
        .collect()
}
fn cstring(memory: &RenderingMemory, address: u32) -> Result<Vec<u8>, RuleError> {
    let mut bytes = Vec::new();
    for offset in 0..255 {
        let byte = memory.byte(address + offset)?;
        if byte == 0 {
            return Ok(bytes);
        }
        bytes.push(byte);
    }
    Err(RuleError::Invalid(
        "unterminated character dialog text".into(),
    ))
}
fn pascal(memory: &RenderingMemory, address: u32) -> Result<Vec<u8>, RuleError> {
    read(memory, address, usize::from(memory.byte(address)?) + 1)
}
impl CharacterUiServices for DialogServices<'_> {
    fn observe(
        &mut self,
        request: &CharacterUiRequest,
        source: &RenderingMemory,
    ) -> Result<CharacterUiObservation, RuleError> {
        let mut memory = source.clone();
        match *request {
            CharacterUiRequest::GetDialogItem {
                item,
                type_address,
                handle_address,
                rect_address,
                ..
            } => {
                memory.insert_region(type_address, &8_i16.to_be_bytes());
                memory.insert_region(handle_address, &(item as u32).to_be_bytes());
                memory.insert_region(rect_address, &[0; 8]);
            }
            CharacterUiRequest::TextSize(size) => self.size = size as u16,
            CharacterUiRequest::NumberToString { value, destination } => {
                let text = value.to_string();
                let mut raw = vec![text.len() as u8];
                raw.extend_from_slice(text.as_bytes());
                memory.insert_region(destination, &raw);
            }
            CharacterUiRequest::GetIndexedString {
                destination,
                resource,
                index,
            } => {
                let table = self.strings.get(&resource).ok_or_else(|| {
                    RuleError::Invalid(format!("character STR# {resource} missing"))
                })?;
                if index < 1 || centurions_content::word(table, 0)? < index {
                    return Err(RuleError::Invalid(
                        "character string index outside resource".into(),
                    ));
                }
                let mut offset = 2;
                for _ in 1..index {
                    offset += 1 + usize::from(centurions_content::span(table, offset, 1)?[0]);
                }
                let length = usize::from(centurions_content::span(table, offset, 1)?[0]);
                memory.insert_region(
                    destination,
                    centurions_content::span(table, offset, length + 1)?,
                );
            }
            CharacterUiRequest::SetDialogItemText {
                handle,
                text_address,
            } => {
                let mut raw = vec![0, 1];
                raw.extend(pascal(&memory, text_address)?);
                self.fields
                    .insert(handle as i16, (ui::string_list(&raw)?.remove(0), self.size));
            }
            CharacterUiRequest::CopyCString {
                destination,
                source,
            } => {
                let mut bytes = cstring(&memory, source)?;
                bytes.push(0);
                memory.insert_region(destination, &bytes);
            }
            CharacterUiRequest::AppendCString {
                destination,
                source,
                count,
            } => {
                let mut bytes = cstring(&memory, destination)?;
                bytes.extend(
                    cstring(&memory, source)?
                        .into_iter()
                        .take(usize::from(count)),
                );
                if bytes.len() > 255 {
                    return Err(RuleError::Invalid("character text overflow".into()));
                }
                bytes.push(0);
                memory.insert_region(destination, &bytes);
            }
            CharacterUiRequest::CToPascal(address) => {
                let bytes = cstring(&memory, address)?;
                let mut raw = vec![bytes.len() as u8];
                raw.extend(bytes);
                memory.insert_region(address, &raw);
            }
            CharacterUiRequest::PascalToC(address) => {
                let mut bytes = pascal(&memory, address)?[1..].to_vec();
                bytes.push(0);
                memory.insert_region(address, &bytes);
            }
            CharacterUiRequest::SetPort(_)
            | CharacterUiRequest::BackPixPat(_)
            | CharacterUiRequest::ForeColor(_)
            | CharacterUiRequest::TextFont(_)
            | CharacterUiRequest::RgbForeColor(_) => {}
            _ => {
                return Err(RuleError::Unsupported {
                    action: "character_information",
                    dependency: "unexpected dialog service",
                });
            }
        }
        Ok(CharacterUiObservation { value: 0, memory })
    }
    fn profile_observations(
        &mut self,
        _: i16,
        _: i16,
        _: &FileContext,
    ) -> Result<Vec<FileObservation>, RuleError> {
        Err(RuleError::Unsupported {
            action: "character_information",
            dependency: "unexpected profile service",
        })
    }
}
