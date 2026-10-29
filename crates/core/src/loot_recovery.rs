//! booty10 returns held wealth and the displaced item WORD prefix.
use crate::{Content, Event, InventoryRecord, Phase, Presentation, RuleError, Session};
use serde::{Deserialize, Serialize};
mod work;
pub use work::RecoveryWork;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryReturn {
    Scripted,
    Ordinary,
    Confiscation { detached: bool },
}

pub(crate) fn begin(
    state: &mut Session,
    content: &Content,
    caller: RecoveryReturn,
) -> Result<Vec<Event>, RuleError> {
    if state.recovery_return.is_some() {
        return Err("recovery loot is already pending".into());
    }
    crate::combat::runtime::mark_turn_cleanup(state);
    let ids = state.native_work.recovery_work.prefix();
    let money = state
        .confiscated
        .as_ref()
        .map_or(state.native_work.recovery_work.held_money, |saved| {
            saved.money
        });
    let events = crate::combat::runtime::refit_after_combat(state, content)?;
    state.party_currency = state.party_currency.wrapping_add(money);
    // booty mode 10 moves the held money into a3 and clears -$96 and 0x443392
    // (0x10586c..0x1058bc).
    state.booty_tally = crate::BootyTally {
        credits: money,
        ..Default::default()
    };
    state.native_work.recovery_work.held_money = 0;
    if let Some(saved) = state.confiscated.as_mut() {
        saved.money = 0;
    }
    state.booty_items.fill(InventoryRecord([0; 22]));
    for (record, id) in state.booty_items.iter_mut().zip(ids) {
        record.set_id(id);
    }
    // The common award tail adds an exact zero; only its signed fallback can change a LONG.
    for warrior in &mut state.warriors {
        if warrior.stamina()? > 0
            && warrior.record().automatic_mode()? >= 0
            && warrior.record().active_flag() != 0
        {
            let points = warrior.record().upgrade_points();
            if points < 0 {
                warrior
                    .record_mut()
                    .set_upgrade_points(i32::from(((points >> 16) as i16).wrapping_abs()));
            }
        }
    }
    let health = &mut state.native_work.health_context;
    health.unavailable = 0;
    if state.combat.is_some() {
        crate::combat::runtime::detach_reward_combat(state)?;
    }
    state.phase = Phase::Exploration;
    if let Some(story) = state.combat_story.as_mut() {
        story.child_returned = true;
    }
    if matches!(
        caller,
        RecoveryReturn::Scripted | RecoveryReturn::Confiscation { detached: true }
    ) {
        state.scripted_cleanup = Some(crate::story_cleanup::Stage::Rewards);
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

pub(crate) fn close(
    state: &mut Session,
    content: &Content,
) -> Result<Option<Vec<Event>>, RuleError> {
    let Some(caller) = state.recovery_return.take() else {
        return Ok(None);
    };
    let mut events = crate::combat::runtime::refit_after_combat(state, content)?;
    events.extend(match caller {
        RecoveryReturn::Scripted => crate::story_cleanup::close_rewards(state, content)?
            .ok_or("scripted storage return missing")?,
        RecoveryReturn::Ordinary => {
            state.reload_battle_land()?;
            crate::combat::runtime::close_ordinary_rewards(state, content)?
                .ok_or("ordinary storage return missing")?
        }
        RecoveryReturn::Confiscation { detached } => {
            if detached {
                state.scripted_cleanup = Some(crate::story_cleanup::Stage::Complete);
            }
            crate::exploration::run_script(state, content)?
        }
    });
    Ok(Some(events))
}

pub(crate) fn validate(state: &Session) -> Result<(), RuleError> {
    state.native_work.recovery_work.validate()?;
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
                RecoveryReturn::Scripted | RecoveryReturn::Confiscation { detached: true }
            ) && state.scripted_cleanup != Some(crate::story_cleanup::Stage::Rewards)
        {
            return Err("invalid recovery loot return".into());
        }
    }
    Ok(())
}
