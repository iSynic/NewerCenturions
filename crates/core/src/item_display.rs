//! Narrow adapters for original inventory display routines; no complete registry dependency.
use crate::{
    RuleError, recovered_drawing::*, recovered_geometry::BodyRect, recovered_item_ui::*,
    recovered_rendering::RenderingMemory, recovered_text::*,
};
use centurions_content::{Content, InventoryRecord, native_text::NativeText, ui, word};
use std::collections::BTreeMap;
mod detail;
pub(crate) use detail::arithmetic as finite_arithmetic;
pub use detail::{attack_estimate, detail};

pub fn row(
    content: &Content,
    item: &InventoryRecord,
    native: &NativeText,
    strings: &BTreeMap<i16, Vec<u8>>,
) -> Result<Vec<ItemUiEffect>, RuleError> {
    let mut memory = RenderingMemory::default();
    memory.insert_region(0x400ef8, native.bytes(0x400ef8, 140)?);
    memory.insert_region(
        0x45363c,
        &crate::ammunition::definition(content, item.id())?,
    );
    memory.insert_region(0x4536d2, &[0; 150]);
    memory.insert_region(0x41fc48, &item.0);
    memory.insert_region(0x4471f2, &[0; 4]);
    memory.insert_region(0x479ed8, &[0; 256]);
    memory.insert_region(0x3fe302, &0x700000_u32.to_be_bytes());
    memory.insert_region(0x700000, &[0; 80]);
    let ammo = word(&item.0, 4)?;
    if ammo != 0 {
        let id = ammo.unsigned_abs() as usize;
        let definition = content
            .items
            .get(id)
            .ok_or("display ammunition unavailable")?;
        memory.insert_region(
            crate::recovered_records::ITEM_TABLE_ADDRESSES[id / 200] + (id % 200 * 150) as u32,
            definition,
        );
    }
    Ok(show_item_stats(&memory, &mut Services { strings })?.effects)
}

struct Services<'a> {
    strings: &'a BTreeMap<i16, Vec<u8>>,
}
fn unavailable<T>() -> Result<T, RuleError> {
    Err("unrequested inventory drawing service".into())
}
impl NumericServices for Services<'_> {
    fn number_to_string(&mut self, value: i32) -> Result<PascalString, RuleError> {
        PascalString::from_payload(value.to_string().as_bytes())
    }
    fn string_to_number(&mut self, _: &PascalString) -> Result<i32, RuleError> {
        unavailable()
    }
}
impl DrawingServices for Services<'_> {
    fn indexed_string(&mut self, resource_id: i16, index: i16) -> Result<PascalString, RuleError> {
        let bytes = self.strings.get(&resource_id).ok_or_else(|| {
            RuleError::Invalid(format!(
                "inventory string resource {resource_id} unavailable"
            ))
        })?;
        let value = ui::string_list(bytes)?
            .get(index.wrapping_sub(1) as usize)
            .cloned()
            .ok_or_else(|| {
                RuleError::Invalid(format!(
                    "inventory string {resource_id}:{index} unavailable"
                ))
            })?;
        let (value, _, errors) = encoding_rs::MACINTOSH.encode(&value);
        if errors {
            return Err("inventory modifier text is not MacRoman".into());
        }
        PascalString::from_payload(&value)
    }
    fn color_icon(&mut self, _: i16) -> Result<u32, RuleError> {
        unavailable()
    }
    fn current_world(&mut self) -> Result<(u32, u32), RuleError> {
        unavailable()
    }
    fn picture(&mut self, _: i16) -> Result<PictureObservation, RuleError> {
        unavailable()
    }
    fn new_world(&mut self, _: i16, _: BodyRect) -> Result<(i16, BuildingWorld), RuleError> {
        unavailable()
    }
}
impl ItemUiServices for Services<'_> {
    fn observe(
        &mut self,
        _: &ItemUiRequest,
        _: &RenderingMemory,
    ) -> Result<ItemUiReply, RuleError> {
        unavailable()
    }
}
