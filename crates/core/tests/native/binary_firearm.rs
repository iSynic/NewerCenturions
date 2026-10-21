use crate::{prelude::*, support};

#[test]
fn reload_matches_original_record_reserve_and_sound_order() {
    let cases: serde_json::Value = support::fixture("reload.json");
    for case in cases.array("cases").iter() {
        let mut warrior = Warrior::read(&case.bytes("input")).unwrap();
        let mut reserve = case.n::<i32>("reserve");
        let mut random = RandomState::new(1);
        let (complete, sounds) = firearm::reload(
            &mut warrior,
            &mut reserve,
            case.n::<i64>("silent") != 0,
            &mut random,
        )
        .unwrap();
        support::native_expect! {
            bytes warrior.bytes => &case["after"], "warrior {index}";
            scalar reserve => &case["after_reserve"], "binary_firearm";
            scalar u64::from(complete) => &case["returned"], "binary_firearm";
            rng random => (&case["seed"], &case["draws"]), "binary_firearm";
            scalar (sounds) => &case["sounds"], "binary_firearm";
        }
    }
}
#[test]
fn firearm_preparation_and_resolution_match_original() {
    let cases: serde_json::Value = support::fixture("gun-resolve.json");
    let attacks = support::data("Data S");
    for (index, case) in cases.array("cases").iter().enumerate() {
        let mut warrior = Warrior::read(&case.bytes("input")).unwrap();
        let prepared = firearm::prepare(
            &warrior,
            &case.bytes("weapon"),
            &case.bytes("ammo"),
            &attacks,
        )
        .unwrap();
        assert_eq!(prepared.packet, case.bytes("packet"), "packet {index}");
        let mut target = MonsterRecord {
            bytes: case.bytes("target"),
        };
        let mut random = RandomState::new(case.n::<u32>("initial_seed"));
        let result =
            firearm::resolve(&mut warrior, &mut target, &prepared, &mut random, 0).unwrap();
        support::native_expect! {
            bytes warrior.bytes => &case["after"], "warrior {index}";
            bytes target.bytes => &case["after_target"], "target {index}";
            rng random => (&case["seed"], &case["draws"]), "binary_firearm";
        }
        let sounds: Vec<i16> = case.n::<Vec<i16>>("sounds");
        assert_eq!(result.sounds, sounds, "sounds {index}");
    }
}

#[test]
fn enemy_chemical_firearms_match_original_whole_records_and_randomness() {
    let fixtures: serde_json::Value = support::fixture("enemy-gun-resolve.json");
    let attacks = support::data("Data S");
    let mut hits = 0;
    for (index, case) in fixtures.array("cases").iter().enumerate() {
        let mut warrior = Warrior::read(&case.bytes("input")).unwrap();
        let mut enemy = MonsterRecord {
            bytes: case.bytes("target"),
        };
        let prepared = firearm::prepare_enemy(&mut enemy, &case.bytes("weapon"), &attacks).unwrap();
        assert_eq!(prepared.packet, case.bytes("packet"), "packet {index}");
        let mut random = RandomState::new(case.n::<u32>("initial_seed"));
        let result =
            firearm::resolve_enemy(&mut warrior, &mut enemy, &prepared, &mut random, 0).unwrap();
        hits += usize::from(result.damage > 0);
        support::native_expect! {
            bytes warrior.bytes => &case["after"], "warrior {index}";
            bytes enemy.bytes => &case["after_target"], "enemy {index}";
            rng random => (&case["seed"], &case["draws"]), "binary_firearm";
        }
        let sounds: Vec<i16> = case.n::<Vec<i16>>("sounds");
        assert_eq!(result.sounds, sounds, "sounds {index}");
    }
    assert!(hits > 0);
}
