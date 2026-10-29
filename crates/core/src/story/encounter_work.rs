//! Headers at 0x443432/0x443396 and the nine shared 256-byte label buffers.
//! File edits replace a header; redraws keep the labels left by the last loader.
use crate::{RuleError, story::MENU_RECORD_BYTES, typed_encounter};
use centurions_content::span;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncounterWork {
    simple: Vec<u8>,
    typed: Vec<u8>,
    labels: [Vec<u8>; 9],
}

impl Default for EncounterWork {
    fn default() -> Self {
        Self::startup()
    }
}

impl EncounterWork {
    /// CODE1/DATA initializes both headers and all nine shared labels to zero.
    pub(crate) fn startup() -> Self {
        Self {
            simple: vec![0; 106],
            typed: vec![0; 156],
            labels: std::array::from_fn(|_| vec![0; 256]),
        }
    }
    pub(crate) fn validate(&self) -> Result<(), RuleError> {
        if self.simple.len() != 106
            || self.typed.len() != 156
            || self.labels.iter().any(|v| v.len() > 256)
        {
            return Err(RuleError::Invalid(
                "invalid retained encounter buffers".into(),
            ));
        }
        Ok(())
    }
    pub fn header(&self, typed: bool) -> &[u8] {
        if typed { &self.typed } else { &self.simple }
    }
    pub(crate) fn replace_header(&mut self, typed: bool, header: &[u8]) -> Result<(), RuleError> {
        if header.len() != if typed { 156 } else { 106 } {
            return Err(RuleError::Invalid("encounter header size".into()));
        }
        *if typed {
            &mut self.typed
        } else {
            &mut self.simple
        } = header.to_vec();
        Ok(())
    }
    pub fn load(&mut self, table: &[u8], id: i16, typed: bool) -> Result<(), RuleError> {
        let stride = if typed {
            typed_encounter::RECORD_BYTES
        } else {
            MENU_RECORD_BYTES
        };
        let id =
            usize::try_from(id).map_err(|_| RuleError::Invalid("negative encounter".into()))?;
        let record = span(table, id * stride, stride)?;
        let (header, width, count) = if typed { (156, 40, 9) } else { (106, 80, 4) };
        self.replace_header(typed, &record[..header])?;
        for slot in 0..count {
            let mut label = vec![0; 256];
            label[..width]
                .copy_from_slice(&record[header + slot * width..header + (slot + 1) * width]);
            self.labels[slot] = label;
        }
        Ok(())
    }
    /// resetgamefiles clears simple labels; original save/load copy without clearing.
    /// A short fread still replaces its available prefix before returning zero.
    pub(crate) fn copy_file(
        &mut self,
        table: &[u8],
        typed: bool,
        clear: bool,
    ) -> Result<(), RuleError> {
        let (header, width, count) = if typed { (156, 40, 9) } else { (106, 80, 4) };
        let mut offset = 0;
        while offset < table.len() {
            let size = header.min(table.len() - offset);
            if size == header {
                self.replace_header(typed, &table[offset..offset + size])?;
            } else {
                let mut retained = self.header(typed).to_vec();
                retained[..size].copy_from_slice(&table[offset..offset + size]);
                self.replace_header(typed, &retained)?;
                break;
            }
            offset += size;
            for slot in 0..count {
                if clear {
                    self.labels[slot] = vec![0; 256];
                }
                let size = width.min(table.len() - offset);
                if size != 0 {
                    let label = &mut self.labels[slot];
                    let overlap = size.min(label.len());
                    label[..overlap].copy_from_slice(&table[offset..offset + overlap]);
                    if size > overlap {
                        label.extend_from_slice(&table[offset + overlap..offset + size]);
                    }
                    offset += size;
                }
            }
        }
        Ok(())
    }
    pub(crate) fn reset_scenario(
        &mut self,
        scenario: &centurions_content::mission::ScenarioContent,
    ) -> Result<(), RuleError> {
        for level in 0..3 {
            let records = if scenario.level == level {
                Some(scenario)
            } else {
                scenario.catalog.get(&(scenario.name.clone(), level))
            };
            if let Some(map) = records.and_then(|records| records.maps.first()) {
                self.copy_file(&map.menus, false, true)?;
                self.copy_file(&map.typed_encounters, true, false)?;
            }
        }
        Ok(())
    }
    pub(crate) fn clear_disabled_labels(&mut self) -> Result<(), RuleError> {
        let disabled: Vec<_> = self.header(false)[96..100]
            .iter()
            .map(|v| *v == 0)
            .collect();
        for (slot, disabled) in disabled.into_iter().enumerate() {
            if disabled {
                self.labels[slot] = vec![0; 256];
            }
        }
        Ok(())
    }
    pub fn record(&self, typed: bool) -> Result<Vec<u8>, RuleError> {
        let mut record = self.header(typed).to_vec();
        let (width, count) = if typed { (40, 9) } else { (80, 4) };
        for label in &self.labels[..count] {
            let label = label
                .get(..width)
                .ok_or_else(|| RuleError::from("encounter label shorter than its width"))?;
            record.extend_from_slice(label);
        }
        Ok(record)
    }
    pub(crate) fn eliminate(&mut self, branches: u8) -> Result<(), RuleError> {
        let mut header = self.header(true).to_vec();
        typed_encounter::eliminate_branches(&mut header, branches);
        self.replace_header(true, &header)
    }
    pub(crate) fn disable_choice(&mut self, choice: i16) -> Result<(), RuleError> {
        if !(1..=4).contains(&choice) {
            return Err(RuleError::Invalid("native menu item is unavailable".into()));
        }
        let mut header = self.header(false).to_vec();
        header[95 + choice as usize] = 0;
        self.replace_header(false, &header)
    }
}
