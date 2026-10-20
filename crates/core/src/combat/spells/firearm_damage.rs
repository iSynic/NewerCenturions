//! Firearm damage for combat/spells.
use super::{
    AttackResult, DamageFlags, DamagePacket, RandomState, RuleError, firearm, put_word,
    resolve_damage,
};
use centurions_content::combat::MonsterRecord;
use centurions_content::word;

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
    next_random.range(i16::from(p[13] as i8), i16::from(p[14] as i8));
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
    if p.len() != 58 || p[19] != 9 || word(p, 40)? != 0 {
        return Err(RuleError::Unsupported {
            action: "combat.enemy_firearm",
            dependency: "other ranged body effects",
        });
    }
    let mut next_random = random.clone();
    let mut next = target.clone();
    let m = &mut next.bytes;
    if !prepared.armor_filter && m[0xb3] == 0 {
        let mut hit = 33_i16
            .wrapping_add(word(&caster.bytes, 0)?.wrapping_mul(5))
            .wrapping_add(word(&caster.bytes, 0x66)?)
            .wrapping_sub(word(m, 0x62)?)
            .wrapping_sub(next_random.bound(word(m, 4)? / 3))
            .wrapping_sub(word(m, 0)?.wrapping_mul(3))
            .wrapping_sub(i16::from(m[0x104] as i8).wrapping_mul(word(m, 0)?));
        hit = hit.wrapping_add(match word(m, 0x10)?.wrapping_abs() {
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
        let health = word(m, 0x110)?.wrapping_sub(damage.total);
        put_word(m, 0x110, health);
        m[0x105] = 1;
        if word(m, 0x110)? > 0 {
            sounds.push(prepared.sound);
        }
    }
    let result = AttackResult {
        feedback: None,
        damage: damage.total,
        killed: damage.total != 0 && word(m, 0x110)? < 1,
        sounds,
    };
    *target = next;
    *random = next_random;
    Ok(result)
}
