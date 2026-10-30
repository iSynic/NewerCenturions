//! Reading the battle map.
use super::CombatField;
use super::map_alias::MapAliases;
use crate::{RandomState, RuleError};

/// The tile a monster steps onto, read without clipping at the map edge.
///
/// Original: movemonster
pub(super) fn movement_tile(
    map: &CombatField,
    aliases: MapAliases<'_>,
    position: (i16, i16),
    offset: (i16, i16),
) -> Result<i16, RuleError> {
    let (x, y) = crate::combat::records::grid_point(position, map.map_shift);
    let (x, y) = (x + i32::from(offset.0), y + i32::from(offset.1));
    aliases
        .read(map, x * 90 + y)?
        .ok_or(RuleError::Unsupported {
            action: "combat.move",
            dependency: "native map read outside the map and the represented memory beside it",
        })
}

/// Builds the battle map from the thirty-by-thirty stretch of land around the party: each land
/// square becomes a three-by-three block of tiles. When a step takes five minutes (outdoors),
/// foliage is scattered over open ground, drawing random numbers.
pub fn build_tactical_map(
    mission: &centurions_content::mission::OpeningMission,
    position: (i32, i32),
    edge_word: i16,
    random: &mut RandomState,
) -> Result<CombatField, RuleError> {
    let property = |tile: i16, offset: usize| -> Result<i16, RuleError> {
        Ok(mission.tile_word(tile, offset)?)
    };
    centurions_content::mission::OpeningMission::index(position.0, position.1)?;
    let requested = (position.0 - 15, position.1 - 15);
    let origin = (requested.0.clamp(0, 60), requested.1.clamp(0, 60));
    let shift = (
        ((requested.0 - origin.0) * 3) as i16,
        ((requested.1 - origin.1) * 3) as i16,
    );
    let mut map = CombatField {
        tiles: vec![0; super::MAP_CELLS],
        edge_word,
        map_shift: shift,
    };
    for y in 0..30 {
        for x in 0..30 {
            let index =
                centurions_content::mission::OpeningMission::index(origin.0 + x, origin.1 + y)?;
            let floor = mission.floor[index];
            let tile = if floor <= 0 || floor >= 600 || mission.events[index] != 0 {
                mission.default_tile
            } else {
                floor
            };
            for dy in 0..3 {
                for dx in 0..3 {
                    map.tiles[((x * 3 + dx) * 90 + y * 3 + dy) as usize] =
                        property(tile, (20 + dy * 6 + dx * 2) as usize)?;
                }
            }
        }
    }
    if mission.minutes_per_tick == 5 {
        let foliage_mode = if property(206, 12)? == 0 { 2 } else { 1 };
        for y in 0..90 {
            for x in 0..90 {
                let index = x * 90 + y;
                let tile = map.tiles[index];
                let above = if index == 0 {
                    map.edge_word
                } else {
                    map.tiles[index - 1]
                };
                let sum = |tile, offsets: &[usize]| -> Result<i16, RuleError> {
                    offsets.iter().try_fold(0_i16, |sum, offset| {
                        Ok(sum.wrapping_add(property(tile, *offset)?))
                    })
                };
                let blocked = sum(tile, &[4, 6, 8, 12, 10])? != 0
                    || sum(above, &[4, 6, 8, 12])?.wrapping_add(property(tile, 10)?) != 0;
                let foliage = property(tile, 16)? != 0;
                if blocked {
                    if foliage {
                        map.tiles[index] = mission.default_tile;
                    }
                    continue;
                }
                if !foliage && random.bound(100) >= 5 {
                    continue;
                }
                if foliage_mode == 1 {
                    if random.bound(100) < 20 {
                        map.tiles[index] = random.range(207, 211);
                        let replacement = if (201..=211).contains(&above) {
                            206
                        } else {
                            random.range(201, 205)
                        };
                        if index == 0 {
                            map.edge_word = replacement;
                        } else {
                            map.tiles[index - 1] = replacement;
                        }
                        continue;
                    }
                } else if random.bound(100) < 20 && above > 113 {
                    map.tiles[index] = random.range(207, 211);
                    let replacement = map.tiles[index].wrapping_sub(6);
                    if index == 0 {
                        map.edge_word = replacement;
                    } else {
                        map.tiles[index - 1] = replacement;
                    }
                    continue;
                }
                map.tiles[index] = if random.bound(100) < 10 {
                    random.range(212, 222)
                } else {
                    mission.default_tile
                };
            }
        }
    }
    for tile in &mut map.tiles {
        *tile = tile.wrapping_add(1000);
    }
    Ok(map)
}
