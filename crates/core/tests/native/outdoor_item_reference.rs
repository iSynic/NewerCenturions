//! Native selected Items combat-zero prefixes and the encounter2 return.
use super::outdoor_deaths::{prepare, verify};
use crate::test_support::{bytes, fixture};
use crate::{Command, Content, Event, RuleError, Session};
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
    let mut shared = source.clone();
    shared["before"]["mask"] = before["mask"].as_str().unwrap()[..24].into();
    let (mut state, mut content) = prepare(&shared);
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
    state.holdover = bytes(&before["allies"]);
    state.held_over = scalar(&before["ally_count"]) as usize;
    state.globals.define_spells = bytes(&before["psi_memory"]);
    state.globals.inventory_record =
        InventoryRecord::read(&bytes(&before["inventory_record"])).unwrap();
    state.globals.monsters.num_mon = scalar(&before["npc_count"]);
    state.scenario_duration_tail = Some(scalar(&before["scenario_duration_tail"]));
    let position = state.exploration.as_mut().unwrap();
    position.minute = scalar(&before["clock_minute"]);
    position.hour = scalar(&before["clock_hour"]);
    position.day = scalar(&before["clock_day"]);
    position.fatigue = scalar(&before["fatigue"]);
    let mask = words(&before["mask"]);
    state.globals.party_track.copy_from_slice(&mask[..6]);
    state.globals.track_tail = mask[6..].to_vec();
    assert!(state.combat.is_none());
    assert_eq!(scalar(&before["combat_mode"]), 0);
    (state, content)
}

fn verify_output(state: &Session, content: &Content, events: &[Event], raw: &Value) {
    let mut projection = raw.clone();
    // Flash's controlled presentation body omits its nested sound call.
    // Its captured sound argument still records that ordered request.
    projection["calls"] = raw["calls"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .flat_map(|(index, call)| {
            let mut calls = vec![call.clone()];
            let nested_sound_recorded = raw["calls"].as_array().unwrap()[index + 1..]
                .iter()
                .take_while(|next| next["id"] != "Flash")
                .any(|next| {
                    next["id"] == "sound" && next["arguments"]["id"] == call["arguments"]["sound"]
                });
            if call["id"] == "Flash"
                && scalar(&call["arguments"]["sound"]) != 0
                && !nested_sound_recorded
            {
                calls.push(json!({"id":"sound","arguments":{"id":call["arguments"]["sound"]}}));
            }
            calls
        })
        .collect();
    projection["memory"]["mask"]["hex"] =
        raw["memory"]["mask"]["hex"].as_str().unwrap()[..24].into();
    projection["comparison"] = raw["comparison"].clone();
    projection["comparison"]["native_events_projection"] = "requested_audio".into();
    verify(state, content, events, &projection);
    assert_eq!(
        state.globals.character_bare_hand_bonus,
        scalar(&raw["memory"]["character_work_bonus"]["hex"])
    );
    assert_eq!(
        state.scenario_duration_tail,
        Some(scalar(&raw["memory"]["scenario_duration_tail"]["hex"]))
    );
    assert_eq!(
        state.globals.monsters.num_mon,
        scalar(&raw["memory"]["npc_count"]["hex"])
    );
    let position = state.exploration.as_ref().unwrap();
    for (name, actual) in [
        ("clock_minute", position.minute),
        ("clock_hour", position.hour),
        ("clock_day", position.day),
        ("fatigue", position.fatigue),
    ] {
        assert_eq!(
            actual,
            scalar(&raw["memory"][name]["hex"]),
            "{} {name}",
            raw["name"]
        );
    }
    let mask: Vec<_> = state
        .globals
        .party_track
        .iter()
        .chain(&state.globals.track_tail)
        .flat_map(|value| value.to_be_bytes())
        .collect();
    for (name, actual) in [
        ("mask", mask),
        ("psi_memory", state.globals.define_spells.clone()),
        ("allies", state.holdover.clone()),
        (
            "inventory_record",
            state.globals.inventory_record.0.to_vec(),
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
}

#[test]
fn outdoor_items_match_native_load_charge_resolution_and_return_work() {
    let captures = fixture("outdoor-item-returns.json");
    let mut compared = [0; 2];
    for (section, typed) in [("cases", false), ("caller_cases", true)] {
        for raw in captures[section].as_array().unwrap() {
            assert_eq!(raw["status"], "completed");
            let source = &raw["declarations"]["source_input"];
            assert_eq!(
                source["parent"],
                if typed {
                    "encounter2-items-outdoor"
                } else {
                    "items-outdoor-selected"
                }
            );
            compared[usize::from(typed)] += 1;
            let (mut state, content) = input(source);
            if typed {
                let mut record = bytes(&source["before"]["typed_record"]);
                assert_eq!(record.len(), 156);
                // The captured encounter returns before these unread labels and
                // no-effect Rust caller metadata. No native state is filled in.
                record.resize(516, 0);
                state
                    .globals
                    .encounter_headers
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
            }
            let owner = source["actor"].as_u64().unwrap() as usize;
            let slot = source["slot"].as_u64().unwrap() as usize;
            let result = crate::outdoor_psi::begin_item_selected(&mut state, &content, owner, slot);
            let mut events = if raw["pc"] == "0x12a1d6" {
                assert!(
                    matches!(result, Err(RuleError::RefusedAfterEffects { feedback }) if feedback.is_empty())
                );
                vec![]
            } else {
                result.unwrap_or_else(|error| panic!("{}: {error}", raw["name"]))
            };
            for command in source["choice_commands"].as_array().unwrap() {
                let command: Command = serde_json::from_value(command.clone()).unwrap();
                events.extend(
                    crate::apply(command, &mut state, &content)
                        .unwrap_or_else(|error| panic!("{}: {error}", raw["name"])),
                );
            }
            assert!(state.outdoor_psi.is_none());
            if typed {
                let mut record = bytes(&source["before"]["typed_record"]);
                record.resize(516, 0);
                assert_eq!(
                    crate::typed_encounter::select_item(
                        &record,
                        source["item"].as_i64().unwrap() as i16
                    )
                    .unwrap() as u64,
                    raw["registers"]["D0"].as_u64().unwrap()
                );
                assert_eq!(
                    state.globals.encounter_headers.header(true),
                    bytes(&raw["memory"]["typed_record"]["hex"])
                );
            }
            verify_output(&state, &content, &events, raw);
            assert!(
                state.combat.is_none()
                    && state.medical_continuation.is_none()
                    && state.psi_selection.is_none()
            );
        }
    }
    assert_eq!(compared, [5, 1]);
}

#[test]
fn inventory_opening_and_owner_switch_match_native_work_across_save() {
    let captures = fixture("inventory-opening.json");
    let cases = captures["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 2);
    let mut compared = [0; 2];
    for (index, raw) in cases.iter().enumerate() {
        assert_eq!(raw["status"], "completed");
        assert_eq!(raw["pc"], "0x129e42");
        let source = &raw["declarations"]["source_input"];
        let (mut state, content) = input(source);
        let command_name = if source["command"].get("OpenInventory").is_some() {
            compared[0] += 1;
            "OpenInventory"
        } else {
            compared[1] += 1;
            "SwitchInventoryOwner"
        };
        assert_eq!(source["command"][command_name]["quick_info"], false);
        if command_name == "SwitchInventoryOwner" {
            // A command without WORD4471aa keeps its startup row0 for quickinfo.
            let mut quick = source["command"].clone();
            quick[command_name]["quick_info"] = true.into();
            let quick: Command = serde_json::from_value(quick).unwrap();
            crate::apply(quick, &mut state.clone(), &content).unwrap();
        }
        if command_name == "SwitchInventoryOwner" {
            // The same-owner chooser guard returns before the switch body.
            let mut same = source["command"].clone();
            same[command_name]["warrior"] = state.globals.health_context.char_select_new.into();
            same[command_name]["quick_info"] = true.into();
            let same: Command = serde_json::from_value(same).unwrap();
            let before = state.clone();
            assert!(crate::apply(same, &mut state, &content).unwrap().is_empty());
            assert!(state == before, "same-owner chooser changed state");
        }
        let command: Command = serde_json::from_value(source["command"].clone()).unwrap();
        let events = crate::apply(command, &mut state, &content)
            .unwrap_or_else(|error| panic!("{}: {error}", raw["name"]));
        let path = std::env::temp_dir().join(format!(
            "centurions-inventory-opening-{}-{index}.json",
            std::process::id()
        ));
        crate::save(&path, &state, &content).unwrap();
        let restored = crate::load(&path, &content).unwrap();
        assert!(
            state == restored,
            "{} checkpoint changed state",
            raw["name"]
        );
        std::fs::remove_file(path).unwrap();
        verify_output(&restored, &content, &events, raw);
    }
    assert_eq!(compared, [1, 1]);
}

#[test]
fn historical_item_snapshots_match_the_resolver_and_recorded_owner_mask_endpoint() {
    let content = crate::test_support::content();
    let mut compared = [0; 3];
    for (group, name) in [
        "outdoor-medical.json",
        "encounter-medical.json",
        "outdoor-indexed-items.json",
    ]
    .into_iter()
    .enumerate()
    {
        let captures = fixture(name);
        for case in captures["cases"].as_array().unwrap() {
            compared[group] += 1;
            let prepared = &case["prepared"];
            let mut state = Session::new(&content);
            for command in [
                Command::ImportStarters,
                Command::EnterMission {
                    name: "The Thunder of Guns".into(),
                },
            ] {
                crate::apply(command, &mut state, &content).unwrap();
            }
            state.warriors = bytes(&prepared["party"])
                .as_chunks::<3138>()
                .0
                .iter()
                .map(|raw| centurions_content::Warrior::read(raw).unwrap())
                .collect();
            // These party-only inputs do not consume NPC coordinates/count or an
            // unrecorded selected-warrior WORD. Keep the complete captured records.
            state.globals.monsters.slots = bytes(&prepared["all_records"])
                .as_chunks::<288>()
                .0
                .iter()
                .map(|raw| {
                    Some(crate::combat::globals::MonsterSlot {
                        record: centurions_content::combat::MonsterRecord {
                            bytes: raw.to_vec(),
                        },
                        position: None,
                    })
                })
                .collect();
            state.globals.battle_order = crate::combat::globals::BattleOrder {
                order: words(&prepared["order"]),
                index: prepared["cursor"].as_u64().unwrap() as usize,
            };
            state.globals.loaded_items = crate::ammunition::LoadedItems {
                item: Some(bytes(&prepared["item_work"])),
                weapon: Some(bytes(&prepared["weapon_work"])),
                ammo: Some(bytes(&prepared["ammo_work"])),
            };
            state
                .globals
                .weapon_traits
                .load_pair(
                    &bytes(&prepared["weapon_work"]),
                    &bytes(&prepared["ammo_work"]),
                )
                .unwrap();
            state.globals.loaded_item_field32 = prepared["item_context"].as_i64().unwrap() as i16;
            state.globals.define_spells = bytes(&prepared["memory"]);
            state.party_condition = bytes(&prepared["party_buffs"]);
            state.scenario_duration_tail = prepared["scenario_duration_tail"]
                .as_i64()
                .map(|value| value as i16);
            state.globals.health_context = crate::story_health::HealthContext {
                kill_party: prepared["unavailable"].as_i64().unwrap() as i8,
                // Exact old probe mapping: selected_member is BYTE44273f.
                char_select_new: prepared["selected_member"].as_u64().unwrap() as usize,
                effect: prepared["health_effect"].as_i64().unwrap() as i8,
                last_body: prepared["health_last_body"].as_i64().unwrap() as i16,
                marks_affected: prepared["heal_marks_affected"].as_i64().unwrap() != 0,
                affected: words(&prepared["markers"])[..6].try_into().unwrap(),
            };
            let packet = bytes(&prepared["packet"]);
            let identity: [i16; 3] = words(&prepared["identity"]).try_into().unwrap();
            let id = (identity[0] + 1) * 1000 + (identity[1] + 1) * 100 + identity[2] + 1;
            let count = prepared["count"].as_i64().unwrap() as i16;
            state.globals.spell_info = packet.clone();
            state.globals.cast_identity = identity;
            state.globals.attack_count = count;
            state.random = crate::RandomState::new(prepared["seed"].as_u64().unwrap() as u32);
            state.random.draws = prepared["draws"].as_u64().unwrap();
            let mut mask = [0; 110];
            let target = case["target"].as_i64().unwrap();
            if case["global"] == true {
                mask[..state.warriors.len()].fill(1);
            } else if target >= 0 {
                mask[target as usize] = 1;
            }
            assert_eq!(mask.as_slice(), words(&prepared["mask"]));
            state.publish_spell_targets(&mask);
            let outcome =
                crate::story_spell::resolve_outdoors(&mut state, &content, &packet, id, count)
                    .unwrap();
            assert!(outcome.flow == crate::combat::runtime::TurnFlow::Returned);
            let expected = &case["after"];
            assert_eq!(
                state
                    .warriors
                    .iter()
                    .flat_map(|warrior| warrior.bytes.iter().copied())
                    .collect::<Vec<_>>(),
                bytes(&expected["party"])
            );
            assert_eq!(
                state
                    .globals
                    .monsters
                    .slots
                    .iter()
                    .flat_map(|slot| slot.as_ref().unwrap().record.bytes.iter().copied())
                    .collect::<Vec<_>>(),
                bytes(&expected["all_records"])
            );
            assert_eq!(
                state.globals.loaded_items.item.as_ref().unwrap(),
                &bytes(&expected["item_work"])
            );
            assert_eq!(
                state.globals.loaded_items.weapon.as_ref().unwrap(),
                &bytes(&expected["weapon_work"])
            );
            assert_eq!(
                state.globals.loaded_items.ammo.as_ref().unwrap(),
                &bytes(&expected["ammo_work"])
            );
            assert_eq!(state.globals.define_spells, bytes(&expected["memory"]));
            assert_eq!(state.globals.spell_info, bytes(&expected["packet"]));
            assert_eq!(
                state.globals.cast_identity.as_slice(),
                words(&expected["identity"])
            );
            assert_eq!(
                state.globals.attack_count,
                expected["count"].as_i64().unwrap() as i16
            );
            assert_eq!(state.random.seed, expected["seed"].as_u64().unwrap() as u32);
            assert_eq!(state.random.draws, expected["draws"].as_u64().unwrap());
            assert_eq!(state.spell_targets().unwrap(), mask);
            // The historical typed endpoint12c436 includes the owner bit at
            //12c430, before the later marks clear. Project that one bit on a copy.
            let mut endpoint_mask = mask;
            if case.get("encounter").is_some() {
                endpoint_mask[case["actor"].as_u64().unwrap() as usize] = 1;
            }
            assert_eq!(endpoint_mask.as_slice(), words(&expected["mask"]));
            if case["global"] == true {
                assert_eq!(
                    state.scenario_duration_tail,
                    Some(expected["scenario_duration_tail"].as_i64().unwrap() as i16)
                );
            }
            // PREPARED is the151022 input. These declared earlier Items/getchoice
            // requests plus actual resolver audio retain the full old audio oracle.
            let mut sounds = vec![655];
            if case["global"] != true {
                sounds.push(6001);
                if target >= 0 {
                    sounds.push(141);
                }
            }
            sounds.extend(outcome.events.iter().filter_map(|event| match event {
                Event::Sound { resource_id } => Some(*resource_id),
                Event::Feedback(crate::feedback::GameFeedback::CombatResult {
                    code,
                    audible: true,
                    ..
                }) => crate::feedback::combat_result(*code).map(|(_, sound)| sound),
                _ => None,
            }));
            assert_eq!(
                sounds,
                case["sounds"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|value| value.as_i64().unwrap() as i16)
                    .collect::<Vec<_>>()
            );
        }
    }
    assert_eq!(compared, [5, 3, 2]);
}

// Optional quickinfo132b0c on entry, against the original opening caller run on
// Codex's controlled quickinfo input (analysis/backlog-43/native/make_quickinfo.py).
#[test]
fn quick_information_on_open_matches_native_item_ammo_and_spell_work() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../analysis/backlog-43/native/quickinfo-opening.json");
    let capture: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    let mut raw = capture["cases"][1].clone();
    assert_eq!(raw["pc"], "0x129e42");
    // Every string request here is quickinfo drawing its panel; the desktop draws
    // that panel from the same work buffers, which are compared below.
    let panel: Vec<Value> = raw["calls"].as_array().unwrap().iter()
        .filter(|call| call["id"] == "GetIndString")
        .map(|call| json!({"instruction": call["instruction"], "resource": scalar(&call["arguments"]["resource"]), "index": scalar(&call["arguments"]["index"])}))
        .collect();
    raw["comparison"]["excluded_resource_requests"] = panel.into();
    let mut source =
        fixture("inventory-opening.json")["cases"][0]["declarations"]["source_input"].clone();
    let mut party = bytes(&source["before"]["warriors"]);
    let row = 4 * 3138 + 0x132 + 3 * 22;
    party[row + 4..row + 6].copy_from_slice(&200_i16.to_be_bytes());
    for (offset, value) in [
        (11, 2),
        (13, 3),
        (14, 4),
        (15, 2),
        (16, 1),
        (17, 2),
        (18, 3),
        (19, 4),
        (20, 5),
    ] {
        party[row + offset] = value;
    }
    source["before"]["warriors"] = party
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>()
        .into();
    let (mut state, mut content) = input(&source);
    content.items[1][44..46].copy_from_slice(&1101_i16.to_be_bytes());
    // The later updatecharinfo suffix reloads ITEM work, so compare quickinfo's
    // own writes at its return (129b68) on the compacted owner.
    let prefix = &capture["cases"][0]["memory"];
    let mut detail = state.clone();
    crate::items::compact(&mut detail.warriors[4]).unwrap();
    crate::items::quick_info(&mut detail, &content, 4, 0).unwrap();
    let work = &detail.globals;
    assert_eq!(
        work.loaded_items.item.as_deref().unwrap(),
        bytes(&prefix["item_work"]["hex"])
    );
    assert_eq!(
        work.loaded_items.ammo.as_deref().unwrap(),
        bytes(&prefix["ammo_work"]["hex"])
    );
    assert_eq!(work.spell_info, bytes(&prefix["packet"]["hex"]));
    assert_eq!(
        work.cast_identity.to_vec(),
        words(&prefix["identity"]["hex"])
    );
    assert_eq!(
        work.weapon_traits.ammunition_armor_filter,
        scalar(&prefix["ammo_filter"]["hex"])
    );
    let command = Command::OpenInventory {
        warrior: 4,
        quick_info: true,
    };
    let events = crate::apply(command, &mut state, &content).unwrap();
    verify_output(&state, &content, &events, &raw);
}
