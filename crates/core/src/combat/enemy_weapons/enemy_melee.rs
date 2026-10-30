//! A monster's armed melee against another monster, including reflected natural
//! effects.
use crate::combat::ally_turns::{MonsterMelee, Struck};
use crate::combat::attack::melee_accuracy;
use crate::combat::natural_conditions;
use crate::{
    RandomState, RuleError,
    combat::{
        Combat, attack,
        damage::{DamageFlags, DamagePacket, resolve_damage},
    },
};
use attack::AttackResult;
use centurions_content::ITEM_BYTES;
use centurions_content::combat::MONSTER_BYTES;
use centurions_content::records::{
    Condition, DamageTargetKind, EnemyRecord, EnemyRecordMut, ItemDefinition, SpellInfoRecordMut,
};
/// Resolves a monster's armed blow at another monster, committed only if the whole blow succeeds.
/// Returns the result, the packet, and whether the blow was reflected.
pub(super) fn resolve(
    combat: &mut Combat,
    random: &mut RandomState,
    bodies: (i16, i16, usize, i16),
    weapon: &[u8],
    difficulty: i16,
    packet: Vec<u8>,
) -> Result<(AttackResult, Vec<u8>, bool), RuleError> {
    let (attacker, target, index, actor) = bodies;
    let body_index = |body: i16| {
        usize::try_from(i32::from(body) - 10)
            .ok()
            .filter(|index| *index < combat.roster.records.len())
            .ok_or_else(|| RuleError::Invalid("enemy melee body outside records".into()))
    };
    let (attacker_index, target_index, actor) = (
        body_index(attacker)?,
        body_index(target)?,
        usize::try_from(actor - 10)
            .ok()
            .filter(|index| *index < 100)
            .ok_or("invalid retained melee actor")?,
    );
    if attacker_index == target_index
        || index >= 6
        || weapon.len() != ITEM_BYTES
        || combat.roster.mon_pos.len() <= attacker_index.max(target_index)
    {
        return Err(RuleError::Invalid("enemy melee body or definition".into()));
    }
    let source = &combat.roster.records[attacker_index].bytes;
    let victim = &combat.roster.records[target_index].bytes;
    if source.len() != MONSTER_BYTES || victim.len() != MONSTER_BYTES {
        return Err(RuleError::Invalid("enemy melee record size".into()));
    }
    if ItemDefinition(weapon).item_type()? != 2 {
        return Err(RuleError::Unsupported {
            action: "combat.enemy_melee",
            dependency: "nonphysical melee weapon definition",
        });
    }
    let mut next = combat.clone();
    let mut rng = random.clone();
    let (source, victim) = (source.clone(), victim.clone());
    if EnemyRecord::from_bytes(&source).weapon()? != 0 {
        let id = EnemyRecord::from_bytes(&source).melee_weapon()?;
        next.story_record_mut(actor)?.record_mut().set_weapon(id);
    }
    let positions = (
        next.roster.mon_pos[attacker_index].0,
        next.roster.mon_pos[target_index].0,
    );
    let mut records = [
        next.roster.records[attacker_index].bytes.clone(),
        next.roster.records[target_index].bytes.clone(),
    ];
    let outcome = weapon_core(
        (&source, &victim),
        &mut records,
        positions,
        (index, weapon),
        difficulty,
        packet,
        &mut rng,
    )?;
    let [attacker_bytes, target_bytes] = records;
    next.roster.records[attacker_index].bytes = attacker_bytes;
    next.roster.records[target_index].bytes = target_bytes;
    *combat = next;
    *random = rng;
    Ok((outcome.result, outcome.packet, outcome.reflected))
}

/// One monster's armed blow on another, or its natural fallback; see
/// `natural_core`.
///
/// Original: attack2
pub(crate) fn weapon_core(
    (source, victim): (&[u8], &[u8]),
    records: &mut [Vec<u8>; 2],
    (attacker_x, target_x): (i16, i16),
    (index, weapon): (usize, &[u8]),
    difficulty: i16,
    mut packet: Vec<u8>,
    rng: &mut RandomState,
) -> Result<MonsterMelee, RuleError> {
    let loaded = EnemyRecord::from_bytes(source).weapon()? != 0;
    let bonus = EnemyRecord::from_bytes(source)
        .damage_plus()?
        .wrapping_add(EnemyRecord::from_bytes(source).condition(Condition::BonusDamage));
    let condition = EnemyRecord::from_bytes(source)
        .natural_attack(index)
        .condition;
    let [attacker_bytes, target_bytes] = records;
    let weapon_condition = if loaded {
        natural_conditions::admit_weapon_condition(
            weapon,
            target_bytes,
            DamageTargetKind::Enemy,
            false,
            rng,
        )?
    } else {
        None
    };
    EnemyRecordMut::from_bytes(attacker_bytes).face_towards(attacker_x, target_x);
    let hit = melee_accuracy::enemy_against_enemy(
        &EnemyRecord::from_bytes(source),
        &EnemyRecord::from_bytes(victim),
        if loaded {
            i16::from(ItemDefinition(weapon).base_accuracy())
        } else {
            0
        },
    )?;
    let hit = rng.bound(100)
        <= hit.clamp(
            15_i16.wrapping_sub(difficulty.wrapping_mul(5)),
            90_i16.wrapping_sub(difficulty.wrapping_mul(5)),
        );
    let immunity = EnemyRecord::from_bytes(victim).weapon_immunity()?;
    let rejected = immunity == -1 && loaded && ItemDefinition(weapon).immunity_class()? != -1
        || immunity == -2 && loaded && ItemDefinition(weapon).immunity_class()? != -2
        || !matches!(immunity, -2..=0)
            && (if loaded {
                ItemDefinition(weapon).description_id()?
            } else {
                0
            })
            .wrapping_sub(1024)
                != immunity;
    if hit && rejected {
        let code = match immunity {
            -1 => -50,
            -2 => -51,
            _ => -45,
        };
        let feedback = Some(attack::AttackFeedback {
            code,
            amount: 0,
            recipient: attack::AttackRecipient::Attacker,
        });
        if loaded && matches!(immunity, -1 | -2) {
            EnemyRecordMut::from_bytes(attacker_bytes).set_weapon(0);
        }
        return Ok(MonsterMelee {
            result: AttackResult {
                icon: 0,
                feedback,
                damage: 0,
                killed: false,
                sounds: vec![],
            },
            packet,
            reflected: false,
            hit,
            immune: Some(code),
            victory_delta: 0,
            killed: None,
        });
    }
    let reflected = EnemyRecord::from_bytes(victim).condition(Condition::PhysicalReflection) != 0
        && rng.bound(100) < 34
        && EnemyRecord::from_bytes(source).weapon_immunity()? == 0;
    let (target, struck) = if reflected {
        (&mut *attacker_bytes, Struck::Attacker)
    } else {
        (&mut *target_bytes, Struck::Target)
    };
    let mut icon = 0;
    let mut damage = if hit {
        EnemyRecordMut::from_bytes(target).mark_been_attacked();
        SpellInfoRecordMut(&mut packet).set_damage_bounds(
            bonus.wrapping_add(1),
            bonus.wrapping_add(if loaded {
                ItemDefinition(weapon).melee_damage()?
            } else {
                0
            }),
        );
        for element in 0..7 {
            SpellInfoRecordMut(&mut packet).set_elemental_damage(
                element,
                if loaded {
                    ItemDefinition(weapon).elemental_damage(element)?
                } else {
                    0
                },
            );
        }
        if reflected {
            let result = attack::resolve_reflected_attack(
                target,
                &packet,
                index,
                !loaded,
                weapon_condition,
                rng,
            )?;
            let killed = result.killed.then_some(struck);
            return Ok(MonsterMelee {
                result,
                packet,
                reflected: true,
                hit,
                immune: None,
                victory_delta: 0,
                killed,
            });
        }
        resolve_damage(
            target,
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
            rng,
        )
        .map(|resolved| {
            icon = resolved.sound;
            resolved.total
        })?
    } else {
        0
    };
    let mut sounds = vec![];
    let mut clear_target = false;
    let mut feedback = None;
    let mut natural = None;
    if damage != 0 {
        if !loaded {
            if let Some(mut effect) =
                natural_conditions::monster_attack_effect(target, source, index, rng)?
            {
                feedback = effect.feedback.take();
                damage = damage.wrapping_add(effect.extra_damage);
                if effect.applied && condition == 5 {
                    sounds.push(667);
                }
                clear_target = effect.applied && condition == 10;
                natural = Some(effect);
            } else {
                damage = 0;
            }
        } else if let Some(condition) = weapon_condition {
            feedback = Some(natural_conditions::apply_weapon_condition(
                target,
                DamageTargetKind::Enemy,
                condition,
            )?);
        }
        EnemyRecordMut::from_bytes(target).mark_been_attacked();
        let health = EnemyRecord::from_bytes(target)
            .stamina()?
            .wrapping_sub(damage);
        EnemyRecordMut::from_bytes(target).set_stamina(health);
    }
    let killed = EnemyRecord::from_bytes(target).stamina()? < 1;
    let result = AttackResult {
        icon,
        feedback,
        damage,
        killed,
        sounds,
    };
    if let Some(effect) = natural {
        effect.credit_attacker(attacker_bytes)?;
    }
    if clear_target {
        EnemyRecordMut::from_bytes(attacker_bytes).set_target(-1);
    }
    Ok(MonsterMelee {
        result,
        packet,
        reflected,
        hit,
        immune: None,
        victory_delta: 0,
        killed: killed.then_some(struck),
    })
}
