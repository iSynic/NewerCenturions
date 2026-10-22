use crate::{prelude::*, support};
use centurions_core::feedback::NativeFeedback;

/// newland mode -1 on constructed records: opcode 104, opcode 127 and values with
/// no switch arm, each followed by sound 5.
#[test]
fn random_battles_monster_test_and_unarmed_opcodes_match_original_newland() {
    let content = support::content();
    for (index, case) in support::fixture("story-trivial.json")
        .array("cases")
        .iter()
        .enumerate()
    {
        let mut state = support::mission_session(&content);
        state.native_work.random_battles = case.n("random_battles_before");
        if case.n::<i16>("opcode") == 127 {
            let mut combat = support::battle();
            let (ids, stamina) = (case.n::<Vec<i16>>("ids"), case.n::<Vec<i16>>("stamina"));
            combat.enemies.records = (0..case.n::<usize>("enemies"))
                .map(|enemy| {
                    let mut bytes = vec![0; 288];
                    bytes[0x103] = ids[enemy] as u8;
                    bytes[0x110..0x112].copy_from_slice(&stamina[enemy].to_be_bytes());
                    MonsterRecord { bytes }
                })
                .collect();
            combat.enemies.positions = vec![(0, 0); combat.enemies.records.len()];
            state.combat = Some(combat);
            state.phase = centurions_core::Phase::Combat;
        }
        let position = state.exploration.as_mut().unwrap();
        position.script = Some(ScriptState::scheduled(case.bytes("record"), 0));
        position.presentation = Some(Presentation::Dialogue {
            id: 0,
            text: String::new(),
        });
        let events = apply(Command::Continue, &mut state, &content)
            .unwrap_or_else(|error| panic!("case {index}: {error:?}"));
        assert_eq!(
            support::event_sounds(&events),
            case.n::<Vec<i16>>("sounds"),
            "case {index} sounds"
        );
        assert_eq!(
            state.native_work.random_battles,
            case.n::<u8>("random_battles"),
            "case {index} BYTE402f42"
        );
    }
}

/// newland mode -1 on constructed records for opcodes 19, 74 and 84 (registered
/// scenario), with the case's extra record at index 6.
#[test]
fn random_string_picked_psi_and_registered_check_match_original_newland() {
    for (index, case) in support::fixture("story-text.json")
        .array("cases")
        .iter()
        .enumerate()
    {
        if case.n::<u8>("registration") != b'b' {
            continue;
        }
        let mut content = support::content();
        let map = &mut content.scenario.as_mut().unwrap().maps[0];
        std::sync::Arc::make_mut(&mut map.shared)
            .combat
            .as_mut()
            .unwrap()
            .encounters[60..70]
            .copy_from_slice(&case.bytes("extra"));
        let mut state = support::mission_session(&content);
        state.warriors = case
            .array("party")
            .iter()
            .map(|warrior| Warrior::read(&support::bytes(warrior)).unwrap())
            .collect();
        state.native_work.party_mask = [1, 1, 1, 0, 0, 0];
        state.random = RandomState::new(case.n("initial_seed"));
        let position = state.exploration.as_mut().unwrap();
        position.script = Some(ScriptState::scheduled(case.bytes("record"), 0));
        position.presentation = Some(Presentation::Dialogue {
            id: 0,
            text: String::new(),
        });
        let (mut sounds, mut texts) = (vec![], vec![]);
        for _ in 0..3 {
            let events = apply(Command::Continue, &mut state, &content)
                .unwrap_or_else(|error| panic!("case {index}: {error:?}"));
            sounds.extend(support::event_sounds(&events));
            match &state.exploration.as_ref().unwrap().presentation {
                Some(Presentation::Dialogue { id, .. }) => texts.push(*id),
                _ => break,
            }
        }
        let psi: Vec<i16> = state
            .warriors
            .iter()
            .map(|w| w.record().psi_points().unwrap())
            .collect();
        assert_eq!(sounds, case.n::<Vec<i16>>("sounds"), "case {index} sounds");
        assert_eq!(texts, case.n::<Vec<i16>>("texts"), "case {index} texts");
        assert_eq!(psi, case.n::<Vec<i16>>("psi_after"), "case {index} Psi");
        support::assert_rng(&state.random, &case["seed"], &case["draws"]);
    }
}

/// buttonchoice's exhausted-move refusal and newland66, as camp-fatigue.json records.
#[test]
fn exhausted_movement_and_camping_flag_match_original() {
    let content = support::content();
    for (index, case) in support::fixture("camp-fatigue.json")
        .array("cases")
        .iter()
        .enumerate()
    {
        let mut state = support::mission_session(&content);
        if case["kind"] == "move" {
            state.exploration.as_mut().unwrap().fatigue = case.n("fatigue");
            state.native_work.camp_disabled = case.n::<u8>("camp_disabled") != 0;
            state.field_controls.get_or_insert_default().camping = case.n::<u8>("camping") != 0;
            let refused = matches!(
                apply(Command::Move { dx: 1, dy: 0 }, &mut state, &content),
                Err(RuleError::Refused { feedback }) if feedback == [
                    NativeFeedback::Warning { index: 54 },
                    NativeFeedback::Tutorial { first: 80, last: 80 },
                ]
            );
            let calls = case["calls"].as_str().unwrap();
            assert_eq!(
                refused,
                calls.starts_with("warn:54"),
                "case {index} {calls}"
            );
            continue;
        }
        state.native_work.camp_disabled = case.n::<u8>("old") != 0;
        let mut record = vec![0; 42];
        record[8..10].copy_from_slice(&66_i16.to_be_bytes());
        record[24..26].copy_from_slice(&case.n::<i16>("parameter").to_be_bytes());
        let position = state.exploration.as_mut().unwrap();
        position.script = Some(ScriptState::scheduled(record, 0));
        position.presentation = Some(Presentation::Dialogue {
            id: 0,
            text: String::new(),
        });
        let events = apply(Command::Continue, &mut state, &content).unwrap();
        let flashes: Vec<String> = events
            .iter()
            .filter_map(|event| match event {
                Event::Feedback(NativeFeedback::Flash {
                    address,
                    x,
                    y,
                    wait,
                    sound,
                }) => Some(format!("{address}:{x}:{y}:{wait}:{sound}")),
                _ => None,
            })
            .collect();
        assert_eq!(
            flashes.join(","),
            case["flashes"].as_str().unwrap(),
            "case {index}"
        );
        assert_eq!(
            i16::from(state.native_work.camp_disabled),
            case.n::<i16>("after")
        );
    }
}

/// newland mode -1 for opcodes 28, 61, 103 and 106 with extra record 6, the party
/// at 40, 40. Boat cases are skipped: the reconstruction never boards.
#[test]
fn redraw_shift_boat_camping_and_light_match_original_newland() {
    for (index, case) in support::fixture("story-map.json")
        .array("cases")
        .iter()
        .enumerate()
    {
        if case.n::<u8>("boat") != 0 {
            continue;
        }
        let mut content = support::content();
        let map = &mut content.scenario.as_mut().unwrap().maps[0];
        std::sync::Arc::make_mut(&mut map.shared)
            .combat
            .as_mut()
            .unwrap()
            .encounters[60..70]
            .copy_from_slice(&case.bytes("extra"));
        let mut state = support::mission_session(&content);
        state.random = RandomState::new(case.n("initial_seed"));
        state.field_controls.get_or_insert_default().camping = case.n::<u8>("camping") != 0;
        let position = state.exploration.as_mut().unwrap();
        (position.x, position.y) = (40, 40);
        position.region_state[522] = case.n("light_before");
        position.script = Some(ScriptState::scheduled(case.bytes("record"), 0));
        position.presentation = Some(Presentation::Dialogue {
            id: 0,
            text: String::new(),
        });
        let events = apply(Command::Continue, &mut state, &content)
            .unwrap_or_else(|error| panic!("case {index}: {error:?}"));
        let position = state.exploration.as_ref().unwrap();
        assert_eq!(
            support::event_sounds(&events),
            case.n::<Vec<i16>>("sounds"),
            "case {index} sounds"
        );
        assert_eq!(
            (position.x, position.y),
            (case.n("x"), case.n("y")),
            "case {index} position"
        );
        assert_eq!(
            position.region_state[522],
            case.n::<u8>("light"),
            "case {index} light"
        );
        support::assert_rng(&state.random, &case["seed"], &case["draws"]);
    }
}

/// newland mode -1 for opcodes 57, 70 and 92 over two constructed region tables, and
/// 57 with the original loadland and loadpixmap (story-tile-set.json) from either land:
/// the current land's tile properties follow its live tile-set byte. The land a probe
/// could not load (57's word-pushed target) and a closing loadpixmap that quits are
/// refused instead.
#[test]
fn land_light_saved_position_and_rectangle_edits_match_original_newland() {
    for (name, index, case) in ["story-land.json", "story-tile-set.json"]
        .into_iter()
        .flat_map(|name| {
            let fixture = support::fixture(name);
            let cases = fixture.array("cases").to_vec();
            cases
                .into_iter()
                .enumerate()
                .map(move |(i, case)| (name, i, case))
        })
    {
        let context = format!("{name} case {index}");
        let mut content = support::content();
        let maps = &mut content.scenario.as_mut().unwrap().maps;
        let mut tables = (*maps[0].shared).clone();
        let extras = &mut tables.combat.as_mut().unwrap().encounters;
        extras[60..70].copy_from_slice(&case.bytes("extra6"));
        if case.get("extra7").is_some() {
            extras[70..80].copy_from_slice(&case.bytes("extra7"));
        }
        // Extra records belong to the level, not to one land.
        let tables = std::sync::Arc::new(tables);
        for map in maps.iter_mut() {
            map.shared = tables.clone();
        }
        let mut state = support::mission_session(&content);
        let lands: Vec<Vec<u8>> = case
            .array("lands_before")
            .iter()
            .map(support::bytes)
            .collect();
        let current = case["current"].as_i64().unwrap_or(0) as i32;
        let land = |index: i32| {
            let mut records = centurions_core::MapState::authored(content.map(index).unwrap());
            records.regions = lands[index as usize].clone();
            records
        };
        state.visited_maps.insert(1 - current, land(1 - current));
        if case.get("saved_before").is_some() {
            state.saved_position = case.n::<Vec<u8>>("saved_before").try_into().unwrap();
        }
        let position = state.exploration.as_mut().unwrap();
        position.restore_map(current, land(current));
        (position.x, position.y) = (40, 40);
        position.script = Some(ScriptState::scheduled(case.bytes("record"), 0));
        position.presentation = Some(Presentation::Dialogue {
            id: 0,
            text: String::new(),
        });
        let result = apply(Command::Continue, &mut state, &content);
        if case["undefined"] == true || case["fatal"] == true {
            assert!(
                matches!(result, Err(RuleError::Unsupported { .. })),
                "{context} is refused: {result:?}"
            );
            continue;
        }
        let events = result.unwrap_or_else(|error| panic!("{context}: {error:?}"));
        let position = state.exploration.as_ref().unwrap();
        let after = |land: i32| match position.map == land {
            true => position.region_state.clone(),
            false => state.visited_maps[&land].regions.clone(),
        };
        let expected: Vec<Vec<u8>> = case
            .array("lands_after")
            .iter()
            .map(support::bytes)
            .collect();
        assert_eq!(
            [after(0), after(1)],
            [expected[0].clone(), expected[1].clone()],
            "{context} lands"
        );
        assert_eq!(position.map, case.n::<i32>("map"), "{context}");
        if case.get("x").is_some() {
            assert_eq!(
                (position.x, position.y),
                (case.n("x"), case.n("y")),
                "{context}"
            );
            assert_eq!(
                state.saved_position.to_vec(),
                case.n::<Vec<u8>>("saved_after"),
                "{context}"
            );
        }
        if case.get("tile_properties").is_some() {
            let live = state.land(&content).unwrap();
            assert_eq!(
                live.terrain,
                case.n::<u8>("loaded_set"),
                "{context} tile set"
            );
            support::assert_bytes(
                &live.tile_properties,
                &case.bytes("tile_properties"),
                &context,
            );
            assert_eq!(
                (live.default_tile, live.minutes_per_tick),
                (
                    case.n::<i16>("default_tile"),
                    if case.n::<i16>("tick_flag") == 0 {
                        5
                    } else {
                        1
                    }
                ),
                "{context} default tile and tick"
            );
        }
        assert_eq!(
            support::event_sounds(&events),
            case.n::<Vec<i16>>("sounds"),
            "{context} sounds"
        );
    }
}

/// newland mode -1 for the "Branch on" opcodes 40, 64, 67, 72, 75 and 81, with
/// door records 30 and 31 announcing which extra action point ran.
#[test]
fn branch_opcodes_take_the_original_extra_action_point() {
    let fixture = support::fixture("story-branch.json");
    for (index, case) in fixture.array("cases").iter().enumerate() {
        let mut content = support::content();
        let map = &mut content.scenario.as_mut().unwrap().maps[0];
        let shared = std::sync::Arc::make_mut(&mut map.shared);
        shared.combat.as_mut().unwrap().encounters[60..70].copy_from_slice(&case.bytes("extra"));
        for (door, record) in fixture.array("doors").iter().enumerate() {
            let offset = (30 + door) * 42;
            shared.door_scripts[offset..offset + 42].copy_from_slice(&support::bytes(record));
        }
        let mut state = support::mission_session(&content);
        state.warriors = case
            .array("party")
            .iter()
            .map(|warrior| Warrior::read(&support::bytes(warrior)).unwrap())
            .collect();
        let mask = case.n::<i16>("mask");
        state.native_work.party_mask = std::array::from_fn(|m| mask >> m & 1);
        state.flags[10..13].fill(1);
        state.party_buffs[8..10].copy_from_slice(&case.n::<i16>("buff4").to_be_bytes());
        let position = state.exploration.as_mut().unwrap();
        (position.day, position.hour) = (2, 10);
        position.script = Some(ScriptState::scheduled(case.bytes("record"), 0));
        position.presentation = Some(Presentation::Dialogue {
            id: 0,
            text: String::new(),
        });
        let events = apply(Command::Continue, &mut state, &content)
            .unwrap_or_else(|error| panic!("case {index}: {error:?}"));
        assert_eq!(
            support::event_sounds(&events),
            case.n::<Vec<i16>>("sounds"),
            "case {index}"
        );
    }
}

/// newland mode -1 for opcodes 20, 34, 55 and 86. Opcode 20's destination event sits
/// in header slot 5 of an otherwise empty table; opcode 34's caller frame resuming
/// past the last action is refused.
#[test]
fn chaining_opcodes_match_original_newland() {
    let fixture = support::fixture("story-chain.json");
    for (index, case) in fixture.array("cases").iter().enumerate() {
        let mut content = support::content();
        let map = &mut content.scenario.as_mut().unwrap().maps[0];
        let shared = std::sync::Arc::make_mut(&mut map.shared);
        shared.combat.as_mut().unwrap().encounters[60..70].copy_from_slice(&case.bytes("extra"));
        for (door, record) in fixture.array("doors").iter().enumerate() {
            let offset = (30 + door) * 42;
            shared.door_scripts[offset..offset + 42].copy_from_slice(&support::bytes(record));
        }
        let mut state = support::mission_session(&content);
        state.random = RandomState::new(case.n("initial_seed"));
        state.warriors = case
            .array("party")
            .iter()
            .map(|warrior| Warrior::read(&support::bytes(warrior)).unwrap())
            .collect();
        let mask = case.n::<i16>("mask");
        state.native_work.party_mask = std::array::from_fn(|m| mask >> m & 1);
        state.field_controls.get_or_insert_default().camping = case.n::<u8>("camping") != 0;
        let mut other = centurions_core::MapState::authored(content.map(1).unwrap());
        let position = state.exploration.as_mut().unwrap();
        (position.x, position.y) = (40, 40);
        let mut script = ScriptState::scheduled(case.bytes("record"), 0);
        let opcode = case.n::<i16>("opcode");
        if opcode == 20 {
            let extra = case.bytes("extra");
            let word = |i: usize| i16::from_be_bytes([extra[i], extra[i + 1]]);
            let at = |value: i16| if value < 0 { 40 } else { value as usize };
            let destination = at(word(2)) * 90 + at(word(4));
            let mut event = case.bytes("event");
            event.resize(42, 0);
            let (headers, floor) = if word(0) == 1 {
                (&mut other.event_headers, &mut other.floor)
            } else {
                (&mut position.event_headers, &mut position.floor)
            };
            *headers = vec![0; 4200];
            headers[210..252].copy_from_slice(&event);
            floor[destination] = case.n("floor");
        }
        if opcode == 34 {
            script.menu = Some(centurions_core::MenuContinuation {
                id: 0,
                remaining: 1,
                original: fixture.bytes("caller"),
                typed: false,
                eliminated: 0,
            });
            script.caller_action = Some(case.n("caller_action"));
        }
        position.script = Some(script);
        position.presentation = Some(Presentation::Dialogue {
            id: 0,
            text: String::new(),
        });
        state.visited_maps.insert(1, other);
        let result = apply(Command::Continue, &mut state, &content);
        if opcode == 34 && case.n::<i16>("caller_action") == 7 {
            assert!(result.is_err(), "case {index} is refused");
            continue;
        }
        let events = result.unwrap_or_else(|error| panic!("case {index}: {error:?}"));
        let position = state.exploration.as_ref().unwrap();
        assert_eq!(
            (position.map, position.x, position.y),
            (case.n("map"), case.n("x"), case.n("y")),
            "case {index}"
        );
        assert_eq!(
            support::event_sounds(&events),
            case.n::<Vec<i16>>("sounds"),
            "case {index}"
        );
        support::assert_rng(&state.random, &case["seed"], &case["draws"]);
    }
}

/// newland mode -1 for opcodes 43, 51, 65, 71, 91 and 121. Opcode 65 is compared at
/// booty's entry: its items in order and the count left in the caller local.
#[test]
fn party_shop_treasure_and_battle_opcodes_match_original_newland() {
    let fixture = support::fixture("story-effects.json");
    for (index, case) in fixture.array("cases").iter().enumerate() {
        let mut content = support::content();
        let map = &mut content.scenario.as_mut().unwrap().maps[0];
        std::sync::Arc::make_mut(&mut map.shared)
            .combat
            .as_mut()
            .unwrap()
            .encounters[60..70]
            .copy_from_slice(&case.bytes("extra"));
        let mut state = support::mission_session(&content);
        state.random = RandomState::new(case.n("initial_seed"));
        state.warriors = case
            .array("party")
            .iter()
            .map(|warrior| Warrior::read(&support::bytes(warrior)).unwrap())
            .collect();
        let mask = case.n::<i16>("mask");
        state.native_work.party_mask = std::array::from_fn(|m| mask >> m & 1);
        let mut shops = content.map(0).unwrap().shops.clone();
        shops[3002..6004].copy_from_slice(&fixture.bytes("shop"));
        state.shops.insert(state.scenario_level, shops);
        if case.n::<i16>("battle") != 0 {
            let mut combat = support::battle();
            combat.enemies.records = case
                .array("npcs")
                .iter()
                .map(|npc| MonsterRecord {
                    bytes: support::bytes(npc),
                })
                .collect();
            combat.enemies.positions = vec![(0, 0); combat.enemies.records.len()];
            combat.squad.positions.truncate(state.warriors.len());
            combat.squad.under_tiles.truncate(state.warriors.len());
            state.combat = Some(combat);
            state.phase = centurions_core::Phase::Combat;
        }
        let position = state.exploration.as_mut().unwrap();
        position.script = Some(ScriptState::scheduled(case.bytes("record"), 0));
        position.presentation = Some(Presentation::Dialogue {
            id: 0,
            text: String::new(),
        });
        let events = apply(Command::Continue, &mut state, &content)
            .unwrap_or_else(|error| panic!("case {index}: {error:?}"));
        let opcode = case.n::<i16>("opcode");
        if opcode == 65 {
            let booty = case.bytes("booty");
            let mut items: Vec<i16> = booty[..40]
                .as_chunks::<2>()
                .0
                .iter()
                .map(|word| i16::from_be_bytes(*word))
                .filter(|&id| id > 0)
                .collect();
            let mut shown: Vec<i16> = state
                .booty_items
                .iter()
                .map(|item| item.id())
                .filter(|&id| id != 0)
                .collect();
            // The treasure screen orders the prepared items by ID.
            items.sort();
            shown.sort();
            assert_eq!(shown, items, "case {index} items");
            let script = state.exploration.as_ref().unwrap().script.as_ref().unwrap();
            assert_eq!(
                script.caller_action,
                Some(case.n("caller_action")),
                "case {index}"
            );
            continue;
        }
        let sounds: Vec<i16> = case
            .n::<Vec<i16>>("sounds")
            .into_iter()
            .filter(|&s| s != 0)
            .collect();
        assert_eq!(
            support::event_sounds(&events),
            sounds,
            "case {index} sounds"
        );
        let calls: Vec<String> = events
            .iter()
            .filter_map(|event| match event {
                Event::Feedback(NativeFeedback::CombatResult { body, code, .. }) => {
                    Some(format!("result {body} {code}"))
                }
                Event::CombatantRemoved { body } => Some(format!("destroy {}", body - 10)),
                _ => None,
            })
            .collect();
        let expected: Vec<&str> = case
            .array("calls")
            .iter()
            .map(|call| call.as_str().unwrap())
            .filter(|call| !call.starts_with("effect"))
            .collect();
        assert_eq!(calls, expected, "case {index} calls");
        for (member, after) in case.array("party_after").iter().enumerate() {
            let (actual, expected) = (&state.warriors[member].bytes, support::bytes(after));
            let differences: Vec<_> = (0..expected.len())
                .filter(|&i| actual[i] != expected[i])
                .map(|i| (i, actual[i], expected[i]))
                .collect();
            assert!(
                differences.is_empty(),
                "case {index} member {member}: {differences:?}"
            );
        }
        if opcode == 51 {
            assert_eq!(
                state.shops[&state.scenario_level][3002..6004],
                case.bytes("shop_after"),
                "case {index} shop"
            );
        }
        assert_eq!(
            state.native_work.coordinates_hidden,
            case.n::<i16>("coordinates_hidden"),
            "case {index}"
        );
        support::assert_rng(&state.random, &case["seed"], &case["draws"]);
    }
}

/// newland mode -1 for opcodes 50, 53, 58, 59, 60, 63 and 90, with the pick mask,
/// members, difficulty, floor word and time as the case sets them. Opcode 63 is
/// compared at timeclick's entry; a time outside the clock is refused.
#[test]
fn selection_difficulty_time_and_points_opcodes_match_original_newland() {
    let fixture = support::fixture("story-selection.json");
    for (index, case) in fixture.array("cases").iter().enumerate() {
        let mut content = support::content();
        let map = &mut content.scenario.as_mut().unwrap().maps[0];
        let shared = std::sync::Arc::make_mut(&mut map.shared);
        shared.combat.as_mut().unwrap().encounters[60..70].copy_from_slice(&case.bytes("extra"));
        for (door, record) in fixture.array("doors").iter().enumerate() {
            let offset = (30 + door) * 42;
            shared.door_scripts[offset..offset + 42].copy_from_slice(&support::bytes(record));
        }
        let mut state = support::mission_session(&content);
        state.warriors = case
            .array("party")
            .iter()
            .map(|warrior| Warrior::read(&support::bytes(warrior)).unwrap())
            .collect();
        let mask = case.n::<i16>("mask");
        state.native_work.party_mask = std::array::from_fn(|m| mask >> m & 1);
        state.difficulty = case.n("difficulty");
        let position = state.exploration.as_mut().unwrap();
        (position.x, position.y) = (40, 40);
        position.floor[40 * 90 + 40] = case.n("floor");
        let time = case.n::<Vec<i16>>("time");
        (position.day, position.hour, position.minute) = (time[0], time[1], time[2]);
        position.script = Some(ScriptState::scheduled(case.bytes("record"), 0));
        position.presentation = Some(Presentation::Dialogue {
            id: 0,
            text: String::new(),
        });
        let result = apply(Command::Continue, &mut state, &content);
        let clock = case.array("calls").first().map(|call| {
            let fields: Vec<i16> = call
                .as_str()
                .unwrap()
                .split(' ')
                .skip(1)
                .map(|f| f.parse().unwrap())
                .collect();
            fields
        });
        let outside =
            |f: &Vec<i16>| !(0..24).contains(&f[3]) || !(0..60).contains(&f[4]) || f[2] < 1;
        if clock.as_ref().is_some_and(outside) {
            assert!(result.is_err(), "case {index} is refused");
            continue;
        }
        let events = result.unwrap_or_else(|error| panic!("case {index}: {error:?}"));
        assert_eq!(
            support::event_sounds(&events),
            case.n::<Vec<i16>>("sounds"),
            "case {index} sounds"
        );
        assert_eq!(
            state.native_work.party_mask.to_vec(),
            case.n::<Vec<i16>>("mask_after"),
            "case {index} mask"
        );
        for (member, after) in case.array("party_after").iter().enumerate() {
            assert_eq!(
                state.warriors[member].bytes,
                support::bytes(after),
                "case {index} member {member}"
            );
        }
        if let Some(fields) = clock {
            let position = state.exploration.as_ref().unwrap();
            assert_eq!(
                [position.day, position.hour, position.minute],
                [fields[2], fields[3], fields[4]],
                "case {index} time"
            );
        }
    }
}

/// newland mode -1 for opcode 27, which draws the scenario PICT, and opcode 41 over a
/// constructed two-record simple-encounter table.
#[test]
fn picture_and_encounter_choice_opcodes_match_original_newland() {
    let fixture = support::fixture("story-picture.json");
    for (index, case) in fixture.array("cases").iter().enumerate() {
        let mut content = support::content();
        let map = &mut content.scenario.as_mut().unwrap().maps[0];
        let shared = std::sync::Arc::make_mut(&mut map.shared);
        shared.combat.as_mut().unwrap().encounters[60..70].copy_from_slice(&case.bytes("extra"));
        shared.menus = fixture.bytes("encounters");
        let mut state = support::mission_session(&content);
        let position = state.exploration.as_mut().unwrap();
        position.script = Some(ScriptState::scheduled(case.bytes("record"), 0));
        position.presentation = Some(Presentation::Dialogue {
            id: 0,
            text: String::new(),
        });
        let events = apply(Command::Continue, &mut state, &content)
            .unwrap_or_else(|error| panic!("case {index}: {error:?}"));
        assert_eq!(
            support::event_sounds(&events),
            case.n::<Vec<i16>>("sounds"),
            "case {index}"
        );
        let drawn: Vec<i16> = events
            .iter()
            .filter_map(|event| match event {
                Event::PictureDrawn { resource_id } => Some(*resource_id),
                _ => None,
            })
            .collect();
        let pictures: Vec<i16> = case
            .array("calls")
            .iter()
            .filter_map(|call| call.as_str().unwrap().strip_prefix("GetPicture "))
            .map(|id| id.parse().unwrap())
            .collect();
        assert_eq!(drawn, pictures, "case {index} pictures");
        if case.n::<i16>("opcode") == 41 {
            assert_eq!(
                state.menus,
                case.bytes("encounters_after"),
                "case {index} encounters"
            );
        }
    }
}

/// newland mode -1 for opcodes 14 and -14 with scripted portrait clicks through
/// getchoice, and opcode 44 inside a running complex encounter, which keeps its
/// elimination until CancelMenu writes it to the encounter table.
#[test]
fn party_picks_and_complex_branch_elimination_match_original_newland() {
    let fixture = support::fixture("story-picks.json");
    for (index, case) in fixture.array("cases").iter().enumerate() {
        let mut content = support::content();
        let buffer = fixture.bytes("buffer");
        let map = &mut content.scenario.as_mut().unwrap().maps[0];
        std::sync::Arc::make_mut(&mut map.shared).typed_encounters[..156].copy_from_slice(&buffer);
        let mut state = support::mission_session(&content);
        state.warriors = case
            .array("party")
            .iter()
            .map(|warrior| Warrior::read(&support::bytes(warrior)).unwrap())
            .collect();
        state.native_work.party_mask = [1; 6];
        let typed = case.n::<i16>("opcode") == 44;
        let position = state.exploration.as_mut().unwrap();
        let mut script = ScriptState::scheduled(case.bytes("record"), 0);
        if typed {
            script.menu = Some(centurions_core::MenuContinuation {
                id: 0,
                remaining: 5,
                original: case.bytes("record"),
                typed: true,
                eliminated: 0,
            });
        }
        position.script = Some(script);
        position.presentation = Some(Presentation::Dialogue {
            id: 0,
            text: String::new(),
        });
        let mut sounds =
            support::event_sounds(&apply(Command::Continue, &mut state, &content).unwrap());
        for click in case.n::<Vec<usize>>("clicks") {
            match apply(
                Command::ChooseWarrior { index: click },
                &mut state,
                &content,
            ) {
                Ok(events) => sounds.extend(support::event_sounds(&events)),
                Err(RuleError::Refused { feedback }) => {
                    sounds.extend(feedback.iter().filter_map(|f| match f {
                        NativeFeedback::Sound { resource_id } => Some(*resource_id),
                        _ => None,
                    }))
                }
                Err(error) => panic!("case {index}: {error:?}"),
            }
        }
        assert_eq!(sounds, case.n::<Vec<i16>>("sounds"), "case {index} sounds");
        if typed {
            apply(Command::CancelMenu, &mut state, &content)
                .unwrap_or_else(|error| panic!("case {index}: {error:?}"));
            assert_eq!(
                state.typed_encounters[..156],
                case.bytes("buffer_after"),
                "case {index} buffer"
            );
            continue;
        }
        assert_eq!(
            state.native_work.party_mask.to_vec(),
            case.n::<Vec<i16>>("mask_after"),
            "case {index} mask"
        );
        assert_eq!(
            state.selected_warrior,
            case.n::<usize>("selected"),
            "case {index} selected"
        );
    }
}
