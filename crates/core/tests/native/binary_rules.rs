use crate::prelude::*;

use centurions_core::rules;

#[derive(Deserialize)]
struct Cases {
    rand: Vec<[i16; 3]>,
    range: Vec<[i16; 4]>,
    movement: Vec<[i16; 4]>,
    quickdraw: Vec<[i64; 3]>,
}

#[test]
fn matches_independent_68k_execution() {
    let cases: Cases = serde_json::from_value(crate::support::fixture("rules.json")).unwrap();
    for [bound, raw, expected] in cases.rand {
        assert_eq!(
            rules::random_bound(bound, raw),
            expected,
            "Rand({bound}), trap={raw}"
        );
    }
    for [value, min, max, expected] in cases.range {
        assert_eq!(
            i16::from(rules::inclusive_range(value, min, max)),
            expected,
            "twixt({value},{min},{max})"
        );
    }
    for [cost, dx, dy, expected] in cases.movement {
        assert_eq!(
            rules::movement_cost(cost, dx as i8, dy as i8),
            expected,
            "getmovecost({cost},{dx},{dy})"
        );
    }
    for [seed, next, expected] in cases.quickdraw {
        let mut random = RandomState::new(seed as u32);
        assert_eq!(random.quickdraw(), expected as i16, "QuickDraw seed {seed}");
        assert_eq!(random.seed, next as u32);
        assert_eq!(random.draws, 1);
    }
}
