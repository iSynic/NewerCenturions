//! Source callback/save joins around independently executed native Getup.
use crate::{prelude::*, support};
use centurions_core::combat::globals::{BattleOrder, Monsters, MonsterSlot};
use centurions_core::combat::runtime::{CombatReturn, DeathStory};
use centurions_core::combat::battlefield::Battlefield;

#[test]
fn detached_getup_matches_native_records_work_and_saved_script_return() {
    let actors = support::fixture("retained-getup.json");
    let queues = support::fixture("retained-getup-queue.json");
    let rounds = support::fixture("retained-getup-round.json");
    let fields = support::fixture("retained-getup-field.json");
    for case in actors
        .array("cases")
        .iter()
        .chain(queues.array("cases"))
        .chain(rounds.array("cases"))
        .chain(fields.array("cases"))
    {
        let mut content = content();
        let mut state = input(case, &mut content);
        if case.text("kind") == "field-npc" {
            let mut lost = state.clone();
            let mut slot = lost.globals.monsters.slots[0]
                .as_ref()
                .unwrap()
                .clone();
            slot.record.record_mut().set_stamina(1);
            let point = slot.position.unwrap();
            lost.globals.monsters.slots[55] = Some(slot);
            lost.globals.monster_up = 55;
            lost.globals.battle_order.order[0] = 65;
            lost.globals
                .battlefield.field
                .tiles[(point.0 * 90 + point.1) as usize] = 65;
            let before = lost.clone();
            assert!(matches!(
                apply(Command::Continue, &mut lost, &content),
                Err(RuleError::Unsupported {
                    action: "combat.outdoor_health",
                    dependency: "outdoor death stamina alias beyond native A5 allocation"
                })
            ));
            assert!(lost == before, "late detached field refusal changed state");
        }
        if case.text("kind") == "queue-mixed" {
            let mut lost = state.clone();
            lost.globals
                .health_context
                .kill_party = 4;
            let events = support::saved_command(Command::Continue, &mut lost, &content);
            assert!(lost.warriors.is_empty() && lost.exploration.is_none());
            assert_eq!(
                events
                    .iter()
                    .filter(|event| matches!(event, Event::MissionEnded { defeated: true }))
                    .count(),
                1
            );
        }
        if case.text("kind") == "round-reduce" {
            let mut collision = support::checkpoint(&state, &content, "retained-field-refusal");
            let field = &mut collision
                .globals
                .battlefield
                .spell_queue[..8];
            field.copy_from_slice(&[0, 34, 0, 34, 0, 36, 0, 36]);
            let before = collision.clone();
            assert!(matches!(
                apply(Command::Continue, &mut collision, &content),
                Err(RuleError::Invalid(message)) if message == "spell descriptor"
            ));
            assert!(
                collision == before,
                "late retained round refusal changed state"
            );
        }
        let mut events = support::saved_command(
            Command::WithPreferences {
                command: Box::new(Command::Continue),
                preferences: centurions_core::CommandPreferences {
                    animation_delay: 25,
                    ..Default::default()
                },
            },
            &mut state,
            &content,
        );
        if case.text("kind") == "script" {
            assert!(state.turn_dispatch.is_some() && state.combat.is_none());
            assert_eq!(state.random.draws, 1);
            events.extend(support::saved_command(
                Command::Continue,
                &mut state,
                &content,
            ));
        }
        if matches!(case.text("kind"), "round-script" | "round-replace") {
            assert!(
                state.combat.is_none()
                    && state
                        .round_dispatch
                        .as_ref()
                        .is_some_and(|frame| frame.out_of_combat)
            );
            assert_eq!(state.globals.round_scripts.processing, Some(1));
            if case.text("kind") == "round-script" {
                let mut replaced = state.clone();
                replaced.round_dispatch.as_mut().unwrap().child_battle = true;
                replaced.globals.round_scripts.counter = Some(23);
                replaced.globals.round_scripts.slots = Some([7, 8, 9, 10, 11]);
                replaced.globals.round_scripts.slot = Some(5);
                let expected = replaced.globals.round_scripts.clone();
                support::saved_command(Command::Continue, &mut replaced, &content);
                assert_eq!(replaced.globals.round_scripts.counter, expected.counter);
                assert_eq!(replaced.globals.round_scripts.slots, expected.slots);
                assert_eq!(replaced.globals.round_scripts.slot, expected.slot);
                assert_eq!(replaced.globals.round_scripts.processing, Some(0));
                assert!(replaced.round_dispatch.is_none());
            }
            events.extend(support::saved_command(
                Command::Continue,
                &mut state,
                &content,
            ));
        }
        same(
            &support::record_bytes(&state.warriors),
            &case.bytes("warriors_after"),
            "warriors",
        );
        let npcs: Vec<_> = state
            .globals
            .monsters
            .slots
            .iter()
            .flat_map(|slot| slot.as_ref().unwrap().record.bytes.iter().copied())
            .collect();
        same(&npcs, &case.bytes("npcs_after"), "NPCs");
        if case.get("rounds_after").is_some() {
            assert_eq!(
                state.globals.health_context.effect,
                case.n::<i8>("health_effect")
            );
            let round = &state.globals.round_scripts;
            assert_eq!(round.counter, Some(case.n("counter")));
            assert_eq!(
                round.slots,
                Some(support::words(&case["rounds_after"]).try_into().unwrap())
            );
            assert_eq!(round.slot, Some(case.n("round_slot")));
            assert_eq!(round.processing, Some(case.n("round_flag")));
            assert_eq!(round.blank_round, Some(case.n("inactive")));
            same(&state.holdover, &case.bytes("allies_after"), "allies");
            same(&state.party_condition, &case.bytes("buffs_after"), "buffs");
            same(
                &state.globals.spell_info,
                &case.bytes("packet_after"),
                "packet",
            );
        }
        if case.get("attack").is_some() {
            assert_eq!(
                state.globals.cast_identity,
                <[i16; 3]>::try_from(support::words(&case["identity"])).unwrap()
            );
            assert_eq!(
                state.globals.attack_count,
                case.n::<i16>("attack_count")
            );
            assert_eq!(
                state.globals.party_track,
                support::words(&case["mask"]).as_slice()
            );
            assert_eq!(
                state.globals.health_context.affected,
                support::words(&case["affected"]).as_slice()
            );
            assert_eq!(
                state.globals.health_context.last_body,
                case.n::<i16>("last_body")
            );
        }
        let work = &state.globals.battlefield;
        if case.get("queue_after").is_some() {
            let queue = state.globals.outer_todo_queue.as_ref().unwrap();
            let bytes: Vec<_> = queue
                .entries
                .iter()
                .flat_map(|entry| entry.bytes())
                .collect();
            same(&bytes, &case.bytes("queue_after"), "queue");
            assert_eq!(queue.count, case.n::<usize>("queue_count"));
            assert_eq!(
                state.globals.health_context.last_body,
                case.n::<i16>("last_body")
            );
        }
        same(
            &support::word_bytes(&work.field.tiles),
            &case.bytes("map_after"),
            "map",
        );
        same(&work.spell_queue, &case.bytes("fields_after"), "fields");
        same(&work.collide_check, &case.bytes("used_after"), "usage");
        assert_eq!(
            state.globals.battle_order.index,
            case.n::<usize>("cursor")
        );
        assert_eq!(state.char_select_new, case.n::<usize>("selected_warrior"));
        assert_eq!(
            state.globals.monster_up,
            case.n::<usize>("selected_enemy")
        );
        assert_eq!(
            state.globals.macro_monster,
            case.n::<usize>("script_enemy")
        );
        assert_eq!(
            state
                .globals
                .health_context
                .kill_party,
            case.n::<i8>("unavailable")
        );
        assert_eq!(
            state
                .globals
                .outer_todo_queue
                .as_ref()
                .unwrap()
                .pointer,
            case.n::<i32>("queue_pointer")
        );
        assert_eq!(
            state.globals.death_processing.doing_queue,
            case.n::<u8>("queue_flag")
        );
        assert_eq!(state.globals.in_spell, case.n::<i16>("ready"));
        assert_eq!(state.globals.kill_counts.num_enemy, case.n::<i16>("total"));
        assert_eq!(
            state.globals.kill_counts.kill_mon,
            case.n::<i16>("defeated")
        );
        assert_eq!(state.flags[..100], case.bytes("flags"));
        support::assert_rng(&state.random, &case["seed"], &case["draws"]);
        assert!(
            state.combat.is_none()
                && state.combat_story.is_none()
                && state.turn_dispatch.is_none()
                && state.round_dispatch.is_none()
        );
        let observed: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                Event::Sound { resource_id } => Some(format!("sound:{resource_id}")),
                Event::Feedback(centurions_core::feedback::GameFeedback::CombatResult {
                    body,
                    code,
                    ..
                }) => Some(format!("result:{body}:{code}")),
                Event::Feedback(centurions_core::feedback::GameFeedback::OutdoorBodyStatus {
                    permanent,
                    wait,
                }) => {
                    assert_eq!(*wait, 25);
                    Some(format!("status:{permanent}"))
                }
                Event::Feedback(centurions_core::feedback::GameFeedback::Flash {
                    address,
                    x,
                    y,
                    wait,
                    sound,
                }) => Some(format!("flash:{address}:{x}:{y}:{wait}:{sound}")),
                _ => None,
            })
            .collect();
        let expected: Vec<_> = case
            .array("calls")
            .iter()
            .filter_map(|call| {
                let value = call.as_str().unwrap();
                (value.starts_with("sound:")
                    || value.starts_with("result:")
                    || value.starts_with("status:")
                    || value.starts_with("flash:"))
                .then_some(value.to_owned())
            })
            .collect();
        assert_eq!(observed, expected);
    }
}

fn input(case: &Value, content: &mut Content) -> Session {
    let tables = Arc::make_mut(&mut content.scenario.as_mut().unwrap().maps[0].shared);
    tables.door_scripts.resize(903 * 42, 0);
    tables.door_scripts[900 * 42..901 * 42].copy_from_slice(&case.bytes("script"));
    if case.get("replacement").is_some() {
        tables.door_scripts[902 * 42..903 * 42].copy_from_slice(&case.bytes("replacement"));
        let combat = tables.combat.as_mut().unwrap();
        combat.encounters.resize(901 * 10, 0);
        combat.encounters[900 * 10..901 * 10].copy_from_slice(&case.bytes("extra"));
    }
    if case.get("attack").is_some() {
        let id = case.n::<usize>("spell_id");
        let offset = (id / 1000 - 1) * 2842 + (id % 1000 / 100 - 1) * 406 + (id % 100 - 1) * 58;
        tables.combat.as_mut().unwrap().attacks[offset..offset + 58]
            .copy_from_slice(&case.bytes("attack"));
        let areas = case.bytes("area");
        tables.areas[..areas.len()].copy_from_slice(&areas);
    }
    let mut state = support::mission_session(content);
    state.warriors = support::warriors(&case["warriors_before"]);
    let coordinates = support::words(&case["positions"]);
    state.globals.monsters = Monsters {
        num_mon: 4,
        origin: (5, 5),
        slots: support::monsters(&case["npcs_before"])
            .into_iter()
            .enumerate()
            .map(|(index, record)| {
                Some(MonsterSlot {
                    record,
                    position: Some((
                        i32::from(coordinates[index * 2]) + 5,
                        i32::from(coordinates[index * 2 + 1]) + 5,
                    )),
                })
            })
            .collect(),
    };
    state.globals.battlefield = Battlefield {
        field: CombatField {
            tiles: support::words(&case["map_before"]),
            map_shift: (0, 0),
            edge_word: 0,
        },
        saved_land: None,
        pos: [(30, 30); 6],
        char_under: [1155; 6],
        spell_queue: case.bytes("fields_before"),
        collide_check: case.bytes("used_before"),
        battle: None,
    };
    state.globals.battle_order = BattleOrder {
        order: support::words(&case["order"]),
        index: 0,
    };
    state.globals.turn_round = Some(0);
    state.globals.in_spell = 7;
    state.globals.kill_counts.num_enemy = 5;
    state.globals.kill_counts.kill_mon = 0;
    state.globals.outer_todo_queue = Some(TodoQueue {
        pointer: 0x12340044,
        ..Default::default()
    });
    if case.get("queue_before").is_some() {
        let queue = state.globals.outer_todo_queue.as_mut().unwrap();
        queue.count = case.n("queue_count_before");
        queue.entries = case
            .bytes("queue_before")
            .as_chunks::<14>()
            .0
            .iter()
            .map(|entry| centurions_core::combat::death_scripts::DeathEntry {
                monster: i16::from_be_bytes(entry[..2].try_into().unwrap()),
                faction: i16::from_be_bytes(entry[2..4].try_into().unwrap()),
                script: i16::from_be_bytes(entry[4..6].try_into().unwrap()),
                x: i32::from_be_bytes(entry[6..10].try_into().unwrap()),
                y: i32::from_be_bytes(entry[10..14].try_into().unwrap()),
            })
            .collect();
        state.globals.health_context.last_body = case.n("last_body_before");
    }
    state.random = RandomState::new(case.n("seed_before"));
    if case.get("rounds_before").is_some() {
        state.globals.round_scripts = centurions_core::combat::round_scripts::RoundScripts {
            counter: Some(case.n("counter_before")),
            slots: Some(support::words(&case["rounds_before"]).try_into().unwrap()),
            slot: Some(4),
            processing: Some(0),
            blank_round: Some(case.n("inactive_before")),
        };
        state.globals.battle_order.index = 109;
        state
            .globals
            .health_context
            .kill_party = case.n("unavailable_before");
        state.holdover = case.bytes("allies_before");
        state.party_condition = case.bytes("buffs_before");
        state.globals.spell_info = case.bytes("packet_before");
    }
    if case.get("attack").is_some() {
        state.char_select_new = case.n("selected_before");
        state
            .globals
            .health_context
            .char_select_new = state.char_select_new;
        state.globals.weapon_traits = WeaponTraits {
            sound: case.n("weapon_sound"),
            ..WeaponTraits::zero()
        };
        let positions = support::words(&case["party_positions"]);
        state.globals.battlefield.pos =
            std::array::from_fn(|i| (positions[i * 2], positions[i * 2 + 1]));
    }
    let returned: CombatReturn = serde_json::from_value(serde_json::json!({ "WarriorMove": {
    "actor":0,"direction":[1,0],"plan":{"Ground":{"tile":1155}},"phase":"AfterDeparture","cost":1,
    "interrupted":false,"callback_returned":false,"guard": {
        "actor":0,"mode":2,"direction":[1,0],"started":true,"changed":false,"size":0,
        "phase":{"Guards":{"actor":0,"departure":true,"count":1,"index":0,"changed":false,"interrupted":false,"phase":"AwaitAttack","result":true}}
    }
}})).unwrap();
    state.combat_story = Some(DeathStory {
        return_to: Some(Box::new(returned)),
        direct: true,
        child_returned: true,
        body: 0,
        actor: 0,
        script: 901,
        cursor: 8,
        ..Default::default()
    });
    state.flags.fill(0);
    support::pause_script(&mut state, vec![0; 42], 0);
    state
}

fn same(actual: &[u8], expected: &[u8], owner: &str) {
    assert!(
        actual == expected,
        "{owner}: first difference {:?}",
        actual.iter().zip(expected).position(|(a, b)| a != b)
    );
}
