use crate::RuleError;

pub fn elapsed_ticks(parameter: i16) -> Result<u32, RuleError> {
    if parameter <= 0 {
        return Err(RuleError::Unsupported {
            action: "story.delay",
            dependency: "retained default or unsigned long delay",
        });
    }
    Ok(parameter as u32 + 1)
}
