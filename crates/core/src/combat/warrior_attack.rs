use crate::combat::attack;
use crate::combat::damage;
use crate::{RandomState, RuleError};
use attack::AttackResult;
use centurions_content::Warrior;
use centurions_content::records::{Condition, ItemDefinition, WarriorRecord};
use damage::{DamageFlags, DamagePacket, resolve_damage};

/// Warrior-to-warrior branch of attack 0x100af0, including the unconditional reflection roll.
pub(crate) fn attack(
    warriors: &mut [Warrior],
    actor: usize,
    target: usize,
    attacks: &[u8],
    items: &[Vec<u8>],
    difficulty: i16,
    random: &mut RandomState,
) -> Result<(AttackResult, Vec<u8>, usize), RuleError> {
    let attacker = warriors
        .get(actor)
        .ok_or("melee actor outside party")?
        .clone();
    warriors.get(target).ok_or("melee target outside party")?;
    let weapon = weapon(&attacker.bytes, items)?;
    let mut packet = attack::attack_packet(attacks)?;
    if attacker.record().attacks_used()? >= attacker.record().attack_limit()? {
        return Ok((
            AttackResult {
                feedback: None,
                damage: 0,
                killed: false,
                sounds: vec![45],
            },
            packet,
            target,
        ));
    }
    warriors[actor]
        .record_mut()
        .set_attacks_used(attacker.record().attacks_used()?.wrapping_add(1));
    let defender = &warriors[target].bytes;
    let hit = attack::melee_accuracy::against_warrior(&attacker.bytes, defender)?;
    let missed = random.bound(100)
        > crate::rules::pin(hit, 15 - difficulty * 5, 90 - difficulty * 5)
        && WarriorRecord::from_bytes(defender).turn_skip_duration()? == 0;
    if missed {
        warriors[actor].record_mut().count_miss();
        return Ok((
            AttackResult {
                feedback: None,
                damage: 0,
                killed: false,
                sounds: vec![],
            },
            packet,
            target,
        ));
    }
    let actual = if random.bound(100) < 34
        && WarriorRecord::from_bytes(defender).condition(Condition::PhysicalReflection)? != 0
    {
        actor
    } else {
        target
    };
    prepare_packet(&mut packet, &attacker.bytes, weapon, &attacker.bytes)?;
    let victim = &mut warriors[actual];
    let result = resolve_damage(
        &mut victim.bytes,
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
        random,
    )?;
    let mut damage = result.total;
    if damage <= 0 {
        warriors[actor].record_mut().count_miss();
        return Ok((
            AttackResult {
                feedback: None,
                damage: 0,
                killed: false,
                sounds: vec![],
            },
            packet,
            actual,
        ));
    }
    if random.bound(100) <= attacker.record().major_wound_chance()? {
        damage = damage.wrapping_mul(2);
    }
    if victim.record().turn_skip_duration()? != 0 {
        damage = victim.record().stamina()?;
    }
    victim.record_mut().mark_damaged();
    let health = victim.record().stamina()?.wrapping_sub(damage);
    victim.record_mut().set_stamina(health);
    victim.record_mut().add_damage_received(damage);
    if health < 1 && health > -10 {
        victim.record_mut().set_bleeding_flag(1);
    }
    Ok((
        AttackResult {
            feedback: None,
            damage,
            killed: health < 1,
            sounds: vec![match weapon {
                Some(weapon) => ItemDefinition(weapon).impact_sound()?,
                None => random.bound(2).wrapping_add(401),
            }],
        },
        packet,
        actual,
    ))
}

pub(crate) fn weapon<'a>(
    warrior: &[u8],
    items: &'a [Vec<u8>],
) -> Result<Option<&'a [u8]>, RuleError> {
    let id = WarriorRecord::from_bytes(warrior).melee_weapon()?;
    if id == 0 {
        return Ok(None);
    }
    let definition = items
        .get(id.unsigned_abs() as usize)
        .ok_or("melee weapon definition missing")?;
    if ItemDefinition(definition).category()?.wrapping_abs() != 2
        || ItemDefinition(definition).effect_selector()? == -10
        || ItemDefinition(definition).spell()? > 1100
    {
        return Err(RuleError::Unsupported {
            action: "combat.attack",
            dependency: "spell-triggering melee weapon",
        });
    }
    Ok(Some(definition))
}

pub(crate) fn prepare_packet(
    packet: &mut [u8],
    warrior: &[u8],
    weapon: Option<&[u8]>,
    active: &[u8],
) -> Result<(), RuleError> {
    use centurions_content::records::{
        Condition, ItemDefinition, WarriorRecord, WeaponAttackParameters,
    };
    let warrior = WarriorRecord::from_bytes(warrior);
    let mut attack = WeaponAttackParameters::read(packet)?;
    let bonus = warrior.condition(Condition::BonusDamage)?;
    attack.physical.0 = bonus.wrapping_add(1);
    let maximum = bonus.wrapping_add(warrior.bare_hand_damage()?);
    attack.physical.1 = maximum;
    if let Some(weapon) = weapon {
        let weapon = ItemDefinition(weapon);
        // attack reads the active warrior's second inventory slot, independently
        // of the selected weapon slot. It stores the factor as Single before
        // widening the multiply; do not replace it with a direct f64 ratio.
        let enhancement = WarriorRecord::from_bytes(active)
            .inventory_slot(1)
            .damage_enhancement();
        let factor = f64::from((100. + f32::from(enhancement)) / 100.);
        attack.physical.1 = maximum
            .wrapping_add((f64::from(weapon.melee_damage()?) * factor).trunc() as i32 as i16);
        for (element, maximum) in attack.elemental_maximum.iter_mut().enumerate() {
            let damage = weapon.elemental_damage(element)?;
            if damage != 0 {
                *maximum = (f64::from(damage) * factor).trunc() as i32 as u8;
            }
        }
    }
    attack.write(packet);
    Ok(())
}
