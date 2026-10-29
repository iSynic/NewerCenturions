//! Bounded original attack/checkforenemy over retained warrior and NPC arrays.
//! UI and warning presentation are controlled; ordered sound/resource requests
//! and complete retained records are compared. SANE model calls do not qualify ROM.
use super::detached_attacks::{compare, initial, saved_return};
use crate::Event;
use crate::combat::runtime::detached_warrior::run;
use crate::combat::runtime::reaction_calls::AttackCall;
use crate::combat::runtime::reaction_frame::ReactionFrame;
use crate::feedback::NativeFeedback;
use crate::test_support::{bytes, content, fixture};

#[test]
fn detached_warrior_attacks_match_native_records_work_rng_and_requests() {
    let captures = fixture("detached-warrior.json");
    let original_content = content();
    for case in captures["cases"].as_array().unwrap() {
        assert_eq!(case["status"], "completed");
        let source = &case["declarations"]["source_input"];
        let mut content = original_content.clone();
        if let Some(overrides) = source["custom_row_overrides"].as_array() {
            for row in overrides {
                let index = row["index"].as_u64().unwrap() as usize;
                let raw = bytes(&row["hex"]);
                match row["table"].as_str().unwrap() {
                    "item_definitions" => content.items[index] = raw,
                    "attack_packets" => {
                        assert_eq!(raw.len(), 58);
                        let scenario = content.scenario.as_mut().unwrap();
                        assert_eq!(scenario.name, "The Thunder of Guns");
                        assert_eq!(scenario.level, 0);
                        for map in &mut scenario.maps {
                            if let Some(combat) =
                                std::sync::Arc::make_mut(&mut map.shared).combat.as_mut()
                            {
                                combat.attacks[index * 58..(index + 1) * 58].copy_from_slice(&raw);
                            }
                        }
                    }
                    table => panic!("unexpected native table override {table}"),
                }
            }
        }
        let mut state = initial(source, &content);
        state.native_work.character_bare_hand_bonus =
            source["character_work_bonus"].as_i64().unwrap() as i16;
        let actor = source["actor"].as_i64().unwrap() as i16;
        let (returned, events) = if source["parent"] == "checkforenemy-warrior-guard" {
            let outcome = ReactionFrame::new(actor, 0, (1, 0))
                .unwrap()
                .finish_after_teardown(&mut state, &content)
                .unwrap_or_else(|error| panic!("{}: {error}", case["name"]));
            (outcome.interrupted, outcome.events)
        } else {
            let outcome = run(
                &mut state,
                &content,
                AttackCall {
                    attacker: source["attacker"].as_i64().unwrap() as i16,
                    recipient: source["target"].as_i64().unwrap() as i16,
                    index: 0,
                    retained_actor: actor,
                },
            )
            .unwrap_or_else(|error| panic!("{}: {error}", case["name"]));
            (outcome.returned, outcome.events)
        };
        assert_eq!(
            returned,
            case["registers"]["D0"].as_u64().unwrap() & 0xffff != 0,
            "{} return",
            case["name"]
        );
        compare(&state, &events, case);
        let bonus = bytes(&case["memory"]["character_work_bonus"]["hex"]);
        assert_eq!(
            state.native_work.character_bare_hand_bonus,
            i16::from_be_bytes(bonus.try_into().unwrap()),
            "{} retained character bonus",
            case["name"]
        );
        let warnings: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                Event::Feedback(NativeFeedback::Warning { index }) => Some(*index),
                _ => None,
            })
            .collect();
        let native: Vec<_> = case["calls"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|call| call["id"] == "warning")
            .map(|call| {
                let raw = bytes(&call["arguments"]["id"]);
                i16::from_be_bytes(raw.try_into().unwrap())
            })
            .collect();
        assert_eq!(warnings, native, "{} warning requests", case["name"]);
        if source["parent"] == "checkforenemy-warrior-guard" {
            let mut saved = initial(source, &content);
            saved.native_work.character_bare_hand_bonus =
                source["character_work_bonus"].as_i64().unwrap() as i16;
            saved_return(&mut saved, &content);
            let events = crate::apply(crate::Command::Continue, &mut saved, &content).unwrap();
            compare(&saved, &events, case);
        }
    }
}

#[test]
fn retained_character_bonus_survives_save_and_mission_end_after_an_actual_view_writer() {
    use centurions_content::records::DevelopmentField;

    let content = content();
    let captures = fixture("detached-warrior.json");
    let mut state = initial(
        &captures["cases"][0]["declarations"]["source_input"],
        &content,
    );
    state.selected_warrior = 0;
    DevelopmentField::BareHandBonus.write(&mut state.warriors[0].bytes, -4);
    DevelopmentField::BareHandBonus.write(&mut state.warriors[1].bytes, 11);
    state.native_work.character_bare_hand_bonus = -23;
    let path = std::env::temp_dir().join(format!(
        "centurions-character-work-{}.json",
        std::process::id()
    ));
    crate::save(&path, &state, &content).unwrap();
    assert_eq!(crate::load(&path, &content).unwrap(), state);
    crate::apply(
        crate::Command::ViewWarrior { warrior: 1 },
        &mut state,
        &content,
    )
    .unwrap();
    assert_eq!(state.selected_warrior, 0);
    assert_eq!(state.native_work.character_bare_hand_bonus, 11);
    crate::apply(crate::Command::EndMission, &mut state, &content).unwrap();
    assert_eq!(state.native_work.character_bare_hand_bonus, 11);
    std::fs::remove_file(path).unwrap();
}
