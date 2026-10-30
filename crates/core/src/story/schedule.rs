use crate::RuleError;
use centurions_content::records::{ScheduleEditExtra, ScheduleEntry, ScheduleEntryMut};

/// Edits a calendar schedule entry: the chance and interval are replaced when given, and the due
/// day is reset to today and/or moved on by an amount.
pub fn edit(schedule: &mut [u8], extra: &[u8], current_day: i16) -> Result<(), RuleError> {
    let extra = ScheduleEditExtra::new(extra)?;
    let index = usize::from(extra.entry()?);
    let chance = extra.chance()?;
    let interval = extra.interval()?;
    let reset = extra.reset()?;
    let addition = extra.addition()?;
    if !schedule.len().is_multiple_of(ScheduleEntry::BYTES) {
        return Err(RuleError::Invalid("invalid schedule record size".into()));
    }
    let mut record = ScheduleEntryMut::in_table(schedule, index).ok_or(RuleError::Unsupported {
        action: "story.schedule_edit",
        dependency: "scheduled record outside the loaded table",
    })?;
    let mut day = record.due_day()?;
    if chance >= 0 {
        record.set_chance(chance);
    }
    if interval >= 0 {
        record.set_interval(interval);
    }
    if reset != 0 {
        day = current_day;
    }
    if addition >= 0 {
        day = day.wrapping_add(addition);
    }
    record.set_due_day(day);
    Ok(())
}
