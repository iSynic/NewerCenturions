//! Enemy natural melee (attack2 0x102038), including reflected conditions.
use super::{AttackResult, melee_accuracy, resolve_reflected_attack};
use crate::{
    RandomState, RuleError,
    combat::{
        damage::{DamageFlags, DamagePacket, resolve_damage},
        natural_conditions, party_buffs,
    },
};
use centurions_content::{
    Warrior,
    combat::MonsterRecord,
    records::{AttackPacketMut, Condition},
};

pub(super) fn resolve(
    defender: &mut Warrior,
    attacker: &mut MonsterRecord,
    attack_index: usize,
    random: &mut RandomState,
    mut packet: Vec<u8>,
    buffs: &[u8],
    difficulty: i16,
) -> Result<(AttackResult, Vec<u8>, bool), RuleError> {
    if attack_index >= 6 {
        return Err(RuleError::Invalid("natural attack outside record".into()));
    }
    let natural = attacker.record().natural_attack(attack_index);
    let weapon = attacker.record().melee_weapon()?;
    if !matches!(weapon, 0 | 9)
        || (weapon == 9 && !matches!(attacker.record().selected_item()?, 0 | 9))
        || !matches!(
            natural.condition,
            0 | 1 | 2 | 4 | 5 | 6 | 9 | 10 | 11 | 14 | 17
        )
    {
        return Err(RuleError::Unsupported {
            action: "combat.attack",
            dependency: "enemy weapon, attack condition, reflection or unconscious target",
        });
    }
    attacker.record_mut().set_selected_item(0);
    let chance = melee_accuracy::enemy_against_warrior(&attacker.record(), &defender.record(), 0)?;
    let hit = random.bound(100)
        <= chance.clamp(
            15_i16.wrapping_sub(difficulty.wrapping_mul(5)),
            90_i16.wrapping_sub(difficulty.wrapping_mul(5)),
        );
    // Reflection is sampled even after a miss; immunity is checked after its draw.
    let reflected = defender.record().condition(Condition::PhysicalReflection)? != 0
        && random.bound(100) < 34
        && attacker.record().weapon_immunity()? == 0;
    if !hit {
        if !reflected {
            defender.record_mut().count_avoided_attack();
        }
        return Ok(no_damage(packet, reflected));
    }
    let bonus = attacker
        .record()
        .melee_bonus()?
        .wrapping_add(attacker.record().condition(Condition::BonusDamage));
    // Rejected default weapon 9 reloads weapon-zero damage, not natural bounds.
    let rejected_weapon = weapon == 9;
    AttackPacketMut(&mut packet).set_damage_bounds(
        bonus.wrapping_add(if rejected_weapon { 1 } else { natural.minimum }),
        bonus.wrapping_add(if rejected_weapon { 0 } else { natural.maximum }),
    );
    AttackPacketMut(&mut packet).set_impact_sound(if rejected_weapon {
        0
    } else if natural.impact_sound == 0 {
        100
    } else {
        natural.impact_sound
    });
    if reflected {
        let result =
            resolve_reflected_attack(&mut attacker.bytes, &packet, attack_index, true, random)?;
        return Ok((result, packet, true));
    }
    let resolved = resolve_damage(
        &mut defender.bytes,
        false,
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
    if resolved.total == 0 {
        defender.record_mut().count_avoided_attack();
        return Ok(no_damage(packet, reflected));
    }
    let effect = natural_conditions::warrior_attack_effect(
        &mut defender.bytes,
        &attacker.bytes,
        attack_index,
        party_buffs::mind_shield(buffs)?,
        random,
    )?;
    let mut damage = resolved.total;
    let mut sounds = vec![];
    if let Some(adjusted) = party_buffs::attack2_damage(buffs, 0, damage)? {
        damage = adjusted;
        sounds.push(-694);
    }
    damage = damage.wrapping_add(effect.extra_damage);
    let health = defender.record().stamina()?.wrapping_sub(damage);
    let mut record = defender.record_mut();
    record.add_damage_received(damage);
    record.count_hit_received();
    record.set_stamina(health);
    if (-9..=0).contains(&health) {
        record.set_bleeding_flag(1);
    }
    record.mark_damaged();
    Ok((
        AttackResult {
            feedback: effect.feedback,
            damage,
            killed: health < 1,
            sounds,
        },
        packet,
        false,
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
