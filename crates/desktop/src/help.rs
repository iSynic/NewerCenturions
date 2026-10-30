use std::collections::BTreeSet;
use std::path::PathBuf;

mod page;
pub use page::draw as draw_page;

/// Popup-help Done button (item 3 of the PICT 168 strip); Escape and Enter press it.
pub const DONE: usize = 3;

/// The popup help: whether it is turned on, the tutorial page showing, the pages
/// already seen, and the settings file.
pub struct Help {
    pub(crate) settings: serde_json::Value,
    pub enabled: bool,
    pub page: Option<usize>,
    seen: BTreeSet<usize>,
    first: usize,
    last: usize,
    path: Option<PathBuf>,
}

#[cfg(test)]
#[path = "../tests/unit/help_state.rs"]
mod tests;

impl Default for Help {
    /// Popup help on, with no page showing and none seen.
    fn default() -> Self {
        Self {
            settings: serde_json::json!({}),
            enabled: true,
            page: None,
            seen: BTreeSet::new(),
            first: 0,
            last: 0,
            path: None,
        }
    }
}

impl Help {
    /// Reads the settings file at `path`: whether help is on and the pages seen.
    pub fn preferences(&mut self, path: PathBuf) -> Result<(), String> {
        self.path = Some(path.clone());
        if !path.exists() {
            return Ok(());
        }
        if std::fs::metadata(&path)
            .map_err(|error| error.to_string())?
            .len()
            > 8192
        {
            return Err("Popup-help preferences exceed their size limit.".into());
        }
        let bytes = std::fs::read(path).map_err(|error| error.to_string())?;
        let value: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        if !value.is_object() {
            return Err("Invalid UI preferences object.".into());
        }
        self.settings = value.clone();
        self.enabled = value["enabled"].as_bool().unwrap_or(true);
        self.seen = value["seen"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|page| {
                page.as_u64()
                    .filter(|page| *page < 256)
                    .map(|page| page as usize)
            })
            .collect();
        Ok(())
    }

    /// Writes the settings file, if one is set.
    pub(crate) fn save(&self) -> Result<(), String> {
        let Some(path) = &self.path else {
            return Ok(());
        };
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let mut value = self.settings.clone();
        value["enabled"] = self.enabled.into();
        value["seen"] = serde_json::to_value(&self.seen).map_err(|error| error.to_string())?;
        let bytes = serde_json::to_vec(&value).map_err(|error| error.to_string())?;
        centurions_core::atomic_write(path, &bytes).map_err(|error| error.to_string())
    }

    /// Shows the first tutorial page of `first` through `last` that has not been
    /// seen yet, if help is on.
    pub fn open(&mut self, first: usize, last: usize) -> Result<(), String> {
        if first > last || last >= 256 {
            return Err("Invalid popup-help page range.".into());
        }
        if !self.enabled {
            return Ok(());
        }
        if let Some(page) = (first..=last).find(|page| !self.seen.contains(page)) {
            self.first = page;
            self.last = last;
            self.page = Some(page);
            self.seen.insert(page);
            self.save()?;
        }
        Ok(())
    }

    /// Carries out a click on the help strip: Done, the previous and next pages, or
    /// turning help off. Returns the sound to play.
    pub fn action(&mut self, item: usize) -> Result<i16, String> {
        let Some(page) = self.page else {
            return Ok(0);
        };
        match item {
            1 | DONE => {
                self.seen.extend(self.first..=self.last);
                self.page = None;
            }
            4 => self.page = Some(page.saturating_sub(1).max(self.first)),
            5 => self.page = (page < self.last).then_some(page + 1),
            6 => {
                self.enabled = false;
                self.page = None;
            }
            _ => return Err("Unknown popup-help control.".into()),
        }
        if let Some(page) = self.page {
            self.seen.insert(page);
        }
        self.save()?;
        Ok(if item == 1 || item == 3 { 148 } else { 141 })
    }

    /// Turns help on and forgets which pages were seen.
    pub fn reset(&mut self) -> Result<(), String> {
        self.enabled = true;
        self.seen.clear();
        self.save()
    }

    /// Turns help on again, keeping the pages seen.
    pub fn resume(&mut self) -> Result<(), String> {
        self.enabled = true;
        self.save()
    }
}
