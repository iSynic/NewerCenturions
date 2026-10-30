use crate::combat::damage;
mod natural;
use natural::resolve as natural_attack_inner;
pub(crate) use natural::resolve_prepared as natural_attack_prepared;
pub(super) mod melee_accuracy;
mod warrior_enemy;
pub use super::warrior_attack::{WarriorMelee, warrior_melee};
use crate::{RandomState, RuleError};
use centurions_content::Warrior;
use centurions_content::combat::MonsterRecord;
use damage::{DamageFlags, DamagePacket, resolve_damage};
use warrior_enemy::resolve as unarmed_attack_inner;
pub(crate) use warrior_enemy::resolve_prepared as warrior_attack_prepared;

/// What a blow did: the damage dealt, whether it killed, any refusal or effect message, and the
/// sounds to play.
#[derive(Debug, PartialEq, Eq)]
pub struct AttackResult {
    pub feedback: Option<AttackFeedback>,
    pub damage: i16,
    pub killed: bool,
    pub sounds: Vec<i16>,
    /// The damage icon a plain hit shows; 0 when no damage was resolved.
    pub icon: i16,
}

/// A message the blow produced: its code and amount, and who sees it.
#[derive(Debug, PartialEq, Eq)]
pub struct AttackFeedback {
    pub code: i16,
    pub amount: i16,
    pub recipient: AttackRecipient,
}

/// Whose side an attack message is shown against.
#[derive(Debug, PartialEq, Eq)]
pub enum AttackRecipient {
    Attacker,
    Defender,
}

impl AttackFeedback {
    /// A message about the defender.
    pub(crate) fn defender(code: i16, amount: i16) -> Self {
        Self {
            code,
            amount,
            recipient: AttackRecipient::Defender,
        }
    }
}

/// The 58-byte attack packet used when no combat attack table is supplied: all zero except 8 in
/// bytes 19 and 20.
pub(crate) fn default_spell_info() -> Vec<u8> {
    let mut packet = vec![0; 58];
    packet[19] = 8;
    packet[20] = 8;
    packet
}

/// The 58-byte attack packet at the fixed place in the combat attack table that plain attacks use.
pub(crate) fn spell_info(attacks: &[u8]) -> Result<Vec<u8>, RuleError> {
    Ok(centurions_content::span(attacks, 3 * 2842 + 2 * 58, 58)?.to_vec())
}

/// A warrior strikes a monster bare-handed with the default attack packet.
pub fn unarmed_attack(
    warrior: &mut Warrior,
    monster: &mut MonsterRecord,
    random: &mut RandomState,
    difficulty: i16,
) -> Result<AttackResult, RuleError> {
    Ok(unarmed_attack_inner(
        warrior,
        monster,
        random,
        default_spell_info(),
        &[0; 58],
        difficulty,
        None,
        true,
    )?
    .0)
}

/// A warrior strikes a monster bare-handed from the table's attack packet; the whole blow is
/// committed only if it succeeds. Returns the result and the packet it leaves behind.
pub fn unarmed_attack_with_packet(
    warrior: &mut Warrior,
    monster: &mut MonsterRecord,
    random: &mut RandomState,
    attacks: &[u8],
    info: &[u8],
    difficulty: i16,
) -> Result<(AttackResult, Vec<u8>), RuleError> {
    if info.len() != 58 {
        return Err(RuleError::Invalid("retained attack packet size".into()));
    }
    let base = spell_info(attacks)?;
    let (mut next_warrior, mut next_enemy, mut next_random) =
        (warrior.clone(), monster.clone(), random.clone());
    let result = unarmed_attack_inner(
        &mut next_warrior,
        &mut next_enemy,
        &mut next_random,
        base,
        info,
        difficulty,
        None,
        true,
    )?;
    *warrior = next_warrior;
    *monster = next_enemy;
    *random = next_random;
    Ok((result.0, result.1))
}

/// A warrior strikes a monster, with the weapon if one is given, from the table's attack packet.
/// Returns the result, the packet, and whether the blow was reflected.
pub fn warrior_attack_with_packet(
    warrior: &mut Warrior,
    monster: &mut MonsterRecord,
    random: &mut RandomState,
    attacks: &[u8],
    info: &[u8],
    difficulty: i16,
    weapon: Option<&[u8]>,
) -> Result<(AttackResult, Vec<u8>, bool), RuleError> {
    warrior_attack_with_base(
        warrior,
        monster,
        random,
        spell_info(attacks)?,
        true,
        info,
        difficulty,
        weapon,
    )
}

/// A warrior attacks an enemy from a prepared packet. A spell weapon supplies its
/// spell's packet and may skip the weapon-condition roll.
///
/// Original: attack
#[allow(clippy::too_many_arguments)]
pub(crate) fn warrior_attack_with_base(
    warrior: &mut Warrior,
    monster: &mut MonsterRecord,
    random: &mut RandomState,
    base: Vec<u8>,
    rolls_condition: bool,
    info: &[u8],
    difficulty: i16,
    weapon: Option<&[u8]>,
) -> Result<(AttackResult, Vec<u8>, bool), RuleError> {
    unarmed_attack_inner(
        warrior,
        monster,
        random,
        base,
        info,
        difficulty,
        weapon,
        rolls_condition,
    )
}

/// A monster's natural attack on a warrior using the default attack packet and no party buffs.
pub fn natural_attack(
    warrior: &mut Warrior,
    monster: &mut MonsterRecord,
    attack_index: usize,
    random: &mut RandomState,
    difficulty: i16,
) -> Result<AttackResult, RuleError> {
    Ok(natural_attack_inner(
        warrior,
        monster,
        attack_index,
        random,
        default_spell_info(),
        &[0; 20],
        difficulty,
    )?
    .0)
}

/// A monster's natural attack on a warrior using the table's attack packet and no party buffs.
pub fn natural_attack_with_packet(
    warrior: &mut Warrior,
    monster: &mut MonsterRecord,
    attack_index: usize,
    random: &mut RandomState,
    attacks: &[u8],
    info: &[u8],
    difficulty: i16,
) -> Result<(AttackResult, Vec<u8>), RuleError> {
    natural_attack_with_buffs(
        warrior,
        monster,
        attack_index,
        random,
        attacks,
        info,
        &[0; 20],
        difficulty,
    )
}

/// A monster's natural attack on a warrior with the party's buffs in force. Returns the result and
/// the packet.
#[allow(clippy::too_many_arguments)]
pub fn natural_attack_with_buffs(
    warrior: &mut Warrior,
    monster: &mut MonsterRecord,
    attack_index: usize,
    random: &mut RandomState,
    attacks: &[u8],
    info: &[u8],
    buffs: &[u8],
    difficulty: i16,
) -> Result<(AttackResult, Vec<u8>), RuleError> {
    let (result, packet, _, _) = natural_attack_with_target(
        warrior,
        monster,
        attack_index,
        random,
        Some(attacks),
        info,
        buffs,
        difficulty,
    )?;
    Ok((result, packet))
}

/// A monster's natural attack on a warrior, committed only if the whole attack succeeds. Also says
/// whether the blow was reflected and the change to the hostile count.
#[allow(clippy::too_many_arguments)]
pub(crate) fn natural_attack_with_target(
    warrior: &mut Warrior,
    monster: &mut MonsterRecord,
    attack_index: usize,
    random: &mut RandomState,
    attacks: Option<&[u8]>,
    info: &[u8],
    buffs: &[u8],
    difficulty: i16,
) -> Result<(AttackResult, Vec<u8>, bool, i16), RuleError> {
    super::party_buffs::validate(buffs)?;
    if info.len() != 58 {
        return Err(RuleError::Invalid("retained attack packet size".into()));
    }
    let base = attacks
        .map(spell_info)
        .transpose()?
        .unwrap_or_else(default_spell_info);
    let (mut next_warrior, mut next_enemy, mut next_random) =
        (warrior.clone(), monster.clone(), random.clone());
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
    *monster = next_enemy;
    *random = next_random;
    Ok(result)
}

/// A blow thrown back at the monster that struck: damage, then the attack's condition when it is a
/// natural attack.
pub(crate) fn resolve_reflected_attack(
    target: &mut [u8],
    packet: &[u8],
    index: usize,
    natural: bool,
    weapon_condition: Option<centurions_content::records::WeaponCondition>,
    random: &mut RandomState,
) -> Result<AttackResult, RuleError> {
    resolve_reflected_attack_with_victory(target, packet, index, natural, weapon_condition, random)
        .map(|(result, _)| result)
}

/// As the plain reflected blow, also giving the change to the hostile count: -1 when a friendly
/// monster is struck by its own reflected condition 10.
pub(crate) fn resolve_reflected_attack_with_victory(
    target: &mut [u8],
    packet: &[u8],
    index: usize,
    natural: bool,
    weapon_condition: Option<centurions_content::records::WeaponCondition>,
    random: &mut RandomState,
) -> Result<(AttackResult, i16), RuleError> {
    let source = target.to_vec();
    centurions_content::records::EnemyRecordMut::from_bytes(target).mark_been_attacked();
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
    let mut total_delta = 0;
    if damage != 0 && natural {
        if let Some(effect) =
            super::natural_conditions::monster_attack_effect(target, &source, index, random)?
        {
            // The reflected attacker is its own target.
            effect.credit_attacker(target)?;
            feedback = effect.feedback;
            damage = damage.wrapping_add(effect.extra_damage);
            if condition == 10 && effect.applied {
                // A reflected friendly monster strikes itself.
                if centurions_content::records::EnemyRecord::from_bytes(&source).faction()? == 0 {
                    total_delta = -1;
                }
                centurions_content::records::EnemyRecordMut::from_bytes(target).set_target(-1);
            }
        } else {
            damage = 0;
        }
    }
    // A reflected blow carries the weapon condition back to the attacker.
    if let Some(condition) = weapon_condition.filter(|_| damage != 0) {
        feedback = Some(super::natural_conditions::apply_weapon_condition(
            target,
            centurions_content::records::DamageTargetKind::Enemy,
            condition,
        )?);
    }
    let health = centurions_content::records::EnemyRecord::from_bytes(target)
        .stamina()?
        .wrapping_sub(damage);
    centurions_content::records::EnemyRecordMut::from_bytes(target).set_stamina(health);
    Ok((
        AttackResult {
            icon: resolved.sound,
            feedback,
            damage,
            killed: health < 1,
            sounds: vec![],
        },
        total_delta,
    ))
}
