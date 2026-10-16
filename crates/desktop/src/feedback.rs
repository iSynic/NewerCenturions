use crate::{assets::Assets, presentation::UiState};
use centurions_core::{Event, feedback::NativeFeedback};
use macroquad::prelude::*;

impl UiState {
    pub fn feedback_sound(&mut self, now: f64) -> Option<Event> {
        let feedback = self.feedback.front()?;
        if self.feedback_started.is_some() {
            return None;
        }
        self.feedback_started = Some(now);
        let resource_id = match feedback {
            NativeFeedback::Flash { sound, .. } | NativeFeedback::ResourceFlash { sound, .. } => {
                *sound
            }
            NativeFeedback::Combat { .. } => -143,
            NativeFeedback::Sound { resource_id } => *resource_id,
            NativeFeedback::Warning { .. } => 6000,
            NativeFeedback::Silent
            | NativeFeedback::Tutorial { .. }
            | NativeFeedback::DialogText { .. } => 0,
        };
        (resource_id != 0).then_some(Event::Sound { resource_id })
    }

    pub fn feedback_input(&mut self, now: f64, pressed: bool) -> bool {
        if self.technical_error.is_some() {
            if pressed {
                self.technical_error = None;
            }
            return true;
        }
        let Some(feedback) = self.feedback.front() else {
            return false;
        };
        if let NativeFeedback::Tutorial { first, last } = feedback {
            self.help_request = Some((*first, *last));
            self.feedback.pop_front();
            self.feedback_started = None;
            return true;
        }
        if matches!(feedback, NativeFeedback::Combat { .. }) {
            if now < self.feedback_until {
                return true;
            }
            if let Some(NativeFeedback::Combat { kind }) = self.feedback.pop_front() {
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
            self.feedback_started = None;
            return false;
        }
        let started = self.feedback_started.unwrap_or(now);
        let done = match feedback {
            NativeFeedback::DialogText { wait, .. } => now - started >= f64::from(*wait) / 60.,
            NativeFeedback::Flash { wait, .. } | NativeFeedback::ResourceFlash { wait, .. }
                if *wait > 0 =>
            {
                now - started >= f64::from(*wait) / 60.
            }
            NativeFeedback::Flash { .. } | NativeFeedback::ResourceFlash { .. } => {
                pressed && now - started >= 15. / 60.
            }
            NativeFeedback::Sound { .. } => now >= self.feedback_until,
            NativeFeedback::Silent => true,
            NativeFeedback::Tutorial { .. } => unreachable!(),
            NativeFeedback::Warning { .. } => pressed && now - started >= 10. / 60.,
            NativeFeedback::Combat { .. } => unreachable!(),
        };
        if done {
            self.feedback.pop_front();
            self.feedback_started = None;
        }
        true
    }

    pub fn draw_feedback(&self, assets: &Assets, warning_origin: Vec2) {
        if let Some(error) = &self.technical_error {
            draw_rectangle(40., 180., 720., 240., Color::from_rgba(25, 25, 25, 255));
            draw_rectangle_lines(40., 180., 720., 240., 2., RED);
            assets.wrapped(error, Rect::new(56., 200., 686., 196.), 14, 18.);
            return;
        }
        let Some(feedback) = self.feedback.front() else {
            return;
        };
        let (text, rect, picture) = match feedback {
            NativeFeedback::DialogText {
                dialog,
                item,
                address,
                ..
            } => {
                if let Ok(text) = assets.native_text.text(*address, true) {
                    assets.wrapped_color(
                        &text,
                        assets.dialog_rect(*dialog, usize::from(*item)),
                        12,
                        14.,
                        YELLOW,
                    );
                }
                return;
            }
            NativeFeedback::ResourceFlash {
                resource_id,
                index,
                x,
                y,
                ..
            } => (
                assets
                    .reference_strings
                    .get(resource_id)
                    .and_then(|list| list.get(index.saturating_sub(1)))
                    .cloned()
                    .ok_or_else(|| {
                        centurions_content::ContentError::Invalid(
                            "original flash resource missing".into(),
                        )
                    }),
                Rect::new(f32::from(*x), f32::from(*y) + 20., 260., 62.),
                156,
            ),
            NativeFeedback::Flash { address, x, y, .. } => (
                assets.native_text.text(*address, false),
                Rect::new(f32::from(*x), f32::from(*y) + 20., 260., 62.),
                156,
            ),
            NativeFeedback::Combat { kind } => (
                assets.native_text.text(kind.address(), true),
                Rect::new(580., 536., 208., 62.),
                156,
            ),
            NativeFeedback::Warning { index } => (
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
            NativeFeedback::Sound { .. }
            | NativeFeedback::Silent
            | NativeFeedback::Tutorial { .. } => return,
        };
        assets.panel(picture, rect);
        if matches!(feedback, NativeFeedback::Warning { .. }) {
            assets.panel(144, Rect::new(rect.x + 4., rect.y + 4., 45., 50.));
        }
        if matches!(
            feedback,
            NativeFeedback::Flash { .. } | NativeFeedback::ResourceFlash { .. }
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
                if matches!(feedback, NativeFeedback::Warning { .. }) {
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
