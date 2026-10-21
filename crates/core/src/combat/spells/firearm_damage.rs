//! Firearm damage for combat/spells.
use super::{
    AttackResult, DamageFlags, DamagePacket, RandomState, RuleError, firearm, resolve_damage,
};
use centurions_content::combat::MonsterRecord;
use centurions_content::records::{AttackPacket, EnemyRecord, EnemyRecordMut};

pub fn resolve_enemy_firearm_body(
    target: &mut MonsterRecord,
    caster: &MonsterRecord,
    prepared: &firearm::PreparedFirearm,
    difficulty: i16,
    random: &mut RandomState,
) -> Result<AttackResult, RuleError> {
    let p = &prepared.packet;
    centurions_content::span(p, 0, 58)?;
    let mut next_random = random.clone();
    let (minimum, maximum) = AttackPacket(p).duration_bounds();
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

pub fn resolve_enemy_firearm_body_after_duration(
    target: &mut MonsterRecord,
    caster: &MonsterRecord,
    prepared: &firearm::PreparedFirearm,
    difficulty: i16,
    random: &mut RandomState,
) -> Result<AttackResult, RuleError> {
    let p = &prepared.packet;
    centurions_content::span(&target.bytes, 0, 288)?;
    centurions_content::span(&caster.bytes, 0, 288)?;
    if p.len() != 58 || AttackPacket(p).defense_category() != 9 || AttackPacket(p).effect()? != 0 {
        return Err(RuleError::Unsupported {
            action: "combat.enemy_firearm",
            dependency: "other ranged body effects",
        });
    }
    let mut next_random = random.clone();
    let mut next = target.clone();
    let m = &mut next.bytes;
    if !prepared.armor_filter && EnemyRecord::from_bytes(m).turn_skip_duration() == 0 {
        let mut hit = 33_i16
            .wrapping_add(caster.record().level()?.wrapping_mul(5))
            .wrapping_add(caster.record().missile_accuracy_bonus()?)
            .wrapping_sub(EnemyRecord::from_bytes(m).missile_dodge_bonus()?)
            .wrapping_sub(next_random.bound(EnemyRecord::from_bytes(m).speed()? / 3))
            .wrapping_sub(EnemyRecord::from_bytes(m).level()?.wrapping_mul(3))
            .wrapping_sub(
                i16::from(EnemyRecord::from_bytes(m).guarding())
                    .wrapping_mul(EnemyRecord::from_bytes(m).level()?),
            );
        hit = hit.wrapping_add(match EnemyRecord::from_bytes(m).size()?.wrapping_abs() {
            0 => -5,
            1 | 2 => 5,
            3 => 10,
            _ => 0,
        });
        let minimum = 15_i16.wrapping_sub(difficulty.wrapping_mul(5));
        let maximum = 90_i16.wrapping_sub(difficulty.wrapping_mul(5));
        if next_random.bound(100) > hit.clamp(minimum, maximum) {
            *random = next_random;
            return Ok(AttackResult {
                feedback: None,
                damage: 0,
                killed: false,
                sounds: vec![],
            });
        }
    }
    let damage = resolve_damage(
        m,
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
        &mut next_random,
    )?;
    let mut sounds = vec![];
    if damage.shield_sound {
        sounds.push(206);
    }
    if damage.total != 0 {
        let health = EnemyRecord::from_bytes(m)
            .stamina()?
            .wrapping_sub(damage.total);
        EnemyRecordMut::from_bytes(m).set_stamina(health);
        EnemyRecordMut::from_bytes(m).mark_damaged();
        if EnemyRecord::from_bytes(m).stamina()? > 0 {
            sounds.push(prepared.sound);
        }
    }
    let result = AttackResult {
        feedback: None,
        damage: damage.total,
        killed: damage.total != 0 && EnemyRecord::from_bytes(m).stamina()? < 1,
        sounds,
    };
    *target = next;
    *random = next_random;
    Ok(result)
}
