use crate::{ContentError, word};

/// A 42-byte event record: location header, eight story opcodes with their
/// parameters, destination level and dispatch stamp. Its layout differs from
/// Realmz events.
pub struct StoryRecord<'a>(pub &'a [u8]);

impl StoryRecord<'_> {
    /// Number of actions an event holds.
    pub const ACTIONS: usize = 8;

    /// Action `cursor` of the event: its opcode and parameter.
    pub fn instruction(&self, cursor: usize) -> Result<(i16, i16), ContentError> {
        let opcode = word(self.0, 8 + cursor * 2)?;
        let parameter = word(self.0, 24 + cursor * 2)?;
        Ok((opcode, parameter))
    }

    /// The stamp that marks the event as dispatched, when the record is long enough.
    pub fn dispatch_stamp(&self) -> Option<u8> {
        self.0.get(41).copied()
    }
}

/// An event record as a view that can change it.
pub struct StoryRecordMut<'a>(pub &'a mut [u8]);

impl StoryRecordMut<'_> {
    /// A suspended death-script caller keeps its own actions and parameters; the header,
    /// destination level and dispatch stamp follow the shared record.
    pub fn copy_return_header(&mut self, shared: &[u8]) {
        self.0[..8].copy_from_slice(&shared[..8]);
        self.0[40..42].copy_from_slice(&shared[40..42]);
    }
    /// A suspended battle caller keeps its own actions, parameters and destination level; the
    /// header and dispatch stamp follow the shared record.
    pub fn copy_battle_return_header(&mut self, shared: &[u8]) {
        self.0[..8].copy_from_slice(&shared[..8]);
        self.0[41] = shared[41];
    }
    /// Loading or branching to another event keeps this event's eight-byte header.
    pub fn replace_body(&mut self, replacement: &[u8]) {
        self.0[8..].copy_from_slice(&replacement[8..]);
    }
    /// Sets the map, column and row the event's destination names.
    pub fn relocate(&mut self, map: i32, x: i32, y: i32) {
        self.0[4] = map as u8;
        self.0[5] = x as u8;
        self.0[6] = y as u8;
    }
    /// Sets the dispatch stamp.
    pub fn write_dispatch_stamp(&mut self, stamp: u8) {
        if let Some(byte) = self.0.get_mut(41) {
            *byte = stamp;
        }
    }
}

/// The EXTRA record of a travel opcode: where the party goes and what it hears
/// and reads on arrival.
pub struct TravelExtra<'a>(pub &'a [u8]);

impl TravelExtra<'_> {
    /// The map to go to; a negative value stays on the current map.
    pub fn map(&self, current: i32) -> Result<i32, ContentError> {
        Ok(Self::keep_negative(word(self.0, 0)?, current))
    }
    /// The column to go to; a negative value keeps the current one.
    pub fn x(&self, current: i32) -> Result<i32, ContentError> {
        Ok(Self::keep_negative(word(self.0, 2)?, current))
    }
    /// The row to go to; a negative value keeps the current one.
    pub fn y(&self, current: i32) -> Result<i32, ContentError> {
        Ok(Self::keep_negative(word(self.0, 4)?, current))
    }
    /// The stored coordinate, or `current` when it is negative.
    fn keep_negative(value: i16, current: i32) -> i32 {
        if value < 0 { current } else { i32::from(value) }
    }
    /// Sound played on arrival.
    pub fn sound(&self) -> Result<i16, ContentError> {
        word(self.0, 6)
    }
    /// Dialogue shown on arrival.
    pub fn dialogue(&self) -> Result<i16, ContentError> {
        word(self.0, 8)
    }
}

/// The EXTRA record of an opcode that works on a rectangle of cells of a map.
pub struct ObjectRectangleExtra<'a>(pub &'a [u8]);

impl ObjectRectangleExtra<'_> {
    /// The map the rectangle lies on.
    pub fn map(&self) -> Result<i32, ContentError> {
        Ok(i32::from(word(self.0, 0)?))
    }
    /// The rectangle's first corner.
    pub fn first(&self) -> Result<(i32, i32), ContentError> {
        Ok((i32::from(word(self.0, 2)?), i32::from(word(self.0, 6)?)))
    }
    /// The rectangle's last corner.
    pub fn last(&self) -> Result<(i32, i32), ContentError> {
        Ok((i32::from(word(self.0, 4)?), i32::from(word(self.0, 8)?)))
    }
}

/// How a story changes the party's fatigue.
pub enum FatigueChange {
    Keep,
    Set(i16),
    Percent(i16),
}

impl FatigueChange {
    /// Reads the change an EXTRA record asks for.
    pub fn decode(extra: &[u8]) -> Result<Self, ContentError> {
        Ok(match word(extra, 0)? {
            1 => Self::Set(135),
            2 => Self::Set(4),
            3 => Self::Percent(word(extra, 4)?),
            _ => Self::Keep,
        })
    }
}

/// What a story forbids the party to recover; each restriction is nonzero when it
/// is in force.
pub struct RecoveryRestrictions {
    /// The party cannot use Psi.
    pub psi: u8,
    /// Monsters cannot cast scans.
    pub enemy_spells: u8,
    /// Energy does not recharge.
    pub energy: u8,
}

impl RecoveryRestrictions {
    /// Each restriction is the low byte of its EXTRA word, not a boolean.
    pub fn decode(extra: &[u8]) -> Self {
        Self {
            psi: extra[1],
            enemy_spells: extra[3],
            energy: extra[5],
        }
    }
}
