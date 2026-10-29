//! newland2 rereads shared EXTRA after combat, then calls booty once.
use super::battle_return::{finish_prepared_battle_wrapper, refit_after_combat};
use crate::{Event, Phase, Presentation, RuleError, Session};
use centurions_content::{Content, word};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrdinaryBattleRewards {
    /// None until the caller evaluates EXTRA443312 at142902.
    mode: Option<i16>,
    #[serde(default)]
    pub(super) loot: bool,
}

impl OrdinaryBattleRewards {
    pub(crate) fn storage_window(&self) -> bool {
        self.mode == Some(10) && self.loot
    }

    pub(crate) fn validate(&self, state: &Session) -> Result<(), RuleError> {
        let mut cleanup = state.scripted_cleanup.as_ref();
        while let Some(crate::story_cleanup::Stage::RetainedGetupSurvivors { parent }) = cleanup {
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
                || super::movement_return::npc_pending(state)
                || matches!(
                    cleanup,
                    Some(
                        crate::story_cleanup::Stage::Rewards
                            | crate::story_cleanup::Stage::Complete
                            | crate::story_cleanup::Stage::RetainedSurvivors { .. }
                    )
                )
        };
        let retained_mode_matches = match cleanup {
            Some(crate::story_cleanup::Stage::RetainedSurvivors {
                mode,
                ordinary: true,
            }) => {
                self.mode == Some(*mode)
                    || *mode == 2
                        && self
                            .mode
                            .is_some_and(|requested| !matches!(requested, 2 | 10))
                        && state.native_work.early_booty != 0
            }
            _ => true,
        };
        if !valid
            || !retained_mode_matches
            || matches!(cleanup,
                Some(crate::story_cleanup::Stage::RetainedSurvivors { ordinary: true, .. })
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

pub(super) fn freeze(state: &mut Session) -> Result<i16, RuleError> {
    let Some(pending) = state.ordinary_battle_rewards.as_ref() else {
        if state.battle_treasure != 0 {
            return Err(RuleError::from("battle treasure without its reward caller"));
        }
        return crate::combat::reward_work::effective_mode(state, 0);
    };
    let mode = match pending.mode {
        Some(mode) => mode,
        None => word(&state.native_work.extra_record, 8)?,
    };
    let effective = crate::combat::reward_work::effective_mode(state, mode)?;
    if !matches!(effective, 2 | 10) {
        super::super::rewards::RewardMode::select(effective, &state.native_work.treasure_record)?;
    }
    state.ordinary_battle_rewards.as_mut().unwrap().mode = Some(mode);
    Ok(effective)
}

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
        super::reward_cleanup::validate(combat.enemies.records.len() as i16)?;
    }
    let events = if entering && !matches!(mode, 2 | 10) {
        refit_after_combat(state, content)?
    } else {
        vec![]
    };
    Ok((mode, events))
}

pub(super) fn defer(state: &mut Session) -> bool {
    let Some(pending) = state.ordinary_battle_rewards.as_mut() else {
        return false;
    };
    pending.loot = true;
    true
}

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

pub(super) fn storage(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let events = crate::loot_recovery::begin(
        state,
        content,
        crate::loot_recovery::RecoveryReturn::Ordinary,
    )?;
    defer(state);
    Ok(events)
}

pub(crate) fn install(
    state: &mut Session,
    content: &Content,
    mode: i16,
    rewards: crate::combat::rewards::Rewards,
) -> Result<(), RuleError> {
    // Booty clears this table; generated items and sorting also write it.
    state.native_work.loot_quality = if mode == 5 || rewards.items.is_empty() {
        Some(vec![0; 1000])
    } else {
        None
    };
    if !matches!(mode, 0 | 5) {
        centurions_content::records::TreasureRecordMut(&mut state.native_work.treasure_record)
            .consume_items()?;
    }
    if let Some(combat) = state.combat.as_mut() {
        combat.unavailable = rewards.unavailable;
        combat.enemies.records = rewards.enemies;
    } else {
        state.native_work.health_context.unavailable = rewards.unavailable;
    }
    if let Some(id) = rewards.last_item_id {
        state.native_work.loaded_item_field32 =
            state.native_work.ammunition_work.load_item(content, id)?;
    }
    state.warriors = rewards.warriors;
    if let Some(record) = rewards.inventory_record {
        state.native_work.inventory_record = record;
    }
    state.party_currency = rewards.currency;
    state.booty_tally = crate::BootyTally {
        credits: rewards.credits,
        share: rewards.share,
        experience: rewards.total,
    };
    for item in &mut state.booty_items {
        item.0.fill(0);
    }
    for (index, item) in rewards.items.into_iter().enumerate() {
        if index >= state.booty_items.len() {
            return Err(RuleError::Invalid(
                "battle loot exceeds booty capacity".into(),
            ));
        }
        state.booty_items[index] = item;
    }
    Ok(())
}

pub(super) fn outer(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    if suppressed(state)? {
        return finish(state, content);
    }
    let mode = freeze(state)?;
    if mode == 10 {
        return storage(state, content);
    }
    super::retained_rewards::begin(state, content, mode, true)
}

fn suppressed(state: &Session) -> Result<bool, RuleError> {
    Ok(state.native_work.battle_rewards_suppressed != 0)
}
