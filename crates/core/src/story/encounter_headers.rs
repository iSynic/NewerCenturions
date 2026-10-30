//! The simple and typed encounter headers and the nine shared 256-byte labels.
//! File edits replace a header; redraws keep the labels the last encounter
//! loaded.
use crate::{RuleError, story::MENU_RECORD_BYTES, typed_encounter};
use centurions_content::records::EncounterMenuMut;
use centurions_content::span;
use serde::{Deserialize, Serialize};

/// Size of a simple (menu) encounter header.
pub(crate) const SIMPLE_HEADER_BYTES: usize = 106;
/// Size of a typed encounter header.
pub(crate) const TYPED_HEADER_BYTES: usize = 156;

/// The header length of a typed or simple encounter.
pub(crate) fn header_bytes(typed: bool) -> usize {
    if typed {
        TYPED_HEADER_BYTES
    } else {
        SIMPLE_HEADER_BYTES
    }
}

/// The simple and typed encounter headers and the nine labels they share.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncounterHeaders {
    simple: Vec<u8>,
    typed: Vec<u8>,
    labels: [Vec<u8>; 9],
}

impl Default for EncounterHeaders {
    /// A new game's headers and labels.
    fn default() -> Self {
        Self::startup()
    }
}

impl EncounterHeaders {
    /// A new game starts with both headers and all nine labels empty.
    pub(crate) fn startup() -> Self {
        Self {
            simple: vec![0; SIMPLE_HEADER_BYTES],
            typed: vec![0; TYPED_HEADER_BYTES],
            labels: std::array::from_fn(|_| vec![0; 256]),
        }
    }
    /// Checks that the headers and labels have their full sizes.
    pub(crate) fn validate(&self) -> Result<(), RuleError> {
        if self.simple.len() != SIMPLE_HEADER_BYTES
            || self.typed.len() != TYPED_HEADER_BYTES
            || self.labels.iter().any(|v| v.len() > 256)
        {
            return Err(RuleError::Invalid(
                "invalid retained encounter buffers".into(),
            ));
        }
        Ok(())
    }
    /// The simple or the typed encounter header.
    pub fn header(&self, typed: bool) -> &[u8] {
        if typed { &self.typed } else { &self.simple }
    }
    /// Replaces a header with one of the right size.
    pub(crate) fn replace_header(&mut self, typed: bool, header: &[u8]) -> Result<(), RuleError> {
        if header.len() != header_bytes(typed) {
            return Err(RuleError::Invalid("encounter header size".into()));
        }
        *if typed {
            &mut self.typed
        } else {
            &mut self.simple
        } = header.to_vec();
        Ok(())
    }
    /// Loads an encounter's header and labels from the scenario's table.
    pub fn load(&mut self, table: &[u8], id: i16, typed: bool) -> Result<(), RuleError> {
        let stride = if typed {
            typed_encounter::RECORD_BYTES
        } else {
            MENU_RECORD_BYTES
        };
        let id =
            usize::try_from(id).map_err(|_| RuleError::Invalid("negative encounter".into()))?;
        let record = span(table, id * stride, stride)?;
        let (header, width, count) = if typed {
            (TYPED_HEADER_BYTES, 40, 9)
        } else {
            (SIMPLE_HEADER_BYTES, 80, 4)
        };
        self.replace_header(typed, &record[..header])?;
        for slot in 0..count {
            let mut label = vec![0; 256];
            label[..width]
                .copy_from_slice(&record[header + slot * width..header + (slot + 1) * width]);
            self.labels[slot] = label;
        }
        Ok(())
    }
    /// Resetting the game files clears the simple labels; saving and loading copy
    /// them without clearing. A short read still replaces what it read.
    pub(crate) fn copy_file(
        &mut self,
        table: &[u8],
        typed: bool,
        clear: bool,
    ) -> Result<(), RuleError> {
        let (header, width, count) = if typed {
            (TYPED_HEADER_BYTES, 40, 9)
        } else {
            (SIMPLE_HEADER_BYTES, 80, 4)
        };
        let mut offset = 0;
        while offset < table.len() {
            let size = header.min(table.len() - offset);
            if size == header {
                self.replace_header(typed, &table[offset..offset + size])?;
            } else {
                let mut header = self.header(typed).to_vec();
                header[..size].copy_from_slice(&table[offset..offset + size]);
                self.replace_header(typed, &header)?;
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
    /// Loads the headers of a scenario's levels when the scenario starts.
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
    /// Blanks the labels of the menu entries that are disabled.
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
    /// The header followed by the labels the encounter shows, forming the record of the simple or
    /// typed encounter.
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
    /// Turns each eliminated branch of the typed encounter into a lone End Script action.
    pub(crate) fn eliminate(&mut self, branches: u8) -> Result<(), RuleError> {
        let mut header = self.header(true).to_vec();
        typed_encounter::eliminate_branches(&mut header, branches);
        self.replace_header(true, &header)
    }
    /// Disables one of the four choices of the simple encounter.
    pub(crate) fn disable_choice(&mut self, choice: i16) -> Result<(), RuleError> {
        if !(1..=4).contains(&choice) {
            return Err(RuleError::Invalid("native menu item is unavailable".into()));
        }
        let mut header = self.header(false).to_vec();
        EncounterMenuMut(&mut header).clear_choice(choice as usize);
        self.replace_header(false, &header)
    }
}
