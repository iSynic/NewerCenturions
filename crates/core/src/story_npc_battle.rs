use crate::{RandomState, RuleError, combat::death_scripts::EncounterBattleFrame};
use centurions_content::Warrior;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NpcBattleStart {
    pub frame: EncounterBattleFrame,
    pub battle: i16,
    pub maximum_battle: i16,
    pub escape_script: i16,
    pub sound: i16,
    pub text: i16,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum NpcBattleReturn {
    Abort,
    Reward,
    EscapeScript(i16),
    EscapeMove,
}

impl NpcBattleStart {
    pub fn capture(extra: &[u8], header: &[u8], action_index: i16) -> Result<Self, RuleError> {
        if extra.len() != 10 {
            return Err(RuleError::Invalid("invalid NPC battle extra record".into()));
        }
        let frame = EncounterBattleFrame::capture(header, action_index)?;
        let word = |offset| i16::from_be_bytes([extra[offset], extra[offset + 1]]);
        Ok(Self {
            frame,
            battle: word(0),
            maximum_battle: word(2),
            escape_script: word(4),
            sound: word(6),
            text: word(8),
        })
    }

    /// The caller plays sound and text before selecting its battle.
    pub fn select_battle(&mut self, random: &mut RandomState) -> i16 {
        if self.maximum_battle != 0 {
            self.battle = random.range(self.battle, self.maximum_battle);
        }
        self.battle
    }

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
    if warriors.is_empty() || warriors.len() > 6 || warriors.iter().any(|w| w.bytes.len() != 3138) {
        return Err(RuleError::Invalid("invalid NPC battle party".into()));
    }
    match outcome {
        NpcBattleReturn::Abort => Ok(None),
        NpcBattleReturn::Reward | NpcBattleReturn::EscapeScript(_) => {
            for warrior in warriors {
                warrior.bytes[0x9df] = 1;
            }
            Ok(None)
        }
        NpcBattleReturn::EscapeMove => {
            for warrior in warriors {
                let level = i16::from_be_bytes(warrior.bytes[0x3a..0x3c].try_into().unwrap());
                // The original EXT.L retains the signed low word of MULS.W.
                let award = i32::from(level.wrapping_mul(200));
                let current = u32::from_be_bytes(warrior.bytes[0x9ce..0x9d2].try_into().unwrap());
                warrior.bytes[0x9ce..0x9d2]
                    .copy_from_slice(&current.wrapping_add(award as u32).to_be_bytes());
            }
            Ok(Some(remembered_move))
        }
    }
}
