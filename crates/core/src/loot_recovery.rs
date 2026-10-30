//! Booty mode 10 gives back the party's held money and displaced items.
use crate::combat::runtime;
use crate::{Content, Event, InventoryRecord, Phase, Presentation, RuleError, Session};
use serde::{Deserialize, Serialize};
mod storage;
pub use storage::Storage;

/// Who is waiting on the held property to be given back: a scenario script, ordinary play, or a
/// confiscation (in a battle or not).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryReturn {
    Scripted,
    Ordinary,
    Confiscation {
        #[serde(rename = "detached")]
        in_combat: bool,
    },
}

/// Gives the party back the money and items taken from it: the money is added to the party's and
/// the items are offered as loot.
pub(crate) fn begin(
    state: &mut Session,
    content: &Content,
    caller: RecoveryReturn,
) -> Result<Vec<Event>, RuleError> {
    if state.recovery_return.is_some() {
        return Err("recovery loot is already pending".into());
    }
    runtime::mark_turn_cleanup(state);
    let ids = state.globals.storage.prefix();
    let money = state
        .confiscated
        .as_ref()
        .map_or(state.globals.storage.wealth_store, |saved| saved.money);
    let events = runtime::refit_after_combat(state, content)?;
    state.party_currency = state.party_currency.wrapping_add(money);
    state.booty_tally = crate::BootyTally {
        credits: money,
        ..Default::default()
    };
    state.globals.storage.wealth_store = 0;
    if let Some(saved) = state.confiscated.as_mut() {
        saved.money = 0;
    }
    state.booty_list.fill(InventoryRecord([0; 22]));
    for (record, id) in state.booty_list.iter_mut().zip(ids) {
        record.set_id(id);
    }
    // The shared award adds exactly zero here; only its signed fallback can
    // change a total.
    for warrior in &mut state.warriors {
        if warrior.stamina()? > 0
            && warrior.record().computer_control()? >= 0
            && warrior.record().in_battle() != 0
        {
            let points = warrior.record().upgrade_points();
            if points < 0 {
                warrior
                    .record_mut()
                    .set_upgrade_points(i32::from(((points >> 16) as i16).wrapping_abs()));
            }
        }
    }
    let health = &mut state.globals.health_context;
    health.kill_party = 0;
    if state.combat.is_some() {
        runtime::leave_reward_battle(state)?;
    }
    state.phase = Phase::Exploration;
    if let Some(story) = state.combat_story.as_mut() {
        story.child_returned = true;
    }
    if matches!(
        caller,
        RecoveryReturn::Scripted | RecoveryReturn::Confiscation { in_combat: true }
    ) {
        state.scripted_cleanup = Some(crate::story::cleanup::Stage::Rewards);
    }
    state.recovery_return = Some(caller);
    state
        .exploration
        .as_mut()
        .ok_or("recovery map missing")?
        .presentation = Some(Presentation::Loot);
    let mut events = events;
    events.push(Event::StoryPresented);
    Ok(events)
}

/// Closes the loot window opened by a return of held property and carries on with whoever was
/// waiting for it. Does nothing when no return is pending.
pub(crate) fn close(
    state: &mut Session,
    content: &Content,
) -> Result<Option<Vec<Event>>, RuleError> {
    let Some(caller) = state.recovery_return.take() else {
        return Ok(None);
    };
    let mut events = runtime::refit_after_combat(state, content)?;
    events.extend(match caller {
        RecoveryReturn::Scripted => crate::story::cleanup::close_rewards(state, content)?
            .ok_or("scripted storage return missing")?,
        RecoveryReturn::Ordinary => {
            state.reload_battle_land()?;
            runtime::close_ordinary_rewards(state, content)?
                .ok_or("ordinary storage return missing")?
        }
        RecoveryReturn::Confiscation { in_combat } => {
            if in_combat {
                state.scripted_cleanup = Some(crate::story::cleanup::Stage::Complete);
            }
            crate::exploration::run_script(state, content)?
        }
    });
    Ok(Some(events))
}

/// Checks that the held property and any pending return are consistent.
pub(crate) fn validate(state: &Session) -> Result<(), RuleError> {
    state.globals.storage.validate()?;
    if let Some(caller) = &state.recovery_return {
        let mut page = state
            .exploration
            .as_ref()
            .and_then(|position| position.presentation.as_ref());
        while let Some(Presentation::Messages { next, .. }) = page {
            page = next.as_deref();
        }
        if state.phase != Phase::Exploration
            || state.combat.is_some()
            || !matches!(page, Some(Presentation::Loot))
            || matches!(caller, RecoveryReturn::Ordinary)
                && state
                    .ordinary_battle_rewards
                    .as_ref()
                    .is_none_or(|pending| !pending.storage_window())
            || matches!(caller, RecoveryReturn::Confiscation { .. })
                && state
                    .confiscated
                    .as_ref()
                    .is_none_or(|saved| saved.active || saved.money != 0)
            || matches!(
                caller,
                RecoveryReturn::Scripted | RecoveryReturn::Confiscation { in_combat: true }
            ) && state.scripted_cleanup != Some(crate::story::cleanup::Stage::Rewards)
        {
            return Err("invalid recovery loot return".into());
        }
    }
    Ok(())
}
