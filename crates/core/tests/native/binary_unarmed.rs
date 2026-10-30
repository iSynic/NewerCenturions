use crate::{prelude::*, support};

use centurions_core::combat::attack::{unarmed_attack, warrior_attack_with_packet};

#[test]
fn unarmed_attack_matches_original_routine() {
    let content = support::content();
    let attacks = &content.map(0).unwrap().combat.as_ref().unwrap().attacks;
    // The last fixture holds constructed selector -10 weapons for each case.
    for name in [
        "unarmed.json",
        "difficulty-unarmed.json",
        "warrior-weapon-condition.json",
    ] {
        let fixture: serde_json::Value = support::fixture(name);
        for (index, case) in fixture.array("cases").iter().enumerate() {
            let mut warrior = Warrior::read(&case.bytes("input")).unwrap();
            let mut target = MonsterRecord {
                bytes: case.bytes("target"),
            };
            let mut random = RandomState::new(case.n::<u32>("initial_seed"));
            let difficulty = case["difficulty"].as_i64().unwrap_or(0) as i16;
            let result = match case.get("weapon") {
                Some(_) => {
                    warrior_attack_with_packet(
                        &mut warrior,
                        &mut target,
                        &mut random,
                        attacks,
                        &[0; 58],
                        difficulty,
                        Some(&case.bytes("weapon")),
                    )
                    .unwrap()
                    .0
                }
                None => unarmed_attack(&mut warrior, &mut target, &mut random, difficulty).unwrap(),
            };
            support::native_expect! {
                bytes warrior.bytes => &case["after"], &format!("{name} warrior case {index}");
                bytes target.bytes => &case["after_target"], &format!("{name} enemy case {index}");
                rng random => (&case["seed"], &case["draws"]), "binary_unarmed";
                scalar (result.sounds) => &case["sounds"], "binary_unarmed";
            }
        }
    }
}

/// Spell weapons above 1100, conditions on reflected blows, and warrior-against-warrior
/// conditions, each on a constructed item 180 carried by the case.
#[test]
fn melee_weapon_branches_match_original_attack_and_attack2() {
    use centurions_core::combat::attack::{WarriorMelee, warrior_melee};
    let content = support::content();
    let attacks = &content.map(0).unwrap().combat.as_ref().unwrap().attacks;
    let fixture: serde_json::Value = support::fixture("melee-weapon-branches.json");
    for (index, case) in fixture.array("cases").iter().enumerate() {
        let context = format!("melee-weapon-branches case {index} {}", case["group"]);
        let weapon = case.bytes("weapon");
        let mut combat = support::battle();
        combat.battle_order[1] = case.n("order");
        combat.roster.records = vec![MonsterRecord {
            bytes: case.bytes("target"),
        }];
        combat.roster.mon_pos = vec![(20, 20)];
        let mut party = vec![Warrior::read(&case.bytes("input")).unwrap()];
        let defender: i16 = case.n("defender");
        match defender {
            1 => party.push(Warrior::read(&case.bytes("other")).unwrap()),
            11 => {
                combat.roster.records.push(MonsterRecord {
                    bytes: case.bytes("other"),
                });
                combat.roster.mon_pos.push((21, 20));
            }
            _ => {}
        }
        let mut random = RandomState::new(case.n::<u32>("initial_seed"));
        let (sounds, packet) = if case.n::<i16>("attacker") == 0 {
            let mut items = content.items.clone();
            items[180] = weapon;
            let mut buffs = [0; 20];
            buffs[15] = case.n("mind_shield");
            match warrior_melee(
                &mut combat,
                &mut party,
                &mut random,
                0,
                defender,
                &items,
                attacks,
                &[0; 58],
                &buffs,
            )
            .unwrap()
            {
                WarriorMelee::Struck { result, packet, .. } => (result.sounds, packet.unwrap()),
                WarriorMelee::Halted { packet, warned } => {
                    (if warned { vec![39] } else { vec![] }, packet)
                }
            }
        } else {
            let bodies = (10, defender, 0, 10);
            let (result, packet) = if defender == 11 {
                enemy_weapons::melee_enemy_with_packet(
                    &mut combat,
                    &mut random,
                    bodies,
                    &weapon,
                    0,
                    attacks,
                    &[0; 58],
                )
            } else {
                enemy_weapons::melee_warrior_with_packet(
                    &mut combat,
                    &mut party,
                    &mut random,
                    bodies,
                    &weapon,
                    0,
                    attacks,
                    &[0; 58],
                )
            }
            .unwrap();
            (result.sounds, packet)
        };
        support::native_expect! {
            bytes party[0].bytes => &case["after"], &context;
            bytes combat.roster.records[0].bytes => &case["after_target"], &context;
            bytes packet => &case["packet"], &context;
            rng random => (&case["seed"], &case["draws"]), &context;
            scalar sounds => &case["sounds"], &context;
        }
        match defender {
            1 => support::assert_bytes(&party[1].bytes, &case.bytes("after_other"), &context),
            11 => support::assert_bytes(
                &combat.roster.records[1].bytes,
                &case.bytes("after_other"),
                &context,
            ),
            _ => {}
        }
    }
}
