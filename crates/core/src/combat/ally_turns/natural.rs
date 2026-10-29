//! Natural enemy-to-enemy attacks, including controlled allies and reflection.
use super::position;
use crate::{
    RandomState, RuleError,
    combat::{
        Combat,
        attack::{self, AttackResult, melee_accuracy},
        damage::{DamageFlags, DamagePacket, resolve_damage},
        lifecycle,
    },
};
use centurions_content::{
    Warrior,
    combat::MonsterRecord,
    records::{AttackPacketMut, Condition, EnemyRecord, EnemyRecordMut},
};

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
        attack::default_attack_packet(),
        None,
    )?
    .0)
}

#[allow(clippy::too_many_arguments)]
pub fn natural_attack_enemy_with_packet(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    attacker: i16,
    target: i16,
    index: usize,
    attacks: &[u8],
    retained: &[u8],
) -> Result<(AttackResult, Vec<u8>), RuleError> {
    natural_attack_enemy_with_actor(
        combat,
        warriors,
        random,
        (attacker, target, index, attacker),
        attacks,
        retained,
    )
}

pub fn natural_attack_enemy_with_actor(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    bodies: (i16, i16, usize, i16),
    attacks: &[u8],
    retained: &[u8],
) -> Result<(AttackResult, Vec<u8>), RuleError> {
    let (result, packet, _) = natural_attack_enemy_with_target(
        combat,
        warriors,
        random,
        bodies,
        Some(attacks),
        retained,
    )?;
    Ok((result, packet))
}

pub(crate) fn natural_attack_enemy_with_target(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    bodies: (i16, i16, usize, i16),
    attacks: Option<&[u8]>,
    retained: &[u8],
) -> Result<(AttackResult, Vec<u8>, bool), RuleError> {
    natural_attack_enemy_with_processing(combat, warriors, random, bodies, attacks, retained, None)
}

pub(crate) fn natural_attack_enemy_with_processing(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    bodies: (i16, i16, usize, i16),
    attacks: Option<&[u8]>,
    retained: &[u8],
    processing: Option<crate::combat::death_scripts::DeathProcessing>,
) -> Result<(AttackResult, Vec<u8>, bool), RuleError> {
    let (attacker, target, index, retained_actor) = bodies;
    if retained.len() != 58 {
        return Err(RuleError::Invalid("retained attack packet size".into()));
    }
    let base = attacks
        .map(attack::attack_packet)
        .transpose()?
        .unwrap_or_else(attack::default_attack_packet);
    let (mut next_combat, mut next_warriors, mut next_random) =
        (combat.clone(), warriors.to_vec(), random.clone());
    let result = natural_attack_enemy_inner(
        &mut next_combat,
        &mut next_warriors,
        &mut next_random,
        attacker,
        target,
        index,
        retained_actor,
        base,
        processing,
    )?;
    *combat = next_combat;
    warriors.clone_from_slice(&next_warriors);
    *random = next_random;
    Ok(result)
}

#[allow(clippy::too_many_arguments)]
fn natural_attack_enemy_inner(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    attacker: i16,
    target: i16,
    index: usize,
    retained_actor: i16,
    packet: Vec<u8>,
    processing: Option<crate::combat::death_scripts::DeathProcessing>,
) -> Result<(AttackResult, Vec<u8>, bool), RuleError> {
    if attacker < 10 || target < 10 || attacker == target || index >= 6 {
        return Err(RuleError::Invalid("invalid monster attack".into()));
    }
    let attacker_index = (attacker - 10) as usize;
    let target_index = (target - 10) as usize;
    let source = combat
        .enemies
        .records
        .get(attacker_index)
        .ok_or_else(|| RuleError::Invalid("attacker missing".into()))?
        .bytes
        .clone();
    let victim = &combat
        .enemies
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
    if source_record.selected_item()? != 0 {
        let retained_actor = usize::try_from(retained_actor - 10)
            .map_err(|_| RuleError::Invalid("invalid retained enemy actor".into()))?;
        let record = combat.story_record_mut(retained_actor)?;
        record.record_mut().set_selected_item(0);
    }
    let origin = position(combat, attacker);
    let destination = position(combat, target);
    let mut records = [
        combat.enemies.records[attacker_index].bytes.clone(),
        combat.enemies.records[target_index].bytes.clone(),
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
    combat.enemies.records[attacker_index].bytes = attacker_bytes;
    combat.enemies.records[target_index].bytes = target_bytes;
    combat.victory_work.add_total(outcome.victory_delta);
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

/// The record attack2's NPC-target branch strikes: its target, or the attacker
/// after the reflection redirect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Struck {
    Attacker,
    Target,
}

/// attack2's NPC-against-NPC branch over two NPC records.
pub(crate) struct NpcMelee {
    pub result: AttackResult,
    pub packet: Vec<u8>,
    pub reflected: bool,
    /// Rand(100) admitted the hit, before the immunity test.
    pub hit: bool,
    /// The target's weapon immunity refused the hit (feedback code); attack2 returns.
    pub immune: Option<i16>,
    /// The WORD4435c4 change the caller applies before killbody.
    pub victory_delta: i16,
    /// The live caller's killbody, after the victory change.
    pub killed: Option<Struck>,
}

impl NpcMelee {
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

/// `source` and `victim` are the records as attack2 found them; `records` holds the
/// attacker and target records it writes. Facing, accuracy, immunity, reflection,
/// damage and the NPC condition switch keep the live order and draws.
pub(crate) fn natural_core(
    (source, victim): (&[u8], &[u8]),
    records: &mut [Vec<u8>; 2],
    (origin_x, target_x): (i16, i16),
    index: usize,
    difficulty: i16,
    mut packet: Vec<u8>,
    random: &mut RandomState,
) -> Result<NpcMelee, RuleError> {
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
        return Ok(NpcMelee::missed(packet, false, true, Some(-45)));
    }
    let reflected = victim_record.condition(Condition::PhysicalReflection) != 0
        && random.bound(100) < 34
        && source_record.weapon_immunity()? == 0;
    if !hit {
        return Ok(NpcMelee::missed(packet, reflected, false, None));
    }
    let bonus = source_record
        .melee_bonus()?
        .wrapping_add(source_record.condition(Condition::BonusDamage));
    AttackPacketMut(&mut packet).set_damage_bounds(
        bonus.wrapping_add(natural.minimum),
        bonus.wrapping_add(natural.maximum),
    );
    AttackPacketMut(&mut packet).set_impact_sound(if natural.impact_sound == 0 {
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
        return Ok(NpcMelee {
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
            return Ok(NpcMelee::missed(packet, false, true, None));
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
                    EnemyRecordMut::from_bytes(attacker_bytes).set_target_body(-1);
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
        EnemyRecordMut::from_bytes(target_bytes).mark_damaged();
    }
    Ok(NpcMelee {
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

/// Select result audio from the retained attack index and loaded impact word.
pub fn result_audio(
    attacker: &MonsterRecord,
    index: usize,
    damage: i16,
    loaded_impact: i16,
) -> Result<i16, RuleError> {
    if attacker.bytes.len() != 288 || index >= 6 {
        return Err(RuleError::Invalid(
            "monster result audio record or index".into(),
        ));
    }
    if damage == 0 {
        return Ok(650);
    }
    if attacker.record().selected_item()? != 0 {
        return Ok(loaded_impact);
    }
    Ok(attacker.record().natural_attack(index).impact_sound)
}
