use crate::{RuleError, Session};
use centurions_content::Content;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::Path;

const CURRENT_SCHEMA_VERSION: u32 = 62;

#[derive(Serialize, Deserialize)]
pub(crate) struct Save {
    pub(crate) schema_version: u32,
    pub(crate) session: Session,
}

pub fn save(path: &Path, state: &Session, content: &Content) -> Result<(), RuleError> {
    state.validate(content)?;
    let bytes = serde_json::to_vec_pretty(&Save {
        schema_version: CURRENT_SCHEMA_VERSION,
        session: state.clone(),
    })?;
    atomic_write(path, &bytes)
}

pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), RuleError> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;
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
    // std::fs::rename uses replacement on Windows and Unix, keeping the old save intact until this step.
    if let Err(error) = written.and_then(|()| fs::rename(&temporary, path)) {
        let _ = fs::remove_file(&temporary);
        return Err(error.into());
    }
    Ok(())
}

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
