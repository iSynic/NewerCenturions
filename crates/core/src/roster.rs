use crate::{Phase, RuleError, Session, atomic_write};
use centurions_content::records::{CharacterIdentityMut, ItemDefinition, LearnedScansMut};
use centurions_content::{Content, Warrior};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone, Serialize, Deserialize)]
pub struct Roster {
    schema_version: u32,
    content_identity: String,
    pub warriors: Vec<Warrior>,
    #[serde(skip)]
    path: Option<PathBuf>,
}
impl Roster {
    pub fn supplied(content: &Content) -> Self {
        Self {
            schema_version: 1,
            content_identity: content.identity.clone(),
            warriors: content.warriors.clone(),
            path: None,
        }
    }
    pub fn open(path: &Path, content: &Content) -> Result<Self, RuleError> {
        let mut roster = match std::fs::metadata(path) {
            Ok(metadata) => {
                if metadata.len() > 32 * 1024 * 1024 {
                    return Err(RuleError::Invalid(
                        "warrior roster exceeds size limit".into(),
                    ));
                }
                serde_json::from_slice::<Self>(&std::fs::read(path)?)?
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Self::supplied(content),
            Err(error) => return Err(error.into()),
        };
        if roster.schema_version != 1 || roster.content_identity != content.identity {
            return Err(RuleError::Invalid("incompatible warrior roster".into()));
        }
        roster.validate()?;
        roster.path = Some(path.to_path_buf());
        Ok(roster)
    }
    fn validate(&self) -> Result<(), RuleError> {
        if self.warriors.len() > 1024 {
            return Err(RuleError::Invalid(
                "warrior roster capacity exceeded".into(),
            ));
        }
        for (i, warrior) in self.warriors.iter().enumerate() {
            validate_warrior(warrior)?;
            if self.warriors[..i]
                .iter()
                .any(|w| w.original_sha256 == warrior.original_sha256)
            {
                return Err(RuleError::Invalid(
                    "duplicate warrior identity in roster".into(),
                ));
            }
        }
        Ok(())
    }
    pub fn store(&mut self, warriors: &[Warrior]) -> Result<(), RuleError> {
        let mut next = self.clone();
        for warrior in warriors {
            validate_warrior(warrior)?;
            if let Some(index) = next
                .warriors
                .iter()
                .position(|w| w.original_sha256 == warrior.original_sha256)
            {
                next.warriors[index] = warrior.clone();
            } else {
                next.warriors.push(warrior.clone());
            }
        }
        next.validate()?;
        if let Some(path) = &next.path {
            atomic_write(path, &serde_json::to_vec_pretty(&next)?)?;
        }
        *self = next;
        Ok(())
    }
}
/// savecharacter 0x1374bc filters temporary mission items and verifies the saved copy.
pub fn mission_records(state: &Session, content: &Content) -> Result<Vec<Warrior>, RuleError> {
    let mission = state
        .exploration
        .as_ref()
        .and_then(|p| content.map(p.map))
        .ok_or("mission roster profiles missing")?;
    let mut records = state.warriors.clone();
    for (member, warrior) in records.iter_mut().enumerate() {
        for slot in (0..warrior.inventory()?.len()).rev() {
            let id = warrior.inventory()?[slot].id();
            let definition = content
                .items
                .get(usize::from(id.unsigned_abs()))
                .ok_or("roster item definition missing")?;
            let kind = ItemDefinition(definition).category()?;
            if kind < 0 || matches!(kind, 23..=25) {
                crate::equipment::drop(warrior, slot, content, mission)?;
            }
        }
        CharacterIdentityMut(&mut warrior.bytes).refresh_save_checksums()?;
        if let Some(psi) = &state.psi_memory {
            LearnedScansMut(&mut warrior.bytes)
                .restore_selection_memory(&psi[member * 80..(member + 1) * 80]);
        }
    }
    Ok(records)
}

fn validate_warrior(warrior: &Warrior) -> Result<(), RuleError> {
    if Warrior::read(&warrior.bytes)?.name != warrior.name
        || warrior.original_sha256.len() != 64
        || !warrior
            .original_sha256
            .bytes()
            .all(|b| b.is_ascii_hexdigit())
    {
        return Err(RuleError::Invalid("invalid roster warrior".into()));
    }
    warrior.inventory()?;
    Ok(())
}
pub(crate) fn validate_squad(warriors: &[Warrior]) -> Result<(), RuleError> {
    if !(1..=6).contains(&warriors.len()) {
        return Err(RuleError::Invalid(
            "squad requires one to six warriors".into(),
        ));
    }
    for (i, warrior) in warriors.iter().enumerate() {
        validate_warrior(warrior)?;
        if warriors[..i]
            .iter()
            .any(|w| w.original_sha256 == warrior.original_sha256)
        {
            return Err(RuleError::Invalid("duplicate squad warrior".into()));
        }
    }
    Ok(())
}
pub(crate) fn admit_management(state: &Session) -> Result<(), RuleError> {
    if state.phase != Phase::Exploration
        || state.medical_continuation.is_some()
        || state
            .exploration
            .as_ref()
            .is_none_or(|p| p.presentation.is_some() || p.script.is_some())
    {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    Ok(())
}
