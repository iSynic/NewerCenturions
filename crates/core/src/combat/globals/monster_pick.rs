//! The monster template: the one monster record being built or read, apart from
//! the 100 monster slots.
use crate::{RuleError, Session};
use centurions_content::combat::MONSTER_BYTES;
use serde::{Deserialize, Serialize};

/// The monster template the bestiary fills: a monster record whose bytes may not all be known.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MonsterPick {
    bytes: Vec<Option<u8>>,
}

impl Default for MonsterPick {
    /// A template with no byte known.
    fn default() -> Self {
        Self {
            bytes: vec![None; MONSTER_BYTES],
        }
    }
}

impl MonsterPick {
    /// The template is zero before any battle fills it.
    pub(crate) fn startup() -> Self {
        Self {
            bytes: vec![Some(0); MONSTER_BYTES],
        }
    }

    /// Whether no byte of the template is known.
    pub fn is_unknown(&self) -> bool {
        self.bytes.iter().all(Option::is_none)
    }

    /// The whole template when every byte is known.
    pub fn known(&self) -> Option<Vec<u8>> {
        self.bytes.iter().copied().collect()
    }

    /// Replaces the whole template, as reading the bestiary does.
    pub(crate) fn set(&mut self, record: &[u8]) {
        self.bytes = record.iter().copied().map(Some).collect();
    }

    /// The template is one monster record long.
    pub(crate) fn validate(&self) -> Result<(), RuleError> {
        if self.bytes.len() != MONSTER_BYTES {
            return Err("invalid retained monster work length".into());
        }
        Ok(())
    }

    /// Forgets the template: something rebuilt it.
    pub(crate) fn invalidate(&mut self) {
        self.bytes.fill(None);
    }

    /// Booty clears the template's reward fields when its cursor runs to 100.
    pub(crate) fn clear_reward_fields(&mut self) {
        for field in centurions_content::records::EnemyRecord::REWARD_CLEARED_FIELDS {
            self.bytes[field].fill(Some(0));
        }
    }
}

impl Session {
    /// Something rebuilt the template that this record does not follow.
    pub(crate) fn invalidate_monster_pick(&mut self) {
        self.globals.monster_pick.invalidate();
        if let Some(combat) = self.combat.as_mut() {
            combat.monster_pick_written = false;
            combat.monster_template = None;
        }
    }

    /// After a command that rewrote the template, drops the lent copy and forgets the template.
    pub(crate) fn store_monster_pick(&mut self) {
        if let Some(combat) = self.combat.as_mut()
            && std::mem::take(&mut combat.monster_pick_written)
        {
            combat.monster_template = None;
            self.globals.monster_pick.invalidate();
        }
    }

    /// Lend the known record to the combat kernels for one command.
    pub(crate) fn scope_monster_template(&mut self) {
        if let Some(combat) = self.combat.as_mut() {
            combat.monster_template = self.globals.monster_pick.known();
        }
    }

    /// Takes back the template lent for a command.
    pub(crate) fn release_monster_template(&mut self) {
        if let Some(combat) = self.combat.as_mut() {
            combat.monster_template = None;
        }
    }
}

impl crate::combat::Combat {
    /// The template as spelllist copies it, unless something rewrote it in this
    /// command.
    pub(crate) fn monster_template(&self) -> Option<&[u8]> {
        self.monster_template
            .as_deref()
            .filter(|_| !self.monster_pick_written)
    }

    /// resolvespell copies each chosen monster into the template before its race,
    /// resistance and usage tests.
    pub(crate) fn mark_monster_spell_pick(&mut self, packet: &[u8], mask: &[i16]) {
        if packet.len() == 58
            && !centurions_content::records::SpellInfoRecord(packet).is_party_duration()
            && mask
                .get(10..)
                .is_some_and(|tail| tail.iter().any(|selected| *selected > 0))
        {
            self.monster_pick_written = true;
        }
    }
}
