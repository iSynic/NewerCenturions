use crate::combat::damage;
use crate::combat::natural_conditions::{admit_weapon_condition, apply_weapon_condition};
use crate::combat::{Combat, attack};
use crate::{RandomState, RuleError};
use attack::AttackResult;
use centurions_content::Warrior;
use centurions_content::records::{
    Condition, DamageTargetKind, DevelopmentField, ItemDefinition, WarriorRecord,
};
use damage::{DamageFlags, DamagePacket, resolve_damage};

/// The loaded weapon and the attacker's melee slot. A bare-handed attack uses
/// definition zero for immunity.
#[derive(Clone, Copy)]
pub(crate) struct PreparedMeleeWeapon<'a> {
    pub definition: &'a [u8],
    pub armed: bool,
}

/// A warrior attacks another warrior. Reflection is always rolled. The weapon's
/// condition is rolled against the target before the hit roll and lands on
/// whichever warrior takes the damage.
///
/// Original: attack
#[allow(clippy::too_many_arguments)]
pub(crate) fn attack(
    warriors: &mut [Warrior],
    actor: usize,
    target: usize,
    base: Vec<u8>,
    rolls_condition: bool,
    mind_shield: bool,
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
    if attacker.record().attacks_used()? >= attacker.record().attack_limit()? {
        return Ok((
            AttackResult {
                icon: 0,
                feedback: None,
                damage: 0,
                killed: false,
                sounds: vec![45],
            },
            base,
            target,
        ));
    }
    warriors[actor]
        .record_mut()
        .set_attacks_used(attacker.record().attacks_used()?.wrapping_add(1));
    attack_prepared(
        warriors,
        actor,
        target,
        base,
        0,
        rolls_condition,
        mind_shield,
        weapon.map(|definition| PreparedMeleeWeapon {
            definition,
            armed: true,
        }),
        &attacker.bytes,
        0,
        difficulty,
        random,
    )
}

/// A warrior-against-warrior blow once the caller has allowed and counted it.
/// `selected` supplies the inventory slot's enhancement.
///
/// Original: attack
#[allow(clippy::too_many_arguments)]
pub(crate) fn attack_prepared(
    warriors: &mut [Warrior],
    actor: usize,
    target: usize,
    base: Vec<u8>,
    attack_count: i16,
    rolls_condition: bool,
    mind_shield: bool,
    weapon: Option<PreparedMeleeWeapon<'_>>,
    selected: &[u8],
    bare_hand_bonus: i16,
    difficulty: i16,
    random: &mut RandomState,
) -> Result<(AttackResult, Vec<u8>, usize), RuleError> {
    let attacker = warriors
        .get(actor)
        .ok_or("melee actor outside party")?
        .clone();
    warriors.get(target).ok_or("melee target outside party")?;
    let mut packet = base;
    let condition = match weapon.filter(|_| rolls_condition) {
        Some(weapon) => admit_weapon_condition(
            weapon.definition,
            &mut warriors[target].bytes,
            DamageTargetKind::Warrior,
            mind_shield,
            random,
        )?,
        None => None,
    };
    let defender = &warriors[target].bytes;
    let hit = attack::melee_accuracy::against_warrior(&attacker.bytes, defender)?;
    let missed = random.bound(100)
        > crate::rules::pin(hit, 15 - difficulty * 5, 90 - difficulty * 5)
        && WarriorRecord::from_bytes(defender).turn_skip_duration()? == 0;
    if missed {
        warriors[actor].record_mut().count_miss();
        return Ok((
            AttackResult {
                icon: 0,
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
    prepare_packet_with_context(
        &mut packet,
        &attacker.bytes,
        weapon
            .filter(|weapon| weapon.armed)
            .map(|weapon| weapon.definition),
        selected,
        bare_hand_bonus,
    )?;
    let victim = &mut warriors[actual];
    let result = resolve_damage(
        &mut victim.bytes,
        false,
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
    let mut damage = result.total;
    if damage <= 0 {
        warriors[actor].record_mut().count_miss();
        return Ok((
            AttackResult {
                icon: 0,
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
    let feedback = condition
        .map(|condition| {
            apply_weapon_condition(&mut victim.bytes, DamageTargetKind::Warrior, condition)
        })
        .transpose()?;
    if victim.record().turn_skip_duration()? != 0 {
        damage = victim.record().stamina()?;
    }
    victim.record_mut().mark_been_attacked();
    let health = victim.record().stamina()?.wrapping_sub(damage);
    victim.record_mut().set_stamina(health);
    victim.record_mut().add_damage_received(damage);
    if health < 1 && health > -10 {
        victim.record_mut().set_bleeding(1);
    }
    Ok((
        AttackResult {
            icon: result.sound,
            feedback,
            damage,
            killed: health < 1,
            sounds: vec![match weapon.filter(|weapon| weapon.armed) {
                Some(weapon) => ItemDefinition(weapon.definition).impact_sound()?,
                None => random.bound(2).wrapping_add(401),
            }],
        },
        packet,
        actual,
    ))
}

/// How a warrior's melee attack ended.
pub enum WarriorMelee {
    /// `packet` is the attack packet left after a blow under the attack limit;
    /// `target` is the body that took it.
    Struck {
        result: AttackResult,
        packet: Option<Vec<u8>>,
        target: i16,
    },
    /// A type-9 spell weapon ended the attack; `warned` is warning 39.
    Halted { packet: Vec<u8>, warned: bool },
}

/// A warrior's melee against a monster (bodies 10 and up) or another warrior.
///
/// Original: attack
#[allow(clippy::too_many_arguments)]
pub fn warrior_melee(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    attacker: i16,
    target: i16,
    items: &[Vec<u8>],
    attacks: &[u8],
    info: &[u8],
    buffs: &[u8],
) -> Result<WarriorMelee, RuleError> {
    let actor = usize::try_from(attacker).map_err(|_| "melee actor outside party")?;
    let current_turn = combat.active_body() == Some(attacker);
    let warrior = warriors.get_mut(actor).ok_or("melee actor outside party")?;
    let weapon = weapon(&warrior.bytes, items)?;
    let under_limit = warrior.record().attacks_used()? < warrior.record().attack_limit()?;
    let (base, rolls_condition) = match spell_weapon(
        warrior,
        weapon.filter(|_| under_limit),
        attacks,
        current_turn,
    )? {
        None => (attack::spell_info(attacks)?, true),
        Some(SpellWeapon::Strike {
            packet,
            rolls_condition,
        }) => (packet, rolls_condition),
        Some(SpellWeapon::Halt { packet, warned }) => {
            let count = warrior.record().attacks_used()?.wrapping_add(1);
            warrior.record_mut().set_attacks_used(count);
            return Ok(WarriorMelee::Halted { packet, warned });
        }
    };
    let (result, packet, target) = if target >= 10 {
        let monster = combat
            .roster
            .records
            .get_mut((target - 10) as usize)
            .ok_or("melee target outside the enemies")?;
        let (result, packet, reflected) = attack::warrior_attack_with_base(
            &mut warriors[actor],
            monster,
            random,
            base,
            rolls_condition,
            info,
            combat.difficulty,
            weapon,
        )?;
        (result, packet, if reflected { attacker } else { target })
    } else {
        let (result, packet, actual) = self::attack(
            warriors,
            actor,
            usize::try_from(target).map_err(|_| "melee target outside party")?,
            base,
            rolls_condition,
            crate::combat::party_buffs::mind_shield(buffs)?,
            items,
            combat.difficulty,
            random,
        )?;
        (result, packet, actual as i16)
    };
    Ok(WarriorMelee::Struck {
        result,
        packet: under_limit.then_some(packet),
        target,
    })
}

/// The melee weapon definition a warrior swings, if it holds one. An item outside the weapon
/// category in that slot is not supported.
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
    if ItemDefinition(definition).item_type()?.wrapping_abs() != 2 {
        return Err(RuleError::Unsupported {
            action: "combat.attack",
            dependency: "melee slot item outside the weapon category",
        });
    }
    Ok(Some(definition))
}

/// A melee weapon that names a spell above 1100 strikes with that spell's packet.
/// The weapon-condition test then reads the spell as its mode, so any condition is
/// admitted without a roll.
pub(crate) enum SpellWeapon {
    Strike {
        packet: Vec<u8>,
        rolls_condition: bool,
    },
    /// The attack ends; `warned` is warning 39 for the turn's warrior.
    Halt { packet: Vec<u8>, warned: bool },
}

/// A type-9 spell weapon never strikes for a warrior under player control. A warrior
/// on automatic whose bare-hand damage is below 8 returns to fists and strikes
/// without the weapon-condition roll; otherwise the attack ends.
///
/// Original: attack, toweap
pub(crate) fn spell_weapon(
    warrior: &mut Warrior,
    weapon: Option<&[u8]>,
    attacks: &[u8],
    current_turn: bool,
) -> Result<Option<SpellWeapon>, RuleError> {
    let Some(definition) = weapon.map(ItemDefinition) else {
        return Ok(None);
    };
    let spell = definition.spell()?;
    if spell <= 1100 {
        return Ok(None);
    }
    let packet = crate::combat::spells::load_packet(attacks, spell)?;
    if packet[20] != 9 {
        return Ok(Some(SpellWeapon::Strike {
            packet,
            rolls_condition: true,
        }));
    }
    if warrior.record().computer_control()? == 0 {
        return Ok(Some(SpellWeapon::Halt {
            packet,
            warned: current_turn,
        }));
    }
    // Without the panel's character, the bare-hand bonus is not added.
    if warrior.record().bare_hand_damage()? >= 8 {
        return Ok(Some(SpellWeapon::Halt {
            packet,
            warned: false,
        }));
    }
    warrior.record_mut().set_toggle(false);
    DevelopmentField::MeleeSound.write(&mut warrior.bytes, definition.impact_sound()?);
    Ok(Some(SpellWeapon::Strike {
        packet,
        rolls_condition: false,
    }))
}

/// Sets a warrior's melee damage: bonus damage plus one up to the bonus plus bare-hand damage, then
/// the weapon's melee and elemental damage scaled by the active warrior's second inventory slot
/// enhancement.
///
/// Original: attack
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

/// Prepares the attack packet; the bare-hand bonus counts only for an empty melee
/// slot.
///
/// Original: attack
pub(crate) fn prepare_packet_with_context(
    packet: &mut [u8],
    warrior: &[u8],
    weapon: Option<&[u8]>,
    selected: &[u8],
    bare_hand_bonus: i16,
) -> Result<(), RuleError> {
    prepare_packet(packet, warrior, weapon, selected)?;
    if weapon.is_none() {
        let mut attack = centurions_content::records::WeaponAttackParameters::read(packet)?;
        attack.physical.1 = attack.physical.1.wrapping_add(bare_hand_bonus);
        attack.write(packet);
    }
    Ok(())
}
