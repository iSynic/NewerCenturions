//! Current MONSTER work record at440dbc, separate from the100 NPC slots.
use crate::{RuleError, Session};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MonsterWork {
    bytes: Vec<Option<u8>>,
}

impl Default for MonsterWork {
    fn default() -> Self {
        Self {
            bytes: vec![None; 288],
        }
    }
}

impl MonsterWork {
    /// The cold DATA record at 440dbc is zero before any battle copies into it.
    pub(crate) fn startup() -> Self {
        Self {
            bytes: vec![Some(0); 288],
        }
    }

    pub fn is_unknown(&self) -> bool {
        self.bytes.iter().all(Option::is_none)
    }

    pub fn known(&self) -> Option<Vec<u8>> {
        self.bytes.iter().copied().collect()
    }

    /// A represented whole-record read, such as updatemonstermenu's last fread.
    pub(crate) fn set(&mut self, record: &[u8]) {
        self.bytes = record.iter().copied().map(Some).collect();
    }

    pub(crate) fn validate(&self) -> Result<(), RuleError> {
        if self.bytes.len() != 288 {
            return Err("invalid retained monster work length".into());
        }
        Ok(())
    }

    pub(crate) fn invalidate(&mut self) {
        self.bytes.fill(None);
    }

    /// booty1060ae/c6/d8 reaches this allocation when the final cursor is100.
    pub(crate) fn clear_reward_fields(&mut self) {
        for field in centurions_content::records::EnemyRecord::REWARD_CLEARED_FIELDS {
            self.bytes[field].fill(Some(0));
        }
    }
}

impl Session {
    /// A represented native copy/build occurred; its full resulting bytes are unowned.
    pub(crate) fn invalidate_monster_work(&mut self) {
        self.native_work.monster_work.invalidate();
        if let Some(combat) = self.combat.as_mut() {
            combat.monster_work_written = false;
            combat.monster_template = None;
        }
    }

    pub(crate) fn retain_monster_work(&mut self) {
        if let Some(combat) = self.combat.as_mut()
            && std::mem::take(&mut combat.monster_work_written)
        {
            combat.monster_template = None;
            self.native_work.monster_work.invalidate();
        }
    }

    /// Lend the known record to the combat kernels for one command.
    pub(crate) fn scope_monster_template(&mut self) {
        if let Some(combat) = self.combat.as_mut() {
            combat.monster_template = self.native_work.monster_work.known();
        }
    }

    pub(crate) fn release_monster_template(&mut self) {
        if let Some(combat) = self.combat.as_mut() {
            combat.monster_template = None;
        }
    }
}

impl super::Combat {
    /// MONSTER work as spelllist copies it, unless a writer ran in this command.
    pub(crate) fn monster_template(&self) -> Option<&[u8]> {
        self.monster_template
            .as_deref()
            .filter(|_| !self.monster_work_written)
    }

    /// resolvespell copies each selected NPC before race/resistance/usage filters.
    pub(crate) fn mark_monster_spell_work(&mut self, packet: &[u8], mask: &[i16]) {
        if packet.len() == 58
            && centurions_content::records::AttackPacket(packet).targeting_mode() != 7
            && mask
                .get(10..)
                .is_some_and(|tail| tail.iter().any(|selected| *selected > 0))
        {
            self.monster_work_written = true;
        }
    }
}
