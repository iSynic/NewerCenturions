//! Spatial queries recovered from the original ordered instructions.

use std::fmt;

pub type Position = (i16, i16);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpatialError {
    MissingPosition { body: i16 },
    MissingWarriorField { index: i16, offset: usize },
    MissingEnemyField { index: i16, offset: usize },
    MissingMapWord { row: i32, column: i32 },
    MissingDistanceSlot { body: i16 },
    NonterminatingByteLoop,
    UnorderedDirection,
    LayoutRead(String),
    Visibility(String),
}

impl fmt::Display for SpatialError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "spatial query unavailable: {self:?}")
    }
}

impl std::error::Error for SpatialError {}

pub trait SpatialServices {
    fn warrior_position(&self, index: i16) -> Option<Position>;
    fn enemy_position(&self, index: i16) -> Option<Position>;
    fn position(&self, body: i16) -> Option<Position> {
        if body <= 9 {
            self.warrior_position(body)
        } else {
            self.enemy_position(body.wrapping_sub(10))
        }
    }
    fn warrior_byte(&self, index: i16, offset: usize) -> Option<u8>;
    fn warrior_word(&self, index: i16, offset: usize) -> Option<i16>;
    fn enemy_byte(&self, index: i16, offset: usize) -> Option<u8>;
    fn enemy_word(&self, index: i16, offset: usize) -> Option<i16>;
    fn map_word(&self, row: i32, column: i32) -> Option<i16>;
}

pub struct SpatialTables<'a> {
    pub warriors: &'a [u8],
    pub enemies: &'a [u8],
    pub warrior_positions: &'a [Position],
    pub enemy_positions: &'a [Position],
    pub map: &'a [i16],
}

fn record_byte(bytes: &[u8], stride: usize, index: i16, offset: usize) -> Option<u8> {
    bytes
        .get(usize::from(index as u16) * stride + offset)
        .copied()
}

fn record_word(bytes: &[u8], stride: usize, index: i16, offset: usize) -> Option<i16> {
    let start = usize::from(index as u16) * stride + offset;
    Some(i16::from_be_bytes(
        bytes.get(start..start + 2)?.try_into().ok()?,
    ))
}

impl SpatialServices for SpatialTables<'_> {
    fn warrior_position(&self, index: i16) -> Option<Position> {
        self.warrior_positions
            .get(usize::try_from(index).ok()?)
            .copied()
    }

    fn enemy_position(&self, index: i16) -> Option<Position> {
        self.enemy_positions
            .get(usize::try_from(index).ok()?)
            .copied()
    }

    fn warrior_byte(&self, index: i16, offset: usize) -> Option<u8> {
        record_byte(self.warriors, 3138, index, offset)
    }

    fn warrior_word(&self, index: i16, offset: usize) -> Option<i16> {
        record_word(self.warriors, 3138, index, offset)
    }

    fn enemy_byte(&self, index: i16, offset: usize) -> Option<u8> {
        record_byte(self.enemies, 288, index, offset)
    }

    fn enemy_word(&self, index: i16, offset: usize) -> Option<i16> {
        record_word(self.enemies, 288, index, offset)
    }

    fn map_word(&self, row: i32, column: i32) -> Option<i16> {
        let address = row.wrapping_mul(180).wrapping_add(column.wrapping_mul(2));
        self.map.get(usize::try_from(address / 2).ok()?).copied()
    }
}

fn position(
    services: &(impl SpatialServices + ?Sized),
    body: i16,
) -> Result<Position, SpatialError> {
    services
        .position(body)
        .ok_or(SpatialError::MissingPosition { body })
}

fn warrior_word(
    services: &(impl SpatialServices + ?Sized),
    index: i16,
    offset: usize,
) -> Result<i16, SpatialError> {
    services
        .warrior_word(index, offset)
        .ok_or(SpatialError::MissingWarriorField { index, offset })
}

fn warrior_position(
    services: &(impl SpatialServices + ?Sized),
    index: i16,
) -> Result<Position, SpatialError> {
    services
        .warrior_position(index)
        .ok_or(SpatialError::MissingPosition { body: index })
}

fn enemy_position(
    services: &(impl SpatialServices + ?Sized),
    index: i16,
) -> Result<Position, SpatialError> {
    services
        .enemy_position(index)
        .ok_or(SpatialError::MissingPosition {
            body: index.wrapping_add(10),
        })
}

fn warrior_byte(
    services: &(impl SpatialServices + ?Sized),
    index: i16,
    offset: usize,
) -> Result<u8, SpatialError> {
    services
        .warrior_byte(index, offset)
        .ok_or(SpatialError::MissingWarriorField { index, offset })
}

fn enemy_word(
    services: &(impl SpatialServices + ?Sized),
    index: i16,
    offset: usize,
) -> Result<i16, SpatialError> {
    services
        .enemy_word(index, offset)
        .ok_or(SpatialError::MissingEnemyField { index, offset })
}

fn enemy_byte(
    services: &(impl SpatialServices + ?Sized),
    index: i16,
    offset: usize,
) -> Result<u8, SpatialError> {
    services
        .enemy_byte(index, offset)
        .ok_or(SpatialError::MissingEnemyField { index, offset })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RangeWork {
    pub squared_distance: f64,
    pub distance: f64,
    pub range_word: i16,
}

/// getrangetobody, including WORD subtraction and the final LONG-to-WORD truncation.
pub fn getrangetobody(
    body: i16,
    target: Position,
    services: &(impl SpatialServices + ?Sized),
) -> Result<RangeWork, SpatialError> {
    let origin = position(services, body)?;
    let first = f64::from(f32::from(target.0.wrapping_sub(origin.0)));
    let second = f64::from(f32::from(target.1.wrapping_sub(origin.1)));
    let squared_distance = first * first + second * second;
    let distance = squared_distance.sqrt();
    Ok(RangeWork {
        squared_distance,
        distance,
        range_word: distance.trunc() as i32 as i16,
    })
}

pub fn checkrange(
    body: i16,
    target: Position,
    maximum: i16,
    services: &(impl SpatialServices + ?Sized),
) -> Result<(bool, RangeWork), SpatialError> {
    let work = getrangetobody(body, target, services)?;
    Ok((work.range_word <= maximum, work))
}

/// direction converts the two native extended arguments to SINGLE first.
pub fn direction(first: f64, second: f64, incoming_d1: i8) -> Result<i16, SpatialError> {
    let first = first as f32;
    let second = second as f32;
    if first.is_nan() || second.is_nan() {
        return Err(SpatialError::UnorderedDirection);
    }
    let value = if first > 0.0 {
        if second < 0.0 {
            1
        } else if second == 0.0 {
            2
        } else {
            3
        }
    } else if first < 0.0 {
        if second > 0.0 {
            5
        } else if second == 0.0 {
            6
        } else {
            7
        }
    } else if second > 0.0 {
        4
    } else if second < 0.0 {
        0
    } else {
        incoming_d1
    };
    Ok(i16::from(value))
}

pub trait LayoutServices {
    /// None means fopen failed; returned bytes are the actual fread prefix.
    fn read_layout(&self) -> Result<Option<Vec<u8>>, SpatialError>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayoutInput {
    pub argument: i32,
    pub horizontal_transition: u8,
    pub vertical_transition: u8,
    pub first_coordinate: i32,
    pub second_coordinate: i32,
    pub previous_layout: [u8; 256],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayoutPlan {
    pub native_return: u32,
    pub layout: [u8; 256],
}

/// checklayout retains unread bytes and applies transitions even after a failed scan.
pub fn checklayout(
    input: LayoutInput,
    services: &(impl LayoutServices + ?Sized),
) -> Result<LayoutPlan, SpatialError> {
    let mut layout = input.previous_layout;
    let Some(bytes) = services.read_layout()? else {
        return Ok(LayoutPlan {
            native_return: (-2_i32) as u32,
            layout,
        });
    };
    let count = bytes.len().min(layout.len());
    layout[..count].copy_from_slice(&bytes[..count]);
    let lookup = if input.argument == 0 {
        -1
    } else {
        input.argument as i16
    };
    let read = |row: i16, column: i16| {
        let offset = row as usize * 32 + column as usize * 2;
        i16::from_be_bytes([layout[offset], layout[offset + 1]])
    };
    let mut row = 8_i16;
    let mut column = 16_i16;
    'scan: for candidate_row in 0..8 {
        for candidate_column in 0..16 {
            if read(candidate_row, candidate_column) == lookup {
                row = candidate_row;
                column = candidate_column;
                break 'scan;
            }
        }
    }
    if input.horizontal_transition != 0 {
        if input.first_coordinate == 0 {
            column -= 1;
        }
        if input.first_coordinate == 9 {
            column += 1;
        }
    }
    if input.vertical_transition != 0 {
        if input.second_coordinate == 0 {
            row -= 1;
        }
        if input.second_coordinate == 9 {
            row += 1;
        }
    }
    let native_return = if !(0..16).contains(&column) || !(0..8).contains(&row) {
        (-2_i32) as u32
    } else {
        match read(row, column) {
            value if value == 0 || value == lookup => (-2_i32) as u32,
            -1 => 0,
            value => u32::from(value as u16),
        }
    };
    Ok(LayoutPlan {
        native_return,
        layout,
    })
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClosestPlan {
    pub enemy_index: i16,
    pub range_calls: Vec<RangeWork>,
}

pub fn getclosest(
    body: i16,
    species: i16,
    maximum: i16,
    enemy_count: i16,
    services: &(impl SpatialServices + ?Sized),
) -> Result<ClosestPlan, SpatialError> {
    let maximum = if maximum == 0 { 1000 } else { maximum };
    let mut best = maximum;
    let mut plan = ClosestPlan {
        enemy_index: -1,
        range_calls: Vec::new(),
    };
    for index in 0..enemy_count {
        if enemy_word(services, index, 0x110)? <= 0
            || i16::from(enemy_byte(services, index, 0x103)? as i8) != species
        {
            continue;
        }
        let work = getrangetobody(body, enemy_position(services, index)?, services)?;
        if work.range_word < best && work.range_word < maximum {
            best = work.range_word;
            plan.enemy_index = index;
        }
        plan.range_calls.push(work);
    }
    Ok(plan)
}

pub fn checkformechanicalunits(
    active_warrior: i16,
    origin: (i32, i32),
    incoming_d5: i16,
    services: &(impl SpatialServices + ?Sized),
) -> Result<bool, SpatialError> {
    let point = warrior_position(services, active_warrior)?;
    let mode = if warrior_word(services, active_warrior, 0xf6)? != 0 {
        2
    } else if warrior_word(services, active_warrior, 0xf4)? != 0 {
        1
    } else {
        incoming_d5
    };
    for column_offset in -1..=1 {
        for row_offset in -1..=1 {
            let row = origin
                .0
                .wrapping_add(i32::from(point.0))
                .wrapping_add(row_offset);
            let column = origin
                .1
                .wrapping_add(i32::from(point.1))
                .wrapping_add(column_offset);
            let body = services
                .map_word(row, column)
                .ok_or(SpatialError::MissingMapWord { row, column })?
                .wrapping_abs();
            if !(10..110).contains(&body) {
                continue;
            }
            let index = body - 10;
            if enemy_word(services, index, 0xe)? == warrior_word(services, active_warrior, 0x10)?
                || enemy_word(services, index, 0x110)? <= 0
            {
                continue;
            }
            if (mode == 1 || mode == 2) && enemy_byte(services, index, 0xb0)? == 1 {
                return Ok(true);
            }
            if mode == 2 && enemy_byte(services, index, 0xb1)? != 0 {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

pub trait RangeServices: SpatialServices {
    fn can_see(&self, from: Position, to: Position) -> Result<bool, SpatialError>;
}

pub struct RangeTables<'a, F> {
    pub spatial: SpatialTables<'a>,
    pub visibility: F,
}

impl<F> SpatialServices for RangeTables<'_, F> {
    fn warrior_position(&self, index: i16) -> Option<Position> {
        self.spatial.warrior_position(index)
    }
    fn enemy_position(&self, index: i16) -> Option<Position> {
        self.spatial.enemy_position(index)
    }
    fn warrior_byte(&self, index: i16, offset: usize) -> Option<u8> {
        self.spatial.warrior_byte(index, offset)
    }
    fn warrior_word(&self, index: i16, offset: usize) -> Option<i16> {
        self.spatial.warrior_word(index, offset)
    }
    fn enemy_byte(&self, index: i16, offset: usize) -> Option<u8> {
        self.spatial.enemy_byte(index, offset)
    }
    fn enemy_word(&self, index: i16, offset: usize) -> Option<i16> {
        self.spatial.enemy_word(index, offset)
    }
    fn map_word(&self, row: i32, column: i32) -> Option<i16> {
        self.spatial.map_word(row, column)
    }
}

impl<F: Fn(Position, Position) -> Result<bool, SpatialError>> RangeServices for RangeTables<'_, F> {
    fn can_see(&self, from: Position, to: Position) -> Result<bool, SpatialError> {
        (self.visibility)(from, to)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RangeInput {
    pub body: i8,
    pub faction: i8,
    pub mode: i8,
    pub highest_warrior: i8,
    pub enemy_count: i16,
    pub maximum: i16,
    pub closest: i16,
    pub visibility_fields: (i8, i8),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RangeWrite {
    DistanceSlot { body: i16, distance: i16 },
    SquaredDistance(f64),
    Distance(f64),
    RangeWord(i16),
    Closest(i16),
}

#[derive(Clone, Debug, PartialEq)]
pub struct RangePlan {
    pub distances: [i16; 112],
    pub matched_count: i16,
    pub closest: i16,
    pub last_range_work: Option<RangeWork>,
    pub writes: Vec<RangeWrite>,
}

fn append_work(plan: &mut RangePlan, work: RangeWork) {
    plan.writes.extend([
        RangeWrite::SquaredDistance(work.squared_distance),
        RangeWrite::Distance(work.distance),
        RangeWrite::RangeWord(work.range_word),
    ]);
    plan.last_range_work = Some(work);
}

fn admit(
    plan: &mut RangePlan,
    body: i16,
    work: RangeWork,
    matched: &mut i8,
) -> Result<(), SpatialError> {
    let slot = usize::try_from(body)
        .ok()
        .and_then(|index| plan.distances.get_mut(index))
        .ok_or(SpatialError::MissingDistanceSlot { body })?;
    *slot = work.range_word;
    plan.writes.push(RangeWrite::DistanceSlot {
        body,
        distance: work.range_word,
    });
    *matched = matched.wrapping_add(1);
    Ok(())
}

fn update_closest(plan: &mut RangePlan, work: RangeWork) {
    if work.distance < f64::from(plan.closest) {
        plan.closest = work.range_word;
        plan.writes.push(RangeWrite::Closest(plan.closest));
    }
}

/// getrange retains the different warrior and enemy distance comparisons.
pub fn getrange(
    input: RangeInput,
    services: &(impl RangeServices + ?Sized),
) -> Result<RangePlan, SpatialError> {
    if input.highest_warrior == i8::MAX || input.enemy_count > i16::from(i8::MAX) {
        return Err(SpatialError::NonterminatingByteLoop);
    }
    let body = i16::from(input.body);
    let origin = position(services, body)?;
    let mut plan = RangePlan {
        distances: [127; 112],
        matched_count: 0,
        closest: input.closest,
        last_range_work: None,
        writes: (0..112)
            .map(|body| RangeWrite::DistanceSlot {
                body,
                distance: 127,
            })
            .collect(),
    };
    let mut matched = 0_i8;
    let visibility =
        i16::from(input.visibility_fields.0) + i16::from(input.visibility_fields.1) > 0;
    let same_faction = input.mode == 4;
    for index in 0..=i16::from(input.highest_warrior) {
        if warrior_byte(services, index, 0x9df)? == 0 || warrior_word(services, index, 0x50)? <= 0 {
            continue;
        }
        if (warrior_word(services, index, 0x10)? == i16::from(input.faction)) != same_faction {
            continue;
        }
        let target = warrior_position(services, index)?;
        let (in_range, work) = checkrange(body, target, input.maximum, services)?;
        append_work(&mut plan, work);
        if in_range && (!visibility || services.can_see(origin, target)?) {
            admit(&mut plan, index, work, &mut matched)?;
        }
        update_closest(&mut plan, work);
    }
    for index in 0..input.enemy_count {
        if enemy_word(services, index, 0x110)? <= 0 {
            continue;
        }
        if (enemy_word(services, index, 0xe)? == i16::from(input.faction)) != same_faction
            || enemy_byte(services, index, 0xb0)? == 0xff
        {
            continue;
        }
        let target = enemy_position(services, index)?;
        let (_, work) = checkrange(body, target, input.maximum, services)?;
        append_work(&mut plan, work);
        let visible = !visibility || services.can_see(origin, target)?;
        if work.distance <= f64::from(input.maximum) && visible {
            admit(&mut plan, index.wrapping_add(10), work, &mut matched)?;
        }
        update_closest(&mut plan, work);
    }
    plan.matched_count = i16::from(matched);
    Ok(plan)
}
