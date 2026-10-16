use crate::prelude::*;

use centurions_core::rules;

#[test]
fn matches_independent_68k_execution() {
    let cases = crate::support::fixture("rules.json");
    for [bound, raw, expected] in cases.n::<Vec<[i16; 3]>>("rand") {
        assert_eq!(
            rules::random_bound(bound, raw),
            expected,
            "Rand({bound}), trap={raw}"
        );
    }
    for [value, min, max, expected] in cases.n::<Vec<[i16; 4]>>("range") {
        assert_eq!(
            i16::from(rules::inclusive_range(value, min, max)),
            expected,
            "twixt({value},{min},{max})"
        );
    }
    for [cost, dx, dy, expected] in cases.n::<Vec<[i16; 4]>>("movement") {
        assert_eq!(
            rules::movement_cost(cost, dx as i8, dy as i8),
            expected,
            "getmovecost({cost},{dx},{dy})"
        );
    }
    for [seed, next, expected] in cases.n::<Vec<[i64; 3]>>("quickdraw") {
        let mut random = RandomState::new(seed as u32);
        assert_eq!(random.quickdraw(), expected as i16, "QuickDraw seed {seed}");
        assert_eq!(random.seed, next as u32);
        assert_eq!(random.draws, 1);
    }
}
