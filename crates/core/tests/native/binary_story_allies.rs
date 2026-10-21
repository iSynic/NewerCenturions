use crate::{prelude::*, support};

use centurions_core::story_allies::add_ally;

// Recruitment, footprints and actual turns share the saved campaign command boundary.
#[test]
fn authored_large_allies_enter_battle_and_resume_their_turns() {
    let content = content();
    let mut state = support::mission_session(&content);
    let map = content.map(9).unwrap();
    let position = state.exploration.as_mut().unwrap();
    position.restore_map(9, centurions_core::MapState::authored(map));
    position.script = Some(support::script_state(map.script(271).unwrap().to_vec(), 3));
    position.presentation = Some(Presentation::Dialogue {
        id: 1025,
        text: map.dialogue(1025).unwrap(),
    });
    apply(Command::Continue, &mut state, &content).unwrap();
    assert_eq!(state.ally_count, 7);
    let position = state.exploration.as_mut().unwrap();
    position.script = Some(support::script_state(
        content.map(9).unwrap().script(55).unwrap().to_vec(),
        0,
    ));
    apply(Command::Continue, &mut state, &content).unwrap();
    assert_eq!(current_combat(&state).enemies.records.len(), 8);
    state.combat.as_mut().unwrap().enemies.records[7].bytes[0x110..0x112]
        .copy_from_slice(&30000_i16.to_be_bytes());
    let mut resumed = support::checkpoint(&state, &content, "large-allies");
    for body in 10..17 {
        for candidate in [&mut state, &mut resumed] {
            let combat = candidate.combat.as_mut().unwrap();
            combat.order[combat.cursor] = body;
            combat.turn_origin = None;
            combat.enemies.records[(body - 10) as usize].bytes[0x109] = 2;
        }
        support::replay_command(Command::RunEnemyTurn, &mut state, &mut resumed, &content);
    }
}

#[test]
fn joined_biologists_match_complete_original_records_and_rng() {
    let fixture: serde_json::Value = support::fixture("story-allies.json");
    let definition = fixture.bytes("definition");
    for case in fixture.array("cases") {
        let mut records = case.bytes("before");
        let mut count = case.n::<usize>("count_before");
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let mut warnings = Vec::new();
        for _ in 0..case.n::<u64>("calls") {
            let result = add_ally(
                &records,
                count,
                &definition,
                case.n::<i16>("difficulty"),
                &mut random,
            )
            .unwrap();
            records = result.records;
            count = result.count;
            if let Some(warning) = result.warning {
                warnings.push(warning);
            }
        }
        support::native_expect! {
            bytes records => &case["after"], format!("case count {} seed {}",case["count_before"],case["seed_before"]);
            scalar count => &case["count_after"], "binary_story_allies";
            rng random => (&case["seed_after"], &case["rng_draws"]), "binary_story_allies";
            scalar (warnings) => &case["warnings"], "binary_story_allies";
        }
    }
}
