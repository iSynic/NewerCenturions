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
    records::{AttackPacketMut, Condition, EnemyRecord},
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
    mut packet: Vec<u8>,
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
    let victim_record = EnemyRecord::from_bytes(victim);
    let natural = source_record.natural_attack(index);
    let condition = natural.condition;
    if source_record.melee_weapon()? != 0
        || !matches!(condition, 0 | 1 | 2 | 4 | 5 | 6 | 9 | 10 | 11 | 14 | 17)
    {
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
    combat.enemies.records[attacker_index]
        .record_mut()
        .face_towards(origin.0, destination.0);
    let hit = melee_accuracy::enemy_against_enemy(&source_record, &victim_record, 0)?;
    let hit = random.bound(100)
        <= hit.clamp(
            15_i16.wrapping_sub(combat.difficulty.wrapping_mul(5)),
            90_i16.wrapping_sub(combat.difficulty.wrapping_mul(5)),
        );
    let immunity = victim_record.weapon_immunity()?;
    if hit && !matches!(immunity, -2..=0 | -1024) {
        return Ok((
            AttackResult {
                feedback: None,
                damage: 0,
                killed: false,
                sounds: vec![],
            },
            packet,
            false,
        ));
    }
    let reflected = victim_record.condition(Condition::PhysicalReflection) != 0
        && random.bound(100) < 34
        && source_record.weapon_immunity()? == 0;
    if !hit {
        return Ok((
            AttackResult {
                feedback: None,
                damage: 0,
                killed: false,
                sounds: vec![],
            },
            packet,
            reflected,
        ));
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
        let result = attack::resolve_reflected_attack(
            &mut combat.enemies.records[attacker_index].bytes,
            &packet,
            index,
            true,
            random,
        )?;
        if result.killed {
            lifecycle::kill_body(combat, warriors, attacker)?;
        }
        return Ok((result, packet, true));
    }
    let resolved = resolve_damage(
        &mut combat.enemies.records[target_index].bytes,
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
    if resolved.total != 0 {
        let Some(effect) = crate::combat::natural_conditions::monster_attack_effect(
            &mut combat.enemies.records[target_index].bytes,
            &source,
            index,
            random,
        )?
        else {
            return Ok((
                AttackResult {
                    feedback: None,
                    damage: 0,
                    killed: false,
                    sounds: vec![],
                },
                packet,
                false,
            ));
        };
        extra_damage = effect.extra_damage;
        feedback = effect.feedback;
        if effect.applied {
            match condition {
                9 => sounds.push(104),
                14 => sounds.push(344),
                10 => combat.enemies.records[attacker_index]
                    .record_mut()
                    .set_target_body(-1),
                _ => {}
            }
        }
    }
    let health = combat.enemies.records[target_index]
        .record()
        .stamina()?
        .wrapping_sub(resolved.total)
        .wrapping_sub(extra_damage);
    if resolved.total != 0 {
        combat.enemies.records[target_index]
            .record_mut()
            .set_stamina(health);
        combat.enemies.records[target_index]
            .record_mut()
            .mark_damaged();
    }
    if health < 1 {
        lifecycle::kill_body(combat, warriors, target)?;
    }
    Ok((
        AttackResult {
            feedback,
            damage: resolved.total.wrapping_add(extra_damage),
            killed: health < 1,
            sounds,
        },
        packet,
        false,
    ))
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
