use crate::{RandomState, RuleError, combat::death_scripts::EncounterBattleFrame};
use centurions_content::{WARRIOR_BYTES, Warrior, records::NpcBattleExtra};
use serde::{Deserialize, Serialize};

/// An NPC fight a script starts: the encounter frame to return to, the battle or range of battles,
/// the escape script, and the sound and text that open it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NpcBattleStart {
    pub frame: EncounterBattleFrame,
    pub battle: i16,
    pub maximum_battle: i16,
    pub escape_script: i16,
    pub sound: i16,
    pub text: i16,
}

/// How a script goes on after an NPC fight: abandoned, the rewards, an escape script, or the party
/// moving back.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum NpcBattleReturn {
    Abort,
    Reward,
    EscapeScript(i16),
    EscapeMove,
}

impl NpcBattleStart {
    /// Reads an NPC fight from its extra record and the encounter header.
    pub fn capture(extra: &[u8], header: &[u8], action_index: i16) -> Result<Self, RuleError> {
        if extra.len() != NpcBattleExtra::BYTES {
            return Err(RuleError::Invalid("invalid NPC battle extra record".into()));
        }
        let frame = EncounterBattleFrame::capture(header, action_index)?;
        let extra = NpcBattleExtra(extra);
        Ok(Self {
            frame,
            battle: extra.battle()?,
            maximum_battle: extra.maximum_battle()?,
            escape_script: extra.escape_script()?,
            sound: extra.sound()?,
            text: extra.text()?,
        })
    }

    /// The caller plays sound and text before selecting its battle.
    pub fn select_battle(&mut self, random: &mut RandomState) -> i16 {
        if self.maximum_battle != 0 {
            self.battle = random.range(self.battle, self.maximum_battle);
        }
        self.battle
    }

    /// How the script carries on after the fight: abandoned if aborted, the rewards if the party
    /// did not escape, otherwise the escape script or a step back.
    pub fn return_kind(&self, aborted: bool, escaped: bool) -> NpcBattleReturn {
        if aborted {
            NpcBattleReturn::Abort
        } else if !escaped {
            NpcBattleReturn::Reward
        } else if self.escape_script == -1 {
            NpcBattleReturn::EscapeMove
        } else {
            NpcBattleReturn::EscapeScript(self.escape_script)
        }
    }

    /// Called after the caller's world-map reload, before reward preparation.
    pub fn restore(&self, header: &mut [u8], aborted: bool) -> Result<Option<i16>, RuleError> {
        if aborted {
            Ok(None)
        } else {
            self.frame.restore(header).map(Some)
        }
    }
}

/// Reward completion or script loading must precede this shared caller suffix.
pub fn finish(
    outcome: NpcBattleReturn,
    warriors: &mut [Warrior],
    remembered_move: [u8; 2],
) -> Result<Option<[u8; 2]>, RuleError> {
    if warriors.is_empty()
        || warriors.len() > 6
        || warriors.iter().any(|w| w.bytes.len() != WARRIOR_BYTES)
    {
        return Err(RuleError::Invalid("invalid NPC battle party".into()));
    }
    match outcome {
        NpcBattleReturn::Abort => Ok(None),
        NpcBattleReturn::Reward | NpcBattleReturn::EscapeScript(_) => {
            for warrior in warriors {
                warrior.record_mut().set_in_battle(1);
            }
            Ok(None)
        }
        NpcBattleReturn::EscapeMove => {
            for warrior in warriors {
                let level = warrior.record().level()?;
                // The award keeps the signed 16-bit product.
                let award = i32::from(level.wrapping_mul(200));
                let current = warrior.record().remaining_experience()?;
                warrior
                    .record_mut()
                    .set_remaining_experience(current.wrapping_add(award));
            }
            Ok(Some(remembered_move))
        }
    }
}
