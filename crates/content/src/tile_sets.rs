//! BD tile-property files. Entering a land loads the tile set its region table
//! names when that differs from the set already loaded: the default tile, the
//! tick flag, 60 further bytes, then up to 16000 bytes of 40-byte tile
//! properties. Supplied sets hold 300 records (12064 bytes); a shorter file
//! leaves the rest of the table as an earlier land left it.
use crate::{ContentError, span, word};

/// The installation-wide standard sets. Custom sets belong to a scenario level.
pub const TILE_SETS: usize = 9;

/// One BD tile-property file as a land uses it.
#[derive(Clone, Debug)]
pub struct TileSet {
    /// The terrain number that names the file: 1 to 9 for the standard sets, 15 to 19
    /// for a scenario's own.
    pub terrain: u8,
    /// The 40-byte tile property records, up to 16000 bytes.
    pub properties: Vec<u8>,
    /// The tile that fills a map cell no tile is placed on.
    pub default_tile: i16,
    /// Game minutes a step on this terrain takes: 1, or 5 when the tick flag is clear.
    pub minutes_per_tick: i16,
}

/// The name of the BD file for `terrain`, or none when no tile set has that number.
pub fn file_name(terrain: u8) -> Option<&'static str> {
    Some(match terrain {
        1 => "Data P BD",
        2 => "Data Swamp BD",
        3 => "Data Snow BD",
        4 => "Data Sub BD",
        5 => "Data Desert BD",
        6 => "Data Lava BD",
        7 => "Data Castle BD",
        8 => "Data Techno BD",
        9 => "Data Biologic BD",
        15..=19 => " BD",
        _ => return None,
    })
}

/// Reads one tile set's BD file.
///
/// Original: loadpixmap
pub fn read(
    read: &mut impl FnMut(&str) -> Result<Vec<u8>, ContentError>,
    terrain: u8,
) -> Result<TileSet, ContentError> {
    let name = file_name(terrain)
        .ok_or_else(|| ContentError::Invalid(format!("unknown terrain {terrain}")))?;
    let path = if terrain >= 15 {
        format!("{}{name}", crate::mission::SCENARIO)
    } else {
        format!("TheNewCenturions/Data Files/{name}")
    };
    let bytes = read(&path)?;
    let properties = span(&bytes, 64, bytes.len().saturating_sub(64).min(16000))?;
    Ok(TileSet {
        terrain,
        properties: properties.to_vec(),
        default_tile: word(&bytes, 0)?,
        minutes_per_tick: if word(&bytes, 2)? == 0 { 5 } else { 1 },
    })
}

/// The tile set whose terrain picture is `id`: a land's terrain picture is PICT
/// terrain + 299.
pub fn picture_terrain(id: i16) -> Option<u8> {
    let terrain = u8::try_from(id.checked_sub(299)?).ok()?;
    file_name(terrain).map(|_| terrain)
}

/// Every tile set the installation supplies; a missing file is left out, so only a land
/// switching to it is refused.
pub fn read_all(mut read: impl FnMut(&str) -> Result<Vec<u8>, ContentError>) -> Vec<TileSet> {
    (1..=TILE_SETS as u8)
        .filter_map(|terrain| self::read(&mut read, terrain).ok())
        .collect()
}
