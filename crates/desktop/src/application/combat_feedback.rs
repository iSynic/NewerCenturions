//! Combat feedback workflow ownership.
use super::Application;
use centurions_core::{Event, Session};
use macroquad::prelude::*;

impl Application {
    pub fn combat_notices(&self, previous_round: Option<i16>, state: &Session) -> Vec<Event> {
        let Some(combat) = &state.combat else {
            return vec![];
        };
        if previous_round.is_none_or(|round| round == combat.round) {
            return vec![];
        }
        use centurions_core::feedback::NativeFeedback;
        let mut events = vec![];
        if !self.native_preferences.enabled(14)
            && state
                .warriors
                .iter()
                .any(|w| w.record().bleeding_flag() != 0)
        {
            events.push(Event::Feedback(NativeFeedback::Flash {
                address: 0x403708,
                x: 30,
                y: 100,
                wait: 120,
                sound: 10122,
            }));
            events.push(Event::Feedback(NativeFeedback::Tutorial {
                first: 118,
                last: 118,
            }));
        }
        if !self.native_preferences.enabled(13) {
            events.push(Event::Feedback(NativeFeedback::Flash {
                address: 0x4036f2,
                x: 30,
                y: 100,
                wait: -1,
                sound: 0,
            }));
        }
        // getup1263ca..1263f4 plays the round sound twice, then closes the flash.
        for _ in 0..2 {
            events.push(Event::Feedback(NativeFeedback::Sound { resource_id: -139 }));
        }
        if !self.native_preferences.enabled(13) {
            events.push(Event::Feedback(NativeFeedback::Flash {
                address: 0x4036f1,
                x: 30,
                y: 100,
                wait: -1,
                sound: -139,
            }));
        }
        events
    }
    /// Preserve round sound order while presenting the paired nonblocking native flash.
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
        let sounds = if self.native_preferences.enabled(13) {
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
                    centurions_core::feedback::NativeFeedback::Sound { resource_id },
                ));
            } else {
                events.push(event);
            }
        }
    }
}
