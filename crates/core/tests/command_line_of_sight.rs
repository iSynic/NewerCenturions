//! The animation delay preference shortens the stretch of a sight line that is tested.
use crate::support;
use centurions_core::combat::lifecycle::can_see;

#[test]
fn a_blocker_in_the_last_stretch_hides_the_target_only_without_a_delay() {
    let mut combat = support::battle();
    combat.field.tiles[5 * 90 + 10] = 1000;
    let mut mission = support::mission();
    mission.map_stats = vec![0; 40];
    mission.map_stats[12..14].copy_from_slice(&1i16.to_be_bytes());
    let sees = |delay| can_see(&combat, &mission, (0, 0), (0, 5), delay).unwrap();
    // At a delay of 20 the 128 steps stop short of the fifth cell.
    assert!(!sees(0));
    assert!(sees(20));
}
