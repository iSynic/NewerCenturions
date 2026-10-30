//! Reads the starter warriors, item table, scenario catalog, tutorial text and
//! application code named by the extraction manifest, in the original order.
use crate::{
    ContentError, Fork, ITEM_BYTES, Manifest, Resource, WARRIOR_BYTES, Warrior, digest,
    installation, mission, read_fork, read_resource, span, tile_sets, ui,
};
use std::collections::BTreeMap;
use std::path::Path;

/// The leading Data ID records kept as every scenario's item cache.
const ITEM_CACHE_RECORDS: usize = 200;
/// The Data ID records of the base item table; scenario items follow them.
pub(crate) const ITEM_TABLE_RECORDS: usize = 800;
/// A scenario's Data MENU: 251 big-endian word bestiary record numbers.
const BESTIARY_ORDER_BYTES: usize = 502;

/// Starter warriors, the base item records and the item cache.
pub(crate) type StartersAndItems = (Vec<Warrior>, Vec<Vec<u8>>, Vec<u8>);

/// Every installed scenario level, by name and level.
pub(crate) type ScenarioCatalog = BTreeMap<(String, u8), mission::ScenarioContent>;

/// The starter warriors, the base item table and the item cache, checked against
/// the manifest in fork order.
pub(crate) fn read_starters_and_item_table(
    originals: &Path,
    forks: &[Fork],
) -> Result<StartersAndItems, ContentError> {
    let mut warriors = Vec::new();
    let mut items = Vec::new();
    let mut item_cache = None;
    for fork in forks.iter().filter(|f| f.kind == "data") {
        let starter =
            fork.path.starts_with("TheNewCenturions/Warriors/") && fork.size == WARRIOR_BYTES;
        let item_table = fork.path == "TheNewCenturions/Data Files/Data ID";
        if !starter && !item_table {
            continue;
        }
        let bytes = installation::read_data(originals, &fork.path)?;
        if bytes.len() != fork.size || digest(&bytes) != fork.sha256 {
            return Err(ContentError::Invalid(format!(
                "fork identity mismatch: {}",
                fork.path
            )));
        }
        if starter {
            warriors.push(Warrior::read(&bytes)?);
        }
        if item_table {
            if bytes.len() % ITEM_BYTES != 0 {
                return Err(ContentError::Invalid("partial item record".into()));
            }
            item_cache = Some(span(&bytes, 0, ITEM_CACHE_RECORDS * ITEM_BYTES)?.to_vec());
            items = span(&bytes, 0, ITEM_TABLE_RECORDS * ITEM_BYTES)?
                .as_chunks::<ITEM_BYTES>()
                .0
                .iter()
                .map(|record| record.to_vec())
                .collect();
        }
    }
    let item_cache =
        item_cache.ok_or_else(|| ContentError::Invalid("missing original item cache".into()))?;
    Ok((warriors, items, item_cache))
}

/// Every installed scenario level, keyed by scenario name and level.
pub(crate) fn read_scenarios(
    originals: &Path,
    extracted: &Path,
    manifest: &Manifest,
    item_cache: &[u8],
) -> Result<ScenarioCatalog, ContentError> {
    let mut catalog = BTreeMap::new();
    let tile_sets = std::sync::Arc::new(tile_sets::read_all(|path| {
        read_fork(originals, &manifest.forks, path)
    }));
    for fork in manifest
        .forks
        .iter()
        .filter(|f| f.kind == "data" && f.path.ends_with("/Data CS"))
    {
        let Some(relative) = fork.path.strip_prefix("TheNewCenturions/Scenarios/") else {
            continue;
        };
        let parts: Vec<_> = relative.split('/').collect();
        if parts.len() != 3 {
            return Err(ContentError::Invalid("invalid scenario directory".into()));
        }
        let level = parts[1]
            .parse::<u8>()
            .map_err(|_| ContentError::Invalid("invalid scenario level".into()))?;
        let mut selected = mission::ScenarioContent::read_level(
            |path| read_fork(originals, &manifest.forks, path),
            parts[0],
            level,
            tile_sets.clone(),
        )?;
        let mut tables = (*selected.maps[0].shared).clone();
        tables.item_cache = item_cache.to_vec();
        read_level_tables(originals, extracted, manifest, parts[0], level, &mut tables)?;
        let tables = std::sync::Arc::new(tables);
        for map in &mut selected.maps {
            map.shared = tables.clone();
        }
        catalog.insert((parts[0].to_string(), level), selected);
    }
    Ok(catalog)
}

/// A level's bestiary order, movies, note labels and opening prologue.
fn read_level_tables(
    originals: &Path,
    extracted: &Path,
    manifest: &Manifest,
    scenario: &str,
    level: u8,
    tables: &mut mission::StoryTables,
) -> Result<(), ContentError> {
    let archive = format!("TheNewCenturions/Scenarios/{scenario}/{level}/Data MENU");
    if manifest
        .forks
        .iter()
        .any(|fork| fork.kind == "data" && fork.path == archive)
    {
        let bytes = read_fork(originals, &manifest.forks, &archive)?;
        if bytes.len() != BESTIARY_ORDER_BYTES {
            return Err(ContentError::Invalid("invalid bestiary order table".into()));
        }
        tables.archive_table = Some(
            bytes
                .as_chunks::<2>()
                .0
                .iter()
                .map(|bytes| u16::from_be_bytes(*bytes))
                .collect(),
        );
    }
    let owner = format!("TheNewCenturions/Scenarios/{scenario}/{level}/Scenario");
    for resource in manifest
        .resources
        .iter()
        .filter(|r| r.r#type == "TEXT" && r.original == owner)
    {
        tables.movies.insert(
            resource.id,
            encoding_rs::MACINTOSH
                .decode(&read_resource(extracted, resource)?)
                .0
                .replace('\r', "\n"),
        );
    }
    for resource in manifest
        .resources
        .iter()
        .filter(|r| r.r#type == "STR#" && matches!(r.id, -101 | -102) && r.original == owner)
    {
        let labels = ui::string_list(&read_resource(extracted, resource)?)?;
        if resource.id == -102 {
            tables.note_labels = labels;
        } else {
            tables.unknown_note_labels = labels;
        }
    }
    tables.prologue = tables
        .movies
        .get(&1000)
        .cloned()
        .ok_or_else(|| ContentError::Invalid(format!("opening prologue missing: {owner}")))?;
    Ok(())
}

/// The Thunder of Guns' first level, or else any installed first level.
pub(crate) fn starting_scenario(
    catalog: &ScenarioCatalog,
) -> Result<mission::ScenarioContent, ContentError> {
    catalog
        .get(&("The Thunder of Guns".into(), 0))
        .or_else(|| {
            catalog
                .iter()
                .find(|((_, level), _)| *level == 0)
                .map(|(_, value)| value)
        })
        .cloned()
        .ok_or_else(|| ContentError::Invalid("no installed starting scenario".into()))
}

/// The Tutorial Text records: up to 256 Pascal strings of 256 bytes.
pub(crate) fn read_tutorials(
    originals: &Path,
    forks: &[Fork],
) -> Result<Vec<String>, ContentError> {
    let tutorial_bytes = read_fork(
        originals,
        forks,
        "TheNewCenturions/Data Files/Tutorial Text",
    )?;
    let (records, tail) = tutorial_bytes.as_chunks::<256>();
    if !tail.is_empty() || records.len() > 256 {
        return Err(ContentError::Invalid("invalid tutorial text table".into()));
    }
    Ok(records
        .iter()
        .map(|record| {
            encoding_rs::MACINTOSH
                .decode(&record[1..1 + usize::from(record[0])])
                .0
                .replace('\r', "\n")
        })
        .collect())
}

/// Caches the application's CODE 0 and DATA 0 resources as their own source forks.
pub(crate) fn cache_application_code(
    extracted: &Path,
    resources: &mut [Resource],
) -> Result<(), ContentError> {
    for resource in resources {
        if resource.original.ends_with("/New Centurions")
            && resource.id == 0
            && matches!(resource.r#type.as_str(), "CODE" | "DATA")
            && resource.source_fork.is_none()
        {
            let bytes = read_resource(extracted, resource)?;
            resource.source_fork = Some(installation::SourceFork {
                path: extracted
                    .join(&resource.output)
                    .to_string_lossy()
                    .into_owned(),
                offset: 0,
                cached: Some(std::sync::Arc::from(bytes)),
            });
        }
    }
    Ok(())
}
