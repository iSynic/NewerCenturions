//! BD tile-property files. loadland 0x13107a calls loadpixmap 0x131182 with the land's
//! tile-set byte (region-table offset 520); when it differs from the loaded set, loadpixmap
//! reads the file's default tile and tick flag, 60 further bytes, and 16000 bytes of tile
//! properties, of which the 300 recovered 40-byte records are kept.
use crate::{ContentError, span, word};

/// Tile sets 1 to 9; loadpixmap names custom files for 15 and above.
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
        _ => return None,
    })
}

pub fn read(
    read: &mut impl FnMut(&str) -> Result<Vec<u8>, ContentError>,
    terrain: u8,
) -> Result<TileSet, ContentError> {
    let name = file_name(terrain)
        .ok_or_else(|| ContentError::Invalid(format!("unknown terrain {terrain}")))?;
    let bytes = read(&format!("TheNewCenturions/Data Files/{name}"))?;
    Ok(TileSet {
        terrain,
        properties: span(&bytes, 64, 12000)?.to_vec(),
        default_tile: word(&bytes, 0)?,
        minutes_per_tick: if word(&bytes, 2)? == 0 { 5 } else { 1 },
    })
}

/// Every tile set the installation supplies; a missing file is left out, so only a land
/// switching to it is refused.
pub fn read_all(mut read: impl FnMut(&str) -> Result<Vec<u8>, ContentError>) -> Vec<TileSet> {
    (1..=TILE_SETS as u8)
        .filter_map(|terrain| self::read(&mut read, terrain).ok())
        .collect()
}
