use crate::RuleError;
use centurions_content::{
    records::{StoryRecord, StoryRecordMut},
    span,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Location {
    pub map: i32,
    pub x: i32,
    pub y: i32,
    pub level: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DestinationPlan {
    pub target: Location,
    pub reload: bool,
}

pub fn destination(
    record: &[u8],
    current: Location,
    combat: bool,
    chained: bool,
) -> Result<Option<DestinationPlan>, RuleError> {
    let record = StoryRecord(span(record, 0, StoryRecord::BYTES)?);
    if combat || chained {
        return Ok(None);
    }
    let target = Location {
        map: i32::from(record.destination_map()),
        x: i32::from(record.destination_x()),
        y: i32::from(record.destination_y()),
        level: record.destination_level(),
    };
    // The original compares the destination level as signed and the current level as unsigned.
    let reload = target.map != current.map
        || i16::from(record.destination_level() as i8) != i16::from(current.level);
    Ok((reload || target.x != current.x || target.y != current.y)
        .then_some(DestinationPlan { target, reload }))
}

pub fn persistent_record(
    record: &[u8],
    origin_x: i32,
    origin_y: i32,
) -> Result<[u8; StoryRecord::BYTES], RuleError> {
    let mut result: [u8; StoryRecord::BYTES] =
        span(record, 0, StoryRecord::BYTES)?.try_into().unwrap();
    StoryRecordMut(&mut result).set_destination_position(origin_x, origin_y);
    Ok(result)
}

pub fn disable_on_exhaustion(parameter_kind: i16, menu_active: i16) -> bool {
    parameter_kind == 0 && menu_active == 0
}
