//! Native Items combat return after the independently declared inventory UI join.
use super::outdoor_deaths::{prepare, verify};
use crate::combat::item_activation::{Caller, activate_selected, encounter_return};
use crate::test_support::{bytes, fixture};
use crate::{Content, Event, Phase, RuleError, Session};
use centurions_content::InventoryRecord;
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

fn input(source: &Value) -> (Session, Content) {
    let before = &source["before"];
    let mut input = source.clone();
    input["before"]["mask"] = before["mask"].as_str().unwrap()[..24].into();
    let (mut state, mut content) = prepare(&input);
    if let Some(definition) = source["definition_override"].as_str() {
        let item = source["item"].as_u64().unwrap() as usize;
        let definition = bytes(&json!(definition));
        let bank = bytes(&before[format!("definition_bank_{}", item / 200 * 200)]);
        assert_eq!(&bank[item % 200 * 150..(item % 200 + 1) * 150], definition);
        content.items[item] = definition;
    }
    if let Some(packet) = source["packet_override"].as_str() {
        let id = source["spell"].as_i64().unwrap() as usize;
        let offset = (id / 1000 - 1) * 2842 + (id % 1000 / 100 - 1) * 406 + (id % 100 - 1) * 58;
        let attacks =
            &mut std::sync::Arc::make_mut(&mut content.scenario.as_mut().unwrap().maps[0].shared)
                .combat
                .as_mut()
                .unwrap()
                .attacks;
        attacks[offset..offset + 58].copy_from_slice(&bytes(&json!(packet)));
    }
    state.allies = bytes(&before["allies"]);
    state.ally_count = scalar(&before["ally_count"]) as usize;
    state.native_work.psi_memory = bytes(&before["psi_memory"]);
    state.native_work.inventory_record =
        InventoryRecord::read(&bytes(&before["inventory_record"])).unwrap();
    state.native_work.npc_backing.active_count = scalar(&before["npc_count"]);
    let mask = words(&before["mask"]);
    state.native_work.party_mask.copy_from_slice(&mask[..6]);
    state.native_work.spell_target_tail = mask[6..].to_vec();

    // Only unread Rust battle metadata comes from this foundation. Every field
    // consumed by the captured Items suffix comes from its literal native input.
    let mut combat: crate::combat::Combat =
        serde_json::from_value(fixture("player-psi.json")["route_foundation"]["combat"].clone())
            .unwrap();
    let work = &state.native_work.tactical_work;
    combat.difficulty = state.difficulty;
    combat.map = work.map.clone();
    combat.squad.positions = work.positions.to_vec();
    combat.squad.under_tiles = work.under_tiles.to_vec();
    combat.enemies.records = state
        .native_work
        .npc_backing
        .slots
        .iter()
        .map(|slot| slot.as_ref().unwrap().record.clone())
        .collect();
    combat.enemies.positions = words(&before["npc_positions"])
        .as_chunks::<2>()
        .0
        .iter()
        .map(|point| (point[0], point[1]))
        .collect();
    combat.retained_npcs = state.native_work.npc_backing.clone();
    combat.order = words(&before["order"]);
    combat.cursor = scalar(&before["cursor"]) as usize;
    combat.round = source["turn_round"].as_i64().unwrap() as i16;
    combat.inactive_rounds = scalar(&before["inactive"]);
    combat.spell_fields = work.spell_fields.clone();
    combat.field_used = work.field_used.clone();
    combat.ready = state.native_work.ready;
    combat.unavailable = state.native_work.health_context.unavailable;
    combat.attack_packet = state.native_work.attack_packet.clone();
    combat.attack_identity = state.native_work.attack_identity;
    combat.weapon_work = state.native_work.weapon_work;
    combat.victory_work = state.native_work.victory_work;
    combat.neighbor_work = state.native_work.neighbor_work.clone();
    combat.selected_enemy = state.native_work.selected_enemy;
    combat.target_tail = mask[6..].to_vec();
    combat.script_enemy = scalar(&before["script_enemy"]) as usize;
    state.combat = Some(combat);
    state.phase = Phase::Combat;
    assert_eq!(scalar(&before["combat_mode"]), 1);
    state.validate(&content).unwrap();
    (state, content)
}

#[test]
fn combat_encounter_item_prefix_matches_native_records_work_and_returns() {
    let captures = fixture("combat-encounter-items.json");
    let mut compared = [0; 3];
    for (section, joined, callback) in [
        ("cases", false, false),
        ("caller_cases", true, false),
        ("callback_cases", false, true),
    ] {
        for raw in captures[section].as_array().unwrap() {
            let source = &raw["declarations"]["source_input"];
            assert_eq!(
                source["parent"],
                if callback {
                    "newland-typed-item-callback"
                } else if joined {
                    "encounter2-items"
                } else {
                    "items-combat-typed"
                }
            );
            compared[if callback { 2 } else { usize::from(joined) }] += 1;
            let (mut state, mut content) = input(source);
            if joined {
                let mut record = bytes(&source["before"]["typed_record"]);
                assert_eq!(record.len(), 156);
                // Labels and Rust modal metadata are unread by the captured return.
                record.resize(516, 0);
                state
                    .native_work
                    .encounter_work
                    .load(&record, 0, true)
                    .unwrap();
                let original = bytes(&source["before"]["script_actions"]);
                let mut script = crate::ScriptState::scheduled(original.clone(), 0);
                script.menu = Some(crate::MenuContinuation {
                    id: 0,
                    remaining: 1,
                    original,
                    typed: true,
                    eliminated: 0,
                });
                let position = state.exploration.as_mut().unwrap();
                position.script = Some(script);
                position.presentation = Some(crate::Presentation::Typed {
                    id: 0,
                    text: String::new(),
                });
                state.validate(&content).unwrap();
            }
            let events = if callback {
                let record = bytes(&source["script_record"]);
                let table_id = centurions_content::word(&record, 24).unwrap() as usize;
                let typed = bytes(&source["typed_record_override"]);
                assert_eq!(typed.len(), 516);
                let table = &mut std::sync::Arc::make_mut(
                    &mut content.scenario.as_mut().unwrap().maps[0].shared,
                )
                .typed_encounters;
                table[table_id * 516..(table_id + 1) * 516].copy_from_slice(&typed);
                let flags = bytes(&source["before"]["story_flags"]);
                state.flags[..flags.len()].copy_from_slice(&flags);
                let position = state.exploration.as_mut().unwrap();
                position.script = Some(crate::ScriptState::scheduled(record, 0));
                position.presentation = Some(crate::Presentation::Dialogue {
                    id: 0,
                    text: String::new(),
                });
                let entry = crate::apply(crate::Command::Continue, &mut state, &content).unwrap();
                // The native encounter wrapper omits its selection UI prelude.
                // The public entry sound is outside that captured service scope.
                assert_eq!(
                    entry
                        .iter()
                        .filter_map(|event| if let Event::Sound { resource_id } = event {
                            Some(*resource_id)
                        } else {
                            None
                        })
                        .collect::<Vec<_>>(),
                    [20005]
                );
                assert!(
                    matches!(state.exploration.as_ref().unwrap().presentation, Some(crate::Presentation::Typed { id, .. }) if id == table_id as i16)
                );
                let path = std::env::temp_dir().join(format!(
                    "centurions-typed-item-callback-{}.json",
                    std::process::id()
                ));
                crate::save(&path, &state, &content).unwrap();
                let restored = crate::load(&path, &content).unwrap();
                assert!(state == restored, "typed callback checkpoint changed state");
                state = restored;
                std::fs::remove_file(path).unwrap();
                let events = crate::apply(
                    crate::Command::UseItem {
                        warrior: source["actor"].as_u64().unwrap() as usize,
                        slot: source["slot"].as_u64().unwrap() as usize,
                    },
                    &mut state,
                    &content,
                )
                .unwrap();
                assert_eq!(
                    &state.flags[..flags.len()],
                    bytes(&raw["memory"]["story_flags"]["hex"])
                );
                assert!(state.exploration.as_ref().unwrap().script.is_none());
                assert!(state.exploration.as_ref().unwrap().presentation.is_none());
                assert_eq!(raw["registers"]["D0"], 0);
                events
            } else {
                let result = activate_selected(
                    &mut state,
                    &content,
                    source["actor"].as_u64().unwrap() as usize,
                    source["slot"].as_u64().unwrap() as usize,
                    Caller::Encounter,
                );
                if raw["pc"] == "0x129aca" {
                    let Err(RuleError::RefusedAfterEffects { feedback }) = result else {
                        panic!("{} expected the native refusal", raw["name"]);
                    };
                    let expected: Vec<_> = raw["calls"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .filter(|call| call["id"] == "warning")
                        .map(|call| crate::feedback::NativeFeedback::Warning {
                            index: scalar(&call["arguments"]["index"]),
                        })
                        .collect();
                    assert_eq!(feedback, expected);
                    feedback.into_iter().map(Event::Feedback).collect()
                } else {
                    assert_eq!(raw["pc"], "0x900000");
                    let result = result.unwrap_or_else(|error| panic!("{}: {error}", raw["name"]));
                    let returned = if joined {
                        i16::from(
                            encounter_return(&mut state, &content, result.item)
                                .unwrap()
                                .unwrap(),
                        )
                    } else {
                        result.item
                    };
                    assert_eq!(returned as u64, raw["registers"]["D0"].as_u64().unwrap());
                    result.events
                }
            };
            let mut projection = raw.clone();
            projection["memory"]["mask"]["hex"] =
                raw["memory"]["mask"]["hex"].as_str().unwrap()[..24].into();
            let mut requests = vec![
                json!({"instruction":1994,"resource":601,"index":101}),
                json!({"instruction":2004,"resource":601,"index":101}),
                json!({"instruction":11607,"resource":133,"index":12}),
                json!({"instruction":2345,"resource":601,"index":101}),
            ];
            requests.extend(
                [114, 138, 162, 186, 210, 234, 258, 282, 306]
                    .into_iter()
                    .map(|instruction| json!({"instruction":instruction,"resource":3,"index":1})),
            );
            projection["comparison"] = json!({"excluded_resource_requests": requests});
            verify(&state, &content, &events, &projection);
            assert_eq!(
                state.native_work.character_bare_hand_bonus,
                scalar(&raw["memory"]["character_work_bonus"]["hex"])
            );
            let actual: Vec<_> = (state.spell_targets().unwrap().iter())
                .flat_map(|value| value.to_be_bytes())
                .collect();
            for (name, actual) in [
                ("mask", actual),
                ("psi_memory", state.native_work.psi_memory.clone()),
                ("allies", state.allies.clone()),
                (
                    "inventory_record",
                    state.native_work.inventory_record.0.to_vec(),
                ),
            ] {
                let expected = bytes(&raw["memory"][name]["hex"]);
                assert!(
                    actual == expected,
                    "{} {name}: first byte {:?}",
                    raw["name"],
                    actual.iter().zip(&expected).position(|(a, b)| a != b)
                );
            }
            if joined || callback {
                assert_eq!(
                    state.native_work.encounter_work.header(true),
                    bytes(&raw["memory"]["typed_record"]["hex"])
                );
            }
            assert!(state.medical_continuation.is_none() && state.psi_selection.is_none());
            assert!(state.outdoor_psi.is_none());
            assert_eq!(
                state.combat.as_ref().unwrap().ready,
                state.native_work.ready
            );
        }
    }
    assert_eq!(compared, [5, 2, 1]);
}
