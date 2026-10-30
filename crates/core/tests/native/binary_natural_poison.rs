use crate::{prelude::*, support};
#[test]
fn poison_natural_attack_matches_original_complete_records_and_random_consumption() {
    for name in [
        "natural-poison.json",
        "natural-pain.json",
        "natural-chemical.json",
        "natural-reflection.json",
        "natural-unloaded-charm.json",
        // Constructed attackers: conditions Thunder never authors against warriors.
        "natural-elements.json",
        "natural-drain.json",
        "natural-npc.json",
        "natural-held.json",
        // Constructed selector -10 weapon definitions carried by each case.
        "weapon-condition.json",
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
        if case.get("victim").is_some() {
            compare_npc_target(index, case);
            continue;
        }
        let mut warrior = Warrior::read(&case.bytes("input")).unwrap();
        let mut target = MonsterRecord {
            bytes: case.bytes("target"),
        };
        let mut random = RandomState::new(case.n::<u32>("initial_seed"));
        let result = if let Some(content) = &content {
            let mut combat = support::battle();
            combat.roster.records = vec![target];
            combat.roster.mon_pos = vec![(0, 0)];
            combat.squad.pos = vec![(0, 0)];
            let mut party = vec![warrior];
            let (result, _) = centurions_core::combat::enemy_weapons::melee_warrior_with_packet(
                &mut combat,
                &mut party,
                &mut random,
                (10, 0, case.n("attack"), 10),
                case.get("weapon")
                    .map_or_else(
                        || {
                            content.items[word(&case.bytes("target"), 0x28).unwrap() as usize]
                                .clone()
                        },
                        |_| case.bytes("weapon"),
                    )
                    .as_slice(),
                0,
                &content.map(0).unwrap().combat.as_ref().unwrap().attacks,
                &[0; 58],
            )
            .unwrap();
            warrior = party.remove(0);
            target = combat.roster.records.remove(0);
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

/// Body 10 attacks body 11 one cell to its right, with the party untouched.
fn compare_npc_target(index: usize, case: &Value) {
    let mut combat = support::battle();
    combat.roster.records = ["target", "victim"]
        .map(|name| MonsterRecord {
            bytes: case.bytes(name),
        })
        .to_vec();
    combat.roster.mon_pos = vec![(20, 20), (21, 20)];
    let mut party = vec![Warrior::read(&case.bytes("input")).unwrap()];
    let mut random = RandomState::new(case.n::<u32>("initial_seed"));
    let result = centurions_core::combat::ally_turns::natural_attack_enemy(
        &mut combat,
        &mut party,
        &mut random,
        10,
        11,
        case.n("attack"),
    )
    .unwrap();
    support::native_expect! {
        bytes party[0].bytes => &case["after"], &format!("party case {index}");
        bytes combat.roster.records[0].bytes => &case["after_target"], &format!("attacker case {index}");
        bytes combat.roster.records[1].bytes => &case["after_victim"], &format!("victim case {index}");
        rng random => (&case["seed"], &case["draws"]), "binary_natural_poison";
        scalar (result.sounds) => &case["sounds"], "binary_natural_poison";
    }
}
