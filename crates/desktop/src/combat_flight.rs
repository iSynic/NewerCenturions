//! cast 0x10d4fe's flight for one target: after the launch sound (0x10db1a) the
//! packet's byte 17 (BYTE44375f, switched at 0x10dca2) selects a PICT 401 sprite
//! walk, a patXor line or framed ovals. Geometry and timing only; the presentation
//! layer draws the frames.
use centurions_core::Flight;

/// One cast 0x10d4fe flight, timed in delay(1) ticks.
#[derive(Clone, Copy, Debug)]
pub struct FlightFrames {
    pub flight: Flight,
    /// The walk's step count d6 (0x10d5a0..0x10d5d0), at least 1 (0x10db82).
    pub steps: i16,
    pub started: f64,
    pub tick_seconds: f64,
    pub recentred: bool,
    /// The ticks this part of the walk shows: a ray's walk is cut where
    /// resolvespell presents a crossed body (0x10e22a).
    pub window: (i16, i16),
}

/// What a flight shows during one tick.
pub enum FlightFrame {
    /// The default arm 0x10e074..0x10e5d6 copies a 32-pixel PICT 401 sprite
    /// transparently at its walk point, `source` in sprite pixels.
    Sprite {
        point: (i16, i16),
        source: (i16, i16),
    },
    /// 0x10ddde/0x10de70: a patXor line between the cell centres in a square pen.
    Line {
        from: (i16, i16),
        to: (i16, i16),
        pen: i16,
    },
    /// 0x10df26..0x10e06e: patXor ovals in a 4-pixel pen, as (left, top, right,
    /// bottom) pixel rectangles.
    Ovals([Option<(i16, i16, i16, i16)>; 2]),
}

impl FlightFrames {
    pub fn new(flight: Flight, steps: i16, started: f64, tick_seconds: f64) -> Self {
        let steps = steps.max(1);
        Self {
            flight,
            steps,
            started,
            tick_seconds,
            recentred: false,
            window: (0, Self::ticks(flight.kind, steps)),
        }
    }

    /// The delay(1) ticks a sprite walk has made before testing step `step`.
    pub fn ticks_before(step: i16) -> i16 {
        ((step - 1) / 15).clamp(0, 8)
    }

    /// The part of the walk from tick `first` up to `last`.
    pub fn part(self, first: i16, last: i16) -> Self {
        Self {
            window: (first, last.max(first)),
            ..self
        }
    }

    /// The window's tick count.
    pub fn length(&self) -> i16 {
        self.window.1 - self.window.0
    }

    /// The delay(1) calls the arm makes: the sprite walk waits after steps 15, 30,
    /// ..., 120 (0x10e58c..0x10e5c6); the line and oval arms wait eight times.
    pub fn ticks(kind: i8, steps: i16) -> i16 {
        match kind {
            13 => 0,
            23..=27 => 8,
            _ => ((steps - 1) / 15).clamp(0, 8),
        }
    }

    /// The walk point after `step` increments of the single-precision stride
    /// (0x10dbce..0x10dc0a), truncated like the core's ray.
    fn point(&self, step: i16) -> (i16, i16) {
        let (from, to) = (self.flight.from, self.flight.to);
        let along = |a: i16, b: i16| {
            let stride = f32::from(b.wrapping_sub(a)) / f32::from(self.steps);
            (f32::from(a) + stride * f32::from(step)).trunc() as i16
        };
        (along(from.0, to.0), along(from.1, to.1))
    }

    pub fn frame(&self, now: f64) -> Option<FlightFrame> {
        let tick = ((now - self.started) / self.tick_seconds).floor();
        if !(0. ..f64::from(self.length())).contains(&tick) {
            return None;
        }
        let tick = self.window.0 + tick as i16;
        let (from, to) = (self.flight.from, self.flight.to);
        let centre = |p: (i16, i16)| (p.0 + 16, p.1 + 16);
        Some(match self.flight.kind {
            23 | 24 => {
                let mut to = to;
                if self.flight.kind == 24 {
                    // 0x10de8a..0x10dea6 add Rand(10) - 5 to the target point each
                    // pass; the core drew those offsets from the session.
                    let jitter = self.flight.jitter;
                    for pass in 0..=tick as usize {
                        to.0 += i16::from(jitter[pass * 2]);
                        to.1 += i16::from(jitter[pass * 2 + 1]);
                    }
                }
                FlightFrame::Line {
                    from: centre(from),
                    to: centre(to),
                    pen: (tick - 4).abs(),
                }
            }
            25..=27 => {
                // 25 starts inset 12 and grows 4 a side; 26 and 27 start outset 32
                // and shrink 4; 27 also frames an inner oval inset 32 - 8 * pass.
                let (start, grow) = if self.flight.kind == 25 {
                    (12, -4)
                } else {
                    (-32, 4)
                };
                let rect = |inset: i16| {
                    (
                        to.0 + inset,
                        to.1 + inset,
                        to.0 + 32 - inset,
                        to.1 + 32 - inset,
                    )
                };
                let outer = start + grow * tick;
                let inner = (self.flight.kind == 27).then(|| rect(outer + 32 - 8 * tick));
                FlightFrame::Ovals([Some(rect(outer)), inner])
            }
            13 => return None,
            kind => FlightFrame::Sprite {
                point: self.point((tick + 1) * 15),
                source: (i16::from(kind) * 32 - 32, self.sprite_row()),
            },
        })
    }

    /// 0x10e3ec..0x10e562: row 64 rising, 96 falling, 128 rightward, 160 leftward.
    fn sprite_row(&self) -> i16 {
        let dx = self.flight.to.0.wrapping_sub(self.flight.from.0);
        let dy = self.flight.to.1.wrapping_sub(self.flight.from.1);
        if dx.unsigned_abs() > dy.unsigned_abs() {
            if dx < 0 { 160 } else { 128 }
        } else if dy > 0 {
            96
        } else {
            64
        }
    }
}
