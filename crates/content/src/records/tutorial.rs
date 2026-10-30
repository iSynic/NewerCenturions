use crate::{ContentError, word};

/// The 1,000 pairs of rectangles loaded from Tutorial bytes 1000..17000.
pub struct TutorialRectangles<'a>(pub &'a [u8]);

impl TutorialRectangles<'_> {
    /// A word read at a signed index from the start of the tactical map. Moving
    /// a monster indexes the map linearly, and a negative index lands in this
    /// table, which lies just before the map.
    pub fn tactical_alias(&self, index: i32) -> Result<Option<i16>, ContentError> {
        let offset = 0x4085b8_i64 + i64::from(index) * 2 - 0x4046ec;
        if !(0..16000).contains(&offset) {
            return Ok(None);
        }
        word(self.0, offset as usize).map(Some)
    }
}
