use crate::combat::{Combat, attack, firearm};
mod enemy_melee;
mod warrior_melee;
use crate::{RandomState, RuleError};
use attack::AttackResult;
use centurions_content::Warrior;
use centurions_content::combat::MonsterRecord;
use centurions_content::records::{AttackPacket, ItemDefinition};
use enemy_melee::resolve as melee_enemy_inner;
use firearm::PreparedFirearm;
use warrior_melee::resolve as melee_inner;

pub fn ammo_id(enemy: &MonsterRecord, item_context: i16) -> Result<i16, RuleError> {
    Ok(if item_context == 0 {
        0
    } else {
        enemy.record().ammunition()?
    })
}

pub fn prepare(
    enemy: &mut MonsterRecord,
    weapon: &[u8],
    ammo: &[u8],
    attacks: &[u8],
) -> Result<PreparedFirearm, RuleError> {
    if enemy.bytes.len() != 288 || weapon.len() != 150 || ammo.len() != 150 {
        return Err(RuleError::Invalid("enemy weapon record size".into()));
    }
    let id = enemy.record().primary_weapon()?;
    if id <= 0 || !matches!(ItemDefinition(weapon).spell()?, 4101 | 4102) {
        return Err(RuleError::Unsupported {
            action: "combat.enemy_firearm",
            dependency: "enemy firearm outside recovered definitions",
        });
    }
    let packet = firearm::prepare_packet(weapon, ammo, attacks)?;
    if !AttackPacket(&packet).is_direct_weapon_damage()? {
        return Err(RuleError::Unsupported {
            action: "combat.enemy_firearm",
            dependency: "enemy firearm status effects",
        });
    }
    enemy.record_mut().set_selected_item(id);
    Ok(PreparedFirearm {
        sound: AttackPacket(&packet).impact_sound()?,
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

pub fn resolve(
    warrior: &mut Warrior,
    enemy: &mut MonsterRecord,
    prepared: &PreparedFirearm,
    random: &mut RandomState,
    difficulty: i16,
) -> Result<AttackResult, RuleError> {
    if warrior.bytes.len() != 3138 || enemy.bytes.len() != 288 || prepared.packet.len() != 58 {
        return Err(RuleError::Invalid("enemy firearm record size".into()));
    }
    let mut victim = warrior.clone();
    let mut attacker = enemy.clone();
    let mut next = random.clone();
    let result =
        firearm::resolve_enemy(&mut victim, &mut attacker, prepared, &mut next, difficulty)?;
    *warrior = victim;
    *enemy = attacker;
    *random = next;
    Ok(result)
}

pub fn melee(
    warrior: &mut Warrior,
    enemy: &mut MonsterRecord,
    weapon: &[u8],
    random: &mut RandomState,
    difficulty: i16,
    attacker_x: i16,
    target_x: i16,
) -> Result<AttackResult, RuleError> {
    Ok(melee_inner(
        warrior,
        enemy,
        weapon,
        random,
        difficulty,
        attacker_x,
        target_x,
        0,
        attack::default_attack_packet(),
        &[0; 20],
    )?
    .0)
}

#[allow(clippy::too_many_arguments)]
pub fn melee_warrior_with_packet(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    bodies: (i16, i16, usize, i16),
    weapon: &[u8],
    difficulty: i16,
    attacks: &[u8],
    retained: &[u8],
) -> Result<(AttackResult, Vec<u8>), RuleError> {
    melee_warrior_with_buffs(
        combat, warriors, random, bodies, weapon, difficulty, attacks, retained, &[0; 20],
    )
}

#[allow(clippy::too_many_arguments)]
pub fn melee_warrior_with_buffs(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    bodies: (i16, i16, usize, i16),
    weapon: &[u8],
    difficulty: i16,
    attacks: &[u8],
    retained: &[u8],
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
        retained,
        buffs,
    )?;
    Ok((result, packet))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn melee_warrior_with_target(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    bodies: (i16, i16, usize, i16),
    weapon: &[u8],
    difficulty: i16,
    attacks: Option<&[u8]>,
    retained: &[u8],
    buffs: &[u8],
) -> Result<(AttackResult, Vec<u8>, bool), RuleError> {
    let (attacker, target, index, actor) = bodies;
    let body_index = |body: i16| {
        usize::try_from(i32::from(body) - 10)
            .ok()
            .filter(|index| *index < combat.enemies.records.len())
            .ok_or_else(|| RuleError::Invalid("enemy melee body outside records".into()))
    };
    let a = body_index(attacker)?;
    let actor = usize::try_from(actor - 10)
        .ok()
        .filter(|index| *index < 100)
        .ok_or("invalid retained melee actor")?;
    let target = usize::try_from(target)
        .ok()
        .filter(|index| *index < 6 && *index < warriors.len())
        .ok_or_else(|| RuleError::Invalid("enemy melee warrior outside party".into()))?;
    if retained.len() != 58
        || combat.enemies.positions.len() <= a
        || combat.squad.positions.len() <= target
    {
        return Err(RuleError::Invalid("enemy melee context size".into()));
    }
    let source = combat.enemies.records[a].record();
    let loaded = source.selected_item()? != 0;
    let id = source.melee_weapon()?;
    let mut next = combat.clone();
    let mut next_warriors = warriors.to_vec();
    let mut rng = random.clone();
    if loaded {
        next.story_record_mut(actor)?
            .record_mut()
            .set_selected_item(id);
    }
    let result = melee_inner(
        &mut next_warriors[target],
        &mut next.enemies.records[a],
        weapon,
        &mut rng,
        difficulty,
        combat.enemies.positions[a].0,
        combat.squad.positions[target].0,
        index,
        attacks
            .map(attack::attack_packet)
            .transpose()?
            .unwrap_or_else(attack::default_attack_packet),
        buffs,
    )?;
    *combat = next;
    warriors.clone_from_slice(&next_warriors);
    *random = rng;
    Ok(result)
}

#[allow(clippy::too_many_arguments)]
pub fn melee_enemy_with_packet(
    combat: &mut Combat,
    random: &mut RandomState,
    bodies: (i16, i16, usize, i16),
    weapon: &[u8],
    difficulty: i16,
    attacks: &[u8],
    retained: &[u8],
) -> Result<(AttackResult, Vec<u8>), RuleError> {
    let (result, packet, _) = melee_enemy_with_target(
        combat,
        random,
        bodies,
        weapon,
        difficulty,
        Some(attacks),
        retained,
    )?;
    Ok((result, packet))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn melee_enemy_with_target(
    combat: &mut Combat,
    random: &mut RandomState,
    bodies: (i16, i16, usize, i16),
    weapon: &[u8],
    difficulty: i16,
    attacks: Option<&[u8]>,
    retained: &[u8],
) -> Result<(AttackResult, Vec<u8>, bool), RuleError> {
    if retained.len() != 58 {
        return Err(RuleError::Invalid("retained attack packet size".into()));
    }
    melee_enemy_inner(
        combat,
        random,
        bodies,
        weapon,
        difficulty,
        attacks
            .map(attack::attack_packet)
            .transpose()?
            .unwrap_or_else(attack::default_attack_packet),
    )
}

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
        attack::default_attack_packet(),
    )?
    .0)
}

#[allow(clippy::too_many_arguments)]
pub fn melee_with_packet(
    warrior: &mut Warrior,
    enemy: &mut MonsterRecord,
    weapon: &[u8],
    random: &mut RandomState,
    difficulty: i16,
    attacker_x: i16,
    target_x: i16,
    attacks: &[u8],
    retained: &[u8],
) -> Result<(AttackResult, Vec<u8>), RuleError> {
    melee_with_buffs(
        warrior, enemy, weapon, random, difficulty, attacker_x, target_x, attacks, retained,
        &[0; 20],
    )
}

#[allow(clippy::too_many_arguments)]
pub fn melee_with_buffs(
    warrior: &mut Warrior,
    enemy: &mut MonsterRecord,
    weapon: &[u8],
    random: &mut RandomState,
    difficulty: i16,
    attacker_x: i16,
    target_x: i16,
    attacks: &[u8],
    retained: &[u8],
    buffs: &[u8],
) -> Result<(AttackResult, Vec<u8>), RuleError> {
    if retained.len() != 58 {
        return Err(RuleError::Invalid("retained attack packet size".into()));
    }
    let (result, packet, _) = melee_inner(
        warrior,
        enemy,
        weapon,
        random,
        difficulty,
        attacker_x,
        target_x,
        0,
        attack::attack_packet(attacks)?,
        buffs,
    )?;
    Ok((result, packet))
}
