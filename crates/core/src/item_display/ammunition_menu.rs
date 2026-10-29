//! loadammoproc1300ec's MENU 1000 item text: SetItem for item 1, AppendMenu
//! for the rest, each name from GetIndString(getselection(ID) + 1, ID - base + 1).
use super::{BTreeMap, NativeText, RuleError, indexed_string};
use crate::items::AmmunitionEntry;

/// One popup row: its text, whether it is enabled and checked, and whether
/// it is a divider.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AmmunitionMenuItem {
    pub text: String,
    pub enabled: bool,
    pub checked: bool,
    pub divider: bool,
}

pub fn ammunition_menu_items(
    (item, entries): (i16, &[AmmunitionEntry]),
    native: &NativeText,
    strings: &BTreeMap<i16, Vec<u8>>,
) -> Result<Vec<AmmunitionMenuItem>, RuleError> {
    let name = |id: i16| -> Result<String, RuleError> {
        let base = crate::rules::selection_base(id);
        let text = indexed_string(strings, base.wrapping_add(1), id - base + 1)?;
        Ok(encoding_rs::MACINTOSH.decode(text.payload()).0.into_owned())
    };
    let c = |address| -> Result<String, RuleError> { Ok(native.text(address, false)?) };
    // Name, five spaces, NumToString of the LONG reserve, then " Rds" or " Units".
    let stock = |id, stock: i32, units: bool| -> Result<String, RuleError> {
        Ok(format!(
            "{}{}{}{}",
            name(id)?,
            c(0x40316e)?,
            stock,
            c(if units { 0x403166 } else { 0x403160 })?
        ))
    };
    entries
        .iter()
        .map(|entry| {
            let text = match entry {
                AmmunitionEntry::Unload => format!("{}{}", c(0x403174)?, name(item)?),
                AmmunitionEntry::NotLoaded => {
                    format!("{}{}{}", c(0x4031a8)?, name(item)?, c(0x403198)?)
                }
                AmmunitionEntry::Fixed => {
                    format!("{}{}{}", c(0x4031a8)?, name(item)?, c(0x403182)?)
                }
                AmmunitionEntry::Load {
                    description,
                    stock: amount,
                    units,
                    ..
                }
                | AmmunitionEntry::Information {
                    description,
                    stock: amount,
                    units,
                } => stock(*description, *amount, *units)?,
                AmmunitionEntry::Divider => native.text(0x40315e, true)?,
                AmmunitionEntry::Heading(address) => native.text(*address, true)?,
            };
            Ok(AmmunitionMenuItem {
                text,
                enabled: entry.enabled(),
                checked: matches!(entry, AmmunitionEntry::Load { checked: true, .. }),
                divider: *entry == AmmunitionEntry::Divider,
            })
        })
        .collect()
}
