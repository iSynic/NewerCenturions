//! Geometry actions recovered from the original ordered instructions.

use std::fmt;

pub use crate::combat::lifecycle::can_see as cansee;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GeometryError {
    MissingMapWord { row: i32, column: i32 },
    MissingTileVisibility { tile: i16 },
    MissingWarriorPosition { body: i16 },
    MissingEnemyPosition { index: i16 },
    MissingEnemySize { index: u16 },
}

impl fmt::Display for GeometryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingMapWord { row, column } => {
                write!(f, "visibility map word unavailable at {row},{column}")
            }
            Self::MissingTileVisibility { tile } => {
                write!(f, "visibility field12 unavailable for tile {tile}")
            }
            Self::MissingWarriorPosition { body } => {
                write!(f, "body rectangle warrior position unavailable for {body}")
            }
            Self::MissingEnemyPosition { index } => {
                write!(f, "body rectangle enemy position unavailable for {index}")
            }
            Self::MissingEnemySize { index } => {
                write!(f, "body rectangle enemy size unavailable for {index}")
            }
        }
    }
}

impl std::error::Error for GeometryError {}

pub trait VisibilityServices {
    fn map_word(&self, row: i32, column: i32) -> Option<i16>;
    fn tile_visibility(&self, tile: i16) -> Option<i16>;
}

pub struct VisibilityTables<'a> {
    pub map: &'a [i16],
    pub tile_records: &'a [u8],
}

impl VisibilityServices for VisibilityTables<'_> {
    fn map_word(&self, row: i32, column: i32) -> Option<i16> {
        let offset = row.wrapping_mul(180).wrapping_add(column.wrapping_mul(2));
        self.map.get(usize::try_from(offset / 2).ok()?).copied()
    }

    fn tile_visibility(&self, tile: i16) -> Option<i16> {
        let offset = usize::from(tile as u16) * 40 + 12;
        let bytes: [u8; 2] = self.tile_records.get(offset..offset + 2)?.try_into().ok()?;
        Some(i16::from_be_bytes(bytes))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VisibilityInput {
    pub center: (i32, i32),
    pub map_origin: (i32, i32),
    pub ignore_walls: i16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VisibilityWrite {
    pub row: u8,
    pub column: u8,
}

impl VisibilityWrite {
    pub fn index(self) -> usize {
        usize::from(self.row) * 90 + usize::from(self.column)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VisibilityPlan {
    /// Each entry writes byte1, retaining repeated writes and native order.
    pub writes: Vec<VisibilityWrite>,
}

/// cansee2: two directional sweeps, ten fans each, six steps per fan.
pub fn cansee2(
    input: VisibilityInput,
    services: &(impl VisibilityServices + ?Sized),
) -> Result<VisibilityPlan, GeometryError> {
    reveal(input, services, 10, 5, [6, 6])
}

/// cansee3: fifteen fans; the first sweep takes nine steps, the second eight.
pub fn cansee3(
    input: VisibilityInput,
    services: &(impl VisibilityServices + ?Sized),
) -> Result<VisibilityPlan, GeometryError> {
    reveal(input, services, 15, 8, [9, 8])
}

fn reveal(
    input: VisibilityInput,
    services: &(impl VisibilityServices + ?Sized),
    fans: i16,
    pivot: i16,
    limits: [i16; 2],
) -> Result<VisibilityPlan, GeometryError> {
    let mut writes = Vec::new();
    for (sweep, limit) in limits.into_iter().enumerate() {
        for fixed_direction in [-1_i16, 1] {
            for fan in 0..fans {
                let mut row = input.center.0 as i16;
                let mut column = input.center.1 as i16;
                let mut fan_direction = match fan.cmp(&pivot) {
                    std::cmp::Ordering::Less => -1_i16,
                    std::cmp::Ordering::Equal => 0,
                    std::cmp::Ordering::Greater => 1,
                };
                for _ in 0..limit {
                    if (if sweep == 0 { row } else { column }) == fan {
                        fan_direction = 0;
                    }
                    let map_row = input.map_origin.0.wrapping_add(i32::from(row));
                    let map_column = input.map_origin.1.wrapping_add(i32::from(column));
                    if map_row < 0 || map_column < 0 {
                        continue;
                    }
                    let tile = services.map_word(map_row, map_column).ok_or(
                        GeometryError::MissingMapWord {
                            row: map_row,
                            column: map_column,
                        },
                    )?;
                    let blocked = if input.ignore_walls == 0 && tile > 0 && tile < 600 {
                        services
                            .tile_visibility(tile)
                            .ok_or(GeometryError::MissingTileVisibility { tile })?
                            == 1
                    } else {
                        false
                    };
                    let away_from_center = if sweep == 0 {
                        i32::from(column) != input.center.1
                    } else {
                        i32::from(row) != input.center.0
                    };
                    if map_row < 90 && map_column < 90 {
                        writes.push(VisibilityWrite {
                            row: map_row as u8,
                            column: map_column as u8,
                        });
                    }
                    if sweep == 0 {
                        row = row.wrapping_add(fan_direction);
                        column = column.wrapping_add(fixed_direction);
                    } else {
                        row = row.wrapping_add(fixed_direction);
                        column = column.wrapping_add(fan_direction);
                    }
                    if blocked && away_from_center {
                        break;
                    }
                }
            }
        }
    }
    Ok(VisibilityPlan { writes })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BodyPosition {
    pub horizontal: i16,
    pub vertical: i16,
}

pub trait BodyRectServices {
    fn warrior_position(&self, body: i16) -> Option<BodyPosition>;
    fn enemy_position(&self, index: i16) -> Option<BodyPosition>;
    fn enemy_size(&self, index: u16) -> Option<i16>;
}

pub struct BodyTables<'a> {
    pub warrior_positions: &'a [BodyPosition],
    pub enemy_positions: &'a [BodyPosition],
    pub enemy_records: &'a [u8],
}

impl BodyRectServices for BodyTables<'_> {
    fn warrior_position(&self, body: i16) -> Option<BodyPosition> {
        self.warrior_positions
            .get(usize::try_from(body).ok()?)
            .copied()
    }

    fn enemy_position(&self, index: i16) -> Option<BodyPosition> {
        self.enemy_positions
            .get(usize::try_from(index).ok()?)
            .copied()
    }

    fn enemy_size(&self, index: u16) -> Option<i16> {
        let offset = usize::from(index) * 288 + 16;
        let bytes: [u8; 2] = self
            .enemy_records
            .get(offset..offset + 2)?
            .try_into()
            .ok()?;
        Some(i16::from_be_bytes(bytes))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BodyRect {
    pub top: i16,
    pub left: i16,
    pub bottom: i16,
    pub right: i16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RectField {
    Top,
    Left,
    Bottom,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RectWrite {
    pub field: RectField,
    pub value: i16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BodyRectPlan {
    pub rect: BodyRect,
    pub writes: Vec<RectWrite>,
}

pub fn calcbodyrect(
    body: i16,
    services: &(impl BodyRectServices + ?Sized),
) -> Result<BodyRectPlan, GeometryError> {
    let enemy = body > 9;
    let index = body.wrapping_sub(10);
    let position = if enemy {
        services
            .enemy_position(index)
            .ok_or(GeometryError::MissingEnemyPosition { index })?
    } else {
        services
            .warrior_position(body)
            .ok_or(GeometryError::MissingWarriorPosition { body })?
    };
    let top = position.vertical.wrapping_mul(32);
    let left = position.horizontal.wrapping_mul(32);
    let mut rect = BodyRect {
        top,
        left,
        bottom: top.wrapping_add(32),
        right: left.wrapping_add(32),
    };
    let mut writes = vec![
        RectWrite {
            field: RectField::Top,
            value: rect.top,
        },
        RectWrite {
            field: RectField::Left,
            value: rect.left,
        },
        RectWrite {
            field: RectField::Right,
            value: rect.right,
        },
        RectWrite {
            field: RectField::Bottom,
            value: rect.bottom,
        },
    ];
    if enemy {
        let size = services
            .enemy_size(index as u16)
            .ok_or(GeometryError::MissingEnemySize {
                index: index as u16,
            })?;
        if matches!(size, -3 | 1 | 3) {
            rect.top = rect.top.wrapping_sub(32);
            writes.push(RectWrite {
                field: RectField::Top,
                value: rect.top,
            });
        }
        if matches!(size, -3 | 2 | 3) {
            rect.left = rect.left.wrapping_sub(32);
            writes.push(RectWrite {
                field: RectField::Left,
                value: rect.left,
            });
        }
    }
    Ok(BodyRectPlan { rect, writes })
}
