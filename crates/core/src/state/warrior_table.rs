//! Warrior table records 0x448aa0 + body * 0xc42 that no squad member owns.
//!
//! Bodies 0..5 are the party table the save/load pair 0x15298c/0x12f2fe transfers
//! whole; records past the squad keep whatever the last writer left there.
//! Bodies 6 and 7 are upgradewarrior's scratch records 0x44d42c and 0x44e06e, and
//! bodies 8..13 the editor backup 0x44ecb0. All of them are zero at launch.
use crate::{RuleError, Session};
use centurions_content::{WARRIOR_BYTES, Warrior};
use serde::{Deserialize, Serialize};

/// Bodies 0..13: the six party records, the two Upgrade records and the backup.
const TABLE_RECORDS: usize = 14;

/// Records outside the squad as (body, bytes) sorted by body; an absent record is zero.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WarriorTable(Vec<(usize, Vec<u8>)>);

impl WarriorTable {
    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn get(&self, body: usize) -> Option<&[u8]> {
        self.0
            .iter()
            .find(|(held, _)| *held == body)
            .map(|(_, bytes)| bytes.as_slice())
    }

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

    pub(crate) fn word(&self, body: usize, offset: usize) -> Option<i16> {
        if body >= TABLE_RECORDS || offset + 2 > WARRIOR_BYTES {
            return None;
        }
        Some(self.get(body).map_or(0, |bytes| {
            i16::from_be_bytes([bytes[offset], bytes[offset + 1]])
        }))
    }

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
        let mut table = self.native_work.warrior_table.clone();
        for (body, warrior) in self.warriors.iter().enumerate().skip(warriors.len()) {
            table.store(body, warrior.bytes.clone())?;
        }
        for body in 0..warriors.len() {
            table.0.retain(|(held, _)| *held != body);
        }
        self.native_work.warrior_table = table;
        self.warriors = warriors;
        Ok(())
    }

    /// The record at 0x448aa0 + body * 0xc42 for bodies 0..13, squad or held.
    pub(crate) fn table_record(&self, body: usize) -> Option<Warrior> {
        if let Some(warrior) = self.warriors.get(body) {
            return Some(warrior.clone());
        }
        self.native_work
            .warrior_table
            .bytes(body)
            .map(|bytes| Warrior {
                name: String::new(),
                original_sha256: String::new(),
                bytes,
            })
    }

    /// Writes back records 0.. as a kernel left them: the squad, then held records.
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
            None => self.native_work.warrior_table.store(body, record.bytes),
        }
    }
}
