//! Inventory presentation from named item fields and original text/resources.
use crate::RuleError;
use centurions_content::{Content, InventoryRecord, native_text::NativeText, ui};
use output::{BodyRect, Draw, PanelEffect, PanelPlan, PascalString};
use std::collections::BTreeMap;
mod detail;
pub use detail::detail;
mod estimate;
pub use estimate::attack_estimate;
pub mod output;

mod row;
pub use row::row;

/// GetIndString from the loaded STR# resources, encoded as MacRoman.
pub fn indexed_string(
    strings: &BTreeMap<i16, Vec<u8>>,
    resource_id: i16,
    index: i16,
) -> Result<PascalString, RuleError> {
    let bytes = strings.get(&resource_id).ok_or_else(|| {
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
