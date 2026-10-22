use crate::{prelude::*, support};

use centurions_core::combat::attack::natural_attack_with_packet;

#[test]
fn required_parts_ranged_to_natural_fallback_matches_original_full_records_and_randomness() {
    let fixture: serde_json::Value = support::fixture("parts-natural.json");
    let cases = fixture.array("cases");

    for (index, case) in cases.iter().enumerate() {
        let mut party = case.bytes("warriors_before");
        let member = case.n::<usize>("member");
        let mut warrior = crate::support::warrior_at(&party, member);
        let mut enemy = MonsterRecord {
            bytes: case.bytes("enemy_before"),
        };
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let (result, packet) = natural_attack_with_packet(
            &mut warrior,
            &mut enemy,
            case.n::<usize>("attack"),
            &mut random,
            &support::data("Data S"),
            &[0; 58],
            0,
        )
        .unwrap();
        assert_eq!(packet, case.bytes("packet"), "case{index} packet");
        crate::support::put_warrior(&mut party, member, &warrior);
        support::native_expect! {
            bytes party => &case["warriors_after"], "case{index} party";
            bytes enemy.bytes => &case["enemy_after"], "case{index} enemy";
            rng random => (&case["seed_after"], &case["rng_draws"]), "binary_parts_natural";
            scalar (result.sounds) => &case["sounds"], "binary_parts_natural";
        }
    }
}
