//! Bounded original attack2/checkforenemy records and saved source caller joins.
use crate::combat::battlefield::Battlefield;
use crate::combat::globals::{BattleOrder, MonsterSlot, Monsters};
use crate::combat::neighbor_scan::Neighbors;
use crate::combat::runtime::out_of_combat_attack::run;
use crate::combat::runtime::reaction_calls::AttackCall;
use crate::combat::runtime::reaction_frame::ReactionFrame;
use crate::combat::runtime::{CombatReturn, DeathStory};
use crate::combat::weapon_traits::WeaponTraits;
use crate::test_support::{bytes, content, fixture};
use crate::{Command, Content, Event, Presentation, RandomState, ScriptState, Session};
use centurions_content::combat::MonsterRecord;
use centurions_content::{Warrior, word};
use serde_json::{Value, json};

fn words(value: &Value) -> Vec<i16> {
    bytes(value)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| i16::from_be_bytes(*pair))
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

fn encoded(values: impl IntoIterator<Item = i16>) -> Vec<u8> {
    values.into_iter().flat_map(i16::to_be_bytes).collect()
}

#[test]
fn detached_attacks_match_native_records_work_rng_and_ordered_sounds() {
    let captures = fixture("detached-attacks.json");
    let original_content = content();
    for case in captures["cases"].as_array().unwrap() {
        assert_eq!(case["status"], "completed");
        let source = &case["declarations"]["source_input"];
        let mut content = original_content.clone();
        if let Some(weapon) = source.get("weapon_override") {
            content.items[weapon["id"].as_u64().unwrap() as usize] = bytes(&weapon["hex"]);
        }
        let mut state = initial(source, &content);
        let (returned, events) = if source["parent"] == "attack2" {
            let outcome = run(
                &mut state,
                &content,
                AttackCall {
                    attacker: source["attacker"].as_i64().unwrap() as i16,
                    recipient: source["target"].as_i64().unwrap() as i16,
                    index: source["index"].as_u64().unwrap() as usize,
                    turn_actor: 10,
                },
            )
            .unwrap_or_else(|error| panic!("{}: {error}", case["name"]));
            (outcome.returned, outcome.events)
        } else {
            let actor = state.globals.battle_order.order[0];
            let outcome = ReactionFrame::new(actor, 0, (1, 0))
                .unwrap()
                .finish_after_teardown(&mut state, &content)
                .unwrap_or_else(|error| panic!("{}: {error}", case["name"]));
            (outcome.interrupted, outcome.events)
        };
        assert_eq!(
            returned,
            case["registers"]["D0"].as_u64().unwrap() != 0,
            "{} return",
            case["name"]
        );
        compare(&state, &events, case);
        if case["name"] == "weapon-condition-retained-item-shield" {
            // GetIndString (CODE 2 +0x17c) leaves the text empty for an index of 0 or below.
            let mut empty = initial(source, &content);
            let item = empty.globals.loaded_items.item.as_mut().unwrap();
            item[46..48].copy_from_slice(&0_i16.to_be_bytes());
            let call = AttackCall {
                attacker: source["attacker"].as_i64().unwrap() as i16,
                recipient: source["target"].as_i64().unwrap() as i16,
                index: source["index"].as_u64().unwrap() as usize,
                turn_actor: 10,
            };
            let outcome = run(&mut empty, &content, call).unwrap();
            let condition = |event: &Event| {
                matches!(
                    event,
                    Event::Feedback(crate::feedback::GameFeedback::CombatResourceResult {
                        resource_id: 134,
                        ..
                    })
                )
            };
            assert_eq!(outcome.events.len(), events.len());
            for (actual, original) in outcome.events.iter().zip(&events) {
                if condition(original) {
                    assert_eq!(
                        *actual,
                        Event::Feedback(crate::feedback::GameFeedback::CombatResourceResult {
                            resource_id: 134,
                            index: 0,
                        })
                    );
                } else {
                    assert_eq!(actual, original);
                }
            }
            assert_eq!(empty.warriors, state.warriors);
            assert_eq!(empty.random, state.random);
        }
        if source["parent"] == "checkforenemy" {
            let mut lost = initial(source, &content);
            lost.warriors[3].record_mut().set_stamina(1);
            lost.globals.health_context.kill_party = 5;
            saved_return(&mut lost, &content);
            let events = crate::apply(Command::Continue, &mut lost, &content).unwrap();
            assert_eq!(lost.phase, crate::Phase::SquadSelection);
            assert!(events.contains(&Event::MissionEnded { defeated: true }));
            assert!(
                !events
                    .iter()
                    .any(|event| matches!(event, Event::AttackResolved { .. }))
            );
            let mut saved = initial(source, &content);
            saved_return(&mut saved, &content);
            let events = crate::apply(Command::Continue, &mut saved, &content).unwrap();
            compare(&saved, &events, case);
            assert!(saved.combat.is_none() && saved.combat_story.is_none());
        }
    }
}

#[test]
fn saved_reaction_strike_resumes_attack2_against_an_npc_target() {
    let captures = fixture("detached-attacks.json");
    let case = captures["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["name"] == "nonlethal")
        .unwrap();
    let content = content();
    let mut state = initial(&case["declarations"]["source_input"], &content);
    // NPC 10 struck once; its next natural attack goes to its target body, NPC 11.
    let backing = &mut state.globals.monsters;
    let mut victim = backing.slot(0).unwrap().clone();
    victim.position = victim.position.map(|(x, y)| (x + 1, y));
    *backing.slot_mut(1).unwrap() = victim;
    let striker = &mut backing.slot_mut(0).unwrap().record;
    striker.record_mut().set_target(11);
    state.globals.monster_up = 0;
    state.globals.weapon_traits.natural_attack_index = 0;
    saved_strike_return(&mut state, &content, false);
    let draws = state.random.draws;
    let events = crate::apply(Command::Continue, &mut state, &content).unwrap();
    assert!(events.iter().any(|event| matches!(
        event,
        Event::AttackResolved {
            attacker: 10,
            target: 11,
            ..
        }
    )));
    assert!(state.random.draws > draws);
}

pub(super) fn initial(input: &Value, content: &Content) -> Session {
    let before = &input["before"];
    let mut state = Session::new(content);
    for command in [
        Command::ImportStarters,
        Command::EnterMission {
            name: "The Thunder of Guns".into(),
        },
    ] {
        crate::apply(command, &mut state, content).unwrap();
    }
    state.warriors = bytes(&before["warriors"])
        .as_chunks::<3138>()
        .0
        .iter()
        .map(|raw| Warrior::read(raw).unwrap())
        .collect();
    state.difficulty = 0; // The native declarations explicitly initialize442576 to0.
    state.char_select_new = scalar(&before["selected_warrior"]) as usize;
    state.globals.monster_up = scalar(&before["selected_enemy"]) as usize;
    state.exploration.as_mut().unwrap().last_move = bytes(&before["direction"])
        .into_iter()
        .map(|value| value as i8)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    let npc_positions = words(&before["npc_positions"]);
    state.globals.monsters = Monsters {
        num_mon: 1,
        origin: (5, 5),
        slots: bytes(&before["npcs"])
            .as_chunks::<288>()
            .0
            .iter()
            .enumerate()
            .map(|(index, raw)| {
                Some(MonsterSlot {
                    record: MonsterRecord {
                        bytes: raw.to_vec(),
                    },
                    position: Some((
                        i32::from(npc_positions[index * 2]) + 5,
                        i32::from(npc_positions[index * 2 + 1]) + 5,
                    )),
                })
            })
            .collect(),
    };
    let positions = words(&before["party_positions"]);
    let under = words(&before["under"]);
    state.globals.battlefield = Battlefield {
        field: crate::combat::CombatField {
            tiles: words(&before["map"]),
            map_shift: (0, 0),
            edge_word: 0,
        },
        saved_land: None,
        pos: std::array::from_fn(|index| (positions[index * 2], positions[index * 2 + 1])),
        char_under: std::array::from_fn(|index| under[index]),
        spell_queue: bytes(&before["fields"]),
        collide_check: bytes(&before["used"]),
        battle: None,
    };
    state.globals.battle_order = BattleOrder {
        order: words(&before["order"]),
        index: scalar(&before["cursor"]) as usize,
    };
    state.globals.kill_counts.num_enemy = scalar(&before["total"]);
    state.globals.kill_counts.kill_mon = scalar(&before["defeated"]);
    state.globals.in_spell = scalar(&before["ready"]);
    state.globals.round_scripts.blank_round = Some(scalar(&before["inactive"]));
    let presence = words(&before["presence"]);
    state.globals.neighbors = Neighbors {
        departure: scalar(&before["departure"]),
        presence: std::array::from_fn(|index| presence[index * 110..(index + 1) * 110].to_vec()),
        bodies: words(&before["bodies"]),
    };
    state.globals.party_track = words(&before["mask"]).try_into().unwrap();
    state.globals.spell_info = bytes(&before["packet"]);
    state.globals.cast_identity = words(&before["identity"]).try_into().unwrap();
    state.globals.attack_count = scalar(&before["attack_count"]);
    let item = bytes(&before["item_work"]);
    let weapon = bytes(&before["weapon_work"]);
    let ammo = bytes(&before["ammo_work"]);
    state.globals.loaded_item_field32 = word(&item, 32).unwrap();
    state.globals.weapon_traits = WeaponTraits {
        natural_attack_index: scalar(&before["natural_index"]),
        sound: word(&weapon, 14).unwrap(),
        weapon_armor_filter: word(&weapon, 136).unwrap(),
        ammunition_armor_filter: word(&ammo, 136).unwrap(),
    };
    state.globals.loaded_items = crate::ammunition::LoadedItems {
        item: Some(item),
        weapon: Some(weapon),
        ammo: Some(ammo),
    };
    state.globals.health_context = crate::story_health::HealthContext {
        kill_party: scalar(&before["health_unavailable"]) as i8,
        char_select_new: scalar(&before["health_selected"]) as usize,
        effect: state.globals.spell_info[18] as i8,
        last_body: scalar(&before["health_last_body"]),
        marks_affected: scalar(&before["health_marks"]) != 0,
        affected: words(&before["health_affected"]).try_into().unwrap(),
    };
    state.party_condition = bytes(&before["buffs"]);
    state.random = RandomState::new(input["seed"].as_u64().unwrap() as u32);
    state.random.draws = input["draws"].as_u64().unwrap();
    state
}

pub(super) fn compare(state: &Session, events: &[Event], case: &Value) {
    let excluded = |name: &str| {
        case["comparison"]["excluded_record_owners"]
            .as_array()
            .is_some_and(|owners| owners.iter().any(|owner| owner == name))
    };
    let work = &state.globals.battlefield;
    let health = &state.globals.health_context;
    let neighbors = &state.globals.neighbors;
    let buffers = &state.globals.loaded_items;
    let backing = &state.globals.monsters;
    let records = [
        (
            "warriors",
            state
                .warriors
                .iter()
                .flat_map(|record| record.bytes.iter().copied())
                .collect(),
        ),
        (
            "npcs",
            backing
                .slots
                .iter()
                .flat_map(|slot| slot.as_ref().unwrap().record.bytes.iter().copied())
                .collect(),
        ),
        ("map", encoded(work.field.tiles.iter().copied())),
        (
            "party_positions",
            encoded(work.pos.iter().flat_map(|point| [point.0, point.1])),
        ),
        (
            "npc_positions",
            encoded(backing.slots.iter().flat_map(|slot| {
                let point = slot.as_ref().unwrap().position.unwrap();
                [
                    (point.0 - backing.origin.0) as i16,
                    (point.1 - backing.origin.1) as i16,
                ]
            })),
        ),
        ("under", encoded(work.char_under)),
        ("fields", work.spell_queue.clone()),
        ("used", work.collide_check.clone()),
        ("packet", state.globals.spell_info.clone()),
        ("item_work", buffers.item.as_ref().unwrap().clone()),
        ("weapon_work", buffers.weapon.as_ref().unwrap().clone()),
        ("ammo_work", buffers.ammo.as_ref().unwrap().clone()),
        (
            "presence",
            encoded(
                neighbors
                    .presence
                    .iter()
                    .flat_map(|row| row.iter().copied()),
            ),
        ),
        ("bodies", encoded(neighbors.bodies.iter().copied())),
        ("mask", encoded(state.globals.party_track)),
        ("identity", encoded(state.globals.cast_identity)),
        ("health_affected", encoded(health.affected)),
        (
            "order",
            encoded(state.globals.battle_order.order.iter().copied()),
        ),
        (
            "direction",
            state
                .exploration
                .as_ref()
                .map(|exploration| exploration.last_move.map(|value| value as u8).to_vec())
                .unwrap_or_else(|| {
                    assert!(excluded("direction"), "missing represented direction owner");
                    vec![]
                }),
        ),
        ("buffs", state.party_condition.clone()),
    ];
    for (name, actual) in records {
        if excluded(name) {
            continue;
        }
        let expected = bytes(&case["memory"][name]["hex"]);
        assert!(
            actual == expected,
            "{} {name}: first byte {:?}",
            case["name"],
            actual.iter().zip(&expected).position(|(a, b)| a != b)
        );
    }
    for (name, actual) in [
        ("item_field32", state.globals.loaded_item_field32),
        ("weapon_sound", state.globals.weapon_traits.sound),
        (
            "weapon_filter",
            state.globals.weapon_traits.weapon_armor_filter,
        ),
        (
            "ammo_filter",
            state.globals.weapon_traits.ammunition_armor_filter,
        ),
        (
            "natural_index",
            state.globals.weapon_traits.natural_attack_index,
        ),
        ("inactive", state.globals.round_scripts.blank_round.unwrap()),
        ("health_unavailable", i16::from(health.kill_party)),
        ("health_effect", i16::from(health.effect)),
        ("health_last_body", health.last_body),
        ("health_marks", i16::from(health.marks_affected)),
        ("health_selected", health.char_select_new as i16),
        ("selected_warrior", state.char_select_new as i16),
        ("selected_enemy", state.globals.monster_up as i16),
        ("ready", state.globals.in_spell),
        ("total", state.globals.kill_counts.num_enemy),
        ("defeated", state.globals.kill_counts.kill_mon),
        ("attack_count", state.globals.attack_count),
        ("departure", neighbors.departure),
        ("cursor", state.globals.battle_order.index as i16),
    ] {
        if case["comparison"]["excluded_work_owners"]
            .as_array()
            .is_some_and(|owners| owners.iter().any(|owner| owner == name))
        {
            continue;
        }
        assert_eq!(
            actual,
            scalar(&case["memory"][name]["hex"]),
            "{} {name}",
            case["name"]
        );
    }
    assert_eq!(
        state.random.seed,
        case["seed"].as_u64().unwrap() as u32,
        "{} seed",
        case["name"]
    );
    assert_eq!(
        state.random.draws,
        case["draws"].as_u64().unwrap(),
        "{} draws",
        case["name"]
    );
    let sounds: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            Event::Sound { resource_id } => Some(*resource_id),
            Event::Feedback(crate::feedback::GameFeedback::CombatResult {
                code,
                audible: true,
                ..
            }) => crate::feedback::combat_result(*code).map(|(_, sound)| sound),
            _ => None,
        })
        .collect();
    let native: Vec<_> = case["calls"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|call| call["id"] == "sound")
        .map(|call| scalar(&call["arguments"]["id"]))
        .collect();
    assert_eq!(sounds, native, "{} ordered sounds", case["name"]);
    let resources: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            Event::Feedback(crate::feedback::GameFeedback::CombatResourceResult {
                resource_id,
                index,
            }) => Some((*resource_id, *index)),
            _ => None,
        })
        .collect();
    let native: Vec<_> = case["calls"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|call| call["id"] == "GetIndString")
        .filter(|call| {
            !case["comparison"]["excluded_resource_requests"]
                .as_array()
                .is_some_and(|excluded| {
                    excluded.iter().any(|request| {
                        assert_ne!(request["resource"], 134, "condition text is compared");
                        request["instruction"] == call["instruction"]
                            && request["resource"] == scalar(&call["arguments"]["resource"])
                            && request["index"] == scalar(&call["arguments"]["index"])
                    })
                })
        })
        .map(|call| {
            (
                scalar(&call["arguments"]["resource"]),
                scalar(&call["arguments"]["index"]) as usize,
            )
        })
        .collect();
    assert_eq!(resources, native, "{} text resource requests", case["name"]);
}

pub(super) fn saved_return(state: &mut Session, content: &Content) {
    saved_strike_return(state, content, true);
}

/// A checkforenemy frame waiting on its strike's return; false runs the next strike.
fn saved_strike_return(state: &mut Session, content: &Content, returned: bool) {
    static NEXT_SAVE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let caller: CombatReturn = serde_json::from_value(json!({"WarriorMove": {
        "actor":0,"direction":[1,0],"plan":{"Ground":{"tile":1155}},"phase":"AfterStep","cost":2,
        "interrupted":false,"callback_returned":false,"guard": {
            "actor":10,"mode":0,"direction":[1,0],"started":false,"changed":false,"size":0,
            "phase":{"AwaitStrike":returned}
        }
    }}))
    .unwrap();
    state.combat_story = Some(DeathStory {
        return_to: Some(Box::new(caller)),
        direct: true,
        child_returned: true,
        body: 0,
        actor: 0,
        script: 1,
        cursor: 8,
        ..Default::default()
    });
    let position = state.exploration.as_mut().unwrap();
    position.script = Some(ScriptState::scheduled(vec![0; 42], 0));
    position.presentation = Some(Presentation::Dialogue {
        id: 0,
        text: String::new(),
    });
    let path = std::env::temp_dir().join(format!(
        "centurions-detached-attack-{}-{}.json",
        std::process::id(),
        NEXT_SAVE.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    crate::save(&path, state, content).unwrap();
    let restored = crate::load(&path, content).unwrap();
    assert!(*state == restored, "saved reaction return changed state");
    std::fs::remove_file(path).unwrap();
    *state = restored;
}
