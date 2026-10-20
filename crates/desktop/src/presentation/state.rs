//! State presentation.
use super::{Action, OutdoorAnimation, UiState};
use centurions_content::Content;
use centurions_core::{Event, Session};
use macroquad::prelude::*;

impl UiState {
    pub fn resolve_inventory(
        &mut self,
        action: Option<Action>,
        state: &Session,
        content: &Content,
        now: f64,
    ) -> Result<Option<Action>, centurions_core::RuleError> {
        match action {
            Some(Action::Collection(action)) => Ok(crate::collection::resolve(action, state, self)),
            Some(Action::Inventory(command)) => {
                let category = matches!(command, crate::inventory::InventoryAction::Category(_));
                let result = self.inventory.handle(command, state, content, now)?;
                if category {
                    self.feedback
                        .push_back(centurions_core::feedback::NativeFeedback::Sound {
                            resource_id: 141,
                        });
                }
                Ok(result)
            }
            other => Ok(other),
        }
    }
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
    pub fn failure(&mut self, error: centurions_core::RuleError) {
        if let centurions_core::RuleError::Refused { feedback }
        | centurions_core::RuleError::RefusedAfterEffects { feedback } = error
        {
            self.feedback.extend(feedback);
            self.movement.refuse();
        } else {
            self.error(error.to_string());
        }
    }
    pub fn medical_result(&mut self, state: &Session, events: &[Event]) {
        if events
            .iter()
            .any(|event| matches!(event, Event::ItemUpgraded { .. }))
        {
            return;
        }
        if state.medical_continuation.is_some() || state.outdoor_psi.is_some() {
            self.selected = state.selected_warrior;
        }
        self.inspection = None;
        self.psi_pending = None;
        self.medical_cancel = false;
    }

    pub fn events(&mut self, events: &[Event]) {
        for event in events {
            if matches!(event, Event::ItemUpgraded { .. }) {
                self.inventory.selected = None;
            }
            if let Event::Feedback(feedback) = event {
                self.movement.refuse();
                self.feedback.push_back(feedback.clone());
            }
            if let Event::Delay { ticks } = event {
                self.delay_until = self.delay_until.max(get_time())
                    + f64::from(*ticks)
                        / if self.preferences.enabled(10) {
                            120.
                        } else {
                            60.
                        };
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
                    frame_seconds: f64::from((*delay).max(1))
                        / if self.preferences.enabled(9) {
                            120.
                        } else {
                            60.
                        },
                });
            }
            if let Event::CollectionRejected { warning } = event {
                self.feedback
                    .push_back(centurions_core::feedback::NativeFeedback::Warning {
                        index: *warning,
                    });
            }
        }
    }
}
