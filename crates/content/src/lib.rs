use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Component, Path};
use thiserror::Error;

pub mod audio;
pub mod combat;
pub mod graphics;
pub mod installation;
pub mod mission;
pub mod native_text;
pub mod ui;

pub const WARRIOR_BYTES: usize = 0xc42;
pub const ITEM_BYTES: usize = 150;
pub const INVENTORY_BYTES: usize = 22;

#[derive(Debug, Error)]
pub enum ContentError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid content: {0}")]
    Invalid(String),
}

pub fn span(bytes: &[u8], offset: usize, size: usize) -> Result<&[u8], ContentError> {
    offset
        .checked_add(size)
        .and_then(|end| bytes.get(offset..end))
        .ok_or_else(|| {
            ContentError::Invalid(format!("range {offset}+{size} exceeds {}", bytes.len()))
        })
}

pub fn word(bytes: &[u8], offset: usize) -> Result<i16, ContentError> {
    let value = span(bytes, offset, 2)?;
    Ok(i16::from_be_bytes([value[0], value[1]]))
}

pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Warrior {
    pub name: String,
    pub original_sha256: String,
    pub bytes: Vec<u8>,
}

impl Warrior {
    pub fn read(bytes: &[u8]) -> Result<Self, ContentError> {
        if bytes.len() != WARRIOR_BYTES {
            return Err(ContentError::Invalid(format!(
                "warrior size {}, expected {WARRIOR_BYTES}",
                bytes.len()
            )));
        }
        let name_field = span(bytes, 0xa21, 31)?;
        let length = name_field
            .iter()
            .position(|b| *b == 0)
            .ok_or_else(|| ContentError::Invalid("unterminated warrior name".into()))?;
        if length == 0 {
            return Err(ContentError::Invalid("empty warrior name".into()));
        }
        let name_bytes = &name_field[..length];
        Ok(Self {
            name: encoding_rs::MACINTOSH.decode(name_bytes).0.into_owned(),
            original_sha256: digest(bytes),
            bytes: bytes.to_vec(),
        })
    }

    pub fn stamina(&self) -> Result<i16, ContentError> {
        word(&self.bytes, 0x50)
    }
    pub fn psi_points(&self) -> Result<i16, ContentError> {
        word(&self.bytes, 0x58)
    }
    pub fn knows_spell(&self, spell: i16) -> Result<bool, ContentError> {
        let school = spell / 1000;
        let row = spell % 1000 / 100 - 1;
        let column = spell % 100 - 1;
        if school < 1 || !(0..7).contains(&row) || !(0..7).contains(&column) {
            return Ok(false);
        }
        Ok(word(&self.bytes, 0x36)? == school
            && span(&self.bytes, 0x9f0, 49)?[(row * 7 + column) as usize] != 0)
    }
    pub fn inventory(&self) -> Result<Vec<InventoryRecord>, ContentError> {
        let count = word(&self.bytes, 0x1a)?;
        if !(0..=100).contains(&count) {
            return Err(ContentError::Invalid(format!("inventory count {count}")));
        }
        (0..count as usize)
            .map(|i| {
                InventoryRecord::read(span(
                    &self.bytes,
                    0x132 + i * INVENTORY_BYTES,
                    INVENTORY_BYTES,
                )?)
            })
            .collect()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct InventoryRecord(pub [u8; INVENTORY_BYTES]);

impl InventoryRecord {
    pub fn read(bytes: &[u8]) -> Result<Self, ContentError> {
        let mut record = [0; INVENTORY_BYTES];
        record.copy_from_slice(span(bytes, 0, INVENTORY_BYTES)?);
        Ok(Self(record))
    }
    pub fn id(&self) -> i16 {
        i16::from_be_bytes([self.0[0], self.0[1]])
    }
}

#[derive(Debug, Deserialize)]
struct Fork {
    path: String,
    kind: String,
    size: usize,
    sha256: String,
}
#[derive(Debug, Deserialize)]
struct Manifest {
    schema_version: u32,
    input_manifest_sha256: String,
    forks: Vec<Fork>,
    resources: Vec<Resource>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Resource {
    pub original: String,
    pub r#type: String,
    pub id: i16,
    pub size: usize,
    pub sha256: String,
    pub output: String,
    #[serde(default)]
    pub source_fork: Option<installation::SourceFork>,
}

#[derive(Clone, Debug)]
pub struct Content {
    pub identity: String,
    pub tutorials: Vec<String>,
    pub warriors: Vec<Warrior>,
    pub items: Vec<Vec<u8>>,
    pub resources: Vec<Resource>,
    pub scenario: Option<mission::ScenarioContent>,
    pub original_services: Option<OriginalServices>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OriginalServices {
    pub schema_version: u32,
    pub content_identity: String,
    pub executable_code_sha256: String,
    pub registration_accepted: bool,
    pub authority: String,
}

fn safe_path(root: &Path, relative: &str) -> Result<std::path::PathBuf, ContentError> {
    if relative.contains('\\')
        || relative.contains(':')
        || Path::new(relative)
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(ContentError::Invalid(format!("unsafe path {relative:?}")));
    }
    let canonical_root = root.canonicalize()?;
    let path = root.join(relative).canonicalize()?;
    if !path.starts_with(canonical_root) {
        return Err(ContentError::Invalid("path escapes input root".into()));
    }
    Ok(path)
}

impl Content {
    pub fn context(
        &self,
        name: Option<&str>,
        level: u8,
    ) -> Result<std::borrow::Cow<'_, Self>, ContentError> {
        let Some(name) = name else {
            return Ok(std::borrow::Cow::Borrowed(self));
        };
        let current = self
            .scenario
            .as_ref()
            .ok_or_else(|| ContentError::Invalid("scenario unavailable".into()))?;
        if current.name == name && current.level == level {
            return Ok(std::borrow::Cow::Borrowed(self));
        }
        let selected = current.selection(name, level).ok_or_else(|| {
            ContentError::Invalid(format!("scenario level unavailable: {name}/{level}"))
        })?;
        let mut result = self.clone();
        let mut selected = selected.clone();
        selected.catalog = current.catalog.clone();
        result.items.truncate(800);
        result.items.extend(selected.items.iter().cloned());
        result.scenario = Some(selected);
        Ok(std::borrow::Cow::Owned(result))
    }

    pub fn scenario_names(&self) -> Vec<String> {
        let Some(current) = &self.scenario else {
            return vec![];
        };
        let mut names = std::collections::BTreeSet::from([current.name.clone()]);
        names.extend(
            current
                .catalog
                .keys()
                .filter(|(_, level)| *level == 0)
                .map(|(name, _)| name.clone()),
        );
        names.into_iter().collect()
    }

    pub fn map_at(
        &self,
        name: Option<&str>,
        level: u8,
        index: i32,
    ) -> Option<&mission::OpeningMission> {
        let current = self.scenario.as_ref()?;
        current
            .selection(name.unwrap_or(&current.name), level)?
            .maps
            .get(usize::try_from(index).ok()?)
    }

    pub fn map(&self, index: i32) -> Option<&mission::OpeningMission> {
        self.scenario
            .as_ref()?
            .maps
            .get(usize::try_from(index).ok()?)
    }

    pub fn load(originals: &Path, extracted: &Path) -> Result<Self, ContentError> {
        let manifest: Manifest =
            serde_json::from_slice(&fs::read(extracted.join("manifest.json"))?)?;
        if manifest.schema_version != 1 {
            return Err(ContentError::Invalid(
                "unsupported extraction manifest".into(),
            ));
        }
        if digest(&fs::read(originals.join("packsmith-mapping.json"))?)
            != manifest.input_manifest_sha256
        {
            return Err(ContentError::Invalid(
                "original manifest identity changed".into(),
            ));
        }
        Self::from_manifest(originals, extracted, manifest)
    }

    fn from_manifest(
        originals: &Path,
        extracted: &Path,
        manifest: Manifest,
    ) -> Result<Self, ContentError> {
        let mut warriors = Vec::new();
        let mut items = Vec::new();
        let mut item_cache = None;
        for fork in manifest.forks.iter().filter(|f| f.kind == "data") {
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
                item_cache = Some(span(&bytes, 0, 200 * ITEM_BYTES)?.to_vec());
                items = span(&bytes, 0, 800 * ITEM_BYTES)?
                    .as_chunks::<ITEM_BYTES>()
                    .0
                    .iter()
                    .map(|record| record.to_vec())
                    .collect();
            }
        }
        let item_cache = item_cache
            .ok_or_else(|| ContentError::Invalid("missing original item cache".into()))?;
        let mut catalog = std::collections::BTreeMap::new();
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
            )?;
            let mut tables = (*selected.maps[0].shared).clone();
            tables.item_cache = item_cache.clone();
            let owner = format!("TheNewCenturions/Scenarios/{}/{level}/Scenario", parts[0]);
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
            for resource in manifest.resources.iter().filter(|r| {
                r.r#type == "STR#" && matches!(r.id, -101 | -102) && r.original == owner
            }) {
                let labels = ui::string_list(&read_resource(extracted, resource)?)?;
                if resource.id == -102 {
                    tables.note_labels = labels;
                } else {
                    tables.unknown_note_labels = labels;
                }
            }
            tables.prologue = tables.movies.get(&1000).cloned().ok_or_else(|| {
                ContentError::Invalid(format!("opening prologue missing: {owner}"))
            })?;
            let tables = std::sync::Arc::new(tables);
            for map in &mut selected.maps {
                map.shared = tables.clone();
            }
            catalog.insert((parts[0].to_string(), level), selected);
        }
        let mut scenario = catalog
            .get(&("The Thunder of Guns".into(), 0))
            .or_else(|| {
                catalog
                    .iter()
                    .find(|((_, level), _)| *level == 0)
                    .map(|(_, value)| value)
            })
            .cloned()
            .ok_or_else(|| ContentError::Invalid("no installed starting scenario".into()))?;
        items.extend(scenario.items.iter().cloned());
        scenario.catalog = std::sync::Arc::new(catalog);
        warriors.sort_by(|a, b| a.name.cmp(&b.name));
        if warriors.len() != 6 {
            return Err(ContentError::Invalid(format!(
                "expected six starter warriors, found {}",
                warriors.len()
            )));
        }
        if items.is_empty() {
            return Err(ContentError::Invalid("missing item table".into()));
        }
        let tutorial_bytes = read_fork(
            originals,
            &manifest.forks,
            "TheNewCenturions/Data Files/Tutorial Text",
        )?;
        let (records, tail) = tutorial_bytes.as_chunks::<256>();
        if !tail.is_empty() || records.len() > 256 {
            return Err(ContentError::Invalid("invalid tutorial text table".into()));
        }
        let tutorials = records
            .iter()
            .map(|record| {
                encoding_rs::MACINTOSH
                    .decode(&record[1..1 + usize::from(record[0])])
                    .0
                    .replace('\r', "\n")
            })
            .collect();
        let mut resources = manifest.resources;
        for resource in &mut resources {
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
        Ok(Self {
            identity: manifest.input_manifest_sha256,
            tutorials,
            warriors,
            items,
            resources,
            scenario: Some(scenario),
            original_services: None,
        })
    }
}

fn read_fork(root: &Path, forks: &[Fork], path: &str) -> Result<Vec<u8>, ContentError> {
    let fork = forks
        .iter()
        .find(|f| f.kind == "data" && f.path == path)
        .ok_or_else(|| ContentError::Invalid(format!("missing data fork {path}")))?;
    let bytes = installation::read_data(root, path)?;
    if bytes.len() != fork.size || digest(&bytes) != fork.sha256 {
        return Err(ContentError::Invalid(format!(
            "fork identity mismatch: {path}"
        )));
    }
    Ok(bytes)
}

pub fn read_resource(extracted: &Path, resource: &Resource) -> Result<Vec<u8>, ContentError> {
    let bytes = if let Some(source) = &resource.source_fork {
        if let Some(fork) = &source.cached {
            span(fork, source.offset, resource.size)?.to_vec()
        } else {
            let raw = installation::read_file(&safe_path(extracted, &source.path)?)?;
            let fork = installation::fork_payload(&raw, 2)?;
            span(fork, source.offset, resource.size)?.to_vec()
        }
    } else {
        fs::read(safe_path(extracted, &resource.output)?)?
    };
    if bytes.len() != resource.size || digest(&bytes) != resource.sha256 {
        return Err(ContentError::Invalid(format!(
            "resource identity mismatch: {} {}:{}",
            resource.original, resource.r#type, resource.id
        )));
    }
    Ok(bytes)
}
