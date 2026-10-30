//! Big-endian word and long access to the game's record bytes.
//!
//! `word` and `long` report a short slice as `ContentError`. The remaining
//! helpers index directly and panic on a short slice, for record views whose
//! size is fixed by construction.
use crate::{ContentError, span};

/// Reads the signed big-endian 16-bit value at `offset`.
pub fn word(bytes: &[u8], offset: usize) -> Result<i16, ContentError> {
    let value = span(bytes, offset, 2)?;
    Ok(i16::from_be_bytes([value[0], value[1]]))
}

/// Reads the signed big-endian 32-bit value at `offset`.
pub fn long(bytes: &[u8], offset: usize) -> Result<i32, ContentError> {
    Ok(i32::from_be_bytes(
        span(bytes, offset, 4)?.try_into().unwrap(),
    ))
}

/// Reads a word from a record of known size; a short slice panics.
pub(crate) fn word_at(bytes: &[u8], offset: usize) -> i16 {
    i16::from_be_bytes([bytes[offset], bytes[offset + 1]])
}

/// Reads a long from a record of known size; a short slice panics.
pub(crate) fn long_at(bytes: &[u8], offset: usize) -> i32 {
    i32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

/// Stores a word into a record of known size; a short slice panics.
pub(crate) fn put_word(bytes: &mut [u8], offset: usize, value: i16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
}

/// Stores a long into a record of known size; a short slice panics.
pub(crate) fn put_long(bytes: &mut [u8], offset: usize, value: i32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_be_bytes());
}

/// Adds to a long with two's-complement wrapping, as the game's 32-bit add does.
pub(crate) fn add_long_wrapping(bytes: &mut [u8], offset: usize, amount: i32) {
    put_long(bytes, offset, long_at(bytes, offset).wrapping_add(amount));
}
