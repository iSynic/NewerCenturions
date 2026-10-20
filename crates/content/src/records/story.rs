use crate::{ContentError, word};

/// Original 42-byte event record (newland). This differs from Realmz events.
pub struct StoryRecord<'a>(pub &'a [u8]);

impl StoryRecord<'_> {
    pub const ACTIONS: usize = 8;

    pub fn instruction(&self, cursor: usize) -> Result<(i16, i16), ContentError> {
        let opcode = word(self.0, 8 + cursor * 2)?;
        let parameter = word(self.0, 24 + cursor * 2)?;
        Ok((opcode, parameter))
    }

    pub fn dispatch_stamp(&self) -> Option<u8> {
        self.0.get(41).copied()
    }
}

pub struct StoryRecordMut<'a>(pub &'a mut [u8]);

impl StoryRecordMut<'_> {
    /// loaddoor2/branch replacement retains the original eight-byte header.
    pub fn replace_body(&mut self, replacement: &[u8]) {
        self.0[8..].copy_from_slice(&replacement[8..]);
    }
    pub fn relocate(&mut self, map: i32, x: i32, y: i32) {
        self.0[4] = map as u8;
        self.0[5] = x as u8;
        self.0[6] = y as u8;
    }
    pub fn write_dispatch_stamp(&mut self, stamp: u8) {
        if let Some(byte) = self.0.get_mut(41) {
            *byte = stamp;
        }
    }
}

pub struct TravelExtra<'a>(pub &'a [u8]);

impl TravelExtra<'_> {
    pub fn map(&self, current: i32) -> Result<i32, ContentError> {
        Ok(Self::keep_negative(word(self.0, 0)?, current))
    }
    pub fn x(&self, current: i32) -> Result<i32, ContentError> {
        Ok(Self::keep_negative(word(self.0, 2)?, current))
    }
    pub fn y(&self, current: i32) -> Result<i32, ContentError> {
        Ok(Self::keep_negative(word(self.0, 4)?, current))
    }
    fn keep_negative(value: i16, current: i32) -> i32 {
        if value < 0 { current } else { i32::from(value) }
    }
    pub fn sound(&self) -> Result<i16, ContentError> {
        word(self.0, 6)
    }
    pub fn dialogue(&self) -> Result<i16, ContentError> {
        word(self.0, 8)
    }
}

pub struct ObjectRectangleExtra<'a>(pub &'a [u8]);

impl ObjectRectangleExtra<'_> {
    pub fn map(&self) -> Result<i32, ContentError> {
        Ok(i32::from(word(self.0, 0)?))
    }
    pub fn first(&self) -> Result<(i32, i32), ContentError> {
        Ok((i32::from(word(self.0, 2)?), i32::from(word(self.0, 6)?)))
    }
    pub fn last(&self) -> Result<(i32, i32), ContentError> {
        Ok((i32::from(word(self.0, 4)?), i32::from(word(self.0, 8)?)))
    }
}

pub enum FatigueChange {
    Keep,
    Set(i16),
    Percent(i16),
}

impl FatigueChange {
    pub fn decode(extra: &[u8]) -> Result<Self, ContentError> {
        Ok(match word(extra, 0)? {
            1 => Self::Set(135),
            2 => Self::Set(4),
            3 => Self::Percent(word(extra, 4)?),
            _ => Self::Keep,
        })
    }
}

pub struct RecoveryRestrictions {
    pub psi: u8,
    pub enemy_spells: u8,
    pub energy: u8,
}

impl RecoveryRestrictions {
    /// newland 0x143ef6 uses the low byte of each EXTRA word, not a boolean.
    pub fn decode(extra: &[u8]) -> Self {
        Self {
            psi: extra[1],
            enemy_spells: extra[3],
            energy: extra[5],
        }
    }
}
