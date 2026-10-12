use crate::prelude::*;

use centurions_core::combat::initiative;

#[derive(Deserialize)]
struct Fixture {
    warrior_speeds: Vec<i16>,
    enemy_speeds: Vec<i16>,
    cases: Vec<Case>,
}
#[derive(Deserialize)]
struct Case {
    seed: u32,
    after_seed: u32,
    draws: u64,
    mode: i16,
    order: Vec<i16>,
}
#[test]
fn initiative_matches_original_instruction_execution() {
    let fixture: Fixture =
        serde_json::from_value(crate::support::fixture("initiative.json")).unwrap();
    for case in fixture.cases {
        let mut random = RandomState::new(case.seed);
        assert_eq!(
            initiative(
                &fixture.warrior_speeds,
                &fixture.enemy_speeds,
                case.mode,
                &mut random
            )
            .unwrap(),
            case.order
        );
        assert_eq!((random.seed, random.draws), (case.after_seed, case.draws));
    }
}
