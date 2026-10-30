use crate::combat::spells::FIRST_SPELL_DESCRIPTOR;
use crate::combat::{Combat, attack, firearm};
use centurions_content::combat::MONSTER_BYTES;
use centurions_content::{ITEM_BYTES, WARRIOR_BYTES};
mod enemy_melee;
mod warrior_melee;
use crate::{RandomState, RuleError};
use attack::AttackResult;
use centurions_content::Warrior;
use centurions_content::combat::MonsterRecord;
use centurions_content::records::{ItemDefinition, SpellInfoRecord};
use enemy_melee::resolve as melee_enemy_inner;
pub(crate) use enemy_melee::weapon_core as melee_enemy_core;
use firearm::PreparedFirearm;
use warrior_melee::resolve as melee_inner;
pub(crate) use warrior_melee::resolve as melee_party_prepared;

/// The ammunition a monster fires; none unless it has an item to fire.
pub fn ammo_id(monster: &MonsterRecord, item_context: i16) -> Result<i16, RuleError> {
    Ok(if item_context == 0 {
        0
    } else {
        monster.record().ammunition()?
    })
}

/// Readies a monster's missile shot at a warrior from its primary weapon and ammunition. A monster
/// with no primary weapon, or whose weapon is not a firearm (its spell below the first spell
/// descriptor), is reported as unsupported.
pub fn prepare(
    monster: &mut MonsterRecord,
    weapon: &[u8],
    ammo: &[u8],
    attacks: &[u8],
) -> Result<PreparedFirearm, RuleError> {
    if monster.bytes.len() != MONSTER_BYTES
        || weapon.len() != ITEM_BYTES
        || ammo.len() != ITEM_BYTES
    {
        return Err(RuleError::Invalid("enemy weapon record size".into()));
    }
    let id = monster.record().primary_weapon()?;
    if id <= 0 || ItemDefinition(weapon).spell()? < FIRST_SPELL_DESCRIPTOR {
        return Err(RuleError::Unsupported {
            action: "combat.enemy_firearm",
            dependency: "enemy firearm outside recovered definitions",
        });
    }
    let packet = firearm::prepare_packet(weapon, ammo, attacks)?;
    monster.record_mut().set_weapon(id);
    Ok(PreparedFirearm {
        sound: SpellInfoRecord(&packet).impact_sound()?,
        packet,
        slot: 0,
        shots: i16::from(ItemDefinition(weapon).selected_attack_count()),
        rounds: ItemDefinition(weapon).rounds_per_shot()?,
        weight: 0,
        armor_filter: ItemDefinition(weapon)
            .hit_roll_filter()?
            .wrapping_add(ItemDefinition(ammo).hit_roll_filter()?)
            != 0,
    })
}

/// Fires the prepared shot at a warrior. If the shot cannot be resolved, neither side nor the
/// random stream changes.
pub fn resolve(
    warrior: &mut Warrior,
    monster: &mut MonsterRecord,
    prepared: &PreparedFirearm,
    random: &mut RandomState,
    difficulty: i16,
) -> Result<AttackResult, RuleError> {
    if warrior.bytes.len() != WARRIOR_BYTES
        || monster.bytes.len() != MONSTER_BYTES
        || prepared.packet.len() != 58
    {
        return Err(RuleError::Invalid("enemy firearm record size".into()));
    }
    let mut victim = warrior.clone();
    let mut attacker = monster.clone();
    let mut next = random.clone();
    let result =
        firearm::resolve_enemy(&mut victim, &mut attacker, prepared, &mut next, difficulty)?;
    *warrior = victim;
    *monster = attacker;
    *random = next;
    Ok(result)
}

/// A monster's melee blow at a warrior with the default attack packet and no party buffs.
pub fn melee(
    warrior: &mut Warrior,
    monster: &mut MonsterRecord,
    weapon: &[u8],
    random: &mut RandomState,
    difficulty: i16,
    attacker_x: i16,
    target_x: i16,
) -> Result<AttackResult, RuleError> {
    Ok(melee_inner(
        warrior,
        monster,
        weapon,
        random,
        difficulty,
        attacker_x,
        target_x,
        0,
        attack::default_spell_info(),
        &[0; 20],
    )?
    .0)
}

/// A monster's melee blow at a warrior with a retained attack packet and no party buffs. Returns
/// the result and the packet.
#[allow(clippy::too_many_arguments)]
pub fn melee_warrior_with_packet(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    bodies: (i16, i16, usize, i16),
    weapon: &[u8],
    difficulty: i16,
    attacks: &[u8],
    info: &[u8],
) -> Result<(AttackResult, Vec<u8>), RuleError> {
    melee_warrior_with_buffs(
        combat, warriors, random, bodies, weapon, difficulty, attacks, info, &[0; 20],
    )
}

/// A monster's melee blow at a warrior, with a retained attack packet and the party's buffs.
/// Returns the result and the packet.
#[allow(clippy::too_many_arguments)]
pub fn melee_warrior_with_buffs(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    bodies: (i16, i16, usize, i16),
    weapon: &[u8],
    difficulty: i16,
    attacks: &[u8],
    info: &[u8],
    buffs: &[u8],
) -> Result<(AttackResult, Vec<u8>), RuleError> {
    let (result, packet, _) = melee_warrior_with_target(
        combat,
        warriors,
        random,
        bodies,
        weapon,
        difficulty,
        Some(attacks),
        info,
        buffs,
    )?;
    Ok((result, packet))
}

/// Resolves a monster's melee blow at a warrior on the battle map. The blow is all or nothing: on
/// an error the battle, the warriors and the random stream are left as they were.
#[allow(clippy::too_many_arguments)]
pub(crate) fn melee_warrior_with_target(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    bodies: (i16, i16, usize, i16),
    weapon: &[u8],
    difficulty: i16,
    attacks: Option<&[u8]>,
    info: &[u8],
    buffs: &[u8],
) -> Result<(AttackResult, Vec<u8>, bool), RuleError> {
    let (attacker, target, index, actor) = bodies;
    let body_index = |body: i16| {
        usize::try_from(i32::from(body) - 10)
            .ok()
            .filter(|index| *index < combat.roster.records.len())
            .ok_or_else(|| RuleError::Invalid("enemy melee body outside records".into()))
    };
    let attacker_index = body_index(attacker)?;
    let actor = usize::try_from(actor - 10)
        .ok()
        .filter(|index| *index < 100)
        .ok_or("invalid retained melee actor")?;
    let target = usize::try_from(target)
        .ok()
        .filter(|index| *index < 6 && *index < warriors.len())
        .ok_or_else(|| RuleError::Invalid("enemy melee warrior outside party".into()))?;
    if info.len() != 58
        || combat.roster.mon_pos.len() <= attacker_index
        || combat.squad.pos.len() <= target
    {
        return Err(RuleError::Invalid("enemy melee context size".into()));
    }
    let source = combat.roster.records[attacker_index].record();
    let loaded = source.weapon()? != 0;
    let id = source.melee_weapon()?;
    let mut next = combat.clone();
    let mut next_warriors = warriors.to_vec();
    let mut rng = random.clone();
    if loaded {
        next.story_record_mut(actor)?.record_mut().set_weapon(id);
    }
    let result = melee_inner(
        &mut next_warriors[target],
        &mut next.roster.records[attacker_index],
        weapon,
        &mut rng,
        difficulty,
        combat.roster.mon_pos[attacker_index].0,
        combat.squad.pos[target].0,
        index,
        attacks
            .map(attack::spell_info)
            .transpose()?
            .unwrap_or_else(attack::default_spell_info),
        buffs,
    )?;
    *combat = next;
    warriors.clone_from_slice(&next_warriors);
    *random = rng;
    Ok(result)
}

/// A monster's melee blow at another monster with a retained attack packet. Returns the result and
/// the packet.
#[allow(clippy::too_many_arguments)]
pub fn melee_enemy_with_packet(
    combat: &mut Combat,
    random: &mut RandomState,
    bodies: (i16, i16, usize, i16),
    weapon: &[u8],
    difficulty: i16,
    attacks: &[u8],
    info: &[u8],
) -> Result<(AttackResult, Vec<u8>), RuleError> {
    let (result, packet, _) = melee_enemy_with_target(
        combat,
        random,
        bodies,
        weapon,
        difficulty,
        Some(attacks),
        info,
    )?;
    Ok((result, packet))
}

/// Resolves a monster's melee blow at another monster on the battle map. The blow is all or
/// nothing.
#[allow(clippy::too_many_arguments)]
pub(crate) fn melee_enemy_with_target(
    combat: &mut Combat,
    random: &mut RandomState,
    bodies: (i16, i16, usize, i16),
    weapon: &[u8],
    difficulty: i16,
    attacks: Option<&[u8]>,
    info: &[u8],
) -> Result<(AttackResult, Vec<u8>, bool), RuleError> {
    if info.len() != 58 {
        return Err(RuleError::Invalid("retained attack packet size".into()));
    }
    melee_enemy_inner(
        combat,
        random,
        bodies,
        weapon,
        difficulty,
        attacks
            .map(attack::spell_info)
            .transpose()?
            .unwrap_or_else(attack::default_spell_info),
    )
}

/// A monster's melee blow at another monster with the default attack packet.
pub fn melee_enemy(
    combat: &mut Combat,
    random: &mut RandomState,
    bodies: (i16, i16, usize, i16),
    weapon: &[u8],
    difficulty: i16,
) -> Result<AttackResult, RuleError> {
    Ok(melee_enemy_inner(
        combat,
        random,
        bodies,
        weapon,
        difficulty,
        attack::default_spell_info(),
    )?
    .0)
}

/// A monster's melee blow at a warrior, given where each stands and a retained attack packet, with
/// no party buffs. Returns the result and the packet.
#[allow(clippy::too_many_arguments)]
pub fn melee_with_packet(
    warrior: &mut Warrior,
    monster: &mut MonsterRecord,
    weapon: &[u8],
    random: &mut RandomState,
    difficulty: i16,
    attacker_x: i16,
    target_x: i16,
    attacks: &[u8],
    info: &[u8],
) -> Result<(AttackResult, Vec<u8>), RuleError> {
    melee_with_buffs(
        warrior, monster, weapon, random, difficulty, attacker_x, target_x, attacks, info, &[0; 20],
    )
}

/// A monster's melee blow at a warrior, given where each stands, a retained attack packet and the
/// party's buffs. Returns the result and the packet.
#[allow(clippy::too_many_arguments)]
pub fn melee_with_buffs(
    warrior: &mut Warrior,
    monster: &mut MonsterRecord,
    weapon: &[u8],
    random: &mut RandomState,
    difficulty: i16,
    attacker_x: i16,
    target_x: i16,
    attacks: &[u8],
    info: &[u8],
    buffs: &[u8],
) -> Result<(AttackResult, Vec<u8>), RuleError> {
    if info.len() != 58 {
        return Err(RuleError::Invalid("retained attack packet size".into()));
    }
    let (result, packet, _) = melee_inner(
        warrior,
        monster,
        weapon,
        random,
        difficulty,
        attacker_x,
        target_x,
        0,
        attack::spell_info(attacks)?,
        buffs,
    )?;
    Ok((result, packet))
}
