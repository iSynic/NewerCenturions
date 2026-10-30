//! Placing the squad and the monsters on the battle map.
use crate::combat::{
    CombatField, EnemySetup, SquadPlacement, calculate_movement, footprint, monster,
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

/// Places the battle's monsters at normal difficulty.
pub fn place_enemies(
    map: &mut CombatField,
    mission: &centurions_content::mission::OpeningMission,
    battle: &BattleRecord,
    definitions: &[u8],
    base: (i16, i16),
    random: &mut RandomState,
) -> Result<EnemySetup, RuleError> {
    place_enemies_at_difficulty(map, mission, battle, definitions, base, random, 0)
}

/// Places the battle's monsters, scaled for the difficulty.
pub fn place_enemies_at_difficulty(
    map: &mut CombatField,
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
            mon_pos: vec![],
        },
        difficulty,
    )
}

/// Places the battle's monsters after any allies. Each cell of the battle grid names a monster; its
/// level sets its stamina dice, and its armor, agility and psi are varied by chance before the
/// difficulty scales it. A negative monster number reverses its side.
///
/// Original: combatsetup
#[allow(clippy::too_many_arguments)]
pub(super) fn place_enemies_with_allies(
    map: &mut CombatField,
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
            let id = battle.battle_grid[x * 26 + y];
            if id == 0 || result.records.len() == 100 {
                continue;
            }
            let mut monster = MonsterRecord::read(definitions, id)?;
            let bytes = &mut monster.bytes;
            let dice = EnemyRecord::from_bytes(bytes).hit_dice()?;
            if dice == 0 {
                continue;
            }
            // The dice count is a signed byte, so a count above 127 never ends.
            if dice > 127 {
                return Err(RuleError::Unsupported {
                    action: "combat.enemies",
                    dependency: "enemy level outside recovered initialization range",
                });
            }
            super::monster_weapons::validate(bytes)?;
            EnemyRecordMut::from_bytes(bytes).set_target(-1);
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
            let armor = EnemyRecord::from_bytes(bytes).ac()?;
            if armor != 0 {
                EnemyRecordMut::from_bytes(bytes)
                    .set_armor(armor.wrapping_add(random.bound(3) - 2));
            }
            let armor = EnemyRecord::from_bytes(bytes).ac()?.max(0);
            EnemyRecordMut::from_bytes(bytes).set_armor(armor);
            let speed = EnemyRecord::from_bytes(bytes)
                .agility()?
                .wrapping_add(random.bound(3) - 2)
                .max(1);
            EnemyRecordMut::from_bytes(bytes).set_agility(speed);
            let psi = EnemyRecord::from_bytes(bytes).psi_points_max()?;
            let spread = psi / 10;
            let psi = psi.wrapping_add(random.range(-spread, spread));
            EnemyRecordMut::from_bytes(bytes).set_psi_points_max(psi);
            EnemyRecordMut::from_bytes(bytes).set_psi_points(psi);
            let mut stamina = EnemyRecord::from_bytes(bytes).stamina_bonus()?;
            for _ in 0..dice {
                stamina = stamina.wrapping_add(random.bound(8));
            }
            EnemyRecordMut::from_bytes(bytes).set_stamina(stamina);
            EnemyRecordMut::from_bytes(bytes).set_stamina_max(stamina);
            monster::adjust_battle(bytes, difficulty)?;
            if id < 0 {
                let hostile = i16::from(EnemyRecord::from_bytes(bytes).faction()? == 0);
                EnemyRecordMut::from_bytes(bytes).set_faction(hostile);
            }
            let selector = EnemyRecord::from_bytes(bytes).weapon()?;
            super::monster_weapons::initialize(
                bytes,
                random,
                super::monster_weapons::MatchOrder::Last,
            )?;
            // The table index replaces the weapon selector before this test.
            if matches!(selector, 0 | -1) && EnemyRecord::from_bytes(bytes).melee_weapon()? > 0 {
                let weapon = EnemyRecord::from_bytes(bytes).melee_weapon()?;
                EnemyRecordMut::from_bytes(bytes).set_weapon(weapon);
            }
            result.records.push(monster);
            result.mon_pos.push(position);
        }
    }
    Ok(result)
}

/// Places the squad in formation on the map from the start cell along the given direction, and
/// gives each warrior a clean start: no guarding, attacks, bleeding or target. Each living
/// warrior's movement is worked out for their race.
///
/// Original: combatsetup
pub fn place_squad(
    map: &mut CombatField,
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
    let origin = (
        map.map_shift.0.wrapping_add(5),
        map.map_shift.1.wrapping_add(5),
    );
    let mut result = SquadPlacement {
        pos: vec![],
        char_under: vec![],
    };
    for (index, warrior) in warriors.iter_mut().enumerate() {
        let bytes = &mut warrior.bytes;
        let mut record = WarriorRecordMut::from_bytes(bytes);
        record.set_guarding(0);
        record.set_attacks_used(0);
        record.set_spells_so_far(0);
        record.set_faction(0);
        record.set_been_attacked(0);
        record.set_attacks(0);
        record.set_bleeding(0);
        record.set_toggle(false);
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
                WarriorRecordMut::from_bytes(bytes).set_toggle(true);
            }
            CharacterInformation(bytes)
                .gender()?
                .wrapping_mul(8)
                .wrapping_add(30)
        };
        DevelopmentField::MeleeSound.write(bytes, sound);
        WarriorRecordMut::from_bytes(bytes).set_target(-1);
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
        result.pos.push(position);
        result.char_under.push(tile);
        if warrior.stamina()? > 0 {
            let race = warrior.record().race()?;
            let movement = *race_moves
                .get(race.wrapping_sub(1) as usize)
                .ok_or_else(|| RuleError::Invalid("race movement missing".into()))?;
            calculate_movement(warrior, movement)?;
            let value = warrior.record().movement_max()?;
            warrior.record_mut().set_movement(value);
        }
    }
    Ok(result)
}
