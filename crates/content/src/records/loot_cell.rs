//! What the booty screen draws around one cell's item icon.
//!
//! The definition is the item record (`item`) loaded for the cell's id; the
//! instance is the cell's 22-byte booty record. Positions are offsets from the
//! 32x32 icon rectangle, sums wrap at word width, and each label is one of the
//! game's own Pascal strings, identified by its place in the executable's data.
use crate::{ContentError, InventoryRecord, span, word};

/// The pen colours the booty screen uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellColor {
    /// blackColor 33.
    Black,
    /// redColor 205.
    Red,
    /// greenColor 341.
    Green,
    /// blueColor 409.
    Blue,
}

/// Where a run of text starts, relative to the cell's item icon.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellAnchor {
    /// The icon's right edge plus one, top plus `y`.
    Right,
    /// The icon's left edge plus `x`, bottom plus `y`.
    Below,
}

/// One pen move followed by the strings drawn from there.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CellText {
    pub anchor: CellAnchor,
    pub x: i16,
    pub y: i16,
    pub size: u8,
    pub color: CellColor,
    /// The value drawn as a number; `signed` prefixes "+" to positive values.
    pub value: Option<(i16, bool)>,
    /// Pascal strings drawn after the value, at the pen position it left.
    pub labels: Vec<u32>,
}

/// Everything the booty screen draws around one cell's item icon.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LootCell {
    /// For a weapon holding ammunition, the cicn drawn in the 16x16 rectangle at
    /// the icon's right edge plus one, top plus 14.
    pub caliber_icon: Option<i16>,
    pub texts: Vec<CellText>,
}

/// Caliber cicns, indexed by definition word 32 minus 8.
const CALIBER_ICONS: [i16; 25] = [
    0x83d, 0x83d, 0x838, 0x83e, 0x83e, 0x838, 0x841, 0x835, 0x837, 0x836, 0x837, 0x834, 0x838,
    0x836, 0x838, 0x83c, 0x838, 0x839, 0x838, 0x83a, 0x838, 0x83b, 0x838, 0x840, 0x83f,
];

/// Reserve ammunition labels, indexed by definition word 34. A zero entry
/// draws no label.
const RESERVE_LABELS: [u32; 33] = [
    0, 0x4033ec, 0x4033e4, 0x4033dc, 0x4033d4, 0, 0x4033cc, 0x4033c6, 0x4033c0, 0x4033ba, 0,
    0x4033b0, 0x4033a6, 0, 0x40339c, 0x403394, 0x40338a, 0x403386, 0x40337c, 0x403372, 0, 0x40336a,
    0, 0x403362, 0x403356, 0x40334e, 0, 0x40334a, 0, 0x403342, 0, 0x40333a, 0x403334,
];

/// The cicn drawn at 8x8 for each step of loot quality.
pub const QUALITY_ICON: i16 = 0xd0;

/// Upper-left corner of quality step `step`, relative to the icon's top-left.
/// Each step moves nine pixels right; steps 7, 14, 21 and 28 first return 63
/// left and move 8 down.
pub fn quality_position(step: usize) -> (i16, i16) {
    let wraps = [7, 14, 21, 28].iter().filter(|n| step >= **n).count() as i16;
    let column = step as i16 - wraps * 7;
    (-2 + column * 9, -10 + wraps * 8)
}

/// What the booty screen draws around one cell's item icon.
///
/// Original: showp
pub fn loot_cell(definition: &[u8], record: &InventoryRecord) -> Result<LootCell, ContentError> {
    span(definition, 0, 150)?;
    let byte = |offset: usize| i16::from(definition[offset] as i8);
    let field = |offset: usize| i16::from(record.0[offset] as i8);
    let quantity = word(&record.0, 2)?;
    let mut cell = LootCell::default();
    let count = |cell: &mut LootCell| {
        cell.texts.push(CellText {
            anchor: CellAnchor::Right,
            x: 0,
            y: 8,
            size: 10,
            color: CellColor::Black,
            value: Some((quantity, false)),
            labels: vec![],
        })
    };
    let reserve = word(definition, 34)?;
    if word(&record.0, 4)? != 0 {
        let caliber = word(definition, 32)?.wrapping_sub(8);
        cell.caliber_icon = Some(
            usize::try_from(caliber)
                .ok()
                .and_then(|index| CALIBER_ICONS.get(index))
                .copied()
                .unwrap_or(0x838),
        );
        count(&mut cell);
    } else if reserve != 0 {
        count(&mut cell);
        let label = usize::try_from(reserve)
            .ok()
            .and_then(|index| RESERVE_LABELS.get(index))
            .copied()
            .unwrap_or(0);
        cell.texts.push(CellText {
            anchor: CellAnchor::Below,
            x: 1,
            y: 12,
            size: 10,
            color: CellColor::Black,
            value: None,
            labels: [label].into_iter().filter(|a| *a != 0).collect(),
        });
    } else if quantity != 0 {
        count(&mut cell);
    }
    let mut below = |y: i16, color: CellColor, value: Option<(i16, bool)>, labels: Vec<u32>| {
        cell.texts.push(CellText {
            anchor: CellAnchor::Below,
            x: -1,
            y,
            size: 9,
            color,
            value,
            labels,
        })
    };
    if definition[87] != 0 {
        below(12, CellColor::Red, None, vec![0x40332c]);
        let upgrades = [
            (byte(82).wrapping_add(byte(81)), 0x403324),
            (byte(86).wrapping_add(word(definition, 94)?), 0x40331c),
            (byte(85), 0x403312),
            (byte(84), 0x403308),
            (byte(83), 0x4032fe),
            (byte(80), 0x4032f2),
            (byte(79), 0x4032ec),
            (byte(78).wrapping_add(word(definition, 88)?), 0x4032e2),
            (word(definition, 90)?, 0x4032da),
            (word(definition, 148)?, 0x4032d0),
        ];
        let labels = upgrades
            .iter()
            .filter(|(value, _)| *value != 0)
            .map(|(_, label)| *label)
            .collect();
        below(24, CellColor::Red, None, labels);
        let total = [86, 81, 85, 84, 83, 80, 79, 78]
            .iter()
            .fold(0_i16, |sum, offset| sum.wrapping_add(byte(*offset)))
            .wrapping_add(word(definition, 90)?)
            .wrapping_add(word(definition, 148)?);
        let percent = if word(definition, 148)? != 0 {
            vec![0x4032ce]
        } else {
            vec![]
        };
        below(36, CellColor::Green, Some((total, true)), percent);
    } else {
        for (y, color, value, label) in [
            (
                10,
                CellColor::Red,
                field(11).wrapping_add(field(15)),
                0x4032c8,
            ),
            (20, CellColor::Blue, field(18), 0x4032c2),
            (30, CellColor::Green, field(20), 0x4032bc),
            (
                20,
                CellColor::Black,
                field(19).wrapping_add(word(definition, 94)?),
                0x4032b6,
            ),
            (
                30,
                CellColor::Green,
                field(14).wrapping_add(word(definition, 88)?),
                0x4032b0,
            ),
            (
                40,
                CellColor::Red,
                field(16).wrapping_add(byte(83)).wrapping_add(byte(3)),
                0x4032a6,
            ),
            (
                50,
                CellColor::Blue,
                field(17).wrapping_add(byte(84)).wrapping_add(byte(4)),
                0x40329c,
            ),
        ] {
            if value != 0 {
                below(y, color, Some((value, true)), vec![label]);
            }
        }
    }
    Ok(cell)
}
