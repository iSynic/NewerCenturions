use crate::RuleError;
use centurions_content::{span, word};

pub fn edit(schedule: &mut [u8], extra: &[u8], current_day: i16) -> Result<(), RuleError> {
    let extra = span(extra, 0, 10)?;
    let index = usize::from(word(extra, 0)? as u16);
    let chance = word(extra, 2)?;
    let interval = word(extra, 4)?;
    let reset = word(extra, 6)?;
    let addition = word(extra, 8)?;
    if !schedule.len().is_multiple_of(40) {
        return Err(RuleError::Invalid("invalid schedule record size".into()));
    }
    let offset = index * 40;
    let record = schedule
        .get_mut(offset..offset + 40)
        .ok_or(RuleError::Unsupported {
            action: "story.schedule_edit",
            dependency: "scheduled record outside the loaded table",
        })?;
    let mut day = word(record, 0)?;
    if chance >= 0 {
        record[4..6].copy_from_slice(&chance.to_be_bytes());
    }
    if interval >= 0 {
        record[2..4].copy_from_slice(&interval.to_be_bytes());
    }
    if reset != 0 {
        day = current_day;
    }
    if addition >= 0 {
        day = day.wrapping_add(addition);
    }
    record[..2].copy_from_slice(&day.to_be_bytes());
    Ok(())
}
