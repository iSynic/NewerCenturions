//! Warrior melee against an enemy, including damage reflected onto the warrior.
//! attack 0x100af0: retain hit/reflection/damage/critical/sound draw order.
use super::AttackResult;
use crate::{
    RandomState, RuleError,
    combat::damage::{DamageFlags, DamagePacket, resolve_damage},
};
use centurions_content::{
    Warrior,
    combat::MonsterRecord,
    records::{Condition, ItemDefinition},
};

pub(super) fn resolve(
    attacker: &mut Warrior,
    defender: &mut MonsterRecord,
    random: &mut RandomState,
    base: Vec<u8>,
    retained: &[u8],
    difficulty: i16,
    weapon: Option<&[u8]>,
) -> Result<(AttackResult, Vec<u8>, bool), RuleError> {
    if attacker.record().melee_weapon()? != 0 && weapon.is_none() {
        return Err(RuleError::Unsupported {
            action: "combat.attack",
            dependency: "equipped weapon, damage reflection or weapon immunity",
        });
    }
    if attacker.record().attacks_used()? >= attacker.record().attack_limit()? {
        let mut result = no_damage(retained.to_vec(), false);
        result.0.sounds.push(45);
        return Ok(result);
    }
    let mut packet = base;
    let attacks = attacker.record().attacks_used()?.wrapping_add(1);
    attacker.record_mut().set_attacks_used(attacks);
    let chance = super::melee_accuracy::against_enemy(&attacker.bytes, &defender.bytes)?;
    if random.bound(100)
        > chance.clamp(
            15_i16.wrapping_sub(difficulty.wrapping_mul(5)),
            90_i16.wrapping_sub(difficulty.wrapping_mul(5)),
        )
        && defender.record().turn_skip_duration() == 0
    {
        attacker.record_mut().count_miss();
        return Ok(no_damage(packet, false));
    }
    if rejects_weapon(defender.record().weapon_immunity()?, weapon)? {
        return Ok(no_damage(packet, false));
    }
    let reflected =
        defender.record().condition(Condition::PhysicalReflection) != 0 && random.bound(100) < 34;
    crate::combat::warrior_attack::prepare_packet(
        &mut packet,
        &attacker.bytes,
        weapon,
        &attacker.bytes,
    )?;
    let resolved = resolve_damage(
        if reflected {
            &mut attacker.bytes
        } else {
            &mut defender.bytes
        },
        !reflected,
        &DamagePacket {
            bytes: packet.clone(),
            attack_count: 0,
        },
        DamageFlags {
            armor: true,
            saves: true,
            ..Default::default()
        },
        random,
    )?;
    let mut damage = resolved.total;
    if weapon.is_some() && !reflected {
        for category in 0..8 {
            if defender.record().has_creature_category(category) {
                damage =
                    damage.wrapping_add(attacker.record().creature_category_bonus(category)? / 5);
            }
        }
    }
    if damage <= 0 {
        attacker.record_mut().count_miss();
        return Ok(no_damage(packet, reflected));
    }
    if random.bound(100) <= attacker.record().major_wound_chance()? {
        damage = damage.wrapping_mul(2);
    }
    let killed;
    if reflected {
        if attacker.record().turn_skip_duration()? != 0 {
            damage = attacker.record().stamina()?;
        }
        let health = attacker.record().stamina()?.wrapping_sub(damage);
        let mut record = attacker.record_mut();
        record.set_stamina(health);
        record.mark_damaged();
        record.add_damage_received(damage);
        if (-9..=0).contains(&health) {
            record.set_bleeding_flag(1);
        }
        killed = health < 1;
    } else {
        if defender.record().turn_skip_duration() != 0 {
            damage = defender.record().stamina()?;
        }
        defender.record_mut().mark_damaged();
        let health = defender.record().stamina()?.wrapping_sub(damage);
        defender.record_mut().set_stamina(health);
        let mut record = attacker.record_mut();
        record.add_damage_dealt(damage);
        record.count_hit();
        killed = health <= 0;
        if killed {
            record.count_kill();
        }
    }
    let sound = match weapon {
        Some(bytes) => ItemDefinition(bytes).impact_sound()?,
        None => random.bound(2).wrapping_add(401),
    };
    Ok((
        AttackResult {
            feedback: None,
            damage,
            killed,
            sounds: vec![sound],
        },
        packet,
        reflected,
    ))
}

fn no_damage(packet: Vec<u8>, reflected: bool) -> (AttackResult, Vec<u8>, bool) {
    (
        AttackResult {
            feedback: None,
            damage: 0,
            killed: false,
            sounds: vec![],
        },
        packet,
        reflected,
    )
}

fn rejects_weapon(immunity: i16, weapon: Option<&[u8]>) -> Result<bool, RuleError> {
    let weapon = weapon.map(ItemDefinition);
    Ok(match immunity {
        -1 | -2 => weapon
            .map(|weapon| weapon.immunity_class().map(|class| class != immunity))
            .transpose()?
            .unwrap_or(false),
        0 => false,
        _ => {
            weapon
                .map(|weapon| weapon.description_id())
                .transpose()?
                .unwrap_or(0)
                .wrapping_sub(1024)
                != immunity
        }
    })
}
