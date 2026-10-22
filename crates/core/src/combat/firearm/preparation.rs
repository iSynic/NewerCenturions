use super::{AREA_TARGET_ATTACK, MEDIUM_LASER, POINT_TARGET_ATTACK, PreparedFirearm, SMALL_LASER};
use crate::RuleError;
use centurions_content::records::{
    AttackPacket, AttackPacketMut, ItemDefinition, WeaponAttackModifiers, WeaponAttackParameters,
    item_class,
};
use centurions_content::{Warrior, combat::MonsterRecord, word};

pub fn prepare(
    warrior: &Warrior,
    weapon: &[u8],
    ammo: &[u8],
    attacks: &[u8],
) -> Result<PreparedFirearm, RuleError> {
    let id = warrior.record().equipped_weapon()?;
    if warrior.record().firing_mode_flag() == 0 {
        return Err(RuleError::Invalid("firearm mode is not active".into()));
    }
    if warrior.record().firing_mode_flag() != 1
        || weapon.len() != 150
        || ammo.len() != 150
        || ItemDefinition(weapon).category()?.wrapping_abs() != item_class::MISSILE_WEAPON
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
        .position(|i| i.id() == id && i.equipment_flag() == 1)
        .ok_or_else(|| RuleError::Invalid("equipped firearm absent from inventory".into()))?;
    let item = warrior.record().inventory_slot(slot);
    if item.ammunition_id()?.wrapping_abs() != ItemDefinition(ammo).description_id()?.wrapping_abs()
        || ItemDefinition(weapon).ammunition_caliber()?
            != ItemDefinition(ammo).reserve_ammunition_kind()?
        || ItemDefinition(ammo).category()?.wrapping_abs() != item_class::AMMUNITION
    {
        return Err(RuleError::Invalid(
            "firearm ammunition definition mismatch".into(),
        ));
    }
    let descriptor = ItemDefinition(weapon).spell()?;
    if !matches!(descriptor, POINT_TARGET_ATTACK | AREA_TARGET_ATTACK) {
        return Err(RuleError::Unsupported {
            action: "combat.firearm.prepare",
            dependency: "other attack descriptors",
        });
    }
    let mut p = prepare_packet_with_enhancement(weapon, ammo, attacks, item.damage_enhancement())?;
    AttackPacketMut(&mut p).add_range_modifier(item.range_modifier());
    let rounds = ItemDefinition(weapon).rounds_per_shot()?;
    let impact = AttackPacket(&p).impact_sound()?;
    if !AttackPacket(&p).is_direct_weapon_damage()? {
        return Err(RuleError::Unsupported {
            action: "combat.firearm.prepare",
            dependency: "nonphysical firearm effects",
        });
    }
    Ok(PreparedFirearm {
        packet: p,
        slot,
        shots: i16::from(ItemDefinition(weapon).selected_attack_count()),
        rounds,
        weight: ItemDefinition(weapon).unit_weight()?,
        sound: impact,
        armor_filter: ItemDefinition(weapon)
            .hit_roll_filter()?
            .wrapping_add(ItemDefinition(ammo).hit_roll_filter()?)
            != 0,
    })
}

pub(crate) fn prepare_packet(
    weapon: &[u8],
    ammo: &[u8],
    attacks: &[u8],
) -> Result<Vec<u8>, RuleError> {
    prepare_packet_with_enhancement(weapon, ammo, attacks, 0)
}

fn prepare_packet_with_enhancement(
    weapon: &[u8],
    ammo: &[u8],
    attacks: &[u8],
    enhancement: i8,
) -> Result<Vec<u8>, RuleError> {
    let descriptor = ItemDefinition(weapon).spell()?;
    if !matches!(descriptor, POINT_TARGET_ATTACK | AREA_TARGET_ATTACK) {
        return Err(RuleError::Unsupported {
            action: "combat.firearm.prepare",
            dependency: "other attack descriptors",
        });
    }
    let offset = 3 * 2842 + (descriptor - POINT_TARGET_ATTACK) as usize * 58;
    let mut p = centurions_content::span(attacks, offset, 58)?.to_vec();
    let weapon = WeaponAttackModifiers::read(weapon)?;
    let ammunition = WeaponAttackModifiers::read(ammo)?;
    let mut attack = WeaponAttackParameters::read(&p)?;
    compose_weapon_attack(&mut attack, &weapon, &ammunition, enhancement);
    attack.write(&mut p);
    Ok(p)
}

pub fn prepare_enemy(
    enemy: &mut MonsterRecord,
    weapon: &[u8],
    attacks: &[u8],
) -> Result<PreparedFirearm, RuleError> {
    if !matches!(enemy.record().primary_weapon()?, SMALL_LASER | MEDIUM_LASER)
        || enemy.record().ammunition()? != 0
        || ItemDefinition(weapon).spell()? != POINT_TARGET_ATTACK
    {
        return Err(RuleError::Unsupported {
            action: "combat.enemy_firearm",
            dependency: "other enemy weapons or ammunition",
        });
    }
    let id = enemy.record().primary_weapon()?;
    enemy.record_mut().set_selected_item(id);
    let p = prepare_packet(weapon, &[0; 150], attacks)?;
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
        sound: AttackPacket(&p).impact_sound()?,
        packet: p,
        slot: 0,
        shots: i16::from(ItemDefinition(weapon).selected_attack_count()),
        rounds: ItemDefinition(weapon).rounds_per_shot()?,
        weight: 0,
        armor_filter: ItemDefinition(weapon).hit_roll_filter()? != 0,
    })
}

/// prepattack combines weapon and ammunition, then scales burst damage before
/// Single-precision enhancement. Keep narrowing before multiplication: a byte
/// overflowing at composition is not equivalent to saturating at the end.
pub(in crate::combat) fn compose_weapon_attack(
    attack: &mut WeaponAttackParameters,
    weapon: &WeaponAttackModifiers,
    ammunition: &WeaponAttackModifiers,
    enhancement: i8,
) {
    compose_item_attack(attack, weapon, ammunition);
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

/// Shared prepattack fields before firearm-only burst and enhancement scaling.
/// Medical needles use this preparation without the firearm cost multipliers.
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
