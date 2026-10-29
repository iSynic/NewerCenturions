use crate::RuleError;
use centurions_content::word;

/// newland63's WORD writes, before its timeclick(0) call at 0x146202.
/// EXTRA holds mode, day, hour and minute in its first four WORDs.
pub fn alter(clock: (i16, i16, i16), extra: &[u8]) -> Result<(i16, i16, i16), RuleError> {
    let (day, hour, minute) = (word(extra, 2)?, word(extra, 4)?, word(extra, 6)?);
    let (mut d, mut h, mut m) = clock;
    match word(extra, 0)? {
        1 => (d, h, m) = (day, hour, minute),
        2 => {
            (d, h) = (d.wrapping_add(day), h.wrapping_add(hour));
            if h > 23 {
                (d, h) = (d.wrapping_add(1), h - 24);
            }
            m = m.wrapping_add(minute);
            if m > 59 {
                (h, m) = (h.wrapping_add(1), m - 60);
            }
        }
        _ => {}
    }
    Ok((d, h, m))
}
