//! BD tile-property files. loadland 0x13107a calls loadpixmap 0x131182 with the land's
//! tile-set byte (region-table offset 520); when it differs from the loaded set, loadpixmap
//! reads the file's default tile and tick flag, 60 further bytes, and up to 16000 bytes of
//! 40-byte tile properties. Supplied sets hold 300 records (12064 bytes); a shorter read
//! leaves the rest of the native buffer as an earlier land left it.
use crate::{ContentError, span, word};

/// The installation-wide standard sets. Custom sets belong to a scenario level.
pub const TILE_SETS: usize = 9;

#[derive(Clone, Debug)]
pub struct TileSet {
    pub terrain: u8,
    pub properties: Vec<u8>,
    pub default_tile: i16,
    pub minutes_per_tick: i16,
}

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

/// loadpixmap asks GetPicture for the terrain WORD plus299.
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
