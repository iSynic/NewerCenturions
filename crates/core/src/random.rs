use crate::rules;
use crate::trace::{self, DrawCall};
use serde::{Deserialize, Serialize};

/// The game's random number generator: its seed and how many numbers have been drawn from it.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RandomState {
    /// The generator's current seed.
    pub seed: u32,
    /// How many numbers have been drawn.
    pub draws: u64,
}

impl RandomState {
    /// A generator with the given seed and no numbers drawn.
    pub fn new(seed: u32) -> Self {
        Self { seed, draws: 0 }
    }
    /// The next raw random value, as the game's QuickDraw call returns it: a signed 16-bit value.
    #[track_caller]
    pub fn quickdraw(&mut self) -> i16 {
        let result = self.advance();
        trace::draw(DrawCall::Quickdraw, result);
        result
    }
    /// A random number from 1 up to the bound.
    #[track_caller]
    pub fn bound(&mut self, bound: i16) -> i16 {
        let result = rules::random_bound(bound, self.advance());
        trace::draw(DrawCall::Bound(bound), result);
        result
    }
    /// A random number from the minimum up to the maximum, both included.
    #[track_caller]
    pub fn range(&mut self, min: i16, max: i16) -> i16 {
        let result = rules::random_range(min, max, self.advance());
        trace::draw(DrawCall::Range(min, max), result);
        result
    }
    /// Steps the seed (multiplying by 16807 modulo 2147483647), counts the draw and returns the
    /// seed's low 16 bits as a signed value, with -32768 read as 0.
    fn advance(&mut self) -> i16 {
        self.seed = (u64::from(self.seed) * 16807 % 2147483647) as u32;
        self.draws += 1;
        let result = self.seed as i16;
        if result == i16::MIN { 0 } else { result }
    }
}
