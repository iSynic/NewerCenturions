//! The warrior information view and statistics recalculation.
use crate::{
    Command, Event, Phase, RuleError, Session, character, combat, warrior_management_available,
};

/// Runs the commands that show a warrior's information and recalculate statistics.
pub(in crate::commands) fn execute(
    command: Command,
    state: &mut Session,
) -> Result<Vec<Event>, RuleError> {
    match command {
        Command::ViewWarrior { warrior } => {
            if !(warrior_management_available(state)
                || state.phase == Phase::Combat && combat::controls::actor(state).is_ok())
            {
                return Err(RuleError::Refused { feedback: vec![] });
            }
            let record = state.warriors.get(warrior).ok_or_else(|| {
                RuleError::Invalid("information warrior is outside the squad".into())
            })?;
            state.globals.character_bare_hand_bonus = record.record().bare_hand_bonus()?;
            if state.combat.is_some() {
                // combatchoice restores the battle's selected warrior after the view.
                state.set_warrior_selector(state.char_select_new);
            }
            let mut presentation_random = state.random.clone();
            let first = presentation_random.bound(5) as usize - 1;
            let second = presentation_random.bound(5) as usize - 1;
            let sounds = [200, 201, 202, 207, 208];
            let mut events = vec![Event::Sound {
                resource_id: sounds[first],
            }];
            if first != second {
                events.push(Event::Sound {
                    resource_id: sounds[second],
                });
            }
            Ok(events)
        }
        Command::RecalculateStatistics { warrior } => {
            let record = state.warriors.get_mut(warrior).ok_or_else(|| {
                RuleError::Invalid("statistics warrior is outside the squad".into())
            })?;
            character::calculate_statistics(record)?;
            Ok(vec![Event::StatisticsCalculated { warrior }])
        }
        _ => unreachable!("command routed to wrong subsystem"),
    }
}
