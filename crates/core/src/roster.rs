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
            // Linux reports a file parent as NotADirectory where Windows reports
            // NotFound; either way there is no roster yet.
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
                ) =>
            {
                Self::supplied(content)
            }
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
                // The roster copy has no Session to receive party-wide writes.
                let globals = crate::equipment::drop(warrior, slot, content, mission)?;
                crate::equipment::require_no_globals(&globals)?;
            }
        }
        CharacterIdentityMut(&mut warrior.bytes).refresh_save_checksums()?;
        let psi = &state.native_work.psi_memory;
        LearnedScansMut(&mut warrior.bytes)
            .restore_selection_memory(&psi[member * 80..(member + 1) * 80]);
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

/// partyselect's Add with the level's restrictions open (0x148b0c..0x148c9e):
/// warnings 0x86 for a level above the limit, 0x87 for a full squad, 0x88 for a
/// banned race and 0x89 for a banned M.O.S., checked in that order. `squad` is
/// the number of warriors already chosen.
pub fn admit_to_squad(
    restrictions: &[u8],
    warrior: &Warrior,
    squad: usize,
) -> Result<(), RuleError> {
    let limits = centurions_content::records::PartyRestrictions(restrictions);
    let record = warrior.record();
    let refuse = |index| {
        Err(RuleError::Refused {
            feedback: vec![crate::feedback::NativeFeedback::Warning { index }],
        })
    };
    let maximum = limits.maximum_level()?;
    if maximum != 0 && record.level()? > maximum {
        return refuse(0x86);
    }
    let warriors = limits.maximum_warriors()?;
    if warriors != 0 && squad as i32 >= i32::from(warriors) {
        return refuse(0x87);
    }
    let banned = |flag: Option<bool>| {
        flag.ok_or_else(|| RuleError::Invalid("restriction index outside Data RI".into()))
    };
    if banned(limits.race_banned(record.race()?))? {
        return refuse(0x88);
    }
    if banned(limits.profession_banned(record.profession()?))? {
        return refuse(0x89);
    }
    Ok(())
}
