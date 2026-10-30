use super::{MEDIUM_LASER, POINT_TARGET_ATTACK, PreparedFirearm, SMALL_LASER};
use crate::RuleError;
use crate::combat::spells::FIRST_SPELL_DESCRIPTOR;
use centurions_content::ITEM_BYTES;
use centurions_content::records::{
    ItemDefinition, SpellInfoRecord, SpellInfoRecordMut, WeaponAttackModifiers,
    WeaponAttackParameters, item_class,
};
use centurions_content::{Warrior, combat::MonsterRecord, word};

/// Readies a warrior's equipped firearm for a shot: the warrior must be in firearm mode with the
/// ammunition the weapon takes, and the packet carries the damage enhancement and range modifier of
/// the weapon's inventory slot.
pub fn prepare(
    warrior: &Warrior,
    weapon: &[u8],
    ammo: &[u8],
    attacks: &[u8],
) -> Result<PreparedFirearm, RuleError> {
    let id = warrior.record().equipped_weapon()?;
    if warrior.record().toggle_flag() == 0 {
        return Err(RuleError::Invalid("firearm mode is not active".into()));
    }
    if warrior.record().toggle_flag() != 1
        || weapon.len() != ITEM_BYTES
        || ammo.len() != ITEM_BYTES
        || ItemDefinition(weapon).item_type()?.wrapping_abs() != item_class::MISSILE_WEAPON
        || ItemDefinition(weapon).description_id()?.wrapping_abs() != id.wrapping_abs()
    {
        return Err(RuleError::Unsupported {
            action: "combat.firearm.prepare",
            dependency: "other secondary firearm",
        });
    }
    let slot = warrior
        .inventory()?
        .iter()
        .position(|i| i.id() == id && i.equipped() == 1)
        .ok_or_else(|| RuleError::Invalid("equipped firearm absent from inventory".into()))?;
    let item = warrior.record().inventory_slot(slot);
    if item.ammunition_id()?.wrapping_abs() != ItemDefinition(ammo).description_id()?.wrapping_abs()
        || ItemDefinition(weapon).ammunition_caliber()?
            != ItemDefinition(ammo).reserve_ammunition_kind()?
        || ItemDefinition(ammo).item_type()?.wrapping_abs() != item_class::AMMUNITION
    {
        return Err(RuleError::Invalid(
            "firearm ammunition definition mismatch".into(),
        ));
    }
    let mut p = prepare_packet_with_enhancement(weapon, ammo, attacks, item.damage_enhancement())?;
    SpellInfoRecordMut(&mut p).add_range_modifier(item.range_modifier());
    let rounds = ItemDefinition(weapon).rounds_per_shot()?;
    let impact = SpellInfoRecord(&p).impact_sound()?;
    Ok(PreparedFirearm {
        packet: p,
        slot,
        shots: i16::from(ItemDefinition(weapon).selected_attack_count()),
        rounds,
        weight: ItemDefinition(weapon).charge_weight()?,
        sound: impact,
        armor_filter: ItemDefinition(weapon)
            .hit_roll_filter()?
            .wrapping_add(ItemDefinition(ammo).hit_roll_filter()?)
            != 0,
    })
}

/// The attack packet for a weapon and its ammunition, with no damage enhancement.
pub(crate) fn prepare_packet(
    weapon: &[u8],
    ammo: &[u8],
    attacks: &[u8],
) -> Result<Vec<u8>, RuleError> {
    prepare_packet_with_enhancement(weapon, ammo, attacks, 0)
}

/// The attack packet for a weapon and its ammunition. A weapon whose spell descriptor is below the
/// first spell (1101) uses the base attack packet scaled by the weapon and ammunition; any other
/// uses its spell's packet with the weapon and ammunition added to it.
pub(crate) fn prepare_packet_with_enhancement(
    weapon: &[u8],
    ammo: &[u8],
    attacks: &[u8],
    enhancement: i8,
) -> Result<Vec<u8>, RuleError> {
    let descriptor = ItemDefinition(weapon).spell()?;
    let weapon = WeaponAttackModifiers::read(weapon)?;
    let ammunition = WeaponAttackModifiers::read(ammo)?;
    // Below descriptor 1101 the base attack packet stands in for the spell packet
    // and the weapon and ammunition fields.
    if descriptor < FIRST_SPELL_DESCRIPTOR {
        let mut packet = crate::combat::spells::load_packet(attacks, FALLBACK_ATTACK)?;
        let mut attack = WeaponAttackParameters::read(&packet)?;
        scale_firearm_attack(&mut attack, &weapon, &ammunition, enhancement);
        attack.write(&mut packet);
        return Ok(packet);
    }
    let mut packet = crate::combat::spells::load_packet(attacks, descriptor)?;
    let mut attack = WeaponAttackParameters::read(&packet)?;
    compose_weapon_attack(&mut attack, &weapon, &ammunition, enhancement);
    attack.write(&mut packet);
    Ok(packet)
}

/// The base attack packet every attack starts from.
const FALLBACK_ATTACK: i16 = 4103;

/// Readies a monster's laser: only the small and medium lasers, with no ammunition and a plain
/// point-target direct-damage attack, are accepted.
pub fn prepare_enemy(
    monster: &mut MonsterRecord,
    weapon: &[u8],
    attacks: &[u8],
) -> Result<PreparedFirearm, RuleError> {
    if !matches!(
        monster.record().primary_weapon()?,
        SMALL_LASER | MEDIUM_LASER
    ) || monster.record().ammunition()? != 0
        || ItemDefinition(weapon).spell()? != POINT_TARGET_ATTACK
    {
        return Err(RuleError::Unsupported {
            action: "combat.enemy_firearm",
            dependency: "other enemy weapons or ammunition",
        });
    }
    let id = monster.record().primary_weapon()?;
    monster.record_mut().set_weapon(id);
    let p = prepare_packet(weapon, &[0; ITEM_BYTES], attacks)?;
    if p[19] != 9
        || p[20] != 9
        || word(&p, 40)? != 0
        || p[23] != 0
        || p[25..30].iter().any(|v| *v != 0)
    {
        return Err(RuleError::Unsupported {
            action: "combat.enemy_firearm",
            dependency: "other enemy firearm effects",
        });
    }
    Ok(PreparedFirearm {
        sound: SpellInfoRecord(&p).impact_sound()?,
        packet: p,
        slot: 0,
        shots: i16::from(ItemDefinition(weapon).selected_attack_count()),
        rounds: ItemDefinition(weapon).rounds_per_shot()?,
        weight: 0,
        armor_filter: ItemDefinition(weapon).hit_roll_filter()? != 0,
    })
}

/// Combines weapon and ammunition, then scales burst damage before the
/// single-precision enhancement. Each field narrows to a byte before multiplying;
/// a byte that overflows here is not the same as one saturated at the end.
///
/// Original: prepattack
pub(in crate::combat) fn compose_weapon_attack(
    attack: &mut WeaponAttackParameters,
    weapon: &WeaponAttackModifiers,
    ammunition: &WeaponAttackModifiers,
    enhancement: i8,
) {
    compose_item_attack(attack, weapon, ammunition);
    scale_firearm_attack(attack, weapon, ammunition, enhancement);
}

/// Scales a firearm attack: rounds first, then enhancement.
///
/// Original: prepattack
fn scale_firearm_attack(
    attack: &mut WeaponAttackParameters,
    weapon: &WeaponAttackModifiers,
    ammunition: &WeaponAttackModifiers,
    enhancement: i8,
) {
    attack.physical.0 = attack.physical.0.wrapping_mul(weapon.rounds);
    attack.physical.1 = attack.physical.1.wrapping_mul(weapon.rounds);
    for damage in &mut attack.elemental_maximum {
        *damage = damage.wrapping_mul(weapon.rounds as u8);
    }
    let adjustment = i16::from(enhancement).wrapping_add(i16::from(ammunition.enhancement));
    let factor = (100_i32 + i32::from(adjustment)) as f32 / 100.0;
    attack.physical.0 = (f32::from(attack.physical.0) * factor).trunc() as i16;
    attack.physical.1 = (f32::from(attack.physical.1) * factor).trunc() as i16;
    for damage in &mut attack.elemental_maximum {
        *damage = (f32::from(*damage) * factor).trunc() as u8;
    }
}

/// The attack fields every prepared item shares, before a firearm's burst and
/// enhancement. Medical needles use it without the firearm multipliers.
///
/// Original: prepattack
pub(in crate::combat) fn compose_item_attack(
    attack: &mut WeaponAttackParameters,
    weapon: &WeaponAttackModifiers,
    ammunition: &WeaponAttackModifiers,
) {
    attack.count = weapon.count;
    attack.target_filter = weapon.target_filter;
    attack.health_effect = weapon.health_effect;
    attack.range = attack
        .range
        .wrapping_add(weapon.range.wrapping_add(ammunition.range) as u8);
    attack.duration = attack
        .duration
        .wrapping_add(weapon.duration)
        .wrapping_add(ammunition.duration);
    attack.physical.1 = weapon
        .physical_maximum
        .wrapping_add(ammunition.physical_maximum);
    if attack.physical.1 != 0 {
        attack.physical.0 = 1;
    }
    for (element, maximum) in attack.elemental_maximum.iter_mut().enumerate() {
        *maximum = weapon.elemental_maximum[element]
            .wrapping_add(ammunition.elemental_maximum[element]) as u8;
    }
    attack.launch_sound = if ammunition.launch_sound != 0 {
        ammunition.launch_sound
    } else {
        weapon.launch_sound
    };
    attack.impact_sound = if ammunition.impact_sound != 0 {
        ammunition.impact_sound
    } else {
        weapon.impact_sound
    };
    attack.target_mode = if ammunition.target_mode != 0 {
        ammunition.target_mode as u8
    } else {
        weapon.target_mode.wrapping_add(ammunition.target_mode) as u8
    };
    attack.area = if ammunition.area != 0 {
        ammunition.area as u8
    } else {
        weapon.area as u8
    };
}
