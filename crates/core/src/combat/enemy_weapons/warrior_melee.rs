//! A monster's armed melee against a warrior, with its natural fallback and the
//! effects that stand when it stops partway.
use crate::combat::attack::melee_accuracy;
use crate::{
    RandomState, RuleError,
    combat::{
        attack,
        damage::{DamageFlags, DamagePacket, resolve_damage},
        natural_conditions, party_buffs,
    },
};
use attack::AttackResult;
use centurions_content::combat::MONSTER_BYTES;
use centurions_content::records::{
    Condition, DamageTargetKind, EnemyRecord, EnemyRecordMut, SpellInfoRecordMut, WarriorRecord,
    WarriorRecordMut,
};
use centurions_content::{ITEM_BYTES, WARRIOR_BYTES};
use centurions_content::{Warrior, combat::MonsterRecord, records::ItemDefinition};
/// Resolves a monster's blow at a warrior, committed only if the whole blow succeeds: the hit roll,
/// then damage through the warrior's armor and saves and the party's buffs. A monster with no
/// weapon loaded uses its natural attack's damage and effect. Returns the result, the packet, and
/// whether the blow was reflected.
#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve(
    warrior: &mut Warrior,
    monster: &mut MonsterRecord,
    weapon: &[u8],
    random: &mut RandomState,
    difficulty: i16,
    attacker_x: i16,
    target_x: i16,
    index: usize,
    mut packet: Vec<u8>,
    buffs: &[u8],
) -> Result<(AttackResult, Vec<u8>, bool), RuleError> {
    crate::combat::party_buffs::validate(buffs)?;
    if warrior.bytes.len() != WARRIOR_BYTES
        || monster.bytes.len() != MONSTER_BYTES
        || weapon.len() != ITEM_BYTES
        || index >= 35
    {
        return Err(RuleError::Invalid("enemy melee record size".into()));
    }
    let id = monster.record().melee_weapon()?;
    let loaded = monster.record().weapon()? != 0;
    if ItemDefinition(weapon).item_type()? != 2 {
        return Err(RuleError::Unsupported {
            action: "combat.enemy_melee",
            dependency: "other enemy melee weapon or reflection/unconscious target",
        });
    }
    let mut victim = warrior.clone();
    let mut attacker = monster.clone();
    let mut next = random.clone();
    let defender_bytes = &mut victim.bytes;
    let attacker_bytes = &mut attacker.bytes;
    if loaded {
        EnemyRecordMut::from_bytes(attacker_bytes).set_weapon(id);
    }
    let weapon_condition = if loaded {
        natural_conditions::admit_weapon_condition(
            weapon,
            defender_bytes,
            DamageTargetKind::Warrior,
            party_buffs::mind_shield(buffs)?,
            &mut next,
        )?
    } else {
        None
    };
    EnemyRecordMut::from_bytes(attacker_bytes).face_towards(attacker_x, target_x);
    let hit = melee_accuracy::enemy_against_warrior(
        &EnemyRecord::from_bytes(attacker_bytes),
        &WarriorRecord::from_bytes(defender_bytes),
        if loaded {
            i16::from(ItemDefinition(weapon).base_accuracy())
        } else {
            0
        },
    )?;
    let minimum = 15_i16.wrapping_sub(difficulty.wrapping_mul(5));
    let maximum = 90_i16.wrapping_sub(difficulty.wrapping_mul(5));
    let hit = next.bound(100) <= hit.clamp(minimum, maximum);
    let reflected =
        WarriorRecord::from_bytes(defender_bytes).condition(Condition::PhysicalReflection)? != 0
            && next.bound(100) < 34
            && EnemyRecord::from_bytes(attacker_bytes).weapon_immunity()? == 0;
    if !hit {
        if !reflected {
            WarriorRecordMut::from_bytes(defender_bytes).count_avoided_attack();
        }
        *warrior = victim;
        *monster = attacker;
        *random = next;
        return Ok((
            AttackResult {
                icon: 0,
                feedback: None,
                damage: 0,
                killed: false,
                sounds: vec![],
            },
            packet,
            reflected,
        ));
    }
    let bonus = EnemyRecord::from_bytes(attacker_bytes)
        .damage_plus()?
        .wrapping_add(EnemyRecord::from_bytes(attacker_bytes).condition(Condition::BonusDamage));
    if id == 0 {
        let natural = EnemyRecord::from_bytes(attacker_bytes).natural_attack(index);
        SpellInfoRecordMut(&mut packet).set_damage_bounds(
            bonus.wrapping_add(natural.minimum),
            bonus.wrapping_add(natural.maximum),
        );
        SpellInfoRecordMut(&mut packet).set_impact_sound(if natural.impact_sound == 0 {
            100
        } else {
            natural.impact_sound
        });
    } else {
        SpellInfoRecordMut(&mut packet).set_damage_bounds(
            bonus.wrapping_add(1),
            bonus.wrapping_add(if loaded {
                ItemDefinition(weapon).melee_damage()?
            } else {
                0
            }),
        );
        for element in 0..7 {
            let amount = if loaded {
                ItemDefinition(weapon).elemental_damage(element)?
            } else {
                0
            };
            if amount != 0 {
                SpellInfoRecordMut(&mut packet).set_elemental_damage(element, amount);
            }
        }
    }
    if reflected {
        let result = attack::resolve_reflected_attack(
            attacker_bytes,
            &packet,
            index,
            !loaded,
            weapon_condition,
            &mut next,
        )?;
        *warrior = victim;
        *monster = attacker;
        *random = next;
        return Ok((result, packet, true));
    }
    let result = resolve_damage(
        defender_bytes,
        false,
        &DamagePacket {
            bytes: packet.clone(),
            attack_count: 0,
        },
        DamageFlags {
            armor: true,
            saves: true,
            ..Default::default()
        },
        &mut next,
    )?;
    let mut sounds = vec![];
    let mut damage = result.total;
    let mut feedback = None;
    if damage == 0 {
        WarriorRecordMut::from_bytes(defender_bytes).count_avoided_attack();
    } else {
        let natural = if !loaded {
            Some(crate::combat::natural_conditions::warrior_attack_effect(
                defender_bytes,
                attacker_bytes,
                index,
                party_buffs::mind_shield(buffs)?,
                &mut next,
            )?)
        } else {
            None
        };
        if natural.as_ref().is_some_and(|effect| effect.applied)
            && EnemyRecord::from_bytes(attacker_bytes)
                .natural_attack(index)
                .condition
                == 5
        {
            sounds.push(667);
        }
        if let Some(adjusted) = crate::combat::party_buffs::attack2_damage(buffs, 0, damage)? {
            damage = adjusted;
            sounds.push(-694);
        }
        if let Some(condition) = weapon_condition {
            feedback = Some(natural_conditions::apply_weapon_condition(
                defender_bytes,
                DamageTargetKind::Warrior,
                condition,
            )?);
        }
        if let Some(natural) = natural {
            natural.credit_attacker(attacker_bytes)?;
            feedback = natural.feedback;
            damage = damage.wrapping_add(natural.extra_damage);
        }
        WarriorRecordMut::from_bytes(defender_bytes).add_damage_received(damage);
        WarriorRecordMut::from_bytes(defender_bytes).count_hit_received();
        let hp = WarriorRecord::from_bytes(defender_bytes)
            .stamina()?
            .wrapping_sub(damage);
        WarriorRecordMut::from_bytes(defender_bytes).set_stamina(hp);
        if hp < 1 && hp > -10 {
            WarriorRecordMut::from_bytes(defender_bytes).set_bleeding(1);
        }
        WarriorRecordMut::from_bytes(defender_bytes).mark_been_attacked();
    }
    let result = AttackResult {
        icon: result.sound,
        feedback,
        damage,
        killed: WarriorRecord::from_bytes(defender_bytes).stamina()? < 1,
        sounds,
    };
    *warrior = victim;
    *monster = attacker;
    *random = next;
    Ok((result, packet, false))
}
