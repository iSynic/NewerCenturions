//! Placement for combat.
use crate::combat::{
    EnemySetup, SquadPlacement, TacticalMap, calculate_movement, footprint, monster,
};
use crate::{RandomState, RuleError};
use centurions_content::combat::{BattleRecord, MonsterRecord};
use centurions_content::{
    Content, Warrior,
    records::{
        CharacterInformation, DevelopmentField, EnemyRecord, EnemyRecordMut, ItemDefinition,
        WarriorRecord, WarriorRecordMut,
    },
};

pub fn place_enemies(
    map: &mut TacticalMap,
    mission: &centurions_content::mission::OpeningMission,
    battle: &BattleRecord,
    definitions: &[u8],
    base: (i16, i16),
    random: &mut RandomState,
) -> Result<EnemySetup, RuleError> {
    place_enemies_at_difficulty(map, mission, battle, definitions, base, random, 0)
}

pub fn place_enemies_at_difficulty(
    map: &mut TacticalMap,
    mission: &centurions_content::mission::OpeningMission,
    battle: &BattleRecord,
    definitions: &[u8],
    base: (i16, i16),
    random: &mut RandomState,
    difficulty: i16,
) -> Result<EnemySetup, RuleError> {
    place_enemies_with_allies(
        map,
        mission,
        battle,
        definitions,
        base,
        random,
        EnemySetup {
            records: vec![],
            positions: vec![],
        },
        difficulty,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn place_enemies_with_allies(
    map: &mut TacticalMap,
    mission: &centurions_content::mission::OpeningMission,
    battle: &BattleRecord,
    definitions: &[u8],
    base: (i16, i16),
    random: &mut RandomState,
    mut result: EnemySetup,
    difficulty: i16,
) -> Result<EnemySetup, RuleError> {
    for y in 0..26 {
        for x in 0..26 {
            let id = battle.cells[x * 26 + y];
            if id == 0 || result.records.len() == 100 {
                continue;
            }
            let mut monster = MonsterRecord::read(definitions, id)?;
            let bytes = &mut monster.bytes;
            let dice = EnemyRecord::from_bytes(bytes).level()?;
            if dice == 0 {
                continue;
            }
            if !(1..=127).contains(&dice) || EnemyRecord::from_bytes(bytes).selected_item()? < 0 {
                return Err(RuleError::Unsupported {
                    action: "combat.enemies",
                    dependency: "enemy level or random weapon tables",
                });
            }
            EnemyRecordMut::from_bytes(bytes).set_target_body(-1);
            EnemyRecordMut::from_bytes(bytes).set_target_scan_excluded(false);
            EnemyRecordMut::from_bytes(bytes).set_reaction_ready(false);
            let placed = footprint::find_placement(
                map,
                &monster,
                mission,
                result.records.len() as i16 + 10,
                (x as i16, y as i16),
                base,
                random,
            )?;
            *map = placed.map;
            monster = placed.monster;
            let position = placed.position;
            let bytes = &mut monster.bytes;
            EnemyRecordMut::from_bytes(bytes).set_movement_direction(1, 1);
            let armor = EnemyRecord::from_bytes(bytes).armor()?;
            if armor != 0 {
                EnemyRecordMut::from_bytes(bytes)
                    .set_armor(armor.wrapping_add(random.bound(3) - 2));
            }
            let armor = EnemyRecord::from_bytes(bytes).armor()?.max(0);
            EnemyRecordMut::from_bytes(bytes).set_armor(armor);
            let speed = EnemyRecord::from_bytes(bytes)
                .speed()?
                .wrapping_add(random.bound(3) - 2)
                .max(1);
            EnemyRecordMut::from_bytes(bytes).set_speed(speed);
            let psi = EnemyRecord::from_bytes(bytes).maximum_psi()?;
            let spread = psi / 10;
            let psi = psi.wrapping_add(random.range(-spread, spread));
            EnemyRecordMut::from_bytes(bytes).set_maximum_psi(psi);
            EnemyRecordMut::from_bytes(bytes).set_psi_points(psi);
            let mut stamina = EnemyRecord::from_bytes(bytes).base_stamina()?;
            for _ in 0..dice {
                stamina = stamina.wrapping_add(random.bound(8));
            }
            EnemyRecordMut::from_bytes(bytes).set_stamina(stamina);
            EnemyRecordMut::from_bytes(bytes).set_maximum_stamina(stamina);
            monster::adjust_battle(bytes, difficulty)?;
            if id < 0 {
                let hostile = i16::from(EnemyRecord::from_bytes(bytes).faction()? == 0);
                EnemyRecordMut::from_bytes(bytes).set_faction(hostile);
            }
            if EnemyRecord::from_bytes(bytes).selected_item()? == 0
                && EnemyRecord::from_bytes(bytes).melee_weapon()? > 0
            {
                let weapon = EnemyRecord::from_bytes(bytes).melee_weapon()?;
                EnemyRecordMut::from_bytes(bytes).set_selected_item(weapon);
            }
            result.records.push(monster);
            result.positions.push(position);
        }
    }
    Ok(result)
}

pub fn place_squad(
    map: &mut TacticalMap,
    mission: &centurions_content::mission::OpeningMission,
    content: &Content,
    warriors: &mut [Warrior],
    start: (i16, i16),
    direction: i16,
    race_moves: &[i8],
) -> Result<SquadPlacement, RuleError> {
    if !(1..=6).contains(&warriors.len()) || !(1..=360).contains(&direction) {
        return Err(RuleError::Invalid(
            "formation requires one to six warriors and a direction".into(),
        ));
    }
    let vertical = !(45..=315).contains(&direction) || (direction > 135 && direction < 225);
    let shift_lr = if direction > 90 && direction < 270 {
        1
    } else {
        -1
    };
    let shift_ud = if direction < 180 { -1 } else { 1 };
    let origin = (map.shift.0.wrapping_add(5), map.shift.1.wrapping_add(5));
    let mut result = SquadPlacement {
        positions: vec![],
        under_tiles: vec![],
    };
    for (index, warrior) in warriors.iter_mut().enumerate() {
        let bytes = &mut warrior.bytes;
        let mut record = WarriorRecordMut::from_bytes(bytes);
        record.set_guarding(0);
        record.set_attacks_used(0);
        record.set_scans_used(0);
        record.set_faction(0);
        record.set_damage_marker(0);
        record.set_action_points(0);
        record.set_bleeding_flag(0);
        record.set_firing_mode(false);
        record.initialize_placement_marker();
        let weapon = WarriorRecord::from_bytes(bytes).melee_weapon()?;
        let sound = if weapon != 0 {
            ItemDefinition(
                content
                    .items
                    .get(weapon.unsigned_abs() as usize)
                    .ok_or_else(|| RuleError::Invalid("weapon definition missing".into()))?,
            )
            .impact_sound()?
        } else {
            if WarriorRecord::from_bytes(bytes).equipped_weapon()? != 0 {
                WarriorRecordMut::from_bytes(bytes).set_firing_mode(true);
            }
            CharacterInformation(bytes)
                .gender()?
                .wrapping_mul(8)
                .wrapping_add(30)
        };
        DevelopmentField::MeleeSound.write(bytes, sound);
        WarriorRecordMut::from_bytes(bytes).set_target_body(-1);
        let row = index as i16 / 3;
        let mut found = None;
        'search: for ring in 1..=180_i16 {
            for dy in -ring..ring {
                for dx in -ring..ring {
                    let relative = if vertical {
                        (
                            dx + row * shift_lr + start.0 * 3,
                            dy + row * shift_ud + index as i16 + start.1 * 3 - row * 3,
                        )
                    } else {
                        (
                            dx + row * shift_lr + index as i16 + start.0 * 3 - row * 3,
                            dy + row * shift_ud + start.1 * 3,
                        )
                    };
                    let absolute = (relative.0 + origin.0, relative.1 + origin.1);
                    if !(5..85).contains(&absolute.0) || !(5..85).contains(&absolute.1) {
                        continue;
                    }
                    let cell = absolute.0 as usize * 90 + absolute.1 as usize;
                    let tile = map.tiles[cell];
                    if tile > 999 && mission.tile_word(tile - 1000, 4)? == 0 {
                        found = Some((relative, cell, tile));
                        break 'search;
                    }
                }
            }
        }
        let (position, cell, tile) =
            found.ok_or_else(|| RuleError::Invalid("no free squad placement cell".into()))?;
        map.tiles[cell] = index as i16;
        result.positions.push(position);
        result.under_tiles.push(tile);
        if warrior.stamina()? > 0 {
            let race = warrior.record().race()?;
            let movement = *race_moves
                .get(race.wrapping_sub(1) as usize)
                .ok_or_else(|| RuleError::Invalid("race movement missing".into()))?;
            calculate_movement(warrior, movement)?;
            let value = warrior.record().movement_per_round()?;
            warrior.record_mut().set_movement_points(value);
        }
    }
    Ok(result)
}
