//! The captions of a question's two answers, held in the scenario's option texts.

use crate::mission::StoryTables;

/// Bytes the game keeps for one caption: a length byte and up to 255 characters.
const CAPTION_BYTES: usize = 256;
/// Bytes of one record of `Data OD`.
const OPTION_RECORD_BYTES: usize = 25;
/// Bytes of one record of `Data SD2`, which stands in when the scenario has no `Data OD`.
const DIALOGUE_RECORD_BYTES: usize = 256;

/// One answer's caption as the game holds it: a Pascal string in a 256-byte buffer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuestionCaption([u8; CAPTION_BYTES]);

impl QuestionCaption {
    /// The caption the game shows when the option texts give nothing to read: the text it
    /// preloads into the buffer.
    fn preloaded(text: &str) -> Self {
        let mut buffer = [0; CAPTION_BYTES];
        let (bytes, _, _) = encoding_rs::MACINTOSH.encode(text);
        let length = bytes.len().min(CAPTION_BYTES - 1);
        buffer[0] = length as u8;
        buffer[1..=length].copy_from_slice(&bytes[..length]);
        Self(buffer)
    }

    /// The caption's text.
    pub fn text(&self) -> String {
        let length = usize::from(self.0[0]);
        encoding_rs::MACINTOSH
            .decode(&self.0[1..=length])
            .0
            .into_owned()
    }

    /// The character whose lower-case form is the answer's key: the byte after the length, even
    /// when the caption is empty.
    pub fn key_byte(&self) -> u8 {
        self.0[1]
    }
}

impl StoryTables {
    /// The captions of the two answers of a question whose extra names them by number. Both are
    /// read from the one option file: `Data OD` at 25 bytes a record, or else `Data SD2` at 256
    /// bytes a record. A record the file does not reach leaves the preloaded caption in place, and
    /// so does every record after a seek the file refuses: an offset below zero or beyond the
    /// end of the file marks the open file as failed, and a failed file neither seeks nor reads
    /// again.
    ///
    /// Original: question2
    pub fn question_captions(&self, labels: (i16, i16), preloaded: &str) -> [QuestionCaption; 2] {
        let (file, record_bytes) = match &self.option_texts {
            Some(options) => (options.as_slice(), OPTION_RECORD_BYTES),
            None => (self.dialogue.as_slice(), DIALOGUE_RECORD_BYTES),
        };
        let mut failed = false;
        [labels.0, labels.1].map(|label| {
            let mut caption = QuestionCaption::preloaded(preloaded);
            let number = label.wrapping_abs();
            // The narrow records' offset wraps to 16 bits before it is sign-extended.
            let offset = if record_bytes == OPTION_RECORD_BYTES {
                i32::from(number.wrapping_mul(OPTION_RECORD_BYTES as i16))
            } else {
                i32::from(number).wrapping_shl(8)
            };
            let available = usize::try_from(offset)
                .ok()
                .and_then(|offset| file.get(offset..));
            failed |= available.is_none();
            if let Some(available) = available.filter(|_| !failed) {
                let read = available.len().min(record_bytes);
                caption.0[..read].copy_from_slice(&available[..read]);
            }
            caption
        })
    }
}
