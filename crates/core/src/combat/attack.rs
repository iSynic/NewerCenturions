use crate::combat::damage;
mod natural;
use natural::resolve as natural_attack_inner;
pub(super) mod melee_accuracy;
mod warrior_enemy;
use crate::{RandomState, RuleError};
use centurions_content::Warrior;
use centurions_content::combat::MonsterRecord;
use damage::{DamageFlags, DamagePacket, resolve_damage};
use warrior_enemy::resolve as unarmed_attack_inner;

#[derive(Debug, PartialEq, Eq)]
pub struct AttackResult {
    pub feedback: Option<AttackFeedback>,
    pub damage: i16,
    pub killed: bool,
    pub sounds: Vec<i16>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct AttackFeedback {
    pub code: i16,
    pub amount: i16,
    pub recipient: AttackRecipient,
}

#[derive(Debug, PartialEq, Eq)]
pub enum AttackRecipient {
    Attacker,
    Defender,
}

impl AttackFeedback {
    pub(crate) fn defender(code: i16, amount: i16) -> Self {
        Self {
            code,
            amount,
            recipient: AttackRecipient::Defender,
        }
    }
}

pub(crate) fn default_attack_packet() -> Vec<u8> {
    let mut packet = vec![0; 58];
    packet[19] = 8;
    packet[20] = 8;
    packet
}

pub(crate) fn attack_packet(attacks: &[u8]) -> Result<Vec<u8>, RuleError> {
    Ok(centurions_content::span(attacks, 3 * 2842 + 2 * 58, 58)?.to_vec())
}

pub fn unarmed_attack(
    warrior: &mut Warrior,
    enemy: &mut MonsterRecord,
    random: &mut RandomState,
    difficulty: i16,
) -> Result<AttackResult, RuleError> {
    Ok(unarmed_attack_inner(
        warrior,
        enemy,
        random,
        default_attack_packet(),
        &[0; 58],
        difficulty,
        None,
    )?
    .0)
}

pub fn unarmed_attack_with_packet(
    warrior: &mut Warrior,
    enemy: &mut MonsterRecord,
    random: &mut RandomState,
    attacks: &[u8],
    retained: &[u8],
    difficulty: i16,
) -> Result<(AttackResult, Vec<u8>), RuleError> {
    if retained.len() != 58 {
        return Err(RuleError::Invalid("retained attack packet size".into()));
    }
    let base = attack_packet(attacks)?;
    let (mut next_warrior, mut next_enemy, mut next_random) =
        (warrior.clone(), enemy.clone(), random.clone());
    let result = unarmed_attack_inner(
        &mut next_warrior,
        &mut next_enemy,
        &mut next_random,
        base,
        retained,
        difficulty,
        None,
    )?;
    *warrior = next_warrior;
    *enemy = next_enemy;
    *random = next_random;
    Ok((result.0, result.1))
}

pub(crate) fn warrior_attack_with_packet(
    warrior: &mut Warrior,
    enemy: &mut MonsterRecord,
    random: &mut RandomState,
    attacks: &[u8],
    retained: &[u8],
    difficulty: i16,
    weapon: Option<&[u8]>,
) -> Result<(AttackResult, Vec<u8>, bool), RuleError> {
    unarmed_attack_inner(
        warrior,
        enemy,
        random,
        attack_packet(attacks)?,
        retained,
        difficulty,
        weapon,
    )
}

pub fn natural_attack(
    warrior: &mut Warrior,
    enemy: &mut MonsterRecord,
    attack_index: usize,
    random: &mut RandomState,
    difficulty: i16,
) -> Result<AttackResult, RuleError> {
    Ok(natural_attack_inner(
        warrior,
        enemy,
        attack_index,
        random,
        default_attack_packet(),
        &[0; 20],
        difficulty,
    )?
    .0)
}

pub fn natural_attack_with_packet(
    warrior: &mut Warrior,
    enemy: &mut MonsterRecord,
    attack_index: usize,
    random: &mut RandomState,
    attacks: &[u8],
    retained: &[u8],
    difficulty: i16,
) -> Result<(AttackResult, Vec<u8>), RuleError> {
    natural_attack_with_buffs(
        warrior,
        enemy,
        attack_index,
        random,
        attacks,
        retained,
        &[0; 20],
        difficulty,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn natural_attack_with_buffs(
    warrior: &mut Warrior,
    enemy: &mut MonsterRecord,
    attack_index: usize,
    random: &mut RandomState,
    attacks: &[u8],
    retained: &[u8],
    buffs: &[u8],
    difficulty: i16,
) -> Result<(AttackResult, Vec<u8>), RuleError> {
    let (result, packet, _) = natural_attack_with_target(
        warrior,
        enemy,
        attack_index,
        random,
        Some(attacks),
        retained,
        buffs,
        difficulty,
    )?;
    Ok((result, packet))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn natural_attack_with_target(
    warrior: &mut Warrior,
    enemy: &mut MonsterRecord,
    attack_index: usize,
    random: &mut RandomState,
    attacks: Option<&[u8]>,
    retained: &[u8],
    buffs: &[u8],
    difficulty: i16,
) -> Result<(AttackResult, Vec<u8>, bool), RuleError> {
    super::party_buffs::validate(buffs)?;
    if retained.len() != 58 {
        return Err(RuleError::Invalid("retained attack packet size".into()));
    }
    let base = attacks
        .map(attack_packet)
        .transpose()?
        .unwrap_or_else(default_attack_packet);
    let (mut next_warrior, mut next_enemy, mut next_random) =
        (warrior.clone(), enemy.clone(), random.clone());
    let result = natural_attack_inner(
        &mut next_warrior,
        &mut next_enemy,
        attack_index,
        &mut next_random,
        base,
        buffs,
        difficulty,
    )?;
    *warrior = next_warrior;
    *enemy = next_enemy;
    *random = next_random;
    Ok(result)
}

pub(crate) fn resolve_reflected_attack(
    target: &mut [u8],
    packet: &[u8],
    index: usize,
    natural: bool,
    random: &mut RandomState,
) -> Result<AttackResult, RuleError> {
    let source = target.to_vec();
    centurions_content::records::EnemyRecordMut::from_bytes(target).mark_damaged();
    let resolved = resolve_damage(
        target,
        true,
        &DamagePacket {
            bytes: packet.to_vec(),
            attack_count: 0,
        },
        DamageFlags {
            armor: true,
            saves: true,
            ..Default::default()
        },
        random,
    )?;
    let condition = centurions_content::records::EnemyRecord::from_bytes(&source)
        .natural_attack(index)
        .condition;
    let mut damage = resolved.total;
    let mut feedback = None;
    if damage != 0 && natural {
        if let Some(effect) =
            super::natural_conditions::monster_attack_effect(target, &source, index, random)?
        {
            feedback = effect.feedback;
            damage = damage.wrapping_add(effect.extra_damage);
            if condition == 10 && effect.applied {
                centurions_content::records::EnemyRecordMut::from_bytes(target).set_target_body(-1);
            }
        } else {
            damage = 0;
        }
    }
    let health = centurions_content::records::EnemyRecord::from_bytes(target)
        .stamina()?
        .wrapping_sub(damage);
    centurions_content::records::EnemyRecordMut::from_bytes(target).set_stamina(health);
    Ok(AttackResult {
        feedback,
        damage,
        killed: health < 1,
        sounds: vec![],
    })
}
