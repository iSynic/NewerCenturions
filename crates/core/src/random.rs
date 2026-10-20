use crate::{recovered_ai, rules};
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
    pub fn quickdraw(&mut self) -> i16 {
        self.seed = (u64::from(self.seed) * 16807 % 2147483647) as u32;
        self.draws += 1;
        let result = self.seed as i16;
        if result == i16::MIN { 0 } else { result }
    }
    pub fn bound(&mut self, bound: i16) -> i16 {
        rules::random_bound(bound, self.quickdraw())
    }
    pub fn range(&mut self, min: i16, max: i16) -> i16 {
        recovered_ai::random_range(min, max, self)
    }
}
