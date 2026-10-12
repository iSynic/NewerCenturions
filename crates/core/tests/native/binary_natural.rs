use crate::prelude::*;

#[test]
fn natural_attack_matches_original_routine() {
    let fixture: serde_json::Value = crate::support::fixture("natural.json");
    for case in fixture.array("cases").iter() {
        let mut warrior = Warrior::read(&case.bytes("input")).unwrap();
        let mut target = MonsterRecord {
            bytes: case.bytes("target"),
        };
        let mut random = RandomState::new(case.n::<u32>("initial_seed"));
        let result = natural_attack(
            &mut warrior,
            &mut target,
            case.n::<usize>("attack"),
            &mut random,
        )
        .unwrap();
        crate::support::native_expect! {
        bytes warrior.bytes => &case["after"], "warrior case {index}";
        bytes target.bytes => &case["after_target"], "enemy case {index}";
        scalar random.seed => &case["seed"], "binary_natural";
        scalar random.draws => &case["draws"], "binary_natural";
        scalar (result.sounds) => &case["sounds"], "binary_natural";
        }
    }
}
