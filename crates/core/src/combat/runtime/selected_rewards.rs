//! A story battle with selected treasure: two booty calls and the saved action.
use super::{battle_return::finish_selected_battle_wrapper, out_of_combat_rewards, reward_context};
use crate::{Event, Phase, Presentation, RuleError, Session};
use centurions_content::{Content, records::TreasureTable, word};
use serde::{Deserialize, Serialize};

/// Where a story battle with selected treasure stands: fighting (`Battle`), showing the battle's
/// loot or the selected treasure (`BattleLoot`, `Treasure`), dealing the battle's reward
/// (`FirstReward`, or `FinalReward` when no warrior is left), dealing the selected treasure
/// (`SecondReward`), in the getup that grants rewards once the monsters are gone, or cleaning up
/// under a script callback or a move (`Callback`, `Movement`, `MovementCallback`).
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

/// A story battle with selected treasure in progress: its stage, the script action to return to,
/// and whether the party retreated.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectedBattleTreasure {
    // Kept for old saves; the second call rereads the extra record after the first.
    treasure: i16,
    stage: Stage,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    action: Option<u8>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    retreated: bool,
}

impl SelectedBattleTreasure {
    /// Starts a selected-treasure battle for the story's treasure, noting the script action to
    /// return to.
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

    /// Refuses a saved selected-treasure battle that is suspended under a combat return but has
    /// an unknown action or is in a reward stage.
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

    /// Refuses a saved selected-treasure battle whose stage does not fit the windows and cleanup
    /// the game shows.
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
                            crate::story::cleanup::Stage::Survivors
                                | crate::story::cleanup::Stage::RewardSurvivors(_)
                                | crate::story::cleanup::Stage::Rewards
                                | crate::story::cleanup::Stage::Complete
                                | crate::story::cleanup::Stage::OutOfCombatSurvivors { .. }
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
                        Some(crate::story::cleanup::Stage::OutOfCombatSelectedSurvivors { .. })
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
                                Some(
                                    crate::story::cleanup::Stage::OutOfCombatGetupSurvivors { .. }
                                )
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
    // A victorious battle reloads the land, unless a death story already ended the
    // battle.
    if state.combat.is_some() {
        reward_context::leave_battle(state)?;
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
        .any(|warrior| warrior.record().in_battle() != 0);
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
        events.push(Event::Feedback(crate::feedback::GameFeedback::Flash {
            address: 0x401b7e,
            x: 100,
            y: 100,
            wait: 180,
            sound: 6000,
        }));
        2
    };
    events.extend(out_of_combat_rewards::begin_selected(state, content, mode)?);
    Ok(events)
}

/// Whether the story waits for its battle to return.
pub(super) fn awaiting_battle(state: &Session) -> bool {
    state
        .selected_battle_treasure
        .as_ref()
        .is_some_and(|pending| matches!(pending.stage, Stage::Battle | Stage::Callback))
}

/// Whether the turn, round or death script that called the battle still owns the story.
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

/// Notes that the battle's cleanup runs under a script callback; refused when no script owns it.
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

/// Whether a move is still unfinished: a monster's return, a walk question, a death story or a
/// returning script.
pub(super) fn movement_pending(state: &Session) -> bool {
    super::monster_return::owns_pending_return(state)
        || state.out_of_combat_walk_choice.is_some()
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
            .is_some_and(crate::story::cleanup::Stage::pauses)
}

/// Marks the battle as in a move.
pub(super) fn begin_movement(state: &mut Session) {
    if let Some(pending) = state.selected_battle_treasure.as_mut() {
        pending.stage = Stage::Movement;
    }
}

/// Returns to the battle's tally after a move; when no monsters remain the battle's rewards begin.
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
    // Victory counts matter only while the battle is still running.
    if let Some(combat) = &state.combat
        && combat.kill_counts.has_remaining()
    {
        return Ok(vec![]);
    }
    after_battle(state, content)
}

/// Whether the battle is in a move.
pub(super) fn moving(state: &Session) -> bool {
    state
        .selected_battle_treasure
        .as_ref()
        .is_some_and(|pending| matches!(pending.stage, Stage::Movement | Stage::MovementCallback))
}

/// The party retreats from a selected-treasure battle; the rewards granted when the monsters are
/// gone follow.
pub(crate) fn retreat(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    state
        .selected_battle_treasure
        .as_mut()
        .ok_or("selected retreat caller missing")?
        .retreated = true;
    begin_getup(state, content)
}

/// Leaves the battle if it is still running and grants the rewards, as getup does when the monsters
/// are gone.
pub(super) fn begin_getup(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    if state.combat.is_some() {
        reward_context::leave_battle(state)?;
    }
    state
        .selected_battle_treasure
        .as_mut()
        .ok_or("selected Getup caller missing")?
        .stage = Stage::Getup;
    out_of_combat_rewards::begin_getup(state, content)
}

/// Continues after the getup rewards: the turn story returns, then the battle's own rewards follow.
pub(super) fn after_getup(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let mut events = super::turn_stories::finish_returning(state, content)?;
    if !super::monster_return::owns_pending_return(state)
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

/// Puts the battle in the stage of the reward window that has just opened.
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
            // The treasure comes from the extra record as it stands now; a death or
            // round story may have replaced it, even with zero.
            let treasure = word(&state.globals.extra_record, 8)?;
            if treasure == 0 {
                return finish(state, content);
            }
            let map = state.land(content).ok_or("selected treasure map missing")?;
            // The treasure index is unsigned.
            let record = TreasureTable(&map.treasure)
                .record(treasure as u16 as usize)
                .map_err(|_| RuleError::Unsupported {
                    action: "story.selected_battle",
                    dependency: "additional treasure outside the native file",
                })?
                .to_vec();
            state.globals.treasure = record;
            state.globals.monsters.num_mon = 0;
            let pending = state.selected_battle_treasure.as_mut().unwrap();
            pending.treasure = treasure;
            pending.stage = Stage::SecondReward;
            out_of_combat_rewards::begin_selected(state, content, 1)
        }
        Stage::FinalReward | Stage::SecondReward | Stage::Treasure => finish(state, content),
        Stage::Battle | Stage::Callback | Stage::Movement | Stage::MovementCallback => {
            Err("selected battle has not returned".into())
        }
        Stage::Getup => Err("selected Getup has not returned".into()),
    }
}

/// Ends the story's battle: the warriors are put back in the battle, the land is reloaded and the
/// story continues, resuming the movement that was waiting.
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
        warrior.record_mut().set_in_battle(1);
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
        && !super::monster_return::owns_pending_return(state)
        && state.out_of_combat_walk_choice.is_none()
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

/// Continues after the reward window closed, once the booty is collected or stored.
pub(crate) fn after_close(
    state: &mut Session,
    content: &Content,
) -> Result<Option<Vec<Event>>, RuleError> {
    if state.selected_battle_treasure.is_none() {
        return Ok(None);
    }
    if state.booty_list.iter().any(|item| item.id() != 0) {
        return Err("collect or store the equipment before continuing".into());
    }
    returned(state, content).map(Some)
}
