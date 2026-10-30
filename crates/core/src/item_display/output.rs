//! Drawing output of the inventory and estimate panels. Addresses in these
//! effects name the original text and data for comparison; they are never read
//! as game state.
use crate::{RuleError, ammunition::LoadedItems};

/// A length-prefixed string as the game stores text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PascalString(Vec<u8>);

impl Default for PascalString {
    /// The empty string.
    fn default() -> Self {
        Self(vec![0])
    }
}

impl PascalString {
    /// Reads a length-prefixed string from the start of a byte buffer.
    pub fn from_raw(bytes: &[u8]) -> Result<Self, RuleError> {
        let length = bytes
            .first()
            .copied()
            .ok_or_else(|| RuleError::Invalid("missing Pascal string length".into()))?;
        let raw = bytes.get(..usize::from(length) + 1).ok_or_else(|| {
            RuleError::Invalid("Pascal string exceeds its supplied byte buffer".into())
        })?;
        Ok(Self(raw.to_vec()))
    }

    /// Builds a length-prefixed string from its text; at most 255 bytes.
    pub fn from_payload(bytes: &[u8]) -> Result<Self, RuleError> {
        let length = u8::try_from(bytes.len())
            .map_err(|_| RuleError::Invalid("Pascal string exceeds 255 bytes".into()))?;
        let mut raw = Vec::with_capacity(bytes.len() + 1);
        raw.push(length);
        raw.extend_from_slice(bytes);
        Ok(Self(raw))
    }

    /// The string with its length byte.
    pub fn raw(&self) -> &[u8] {
        &self.0
    }

    /// The text without its length byte.
    pub fn payload(&self) -> &[u8] {
        &self.0[1..]
    }
}

/// A rectangle in panel coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BodyRect {
    pub top: i16,
    pub left: i16,
    pub bottom: i16,
    pub right: i16,
}

/// The arithmetic an estimate used: the damage figures are worked out in single precision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArithmeticQualification {
    SourceDerivedBinary32,
}

/// The QuickDraw operations emitted by the item panels.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Draw {
    SetPort(u32),
    ForeColor(i32),
    MoveTo { horizontal: i16, vertical: i16 },
    BackPixelPattern(u32),
    GetColorIcon(i16),
    PlotColorIcon { handle: u32, rect: BodyRect },
    DisposeColorIcon(u32),
    TextFont(i16),
    TextSize(i16),
    DrawString(PascalString),
    GetPicture(i16),
    DrawPicture { handle: u32, bounds: BodyRect },
}

/// One drawing step of an item panel: a QuickDraw operation, a text setting, a number or string
/// conversion, or a block of text in a rectangle.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PanelEffect {
    Drawing(Draw),
    TextMode(i16),
    /// The whole text-face value as the game stores it.
    TextFaceSlot(u16),
    RgbForeColor {
        pointer: u32,
        bytes: [u8; 6],
    },
    RgbBackColor {
        pointer: u32,
        bytes: [u8; 6],
    },
    NumberToString {
        value: i32,
        destination: u32,
        result: PascalString,
    },
    IndexedString {
        resource_id: i16,
        index: i16,
        destination: u32,
        result: PascalString,
    },
    TextBox {
        pointer: u32,
        length: i32,
        bytes: Vec<u8>,
        rectangle_pointer: u32,
        rectangle: BodyRect,
        alignment: i16,
    },
}

/// Everything a panel draws, in order, with the loaded item definitions an estimate leaves behind.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PanelPlan {
    pub effects: Vec<PanelEffect>,
    /// Present only when the estimate's damage calculation is reached.
    pub arithmetic: Option<ArithmeticQualification>,
    /// The estimate's loaded item, ammunition and weapon definitions after
    /// calculation. Item detail panels retain none.
    pub loaded_items: Option<LoadedItems>,
}
