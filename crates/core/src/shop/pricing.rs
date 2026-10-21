//! itemcost 0x13c018 rounds each intermediate result to binary32 before the
//! final signed LONG conversion. Integer percentage math is not equivalent.
use crate::RuleError;

pub(super) fn percentage_price(cost: i32, percent: i16, selling: bool) -> Result<i32, RuleError> {
    let absolute = (cost as f32).abs();
    let percent = if selling { percent.min(100) } else { percent };
    let scaled = (f64::from(absolute) * f64::from(percent)) as f32;
    let divided = (f64::from(scaled) / 100.0) as f32;
    let wide = f64::from(divided);
    if !wide.is_finite() || wide < f64::from(i32::MIN) || wide > f64::from(i32::MAX) {
        return Err(RuleError::Unsupported {
            action: "economy.itemcost",
            dependency: "SANE FLOAT-to-LONG exception result outside signed 32-bit range",
        });
    }
    Ok(divided as i32)
}
