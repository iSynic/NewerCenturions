use crate::{RuleError, Session};
use centurions_content::Content;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::Path;

/// The version of the saved session layout.
const CURRENT_SCHEMA_VERSION: u32 = 62;

/// A saved session with the version of the layout it was written in.
#[derive(Serialize, Deserialize)]
pub(crate) struct Save {
    /// The version of the saved layout.
    pub(crate) schema_version: u32,
    /// The saved game.
    pub(crate) session: Session,
}

/// Saves a session after checking it is valid, replacing the file atomically.
pub fn save(path: &Path, state: &Session, content: &Content) -> Result<(), RuleError> {
    state.validate(content)?;
    let bytes = serde_json::to_vec_pretty(&Save {
        schema_version: CURRENT_SCHEMA_VERSION,
        session: state.clone(),
    })?;
    atomic_write(path, &bytes)
}

/// Writes a file atomically: the bytes go to a temporary file beside it, which then replaces the
/// target.
pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), RuleError> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;
    /// Numbers the temporary files so saves never share one.
    static SAVE_SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let (temporary, mut file) = (0..100)
        .find_map(|_| {
            let sequence = SAVE_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let temporary = parent.join(format!(".session-{}-{sequence}.tmp", std::process::id()));
            match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)
            {
                Ok(file) => Some(Ok((temporary, file))),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => None,
                Err(error) => Some(Err(error)),
            }
        })
        .unwrap_or_else(|| {
            Err(std::io::Error::other(
                "could not allocate a unique save temporary file after 100 attempts",
            ))
        })?;
    let written = file.write_all(bytes).and_then(|()| file.sync_all());
    drop(file);
    // rename replaces the file on Windows and Unix, so the old save stays intact
    // until this step.
    if let Err(error) = written.and_then(|()| fs::rename(&temporary, path)) {
        let _ = fs::remove_file(&temporary);
        return Err(error.into());
    }
    Ok(())
}

/// Loads a saved session, refusing a file over the size limit or of another schema version, and
/// checks it against the scenario content.
pub fn load(path: &Path, content: &Content) -> Result<Session, RuleError> {
    let metadata = fs::metadata(path)?;
    let map_count = content.scenario.as_ref().map_or(0, |scenario| {
        scenario
            .catalog
            .values()
            .map(|level| level.maps.len().min(20))
            .sum::<usize>()
            .max(scenario.maps.len().min(20))
            .min(400)
    });
    if metadata.len() > (2_000_000 + map_count * 1_500_000) as u64 {
        return Err(RuleError::Invalid("save exceeds size limit".into()));
    }
    let saved: Save = serde_json::from_slice(&fs::read(path)?)?;
    // Only the current schema loads; older reconstruction saves are not supported.
    if saved.schema_version != CURRENT_SCHEMA_VERSION {
        return Err(RuleError::Invalid(format!(
            "unsupported save version {}",
            saved.schema_version
        )));
    }
    saved.session.validate(content)?;
    Ok(saved.session)
}
