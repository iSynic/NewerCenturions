use crate::prelude::*;

fn fixture() -> (Content, Value) {
    (
        crate::support::content(),
        crate::support::fixture("learned-portrait-heal.json"),
    )
}

fn project(state: &mut Session, snapshot: &Value) {
    for (warrior, record) in state
        .warriors
        .iter_mut()
        .zip(snapshot.bytes("party").as_chunks::<3138>().0)
    {
        warrior.bytes = Warrior::read(record).unwrap().bytes;
    }
    for (target, key) in [
        (&mut state.crate_items, "crate"),
        (&mut state.booty_items, "loot"),
    ] {
        *target = crate::support::inventory(&bytes(&snapshot[key]));
    }
    state.ammunition = crate::support::longs(&snapshot.bytes("ammunition"));
    state.random.seed = snapshot.n::<u32>("seed");
    state.random.draws = snapshot.n::<u64>("draws");
    state.selected_warrior = snapshot.n::<usize>("selected_member");
    state.loaded_item_field32 = Some(snapshot.n::<i16>("item_context"));
    state.attack_count = snapshot.n::<i16>("count");
    state.attack_packet = Some(snapshot.bytes("packet"));
    state.attack_identity = Some(snapshot.words("identity").try_into().unwrap());
    state.psi_memory = Some(snapshot.bytes("memory"));
    state.party_buffs = snapshot.bytes("party_buffs");
    state
        .party_mask
        .copy_from_slice(&snapshot.words("mask")[..6]);
    state.bandage_ready = Some(snapshot.n::<i16>("aid_gate"));
    let weapon = snapshot.bytes("weapon_work");
    let ammo = snapshot.bytes("ammo_work");
    state.weapon_work = WeaponWork {
        sound: Some(word(&weapon, 14).unwrap()),
        weapon_armor_filter: Some(word(&weapon, 136).unwrap()),
        ammunition_armor_filter: Some(word(&ammo, 136).unwrap()),
    };
    let arena = state.combat.as_mut().unwrap();
    crate::snapshots::records(arena, &|key| snapshot.bytes(key));
    crate::snapshots::geometry(arena, &|key| snapshot.bytes(key), (0, 0));
    arena.cursor = snapshot.n::<usize>("cursor");
    arena.round = snapshot.n::<i16>("round");
    arena.round_scripts.counter = arena.round as i8;
    arena.round_scripts.slots = snapshot.words("round_slots").try_into().unwrap();
    arena.spell_fields = snapshot.bytes("fields");
    arena.field_used = snapshot.bytes("used");
    arena.unavailable = snapshot.n::<i8>("unavailable");
    arena.attack_packet = state.attack_packet.clone();
    arena.attack_identity = state.attack_identity;
    arena.weapon_work = state.weapon_work;
    let mut health = centurions_core::story_health::context(
        &state.warriors,
        state.selected_warrior,
        state.health_context.as_ref(),
    );
    health.unavailable = arena.unavailable;
    health.marks_affected = snapshot.n::<u64>("heal_marks_affected") != 0;
    health
        .affected
        .copy_from_slice(&snapshot.words("markers")[..6]);
    health.effect = state.attack_packet.as_ref().unwrap()[18] as i8;
    state.health_context = Some(health);
}

fn input(fixture: &Value, snapshot: &Value) -> Session {
    let foundation = fixture
        .array("foundations")
        .iter()
        .find(|v| v["session"]["random"]["seed"] == snapshot["seed"])
        .unwrap();
    let mut state = serde_json::from_value(foundation["session"].clone()).unwrap();
    project(&mut state, snapshot);
    state
}
use crate::native_common::refresh_item_work as source_item_refresh;

use crate::support::reject_command as atomic;
use crate::support::same_session as same;

use crate::support::event_sounds as sounds;

fn native_sounds(case: &Value) -> Vec<i16> {
    case.n::<Vec<i16>>("sounds")
}
use crate::support::checkpoint;

#[test]
fn native_learned_portrait_continuations_match_full_session_and_saved_replay() {
    let (content, fixture) = fixture();
    for (index, case) in fixture.array("cases").iter().enumerate() {
        let mut state = input(&fixture, &case["before"]);
        let initial = state.clone();
        let power = case.n::<i16>("power");
        let begin = Command::BeginLearnedHeal { power };
        if !case.n::<bool>("prepared") {
            atomic(&state, &content, begin);
            continue;
        }
        let mut restored = checkpoint(&state, &content, &format!("before-{index}"));
        let mut events = apply(begin.clone(), &mut state, &content).unwrap();
        assert_eq!(
            events,
            apply(begin.clone(), &mut restored, &content).unwrap()
        );
        same(&state, &restored);
        let mut expected = initial.clone();
        project(&mut expected, &case["after_preparation"]);
        source_item_refresh(&mut expected, &content);
        let actor = case.n::<usize>("actor");
        let choice = centurions_core::PsiMenuChoice {
            row: 0,
            column: 0,
            power,
        };
        expected.psi_menu_choices[actor] = Some(choice.clone());
        expected.medical_continuation = Some(MedicalContinuation {
            origin: MedicalOrigin::LearnedHeal,
            actor: case.n::<usize>("actor"),
            item: 0,
            spell: 1101,
            power,
        });
        same(&state, &expected);
        atomic(&state, &content, begin);
        atomic(&state, &content, Command::EndTurn);
        atomic(
            &state,
            &content,
            Command::SelectMedicalTarget {
                selection: MedicalTarget::LivingPoint { x: 1, y: 1 },
            },
        );
        restored = checkpoint(&state, &content, &format!("pending-{index}"));
        let selection = if case["kind"] == "confirmed_cancel" {
            MedicalTarget::ConfirmedCancel
        } else {
            MedicalTarget::Portrait {
                warrior: case.n::<usize>("target"),
            }
        };
        let command = Command::SelectMedicalTarget { selection };
        if matches!(case.text("kind"), "living_portrait" | "terminal_portrait") {
            atomic(&state, &content, command.clone());
            atomic(&restored, &content, command);
            same(&state, &restored);
            continue;
        }
        let result = apply(command.clone(), &mut state, &content).unwrap();
        assert_eq!(
            result,
            apply(command.clone(), &mut restored, &content).unwrap()
        );
        same(&state, &restored);
        events.extend(result);
        let mut expected = initial;
        project(&mut expected, &case["after"]);
        source_item_refresh(&mut expected, &content);
        expected.psi_menu_choices[actor] = Some(choice);
        same(&state, &expected);
        assert_eq!(sounds(&events), native_sounds(case), "learned case {index}");
        atomic(&state, &content, command);
        checkpoint(&state, &content, &format!("after-{index}"));
    }
}

#[test]
fn learned_preparation_rejects_unknown_powers_and_preserves_inventory() {
    let (content, fixture) = fixture();
    let state = input(&fixture, &fixture["cases"][0]["before"]);
    for power in [-1, 0, 5, i16::MAX] {
        atomic(&state, &content, Command::BeginLearnedHeal { power });
    }
    let mut marking = state.clone();
    marking.health_context.as_mut().unwrap().marks_affected = true;
    atomic(&marking, &content, Command::BeginLearnedHeal { power: 1 });
    let mut other_menu = state.clone();
    other_menu.psi_menu_choices[5] = Some(centurions_core::PsiMenuChoice {
        row: 1,
        column: 0,
        power: 5,
    });
    atomic(
        &other_menu,
        &content,
        Command::BeginLearnedHeal { power: 1 },
    );
    for power in [1, 2] {
        let mut prepared = state.clone();
        apply(Command::BeginLearnedHeal { power }, &mut prepared, &content).unwrap();
        for (before, after) in state.warriors.iter().zip(&prepared.warriors) {
            assert_eq!(before.inventory().unwrap(), after.inventory().unwrap());
        }
        assert_eq!(state.random, prepared.random);
    }
}

#[test]
fn consecutive_native_heals_restore_saved_menu_power_without_repeating_costs() {
    let (content, _) = fixture();
    let fixture: Value = crate::support::fixture("learned-heal-memory.json");
    for (index, case) in fixture.array("cases").iter().enumerate() {
        let mut state: Session =
            serde_json::from_value(fixture["foundation"]["session"].clone()).unwrap();
        project(&mut state, &case["before"]);
        let initial = state.clone();
        let second_power = case.n::<i16>("second_power");
        let mut events = vec![];
        for (step, (command, snapshot, pending, remembered_power)) in [
            (
                Command::BeginLearnedHeal { power: 2 },
                "first_prepared",
                true,
                2,
            ),
            (
                Command::SelectMedicalTarget {
                    selection: MedicalTarget::ConfirmedCancel,
                },
                "after_cancel",
                false,
                2,
            ),
            (
                Command::BeginLearnedHeal {
                    power: second_power,
                },
                "second_prepared",
                true,
                second_power,
            ),
            (
                Command::SelectMedicalTarget {
                    selection: MedicalTarget::Portrait { warrior: 1 },
                },
                "after",
                false,
                second_power,
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let mut restored = checkpoint(&state, &content, &format!("memory-{index}-{step}"));
            let result = apply(command.clone(), &mut state, &content).unwrap();
            assert_eq!(result, apply(command, &mut restored, &content).unwrap());
            same(&state, &restored);
            events.extend(result);
            let mut expected = initial.clone();
            project(&mut expected, &case[snapshot]);
            source_item_refresh(&mut expected, &content);
            expected.psi_menu_choices[5] = Some(centurions_core::PsiMenuChoice {
                row: 0,
                column: 0,
                power: remembered_power,
            });
            if pending {
                expected.medical_continuation = Some(MedicalContinuation {
                    origin: MedicalOrigin::LearnedHeal,
                    actor: 5,
                    item: 0,
                    spell: 1101,
                    power: remembered_power,
                });
            }
            same(&state, &expected);
        }
        assert_eq!(sounds(&events), native_sounds(case), "memory case {index}");
    }
}

#[test]
fn owned_prior_spell_row_matches_native_heal_menu_and_saved_continuation() {
    let (content, _) = fixture();
    let fixture: Value = crate::support::fixture("learned-heal-row.json");
    for (index, case) in fixture.array("cases").iter().enumerate() {
        let mut state: Session =
            serde_json::from_value(fixture["foundation"]["session"].clone()).unwrap();
        project(&mut state, &case["before"]);
        let selected = case["before"].bytes("selected");
        state.psi_menu_choices[5] = Some(centurions_core::PsiMenuChoice {
            row: selected[10],
            column: selected[11],
            power: case["before"].words("powers")[5],
        });
        let initial = state.clone();
        let power = case.n::<i16>("power");
        let mut events = vec![];
        for (step, (command, snapshot, pending)) in [
            (Command::BeginLearnedHeal { power }, "prepared", true),
            (
                Command::SelectMedicalTarget {
                    selection: MedicalTarget::Portrait { warrior: 1 },
                },
                "after",
                false,
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let mut restored = checkpoint(&state, &content, &format!("row-{index}-{step}"));
            let result = apply(command.clone(), &mut state, &content).unwrap();
            assert_eq!(result, apply(command, &mut restored, &content).unwrap());
            same(&state, &restored);
            events.extend(result);
            let mut expected = initial.clone();
            project(&mut expected, &case[snapshot]);
            source_item_refresh(&mut expected, &content);
            expected.psi_menu_choices[5] = Some(centurions_core::PsiMenuChoice {
                row: 0,
                column: 0,
                power,
            });
            if pending {
                expected.medical_continuation = Some(MedicalContinuation {
                    origin: MedicalOrigin::LearnedHeal,
                    actor: 5,
                    item: 0,
                    spell: 1101,
                    power,
                });
            }
            same(&state, &expected);
        }
        assert_eq!(sounds(&events), native_sounds(case), "row case {index}");
    }
}

#[test]
fn higher_portrait_powers_match_native_full_session_and_saved_continuations() {
    let (content, _) = fixture();
    let fixture: Value = crate::support::fixture("learned-portrait-power.json");
    for (index, case) in fixture.array("cases").iter().enumerate() {
        let mut state = input(&fixture, &case["before"]);
        let initial = state.clone();
        let power = case.n::<i16>("power");
        let actor = state.combat.as_ref().unwrap().active_body().unwrap() as usize;
        let kind = case.text("kind");
        for snapshot in ["before", "prepared", "after"] {
            assert_eq!(
                case[snapshot]["health_effect"].as_u64().unwrap(),
                bytes(&case[snapshot]["packet"])[18] as u64
            );
            assert_eq!(
                case[snapshot]["health_last_body"].as_i64().unwrap(),
                initial.health_context.as_ref().unwrap().last_body as i64
            );
        }
        let begin = Command::BeginLearnedHeal { power };
        if kind == "psi_below_cost" {
            assert_eq!(initial.warriors[actor].psi_points().unwrap(), 6 * power - 1);
            atomic(&initial, &content, begin);
            continue;
        }
        let mut restored = checkpoint(&state, &content, &format!("power-before-{index}"));
        let mut events = apply(begin.clone(), &mut state, &content).unwrap();
        assert_eq!(events, apply(begin, &mut restored, &content).unwrap());
        same(&state, &restored);
        let choice = centurions_core::PsiMenuChoice {
            row: 0,
            column: 0,
            power,
        };
        let mut expected = initial.clone();
        project(&mut expected, &case["prepared"]);
        source_item_refresh(&mut expected, &content);
        expected.psi_menu_choices[actor] = Some(choice.clone());
        expected.medical_continuation = Some(MedicalContinuation {
            origin: MedicalOrigin::LearnedHeal,
            actor,
            item: 0,
            spell: 1101,
            power,
        });
        same(&state, &expected);

        assert_eq!(state.attack_count, power);
        atomic(&state, &content, Command::EndTurn);
        atomic(
            &state,
            &content,
            Command::SelectMedicalTarget {
                selection: MedicalTarget::Portrait { warrior: 0 },
            },
        );
        restored = checkpoint(&state, &content, &format!("power-pending-{index}"));
        if kind == "psi_equal_cost" {
            assert_eq!(state.warriors[actor].psi_points().unwrap(), 0);
            assert_eq!(sounds(&events), native_sounds(case));
            continue;
        }
        let selection = if kind == "confirmed_cancel" {
            MedicalTarget::ConfirmedCancel
        } else {
            MedicalTarget::Portrait {
                warrior: case.n::<usize>("target"),
            }
        };
        let command = Command::SelectMedicalTarget { selection };
        let result = apply(command.clone(), &mut state, &content).unwrap();
        assert_eq!(
            result,
            apply(command.clone(), &mut restored, &content).unwrap()
        );
        same(&state, &restored);
        events.extend(result);
        let mut expected = initial;
        project(&mut expected, &case["after"]);
        source_item_refresh(&mut expected, &content);
        expected.psi_menu_choices[actor] = Some(choice);
        same(&state, &expected);
        assert_eq!(sounds(&events), native_sounds(case), "power case {index}");
        atomic(&state, &content, command);
        checkpoint(&state, &content, &format!("power-after-{index}"));
    }
}
