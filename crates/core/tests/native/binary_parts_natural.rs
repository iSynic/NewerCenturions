use crate::prelude::*;

use centurions_core::combat::attack::natural_attack_with_packet;

#[test]
fn required_parts_ranged_to_natural_fallback_matches_original_full_records_and_randomness() {
    let fixture: serde_json::Value = crate::support::fixture("parts-natural.json");
    let cases = fixture.array("cases");

    for (index, case) in cases.iter().enumerate() {
        let mut party = case.bytes("warriors_before");
        let member = case.n::<usize>("member");
        let mut warrior = Warrior::read(&party[member * 3138..(member + 1) * 3138]).unwrap();
        let mut enemy = MonsterRecord {
            bytes: case.bytes("enemy_before"),
        };
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let (result, packet) = natural_attack_with_packet(
            &mut warrior,
            &mut enemy,
            case.n::<usize>("attack"),
            &mut random,
            &std::fs::read(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../NewCenturions/TheNewCenturions/Data Files/Data S"
            ))
            .unwrap(),
            &[0; 58],
            0,
        )
        .unwrap();
        assert_eq!(packet, case.bytes("packet"), "case{index} packet");
        party[member * 3138..(member + 1) * 3138].copy_from_slice(&warrior.bytes);
        crate::support::native_expect! {
        bytes party => &case["warriors_after"], "case{index} party";
        bytes enemy.bytes => &case["enemy_after"], "case{index} enemy";
        scalar random.seed => &case["seed_after"], "binary_parts_natural";
        scalar random.draws => &case["rng_draws"], "binary_parts_natural";
        scalar (result.sounds) => &case["sounds"], "binary_parts_natural";
        }
    }
}
