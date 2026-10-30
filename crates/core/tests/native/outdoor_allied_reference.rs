//! Independent buttonchoice ally-copy and combat-zero spell suffix captures.
use super::outdoor_deaths::{prepare, verify};
use crate::combat::globals::BattleOrder;
use crate::combat::runtime::TurnFlow;
use crate::test_support::{bytes, fixture};
use serde_json::{Value, json};

fn words(value: &Value) -> Vec<i16> {
    bytes(value)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|raw| i16::from_be_bytes(*raw))
        .collect()
}

fn scalar(value: &Value) -> i16 {
    let raw = bytes(value);
    if raw.len() == 1 {
        i16::from(raw[0] as i8)
    } else {
        i16::from_be_bytes(raw.try_into().unwrap())
    }
}

#[test]
fn outdoor_allied_suffix_matches_native_copies_damage_and_terminal_returns() {
    for raw in fixture("outdoor-allied.json")["cases"].as_array().unwrap() {
        let source = &raw["declarations"]["source_input"];
        let before = &source["before"];
        let mut party_input = source.clone();
        party_input["before"]["mask"] = before["mask"].as_str().unwrap()[..24].into();
        let (mut state, content) = prepare(&party_input);
        state.holdover = bytes(&before["allies"]);
        state.held_over = scalar(&before["ally_count"]) as usize;
        state.party_condition = bytes(&before["buffs"]);
        state.globals.define_spells = bytes(&before["psi_memory"]);
        let mask = words(&before["mask"]);
        state.globals.party_track.copy_from_slice(&mask[..6]);
        state.globals.track_tail = mask[6..].to_vec();
        state.globals.monsters.num_mon = scalar(&before["npc_count"]);
        state.globals.spell_info = bytes(&before["packet"]);
        state.globals.cast_identity = words(&before["identity"]).try_into().unwrap();
        state.globals.attack_count = scalar(&before["attack_count"]);
        state.globals.in_spell = scalar(&before["ready"]);
        state.globals.battle_order = BattleOrder {
            order: words(&before["order"]),
            index: scalar(&before["cursor"]) as usize,
        };
        let health = &mut state.globals.health_context;
        health.char_select_new = scalar(&before["health_selected"]) as usize;
        health.kill_party = scalar(&before["health_unavailable"]) as i8;
        health.last_body = scalar(&before["health_last_body"]);
        let identity = state.globals.cast_identity;
        let id = (identity[0] + 1) * 1000 + (identity[1] + 1) * 100 + identity[2] + 1;
        let count = state.globals.attack_count;
        let packet = bytes(&source["packet_before"]);
        let initial_menu = state.holdover.clone();
        let path = std::env::temp_dir().join(format!(
            "centurions-native-outdoor-allies-{}-{}.json",
            std::process::id(),
            raw["name"].as_str().unwrap()
        ));
        crate::save(&path, &state, &content).unwrap();
        let restored = crate::load(&path, &content).unwrap();
        assert_eq!(state, restored);
        state = restored;
        std::fs::remove_file(path).unwrap();
        let outcome =
            crate::story_spell::resolve_party_outdoors(&mut state, &content, &packet, id, count)
                .unwrap_or_else(|error| panic!("{}: {error}", raw["name"]));
        let terminal = raw["pc"] == "0x13283e";
        assert!(
            outcome.flow
                == if terminal {
                    TurnFlow::Terminated
                } else {
                    TurnFlow::Returned
                }
        );
        let mut projection = raw.clone();
        projection["memory"]["mask"]["hex"] =
            raw["memory"]["mask"]["hex"].as_str().unwrap()[..24].into();
        projection["comparison"] = json!({"terminal":terminal});
        if terminal {
            // MainLoop has no active logical roster/view. Raw physical party and
            // selection bytes remain in the capture; the ally menu stays owned.
            projection["comparison"]["excluded_record_owners"] = json!(["warriors", "direction"]);
            projection["comparison"]["excluded_work_owners"] = json!(["selected_warrior"]);
            assert_eq!(state.holdover, initial_menu);
        }
        verify(&state, &content, &outcome.events, &projection);
        for (name, actual) in [
            ("allies", state.holdover.clone()),
            ("psi_memory", state.globals.define_spells.clone()),
            ("buffs", state.party_condition.clone()),
            ("packet", state.globals.spell_info.clone()),
            (
                "mask",
                state
                    .globals
                    .party_track
                    .iter()
                    .chain(&state.globals.track_tail)
                    .flat_map(|value| value.to_be_bytes())
                    .collect(),
            ),
        ] {
            assert_eq!(
                actual,
                bytes(&raw["memory"][name]["hex"]),
                "{} {name}",
                raw["name"]
            );
        }
        assert_eq!(
            state.held_over as i16,
            scalar(&raw["memory"]["ally_count"]["hex"])
        );
        assert_eq!(
            state.globals.monsters.num_mon,
            scalar(&raw["memory"]["npc_count"]["hex"])
        );
        assert_eq!(
            scalar(&raw["memory"]["current_caster"]["hex"]),
            scalar(&before["health_selected"])
        );
    }
}
