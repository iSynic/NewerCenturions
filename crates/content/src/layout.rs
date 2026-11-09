//! A scenario's Layout file: which land lies beside which.

/// Rows of lands in the Layout.
pub const LAYOUT_ROWS: usize = 8;
/// Columns of lands in the Layout.
pub const LAYOUT_COLUMNS: usize = 16;

/// The lands of a scenario level laid out on a grid of eight rows by sixteen columns. A cell holds a
/// land number, -1 for land 0, or 0 where no land lies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layout([i16; LAYOUT_ROWS * LAYOUT_COLUMNS]);

impl Layout {
    /// Reads the file as big-endian words, row by row. A file shorter than 256 bytes leaves the
    /// remaining cells empty.
    pub fn read(bytes: &[u8]) -> Self {
        let mut cells = [0; LAYOUT_ROWS * LAYOUT_COLUMNS];
        for (cell, word) in cells.iter_mut().zip(bytes.as_chunks::<2>().0) {
            *cell = i16::from_be_bytes(*word);
        }
        Self(cells)
    }

    /// The cell at a row and column, or `None` outside the grid.
    pub fn cell(&self, row: i16, column: i16) -> Option<i16> {
        let row = usize::try_from(row).ok().filter(|&r| r < LAYOUT_ROWS)?;
        let column = usize::try_from(column)
            .ok()
            .filter(|&c| c < LAYOUT_COLUMNS)?;
        Some(self.0[row * LAYOUT_COLUMNS + column])
    }

    /// The row and column of the first cell, scanning row by row, that holds `value`.
    pub fn find(&self, value: i16) -> Option<(i16, i16)> {
        let at = self.0.iter().position(|&cell| cell == value)?;
        Some(((at / LAYOUT_COLUMNS) as i16, (at % LAYOUT_COLUMNS) as i16))
    }
}
