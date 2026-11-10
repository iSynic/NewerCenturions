//! The farewell panel shown while the game quits.
use crate::{assets::Assets, audio::Audio, placement::Placement};
use centurions_core::Event;
use macroquad::prelude::*;

/// PICT shown by the farewell window (dialog 175).
const FAREWELL_PICTURE: i16 = 19;
/// How long the panel stays before the screen empties: eleven steps of three ticks.
const PANEL_TICKS: f64 = 33.;
/// The pause with the screen empty before the game exits.
const EMPTY_TICKS: f64 = 80.;

/// The quit farewell: its first sound plays at once, the panel stays up while
/// the second sound plays, and the closing sound plays as the screen empties.
pub struct Farewell {
    started: f64,
}

impl Farewell {
    /// Starts the farewell at `now`, queueing its three sounds when sound is on.
    ///
    /// Original: quitgame
    pub fn begin(now: f64, audio: Option<&Audio>, sound: bool) -> Self {
        if sound && let Some(audio) = audio {
            audio.play(&[
                Event::Sound { resource_id: 208 },
                Event::Sound { resource_id: 203 },
                Event::Delay {
                    ticks: PANEL_TICKS as u32,
                },
                Event::Sound { resource_id: 205 },
            ]);
        }
        Self { started: now }
    }

    /// Draws the farewell panel one pixel in from the game window's corner until its time has
    /// passed, then an empty screen.
    ///
    /// Original: quitgame
    pub fn draw(&self, assets: &Assets, placement: Placement, now: f64) {
        clear_background(BLACK);
        if (now - self.started) * crate::TICKS_PER_SECOND < PANEL_TICKS {
            let corner = placement.farewell();
            assets.panel(FAREWELL_PICTURE, Rect::new(corner.x, corner.y, 640., 460.));
        }
    }

    /// Whether the farewell has run its course and the game can exit.
    pub fn finished(&self, now: f64) -> bool {
        (now - self.started) * crate::TICKS_PER_SECOND >= PANEL_TICKS + EMPTY_TICKS
    }
}
