//! A warrior's melee against a monster, including damage reflected onto the
//! warrior. Hit, reflection, damage, critical and sound are drawn in that order.
use super::AttackResult;
use crate::{
    RandomState, RuleError,
    combat::damage::{DamageFlags, DamagePacket, resolve_damage},
    combat::natural_conditions::{admit_weapon_condition, apply_weapon_condition},
    combat::warrior_attack::PreparedMeleeWeapon,
};
use centurions_content::{
    Warrior,
    combat::MonsterRecord,
    records::{Condition, DamageTargetKind, ItemDefinition},
};

/// A warrior's blow on a monster: a warrior who has used all of his attacks this turn hears the
/// warning sound and strikes no blow; otherwise the blow is rolled to hit, checked against the
/// monster's immunity and weapon-reflection, and its damage, wounds and statistics are applied.
/// Wielding a weapon needs that weapon's record.
#[allow(clippy::too_many_arguments)]
pub(super) fn resolve(
    attacker: &mut Warrior,
    defender: &mut MonsterRecord,
    random: &mut RandomState,
    base: Vec<u8>,
    info: &[u8],
    difficulty: i16,
    weapon: Option<&[u8]>,
    rolls_condition: bool,
) -> Result<(AttackResult, Vec<u8>, bool), RuleError> {
    if attacker.record().melee_weapon()? != 0 && weapon.is_none() {
        return Err(RuleError::Unsupported {
            action: "combat.attack",
            dependency: "equipped weapon, damage reflection or weapon immunity",
        });
    }
    if attacker.record().attacks_used()? >= attacker.record().attack_limit()? {
        let mut result = no_damage(info.to_vec(), false);
        result.0.sounds.push(45);
        return Ok(result);
    }
    let selected = attacker.bytes.clone();
    let attacks = attacker.record().attacks_used()?.wrapping_add(1);
    attacker.record_mut().set_attacks_used(attacks);
    let (result, packet, reflected, _) = resolve_prepared(
        attacker,
        defender,
        random,
        base,
        0,
        difficulty,
        weapon.map(|definition| PreparedMeleeWeapon {
            definition,
            armed: true,
        }),
        rolls_condition,
        &selected,
        0,
    )?;
    Ok((result, packet, reflected))
}

/// A warrior's blow on a monster, once the caller has allowed, counted and loaded
/// it. An immunity refusal has its own result code, apart from a miss.
///
/// Original: attack
#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_prepared(
    attacker: &mut Warrior,
    defender: &mut MonsterRecord,
    random: &mut RandomState,
    base: Vec<u8>,
    attack_count: i16,
    difficulty: i16,
    weapon: Option<PreparedMeleeWeapon<'_>>,
    rolls_condition: bool,
    selected: &[u8],
    bare_hand_bonus: i16,
) -> Result<(AttackResult, Vec<u8>, bool, Option<i16>), RuleError> {
    let mut packet = base;
    // Read before the weapon condition can change the target.
    let target_turn_skip = defender.record().turn_skip_duration();
    let target_stamina = defender.record().stamina()?;
    let weapon_condition = match weapon.filter(|_| rolls_condition) {
        Some(weapon) => admit_weapon_condition(
            weapon.definition,
            &mut defender.bytes,
            DamageTargetKind::Enemy,
            false,
            random,
        )?,
        None => None,
    };
    let chance = super::melee_accuracy::against_enemy(&attacker.bytes, &defender.bytes)?;
    if random.bound(100)
        > chance.clamp(
            15_i16.wrapping_sub(difficulty.wrapping_mul(5)),
            90_i16.wrapping_sub(difficulty.wrapping_mul(5)),
        )
        && defender.record().turn_skip_duration() == 0
    {
        attacker.record_mut().count_miss();
        return Ok(prepared_no_damage(packet, false, None));
    }
    if let Some(code) = weapon_refusal(defender.record().weapon_immunity()?, weapon)? {
        return Ok(prepared_no_damage(packet, false, Some(code)));
    }
    let reflected =
        defender.record().condition(Condition::PhysicalReflection) != 0 && random.bound(100) < 34;
    crate::combat::warrior_attack::prepare_packet_with_context(
        &mut packet,
        &attacker.bytes,
        weapon
            .filter(|weapon| weapon.armed)
            .map(|weapon| weapon.definition),
        selected,
        bare_hand_bonus,
    )?;
    let resolved = resolve_damage(
        if reflected {
            &mut attacker.bytes
        } else {
            &mut defender.bytes
        },
        !reflected,
        &DamagePacket {
            bytes: packet.clone(),
            attack_count,
        },
        DamageFlags {
            armor: true,
            saves: true,
            ..Default::default()
        },
        random,
    )?;
    // The creature-category bonus adds to the hit only; its damage share is cleared
    // before damage is resolved.
    let mut damage = resolved.total;
    if damage <= 0 {
        attacker.record_mut().count_miss();
        return Ok(prepared_no_damage(packet, reflected, None));
    }
    if random.bound(100) <= attacker.record().major_wound_chance()? {
        damage = damage.wrapping_mul(2);
    }
    // The condition rolled against the enemy lands on the warrior when the blow reflects.
    let feedback = weapon_condition
        .map(|condition| {
            if reflected {
                apply_weapon_condition(&mut attacker.bytes, DamageTargetKind::Warrior, condition)
            } else {
                apply_weapon_condition(&mut defender.bytes, DamageTargetKind::Enemy, condition)
            }
        })
        .transpose()?;
    let killed;
    if reflected {
        if attacker.record().turn_skip_duration()? != 0 {
            damage = attacker.record().stamina()?;
        }
        let health = attacker.record().stamina()?.wrapping_sub(damage);
        let mut record = attacker.record_mut();
        record.set_stamina(health);
        record.mark_been_attacked();
        record.add_damage_received(damage);
        if (-9..=0).contains(&health) {
            record.set_bleeding(1);
        }
        killed = health < 1;
    } else {
        if target_turn_skip != 0 {
            damage = target_stamina;
        }
        defender.record_mut().mark_been_attacked();
        let health = defender.record().stamina()?.wrapping_sub(damage);
        defender.record_mut().set_stamina(health);
        let mut record = attacker.record_mut();
        record.add_damage_dealt(damage);
        record.count_hit();
        killed = health <= 0;
        if killed {
            record.count_kill();
        }
    }
    let sound = match weapon.filter(|weapon| weapon.armed) {
        Some(weapon) => ItemDefinition(weapon.definition).impact_sound()?,
        None => random.bound(2).wrapping_add(401),
    };
    Ok((
        AttackResult {
            icon: resolved.sound,
            feedback,
            damage,
            killed,
            sounds: vec![sound],
        },
        packet,
        reflected,
        None,
    ))
}

/// The outcome of a blow that did nothing, with the refusal code if immunity stopped it.
fn prepared_no_damage(
    packet: Vec<u8>,
    reflected: bool,
    refusal: Option<i16>,
) -> (AttackResult, Vec<u8>, bool, Option<i16>) {
    let (result, packet, reflected) = no_damage(packet, reflected);
    (result, packet, reflected, refusal)
}

/// The outcome of a blow that did nothing.
fn no_damage(packet: Vec<u8>, reflected: bool) -> (AttackResult, Vec<u8>, bool) {
    (
        AttackResult {
            icon: 0,
            feedback: None,
            damage: 0,
            killed: false,
            sounds: vec![],
        },
        packet,
        reflected,
    )
}

/// The refusal a monster's immunity gives a weapon: immunity -1 and -2 turn away an armed weapon
/// not of that class (-50, -51); any other immunity turns away every weapon but the one it names
/// (-45); none refuses nothing.
fn weapon_refusal(
    immunity: i16,
    weapon: Option<PreparedMeleeWeapon<'_>>,
) -> Result<Option<i16>, RuleError> {
    Ok(match immunity {
        -1 | -2 => weapon
            .filter(|weapon| weapon.armed)
            .map(|weapon| {
                ItemDefinition(weapon.definition)
                    .immunity_class()
                    .map(|class| class != immunity)
            })
            .transpose()?
            .unwrap_or(false)
            .then_some(if immunity == -1 { -50 } else { -51 }),
        0 => None,
        _ => (weapon
            .map(|weapon| ItemDefinition(weapon.definition).description_id())
            .transpose()?
            .unwrap_or(0)
            .wrapping_sub(1024)
            != immunity)
            .then_some(-45),
    })
}
