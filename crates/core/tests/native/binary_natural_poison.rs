use crate::prelude::*;
#[test]
fn poison_natural_attack_matches_original_complete_records_and_random_consumption() {
    let fixture: serde_json::Value = crate::support::fixture("natural-poison.json");
    crate::binary_natural_poison::compare_poison_natural_attack_matches_original_complete_records_and_random_consumption(fixture);
}

pub(super) fn compare_poison_natural_attack_matches_original_complete_records_and_random_consumption(
    fixture: Value,
) {
    for (index, case) in fixture.array("cases").iter().enumerate() {
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
        bytes warrior.bytes => &case["after"], &format!("warrior case {index}");
        bytes target.bytes => &case["after_target"], &format!("enemy case {index}");
        scalar random.seed => &case["seed"], "binary_natural_poison";
        scalar random.draws => &case["draws"], "binary_natural_poison";
        scalar (result.sounds) => &case["sounds"], "binary_natural_poison";
        }
    }
}
