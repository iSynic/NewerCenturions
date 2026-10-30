use crate::RuleError;
use centurions_content::{
    records::{StoryRecord, StoryRecordMut},
    span,
};

/// A place in the scenario: level, map and square.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Location {
    /// The map.
    pub map: i32,
    /// The column.
    pub x: i32,
    /// The row.
    pub y: i32,
    /// The scenario level.
    pub level: u8,
}

/// Where the party goes when a script ends, and whether the land must be reloaded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DestinationPlan {
    pub target: Location,
    pub reload: bool,
}

/// Where a finished script sends the party. Nothing happens in a battle or after a chained event,
/// or when the party is already there.
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
    // The destination level compares as signed and the current level as unsigned.
    let reload = target.map != current.map
        || i16::from(record.destination_level() as i8) != i16::from(current.level);
    Ok((reload || target.x != current.x || target.y != current.y)
        .then_some(DestinationPlan { target, reload }))
}

/// The record of a persistent event with its destination set to the square of the event.
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

/// Whether an exhausted event is disabled: only when no menu is active.
pub fn disable_on_exhaustion(parameter_kind: i16, menu_active: i16) -> bool {
    parameter_kind == 0 && menu_active == 0
}
