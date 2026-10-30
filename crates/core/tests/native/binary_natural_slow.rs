use crate::{prelude::*, support};
#[test]
fn natural_slow_attack_matches_original_complete_records_and_random_consumption() {
    let fixture: serde_json::Value = support::fixture("natural-slow.json");
    crate::binary_natural_poison::compare_poison_natural_attack_matches_original_complete_records_and_random_consumption(fixture);
}

#[test]
fn slow_duration_reduction_matches_original_signed_counter_records() {
    use centurions_core::combat::runtime::reduce_round;
    let fixture: serde_json::Value = support::fixture("slow-reduction.json");
    let mut warriors = support::warriors(&fixture.bytes("before"));
    let mut combat = support::battle();
    combat.battle = 8;
    combat.field.tiles.clear();
    combat.squad.pos.clear();
    combat.squad.char_under.clear();
    combat.battle_order.clear();
    combat.battle_order_index = 0;
    let mut random = RandomState::new(12);
    let events = reduce_round(&mut combat, &mut warriors, &mut random).unwrap();
    assert!(events.is_empty());
    support::native_expect! {
        records &warriors => &fixture["after"], "binary_natural_slow";
        scalar random.draws => &fixture["rng_draws"], "binary_natural_slow";
    }
}
