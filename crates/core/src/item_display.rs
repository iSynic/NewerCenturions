//! Inventory presentation from named item fields and original text/resources.
use crate::{
    RuleError,
    recovered_drawing::{BuildingWorld, DrawingEffect, DrawingServices, PictureObservation},
    recovered_geometry::BodyRect,
    recovered_item_ui::{ItemUiEffect, ItemUiReply, ItemUiRequest, ItemUiServices},
    recovered_rendering::RenderingMemory,
    recovered_text::{NumericServices, PascalString},
};
use centurions_content::{Content, InventoryRecord, native_text::NativeText, ui};
use std::collections::BTreeMap;
mod detail;
pub use detail::detail;
mod estimate;
pub use estimate::attack_estimate;

mod row;
pub use row::row;

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
