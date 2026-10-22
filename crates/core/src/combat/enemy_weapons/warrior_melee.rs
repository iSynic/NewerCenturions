//! Enemy weapon melee against a warrior; preserves natural fallback and partial effects.
use crate::{
    RandomState, RuleError,
    combat::{
        attack,
        damage::{DamageFlags, DamagePacket, resolve_damage},
        natural_conditions, party_buffs,
    },
};
use attack::AttackResult;
use centurions_content::records::{
    AttackPacketMut, Condition, DamageTargetKind, EnemyRecord, EnemyRecordMut, WarriorRecord,
    WarriorRecordMut,
};
use centurions_content::{Warrior, combat::MonsterRecord, records::ItemDefinition};
#[allow(clippy::too_many_arguments)]
pub(super) fn resolve(
    warrior: &mut Warrior,
    enemy: &mut MonsterRecord,
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
    if warrior.bytes.len() != 3138 || enemy.bytes.len() != 288 || weapon.len() != 150 || index >= 6
    {
        return Err(RuleError::Invalid("enemy melee record size".into()));
    }
    let id = enemy.record().melee_weapon()?;
    let loaded = enemy.record().selected_item()? != 0;
    if !matches!(enemy.record().natural_attack(index).condition, 0..=17 | 19)
        || ItemDefinition(weapon).category()? != 2
    {
        return Err(RuleError::Unsupported {
            action: "combat.enemy_melee",
            dependency: "other enemy melee weapon or reflection/unconscious target",
        });
    }
    let mut victim = warrior.clone();
    let mut attacker = enemy.clone();
    let mut next = random.clone();
    let defender_bytes = &mut victim.bytes;
    let attacker_bytes = &mut attacker.bytes;
    if loaded {
        EnemyRecordMut::from_bytes(attacker_bytes).set_selected_item(id);
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
    let hit = crate::combat::attack::melee_accuracy::enemy_against_warrior(
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
        *enemy = attacker;
        *random = next;
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
    let bonus = EnemyRecord::from_bytes(attacker_bytes)
        .melee_bonus()?
        .wrapping_add(EnemyRecord::from_bytes(attacker_bytes).condition(Condition::BonusDamage));
    AttackPacketMut(&mut packet).set_damage_bounds(
        bonus.wrapping_add(1),
        bonus.wrapping_add(if loaded {
            ItemDefinition(weapon).melee_damage()?
        } else {
            0
        }),
    );
    for element in 0..7 {
        AttackPacketMut(&mut packet).set_elemental_damage(
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
            attacker_bytes,
            &packet,
            index,
            !loaded,
            weapon_condition,
            &mut next,
        )?;
        *warrior = victim;
        *enemy = attacker;
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
            WarriorRecordMut::from_bytes(defender_bytes).set_bleeding_flag(1);
        }
        WarriorRecordMut::from_bytes(defender_bytes).mark_damaged();
    }
    let result = AttackResult {
        feedback,
        damage,
        killed: WarriorRecord::from_bytes(defender_bytes).stamina()? < 1,
        sounds,
    };
    *warrior = victim;
    *enemy = attacker;
    *random = next;
    Ok((result, packet, false))
}
