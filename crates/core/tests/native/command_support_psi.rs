use crate::{native_common, prelude::*, snapshots, support};
use native_common::decoded_positions as positions;
use support::{checkpoint, same_session};

// Items12b1ee returns past the older12b194 snapshots through updatecharinfo.
// Compose those static loader effects without treating them as native captures.
fn item_info_return(state: &mut Session, content: &Content, owner: usize) {
    let record = state.warriors[owner].record();
    let firearm = record.displayed_weapon(true).unwrap();
    let ammunition = record.loaded_ammunition().unwrap();
    let melee = record.displayed_weapon(false).unwrap();
    let context = content
        .context(state.mission.as_deref(), state.scenario_level)
        .unwrap();
    if firearm != 0 {
        state.native_work.ammunition_work.item = Some(context.items[firearm as usize].clone());
        if ammunition != 0 {
            let definition = context.items[ammunition as usize].clone();
            state.native_work.weapon_work.ammunition_armor_filter =
                word(&definition, 136).unwrap();
            state.native_work.ammunition_work.ammo = Some(definition);
        }
    }
    if melee != 0 {
        state.native_work.ammunition_work.item = Some(context.items[melee as usize].clone());
    }
    state.native_work.loaded_item_field32 =
        word(state.native_work.ammunition_work.item.as_ref().unwrap(), 32).unwrap();
    state.native_work.ready = 0;
}

pub(crate) fn outdoor(content: &Content, case: &Value) -> Session {
    let mut state = support::mission_session(content);
    if let Some(encounter) = case.get("encounter") {
        let mut record = vec![0; 42];
        record[8..10].copy_from_slice(&5_i16.to_be_bytes());
        record[24..26].copy_from_slice(&(encounter.as_i64().unwrap() as i16).to_be_bytes());
        let position = state.exploration.as_mut().unwrap();
        position.script = Some(centurions_core::ScriptState::scheduled(record, 0));
        position.presentation = Some(Presentation::Dialogue {
            id: 0,
            text: String::new(),
        });
        apply(Command::Continue, &mut state, content).unwrap();
    }
    state.warriors = support::warriors(&case["before"]["party"]);
    if let Some(count) = case.get("party_count") {
        state.warriors.truncate(count.as_u64().unwrap() as usize);
        state.allies = case["before"].bytes("allies");
        state.ally_count = case.n("ally_count");
    }
    state.native_work.psi_memory = case["before"].bytes("memory");
    state.party_buffs = case["before"].bytes("party_buffs");
    if let Some(directory) = case["before"].get("scenario_directory") {
        state.scenario_duration_tail = Some(word(&support::bytes(directory), 50).unwrap());
    }
    if let Some(tail) = case["before"].get("scenario_duration_tail") {
        state.scenario_duration_tail = Some(tail.as_i64().unwrap() as i16);
    }
    state.native_work.attack_packet = case["before"].bytes("packet");
    state.native_work.attack_identity = case["before"].words("identity").try_into().unwrap();
    state.native_work.attack_count = case["before"].n::<i16>("count");
    if let Some(unavailable) = case["before"].get("unavailable") {
        let mut health = centurions_core::story_health::context(&state.warriors, 0, None);
        health.unavailable = unavailable.as_i64().unwrap() as i8;
        state.native_work.health_context = health;
    }
    state.random = if case.get("item").is_some() {
        crate::native_common::random_at(&case["before"], "seed", "draws")
    } else {
        RandomState::new(case.n::<u32>("seed_before"))
    };
    state
}

#[test]
fn typed_loaded_firearm_selects_party_before_returning_the_item() {
    // Controlled caller regression; the medical fixture supplies the party,
    // not an independent native receipt for firearm activation.
    let content = content();
    let mut case = support::fixture("encounter-medical.json")["cases"][0].clone();
    case["encounter"] = 1.into();
    let mut completions = vec![];
    for quantity in [0, 2] {
        let mut state = outdoor(&content, &case);
        let slot = state.warriors[0]
            .inventory()
            .unwrap()
            .iter()
            .position(|i| i.id() == 1)
            .unwrap();
        state.warriors[0]
            .record_mut()
            .set_inventory_quantity(slot, quantity);
        let mut events =
            support::saved_command(Command::UseItem { warrior: 0, slot }, &mut state, &content);
        assert_eq!(state.outdoor_psi.is_some(), quantity > 0);
        if quantity > 0 {
            assert_eq!(state.outdoor_psi.as_ref().unwrap().spell, 4101);
            events.extend(support::saved_command(
                Command::SelectOutdoorPsiTarget { warrior: 0 },
                &mut state,
                &content,
            ));
            assert!(events.iter().any(|e| matches!(
                e,
                Event::SpellResolved {
                    spell: 4101,
                    target: 0,
                    damage: 0,
                    ..
                }
            )));
        }
        assert!(state.outdoor_psi.is_none());
        assert_eq!(
            state.warriors[0]
                .inventory()
                .unwrap()
                .iter()
                .find(|i| i.id() == 1)
                .unwrap()
                .quantity()
                .unwrap(),
            (quantity - 1).max(0)
        );
        completions.push(state.exploration);
    }
    assert_eq!(completions[0], completions[1]);
}

#[test]
fn outdoor_shield_matches_complete_native_records_and_exact_session_across_reload() {
    let content = content();
    let fixture = support::fixture("player-support-psi.json");
    let outdoors = support::fixture("outdoor-psi.json");
    let indexed = support::fixture("outdoor-indexed-psi.json");
    for case in fixture
        .array("cases")
        .iter()
        .filter(|case| case["spell_id"] == 1103)
        .chain(outdoors.array("cases"))
        .chain(indexed.array("cases"))
    {
        let mut state = outdoor(&content, case);
        let before = state.clone();
        let mut resumed = checkpoint(&state, &content, "support-shield-before");
        let command = Command::CastPartyPsi {
            warrior: 5,
            spell: case.n("spell_id"),
            power: case.n::<i16>("power"),
        };
        let events = replay_command(command, &mut state, &mut resumed, &content);
        let expected = &case["after"];
        let mut expected_session = before;
        snapshots::captured_party(&mut expected_session, case, expected);
        expected_session.native_work.psi_memory = expected.bytes("memory");
        expected_session.party_buffs = expected.bytes("party_buffs");
        if let Some(directory) = expected.get("scenario_directory") {
            expected_session.scenario_duration_tail =
                Some(word(&support::bytes(directory), 50).unwrap());
        }
        expected_session.native_work.attack_packet = expected.bytes("packet");
        let identity = expected.bytes("identity");
        expected_session.native_work.attack_identity =
            expected.words("identity").try_into().unwrap();
        expected_session.native_work.attack_count = expected.n::<i16>("count");
        expected_session
            .native_work
            .party_mask
            .copy_from_slice(&expected.words("mask")[..6]);
        expected_session.psi_menu_choices[5] = Some(centurions_core::PsiMenuChoice {
            row: centurions_content::word(&identity, 2).unwrap() as u8,
            column: centurions_content::word(&identity, 4).unwrap() as u8,
            power: expected_session.native_work.attack_count,
        });
        expected_session.random.seed = expected.n::<u32>("seed");
        expected_session.random.draws = expected.n::<u64>("draws");
        support::same_captured_session(&state, &expected_session);
        let native_events: Vec<_> = support::sound_events(case.n::<Vec<i16>>("sounds"));
        assert_eq!(events, native_events);
        checkpoint(&state, &content, "support-shield-after");
    }
    let damage = support::fixture("outdoor-party-psi.json");
    let targeted = support::fixture("outdoor-targets.json");
    let small = support::fixture("outdoor-small-party.json");
    let items = support::fixture("outdoor-medical.json");
    let encounter_items = support::fixture("encounter-medical.json");
    let indexed_items = support::fixture("outdoor-indexed-items.json");
    let revival = support::fixture("outdoor-revival.json");
    let areas = support::fixture("outdoor-area-party.json");
    let restore_life = support::fixture("outdoor-restore-life.json");
    for case in damage
        .array("cases")
        .iter()
        .chain(targeted.array("cases"))
        .chain(small.array("cases"))
        .chain(items.array("cases"))
        .chain(encounter_items.array("cases"))
        .chain(indexed_items.array("cases"))
        .chain(revival.array("cases"))
        .chain(areas.array("cases"))
        .chain(restore_life.array("cases"))
    {
        let mut state = outdoor(&content, case);
        let before = state.clone();
        let mut resumed = checkpoint(&state, &content, "support-outdoor-before");
        let command = if case.get("item").is_some() {
            Command::UseItem {
                warrior: case.n("actor"),
                slot: case.n("slot"),
            }
        } else {
            Command::CastPartyPsi {
                warrior: case.n("caster"),
                spell: case.n("spell_id"),
                power: case.n("power"),
            }
        };
        let mut events = replay_command(command, &mut state, &mut resumed, &content);
        let item_click = serde_json::json!([case["target"]]);
        if let Some(clicks) = case.get("clicks").or_else(|| {
            case.get("item")
                .filter(|_| case["global"] != true)
                .map(|_| &item_click)
        }) {
            for click in clicks.as_array().unwrap() {
                resumed = checkpoint(&state, &content, "support-selection");
                let command = if click.as_i64().unwrap() < 0 {
                    Command::CancelOutdoorPsi
                } else {
                    Command::SelectOutdoorPsiTarget {
                        warrior: click.as_u64().unwrap() as usize,
                    }
                };
                let next = replay_command(command, &mut state, &mut resumed, &content);
                events.extend(next);
            }
        }
        let expected = &case["after"];
        let mut caller = before.clone();
        if case.get("item").is_some() {
            let owner = case.n::<usize>("actor");
            caller.native_work.ammunition_work.item = Some(expected.bytes("item_work"));
            caller.native_work.character_bare_hand_bonus =
                before.warriors[owner].record().bare_hand_bonus().unwrap();
            let health = &mut caller.native_work.health_context;
            health.selected_warrior = owner;
            health.effect = expected.bytes("packet")[18] as i8;
            if case.get("encounter").is_none() {
                item_info_return(&mut caller, &content, owner);
            }
            assert_eq!(state.selected_warrior, before.selected_warrior);
        }
        if case["global"] == true {
            let mut complete = caller.clone();
            snapshots::captured_party(&mut complete, case, expected);
            complete.native_work.attack_packet = expected.bytes("packet");
            complete.native_work.attack_identity =
                expected.words("identity").try_into().unwrap();
            complete.native_work.attack_count = expected.n("count");
            complete
                .native_work
                .party_mask
                .copy_from_slice(&expected.words("mask")[..6]);
            complete.scenario_duration_tail = Some(expected.n("scenario_duration_tail"));
            // buttonchoice 10ac9a's cleartarget; the receipt omits the triples.
            complete.native_work.target_work.cell = (-1, -1);
            complete.random = native_common::random_state(expected);
            same_session(&state, &complete);
        }
        support::compare_record_bytes(
            &state.warriors,
            &support::warriors(&expected["party"])[..state.warriors.len()],
        );
        if expected.get("allies").is_some() {
            let original = expected.bytes("allies");
            assert!(
                state.allies == original,
                "ally bytes differ for spell{} party{} power{} at {:?}",
                case["spell_id"],
                case["party_count"],
                case["power"],
                state.allies.iter().zip(&original).position(|(a, b)| a != b)
            );
        }
        assert!(state.outdoor_psi.is_none());
        if case.get("encounter").is_some() {
            let centurions_core::typed_encounter::TypedContinuation::Branch { body, .. } =
                centurions_core::typed_encounter::continuation(
                    &case.bytes("record"),
                    case.n("selector"),
                    99,
                )
                .unwrap()
            else {
                panic!("native encounter item branch");
            };
            let position = state.exploration.as_ref().unwrap();
            assert_eq!(&position.script.as_ref().unwrap().record[8..40], body);
            assert!(matches!(
                position.presentation,
                Some(Presentation::Dialogue { .. })
            ));
            assert!(events.contains(&Event::StoryPresented));
        }
        if [1604, 3605, 1705, 3705]
            .iter()
            .any(|id| case["spell_id"] == *id)
        {
            assert_eq!(
                state
                    .native_work
                    .health_context
                    .unavailable,
                expected.n::<i8>("unavailable")
            );
        }
        assert_eq!(
            state.native_work.psi_memory,
            expected.bytes("memory")
        );
        assert_eq!(state.random.seed, expected.n::<u32>("seed"));
        assert_eq!(state.random.draws, expected.n::<u64>("draws"));
        assert_eq!(
            state.native_work.party_mask.as_slice(),
            &expected.words("mask")[..6]
        );
        support::native_expect! {
            bytes state.native_work.attack_packet => &expected["packet"], "outdoor packet";
            words state.native_work.attack_identity => &expected["identity"], "outdoor identity";
            scalar state.native_work.attack_count => &expected["count"], "outdoor count";
        }
        if case.get("item").is_some() {
            assert_eq!(
                state.native_work.ammunition_work.item, caller.native_work.ammunition_work.item,
                "source Items return info loader"
            );
            if case.get("encounter").is_none() {
                assert_eq!(
                    state.native_work.ammunition_work.ammo,
                    caller.native_work.ammunition_work.ammo
                );
            }
        }
        assert_eq!(support::event_sounds(&events), case.n::<Vec<i16>>("sounds"));
        checkpoint(&state, &content, "support-outdoor-after");
    }
    for (case_index, case) in support::fixture("encounter-spells.json")
        .array("cases")
        .iter()
        .enumerate()
    {
        let mut state = outdoor(&content, case);
        let record = case.bytes("record");
        state.typed_encounters = content.map(0).unwrap().typed_encounters.clone();
        state.typed_encounters[10 * 516..11 * 516].copy_from_slice(&record);
        let mut script = vec![0; 42];
        script[8..10].copy_from_slice(&5_i16.to_be_bytes());
        script[24..26].copy_from_slice(&10_i16.to_be_bytes());
        let position = state.exploration.as_mut().unwrap();
        position.script = Some(centurions_core::ScriptState::scheduled(script, 0));
        position.presentation = Some(Presentation::Dialogue {
            id: 0,
            text: String::new(),
        });
        apply(Command::Continue, &mut state, &content).unwrap();
        if case_index == 0 {
            for native in support::fixture("encounter-detox.json").array("cases") {
                let mut indexed = state.clone();
                indexed.warriors = support::warriors(&native["warriors_before"]);
                indexed.native_work.psi_memory = vec![0; 480];
                indexed.scenario_duration_tail = Some(-1);
                indexed.random = RandomState::new(12);
                let events = support::saved_command(
                    Command::CastEncounterPsi {
                        warrior: 1,
                        spell: native.n("spell"),
                        power: 1,
                    },
                    &mut indexed,
                    &content,
                );
                support::native_expect! {
                    records &indexed.warriors => &native["warriors_after"], "encounter Detox party";
                    bytes indexed.native_work.psi_memory => &native["memory_after"], "encounter Detox memory";
                    scalar indexed.scenario_duration_tail.unwrap() => &native["scenario_duration_after"], "encounter Detox alias";
                    scalar indexed.random.seed => &native["seed_after"], "encounter Detox seed";
                    scalar indexed.random.draws => &native["draws"], "encounter Detox draws";
                }
                support::expect_array(&support::event_sounds(&events), &native["sounds"]);
                assert!(matches!(
                    indexed.exploration.as_ref().unwrap().presentation,
                    Some(Presentation::Typed { id: 10, .. })
                ));
                checkpoint(&indexed, &content, "encounter-indexed-after");
            }
        }
        let events = support::saved_command(
            Command::CastEncounterPsi {
                warrior: case.n("caster"),
                spell: case.n("spell_id"),
                power: case.n("power"),
            },
            &mut state,
            &content,
        );
        let expected = &case["after"];
        support::native_expect! {
            bytes &support::record_bytes(&state.warriors) => &expected.bytes("party"), "encounter spell party";
        };
        assert_eq!(
            state.native_work.psi_memory,
            expected.bytes("memory")
        );
        assert_eq!(state.party_buffs, expected.bytes("party_buffs"));
        assert_eq!(
            state.native_work.party_mask.as_slice(),
            &expected.words("mask")[..6]
        );
        assert_eq!(
            state.native_work.attack_packet,
            expected.bytes("packet")
        );
        assert_eq!(
            state.native_work.attack_identity.as_slice(),
            expected.words("identity")
        );
        assert_eq!(state.native_work.attack_count, expected.n::<i16>("count"));
        assert_eq!(state.random, native_common::random_state(expected));
        let mut sounds = case.n::<Vec<i16>>("sounds");
        let branch = case.n::<usize>("selector") - 1;
        for slot in 0..8 {
            let opcode = record[branch * 8 + slot];
            if opcode == 1 {
                break;
            }
            if opcode == 9 {
                sounds.push(word(&record, 32 + branch * 16 + slot * 2).unwrap());
            }
        }
        assert_eq!(support::event_sounds(&events), sounds);
        let position = state.exploration.as_ref().unwrap();
        if case.n::<i8>("selector") == 1 {
            assert!(matches!(
                position.presentation,
                Some(Presentation::Dialogue { id: 1123, .. })
            ));
        } else {
            assert!(matches!(
                position.presentation,
                Some(Presentation::Typed { id: 10, .. })
            ));
            assert_eq!(
                position
                    .script
                    .as_ref()
                    .unwrap()
                    .menu
                    .as_ref()
                    .unwrap()
                    .remaining,
                98
            );
        }
        checkpoint(&state, &content, "encounter-spell-after");
    }
    for case in support::fixture("outdoor-type11.json").array("cases") {
        let mut state = support::mission_session(&content);
        state.warriors = support::warriors(&case["warriors_before"]);
        state.native_work.psi_memory = vec![0; 480];
        state.random = RandomState::new(12);
        let position = state.exploration.clone();
        let events = support::saved_command(
            Command::CastPartyPsi {
                warrior: 1,
                spell: case.n("spell"),
                power: 1,
            },
            &mut state,
            &content,
        );
        assert_eq!(state.exploration, position);
        support::native_expect! {
            records &state.warriors => &case["warriors_after"], "type11 party";
            bytes state.native_work.psi_memory => &case["memory_after"], "type11 learned memory";
            scalar state.random.draws => &case["draws"], "type11 RNG";
        }
        assert_eq!(
            state.native_work.party_mask.as_slice(),
            &case.words("mask_after")[..6]
        );
        support::expect_array(&support::event_sounds(&events), &case["sounds"]);
        checkpoint(&state, &content, "outdoor-type11-after");
    }
}

#[test]
fn outdoor_invalid_cost_conditions_and_pending_story_roll_back_the_whole_session() {
    let content = content();
    let fixture = support::fixture("player-support-psi.json");
    let case = fixture
        .array("cases")
        .iter()
        .find(|c| c["spell_id"] == 1103 && c["variant"] == 0)
        .unwrap();
    for command in [
        Command::CastPartyPsi {
            warrior: 6,
            spell: 1103,
            power: 1,
        },
        Command::CastPartyPsi {
            warrior: 5,
            spell: 2404,
            power: 1,
        },
    ] {
        let mut state = outdoor(&content, case);
        let before = state.clone();
        assert!(apply(command.clone(), &mut state, &content).is_err());
        assert_eq!(state, before, "{command:?}");
    }
    let command = Command::CastPartyPsi {
        warrior: 5,
        spell: 1103,
        power: 1,
    };
    let mut state = outdoor(&content, case);
    state.warriors[5].bytes[0x58..0x5a].fill(0);
    let before = state.clone();
    assert!(apply(command.clone(), &mut state, &content).is_err());
    assert_eq!(state, before);
    let mut state = outdoor(&content, case);
    state.exploration.as_mut().unwrap().presentation = Some(centurions_core::Presentation::Loot);
    let before = state.clone();
    assert!(apply(command, &mut state, &content).is_err());
    assert_eq!(state, before);
}

pub(crate) fn combat_state(content: &Content, case: &Value) -> Session {
    let mut state = outdoor(content, case);
    let foundation: Value = support::fixture("player-psi.json");
    let mut combat: centurions_core::combat::Combat =
        serde_json::from_value(foundation["route_foundation"]["combat"].clone()).unwrap();
    combat.map.shift = (0, 0);
    combat.map.tiles = case["before"].words("map");
    combat.squad.positions = positions(&case["before"]["party_positions"]);
    combat.squad.under_tiles = vec![1155; 6];
    combat.enemies.records = support::monsters(&case["before"].bytes("records"));
    combat.enemies.positions = positions(&case["before"]["monster_positions"]);
    combat.order = case["before"].words("order");
    combat.cursor = case["before"].n::<usize>("cursor");
    combat.round = case["before"].n::<i16>("round");
    combat.unavailable = case["before"].n::<i8>("unavailable");
    combat.spell_fields = case["before"].bytes("fields");
    combat.field_used = case["before"].bytes("used");
    combat.attack_packet = state.native_work.attack_packet.clone();
    combat.attack_identity = state.native_work.attack_identity;
    combat.weapon_work = centurions_core::combat::weapon_work::WeaponWork::zero();
    if case["before"].get("weapon_work").is_some() {
        combat
            .weapon_work
            .load_weapon(&case["before"].bytes("weapon_work"))
            .unwrap();
        combat
            .weapon_work
            .load_ammunition(&case["before"].bytes("ammo_work"))
            .unwrap();
    }
    state.native_work.weapon_work = combat.weapon_work;
    state.combat = Some(combat);
    state.phase = centurions_core::Phase::Combat;
    crate::native_common::source_victory_context(&mut state);
    state.native_work.loaded_item_field32 = case["before"].n::<i16>("item_context");
    state
}

#[test]
fn combat_support_and_automatic_initiative_match_complete_native_suffix_and_resume() {
    let content = content();
    let fixture: Value = support::fixture("support-psi-advance.json");
    let npcs = support::fixture("support-npc.json");
    let armor = support::fixture("armor-support.json");
    let advanced = support::fixture("advanced-psi-advance.json");
    let wards = support::fixture("ward-psi.json");
    let healing = support::fixture("npc-healing.json");
    let conditions = support::fixture("condition-psi.json");
    let additional = support::fixture("additional-condition-psi.json");
    let superior = support::fixture("healing-psi.json");
    let learned_heal = support::fixture("learned-healing-psi.json");
    let learned_damage = support::fixture("learned-damage-psi.json");
    let unresisted = support::fixture("unresisted-psi.json");
    let resisted = support::fixture("resisted-conditions.json");
    let multiple = support::fixture("multi-psi.json");
    for case in fixture
        .array("cases")
        .iter()
        .chain(npcs.array("cases"))
        .chain(armor.array("cases"))
        .chain(advanced.array("cases"))
        .chain(wards.array("cases"))
        .chain(healing.array("cases"))
        .chain(conditions.array("cases"))
        .chain(additional.array("cases"))
        .chain(superior.array("cases"))
        .chain(learned_heal.array("cases"))
        .chain(learned_damage.array("cases"))
        .chain(unresisted.array("cases"))
        .chain(resisted.array("cases"))
        .chain(multiple.array("cases"))
    {
        let mut state = combat_state(&content, case);
        let before = state.clone();
        let mut resumed = checkpoint(&state, &content, "support-combat-before");
        let spell = case.n::<i16>("spell_id");
        let command = Command::CastPsi {
            rotation: case.get("rotation").and_then(Value::as_u64).unwrap_or(0) as u8,
            spell,
            power: case.n("power"),
            target: if case["before"].bytes("packet")[1] == 7
                || case["before"].bytes("packet")[1] >= 9
            {
                None
            } else {
                Some((20, 20))
            },
        };
        if let Some(gate) = unresisted["mode_gates"]
            .as_array()
            .and_then(|gates| gates.iter().find(|g| g["spell_id"] == spell))
        {
            assert!(!gate.n::<bool>("mode_allowed"));
            support::reject_command(&state, &content, command);
            continue;
        }
        let events = if let Some(bodies) = case.get("selected_bodies") {
            let mut events = vec![];
            let mut commands = vec![Command::BeginPsiSelection {
                spell,
                power: case.n("power"),
            }];
            for body in bodies.as_array().unwrap() {
                let point = current_combat(&state).squad.positions[body.as_u64().unwrap() as usize];
                commands.push(Command::SelectPsiTarget {
                    x: point.0,
                    y: point.1,
                });
            }
            commands.push(Command::FirePsiSelection);
            for command in commands {
                let result = replay_command(command, &mut state, &mut resumed, &content);
                events.extend(result);
                resumed = checkpoint(&resumed, &content, "support-selected");
            }
            let mut cancelled = before.clone();
            apply(
                Command::BeginPsiSelection {
                    spell,
                    power: case.n("power"),
                },
                &mut cancelled,
                &content,
            )
            .unwrap();
            support::reject_command(&cancelled, &content, Command::FirePsiSelection);
            apply(Command::CancelPsiSelection, &mut cancelled, &content).unwrap();
            assert_eq!(
                digest(&support::record_bytes(&cancelled.warriors)),
                digest(&case.bytes("cancelled_party"))
            );
            assert_eq!(cancelled.random, before.random);
            assert!(cancelled.psi_selection.is_none());
            events
        } else {
            let events = apply(command.clone(), &mut state, &content)
                .unwrap_or_else(|error| panic!("spell {spell}: {error}"));
            assert_eq!(apply(command, &mut resumed, &content).unwrap(), events);
            events
        };
        assert!(state == resumed, "save/resume state differs");
        let expected = &case["after"];
        let mut expected_session = before;
        snapshots::captured_party(&mut expected_session, case, expected);
        let arena = expected_session.combat.as_mut().unwrap();
        snapshots::records(arena, &|key| expected.bytes(key));
        arena.map.tiles = expected.words("map");
        arena.squad.positions = positions(&expected["party_positions"]);
        arena.enemies.positions = positions(&expected["monster_positions"]);
        arena.order = expected.words("order");
        arena.cursor = expected.n::<usize>("cursor");
        arena.round = expected.n::<i16>("round");
        arena.unavailable = expected.n::<i8>("unavailable");
        arena.spell_fields = expected.bytes("fields");
        arena.field_used = expected.bytes("used");
        arena.attack_packet = expected.bytes("packet");
        arena.attack_identity = expected.words("identity").try_into().unwrap();
        expected_session.native_work.attack_packet = arena.attack_packet.clone();
        expected_session.native_work.attack_identity = arena.attack_identity;
        expected_session.native_work.attack_count = expected.n::<i16>("count");
        let identity = expected.words("identity");
        let actor = case["before"].n::<usize>("active_member");
        expected_session.psi_menu_choices[actor] = Some(centurions_core::PsiMenuChoice {
            row: identity[1] as u8,
            column: identity[2] as u8,
            power: expected_session.native_work.attack_count,
        });
        expected_session.native_work.loaded_item_field32 = expected.n::<i16>("item_context");
        expected_session.native_work.psi_memory = expected.bytes("memory");
        expected_session.party_buffs = expected.bytes("party_buffs");
        let mut health = centurions_core::story_health::context(
            &expected_session.warriors,
            expected_session.selected_warrior,
            Some(&expected_session.native_work.health_context),
        );
        health.unavailable = arena.unavailable;
        health.effect = expected.bytes("packet")[18] as i8;
        expected_session.native_work.health_context = health;
        expected_session.bandage_ready = 1;
        expected_session.random.seed = expected.n::<u32>("seed");
        expected_session.random.draws = expected.n::<u64>("draws");
        support::same_captured_session(&state, &expected_session);
        let sounds: Vec<_> = support::event_sounds(&events);
        support::expect_array(&(sounds), &case["sounds"]);
        assert!(matches!(
            events.last(),
            Some(Event::TurnStarted { body: 10, round: 1 })
        ));
        checkpoint(&state, &content, "support-combat-after");
    }
    for blocked in [false, true] {
        let case = fixture
            .array("cases")
            .iter()
            .find(|c| c["spell_id"] == 1305)
            .unwrap();
        let mut state = combat_state(&content, case);
        if blocked {
            state.combat.as_mut().unwrap().map.tiles[25 * 90 + 28] = 1000;
        }
        let before = state.clone();
        let error = apply(
            Command::CastPsi {
                rotation: 0,
                spell: 1305,
                power: 1,
                target: Some(if blocked { (20, 20) } else { (40, 25) }),
            },
            &mut state,
            &content,
        )
        .unwrap_err();
        assert_eq!(state, before);
        assert!(
            matches!(error, centurions_core::RuleError::Refused { feedback } if feedback == vec![centurions_core::feedback::NativeFeedback::Combat { kind:if blocked {centurions_core::feedback::CombatRefusal::Blocked} else {centurions_core::feedback::CombatRefusal::OutOfRange} }])
        );
    }
}

#[test]
fn player_area_cast_reads_land_objects_past_the_right_map_edge() {
    // spelltargets indexes column × 90 + row; column 90 is objects40c500 + 2 * row.
    let content = content();
    let fixture = support::fixture("support-npc.json");
    let case = fixture
        .array("cases")
        .iter()
        .find(|c| c["spell_id"] == 1305)
        .unwrap();
    let mut state = combat_state(&content, case);
    let combat = state.combat.as_mut().unwrap();
    let tiles = combat.map.tiles.clone();
    for (index, tile) in tiles.into_iter().enumerate() {
        combat.map.tiles[(index + 62 * 90) % 8100] = tile;
    }
    combat.map.shift.0 = 62;
    for marker in [0, 7] {
        let mut state = state.clone();
        state.exploration.as_mut().unwrap().objects[..90].fill(marker);
        let command = Command::CastPsi {
            rotation: 0,
            spell: 1305,
            power: case.n("power"),
            target: Some((20, 20)),
        };
        // A party marker 6..9 there is the validation's empty support body.
        match apply(command, &mut state, &content) {
            Err(centurions_core::RuleError::Invalid(reason)) => {
                assert_eq!((marker, reason.as_str()), (7, "empty support body"));
            }
            result => assert!(marker == 0 && result.is_ok(), "{result:?}"),
        }
    }
}

/// Warrior 5 knows `spell` and the support-npc NPC is moved to `body`.
fn npc_body_state(content: &Content, body: usize, spell: i16) -> (Session, (i16, i16)) {
    let fixture = support::fixture("support-npc.json");
    let mut state = combat_state(content, &fixture["cases"][0]);
    let (category, tier, column) = (spell / 1000, spell % 1000 / 100 - 1, spell % 100 - 1);
    state.warriors[5].bytes[0x36..0x38].copy_from_slice(&category.to_be_bytes());
    state.warriors[5].bytes[0x9f0 + (tier * 7 + column) as usize] = 1;
    state.warriors[5].record_mut().set_action_points(6);
    state.native_work.ammunition_work.item = Some(content.items[0].clone());
    let combat = state.combat.as_mut().unwrap();
    let living = combat.enemies.records[0].clone();
    let mut dead = living.clone();
    dead.record_mut().set_stamina(0);
    combat.enemies.records = vec![dead; body - 9];
    combat.enemies.records[body - 10] = living;
    let target = combat.enemies.positions[0];
    combat.enemies.positions.resize(body - 9, target);
    for tile in &mut combat.map.tiles {
        if tile.wrapping_abs() == 10 {
            *tile = if *tile < 0 {
                -(body as i16)
            } else {
                body as i16
            };
        }
    }
    for entry in &mut combat.order {
        if *entry == 10 {
            *entry = body as i16;
        }
    }
    (state, target)
}

#[test]
fn npc_revival_writes_its_warrior_indexed_item_work_alias() {
    // Body 14 indexes 0x448aa0 + 14 * 3138 = ITEM work 0x45363c; spelllist 0x15b9d8
    // effect 64 writes WORD 0x90 = 0, then WORD 0x50 = -9 there.
    let content = content();
    let (mut state, target) = npc_body_state(&content, 14, 1705);
    let item = state.native_work.ammunition_work.item.as_mut().unwrap();
    item[0x50..0x52].copy_from_slice(&(-15_i16).to_be_bytes());
    item[0x90..0x92].copy_from_slice(&3_i16.to_be_bytes());
    let npc = state.combat.as_ref().unwrap().enemies.records[4].clone();
    let events = support::saved_command(
        Command::CastPsi {
            spell: 1705,
            power: 1,
            target: Some(target),
            rotation: 0,
        },
        &mut state,
        &content,
    );
    assert!(events.iter().any(|event| matches!(event, Event::SpellResolved { spell: 1705, target: 14, .. })));
    let item = state.native_work.ammunition_work.item.as_ref().unwrap();
    assert_eq!(item[0x50..0x52], (-9_i16).to_be_bytes());
    assert_eq!(item[0x90..0x92], [0, 0]);
    assert_eq!(state.combat.as_ref().unwrap().enemies.records[4], npc);
}

#[test]
fn restricted_player_cast_uses_retained_item_and_refuses_unrepresented_body_memory() {
    // Controlled body-index regression, separate from a played positioning route.
    let content = content();
    for body in [10_usize, 14, 15, 24] {
        let (mut state, target) = npc_body_state(&content, body, 2206);
        let command = Command::CastPsi {
            spell: 2206,
            power: 1,
            target: Some(target),
            rotation: 0,
        };
        if body == 24 {
            let before = state.clone();
            let error = apply(command, &mut state, &content).unwrap_err();
            assert_eq!(state, before);
            assert!(matches!(
                error,
                centurions_core::RuleError::Unsupported {
                    dependency: "warrior-indexed NPC alias outside the represented owners",
                    ..
                }
            ));
        } else {
            let events = support::saved_command(command, &mut state, &content);
            assert!(events.iter().any(|event| matches!(event, Event::SpellResolved { spell: 2206, target, .. } if *target == body as i16)));
        }
    }
}
