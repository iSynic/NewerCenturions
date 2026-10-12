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

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct VisibilityProbe {
        blocker: Option<(i32, i32)>,
        missing: Option<(i32, i32)>,
        reads: RefCell<Vec<(i32, i32)>>,
        fields: RefCell<Vec<i16>>,
    }

    impl VisibilityProbe {
        fn new() -> Self {
            Self {
                blocker: None,
                missing: None,
                reads: RefCell::new(Vec::new()),
                fields: RefCell::new(Vec::new()),
            }
        }
    }

    impl VisibilityServices for VisibilityProbe {
        fn map_word(&self, row: i32, column: i32) -> Option<i16> {
            self.reads.borrow_mut().push((row, column));
            if self.missing == Some((row, column)) {
                None
            } else {
                Some(i16::from(self.blocker == Some((row, column))))
            }
        }

        fn tile_visibility(&self, tile: i16) -> Option<i16> {
            self.fields.borrow_mut().push(tile);
            Some(1)
        }
    }

    fn input(center: (i32, i32)) -> VisibilityInput {
        VisibilityInput {
            center,
            map_origin: (20, 20),
            ignore_walls: 0,
        }
    }

    fn write(row: u8, column: u8) -> VisibilityWrite {
        VisibilityWrite { row, column }
    }

    #[test]
    fn short_fan_retains_order_duplicates_and_stops_at_fan_coordinate() {
        let services = VisibilityProbe::new();
        let plan = cansee2(input((5, 5)), &services).unwrap();
        assert_eq!(plan.writes.len(), 240);
        assert_eq!(
            &plan.writes[..6],
            &[
                write(25, 25),
                write(24, 24),
                write(23, 23),
                write(22, 22),
                write(21, 21),
                write(20, 20),
            ]
        );
        assert_eq!(plan.writes[11], write(21, 20));
        assert_eq!(plan.writes[120], write(25, 25));
        assert_eq!(services.reads.borrow().len(), 240);
        assert!(services.fields.borrow().is_empty());
    }

    #[test]
    fn long_fan_preserves_nine_then_eight_step_asymmetry() {
        let services = VisibilityProbe::new();
        let plan = cansee3(input((8, 8)), &services).unwrap();
        assert_eq!(plan.writes.len(), 510);
        assert_eq!(plan.writes[8], write(20, 20));
        assert_eq!(plan.writes[270], write(28, 28));
        assert_eq!(plan.writes[277], write(21, 21));
        assert_eq!(plan.writes[278], write(28, 28));
    }

    #[test]
    fn blocking_cell_is_revealed_before_the_next_fan_starts() {
        let mut services = VisibilityProbe::new();
        services.blocker = Some((24, 24));
        let plan = cansee2(input((5, 5)), &services).unwrap();
        assert_eq!(
            &plan.writes[..3],
            &[write(25, 25), write(24, 24), write(25, 25)]
        );
        assert!(!services.fields.borrow().is_empty());
        let mut bypass = input((5, 5));
        bypass.ignore_walls = -1;
        services.fields.borrow_mut().clear();
        assert_eq!(cansee2(bypass, &services).unwrap().writes.len(), 240);
        assert!(services.fields.borrow().is_empty());
    }

    #[test]
    fn negative_lower_bound_stalls_without_reading_or_writing() {
        let services = VisibilityProbe::new();
        let mut state = input((-1, 5));
        state.map_origin = (0, 0);
        assert!(cansee2(state, &services).unwrap().writes.is_empty());
        assert!(services.reads.borrow().is_empty());
    }

    #[test]
    fn full_long_argument_is_retained_for_the_center_comparison() {
        let mut services = VisibilityProbe::new();
        services.blocker = Some((25, 25));
        let plan = cansee2(input((0x10005, 0x10005)), &services).unwrap();
        assert_eq!(plan.writes, vec![write(25, 25); 40]);
        assert_eq!(services.fields.borrow().len(), 40);
    }

    #[test]
    fn upper_map_read_precedes_bounds_check_and_can_diagnose_missing_storage() {
        let mut services = VisibilityProbe::new();
        services.missing = Some((90, 0));
        let mut state = input((90, 0));
        state.map_origin = (0, 0);
        assert_eq!(
            cansee2(state, &services),
            Err(GeometryError::MissingMapWord { row: 90, column: 0 })
        );
        assert_eq!(*services.reads.borrow(), vec![(90, 0)]);
        let map: Vec<i16> = (0..180).collect();
        let tables = VisibilityTables {
            map: &map,
            tile_records: &[],
        };
        assert_eq!(tables.map_word(0, 90), Some(90));
    }

    #[test]
    fn body_rectangle_uses_signed_size_and_native_write_order() {
        let position = BodyPosition {
            horizontal: 3,
            vertical: 4,
        };
        let mut records = vec![0; 288];
        for (size, top, left) in [
            (-3_i16, 96, 64),
            (-2, 128, 96),
            (-1, 128, 96),
            (0, 128, 96),
            (1, 96, 96),
            (2, 128, 64),
            (3, 96, 64),
            (4, 128, 96),
        ] {
            records[16..18].copy_from_slice(&size.to_be_bytes());
            let services = BodyTables {
                warrior_positions: &[],
                enemy_positions: &[position],
                enemy_records: &records,
            };
            let plan = calcbodyrect(10, &services).unwrap();
            assert_eq!(
                plan.rect,
                BodyRect {
                    top,
                    left,
                    bottom: 160,
                    right: 128,
                }
            );
            assert_eq!(
                plan.writes[..4].iter().map(|w| w.field).collect::<Vec<_>>(),
                vec![
                    RectField::Top,
                    RectField::Left,
                    RectField::Right,
                    RectField::Bottom
                ]
            );
            if size == -3 || size == 3 {
                assert_eq!(plan.writes[4].field, RectField::Top);
                assert_eq!(plan.writes[5].field, RectField::Left);
            }
        }
    }

    #[test]
    fn rectangle_word_overflow_and_negative_body_diagnostic_are_explicit() {
        let services = BodyTables {
            warrior_positions: &[BodyPosition {
                horizontal: 1023,
                vertical: 1024,
            }],
            enemy_positions: &[],
            enemy_records: &[],
        };
        assert_eq!(
            calcbodyrect(0, &services).unwrap().rect,
            BodyRect {
                top: i16::MIN,
                left: 32736,
                bottom: -32736,
                right: i16::MIN,
            }
        );
        assert_eq!(
            calcbodyrect(-1, &services),
            Err(GeometryError::MissingWarriorPosition { body: -1 })
        );
    }
}
