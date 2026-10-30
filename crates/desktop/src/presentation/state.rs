//! State presentation.
use super::{Action, OutdoorAnimation, UiState};
use centurions_content::Content;
use centurions_core::{Event, Session};
use macroquad::prelude::*;

impl UiState {
    /// Resolves a booty or Items screen action against the screen's own state;
    /// choosing a category plays the click sound.
    pub fn resolve_inventory(
        &mut self,
        action: Option<Action>,
        state: &Session,
        content: &Content,
        now: f64,
    ) -> Result<Option<Action>, centurions_core::RuleError> {
        match action {
            Some(Action::Collection(action)) => {
                Ok(crate::collection::resolve(action, state, content, self))
            }
            Some(Action::Inventory(command)) => {
                let category = matches!(command, crate::inventory::InventoryAction::Category(_));
                let result = self.inventory.handle(command, state, content, now)?;
                if category {
                    self.feedback
                        .queue
                        .push_back(centurions_core::feedback::GameFeedback::Sound {
                            resource_id: 141,
                        });
                }
                Ok(result)
            }
            other => Ok(other),
        }
    }
    /// Shows a technical error in the window and adds it to the diagnostic file,
    /// which is emptied when it grows past a megabyte.
    pub fn error(&mut self, text: String) {
        self.movement.refuse();
        if self.technical_error.as_ref() != Some(&text) {
            eprintln!("{text}");
            if let Some(path) = &self.diagnostic_path {
                use std::io::Write;
                if let Some(parent) = path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                if std::fs::metadata(path).is_ok_and(|m| m.len() > 1024 * 1024) {
                    let _ = std::fs::remove_file(path);
                }
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                {
                    let _ = writeln!(file, "{text}");
                }
            }
            self.technical_error = Some(text);
        }
    }
    /// Shows why a command was refused: the rule's own message, or a technical
    /// error for any other failure.
    pub fn failure(&mut self, error: centurions_core::RuleError) {
        if matches!(
            &error,
            centurions_core::RuleError::Unsupported {
                action: "inventory.open" | "inventory.scroll",
                ..
            }
        ) && self.inspection == Some(super::Inspection::Items)
        {
            self.inspection = None;
        }
        if let centurions_core::RuleError::Refused { feedback }
        | centurions_core::RuleError::RefusedAfterEffects { feedback } = error
        {
            self.feedback.queue.extend(feedback);
            self.movement.refuse();
        } else {
            self.error(error.to_string());
        }
    }
    /// Closes the target lists after a medical item or heal and clears the scan
    /// choice pending.
    pub fn medical_result(&mut self, state: &Session, events: &[Event]) {
        if events
            .iter()
            .any(|event| matches!(event, Event::ItemUpgraded { .. }))
        {
            return;
        }
        if state.medical_continuation.is_some() || state.outdoor_psi.is_some() {
            self.selected = state.char_select_new;
        }
        self.inspection = None;
        self.psi.pending = None;
        self.targeting.medical_cancel = false;
    }

    /// Takes in the events of a command: messages and sounds to show, delays, and
    /// outdoor effects to play.
    pub fn events(&mut self, events: &[Event]) {
        for event in events {
            if matches!(event, Event::ItemUpgraded { .. }) {
                self.inventory.selected = None;
            }
            if let Event::Feedback(feedback) = event {
                self.movement.refuse();
                self.feedback.queue.push_back(feedback.clone());
            }
            if let Event::Delay { ticks } = event {
                self.delay_until =
                    self.delay_until.max(get_time()) + f64::from(*ticks) / crate::TICKS_PER_SECOND;
            }
            if let Event::OutdoorEffect {
                effect,
                x,
                y,
                size,
                delay,
            } = event
            {
                let started = self
                    .effects
                    .back()
                    .map_or(get_time(), |last| last.started + last.frame_seconds * 8.);
                self.effects.push_back(OutdoorAnimation {
                    effect: *effect,
                    x: *x,
                    y: *y,
                    size: *size,
                    started,
                    frame_seconds: f64::from((*delay).max(1)) / crate::TICKS_PER_SECOND,
                });
            }
            match event {
                Event::PictureDrawn { resource_id } => self.picture = Some(*resource_id),
                Event::ScreenRedrawn
                | Event::Moved { .. }
                | Event::MissionEntered { .. }
                | Event::CombatStarted { .. } => self.picture = None,
                _ => {}
            }
            if let Event::CollectionRejected { warning } = event {
                self.feedback
                    .queue
                    .push_back(centurions_core::feedback::GameFeedback::Warning {
                        index: *warning,
                    });
            }
        }
    }
}
