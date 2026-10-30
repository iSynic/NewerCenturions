//! An enemy's firearm shot resolved against one body.
use super::{
    AttackResult, DamageFlags, DamagePacket, RandomState, RuleError, firearm, resolve_damage,
};
use centurions_content::combat::MONSTER_BYTES;
use centurions_content::combat::MonsterRecord;
use centurions_content::records::{EnemyRecord, EnemyRecordMut, SpellInfoRecord};

/// A monster's firearm shot at another monster: draws the shot's duration, then resolves it.
pub fn resolve_enemy_firearm_body(
    target: &mut MonsterRecord,
    caster: &MonsterRecord,
    prepared: &firearm::PreparedFirearm,
    difficulty: i16,
    random: &mut RandomState,
) -> Result<AttackResult, RuleError> {
    let packet = &prepared.packet;
    centurions_content::span(packet, 0, 58)?;
    let mut next_random = random.clone();
    let (minimum, maximum) = SpellInfoRecord(packet).duration_range();
    next_random.range(minimum, maximum);
    let result = resolve_enemy_firearm_body_after_duration(
        target,
        caster,
        prepared,
        difficulty,
        &mut next_random,
    )?;
    *random = next_random;
    Ok(result)
}

/// A monster's firearm shot at another monster once the duration is drawn: only a plain damage shot
/// of defense category 9 is supported.
pub fn resolve_enemy_firearm_body_after_duration(
    target: &mut MonsterRecord,
    caster: &MonsterRecord,
    prepared: &firearm::PreparedFirearm,
    difficulty: i16,
    random: &mut RandomState,
) -> Result<AttackResult, RuleError> {
    let packet = &prepared.packet;
    centurions_content::span(&target.bytes, 0, MONSTER_BYTES)?;
    centurions_content::span(&caster.bytes, 0, MONSTER_BYTES)?;
    if packet.len() != 58
        || SpellInfoRecord(packet).defense_category() != 9
        || SpellInfoRecord(packet).effect()? != 0
    {
        return Err(RuleError::Unsupported {
            action: "combat.enemy_firearm",
            dependency: "other ranged body effects",
        });
    }
    let mut next_random = random.clone();
    let mut next = target.clone();
    let record = &mut next.bytes;
    if !prepared.armor_filter && EnemyRecord::from_bytes(record).turn_skip_duration() == 0 {
        let mut hit = 33_i16
            .wrapping_add(caster.record().hit_dice()?.wrapping_mul(5))
            .wrapping_add(caster.record().missile_accuracy_bonus()?)
            .wrapping_sub(EnemyRecord::from_bytes(record).missile_dodge_bonus()?)
            .wrapping_sub(next_random.bound(EnemyRecord::from_bytes(record).agility()? / 3))
            .wrapping_sub(EnemyRecord::from_bytes(record).hit_dice()?.wrapping_mul(3))
            .wrapping_sub(
                i16::from(EnemyRecord::from_bytes(record).guarding())
                    .wrapping_mul(EnemyRecord::from_bytes(record).hit_dice()?),
            );
        hit = hit.wrapping_add(
            match EnemyRecord::from_bytes(record).size()?.wrapping_abs() {
                0 => -5,
                1 | 2 => 5,
                3 => 10,
                _ => 0,
            },
        );
        let minimum = 15_i16.wrapping_sub(difficulty.wrapping_mul(5));
        let maximum = 90_i16.wrapping_sub(difficulty.wrapping_mul(5));
        if next_random.bound(100) > hit.clamp(minimum, maximum) {
            *random = next_random;
            return Ok(AttackResult {
                icon: 0,
                feedback: None,
                damage: 0,
                killed: false,
                sounds: vec![],
            });
        }
    }
    let damage = resolve_damage(
        record,
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
        &mut next_random,
    )?;
    let mut sounds = vec![];
    if damage.shield_sound {
        sounds.push(206);
    }
    if damage.total != 0 {
        let health = EnemyRecord::from_bytes(record)
            .stamina()?
            .wrapping_sub(damage.total);
        EnemyRecordMut::from_bytes(record).set_stamina(health);
        EnemyRecordMut::from_bytes(record).mark_been_attacked();
        if EnemyRecord::from_bytes(record).stamina()? > 0 {
            sounds.push(prepared.sound);
        }
    }
    let result = AttackResult {
        icon: damage.sound,
        feedback: None,
        damage: damage.total,
        killed: damage.total != 0 && EnemyRecord::from_bytes(record).stamina()? < 1,
        sounds,
    };
    *target = next;
    *random = next_random;
    Ok(result)
}
