use super::PreparedFirearm;
use crate::combat::attack::AttackResult;
use crate::combat::damage::{DamageFlags, DamagePacket, resolve_damage};
use crate::{RandomState, RuleError};
use centurions_content::WARRIOR_BYTES;
use centurions_content::combat::MONSTER_BYTES;
use centurions_content::{Warrior, combat::MonsterRecord, records::SpellInfoRecord};

/// A monster's firearm shot at a warrior: draws the shot's duration, then resolves it.
pub fn resolve_enemy(
    warrior: &mut Warrior,
    monster: &mut MonsterRecord,
    prepared: &PreparedFirearm,
    random: &mut RandomState,
    difficulty: i16,
) -> Result<AttackResult, RuleError> {
    let (minimum, maximum) = SpellInfoRecord(&prepared.packet).duration_range();
    random.range(minimum, maximum);
    resolve_enemy_after_duration(warrior, monster, prepared, random, difficulty)
}

/// A monster's firearm shot at a warrior once the duration is drawn: 33 plus 5 per hit die and the
/// missile bonus, less the warrior's missile dodge, 5, a draw below a third of his quickness, and
/// guarding times level; clamped by difficulty, then damage through shield, armor and saves. A
/// warrior who has lost his turn, or a shot with the armor filter, is not rolled to hit.
pub fn resolve_enemy_after_duration(
    warrior: &mut Warrior,
    monster: &mut MonsterRecord,
    prepared: &PreparedFirearm,
    random: &mut RandomState,
    difficulty: i16,
) -> Result<AttackResult, RuleError> {
    let packet = &prepared.packet;
    if !prepared.armor_filter && warrior.record().turn_skip_duration()? == 0 {
        let hit = 33_i16
            .wrapping_add(monster.record().hit_dice()?.wrapping_mul(5))
            .wrapping_add(monster.record().missile_accuracy_bonus()?)
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
                icon: 0,
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
            bytes: packet.clone(),
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
        if !SpellInfoRecord(packet).plays_impact_when_wounded() {
            sounds.push(prepared.sound);
        }
        let health = warrior.record().stamina()?.wrapping_sub(result.total);
        warrior.record_mut().set_stamina(health);
        if result.total > 0 {
            warrior.record_mut().mark_been_attacked();
        }
    }
    Ok(AttackResult {
        icon: result.sound,
        feedback: None,
        damage: result.total,
        killed: warrior.record().stamina()? <= 0,
        sounds,
    })
}

/// A warrior's firearm shot at a monster, committed only if the whole shot succeeds.
pub fn resolve(
    warrior: &mut Warrior,
    monster: &mut MonsterRecord,
    prepared: &PreparedFirearm,
    random: &mut RandomState,
    difficulty: i16,
) -> Result<AttackResult, RuleError> {
    centurions_content::span(&warrior.bytes, 0, WARRIOR_BYTES)?;
    centurions_content::span(&monster.bytes, 0, MONSTER_BYTES)?;
    centurions_content::span(&prepared.packet, 0, 58)?;
    if prepared.slot >= warrior.inventory()?.len() || !(-2..=2).contains(&difficulty) {
        return Err("invalid firearm resolver context".into());
    }
    let mut next_warrior = warrior.clone();
    let mut next_enemy = monster.clone();
    let mut next_random = random.clone();
    let result = resolve_inner(
        &mut next_warrior,
        &mut next_enemy,
        prepared,
        &mut next_random,
        difficulty,
    )?;
    *warrior = next_warrior;
    *monster = next_enemy;
    *random = next_random;
    Ok(result)
}

/// A warrior's firearm shot at another warrior, with the duration already drawn: 33 plus the
/// shooter's missile accuracy, less the target's missile dodge, 5, a draw below a third of his
/// quickness, and guarding times level.
pub(crate) fn resolve_friendly_after_duration(
    attacker: &Warrior,
    target: &mut Warrior,
    prepared: &PreparedFirearm,
    random: &mut RandomState,
    difficulty: i16,
) -> Result<AttackResult, RuleError> {
    let packet = &prepared.packet;
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
                icon: 0,
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
            bytes: packet.clone(),
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
        icon: result.sound,
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

/// Draws the shot's duration, then resolves it.
fn resolve_inner(
    warrior: &mut Warrior,
    monster: &mut MonsterRecord,
    prepared: &PreparedFirearm,
    random: &mut RandomState,
    difficulty: i16,
) -> Result<AttackResult, RuleError> {
    let (minimum, maximum) = SpellInfoRecord(&prepared.packet).duration_range();
    random.range(minimum, maximum);
    resolve_after_duration(warrior, monster, prepared, random, difficulty)
}

/// A warrior's firearm shot at a monster once the duration is drawn: 33 plus missile accuracy, less
/// the monster's missile dodge, a draw against its agility, 3 per hit die and its guarding, and
/// adjusted by its size; clamped by difficulty. A hit does damage through shield, armor and saves
/// and counts toward the warrior's statistics. A shot at a monster that has lost its turn is not
/// rolled to hit, nor is one with the armor filter.
pub(crate) fn resolve_after_duration(
    warrior: &mut Warrior,
    monster: &mut MonsterRecord,
    prepared: &PreparedFirearm,
    random: &mut RandomState,
    difficulty: i16,
) -> Result<AttackResult, RuleError> {
    let packet = &prepared.packet;
    let mut hit = 33_i16.wrapping_add(warrior.record().missile_accuracy()?);
    if !prepared.armor_filter && monster.record().turn_skip_duration() == 0 {
        hit = hit
            .wrapping_sub(monster.record().missile_dodge_bonus()?)
            .wrapping_sub(random.bound(monster.record().agility()? / 3));
        hit = hit.wrapping_sub(monster.record().hit_dice()?.wrapping_mul(3));
        hit = hit.wrapping_sub(
            i16::from(monster.record().guarding()).wrapping_mul(monster.record().hit_dice()?),
        );
        hit = hit.wrapping_add(match monster.record().size()?.wrapping_abs() {
            0 => -5,
            1 | 2 => 5,
            3 => 10,
            _ => 0,
        });
        let minimum = 15_i16.wrapping_sub(difficulty.wrapping_mul(5));
        let maximum = 90_i16.wrapping_sub(difficulty.wrapping_mul(5));
        if random.bound(100) > hit.clamp(minimum, maximum) {
            return Ok(AttackResult {
                icon: 0,
                feedback: None,
                damage: 0,
                killed: false,
                sounds: vec![],
            });
        }
    }
    let result = resolve_damage(
        &mut monster.bytes,
        true,
        &DamagePacket {
            bytes: packet.clone(),
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
        let health = monster.record().stamina()?.wrapping_sub(damage);
        monster.record_mut().set_stamina(health);
        warrior.record_mut().add_damage_dealt(damage);
        warrior.record_mut().count_hit();
        if health <= 0 {
            warrior.record_mut().count_kill();
        } else {
            sounds.push(prepared.sound);
        }
    }
    Ok(AttackResult {
        icon: result.sound,
        feedback: None,
        damage,
        killed: monster.record().stamina()? <= 0,
        sounds,
    })
}
