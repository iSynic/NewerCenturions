use crate::{ContentError, word};

/// A map note, the 340-byte record of `Data MD2` that the Maps and Notes menu shows.
pub struct MapNote<'a>(pub &'a [u8]);

/// One mark of a map note: the icon, and the column and row of the cell it stands on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NoteMark {
    /// The cicn drawn.
    pub icon: i16,
    /// The column of the cell, counted in note cells from the left edge of the picture.
    pub column: i16,
    /// The row of the cell, counted in note cells from the top edge of the picture.
    pub row: i16,
}

impl MapNote<'_> {
    /// Size of the record.
    pub const BYTES: usize = 340;
    /// The marks a note holds.
    pub const MARKS: usize = 10;

    /// Mark `index` (0 to 9); an icon of 0 means the slot is empty.
    pub fn mark(&self, index: usize) -> Result<NoteMark, ContentError> {
        if index >= Self::MARKS {
            return Err(ContentError::Invalid(format!("map note mark {index}")));
        }
        let at = index * 6;
        Ok(NoteMark {
            icon: word(self.0, at)?,
            column: word(self.0, at + 2)?,
            row: word(self.0, at + 4)?,
        })
    }
    /// The column of the map where the drawn section starts.
    pub fn start_x(&self) -> Result<i16, ContentError> {
        word(self.0, 60)
    }
    /// The row of the map where the drawn section starts.
    pub fn start_y(&self) -> Result<i16, ContentError> {
        word(self.0, 62)
    }
    /// The land the map section is taken from.
    pub fn level(&self) -> Result<i16, ContentError> {
        word(self.0, 64)
    }
    /// The picture shown instead of a map section; 0 shows the map.
    pub fn picture(&self) -> Result<i16, ContentError> {
        word(self.0, 66)
    }
    /// The side in pixels of one map cell in the drawn section.
    pub fn icon_size(&self) -> Result<i16, ContentError> {
        word(self.0, 68)
    }
    /// A nonzero value names the movie shown in place of everything else.
    pub fn show(&self) -> Result<i16, ContentError> {
        word(self.0, 70)
    }
    /// The rectangle (top, left, bottom, right) that overrides the picture's size.
    pub fn picture_rect(&self) -> Result<[i16; 4], ContentError> {
        Ok([
            word(self.0, 76)?,
            word(self.0, 78)?,
            word(self.0, 80)?,
            word(self.0, 82)?,
        ])
    }
}
