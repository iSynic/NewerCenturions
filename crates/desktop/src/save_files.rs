use crate::presentation::Action;
use centurions_content::Content;
use centurions_core::{RuleError, Session, load, save};
use std::path::{Path, PathBuf};

pub struct SaveFiles {
    pub current: Option<PathBuf>,
    pub default: PathBuf,
    pub explicit: Option<PathBuf>,
    pub protected: Vec<PathBuf>,
}

impl SaveFiles {
    pub fn perform(
        &mut self,
        action: &Action,
        state: &mut Session,
        content: &Content,
        mut choose: impl FnMut(bool, &Path) -> std::io::Result<Option<PathBuf>>,
    ) -> Result<bool, RuleError> {
        let saving = matches!(action, Action::Save | Action::FastSave);
        let path = if let Some(path) = &self.explicit {
            Some(path.clone())
        } else if matches!(action, Action::FastSave) && self.current.is_some() {
            self.current.clone()
        } else {
            choose(saving, self.current.as_ref().unwrap_or(&self.default))?
        };
        let Some(path) = path else { return Ok(false) };
        let path = if path.is_absolute() {
            path
        } else {
            std::env::current_dir()?.join(path)
        };
        if saving {
            let mut ancestor = path.as_path();
            while !ancestor.exists() {
                ancestor = ancestor
                    .parent()
                    .ok_or_else(|| RuleError::Invalid("Invalid save path.".into()))?;
            }
            let ancestor = ancestor.canonicalize()?;
            if self
                .protected
                .iter()
                .filter_map(|p| p.canonicalize().ok())
                .any(|p| ancestor.starts_with(p))
            {
                return Err(RuleError::Invalid(
                    "Cannot save inside original game data directories.".into(),
                ));
            }
            if path.exists() && load(&path, content).is_err() {
                return Err(RuleError::Invalid(
                    "Cannot overwrite a file that is not a compatible reconstructed save.".into(),
                ));
            }
            save(&path, state, content)?;
        } else {
            if path.file_name().is_some_and(|name| name == "Data I1") || path.is_dir() {
                let imported = centurions_core::native_save::import(
                    &path,
                    content,
                    &state.random,
                    (state.library_random_seed, state.library_random_draws),
                )?;
                *state = imported;
                self.current = None;
                self.explicit = None;
                return Ok(true);
            }
            *state = load(&path, content)?;
        }
        self.current = Some(path);
        Ok(true)
    }
}

#[cfg(test)]
#[path = "../tests/unit/save_files.rs"]
mod tests;
