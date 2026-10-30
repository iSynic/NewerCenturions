use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Component, Path};
use thiserror::Error;

pub mod audio;
pub mod bytes;
mod catalog;
pub mod combat;
pub mod game_text;
pub mod graphics;
pub mod installation;
pub mod mission;
pub mod records;
pub mod tile_sets;
pub mod ui;

pub use bytes::word;

/// Size of a warrior record.
pub const WARRIOR_BYTES: usize = 0xc42;
/// Size of an item definition.
pub const ITEM_BYTES: usize = 150;
/// Size of one item a warrior carries.
pub const INVENTORY_BYTES: usize = 22;

/// What can go wrong reading the game's files.
#[derive(Debug, Error)]
pub enum ContentError {
    /// The file could not be read.
    #[error("{0}")]
    Io(#[from] std::io::Error),
    /// A manifest could not be parsed.
    #[error("{0}")]
    Json(#[from] serde_json::Error),
    /// The data is not what the game stores; the text says why.
    #[error("invalid content: {0}")]
    Invalid(String),
}

/// The `size` bytes at `offset`, or an error when the data ends first.
pub fn span(bytes: &[u8], offset: usize, size: usize) -> Result<&[u8], ContentError> {
    offset
        .checked_add(size)
        .and_then(|end| bytes.get(offset..end))
        .ok_or_else(|| {
            ContentError::Invalid(format!("range {offset}+{size} exceeds {}", bytes.len()))
        })
}

/// The SHA-256 of `bytes` as lower-case hexadecimal.
pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// A warrior as a file stores it.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Warrior {
    /// The warrior's name.
    pub name: String,
    /// The digest of the record as read, kept to tell it from the original.
    pub original_sha256: String,
    /// The warrior record, `WARRIOR_BYTES` long.
    pub bytes: Vec<u8>,
}

impl Warrior {
    /// Reads a warrior record; one of the wrong size, or with an unterminated or empty
    /// name, is refused.
    pub fn read(bytes: &[u8]) -> Result<Self, ContentError> {
        if bytes.len() != WARRIOR_BYTES {
            return Err(ContentError::Invalid(format!(
                "warrior size {}, expected {WARRIOR_BYTES}",
                bytes.len()
            )));
        }
        let name_field = records::WarriorRecord::from_bytes(bytes).name_field()?;
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

    /// The warrior's current stamina.
    pub fn stamina(&self) -> Result<i16, ContentError> {
        self.record().stamina()
    }
    /// The warrior's current psi points.
    pub fn psi_points(&self) -> Result<i16, ContentError> {
        self.record().psi_points()
    }
    /// Whether the warrior has learned scan `spell`, a code of school, row and column (school *
    /// 1000 + row * 100 + column, rows and columns counted from one). A warrior knows only scans of
    /// the warrior's own school.
    pub fn knows_spell(&self, spell: i16) -> Result<bool, ContentError> {
        let school = spell / 1000;
        let row = spell % 1000 / 100 - 1;
        let column = spell % 100 - 1;
        if school < 1 || !(0..7).contains(&row) || !(0..7).contains(&column) {
            return Ok(false);
        }
        let record = self.record();
        Ok(record.psi_school()? == school
            && record.known_scans()?[(row * 7 + column) as usize] != 0)
    }
    /// The items the warrior carries, as many as the item count says.
    pub fn inventory(&self) -> Result<Vec<InventoryRecord>, ContentError> {
        let count = self.record().item_count()?;
        if !(0..=100).contains(&count) {
            return Err(ContentError::Invalid(format!("inventory count {count}")));
        }
        (0..count as usize)
            .map(|i| records::InventorySlots(&self.bytes).slot(i))
            .collect()
    }
}

/// One item a warrior carries: its id, charge and equipment state.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct InventoryRecord(pub [u8; INVENTORY_BYTES]);

impl InventoryRecord {
    /// Reads one carried item from the start of `bytes`.
    pub fn read(bytes: &[u8]) -> Result<Self, ContentError> {
        let mut record = [0; INVENTORY_BYTES];
        record.copy_from_slice(span(bytes, 0, INVENTORY_BYTES)?);
        Ok(Self(record))
    }
    /// The item's id; zero is an empty slot.
    pub fn id(&self) -> i16 {
        bytes::word_at(&self.0, 0)
    }
}

/// One file fork named in the manifest, with the size and digest it must have.
#[derive(Debug, Deserialize)]
struct Fork {
    path: String,
    kind: String,
    size: usize,
    sha256: String,
}
/// The list of forks and resources an installation is checked against.
#[derive(Debug, Deserialize)]
struct Manifest {
    schema_version: u32,
    input_manifest_sha256: String,
    forks: Vec<Fork>,
    resources: Vec<Resource>,
}

/// One resource of a resource fork, with the digest that identifies it.
#[derive(Clone, Debug, Deserialize)]
pub struct Resource {
    /// The file whose fork holds the resource.
    pub original: String,
    /// The resource type, such as `PICT` or `STR#`.
    pub r#type: String,
    /// The resource id.
    pub id: i16,
    /// The resource's length in bytes.
    pub size: usize,
    /// The digest of the resource's bytes.
    pub sha256: String,
    /// Where an extracted copy of the resource lies.
    pub output: String,
    #[serde(default)]
    pub source_fork: Option<installation::SourceFork>,
}

/// Everything the game reads from its installation: the starter warriors, item
/// tables, scenarios, resources and tutorial text.
#[derive(Clone, Debug)]
pub struct Content {
    /// The digest of the installation's manifest.
    pub identity: String,
    /// The tutorial texts, from `Tutorial Text`.
    pub tutorials: Vec<String>,
    /// The six starter warriors, sorted by name.
    pub warriors: Vec<Warrior>,
    /// Item definitions: the 800 stock items, then the 200 of the current scenario.
    pub items: Vec<Vec<u8>>,
    /// Every resource of the installation.
    pub resources: Vec<Resource>,
    /// The current scenario level.
    pub scenario: Option<mission::ScenarioContent>,
    /// The original program's service record, when one is supplied.
    pub original_services: Option<OriginalServices>,
}

/// Facts recorded about the original program's services; its registration is
/// accepted.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OriginalServices {
    /// The version of this record's layout.
    pub schema_version: u32,
    /// The digest of the installation the record was made for.
    pub content_identity: String,
    /// The digest of the program's code resources.
    pub executable_code_sha256: String,
    /// Whether registration counts as complete.
    pub registration_accepted: bool,
    /// Who or what supplied the record.
    pub authority: String,
}

/// The path of `relative` below `root`; a name that leaves the root is refused.
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
    /// The content as the scenario `name` at `level` sees it, with that scenario's
    /// maps and items in place of the current ones. `None` for the current scenario.
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
        result.items.truncate(catalog::ITEM_TABLE_RECORDS);
        result.items.extend(selected.items.iter().cloned());
        result.scenario = Some(selected);
        Ok(std::borrow::Cow::Owned(result))
    }

    /// The names of the scenarios the installation holds at level 0.
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

    /// Map `index` of the scenario `name` at `level`; `None` names the current one.
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

    /// A map under the tile set its live region table names; see ScenarioContent::land.
    pub fn land_at(
        &self,
        name: Option<&str>,
        level: u8,
        index: i32,
        terrain: u8,
    ) -> Option<&mission::OpeningMission> {
        let current = self.scenario.as_ref()?;
        current
            .selection(name.unwrap_or(&current.name), level)?
            .land(index, terrain)
    }

    /// Map `index` of the current scenario.
    pub fn map(&self, index: i32) -> Option<&mission::OpeningMission> {
        self.scenario
            .as_ref()?
            .maps
            .get(usize::try_from(index).ok()?)
    }

    /// Reads the installation described by an extraction manifest, checking it against
    /// the original manifest.
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

    /// Builds the content from a manifest: the starter warriors, item table,
    /// scenarios, tutorials and program resources.
    fn from_manifest(
        originals: &Path,
        extracted: &Path,
        manifest: Manifest,
    ) -> Result<Self, ContentError> {
        let (mut warriors, mut items, item_cache) =
            catalog::read_starters_and_item_table(originals, &manifest.forks)?;
        let catalog = catalog::read_scenarios(originals, extracted, &manifest, &item_cache)?;
        let mut scenario = catalog::starting_scenario(&catalog)?;
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
        let tutorials = catalog::read_tutorials(originals, &manifest.forks)?;
        let mut resources = manifest.resources;
        catalog::cache_application_code(extracted, &mut resources)?;
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

/// Reads a data fork the manifest lists, checking its size and digest.
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

/// Reads a resource's bytes, checking its size and digest.
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
