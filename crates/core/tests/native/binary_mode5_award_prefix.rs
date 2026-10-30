//! Original award prefix; runtime mode5 allocation and later party profiles are separate.
use crate::support::{self, NativeCase};
use centurions_core::RandomState;
use centurions_core::combat::rewards::{self, RewardContext, RewardMode};
use serde_json::Value;

#[test]
fn no_loot_awards_only_the_first_npc_after_normalizing_both_records() {
    let path = support::root().join("analysis/backlog-45/local-mode5-prefix.json");
    let fixture: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    let content = support::content();
    let before = &fixture["before"];
    let warriors = support::warriors(&before["warriors_before"]);
    let enemies = support::monsters(&before["npcs_before"]);
    let draws_before = before.n::<u64>("draws_before");
    let currency = before.n::<i32>("currency_before");
    for case in fixture.array("cases") {
        let name = case.text("name");
        let mut random = RandomState {
            seed: before.n("seed_before"),
            draws: draws_before,
        };
        let mode = match case.n::<i16>("mode") {
            0 => RewardMode::Ordinary,
            5 => RewardMode::NoLoot,
            other => panic!("unexpected native award mode {other}"),
        };
        let result = rewards::prepare_mode(
            &warriors,
            &enemies,
            &content,
            RewardContext {
                difficulty: before.n("difficulty"),
                deduction: before.n("npc_denominator"),
                currency,
                mode,
            },
            &mut random,
        )
        .unwrap_or_else(|error| panic!("{name}: {error:?}"));
        assert_eq!(
            support::record_bytes(&result.monsters),
            case.bytes("npcs_after"),
            "{name} complete active NPC records"
        );
        assert_eq!(result.total, case.n::<i32>("total_xp"), "{name} XP total");
        // A3 is the native cash accumulator before its later global credit.
        assert_eq!(
            result.currency,
            currency.wrapping_add(case.n::<i32>("cash_accumulator")),
            "{name} cash"
        );
        assert_eq!(random.seed, case.n::<u32>("seed_after"), "{name} seed");
        assert_eq!(
            random.draws - draws_before,
            case.n::<u64>("delta_draws"),
            "{name} draws"
        );
    }
}
