use crate::prelude::*;

fn fixture() -> serde_json::Value {
    crate::support::fixture("required-enemy-weapons.json")
}
#[test]
fn required_enemy_preparation_matches_native_item_context_and_packet() {
    let fixture = fixture();

    crate::binary_endgame_weapons::compare_required_endgame_enemy_preparation_matches_native_item_context_and_packet(fixture);
}

#[test]
fn required_enemy_firearms_match_whole_party_and_native_randomness() {
    let fixture = fixture();
    let attacks = attacks();
    let cases = fixture.array("firearm_cases");

    for (index, case) in cases.iter().enumerate() {
        let mut party = case.bytes("warriors_before");
        let member = case.n::<usize>("member");
        let mut warrior = Warrior::read(&party[member * 3138..(member + 1) * 3138]).unwrap();
        let mut enemy = MonsterRecord {
            bytes: case.bytes("enemy_before"),
        };
        let prepared = enemy_weapons::prepare(
            &mut enemy,
            &case.bytes("weapon"),
            &case.bytes("ammo"),
            &attacks,
        )
        .unwrap();
        assert_eq!(prepared.packet, case.bytes("packet"), "case{index} packet");
        let mut random = RandomState::new(case.n::<u32>("seed"));
        let result =
            enemy_weapons::resolve(&mut warrior, &mut enemy, &prepared, &mut random, 0).unwrap();
        party[member * 3138..(member + 1) * 3138].copy_from_slice(&warrior.bytes);
        crate::support::native_expect! {
        bytes party => &case["warriors_after"], "case{index} full party";
        bytes enemy.bytes => &case["enemy_after"], "case{index} enemy";
        scalar random.seed => &case["seed_after"], "binary_required_enemy_weapons";
        scalar random.draws => &case["rng_draws"], "binary_required_enemy_weapons";
        scalar (result.sounds) => &case["sounds"], "binary_required_enemy_weapons";
        }
    }
}

#[test]
fn required_melee177_matches_native_damage_statistics_and_facing() {
    let fixture = fixture();
    let items = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../NewCenturions/TheNewCenturions/Data Files/Data ID"
    ))
    .unwrap();
    let weapon = &items[177 * 150..178 * 150];
    let cases = fixture.array("melee_cases");

    for case in cases.iter() {
        let mut party = case.bytes("warriors_before");
        let member = case.n::<usize>("member");
        let mut warrior = Warrior::read(&party[member * 3138..(member + 1) * 3138]).unwrap();
        let mut enemy = MonsterRecord {
            bytes: case.bytes("enemy_before"),
        };
        let mut random = RandomState::new(case.n::<u32>("seed"));
        let result = enemy_weapons::melee(
            &mut warrior,
            &mut enemy,
            weapon,
            &mut random,
            0,
            0,
            40 + member as i16,
        )
        .unwrap();
        party[member * 3138..(member + 1) * 3138].copy_from_slice(&warrior.bytes);
        crate::support::native_expect! {
        bytes party => &case["warriors_after"], "case{index} party";
        bytes enemy.bytes => &case["enemy_after"], "case{index} enemy";
        scalar random.seed => &case["seed_after"], "binary_required_enemy_weapons";
        scalar random.draws => &case["rng_draws"], "binary_required_enemy_weapons";
        scalar (result.sounds) => &case["sounds"], "binary_required_enemy_weapons";
        }
    }
}
