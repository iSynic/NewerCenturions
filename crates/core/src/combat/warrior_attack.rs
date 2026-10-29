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

/// Loaded WEAPON work and the attacker's separately retained melee-slot state.
/// Bare attacks retain definition zero for immunity fields.
#[derive(Clone, Copy)]
pub(crate) struct PreparedMeleeWeapon<'a> {
    pub definition: &'a [u8],
    pub armed: bool,
}

/// Warrior-to-warrior branch of attack 0x100af0, including the unconditional reflection roll.
/// The selector -10 condition is rolled against the target before the hit roll and lands on
/// whichever warrior takes the damage.
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

/// Prepared warrior-to-warrior arithmetic; the caller has admitted and counted the attack.
/// `selected` supplies inventory-slot enhancement; the bare-hand bonus is retained work.
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
    victim.record_mut().mark_damaged();
    let health = victim.record().stamina()?.wrapping_sub(damage);
    victim.record_mut().set_stamina(health);
    victim.record_mut().add_damage_received(damage);
    if health < 1 && health > -10 {
        victim.record_mut().set_bleeding_flag(1);
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

/// One warrior call into attack 0x100af0, after the caller has loaded the weapon work.
pub enum WarriorMelee {
    /// `packet` is the attack packet the call leaves when it was under the attack limit;
    /// `target` is the body that took the blow.
    Struck {
        result: AttackResult,
        packet: Option<Vec<u8>>,
        target: i16,
    },
    /// A type-9 spell weapon ended the attack after its count; `warned` is warning 39.
    Halted { packet: Vec<u8>, warned: bool },
}

/// A warrior's melee against an enemy (bodies 10 and up) or another warrior.
#[allow(clippy::too_many_arguments)]
pub fn warrior_melee(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    attacker: i16,
    target: i16,
    items: &[Vec<u8>],
    attacks: &[u8],
    retained: &[u8],
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
        None => (attack::attack_packet(attacks)?, true),
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
        let enemy = combat
            .enemies
            .records
            .get_mut((target - 10) as usize)
            .ok_or("melee target outside the enemies")?;
        let (result, packet, reflected) = attack::warrior_attack_with_base(
            &mut warriors[actor],
            enemy,
            random,
            base,
            rolls_condition,
            retained,
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
    if ItemDefinition(definition).category()?.wrapping_abs() != 2 {
        return Err(RuleError::Unsupported {
            action: "combat.attack",
            dependency: "melee slot item outside the weapon category",
        });
    }
    Ok(Some(definition))
}

/// attack 0x1011c8: a melee weapon whose word 44 names a spell above 1100 strikes with
/// that spell's packet (loadspell2). The selector -10 block then reads the spell as its
/// mode, so any condition is admitted without a roll.
pub(crate) enum SpellWeapon {
    Strike {
        packet: Vec<u8>,
        rolls_condition: bool,
    },
    /// The attack ends after its count; `warned` is warning 39 for the turn's warrior.
    Halt { packet: Vec<u8>, warned: bool },
}

/// A type-9 spell packet never strikes for a manual warrior. An automatic warrior whose
/// bare-hand damage is below 8 returns to melee (toweap 0x15ef50) and strikes without
/// the selector -10 roll; otherwise the attack ends.
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
    if warrior.record().automatic_mode()? == 0 {
        return Ok(Some(SpellWeapon::Halt {
            packet,
            warned: current_turn,
        }));
    }
    // This record-only compatibility entry has no retained character-work bonus.
    if warrior.record().bare_hand_damage()? >= 8 {
        return Ok(Some(SpellWeapon::Halt {
            packet,
            warned: false,
        }));
    }
    warrior.record_mut().set_firing_mode(false);
    DevelopmentField::MeleeSound.write(&mut warrior.bytes, definition.impact_sound()?);
    Ok(Some(SpellWeapon::Strike {
        packet,
        rolls_condition: false,
    }))
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

/// attack 0x100af0 reads WORD447d96 only for an unequipped melee slot. Keep
/// the weapon's Single-factor calculations in prepare_packet unchanged.
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
