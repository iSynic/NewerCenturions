//! Warrior records that no squad member owns.
//!
//! Bodies 0..5 are the party table, which save and load transfer whole; records
//! past the squad keep whatever the last writer left there. Bodies 6 and 7 are
//! the held Upgrade records, the warrior before and on entering Upgrade, and
//! bodies 8..13 the editor backup. All of them are zero at launch.
use crate::{RuleError, Session};
use centurions_content::{WARRIOR_BYTES, Warrior};
use serde::{Deserialize, Serialize};

/// Bodies 0..13: the six party records, the two Upgrade records and the backup.
const TABLE_RECORDS: usize = 14;
/// The record on entering Upgrade and before each upgrade purchase.
pub(crate) const UPGRADE_PREVIOUS_BODY: usize = 6;
/// The record on entering Upgrade.
pub(crate) const UPGRADE_ENTRY_BODY: usize = 7;

/// Records outside the squad as (body, bytes) sorted by body; an absent record is zero.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WarriorTable(Vec<(usize, Vec<u8>)>);

impl WarriorTable {
    /// Whether no record outside the squad is held.
    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The record held for a body, if any.
    fn get(&self, body: usize) -> Option<&[u8]> {
        self.0
            .iter()
            .find(|(held, _)| *held == body)
            .map(|(_, bytes)| bytes.as_slice())
    }

    /// Checks that the held records are in body order, are outside the squad and are full records
    /// that are not blank.
    pub(crate) fn validate(&self, squad: usize) -> Result<(), RuleError> {
        if !self.0.is_sorted_by(|a, b| a.0 < b.0)
            || self.0.iter().any(|(body, bytes)| {
                !(squad..TABLE_RECORDS).contains(body)
                    || bytes.len() != WARRIOR_BYTES
                    || bytes.iter().all(|byte| *byte == 0)
            })
        {
            return Err("invalid warrior table record".into());
        }
        Ok(())
    }

    /// The held record of a body outside the squad, zero when nothing wrote it.
    pub(crate) fn bytes(&self, body: usize) -> Option<Vec<u8>> {
        (body < TABLE_RECORDS).then(|| {
            self.get(body)
                .map(<[u8]>::to_vec)
                .unwrap_or_else(|| vec![0; WARRIOR_BYTES])
        })
    }

    /// A two-byte value of a record; zero for a body that holds none, and none for a body or offset
    /// outside the table.
    pub(crate) fn word(&self, body: usize, offset: usize) -> Option<i16> {
        if body >= TABLE_RECORDS || offset + 2 > WARRIOR_BYTES {
            return None;
        }
        Some(self.get(body).map_or(0, |bytes| {
            i16::from_be_bytes([bytes[offset], bytes[offset + 1]])
        }))
    }

    /// Holds a record for a body; a blank record is not kept.
    pub(crate) fn store(&mut self, body: usize, bytes: Vec<u8>) -> Result<(), RuleError> {
        if body >= TABLE_RECORDS || bytes.len() != WARRIOR_BYTES {
            return Err("warrior table record outside the table".into());
        }
        if bytes.iter().all(|byte| *byte == 0) {
            self.0.retain(|(held, _)| *held != body);
        } else {
            match self.0.binary_search_by_key(&body, |(held, _)| *held) {
                Ok(index) => self.0[index].1 = bytes,
                Err(index) => self.0.insert(index, (body, bytes)),
            }
        }
        Ok(())
    }

    /// Sets a two-byte value in a record, starting a blank record for a body that holds none.
    pub(crate) fn set_word(
        &mut self,
        body: usize,
        offset: usize,
        value: i16,
    ) -> Result<(), RuleError> {
        let mut bytes = self
            .bytes(body)
            .filter(|_| offset + 2 <= WARRIOR_BYTES)
            .ok_or("warrior table WORD outside the table")?;
        bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
        self.store(body, bytes)
    }
}

impl Session {
    /// Replaces the squad in place. partyselect removes a member by shifting the
    /// later records down and leaves the old last record behind; adding one copies
    /// over the record past the squad. Records past the new squad keep their bytes.
    pub(crate) fn replace_squad(&mut self, warriors: Vec<Warrior>) -> Result<(), RuleError> {
        let mut table = self.globals.warrior_table.clone();
        for (body, warrior) in self.warriors.iter().enumerate().skip(warriors.len()) {
            table.store(body, warrior.bytes.clone())?;
        }
        for body in 0..warriors.len() {
            table.0.retain(|(held, _)| *held != body);
        }
        self.globals.warrior_table = table;
        self.warriors = warriors;
        Ok(())
    }

    /// The warrior record of body 0..13, from the squad or held here.
    pub(crate) fn table_record(&self, body: usize) -> Option<Warrior> {
        if let Some(warrior) = self.warriors.get(body) {
            return Some(warrior.clone());
        }
        self.globals.warrior_table.bytes(body).map(|bytes| Warrior {
            name: String::new(),
            original_sha256: String::new(),
            bytes,
        })
    }

    /// Writes back records from body 0 on: the squad, then held records.
    pub(crate) fn store_table_records(&mut self, records: Vec<Warrior>) -> Result<(), RuleError> {
        if records.len() < self.warriors.len() {
            return Err("warrior table records shorter than the squad".into());
        }
        for (body, record) in records.into_iter().enumerate() {
            self.store_table_record(body, record)?;
        }
        Ok(())
    }

    /// Writes back a record `table_record` returned.
    pub(crate) fn store_table_record(
        &mut self,
        body: usize,
        record: Warrior,
    ) -> Result<(), RuleError> {
        match self.warriors.get_mut(body) {
            Some(warrior) => {
                warrior.bytes = record.bytes;
                Ok(())
            }
            None => self.globals.warrior_table.store(body, record.bytes),
        }
    }
}
