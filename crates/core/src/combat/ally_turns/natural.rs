//! Monsters attacking monsters with natural weapons, including controlled allies and
//! reflection.
use crate::combat::death_scripts::DeathProcessing;
use crate::{
    RandomState, RuleError,
    combat::{
        Combat,
        attack::{self, AttackResult, melee_accuracy},
        damage::{DamageFlags, DamagePacket, resolve_damage},
        lifecycle,
    },
};
use centurions_content::combat::MONSTER_BYTES;
use centurions_content::{
    Warrior,
    combat::MonsterRecord,
    records::{Condition, EnemyRecord, EnemyRecordMut, SpellInfoRecordMut},
};

/// One monster's natural attack on another, with the default attack packet and the attacker as the
/// acting body.
pub fn natural_attack_enemy(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    attacker: i16,
    target: i16,
    index: usize,
) -> Result<AttackResult, RuleError> {
    Ok(natural_attack_enemy_inner(
        combat,
        warriors,
        random,
        attacker,
        target,
        index,
        attacker,
        attack::default_spell_info(),
        None,
    )?
    .0)
}

/// A monster's natural attack on another using the supplied attack table and attack packet; the
/// attacker is the acting body.
#[allow(clippy::too_many_arguments)]
pub fn natural_attack_enemy_with_packet(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    attacker: i16,
    target: i16,
    index: usize,
    attacks: &[u8],
    info: &[u8],
) -> Result<(AttackResult, Vec<u8>), RuleError> {
    natural_attack_enemy_with_actor(
        combat,
        warriors,
        random,
        (attacker, target, index, attacker),
        attacks,
        info,
    )
}

/// A monster's natural attack on another where the body whose turn it is may differ from the
/// attacker; returns the result and the packet.
pub fn natural_attack_enemy_with_actor(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    bodies: (i16, i16, usize, i16),
    attacks: &[u8],
    info: &[u8],
) -> Result<(AttackResult, Vec<u8>), RuleError> {
    let (result, packet, _) =
        natural_attack_enemy_with_target(combat, warriors, random, bodies, Some(attacks), info)?;
    Ok((result, packet))
}

/// As the actor form, also telling the caller whether the blow was reflected back.
pub(crate) fn natural_attack_enemy_with_target(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    bodies: (i16, i16, usize, i16),
    attacks: Option<&[u8]>,
    info: &[u8],
) -> Result<(AttackResult, Vec<u8>, bool), RuleError> {
    natural_attack_enemy_with_processing(combat, warriors, random, bodies, attacks, info, None)
}

/// A monster's natural attack on another, committed only if the whole attack succeeds. The acting
/// monster's selected weapon is cleared first, and a kill goes through the given death processing.
pub(crate) fn natural_attack_enemy_with_processing(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    bodies: (i16, i16, usize, i16),
    attacks: Option<&[u8]>,
    info: &[u8],
    processing: Option<DeathProcessing>,
) -> Result<(AttackResult, Vec<u8>, bool), RuleError> {
    let (attacker, target, index, turn_actor) = bodies;
    if info.len() != 58 {
        return Err(RuleError::Invalid("retained attack packet size".into()));
    }
    let base = attacks
        .map(attack::spell_info)
        .transpose()?
        .unwrap_or_else(attack::default_spell_info);
    let (mut next_combat, mut next_warriors, mut next_random) =
        (combat.clone(), warriors.to_vec(), random.clone());
    let result = natural_attack_enemy_inner(
        &mut next_combat,
        &mut next_warriors,
        &mut next_random,
        attacker,
        target,
        index,
        turn_actor,
        base,
        processing,
    )?;
    *combat = next_combat;
    warriors.clone_from_slice(&next_warriors);
    *random = next_random;
    Ok(result)
}

/// Runs one monster-on-monster natural attack against the live battle: puts the attacker's and
/// target's records through the attack, adjusts the hostile count and kills whichever body fell.
#[allow(clippy::too_many_arguments)]
fn natural_attack_enemy_inner(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    attacker: i16,
    target: i16,
    index: usize,
    turn_actor: i16,
    packet: Vec<u8>,
    processing: Option<DeathProcessing>,
) -> Result<(AttackResult, Vec<u8>, bool), RuleError> {
    if attacker < 10 || target < 10 || attacker == target || index >= 6 {
        return Err(RuleError::Invalid("invalid monster attack".into()));
    }
    let attacker_index = (attacker - 10) as usize;
    let target_index = (target - 10) as usize;
    let source = combat
        .roster
        .records
        .get(attacker_index)
        .ok_or_else(|| RuleError::Invalid("attacker missing".into()))?
        .bytes
        .clone();
    let victim = &combat
        .roster
        .records
        .get(target_index)
        .ok_or_else(|| RuleError::Invalid("target missing".into()))?
        .bytes
        .clone();
    let source_record = EnemyRecord::from_bytes(&source);
    let condition = source_record.natural_attack(index).condition;
    if source_record.melee_weapon()? != 0 || !matches!(condition, 0..=17 | 19) {
        return Err(RuleError::Unsupported {
            action: "combat.monster_attack",
            dependency: "monster weapon, attack condition, reflection or immunity",
        });
    }
    if source_record.weapon()? != 0 {
        let turn_actor = usize::try_from(turn_actor - 10)
            .map_err(|_| RuleError::Invalid("invalid retained enemy actor".into()))?;
        let record = combat.story_record_mut(turn_actor)?;
        record.record_mut().set_weapon(0);
    }
    let origin = combat.body_position(attacker);
    let destination = combat.body_position(target);
    let mut records = [
        combat.roster.records[attacker_index].bytes.clone(),
        combat.roster.records[target_index].bytes.clone(),
    ];
    let outcome = natural_core(
        (&source, victim),
        &mut records,
        (origin.0, destination.0),
        index,
        combat.difficulty,
        packet,
        random,
    )?;
    let [attacker_bytes, target_bytes] = records;
    combat.roster.records[attacker_index].bytes = attacker_bytes;
    combat.roster.records[target_index].bytes = target_bytes;
    combat.kill_counts.add_num_enemy(outcome.victory_delta);
    if let Some(struck) = outcome.killed {
        let body = if struck == Struck::Attacker {
            attacker
        } else {
            target
        };
        lifecycle::kill_body_with_processing(combat, warriors, body, processing)?;
    }
    Ok((outcome.result, outcome.packet, outcome.reflected))
}

/// Who a monster's blow lands on: its target, or itself when reflected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Struck {
    Attacker,
    Target,
}

/// One monster attacks another.
pub(crate) struct MonsterMelee {
    pub result: AttackResult,
    pub packet: Vec<u8>,
    pub reflected: bool,
    /// Rand(100) admitted the hit, before the immunity test.
    pub hit: bool,
    /// The target's weapon immunity refused the hit (feedback code); the attack ends.
    pub immune: Option<i16>,
    /// The change to the hostile count the caller applies before killbody.
    pub victory_delta: i16,
    /// The body killbody takes, after the hostile count changes.
    pub killed: Option<Struck>,
}

impl MonsterMelee {
    /// A blow that dealt no damage.
    fn missed(packet: Vec<u8>, reflected: bool, hit: bool, immune: Option<i16>) -> Self {
        Self {
            result: AttackResult {
                icon: 0,
                feedback: None,
                damage: 0,
                killed: false,
                sounds: vec![],
            },
            packet,
            reflected,
            hit,
            immune,
            victory_delta: 0,
            killed: None,
        }
    }
}

/// One monster's natural attack on another. `source` and `victim` are the records
/// as the attack found them; `records` holds the ones it writes. Facing, accuracy,
/// immunity, reflection, damage and the condition follow in that order.
///
/// Original: attack2
pub(crate) fn natural_core(
    (source, victim): (&[u8], &[u8]),
    records: &mut [Vec<u8>; 2],
    (origin_x, target_x): (i16, i16),
    index: usize,
    difficulty: i16,
    mut packet: Vec<u8>,
    random: &mut RandomState,
) -> Result<MonsterMelee, RuleError> {
    let source_record = EnemyRecord::from_bytes(source);
    let victim_record = EnemyRecord::from_bytes(victim);
    let natural = source_record.natural_attack(index);
    let condition = natural.condition;
    let [attacker_bytes, target_bytes] = records;
    EnemyRecordMut::from_bytes(attacker_bytes).face_towards(origin_x, target_x);
    let hit = melee_accuracy::enemy_against_enemy(&source_record, &victim_record, 0)?;
    let hit = random.bound(100)
        <= hit.clamp(
            15_i16.wrapping_sub(difficulty.wrapping_mul(5)),
            90_i16.wrapping_sub(difficulty.wrapping_mul(5)),
        );
    let immunity = victim_record.weapon_immunity()?;
    if hit && !matches!(immunity, -2..=0 | -1024) {
        return Ok(MonsterMelee::missed(packet, false, true, Some(-45)));
    }
    let reflected = victim_record.condition(Condition::PhysicalReflection) != 0
        && random.bound(100) < 34
        && source_record.weapon_immunity()? == 0;
    if !hit {
        return Ok(MonsterMelee::missed(packet, reflected, false, None));
    }
    let bonus = source_record
        .damage_plus()?
        .wrapping_add(source_record.condition(Condition::BonusDamage));
    SpellInfoRecordMut(&mut packet).set_damage_bounds(
        bonus.wrapping_add(natural.minimum),
        bonus.wrapping_add(natural.maximum),
    );
    SpellInfoRecordMut(&mut packet).set_impact_sound(if natural.impact_sound == 0 {
        100
    } else {
        natural.impact_sound
    });
    if reflected {
        let (result, total_delta) = attack::resolve_reflected_attack_with_victory(
            attacker_bytes,
            &packet,
            index,
            true,
            None,
            random,
        )?;
        let killed = result.killed.then_some(Struck::Attacker);
        return Ok(MonsterMelee {
            result,
            packet,
            reflected: true,
            hit,
            immune: None,
            victory_delta: total_delta,
            killed,
        });
    }
    let resolved = resolve_damage(
        target_bytes,
        true,
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
    let mut sounds = Vec::new();
    let mut extra_damage = 0_i16;
    let mut feedback = None;
    let mut victory_delta = 0;
    if resolved.total != 0 {
        let prior_faction = EnemyRecord::from_bytes(target_bytes).faction()?;
        let Some(effect) = crate::combat::natural_conditions::monster_attack_effect(
            target_bytes,
            source,
            index,
            random,
        )?
        else {
            return Ok(MonsterMelee::missed(packet, false, true, None));
        };
        effect.credit_attacker(attacker_bytes)?;
        extra_damage = effect.extra_damage;
        feedback = effect.feedback;
        if effect.applied {
            match condition {
                9 => sounds.push(104),
                14 => sounds.push(344),
                10 => {
                    let side = EnemyRecord::from_bytes(source).faction()?;
                    if side == 0 {
                        victory_delta = -1;
                    } else if prior_faction == 0 {
                        victory_delta = 1;
                    }
                    EnemyRecordMut::from_bytes(attacker_bytes).set_target(-1);
                }
                _ => {}
            }
        }
    }
    let health = EnemyRecord::from_bytes(target_bytes)
        .stamina()?
        .wrapping_sub(resolved.total)
        .wrapping_sub(extra_damage);
    if resolved.total != 0 {
        EnemyRecordMut::from_bytes(target_bytes).set_stamina(health);
        EnemyRecordMut::from_bytes(target_bytes).mark_been_attacked();
    }
    Ok(MonsterMelee {
        result: AttackResult {
            icon: resolved.sound,
            feedback,
            damage: resolved.total.wrapping_add(extra_damage),
            killed: health < 1,
            sounds,
        },
        packet,
        reflected: false,
        hit,
        immune: None,
        victory_delta,
        killed: (health < 1).then_some(Struck::Target),
    })
}

/// The result sound, from the attack index and the loaded impact sound.
pub fn result_audio(
    attacker: &MonsterRecord,
    index: usize,
    damage: i16,
    loaded_impact: i16,
) -> Result<i16, RuleError> {
    if attacker.bytes.len() != MONSTER_BYTES || index >= 6 {
        return Err(RuleError::Invalid(
            "monster result audio record or index".into(),
        ));
    }
    if damage == 0 {
        return Ok(650);
    }
    if attacker.record().weapon()? != 0 {
        return Ok(loaded_impact);
    }
    Ok(attacker.record().natural_attack(index).impact_sound)
}
