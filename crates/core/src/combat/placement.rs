//! Placement for combat.
use crate::combat::{
    EnemySetup, SquadPlacement, TacticalMap, calculate_movement, footprint, monster, put_word,
};
use crate::{RandomState, RuleError};
use centurions_content::combat::{BattleRecord, MonsterRecord};
use centurions_content::{Content, Warrior, word};

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
            let dice = word(bytes, 0)?;
            if dice == 0 {
                continue;
            }
            if !(1..=127).contains(&dice) || word(bytes, 0x4e)? < 0 {
                return Err(RuleError::Unsupported {
                    action: "combat.enemies",
                    dependency: "enemy level or random weapon tables",
                });
            }
            put_word(bytes, 0x10c, -1);
            bytes[0x82] = 0;
            bytes[0x104] = 0;
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
            bytes[0x107] = 1;
            bytes[0x108] = 1;
            let armor = word(bytes, 8)?;
            if armor != 0 {
                put_word(bytes, 8, armor.wrapping_add(random.bound(3) - 2));
            }
            let armor = word(bytes, 8)?.max(0);
            put_word(bytes, 8, armor);
            let speed = word(bytes, 4)?.wrapping_add(random.bound(3) - 2).max(1);
            put_word(bytes, 4, speed);
            let psi = word(bytes, 0x52)?;
            let spread = psi / 10;
            let psi = psi.wrapping_add(random.range(-spread, spread));
            put_word(bytes, 0x52, psi);
            put_word(bytes, 0x10e, psi);
            let mut stamina = word(bytes, 2)?;
            for _ in 0..dice {
                stamina = stamina.wrapping_add(random.bound(8));
            }
            put_word(bytes, 0x110, stamina);
            put_word(bytes, 0x112, stamina);
            monster::adjust_battle(bytes, difficulty)?;
            if id < 0 {
                let hostile = i16::from(word(bytes, 0xe)? == 0);
                put_word(bytes, 0xe, hostile);
            }
            if word(bytes, 0x4e)? == 0 && word(bytes, 0x28)? > 0 {
                let weapon = word(bytes, 0x28)?;
                put_word(bytes, 0x4e, weapon);
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
        for offset in [0x16, 0xacc, 0xac8, 0x10, 0x14, 0x40] {
            bytes[offset..offset + 2].fill(0);
        }
        bytes[0x9de] = 0;
        bytes[0x9dd] = 0;
        bytes[0x9e0] = 1;
        let weapon = word(bytes, 0xca)?;
        let sound = if weapon != 0 {
            word(
                content
                    .items
                    .get(weapon.unsigned_abs() as usize)
                    .ok_or_else(|| RuleError::Invalid("weapon definition missing".into()))?,
                14,
            )?
        } else {
            if word(bytes, 0xe4)? != 0 {
                bytes[0x9dd] = 1;
            }
            word(bytes, 0x38)?.wrapping_mul(8).wrapping_add(30)
        };
        bytes[0x128..0x12a].copy_from_slice(&sound.to_be_bytes());
        bytes[0x18..0x1a].copy_from_slice(&(-1_i16).to_be_bytes());
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
            let value = word(&warrior.bytes, 0x3e)?;
            warrior.bytes[0x3c..0x3e].copy_from_slice(&value.to_be_bytes());
        }
    }
    Ok(result)
}
