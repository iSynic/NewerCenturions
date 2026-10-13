use crate::prelude::*;

use centurions_core::combat::attack::unarmed_attack;

#[test]
fn unarmed_attack_matches_original_routine() {
    for name in ["unarmed.json", "difficulty-unarmed.json"] {
        let fixture: serde_json::Value = crate::support::fixture(name);
        for case in fixture.array("cases").iter() {
            let mut warrior = Warrior::read(&case.bytes("input")).unwrap();
            let mut target = MonsterRecord {
                bytes: case.bytes("target"),
            };
            let mut random = RandomState::new(case.n::<u32>("initial_seed"));
            let result = unarmed_attack(
                &mut warrior,
                &mut target,
                &mut random,
                case["difficulty"].as_i64().unwrap_or(0) as i16,
            )
            .unwrap();
            crate::support::native_expect! {
            bytes warrior.bytes => &case["after"], "warrior case {index}";
            bytes target.bytes => &case["after_target"], "enemy case {index}";
            scalar random.seed => &case["seed"], "binary_unarmed";
            scalar random.draws => &case["draws"], "binary_unarmed";
            scalar (result.sounds) => &case["sounds"], "binary_unarmed";
            }
        }
    }
}
