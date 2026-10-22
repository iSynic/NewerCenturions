use crate::rules;
use crate::trace::{self, DrawCall};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RandomState {
    pub seed: u32,
    pub draws: u64,
}

impl RandomState {
    pub fn new(seed: u32) -> Self {
        Self { seed, draws: 0 }
    }
    #[track_caller]
    pub fn quickdraw(&mut self) -> i16 {
        let result = self.advance();
        trace::draw(DrawCall::Quickdraw, result);
        result
    }
    #[track_caller]
    pub fn bound(&mut self, bound: i16) -> i16 {
        let result = rules::random_bound(bound, self.advance());
        trace::draw(DrawCall::Bound(bound), result);
        result
    }
    #[track_caller]
    pub fn range(&mut self, min: i16, max: i16) -> i16 {
        let result = rules::random_range(min, max, self.advance());
        trace::draw(DrawCall::Range(min, max), result);
        result
    }
    fn advance(&mut self) -> i16 {
        self.seed = (u64::from(self.seed) * 16807 % 2147483647) as u32;
        self.draws += 1;
        let result = self.seed as i16;
        if result == i16::MIN { 0 } else { result }
    }
}
