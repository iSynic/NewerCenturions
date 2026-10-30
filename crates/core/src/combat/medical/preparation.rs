//! Preparing a medical item's attack, as prepattack prepares any item.
use crate::{RuleError, combat::firearm};
use centurions_content::records::{ItemDefinition, WeaponAttackParameters};
use centurions_content::{Content, Warrior, mission::OpeningMission};

/// Prepares a medicine item's attack packet as an item prepares any attack. A warrior whose
/// inventory rows carry a range modifier, or a damage enhancement the packet would use, is
/// unsupported.
///
/// Original: prepattack
pub(in crate::combat) fn prepared_packet(
    content: &Content,
    mission: &OpeningMission,
    item: i16,
    warrior: &Warrior,
) -> Result<Vec<u8>, RuleError> {
    let weapon = crate::ammunition::definition(content, item)?;
    let ammunition = crate::ammunition::definition(content, 0)?;
    let attacks = &mission
        .combat
        .as_ref()
        .ok_or("item attack content missing")?
        .attacks;
    let packet = firearm::prepare_packet(&weapon, &ammunition, attacks)?;
    let prepared = WeaponAttackParameters::read(&packet)?;
    // Zero damage stays zero under any enhancement, but range still needs the real
    // inventory row, and ammunition enhancement could hide nonzero damage.
    let damage_uses_row = prepared.physical != (0, 0)
        || prepared.elemental_maximum.iter().any(|damage| *damage != 0)
        || ItemDefinition(&ammunition).damage_enhancement() != 0;
    if (0..100).any(|index| {
        let row = warrior.record().inventory_slot(index);
        row.range_modifier() != 0 || damage_uses_row && row.damage_enhancement() != 0
    }) {
        return Err(RuleError::Unsupported {
            action: "combat.item_activation",
            dependency: "prepared item modifier row in combatchoice's register D3 at 1190a8",
        });
    }
    Ok(packet)
}

/// Checks the packet has its fixed size.
pub(super) fn validate_packet(packet: &[u8]) -> Result<(), RuleError> {
    if packet.len() != 58 {
        return Err("prepared medical packet size".into());
    }
    Ok(())
}
