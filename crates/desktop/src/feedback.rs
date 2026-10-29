use crate::{assets::Assets, presentation::UiState};
use centurions_core::{Event, feedback::NativeFeedback};
use macroquad::prelude::*;

impl UiState {
    pub fn prepare_capture_prompt(&mut self, state: &centurions_core::Session) {
        let capturing = state
            .combat
            .as_ref()
            .is_some_and(|c| c.capture_actor.is_some());
        let own_prompt = matches!(
            self.persistent_feedback,
            Some(NativeFeedback::Flash {
                address: 0x4024de,
                ..
            })
        );
        if capturing && self.persistent_feedback.is_none() {
            // Restored targeting redraws its prompt without repeating entry sound or help.
            self.persistent_feedback = Some(NativeFeedback::Flash {
                address: 0x4024de,
                x: 30,
                y: 300,
                wait: -1,
                sound: 0,
            });
        } else if !capturing && own_prompt {
            self.persistent_feedback = None;
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
        if self.feedback.is_empty() {
            self.feedback.push_back(pause.clone());
            self.feedback_started = None;
        }
        self.feedback.front() == Some(&pause)
    }

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
            NativeFeedback::CombatResult { code, audible, .. } => {
                centurions_core::feedback::combat_result(*code)
                    .filter(|_| *audible)
                    .map_or(0, |(_, sound)| sound)
            }
            NativeFeedback::Sound { resource_id } => *resource_id,
            NativeFeedback::Warning { .. } | NativeFeedback::EquipmentLevel { .. } => 6000,
            NativeFeedback::Silent
            | NativeFeedback::CombatResourceResult { .. }
            | NativeFeedback::Tutorial { .. }
            | NativeFeedback::DialogText { .. }
            | NativeFeedback::OutdoorBodyStatus { .. } => 0,
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
        if matches!(feedback, NativeFeedback::Flash { wait, .. } | NativeFeedback::ResourceFlash { wait, .. } if *wait < 0)
        {
            if matches!(feedback, NativeFeedback::Flash { sound, .. } | NativeFeedback::ResourceFlash { sound, .. } if *sound < 0)
                && now < self.feedback_until
            {
                return true;
            }
            self.persistent_feedback = self.feedback.pop_front().filter(|feedback| {
                !matches!(
                    feedback,
                    NativeFeedback::Flash {
                        address: 0x40250d | 0x4036f1,
                        ..
                    }
                )
            });
            self.feedback_started = None;
            return !self.feedback.is_empty();
        }
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
            NativeFeedback::CombatResult { .. } | NativeFeedback::CombatResourceResult { .. } => {
                now - started >= 20. / 60.
            }
            NativeFeedback::OutdoorBodyStatus { wait, .. } => {
                now - started > f64::from(*wait) / 60.
            }
            NativeFeedback::DialogText { wait, .. } => now - started >= f64::from(*wait) / 60.,
            NativeFeedback::Flash { wait, .. } | NativeFeedback::ResourceFlash { wait, .. }
                if *wait > 0 =>
            {
                now - started >= f64::from(*wait) / 60.
            }
            NativeFeedback::Flash { .. }
            | NativeFeedback::ResourceFlash { .. }
            | NativeFeedback::EquipmentLevel { .. } => pressed && now - started >= 15. / 60.,
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
        let Some(feedback) = self
            .feedback
            .front()
            .filter(|f| !matches!(f, NativeFeedback::Sound { .. } | NativeFeedback::Silent))
            .or(self.persistent_feedback.as_ref())
        else {
            return;
        };
        let (text, rect, picture) = match feedback {
            NativeFeedback::OutdoorBodyStatus { permanent, .. } => {
                let address = if *permanent { 0x403294 } else { 0x403286 };
                if let Ok(text) = assets.native_text.text(address, true) {
                    assets.text(&text, 400., 30., 12, BLACK);
                }
                return;
            }
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
            NativeFeedback::Flash { address, x, y, .. } => (
                assets.native_text.text(*address, false),
                Rect::new(f32::from(*x), f32::from(*y) + 20., 260., 62.),
                156,
            ),
            NativeFeedback::EquipmentLevel { level } => (
                assets
                    .native_text
                    .text(0x400548, false)
                    .map(|prefix| format!("{prefix}{level}")),
                Rect::new(50., 70., 260., 62.),
                156,
            ),
            NativeFeedback::Combat { kind } => (
                assets.native_text.text(kind.address(), true),
                Rect::new(580., 536., 208., 62.),
                156,
            ),
            NativeFeedback::CombatResourceResult { resource_id, index } => (
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
            NativeFeedback::CombatResult { code, amount, .. } => {
                // showresults 0x158704..0x1588a4: string(abs(damage)) in red, then
                // yellowColor and 0x400ec2 above one or 0x400eb4 for one; 0x15903a:
                // -1 draws 0x400c4c.
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
                let Ok(label) = assets.native_text.text(address, true) else {
                    return;
                };
                // showresults 0x1585ee..0x158648: TextMode srcOr, TextSize 20 and
                // ForeColor(redColor) for the amount; the amount codes then switch
                // to yellowColor for their label (0x158b76..0x158cf8). In combat
                // it draws PICT 156 over rect 0x44370a (0x15849e..0x1584ae) and moves
                // to WORD40856c + 420, WORD40856e + 443 - 32 in window 128, whose
                // controls sit 20 pixels lower here (scene's combat panels).
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
            NativeFeedback::Flash { .. }
                | NativeFeedback::ResourceFlash { .. }
                | NativeFeedback::EquipmentLevel { .. }
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
