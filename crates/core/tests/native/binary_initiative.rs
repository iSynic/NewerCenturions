use crate::prelude::*;

use centurions_core::combat::initiative;

#[test]
fn initiative_matches_original_instruction_execution() {
    let fixture = crate::support::fixture("initiative.json");
    for case in fixture.array("cases") {
        let mut random = RandomState::new(case.n("seed"));
        assert_eq!(
            initiative(
                &fixture.n::<Vec<i16>>("warrior_speeds"),
                &fixture.n::<Vec<i16>>("enemy_speeds"),
                case.n("mode"),
                &mut random
            )
            .unwrap(),
            case.n::<Vec<i16>>("order")
        );
        assert_eq!(
            (random.seed, random.draws),
            (case.n("after_seed"), case.n("draws"))
        );
    }
}
