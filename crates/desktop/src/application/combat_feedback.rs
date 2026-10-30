//! The messages and sounds that open a new combat round.
use super::Application;
use centurions_core::{Event, Session};
use macroquad::prelude::*;

impl Application {
    /// The notices that open a new combat round: the warning about bleeding warriors with its
    /// tutorial page, then the round sound and the round message; the warning and the message
    /// can each be turned off in the preferences.
    pub fn combat_notices(&self, previous_round: Option<i16>, state: &Session) -> Vec<Event> {
        let Some(combat) = &state.combat else {
            return vec![];
        };
        if previous_round.is_none_or(|round| round == combat.combat_round) {
            return vec![];
        }
        use centurions_core::feedback::GameFeedback;
        let mut events = vec![];
        if !self
            .preferences
            .enabled(crate::preferences::pref::IGNORE_BLEEDING_WARRIORS)
            && state.warriors.iter().any(|w| w.record().bleeding() != 0)
        {
            events.push(Event::Feedback(GameFeedback::Flash {
                address: 0x403708,
                x: 30,
                y: 100,
                wait: 120,
                sound: 10122,
            }));
            events.push(Event::Feedback(GameFeedback::Tutorial {
                first: 118,
                last: 118,
            }));
        }
        if !self
            .preferences
            .enabled(crate::preferences::pref::IGNORE_NEXT_ROUND_MESSAGE)
        {
            events.push(Event::Feedback(GameFeedback::Flash {
                address: 0x4036f2,
                x: 30,
                y: 100,
                wait: -1,
                sound: 0,
            }));
        }
        // getup plays the round sound twice, then closes the flash.
        for _ in 0..2 {
            events.push(Event::Feedback(GameFeedback::Sound { resource_id: -139 }));
        }
        if !self
            .preferences
            .enabled(crate::preferences::pref::IGNORE_NEXT_ROUND_MESSAGE)
        {
            events.push(Event::Feedback(GameFeedback::Flash {
                address: 0x4036f1,
                x: 30,
                y: 100,
                wait: -1,
                sound: -139,
            }));
        }
        events
    }
    /// Places the round notices at the round sound, keeping the sounds in order;
    /// the round message flash does not wait for a click.
    pub fn prepare_combat_round(
        &self,
        previous: Option<i16>,
        state: &Session,
        events: &mut Vec<Event>,
    ) {
        let notices = self.combat_notices(previous, state);
        if notices.is_empty() {
            return;
        }
        let Some(start) = events
            .iter()
            .position(|event| matches!(event, Event::Sound { resource_id: -139 }))
        else {
            return;
        };
        let mut tail = events.split_off(start).into_iter().peekable();
        let sounds = if self
            .preferences
            .enabled(crate::preferences::pref::IGNORE_NEXT_ROUND_MESSAGE)
        {
            2
        } else {
            3
        };
        while let Some(event) = tail.next() {
            if matches!(event, Event::Sound { resource_id: -139 }) {
                events.extend(self.combat_notices(previous, state));
                for _ in 1..sounds {
                    if tail
                        .peek()
                        .is_some_and(|e| matches!(e, Event::Sound { resource_id: -139 }))
                    {
                        tail.next();
                    } else {
                        break;
                    }
                }
            } else if let Some(resource_id) = event.sound() {
                events.push(Event::Feedback(
                    centurions_core::feedback::GameFeedback::Sound { resource_id },
                ));
            } else {
                events.push(event);
            }
        }
    }
}
