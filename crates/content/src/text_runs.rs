//! The style runs of a movie's text: where the font, size and colour change.
use crate::{ContentError, bytes::word, span};

/// Bytes in the run count that opens a style resource.
const COUNT_BYTES: usize = 2;
/// Bytes in one run.
const RUN_BYTES: usize = 20;

/// One stretch of a movie's text drawn in a single style, starting at a
/// character of the text and lasting until the next run begins.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextRun {
    /// The first character of the text the run covers.
    pub start: usize,
    /// The height of a line of the run.
    pub line_height: i16,
    /// The font family the run names.
    pub font: i16,
    /// Whether the run is bold.
    pub bold: bool,
    /// The point size of the run.
    pub size: i16,
    /// The colour of the run, red, green and blue as 16-bit values.
    pub color: [u16; 3],
}

/// Reads the runs of a style resource, in the order the resource lists them.
///
/// Original: movie
pub fn text_runs(bytes: &[u8]) -> Result<Vec<TextRun>, ContentError> {
    let count = usize::from(u16::from_be_bytes(
        span(bytes, 0, COUNT_BYTES)?.try_into().unwrap(),
    ));
    (0..count)
        .map(|index| {
            let run = span(bytes, COUNT_BYTES + index * RUN_BYTES, RUN_BYTES)?;
            let color = |at: usize| u16::from_be_bytes([run[at], run[at + 1]]);
            Ok(TextRun {
                start: u32::from_be_bytes(run[..4].try_into().unwrap()) as usize,
                line_height: word(run, 4)?,
                font: word(run, 8)?,
                bold: run[10] & 1 != 0,
                size: word(run, 12)?,
                color: [color(14), color(16), color(18)],
            })
        })
        .collect()
}
