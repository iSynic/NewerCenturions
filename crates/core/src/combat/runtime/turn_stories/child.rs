//! A battle started by a monster's turn story inside getup.
use crate::combat::round_stories::RoundBattleReturn;
use crate::{Content, RuleError, Session};
use serde::{Deserialize, Serialize};

/// What a battle started by a monster's turn script must restore: whether it is still waited for,
/// the outer battle's rewards and messages, and the cleanup it interrupted.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurnChildReturn {
    pub(super) waiting: bool,
    outer: Option<RoundBattleReturn>,
    cleanup: Option<crate::story::cleanup::Stage>,
}

impl TurnChildReturn {
    /// Refuses a saved child battle that interrupted unfinished cleanup or names a message, script
    /// or action that does not exist.
    pub(super) fn validate(&self, content: &Content, state: &Session) -> Result<(), RuleError> {
        if self
            .cleanup
            .as_ref()
            .is_some_and(|stage| *stage != crate::story::cleanup::Stage::Complete)
        {
            return Err("turn child suspended unfinished cleanup".into());
        }
        if let Some(outer) = &self.outer {
            let map = state.land(content).ok_or("turn child map missing")?;
            if outer.after_message != 0 {
                map.dialogue(outer.after_message)?;
            }
            if let Some(script) = outer.group {
                map.script(script)?;
            }
            if let Some(selected) = &outer.selected {
                selected.validate_suspended()?;
            }
            if let Some(npc) = &outer.npc
                && !(0..=8).contains(&npc.frame.action_index)
            {
                return Err("turn child NPC caller cursor".into());
            }
        }
        Ok(())
    }

    /// Gives back the outer battle's rewards, messages and cleanup once the child battle has
    /// returned.
    pub(super) fn restore(self, state: &mut Session) {
        if let Some(outer) = self.outer {
            state.battle_after_message = outer.after_message;
            state.battle_treasure = outer.treasure;
            state.ordinary_battle_rewards = outer.rewards;
            state.selected_battle_treasure = outer.selected;
            state.battle_group_script = outer.group;
            state.battle_npc = outer.npc;
        }
        state.scripted_cleanup = self.cleanup;
    }
}

/// Marks the turn script as waiting for a battle it started; true when a turn script owns the
/// story.
pub(crate) fn begin(state: &mut Session) -> Result<bool, RuleError> {
    if !state
        .turn_dispatch
        .as_ref()
        .is_some_and(|frame| frame.owns_script(state))
    {
        return Ok(false);
    }
    let cleanup = state.scripted_cleanup.clone();
    if cleanup
        .as_ref()
        .is_some_and(crate::story::cleanup::Stage::pauses)
    {
        return Err("turn child entered during unfinished cleanup".into());
    }
    let outer = (state.combat.is_some() || state.selected_battle_treasure.is_some()).then(|| {
        RoundBattleReturn {
            after_message: state.battle_after_message,
            treasure: state.battle_treasure,
            rewards: state.ordinary_battle_rewards.clone(),
            selected: state.selected_battle_treasure.clone(),
            group: state.battle_group_script,
            npc: state.battle_npc.clone(),
        }
    });
    let frame = state.turn_dispatch.as_mut().unwrap();
    if let Some(child) = frame.child.as_mut() {
        if child.waiting {
            return Err("turn child battle has not returned".into());
        }
        child.waiting = true;
    } else {
        frame.child = Some(TurnChildReturn {
            waiting: true,
            outer,
            cleanup,
        });
    }
    state.scripted_cleanup = None;
    Ok(true)
}

/// The story resumes only after the battle's loot, promotion and messages.
pub(crate) fn resume(state: &mut Session) -> Result<(), RuleError> {
    let in_spell = state.turn_dispatch.as_ref().is_some_and(|frame| {
        frame.child.as_ref().is_some_and(|child| child.waiting) && frame.at_depth(state)
    });
    if !in_spell || state.combat.is_some() {
        return Ok(());
    }
    if state.selected_battle_treasure.is_some()
        || state.ordinary_battle_rewards.is_some()
        || state.recovery_return.is_some()
        || state.survivors.is_some()
        || state
            .exploration
            .as_ref()
            .is_none_or(|position| position.presentation.is_some())
    {
        return Ok(());
    }
    state
        .turn_dispatch
        .as_mut()
        .unwrap()
        .child
        .as_mut()
        .unwrap()
        .waiting = false;
    Ok(())
}

/// Marks the turn script's cleanup as begun.
pub(crate) fn mark_cleanup(state: &mut Session) {
    if state
        .turn_dispatch
        .as_ref()
        .is_some_and(|frame| frame.owns_script(state))
    {
        state.turn_dispatch.as_mut().unwrap().cleanup = true;
    }
}
