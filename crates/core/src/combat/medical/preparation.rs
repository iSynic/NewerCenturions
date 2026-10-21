//! Medical item admission and unscaled prepattack composition.
use super::unsupported;
use crate::{
    RuleError,
    combat::{firearm, spells},
};
use centurions_content::{
    Content,
    mission::OpeningMission,
    records::{ItemDefinition, WeaponAttackModifiers, WeaponAttackParameters},
};

pub(super) fn prepared_packet(
    content: &Content,
    mission: &OpeningMission,
    item: i16,
) -> Result<Vec<u8>, RuleError> {
    let weapon = content
        .items
        .get(item as usize)
        .ok_or_else(|| RuleError::Invalid("needle definition missing".into()))?;
    let ammunition = content
        .items
        .first()
        .ok_or_else(|| RuleError::Invalid("empty ammunition definition missing".into()))?;
    if weapon.len() != 150
        || ammunition.len() != 150
        || ItemDefinition(weapon).category()? != 28
        || ItemDefinition(weapon).effect_selector()? != 699 - item
        || ItemDefinition(weapon).spell()? != 1401
        || ItemDefinition(weapon).attack_count()? != 1
        || ItemDefinition(weapon).unit_weight()? != 5
        || ItemDefinition(weapon).discarded_when_empty()? != 1
    {
        return Err(unsupported("other medical item definition"));
    }
    let attacks = &mission
        .combat
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("medical attack content missing".into()))?
        .attacks;
    let mut packet = spells::load_packet(attacks, 1401)?;
    let mut attack = WeaponAttackParameters::read(&packet)?;
    firearm::compose_item_attack(
        &mut attack,
        &WeaponAttackModifiers::read(weapon)?,
        &WeaponAttackModifiers::read(ammunition)?,
    );
    attack.write(&mut packet);
    validate_packet(&packet)?;
    Ok(packet)
}

pub(super) fn validate_packet(packet: &[u8]) -> Result<(), RuleError> {
    const PREPARED: [u8; 58] = [
        1, 1, 4, 0, 0, 0, 0, 0, 0, 3, 0, 8, 16, 0, 0, 0, 0, 15, 6, 8, 8, 1, 1, 0, 0, 0, 0, 0, 0, 0,
        10, 1, 0, 0, 0, 0, 1, 175, 1, 174, 0, 57, 0, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0,
    ];
    if packet != PREPARED {
        return Err(unsupported("other prepared medical attack"));
    }
    Ok(())
}
