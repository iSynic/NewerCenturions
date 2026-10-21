use crate::{prelude::*, support};
#[test]
fn poison_natural_attack_matches_original_complete_records_and_random_consumption() {
    for name in [
        "natural-poison.json",
        "natural-pain.json",
        "natural-chemical.json",
        "natural-reflection.json",
        "natural-unloaded-charm.json",
    ] {
        compare_poison_natural_attack_matches_original_complete_records_and_random_consumption(
            support::fixture(name),
        );
    }
}

pub(super) fn compare_poison_natural_attack_matches_original_complete_records_and_random_consumption(
    fixture: Value,
) {
    let content = fixture
        .array("cases")
        .iter()
        .any(|case| word(&case.bytes("target"), 0x28).unwrap() > 0)
        .then(support::content);
    for (index, case) in fixture.array("cases").iter().enumerate() {
        let mut warrior = Warrior::read(&case.bytes("input")).unwrap();
        let mut target = MonsterRecord {
            bytes: case.bytes("target"),
        };
        let mut random = RandomState::new(case.n::<u32>("initial_seed"));
        let result = if let Some(content) = &content {
            let mut combat = support::battle();
            combat.enemies.records = vec![target];
            combat.enemies.positions = vec![(0, 0)];
            combat.squad.positions = vec![(0, 0)];
            let mut party = vec![warrior];
            let (result, _) = centurions_core::combat::enemy_weapons::melee_warrior_with_packet(
                &mut combat,
                &mut party,
                &mut random,
                (10, 0, case.n("attack"), 10),
                &content.items[word(&case.bytes("target"), 0x28).unwrap() as usize],
                0,
                &content.map(0).unwrap().combat.as_ref().unwrap().attacks,
                &[0; 58],
            )
            .unwrap();
            warrior = party.remove(0);
            target = combat.enemies.records.remove(0);
            Ok(result)
        } else {
            natural_attack(
                &mut warrior,
                &mut target,
                case.n::<usize>("attack"),
                &mut random,
                0,
            )
        }
        .unwrap();
        support::native_expect! {
            bytes warrior.bytes => &case["after"], &format!("warrior case {index}");
            bytes target.bytes => &case["after_target"], &format!("enemy case {index}");
            rng random => (&case["seed"], &case["draws"]), "binary_natural_poison";
            scalar (result.sounds) => &case["sounds"], "binary_natural_poison";
        }
    }
}
