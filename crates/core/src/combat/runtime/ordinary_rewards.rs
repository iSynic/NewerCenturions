//! A story battle's rewards: the story rereads its extra record, then calls booty once.
use super::battle_return::{finish_prepared_battle_wrapper, refit_after_combat};
use super::booty_setup as reward;
use crate::{Event, Phase, Presentation, RuleError, Session};
use centurions_content::{Content, word};
use serde::{Deserialize, Serialize};

/// A story battle's reward in progress: the reward mode its extra record named and whether the
/// reward window is up.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrdinaryBattleRewards {
    /// None until the caller reads the reward mode from its extra record.
    mode: Option<i16>,
    #[serde(default)]
    pub(super) loot: bool,
}

impl OrdinaryBattleRewards {
    /// Whether the storage window is open for the battle's loot.
    pub(crate) fn storage_window(&self) -> bool {
        self.mode == Some(10) && self.loot
    }

    /// Refuses a saved reward whose window, cleanup, messages or mode do not fit where the game is.
    pub(crate) fn validate(&self, state: &Session) -> Result<(), RuleError> {
        let mut cleanup = state.scripted_cleanup.as_ref();
        while let Some(crate::story::cleanup::Stage::OutOfCombatGetupSurvivors { parent }) = cleanup
        {
            cleanup = parent.as_deref();
        }
        let mut page = state
            .exploration
            .as_ref()
            .and_then(|p| p.presentation.as_ref());
        while let Some(Presentation::Messages { next, .. }) = page {
            page = next.as_deref();
        }
        let valid = if self.loot {
            self.mode.is_some()
                && state.phase == Phase::Exploration
                && state.combat.is_none()
                && matches!(page, Some(Presentation::Loot))
        } else {
            state.combat.is_some()
                || super::movement_return::monster_pending(state)
                || matches!(
                    cleanup,
                    Some(
                        crate::story::cleanup::Stage::Rewards
                            | crate::story::cleanup::Stage::Complete
                            | crate::story::cleanup::Stage::OutOfCombatSurvivors { .. }
                    )
                )
        };
        let out_of_combat_mode_matches = match cleanup {
            Some(crate::story::cleanup::Stage::OutOfCombatSurvivors {
                mode,
                ordinary: true,
            }) => {
                self.mode == Some(*mode)
                    || *mode == 2
                        && self
                            .mode
                            .is_some_and(|requested| !matches!(requested, 2 | 10))
                        && state.globals.coward != 0
            }
            _ => true,
        };
        if !valid
            || !out_of_combat_mode_matches
            || matches!(cleanup,
                Some(crate::story::cleanup::Stage::OutOfCombatSurvivors { ordinary: true, .. })
                    if self.loot)
            || self.storage_window()
                && state.recovery_return != Some(crate::loot_recovery::RecoveryReturn::Ordinary)
            || state.selected_battle_treasure.is_some()
        {
            return Err("invalid ordinary battle reward continuation".into());
        }
        Ok(())
    }
}

/// Fixes the reward mode for the battle, reading it from the extra record the first time and
/// adjusting it for a defeated or fled party.
pub(super) fn freeze(state: &mut Session) -> Result<i16, RuleError> {
    let Some(pending) = state.ordinary_battle_rewards.as_ref() else {
        if state.battle_treasure != 0 {
            return Err(RuleError::from("battle treasure without its reward caller"));
        }
        return reward::effective_mode(state, 0);
    };
    let mode = match pending.mode {
        Some(mode) => mode,
        None => word(&state.globals.extra_record, 8)?,
    };
    let effective = reward::effective_mode(state, mode)?;
    if !matches!(effective, 2 | 10) {
        super::super::rewards::RewardMode::select(effective, &state.globals.treasure)?;
    }
    state.ordinary_battle_rewards.as_mut().unwrap().mode = Some(mode);
    Ok(effective)
}

/// Begins the reward of a story battle: the reward mode is fixed and, on first entry for any mode
/// but 2 and 10, the party's firearms are refitted. Returns the mode.
pub(super) fn begin(
    state: &mut Session,
    content: &Content,
) -> Result<(i16, Vec<Event>), RuleError> {
    let entering = state
        .ordinary_battle_rewards
        .as_ref()
        .is_some_and(|pending| pending.mode.is_none());
    if state.ordinary_battle_rewards.is_some() {
        suppressed(state)?;
    }
    let mode = freeze(state)?;
    if let Some(combat) = state.combat.as_ref() {
        super::reward_cleanup::validate(combat.roster.records.len() as i16)?;
    }
    let events = if entering && !matches!(mode, 2 | 10) {
        refit_after_combat(state, content)?
    } else {
        vec![]
    };
    Ok((mode, events))
}

/// Marks the reward window as up, if a story battle's reward is pending.
pub(super) fn defer(state: &mut Session) -> bool {
    let Some(pending) = state.ordinary_battle_rewards.as_mut() else {
        return false;
    };
    pending.loot = true;
    true
}

/// Closes the reward window of a story battle; refused before the battle has returned.
pub(crate) fn after_close(
    state: &mut Session,
    content: &Content,
) -> Result<Option<Vec<Event>>, RuleError> {
    let Some(pending) = &state.ordinary_battle_rewards else {
        return Ok(None);
    };
    if !pending.loot {
        return Err("ordinary battle has not returned".into());
    }
    finish(state, content).map(Some)
}

/// Finishes a story battle's reward and closes the battle, continuing a death story that waits
/// on it.
pub(crate) fn finish(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    state.ordinary_battle_rewards = None;
    state.battle_treasure = 0;
    state.scripted_cleanup = None;
    let mut events = finish_prepared_battle_wrapper(state, content)?;
    if state
        .combat_story
        .as_ref()
        .is_some_and(|story| story.waiting_child)
    {
        events.extend(super::continue_death(state, content)?);
    }
    Ok(events)
}

/// Opens the storage window for the battle's loot.
pub(super) fn storage(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let events = crate::loot_recovery::begin(
        state,
        content,
        crate::loot_recovery::RecoveryReturn::Ordinary,
    )?;
    defer(state);
    Ok(events)
}

/// Puts the dealt reward into the game: the booty list and its quality, the surviving monsters and
/// warriors, the credits, the share and the victory points.
pub(crate) fn install(
    state: &mut Session,
    content: &Content,
    mode: i16,
    rewards: crate::combat::rewards::Rewards,
) -> Result<(), RuleError> {
    // Booty clears this table; generated items and sorting also write it.
    state.globals.loot_quality = if mode == 5 || rewards.items.is_empty() {
        Some(vec![0; 1000])
    } else {
        None
    };
    if !matches!(mode, 0 | 5) {
        centurions_content::records::TreasureRecordMut(&mut state.globals.treasure)
            .consume_items()?;
    }
    if let Some(combat) = state.combat.as_mut() {
        combat.kill_party = rewards.kill_party;
        combat.roster.records = rewards.monsters;
    } else {
        state.globals.health_context.kill_party = rewards.kill_party;
    }
    if let Some(id) = rewards.last_item_id {
        state.globals.loaded_item_field32 = state.globals.loaded_items.load_item(content, id)?;
    }
    state.warriors = rewards.warriors;
    if let Some(record) = rewards.inventory_record {
        state.globals.inventory_record = record;
    }
    state.party_currency = rewards.currency;
    state.booty_tally = crate::BootyTally {
        credits: rewards.credits,
        share: rewards.share,
        victory_points: rewards.total,
    };
    for item in &mut state.booty_list {
        item.0.fill(0);
    }
    for (index, item) in rewards.items.into_iter().enumerate() {
        if index >= state.booty_list.len() {
            return Err(RuleError::Invalid(
                "battle loot exceeds booty capacity".into(),
            ));
        }
        state.booty_list[index] = item;
    }
    Ok(())
}

/// Hands out the reward of a battle that ended inside another story: storage or the out-of-combat
/// reward, unless rewards are suppressed.
pub(super) fn outer(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    if suppressed(state)? {
        return finish(state, content);
    }
    let mode = freeze(state)?;
    if mode == 10 {
        return storage(state, content);
    }
    super::out_of_combat_rewards::begin(state, content, mode, true)
}

/// Whether battle rewards are suppressed.
fn suppressed(state: &Session) -> Result<bool, RuleError> {
    Ok(state.globals.battle_rewards_suppressed != 0)
}
