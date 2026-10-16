use crate::prelude::*;

use crate::support::field_bytes as bytes;
use centurions_core::story_inventory;
use centurions_core::story_promotion;
use centurions_core::story_promotion::LibraryRandom;
use centurions_core::story_treasure;

use crate::native_common::patched;
use crate::support::record_bytes as flatten;
use crate::support::warriors as party;

#[test]
fn native_close_promotes_each_living_member_once_and_preserves_valid_spells() {
    let fixture: Value = crate::support::fixture("level-promotion.json");

    let content = crate::support::content();
    let map = content.map(0).unwrap();
    let baseline = bytes(&fixture, "warriors");
    let interactive = crate::support::fixture("interactive-promotion.json");
    let cases = fixture.array("cases");

    for (index, case) in cases
        .iter()
        .chain(interactive.array("cases"))
        .enumerate()
        .filter(|(_, c)| c["kind"] == "close")
    {
        let before = patched(baseline.clone(), &case["before_changes"]);
        let warriors = party(&before);
        let initial = warriors.clone();
        let mut library = LibraryRandom {
            seed: case.n::<u32>("library_seed_before"),
            draws: 0,
        };
        let result = story_promotion::close(
            &warriors,
            &map.races,
            &map.profiles,
            &map.combat.as_ref().unwrap().attacks,
            case.n::<bool>("registered"),
            &mut library,
        )
        .unwrap_or_else(|e| panic!("case{index}: {e:?}"));
        let expected = patched(before, &case["after_changes"]);
        if case["controls"].is_null() {
            assert_eq!(flatten(&result.warriors), expected, "case{index}");
        }
        assert_eq!(digest(&expected), case["after_sha256"], "case{index}");
        assert_eq!(warriors, initial, "input party mutated");
        if case.n::<bool>("registered") {
            let mut state = crate::support::mission_session(&content);
            state.warriors = initial;
            state.library_random_seed = case.n("library_seed_before");
            let position = state.exploration.as_mut().unwrap();
            position.presentation = Some(Presentation::Loot);
            position.script = None;
            state.movement_continuation = None;
            let random = state.random.clone();
            let mut events = apply(Command::BeginPromotion, &mut state, &content).unwrap();
            if let Some(controls) = case["controls"].as_array() {
                for item in &controls[..controls.len() - 1] {
                    events.extend(
                        apply(
                            Command::PromotionControl {
                                item: item.as_u64().unwrap() as u16,
                            },
                            &mut state,
                            &content,
                        )
                        .unwrap(),
                    );
                }
            }
            let checkpoint = std::env::temp_dir()
                .join(format!("centurions-promotion-{}.json", std::process::id()));
            while state.promotion.is_some() {
                events.extend(
                    apply(Command::PromotionControl { item: 32 }, &mut state, &content).unwrap(),
                );
                if state.promotion.as_ref().is_some_and(|p| p.question) {
                    let draft = state.promotion.clone();
                    apply(
                        Command::PromotionAnswer { yes: false },
                        &mut state,
                        &content,
                    )
                    .unwrap();
                    assert!(!state.promotion.as_ref().unwrap().question);
                    apply(Command::PromotionControl { item: 32 }, &mut state, &content).unwrap();
                    assert_eq!(state.promotion, draft);
                    save(&checkpoint, &state, &content).unwrap();
                    let mut resumed = load(&checkpoint, &content).unwrap();
                    assert_eq!(state, resumed);
                    let next = apply(Command::PromotionAnswer { yes: true }, &mut state, &content)
                        .unwrap();
                    assert_eq!(
                        next,
                        apply(
                            Command::PromotionAnswer { yes: true },
                            &mut resumed,
                            &content
                        )
                        .unwrap()
                    );
                    events.extend(next);
                    assert_eq!(state, resumed);
                }
            }
            let _ = std::fs::remove_file(checkpoint);
            assert_eq!(
                flatten(&state.warriors),
                expected,
                "command promotion case{index}"
            );
            assert_eq!(state.random, random);
            if let Some(sounds) = case["sounds"].as_array() {
                assert_eq!(
                    events
                        .iter()
                        .filter_map(|event| match event {
                            Event::Sound { resource_id } => Some(*resource_id),
                            _ => None,
                        })
                        .collect::<Vec<_>>(),
                    sounds
                        .iter()
                        .map(|s| s.as_i64().unwrap() as i16)
                        .collect::<Vec<_>>()
                );
            }
            assert_eq!(
                (state.library_random_seed, state.library_random_draws),
                (library.seed, library.draws)
            );
        }
        crate::support::native_expect! {
        scalar library.seed => &case["library_seed_after"], "binary_level_promotion";
        scalar library.draws => &case["library_draws"], "binary_level_promotion";
        }
    }
}

#[test]
fn original_reward_chain_crosses_level_thresholds_at_close() {
    let fixture: Value = crate::support::fixture("level-promotion.json");

    let content = crate::support::content();
    let map = content.map(0).unwrap();
    let baseline = bytes(&fixture, "warriors");
    for case in fixture
        .array("cases")
        .iter()
        .filter(|c| c["kind"] == "chain")
    {
        let mut warriors = party(&patched(baseline.clone(), &case["before_changes"]));
        let mut library = LibraryRandom { seed: 1, draws: 0 };
        let mut random = RandomState {
            seed: case.n::<u32>("seed_before"),
            draws: 0,
        };
        for stage in case.array("stages") {
            let n = stage.n::<u64>("stage");
            let table = bytes(stage, "table");
            let loot = if n == 1 {
                let prepared =
                    story_treasure::prepare(&warriors, &table, &content, &mut random, 0).unwrap();
                warriors = prepared.warriors;
                prepared.items
            } else {
                let amount = i32::from_be_bytes(table[80..84].try_into().unwrap());
                warriors = story_inventory::award_story_experience(&warriors, amount)
                    .unwrap()
                    .warriors;
                vec![]
            };
            warriors = story_promotion::close(
                &warriors,
                &map.races,
                &map.profiles,
                &map.combat.as_ref().unwrap().attacks,
                true,
                &mut library,
            )
            .unwrap()
            .warriors;
            assert_eq!(
                flatten(&warriors),
                bytes(stage, "warriors"),
                "chain stage{n}"
            );
            let mut crate_bytes = vec![0; 22000];
            for (i, item) in loot.iter().enumerate() {
                crate_bytes[i * 22..i * 22 + 22].copy_from_slice(&item.0);
            }
            assert_eq!(crate_bytes, bytes(stage, "crate"), "chain stage{n}");
            crate::support::native_expect! {
            scalar library.seed => &stage["library_seed"], "binary_level_promotion";
            scalar library.draws => &stage["library_draws"], "binary_level_promotion";
            scalar random.seed => &stage["seed"], "binary_level_promotion";
            scalar random.draws => &stage["draws"], "binary_level_promotion";
            }
            assert_eq!(stage["currency"], 937);
        }
    }
}
