//! Continuations for maintained exploration.
use super::{active_map, finish_script, run_script};
use crate::{Event, Presentation, RuleError, Session, combat};
use centurions_content::Content;

pub fn continue_messages(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let position = state
        .exploration
        .as_mut()
        .ok_or_else(|| RuleError::Invalid("no active mission".into()))?;
    let Some(Presentation::Messages { mut pages, next }) = position.presentation.take() else {
        return Err(RuleError::Invalid("no message active".into()));
    };
    if pages.is_empty() {
        return Err(RuleError::Invalid("message continuation is empty".into()));
    }
    pages.remove(0);
    if !pages.is_empty() {
        position.presentation = Some(Presentation::Messages { pages, next });
        return Ok(vec![Event::StoryPresented]);
    }
    if let Some(Presentation::ClinicEntry { percent }) = next.as_deref() {
        return crate::clinic::visit(state, *percent);
    }
    position.presentation = next.map(|page| *page);
    if position.presentation.is_some() {
        return Ok(vec![Event::StoryPresented]);
    }
    if state.phase == crate::Phase::Combat {
        crate::trace::declined(
            "exploration.continue_messages",
            "combat",
            "messages ended during combat",
        );
        Ok(vec![])
    } else {
        run_script(state, content)
    }
}

pub fn continue_story(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    continue_with_spells(state, content, false)
}

pub fn needs_spell_confirmation(state: &Session) -> bool {
    state.warriors.iter().any(|warrior| {
        warrior.stamina().unwrap_or(0) > 0
            && warrior.record().remaining_experience().unwrap() < 0
            && warrior.record().maximum_psi().unwrap_or(0) != 0
            && warrior.record().psi_school().unwrap_or(0) != 0
    })
}

pub fn keep_current_spells(
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    if !needs_spell_confirmation(state)
        || !state
            .exploration
            .as_ref()
            .is_some_and(|position| matches!(position.presentation, Some(Presentation::Loot)))
    {
        return Err(RuleError::Invalid(
            "no pending spell-promotion choice".into(),
        ));
    }
    continue_with_spells(state, content, true)
}

pub(super) fn continue_with_spells(
    state: &mut Session,
    content: &Content,
    keep_spells: bool,
) -> Result<Vec<Event>, RuleError> {
    if state
        .exploration
        .as_ref()
        .is_some_and(|p| p.presentation == Some(Presentation::Shop))
    {
        return crate::shop::close(state, content);
    }
    let loot = state
        .exploration
        .as_ref()
        .is_some_and(|position| matches!(position.presentation, Some(Presentation::Loot)));
    if loot && needs_spell_confirmation(state) && !keep_spells {
        return crate::story_promotion::begin(state, content);
    }
    let mut events = vec![];
    if loot && !state.booty_items.iter().any(|item| item.id() != 0) {
        let map = active_map(state, content)?;
        let mut library_random = crate::story_promotion::LibraryRandom {
            seed: state.library_random_seed,
            draws: state.library_random_draws,
        };
        let close = if state.recovery_return.is_some() {
            crate::story_promotion::close_recovery
        } else {
            crate::story_promotion::close
        };
        let closed = close(
            &state.warriors,
            &map.races,
            &map.profiles,
            &map.combat.as_ref().unwrap().attacks,
            true,
            &mut library_random,
        )?;
        state.warriors = closed.warriors;
        state.library_random_seed = library_random.seed;
        state.library_random_draws = library_random.draws;
        events.extend(
            closed
                .sounds
                .into_iter()
                .map(|resource_id| Event::Sound { resource_id }),
        );
        events.extend(
            closed
                .warnings
                .into_iter()
                .map(|warning| Event::CollectionRejected { warning }),
        );
        for warrior in closed.promoted {
            events.push(Event::LevelAdvanced {
                warrior,
                level: state.warriors[warrior].record().level()?,
            });
        }
    }
    events.extend(finish_story_page(state, content)?);
    Ok(events)
}

pub(crate) fn finish_story_page(
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    let mut events = vec![];
    let position = state
        .exploration
        .as_mut()
        .ok_or_else(|| RuleError::Invalid("no active mission".into()))?;
    if matches!(
        position.presentation,
        Some(
            Presentation::Question { .. }
                | Presentation::BoatQuestion { .. }
                | Presentation::Menu { .. }
                | Presentation::Typed { .. }
                | Presentation::StageChange { .. }
                | Presentation::WarriorChoice { .. }
                | Presentation::PartyPick { .. }
        )
    ) {
        return Err(RuleError::Invalid(
            "choose or cancel the current question or menu".into(),
        ));
    }
    let loot = position.presentation == Some(Presentation::Loot);
    if position.presentation.take().is_none() {
        return Err(RuleError::Invalid("no story page to continue".into()));
    }
    if state.booty_items.iter().any(|item| item.id() != 0) {
        return Err(RuleError::Invalid(
            "collect or store the equipment before continuing".into(),
        ));
    }
    if loot {
        state.booty_tally = crate::BootyTally::default();
    }
    if loot && let Some(continued) = crate::loot_recovery::close(state, content)? {
        return Ok(continued);
    }
    if loot && let Some(continued) = crate::story_cleanup::close_rewards(state, content)? {
        return Ok(continued);
    }
    if loot && let Some(continued) = combat::runtime::close_selected_rewards(state, content)? {
        events.extend(continued);
        return Ok(events);
    }
    if loot && let Some(continued) = combat::runtime::close_ordinary_rewards(state, content)? {
        events.extend(continued);
        return Ok(events);
    }
    if state.actor_replacement.is_some() {
        let (feedback, progress) = crate::story_actor::resume(state, content)?;
        events.extend(feedback);
        match progress {
            crate::story_actor::Progress::Waiting => return Ok(events),
            crate::story_actor::Progress::Stop => {
                events.extend(finish_script(state, content, false)?);
                return Ok(events);
            }
            crate::story_actor::Progress::Continue => {}
        }
    }
    if state
        .combat_story
        .as_ref()
        .is_some_and(|story| story.waiting_child)
    {
        events.extend(combat::runtime::continue_death(state, content)?);
    } else {
        events.extend(run_script(state, content)?);
    }
    Ok(events)
}
