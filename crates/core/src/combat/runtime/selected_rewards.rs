//! The two booty calls and saved ACTION in newland48 (0x14551c..0x1456f0).
use super::{battle_return::finish_selected_battle_wrapper, retained_rewards, reward_context};
use crate::{Event, Phase, Presentation, RuleError, Session};
use centurions_content::{Content, records::TreasureTable, word};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum Stage {
    Battle,
    BattleLoot,
    Treasure,
    FirstReward,
    FinalReward,
    SecondReward,
    Getup,
    Callback,
    Movement,
    MovementCallback,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectedBattleTreasure {
    // Kept in old saves; the second call reads shared EXTRA after the first returns.
    treasure: i16,
    stage: Stage,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    action: Option<u8>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    retreated: bool,
}

impl SelectedBattleTreasure {
    pub(crate) fn new(state: &Session, treasure: i16) -> Result<Self, RuleError> {
        Ok(Self {
            treasure,
            stage: Stage::Battle,
            retreated: false,
            action: state
                .exploration
                .as_ref()
                .and_then(|p| p.script.as_ref())
                .map(|s| s.cursor),
        })
    }

    pub(crate) fn validate_suspended(&self) -> Result<(), RuleError> {
        if self.action.is_some_and(|action| action > 8) {
            return Err("invalid selected-battle action return".into());
        }
        if !matches!(
            self.stage,
            Stage::Battle | Stage::Callback | Stage::Movement | Stage::MovementCallback
        ) {
            return Err("selected battle child outside combat return".into());
        }
        Ok(())
    }

    pub(crate) fn validate(&self, state: &Session) -> Result<(), RuleError> {
        if self.action.is_some_and(|action| action > 8) || state.battle_treasure != 0 {
            return Err("invalid selected-battle treasure continuation".into());
        }
        let mut page = state
            .exploration
            .as_ref()
            .and_then(|p| p.presentation.as_ref());
        while let Some(Presentation::Messages { next, .. }) = page {
            page = next.as_deref();
        }
        let valid = match self.stage {
            Stage::Battle => state.phase == Phase::Combat && state.combat.is_some(),
            Stage::Callback | Stage::MovementCallback => {
                callback_owns(state)
                    && (matches!(
                        state.scripted_cleanup,
                        Some(
                            crate::story_cleanup::Stage::Survivors
                                | crate::story_cleanup::Stage::RewardSurvivors(_)
                                | crate::story_cleanup::Stage::Rewards
                                | crate::story_cleanup::Stage::Complete
                                | crate::story_cleanup::Stage::RetainedSurvivors { .. }
                        )
                    ) || state.recovery_return
                        == Some(crate::loot_recovery::RecoveryReturn::Scripted))
            }
            Stage::Movement => movement_pending(state),
            Stage::BattleLoot | Stage::Treasure => {
                state.phase == Phase::Exploration
                    && state.combat.is_none()
                    && matches!(page, Some(Presentation::Loot))
            }
            Stage::FirstReward | Stage::FinalReward | Stage::SecondReward => {
                state.phase == Phase::Exploration
                    && state.combat.is_none()
                    && state.survivors.is_some()
                    && matches!(
                        state.scripted_cleanup,
                        Some(crate::story_cleanup::Stage::RetainedSelectedSurvivors { .. })
                    )
                    && matches!(page, Some(Presentation::Survivors))
            }
            Stage::Getup => {
                state.phase == Phase::Exploration
                    && state.combat.is_none()
                    && (state.deferred_getup
                        || state.survivors.is_some()
                            && matches!(
                                state.scripted_cleanup,
                                Some(crate::story_cleanup::Stage::RetainedGetupSurvivors { .. })
                            )
                            && matches!(page, Some(Presentation::Survivors)))
            }
        };
        if !valid {
            return Err("invalid selected-battle treasure continuation".into());
        }
        Ok(())
    }
}

/// The selected caller returns from combat before either of its booty calls.
pub(super) fn after_battle(
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    // The live combat loop reloads land on its victory return. A callback that
    // already cleared COMBAT returns before that load (combat114378).
    if state.combat.is_some() {
        reward_context::detach(state)?;
        state.reload_battle_land()?;
    }
    let action = state
        .selected_battle_treasure
        .as_ref()
        .ok_or("selected battle caller missing")?
        .action;
    if let Some(action) = action {
        state
            .exploration
            .as_mut()
            .and_then(|p| p.script.as_mut())
            .ok_or("selected battle script return missing")?
            .cursor = action;
    }
    let active = state
        .warriors
        .iter()
        .any(|warrior| warrior.record().active_flag() != 0);
    state.handled_deaths.clear();
    state.phase = Phase::Exploration;
    state
        .exploration
        .as_mut()
        .ok_or("selected battle map missing")?
        .presentation = None;
    let mut events = vec![];
    let mode = if active {
        state.selected_battle_treasure.as_mut().unwrap().stage = Stage::FirstReward;
        0
    } else {
        state.selected_battle_treasure.as_mut().unwrap().stage = Stage::FinalReward;
        events.push(Event::Feedback(crate::feedback::NativeFeedback::Flash {
            address: 0x401b7e,
            x: 100,
            y: 100,
            wait: 180,
            sound: 6000,
        }));
        2
    };
    events.extend(retained_rewards::begin_selected(state, content, mode)?);
    Ok(events)
}

pub(super) fn awaiting_battle(state: &Session) -> bool {
    state
        .selected_battle_treasure
        .as_ref()
        .is_some_and(|pending| matches!(pending.stage, Stage::Battle | Stage::Callback))
}

fn callback_owns(state: &Session) -> bool {
    if state
        .turn_dispatch
        .as_ref()
        .is_some_and(|frame| frame.owns_script(state))
    {
        return true;
    }
    state
        .round_dispatch
        .as_ref()
        .is_some_and(|frame| !frame.child_battle && frame.owns_script(state))
        || state
            .combat_story
            .as_ref()
            .is_some_and(|story| story.shared_dispatch && !story.waiting_child)
}

pub(crate) fn begin_cleanup(state: &mut Session) -> Result<(), RuleError> {
    if state.selected_battle_treasure.is_none() {
        return Ok(());
    }
    let stage = state.selected_battle_treasure.as_ref().unwrap().stage;
    if !matches!(
        stage,
        Stage::Battle | Stage::Callback | Stage::Movement | Stage::MovementCallback
    ) || !callback_owns(state)
    {
        // Battle scripts run only under turn, round and death owners, and the reward
        // and Getup stages hold their windows until the selected caller returns.
        return Err("combat cleanup without a selected callback owner".into());
    }
    state.selected_battle_treasure.as_mut().unwrap().stage =
        if matches!(stage, Stage::Movement | Stage::MovementCallback) {
            Stage::MovementCallback
        } else {
            Stage::Callback
        };
    Ok(())
}

pub(super) fn movement_pending(state: &Session) -> bool {
    super::npc_return::owns_pending_return(state)
        || state.retained_walk_choice.is_some()
        || state.combat_story.as_ref().is_some_and(|story| {
            story.return_to.is_some() || story.shared_dispatch && !story.waiting_child
        })
        || state
            .round_dispatch
            .as_ref()
            .is_some_and(|frame| !frame.child_battle && frame.owns_script(state))
        || state
            .turn_dispatch
            .as_ref()
            .is_some_and(|frame| frame.returning || frame.owns_script(state))
        || state
            .scripted_cleanup
            .as_ref()
            .is_some_and(crate::story_cleanup::Stage::pauses)
}

pub(super) fn begin_movement(state: &mut Session) {
    if let Some(pending) = state.selected_battle_treasure.as_mut() {
        pending.stage = Stage::Movement;
    }
}

pub(crate) fn after_movement(
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    if !state
        .selected_battle_treasure
        .as_ref()
        .is_some_and(|pending| matches!(pending.stage, Stage::Movement | Stage::MovementCallback))
        || movement_pending(state)
    {
        return Ok(vec![]);
    }
    state.selected_battle_treasure.as_mut().unwrap().stage = Stage::Battle;
    // combat114378 tests COMBAT first; only a live loop then reads victory work.
    if let Some(combat) = &state.combat
        && combat.victory_work.has_remaining()
    {
        return Ok(vec![]);
    }
    after_battle(state, content)
}

pub(super) fn moving(state: &Session) -> bool {
    state
        .selected_battle_treasure
        .as_ref()
        .is_some_and(|pending| matches!(pending.stage, Stage::Movement | Stage::MovementCallback))
}

pub(crate) fn retreat(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    state
        .selected_battle_treasure
        .as_mut()
        .ok_or("selected retreat caller missing")?
        .retreated = true;
    begin_getup(state, content)
}

pub(super) fn begin_getup(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    if state.combat.is_some() {
        reward_context::detach(state)?;
    }
    state
        .selected_battle_treasure
        .as_mut()
        .ok_or("selected Getup caller missing")?
        .stage = Stage::Getup;
    retained_rewards::begin_getup(state, content)
}

pub(super) fn after_getup(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let mut events = super::turn_stories::finish_returning(state, content)?;
    if !super::npc_return::owns_pending_return(state)
        && state
            .selected_battle_treasure
            .as_ref()
            .is_some_and(|pending| pending.stage == Stage::Getup)
    {
        state.selected_battle_treasure.as_mut().unwrap().stage = Stage::Battle;
        events.extend(after_battle(state, content)?);
    }
    Ok(events)
}

pub(super) fn defer(state: &mut Session) -> Result<(), RuleError> {
    let pending = state
        .selected_battle_treasure
        .as_mut()
        .ok_or("selected reward return missing")?;
    pending.stage = match pending.stage {
        Stage::Battle | Stage::FirstReward => Stage::BattleLoot,
        Stage::SecondReward => Stage::Treasure,
        _ => return Err("invalid selected reward window".into()),
    };
    Ok(())
}

/// Called for both an early booty return and a collected reward window.
pub(super) fn returned(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let stage = state
        .selected_battle_treasure
        .as_ref()
        .ok_or("selected reward return missing")?
        .stage;
    match stage {
        Stage::FirstReward | Stage::BattleLoot => {
            // Native1455f6 returns before rereading WORD443312. A death or round
            // story may have replaced EXTRA, including an entry value of zero.
            let treasure = word(&state.native_work.extra_record, 8)?;
            if treasure == 0 {
                return finish(state, content);
            }
            let map = state.land(content).ok_or("selected treasure map missing")?;
            // The seek uses the native unsigned WORD index into current DataTD.
            let record = TreasureTable(&map.treasure)
                .record(treasure as u16 as usize)
                .map_err(|_| RuleError::Unsupported {
                    action: "story.selected_battle",
                    dependency: "additional treasure outside the native file",
                })?
                .to_vec();
            state.native_work.treasure_record = record;
            state.native_work.npc_backing.active_count = 0;
            let pending = state.selected_battle_treasure.as_mut().unwrap();
            pending.treasure = treasure;
            pending.stage = Stage::SecondReward;
            retained_rewards::begin_selected(state, content, 1)
        }
        Stage::FinalReward | Stage::SecondReward | Stage::Treasure => finish(state, content),
        Stage::Battle | Stage::Callback | Stage::Movement | Stage::MovementCallback => {
            Err("selected battle has not returned".into())
        }
        Stage::Getup => Err("selected Getup has not returned".into()),
    }
}

fn finish(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let pending = state
        .selected_battle_treasure
        .take()
        .ok_or("selected caller missing")?;
    state
        .exploration
        .as_mut()
        .ok_or("selected return map missing")?
        .presentation = None;
    for warrior in &mut state.warriors {
        warrior.record_mut().set_active_flag(1);
    }
    state.reload_battle_land()?;
    let mut events = finish_selected_battle_wrapper(state, content, pending.retreated)?;
    // A scheduled script can own this whole selected caller. Its time frame
    // resumes only after the selected return and that script have both ended.
    if state.selected_battle_treasure.is_none()
        && state.phase == Phase::Exploration
        && state.combat.is_none()
        && state.combat_story.is_none()
        && state.round_dispatch.is_none()
        && state.turn_dispatch.is_none()
        && !super::npc_return::owns_pending_return(state)
        && state.retained_walk_choice.is_none()
        && (state.zero_time.is_some() || state.time_continuation.is_some())
        && state
            .exploration
            .as_ref()
            .is_some_and(|position| position.script.is_none() && position.presentation.is_none())
    {
        events.extend(crate::exploration::resume_movement(state, content)?);
    }
    Ok(events)
}

pub(crate) fn after_close(
    state: &mut Session,
    content: &Content,
) -> Result<Option<Vec<Event>>, RuleError> {
    if state.selected_battle_treasure.is_none() {
        return Ok(None);
    }
    if state.booty_items.iter().any(|item| item.id() != 0) {
        return Err("collect or store the equipment before continuing".into());
    }
    returned(state, content).map(Some)
}
