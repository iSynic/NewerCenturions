use std::collections::BTreeSet;
use std::path::PathBuf;

pub struct Help {
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
    fn default() -> Self {
        Self {
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

    fn save(&self) -> Result<(), String> {
        let Some(path) = &self.path else {
            return Ok(());
        };
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let bytes =
            serde_json::to_vec(&serde_json::json!({"enabled": self.enabled, "seen": self.seen}))
                .map_err(|error| error.to_string())?;
        std::fs::write(path, bytes).map_err(|error| error.to_string())
    }

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

    pub fn action(&mut self, item: usize) -> Result<i16, String> {
        let Some(page) = self.page else {
            return Ok(0);
        };
        match item {
            1 | 3 => {
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

    pub fn reset(&mut self) -> Result<(), String> {
        self.enabled = true;
        self.seen.clear();
        self.save()
    }

    pub fn resume(&mut self) -> Result<(), String> {
        self.enabled = true;
        self.save()
    }
}
