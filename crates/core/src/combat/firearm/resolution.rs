use super::PreparedFirearm;
use crate::combat::attack::AttackResult;
use crate::combat::damage::{DamageFlags, DamagePacket, resolve_damage};
use crate::{RandomState, RuleError};
use centurions_content::{Warrior, combat::MonsterRecord, records::AttackPacket};

pub fn resolve_enemy(
    warrior: &mut Warrior,
    enemy: &mut MonsterRecord,
    prepared: &PreparedFirearm,
    random: &mut RandomState,
    difficulty: i16,
) -> Result<AttackResult, RuleError> {
    let (minimum, maximum) = AttackPacket(&prepared.packet).duration_bounds();
    random.range(minimum, maximum);
    resolve_enemy_after_duration(warrior, enemy, prepared, random, difficulty)
}

pub fn resolve_enemy_after_duration(
    warrior: &mut Warrior,
    enemy: &mut MonsterRecord,
    prepared: &PreparedFirearm,
    random: &mut RandomState,
    difficulty: i16,
) -> Result<AttackResult, RuleError> {
    let p = &prepared.packet;
    if !prepared.armor_filter && warrior.record().turn_skip_duration()? == 0 {
        let hit = 33_i16
            .wrapping_add(enemy.record().level()?.wrapping_mul(5))
            .wrapping_add(enemy.record().missile_accuracy_bonus()?)
            .wrapping_sub(warrior.record().missile_dodge()?)
            .wrapping_sub(5)
            .wrapping_sub(random.bound(warrior.record().quickness()? / 3))
            .wrapping_sub(
                warrior
                    .record()
                    .guarding()?
                    .wrapping_mul(warrior.record().level()?),
            );
        if random.bound(100) > hit.clamp(15 - difficulty * 5, 90 - difficulty * 5) {
            return Ok(AttackResult {
                feedback: None,
                damage: 0,
                killed: false,
                sounds: vec![],
            });
        }
    }
    let result = resolve_damage(
        &mut warrior.bytes,
        false,
        &DamagePacket {
            bytes: p.clone(),
            attack_count: prepared.shots,
        },
        DamageFlags {
            shield: true,
            armor: true,
            saves: true,
            armor_filter: prepared.armor_filter,
        },
        random,
    )?;
    let mut sounds = vec![];
    if result.shield_sound {
        sounds.push(206);
    }
    if result.total != 0 {
        if !AttackPacket(p).plays_impact_when_wounded() {
            sounds.push(prepared.sound);
        }
        let health = warrior.record().stamina()?.wrapping_sub(result.total);
        warrior.record_mut().set_stamina(health);
        if result.total > 0 {
            warrior.record_mut().mark_damaged();
        }
    }
    Ok(AttackResult {
        feedback: None,
        damage: result.total,
        killed: warrior.record().stamina()? <= 0,
        sounds,
    })
}

pub fn resolve(
    warrior: &mut Warrior,
    enemy: &mut MonsterRecord,
    prepared: &PreparedFirearm,
    random: &mut RandomState,
    difficulty: i16,
) -> Result<AttackResult, RuleError> {
    centurions_content::span(&warrior.bytes, 0, 3138)?;
    centurions_content::span(&enemy.bytes, 0, 288)?;
    centurions_content::span(&prepared.packet, 0, 58)?;
    if prepared.slot >= warrior.inventory()?.len() || !(-2..=2).contains(&difficulty) {
        return Err("invalid firearm resolver context".into());
    }
    let mut next_warrior = warrior.clone();
    let mut next_enemy = enemy.clone();
    let mut next_random = random.clone();
    let result = resolve_inner(
        &mut next_warrior,
        &mut next_enemy,
        prepared,
        &mut next_random,
        difficulty,
    )?;
    *warrior = next_warrior;
    *enemy = next_enemy;
    *random = next_random;
    Ok(result)
}

pub(crate) fn resolve_friendly_after_duration(
    attacker: &Warrior,
    target: &mut Warrior,
    prepared: &PreparedFirearm,
    random: &mut RandomState,
    difficulty: i16,
) -> Result<AttackResult, RuleError> {
    let p = &prepared.packet;
    if !prepared.armor_filter && target.record().turn_skip_duration()? == 0 {
        let hit = 33_i16
            .wrapping_add(attacker.record().missile_accuracy()?)
            .wrapping_sub(target.record().missile_dodge()?)
            .wrapping_sub(5)
            .wrapping_sub(random.bound(target.record().quickness()? / 3))
            .wrapping_sub(
                target
                    .record()
                    .guarding()?
                    .wrapping_mul(target.record().level()?),
            );
        if random.bound(100) > crate::rules::pin(hit, 15 - difficulty * 5, 90 - difficulty * 5) {
            return Ok(AttackResult {
                feedback: None,
                damage: 0,
                killed: false,
                sounds: vec![],
            });
        }
    }
    let result = resolve_damage(
        &mut target.bytes,
        false,
        &DamagePacket {
            bytes: p.clone(),
            attack_count: prepared.shots,
        },
        DamageFlags {
            shield: true,
            armor: true,
            saves: true,
            armor_filter: prepared.armor_filter,
        },
        random,
    )?;
    let previous = target.record().stamina()?;
    if result.total != 0 && previous >= 0 {
        target
            .record_mut()
            .set_stamina(previous.wrapping_sub(result.total));
    }
    Ok(AttackResult {
        feedback: None,
        damage: result.total,
        killed: previous > 0 && target.record().stamina()? < 1,
        sounds: if result.shield_sound {
            vec![206]
        } else {
            vec![]
        },
    })
}

fn resolve_inner(
    warrior: &mut Warrior,
    enemy: &mut MonsterRecord,
    prepared: &PreparedFirearm,
    random: &mut RandomState,
    difficulty: i16,
) -> Result<AttackResult, RuleError> {
    let (minimum, maximum) = AttackPacket(&prepared.packet).duration_bounds();
    random.range(minimum, maximum);
    resolve_after_duration(warrior, enemy, prepared, random, difficulty)
}

pub(crate) fn resolve_after_duration(
    warrior: &mut Warrior,
    enemy: &mut MonsterRecord,
    prepared: &PreparedFirearm,
    random: &mut RandomState,
    difficulty: i16,
) -> Result<AttackResult, RuleError> {
    let p = &prepared.packet;
    let mut hit = 33_i16.wrapping_add(warrior.record().missile_accuracy()?);
    if !prepared.armor_filter && enemy.record().turn_skip_duration() == 0 {
        hit = hit
            .wrapping_sub(enemy.record().missile_dodge_bonus()?)
            .wrapping_sub(random.bound(enemy.record().speed()? / 3));
        hit = hit.wrapping_sub(enemy.record().level()?.wrapping_mul(3));
        hit = hit.wrapping_sub(
            i16::from(enemy.record().guarding()).wrapping_mul(enemy.record().level()?),
        );
        hit = hit.wrapping_add(match enemy.record().size()?.wrapping_abs() {
            0 => -5,
            1 | 2 => 5,
            3 => 10,
            _ => 0,
        });
        let minimum = 15_i16.wrapping_sub(difficulty.wrapping_mul(5));
        let maximum = 90_i16.wrapping_sub(difficulty.wrapping_mul(5));
        if random.bound(100) > hit.clamp(minimum, maximum) {
            return Ok(AttackResult {
                feedback: None,
                damage: 0,
                killed: false,
                sounds: vec![],
            });
        }
    }
    let result = resolve_damage(
        &mut enemy.bytes,
        true,
        &DamagePacket {
            bytes: p.clone(),
            attack_count: prepared.shots,
        },
        DamageFlags {
            shield: true,
            armor: true,
            saves: true,
            armor_filter: prepared.armor_filter,
        },
        random,
    )?;
    let damage = result.total;
    let mut sounds = vec![];
    if result.shield_sound {
        sounds.push(206);
    }
    if damage != 0 {
        let health = enemy.record().stamina()?.wrapping_sub(damage);
        enemy.record_mut().set_stamina(health);
        warrior.record_mut().add_damage_dealt(damage);
        warrior.record_mut().count_hit();
        if health <= 0 {
            warrior.record_mut().count_kill();
        } else {
            sounds.push(prepared.sound);
        }
    }
    Ok(AttackResult {
        feedback: None,
        damage,
        killed: enemy.record().stamina()? <= 0,
        sounds,
    })
}
