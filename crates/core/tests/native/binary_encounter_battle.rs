use crate::{prelude::*, support};

use centurions_core::combat::encounter_battle;

#[test]
fn encounter_battle_selection_matches_original_signed_words_effects_and_random_consumption() {
    let fixture: serde_json::Value = support::fixture("encounter-battle.json");
    for case in fixture.array("cases") {
        let hex = case.text("extra");
        let extra = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect::<Vec<_>>();
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let result = encounter_battle(&extra, &mut random).unwrap();
        support::expect(result.battle, &case["battle"]);
        assert_eq!(
            result.treasure,
            i16::from_be_bytes(extra[8..10].try_into().unwrap())
        );
        let mut effects = vec![];
        if result.sound != 0 {
            effects.push(serde_json::json!(["sound", result.sound]));
        }
        if result.message != 0 {
            effects.push(serde_json::json!(["message", result.message]));
        }
        assert_eq!(serde_json::json!(effects), case["events"]);
        support::assert_rng(&random, &case["seed_after"], &case["draws"]);
    }
}
