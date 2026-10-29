//! Enemy natural melee (attack2 0x102038), including reflected conditions.
use super::{AttackResult, melee_accuracy};
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
    packet: Vec<u8>,
    buffs: &[u8],
    difficulty: i16,
) -> Result<(AttackResult, Vec<u8>, bool, i16), RuleError> {
    if attack_index >= 6 {
        return Err(RuleError::Invalid("natural attack outside record".into()));
    }
    let weapon = attacker.record().melee_weapon()?;
    if !matches!(weapon, 0 | 9)
        || (weapon == 9 && !matches!(attacker.record().selected_item()?, 0 | 9))
    {
        return Err(RuleError::Unsupported {
            action: "combat.attack",
            dependency: "enemy weapon, attack condition, reflection or unconscious target",
        });
    }
    attacker.record_mut().set_selected_item(0);
    resolve_prepared(
        defender,
        attacker,
        attack_index,
        random,
        packet,
        buffs,
        difficulty,
        weapon != 0,
    )
}

/// attack2 after its caller has loaded natural work and committed selection writers.
#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_prepared(
    defender: &mut Warrior,
    attacker: &mut MonsterRecord,
    attack_index: usize,
    random: &mut RandomState,
    mut packet: Vec<u8>,
    buffs: &[u8],
    difficulty: i16,
    fallback: bool,
) -> Result<(AttackResult, Vec<u8>, bool, i16), RuleError> {
    // attack2 reads natural attack n at 0x93 + 4n of its 0x120-byte record copy, so
    // n up to 34 stays inside the record.
    if attack_index >= 35 {
        return Err(RuleError::Invalid("natural attack outside record".into()));
    }
    let natural = attacker.record().natural_attack(attack_index);
    party_buffs::validate(buffs)?;
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
    let rejected_weapon = fallback;
    AttackPacketMut(&mut packet).set_damage_bounds(
        bonus.wrapping_add(if rejected_weapon { 1 } else { natural.minimum }),
        bonus.wrapping_add(if rejected_weapon { 0 } else { natural.maximum }),
    );
    if !rejected_weapon {
        AttackPacketMut(&mut packet).set_impact_sound(if natural.impact_sound == 0 {
            100
        } else {
            natural.impact_sound
        });
    }
    if reflected {
        let (result, total_delta) = super::resolve_reflected_attack_with_victory(
            &mut attacker.bytes,
            &packet,
            attack_index,
            true,
            None,
            random,
        )?;
        return Ok((result, packet, true, total_delta));
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
    let prior_faction = defender.record().faction()?;
    let effect = natural_conditions::warrior_attack_effect(
        &mut defender.bytes,
        &attacker.bytes,
        attack_index,
        party_buffs::mind_shield(buffs)?,
        random,
    )?;
    effect.credit_attacker(&mut attacker.bytes)?;
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
            icon: resolved.sound,
            feedback: effect.feedback,
            damage,
            killed: health < 1,
            sounds,
        },
        packet,
        false,
        i16::from(natural.condition == 10 && effect.applied && prior_faction == 0),
    ))
}

fn no_damage(packet: Vec<u8>, reflected: bool) -> (AttackResult, Vec<u8>, bool, i16) {
    (
        AttackResult {
            icon: 0,
            feedback: None,
            damage: 0,
            killed: false,
            sounds: vec![],
        },
        packet,
        reflected,
        0,
    )
}
