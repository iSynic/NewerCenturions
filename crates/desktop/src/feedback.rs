use crate::{assets::Assets, presentation::UiState};
use centurions_core::{Event, feedback::GameFeedback};
use macroquad::prelude::*;

/// The messages waiting to be shown, the one that stays on screen, and how long the current one
/// lasts.
#[derive(Default)]
pub struct Feedback {
    pub queue: std::collections::VecDeque<GameFeedback>,
    pub persistent: Option<GameFeedback>,
    pub started: Option<f64>,
    pub until: f64,
}

impl UiState {
    /// Keeps the capture targeting prompt up while a warrior chooses a monster to
    /// capture, without repeating its entry sound or help.
    pub fn prepare_capture_prompt(&mut self, state: &centurions_core::Session) {
        let capturing = state
            .combat
            .as_ref()
            .is_some_and(|c| c.capture_actor.is_some());
        let own_prompt = matches!(
            self.feedback.persistent,
            Some(GameFeedback::Flash {
                address: 0x4024de,
                ..
            })
        );
        if capturing && self.feedback.persistent.is_none() {
            // Restored targeting redraws its prompt without repeating entry sound or help.
            self.feedback.persistent = Some(GameFeedback::Flash {
                address: 0x4024de,
                x: 30,
                y: 300,
                wait: -1,
                sound: 0,
            });
        } else if !capturing && own_prompt {
            self.feedback.persistent = None;
        }
    }

    /// Reconstruct a saved script's owned modal; other queued feedback keeps its order.
    pub fn prepare_script_pause(&mut self, state: &centurions_core::Session) -> bool {
        if !state.exploration.as_ref().is_some_and(|p| {
            matches!(
                p.presentation,
                Some(centurions_core::Presentation::ScriptPause)
            )
        }) {
            return false;
        }
        let pause = centurions_core::feedback::script_pause();
        if self.feedback.queue.is_empty() {
            self.feedback.queue.push_back(pause.clone());
            self.feedback.started = None;
        }
        self.feedback.queue.front() == Some(&pause)
    }

    /// Starts the message at the head of the queue and returns the sound that goes
    /// with it, if any.
    pub fn feedback_sound(&mut self, now: f64) -> Option<Event> {
        let feedback = self.feedback.queue.front()?;
        if self.feedback.started.is_some() {
            return None;
        }
        self.feedback.started = Some(now);
        let resource_id = match feedback {
            GameFeedback::Flash { sound, .. } | GameFeedback::ResourceFlash { sound, .. } => *sound,
            GameFeedback::Combat { .. } => -143,
            GameFeedback::CombatResult { code, audible, .. } => {
                centurions_core::feedback::combat_result(*code)
                    .filter(|_| *audible)
                    .map_or(0, |(_, sound)| sound)
            }
            GameFeedback::Sound { resource_id } => *resource_id,
            GameFeedback::Warning { .. } | GameFeedback::EquipmentLevel { .. } => 6000,
            GameFeedback::Silent
            | GameFeedback::CombatResourceResult { .. }
            | GameFeedback::Tutorial { .. }
            | GameFeedback::DialogText { .. }
            | GameFeedback::OutdoorBodyStatus { .. } => 0,
        };
        (resource_id != 0).then_some(Event::Sound { resource_id })
    }

    /// Answers a click or key on the message showing: an error clears, and a message
    /// waiting to be dismissed moves on to the next. Returns whether the input was
    /// taken.
    pub fn feedback_input(&mut self, now: f64, pressed: bool) -> bool {
        if self.technical_error.is_some() {
            if pressed {
                self.technical_error = None;
            }
            return true;
        }
        let Some(feedback) = self.feedback.queue.front() else {
            return false;
        };
        if matches!(feedback, GameFeedback::Flash { wait, .. } | GameFeedback::ResourceFlash { wait, .. } if *wait < 0)
        {
            if matches!(feedback, GameFeedback::Flash { sound, .. } | GameFeedback::ResourceFlash { sound, .. } if *sound < 0)
                && now < self.feedback.until
            {
                return true;
            }
            self.feedback.persistent = self.feedback.queue.pop_front().filter(|feedback| {
                !matches!(
                    feedback,
                    GameFeedback::Flash {
                        address: 0x40250d | 0x4036f1,
                        ..
                    }
                )
            });
            self.feedback.started = None;
            return !self.feedback.queue.is_empty();
        }
        if let GameFeedback::Tutorial { first, last } = feedback {
            self.help_request = Some((*first, *last));
            self.feedback.queue.pop_front();
            self.feedback.started = None;
            return true;
        }
        if matches!(feedback, GameFeedback::Combat { .. }) {
            if now < self.feedback.until {
                return true;
            }
            if let Some(GameFeedback::Combat { kind }) = self.feedback.queue.pop_front() {
                use centurions_core::feedback::CombatRefusal::*;
                let page = match kind {
                    NeedTarget => 90,
                    OutOfRange => 86,
                    Blocked => 91,
                    InvalidTarget => 88,
                    TooManyTargets => 87,
                    NotEnoughAmmo => 89,
                };
                self.help_request = Some((page, page));
            }
            self.feedback.started = None;
            return false;
        }
        let started = self.feedback.started.unwrap_or(now);
        let done = match feedback {
            GameFeedback::CombatResult { .. } | GameFeedback::CombatResourceResult { .. } => {
                now - started >= 20. / 60.
            }
            GameFeedback::OutdoorBodyStatus { wait, .. } => now - started > f64::from(*wait) / 60.,
            GameFeedback::DialogText { wait, .. } => now - started >= f64::from(*wait) / 60.,
            GameFeedback::Flash { wait, .. } | GameFeedback::ResourceFlash { wait, .. }
                if *wait > 0 =>
            {
                now - started >= f64::from(*wait) / 60.
            }
            GameFeedback::Flash { .. }
            | GameFeedback::ResourceFlash { .. }
            | GameFeedback::EquipmentLevel { .. } => pressed && now - started >= 15. / 60.,
            GameFeedback::Sound { .. } => now >= self.feedback.until,
            GameFeedback::Silent => true,
            GameFeedback::Tutorial { .. } => unreachable!(),
            GameFeedback::Warning { .. } => pressed && now - started >= 10. / 60.,
            GameFeedback::Combat { .. } => unreachable!(),
        };
        if done {
            self.feedback.queue.pop_front();
            self.feedback.started = None;
        }
        true
    }

    /// Draws the technical error, or else the message at the head of the queue (or
    /// the one that stays up), at `warning_origin`.
    pub fn draw_feedback(&self, assets: &Assets, warning_origin: Vec2) {
        if let Some(error) = &self.technical_error {
            draw_rectangle(40., 180., 720., 240., Color::from_rgba(25, 25, 25, 255));
            draw_rectangle_lines(40., 180., 720., 240., 2., RED);
            assets.wrapped(error, Rect::new(56., 200., 686., 196.), 14, 18.);
            return;
        }
        let Some(feedback) = self
            .feedback
            .queue
            .front()
            .filter(|f| !matches!(f, GameFeedback::Sound { .. } | GameFeedback::Silent))
            .or(self.feedback.persistent.as_ref())
        else {
            return;
        };
        let (text, rect, picture) = match feedback {
            GameFeedback::OutdoorBodyStatus { permanent, .. } => {
                let address = if *permanent { 0x403294 } else { 0x403286 };
                if let Ok(text) = assets.game_text.text(address, true) {
                    assets.text(&text, 400., 30., 12, BLACK);
                }
                return;
            }
            GameFeedback::DialogText {
                dialog,
                item,
                address,
                ..
            } => {
                if let Ok(text) = assets.game_text.text(*address, true) {
                    assets.dialog_text(&text, assets.dialog_rect(*dialog, usize::from(*item)));
                }
                return;
            }
            GameFeedback::ResourceFlash {
                resource_id,
                index,
                x,
                y,
                ..
            } => (
                assets
                    .reference_strings
                    .get(resource_id)
                    // GetIndString leaves index 0 and indexes past the list empty.
                    .map(|list| {
                        index
                            .checked_sub(1)
                            .and_then(|index| list.get(index))
                            .cloned()
                            .unwrap_or_default()
                    })
                    .ok_or_else(|| {
                        centurions_content::ContentError::Invalid(
                            "original flash resource missing".into(),
                        )
                    }),
                Rect::new(f32::from(*x), f32::from(*y) + 20., 260., 62.),
                156,
            ),
            GameFeedback::Flash { address, x, y, .. } => (
                assets.game_text.text(*address, false),
                Rect::new(f32::from(*x), f32::from(*y) + 20., 260., 62.),
                156,
            ),
            GameFeedback::EquipmentLevel { level } => (
                assets
                    .game_text
                    .text(crate::game_strings::MINIMUM_SKILL_LEVEL, false)
                    .map(|prefix| format!("{prefix}{level}")),
                Rect::new(50., 70., 260., 62.),
                156,
            ),
            GameFeedback::Combat { kind } => (
                assets.game_text.text(kind.address(), true),
                Rect::new(580., 536., 208., 62.),
                156,
            ),
            GameFeedback::CombatResourceResult { resource_id, index } => (
                assets
                    .reference_strings
                    .get(resource_id)
                    // GetIndString leaves index 0 and indexes past the list empty.
                    .map(|list| {
                        index
                            .checked_sub(1)
                            .and_then(|index| list.get(index))
                            .cloned()
                            .unwrap_or_default()
                    })
                    .ok_or_else(|| {
                        centurions_content::ContentError::Invalid(
                            "original combat result resource missing".into(),
                        )
                    }),
                Rect::new(580., 536., 208., 62.),
                156,
            ),
            GameFeedback::CombatResult { code, amount, .. } => {
                // showresults prints string(abs(damage)) in red, then the plural
                // label above one or the singular for one in yellowColor; -1
                // draws the miss label.
                let hit = match code {
                    0 => Some(if *amount > 1 { 0x400ec2 } else { 0x400eb4 }),
                    -1 => Some(0x400c4c),
                    _ => None,
                };
                let Some((address, _)) = hit
                    .map(|address| (address, 0))
                    .or_else(|| centurions_core::feedback::combat_result(*code))
                else {
                    return;
                };
                let Ok(label) = assets.game_text.text(address, true) else {
                    return;
                };
                // showresults uses TextMode srcOr, TextSize 20 and
                // ForeColor(redColor) for the amount; the amount codes then switch
                // to yellowColor for their label. In combat it draws PICT 156 over
                // the message rectangle and moves to (420, 443 - 32) from the panel
                // origin; the controls sit 20 pixels lower here than in the game's
                // window (scene's combat panels).
                let rect = Rect::new(568., 538., 232., 50.);
                assets.panel(156, rect);
                let red = Color::from_rgba(0xdd, 0x08, 0x06, 255);
                let yellow = Color::from_rgba(0xfc, 0xf3, 0x05, 255);
                let (x, y) = (580., 571.);
                if matches!(code, -23..=-19 | -16 | -15 | 0) {
                    let amount = amount.wrapping_abs().to_string();
                    assets.text(&amount, x, y, 20, red);
                    let width = assets.text_width(&amount, 20);
                    assets.text(&label, x + width, y, 20, yellow);
                } else {
                    assets.text(&label, x, y, 20, red);
                }
                return;
            }
            GameFeedback::Warning { index } => (
                assets
                    .warnings
                    .get(index.saturating_sub(1) as usize)
                    .cloned()
                    .ok_or_else(|| {
                        centurions_content::ContentError::Invalid("warning text missing".into())
                    }),
                Rect::new(warning_origin.x + 31., warning_origin.y + 41., 259., 60.),
                182,
            ),
            GameFeedback::Sound { .. } | GameFeedback::Silent | GameFeedback::Tutorial { .. } => {
                return;
            }
        };
        assets.panel(picture, rect);
        if matches!(feedback, GameFeedback::Warning { .. }) {
            assets.panel(144, Rect::new(rect.x + 4., rect.y + 4., 45., 50.));
        }
        if matches!(
            feedback,
            GameFeedback::Flash { .. }
                | GameFeedback::ResourceFlash { .. }
                | GameFeedback::EquipmentLevel { .. }
        ) {
            draw_rectangle_lines(
                rect.x - 1.,
                rect.y - 1.,
                rect.w + 2.,
                rect.h + 2.,
                1.,
                YELLOW,
            );
        }
        if let Ok(text) = text {
            assets.wrapped_color(
                &text,
                if matches!(feedback, GameFeedback::Warning { .. }) {
                    Rect::new(rect.x + 50., rect.y + 5., 204., 50.)
                } else {
                    Rect::new(rect.x + 10., rect.y + 8., rect.w - 20., rect.h - 10.)
                },
                12,
                15.,
                YELLOW,
            );
        }
    }
}
