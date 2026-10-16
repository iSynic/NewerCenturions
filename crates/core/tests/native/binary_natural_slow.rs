use crate::prelude::*;
#[test]
fn natural_slow_attack_matches_original_complete_records_and_random_consumption() {
    let fixture: serde_json::Value = crate::support::fixture("natural-slow.json");
    crate::binary_natural_poison::compare_poison_natural_attack_matches_original_complete_records_and_random_consumption(fixture);
}

#[test]
fn slow_duration_reduction_matches_original_signed_counter_records() {
    use centurions_core::combat::{Combat, SquadPlacement, TacticalMap, runtime::reduce_round};
    let fixture: serde_json::Value = crate::support::fixture("slow-reduction.json");
    let mut warriors = crate::support::warriors(&fixture.bytes("before"));
    let mut combat = Combat {
        battle: 8,
        map: TacticalMap {
            tiles: vec![],
            edge_word: 0,
            shift: (0, 0),
        },
        squad: SquadPlacement {
            positions: vec![],
            under_tiles: vec![],
        },
        order: vec![],
        cursor: 0,
        ..crate::support::battle()
    };
    let mut random = RandomState::new(12);
    let events = reduce_round(&mut combat, &mut warriors, &mut random).unwrap();
    assert!(events.is_empty());
    crate::support::native_expect! {
    bytes crate::support::record_bytes(&warriors) => &fixture["after"], "binary_natural_slow";
    scalar random.draws => &fixture["rng_draws"], "binary_natural_slow";
    }
}
