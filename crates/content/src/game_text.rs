//! The game's own strings from its DATA resource, decoded as the CODE 1 startup
//! does.

use crate::{Content, ContentError, read_resource, span};
use std::path::Path;

/// The game's text and tables as `DATA` initialises them in memory when the
/// program starts.
pub struct GameText {
    image: Vec<u8>,
    below: usize,
}

impl GameText {
    /// Encodes `text` in the game's Macintosh character set; control characters and
    /// anything the set lacks are refused.
    pub fn encode(text: &str) -> Result<Vec<u8>, ContentError> {
        let (bytes, _, invalid) = encoding_rs::MACINTOSH.encode(text);
        if invalid || bytes.iter().any(|byte| *byte < 32 || *byte == 127) {
            return Err(ContentError::Invalid(
                "text does not fit the original character set".into(),
            ));
        }
        Ok(bytes.into_owned())
    }

    /// Builds the game's initial `DATA` from the executable's `CODE 0` and `DATA 0`
    /// resources.
    pub fn load(content: &Content, root: &Path) -> Result<Self, ContentError> {
        let find = |kind: &str| {
            content
                .resources
                .iter()
                .find(|r| r.r#type == kind && r.id == 0 && r.original.ends_with("/New Centurions"))
                .ok_or_else(|| ContentError::Invalid(format!("executable {kind} 0 missing")))
        };
        let header = read_resource(root, find("CODE")?)?;
        let data = read_resource(root, find("DATA")?)?;
        Self::decode(&header, &data)
    }

    /// Replays the `DATA` initialiser against the layout `CODE 0` declares.
    pub fn decode(header: &[u8], data: &[u8]) -> Result<Self, ContentError> {
        let above = u32::from_be_bytes(span(header, 0, 4)?.try_into().unwrap()) as usize;
        let below = u32::from_be_bytes(span(header, 4, 4)?.try_into().unwrap()) as usize;
        let length = above
            .checked_add(below)
            .filter(|n| *n <= 16 * 1024 * 1024)
            .ok_or_else(|| ContentError::Invalid("executable DATA exceeds memory limit".into()))?;
        let mut image = vec![0; length];
        let mut cursor = 4;
        for _ in 0..3 {
            let offset = i32::from_be_bytes(span(data, cursor, 4)?.try_into().unwrap());
            cursor += 4;
            let mut position = usize::try_from(below as i64 + i64::from(offset))
                .map_err(|_| ContentError::Invalid("negative DATA initialization offset".into()))?;
            loop {
                let opcode = span(data, cursor, 1)?[0];
                cursor += 1;
                if opcode == 0 {
                    break;
                }
                let bytes = if opcode & 0x80 != 0 {
                    let count = usize::from(opcode & 0x7f) + 1;
                    let bytes = span(data, cursor, count)?.to_vec();
                    cursor += count;
                    bytes
                } else if opcode & 0x40 != 0 {
                    position = position
                        .checked_add(usize::from(opcode & 0x3f) + 1)
                        .ok_or_else(|| ContentError::Invalid("DATA offset overflow".into()))?;
                    span(&image, position, 0)?;
                    continue;
                } else if opcode & 0x20 != 0 {
                    let byte = span(data, cursor, 1)?[0];
                    cursor += 1;
                    vec![byte; usize::from(opcode & 0x1f) + 2]
                } else if opcode & 0x10 != 0 {
                    vec![255; usize::from(opcode & 15) + 1]
                } else if opcode <= 2 {
                    position += 4;
                    let mut bytes = vec![255; usize::from(3 - opcode)];
                    bytes.extend_from_slice(span(data, cursor, usize::from(opcode + 1))?);
                    cursor += usize::from(opcode + 1);
                    bytes
                } else if opcode == 3 || opcode == 4 {
                    span(&image, position, 8)?;
                    image[position..position + 2].copy_from_slice(&[0xa9, 0xf0]);
                    let skip = if opcode == 3 { 4 } else { 3 };
                    let count = if opcode == 3 { 2 } else { 3 };
                    image[position + skip..position + skip + count]
                        .copy_from_slice(span(data, cursor, count)?);
                    cursor += count;
                    image[position + 7] = span(data, cursor, 1)?[0];
                    cursor += 1;
                    position += 8;
                    continue;
                } else {
                    return Err(ContentError::Invalid(format!(
                        "unknown DATA initializer {opcode}"
                    )));
                };
                span(&image, position, bytes.len())?;
                image[position..position + bytes.len()].copy_from_slice(&bytes);
                position += bytes.len();
            }
        }
        Ok(Self { image, below })
    }

    /// Reads the string at `address`, a length-prefixed string when `pascal` is set
    /// and otherwise a zero-terminated one.
    pub fn text(&self, address: u32, pascal: bool) -> Result<String, ContentError> {
        let offset = usize::try_from(i64::from(address) - 0x400000 + self.below as i64)
            .map_err(|_| ContentError::Invalid("negative native text address".into()))?;
        let bytes = if pascal {
            let length = usize::from(span(&self.image, offset, 1)?[0]);
            span(&self.image, offset + 1, length)?
        } else {
            let remaining = self
                .image
                .get(offset..)
                .ok_or_else(|| ContentError::Invalid("native text address outside DATA".into()))?;
            let length = remaining
                .iter()
                .take(4096)
                .position(|b| *b == 0)
                .ok_or_else(|| ContentError::Invalid("unterminated native text".into()))?;
            &remaining[..length]
        };
        Ok(encoding_rs::MACINTOSH.decode(bytes).0.into_owned())
    }

    /// Reads `length` raw bytes at `address`.
    pub fn bytes(&self, address: u32, length: usize) -> Result<&[u8], ContentError> {
        let offset = usize::try_from(i64::from(address) - 0x400000 + self.below as i64)
            .map_err(|_| ContentError::Invalid("negative native DATA address".into()))?;
        span(&self.image, offset, length)
    }
}
