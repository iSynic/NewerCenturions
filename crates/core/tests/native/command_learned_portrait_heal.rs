use crate::{native_common, prelude::*, snapshots, support};

pub(super) fn project(state: &mut Session, snapshot: &Value) {
    let read = |key: &str| snapshot.bytes(key);
    snapshots::party_work(state, snapshot, &read);
    snapshots::carried_items(state, &read);
    state.ammunition = support::longs(&read("ammunition"));
    state.char_select_new = snapshot.n("selected_member");
    state.globals.loaded_item_field32 = snapshot.n("item_context");
    state.bandage_ready = snapshot.n("aid_gate");
    state.globals.weapon_traits = snapshots::weapon_traits(&read);
    let arena = state.combat.as_mut().unwrap();
    let before = (arena.battle_order_index, arena.combat_round);
    snapshots::records(arena, &|key| snapshot.bytes(key));
    snapshots::geometry(arena, &|key| snapshot.bytes(key), (0, 0));
    if let Some(rounds) = snapshot.get("inactive_rounds") {
        arena.blank_round = rounds.as_i64().unwrap() as i16;
    }
    arena.battle_order_index = snapshot.n::<usize>("cursor");
    arena.combat_round = snapshot.n::<i16>("round");
    snapshots::turn_origin(arena, before);
    arena.round_macros.counter = snapshot
        .get("round_counter")
        .map_or(arena.combat_round as i8, |value| value.as_i64().unwrap() as i8);
    arena.round_macros.slots = snapshot.words("round_slots").try_into().unwrap();
    snapshots::spell_work(arena, &read);
    arena.kill_party = snapshot.n::<i8>("unavailable");
    arena.spell_info = state.globals.spell_info.clone();
    arena.cast_identity = state.globals.cast_identity;
    arena.weapon_traits = state.globals.weapon_traits;
    let mut health = centurions_core::story_health::context(
        &state.warriors,
        state.char_select_new,
        Some(&state.globals.health_context),
    );
    health.kill_party = arena.kill_party;
    health.marks_affected = snapshot.n::<u64>("heal_marks_affected") != 0;
    health
        .affected
        .copy_from_slice(&snapshot.words("markers")[..6]);
    health.effect = state.globals.spell_info[18] as i8;
    state.globals.health_context = health;
    snapshots::actor_work(state, snapshot);
}

fn input(fixture: &Value, snapshot: &Value) -> Session {
    let foundation = fixture
        .array("foundations")
        .iter()
        .find(|v| v["session"]["random"]["seed"] == snapshot["seed"])
        .unwrap();
    let mut state = serde_json::from_value(foundation["session"].clone()).unwrap();
    project(&mut state, snapshot);
    native_common::source_victory_context(&mut state);
    state
}
use native_common::refresh_item_work as source_item_refresh;

use support::reject_command as atomic;

use support::event_sounds as sounds;

use support::checkpoint;

fn native_sounds(case: &Value) -> Vec<i16> {
    case.n("sounds")
}

fn assert_native_damage(case: &Value, events: &[Event]) {
    if matches!(case.text("kind"), "actual" | "actual_portrait") {
        let damage = case
            .array("effect_events")
            .iter()
            .find(|event| event["address"] == "1518d4")
            .unwrap()
            .n::<u32>("d0") as u16 as i16;
        let resolved: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                Event::SpellResolved {
                    caster,
                    spell,
                    target,
                    damage,
                    defeated,
                    ..
                } => Some((*caster, *spell, *target, *damage, *defeated)),
                _ => None,
            })
            .collect();
        assert_eq!(
            resolved,
            vec![(Some(5), 1101, case.n("target"), damage, false)]
        );
    }
}

#[test]
fn native_learned_portrait_continuations_match_full_session_and_saved_replay() {
    let content = support::content();
    let fixture: Value = support::fixture("learned-portrait-heal.json");
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
        let mut events = replay_command(begin.clone(), &mut state, &mut restored, &content);
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
        support::same_captured_session(&state, &expected);
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
            support::refuse_command(
                &state,
                &content,
                command,
                vec![centurions_core::feedback::GameFeedback::Sound { resource_id: 143 }],
            );
            continue;
        }
        let result = replay_command(command.clone(), &mut state, &mut restored, &content);
        events.extend(result);
        let mut expected = initial;
        project(&mut expected, &case["after"]);
        source_item_refresh(&mut expected, &content);
        expected.psi_menu_choices[actor] = Some(choice);
        support::same_captured_session(&state, &expected);
        assert_eq!(sounds(&events), native_sounds(case), "learned case {index}");
        assert_native_damage(case, &events);
        atomic(&state, &content, command);
        checkpoint(&state, &content, &format!("after-{index}"));
    }
}

#[test]
fn higher_portrait_heals_keep_the_paid_selection_and_original_restoration() {
    let content = support::content();
    let fixture = support::fixture("learned-portrait-heal.json");
    let control = &fixture["cases"][0];
    for spell in [1401_i16, 1602, 3603] {
        let mut state = input(&fixture, &control["before"]);
        let caster = &mut state.warriors[5].bytes;
        caster[0x36..0x38].copy_from_slice(&(spell / 1000).to_be_bytes());
        caster[0x9f0..0xa21].fill(1);
        caster[0x58..0x5a].copy_from_slice(&2000_i16.to_be_bytes());
        state.combat.as_mut().unwrap().weapon_traits =
            centurions_core::combat::weapon_traits::WeaponTraits::zero();
        state.random = RandomState::new(12);
        support::saved_command(
            Command::BeginPsiTargeting { spell, power: 1 },
            &mut state,
            &content,
        );
        let native = support::fixture(&format!("portrait-heal-{spell}-return.json"));
        let paid = state.warriors[5].psi_points().unwrap();
        let events = support::saved_command(
            Command::FirePreparedPsiOnWarrior { warrior: 1 },
            &mut state,
            &content,
        );
        let mut sounds = vec![-141_i16];
        sounds.extend(native.n::<Vec<i16>>("sounds_before_showresults"));
        let combat = current_combat(&state);
        support::native_expect! {
            records &state.warriors => &native["party_after"], "higher portrait party";
            records &combat.roster.records => &native["npcs_after"], "higher portrait NPCs";
            words combat.field.tiles => &native["map_after"], "higher portrait map";
            positions combat.squad.pos => &native["positions_after"], "higher portrait placement";
            rng state.random => (&native["seed_after"], &native["rng_draws"]), "higher portrait RNG";
        }
        assert_eq!(support::event_sounds(&events), sounds);
        assert_eq!(state.warriors[5].psi_points().unwrap(), paid);
        assert!(state.psi_selection.is_none() && state.medical_continuation.is_none());
    }
}

#[test]
fn learned_preparation_rejects_unknown_powers_and_preserves_inventory() {
    let content = support::content();
    let fixture: Value = support::fixture("learned-portrait-heal.json");
    let state = input(&fixture, &fixture["cases"][0]["before"]);
    for case in support::fixture("target-quantity.json").array("cases") {
        let mut pending = state.clone();
        let actor = current_combat(&pending).active_body().unwrap() as usize;
        pending.warriors[actor].bytes[0x58..0x5a].copy_from_slice(&100_i16.to_be_bytes());
        apply(
            Command::BeginLearnedHeal {
                power: case.n("power_before"),
            },
            &mut pending,
            &content,
        )
        .unwrap();
        pending.warriors[actor].bytes[0x58..0x5a]
            .copy_from_slice(&case.n::<i16>("psi_before").to_be_bytes());
        let mut resumed = checkpoint(&pending, &content, "quantity-before");
        let random = pending.random.clone();
        let command = Command::AdjustTargetQuantity {
            increase: case.n("increase"),
        };
        if case["sounds"] == serde_json::json!([143]) {
            let before = pending.clone();
            let Err(RuleError::Refused { feedback }) = apply(command, &mut pending, &content)
            else {
                panic!("expected native quantity refusal");
            };
            assert_eq!(
                feedback,
                vec![centurions_core::feedback::GameFeedback::Sound { resource_id: 143 }]
            );
            assert_eq!(pending, before);
        } else {
            let events = replay_command(command, &mut pending, &mut resumed, &content);
            assert_eq!(sounds(&events), native_sounds(case));
        }
        assert_eq!(
            pending.medical_continuation.as_ref().unwrap().power,
            case.n::<i16>("power_after")
        );
        assert_eq!(
            pending.warriors[actor].psi_points().unwrap(),
            case.n::<i16>("psi_after")
        );
        assert_eq!(pending.random, random);
    }
    for power in [-1, 0, 8, i16::MAX] {
        atomic(&state, &content, Command::BeginLearnedHeal { power });
    }
    let mut marking = state.clone();
    marking
        .globals
        .health_context
        .marks_affected = true;
    let mut unmarked = state.clone();
    for candidate in [&mut marking, &mut unmarked] {
        apply(Command::BeginLearnedHeal { power: 1 }, candidate, &content).unwrap();
    }
    let target = fixture["cases"][0].n::<usize>("target");
    let command = Command::SelectMedicalTarget {
        selection: MedicalTarget::Portrait { warrior: target },
    };
    assert_eq!(
        apply(command.clone(), &mut marking, &content).unwrap(),
        apply(command, &mut unmarked, &content).unwrap()
    );
    assert_eq!(marking.warriors, unmarked.warriors);
    assert_eq!(marking.random, unmarked.random);
    assert_eq!(
        marking
            .globals
            .health_context
            .affected[target],
        -1
    );
    let mut other_menu = state.clone();
    other_menu.psi_menu_choices[5] = Some(centurions_core::PsiMenuChoice {
        row: 1,
        column: 0,
        power: 8,
    });
    atomic(
        &other_menu,
        &content,
        Command::BeginLearnedHeal { power: 1 },
    );
}

#[test]
fn consecutive_native_heals_restore_saved_menu_power_without_repeating_costs() {
    let content = support::content();
    let fixture: Value = support::fixture("learned-heal-memory.json");
    for (index, case) in fixture.array("cases").iter().enumerate() {
        let mut state: Session =
            serde_json::from_value(fixture["foundation"]["session"].clone()).unwrap();
        project(&mut state, &case["before"]);
        native_common::source_victory_context(&mut state);
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
            let result = replay_command(command, &mut state, &mut restored, &content);
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
            support::same_captured_session(&state, &expected);
        }
        assert_eq!(sounds(&events), native_sounds(case), "memory case {index}");
    }
}

#[test]
fn owned_prior_spell_row_matches_native_heal_menu_and_saved_continuation() {
    let content = support::content();
    let fixture: Value = support::fixture("learned-heal-row.json");
    for (index, case) in fixture.array("cases").iter().enumerate() {
        let mut state: Session =
            serde_json::from_value(fixture["foundation"]["session"].clone()).unwrap();
        project(&mut state, &case["before"]);
        native_common::source_victory_context(&mut state);
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
            let result = replay_command(command, &mut state, &mut restored, &content);
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
            support::same_captured_session(&state, &expected);
        }
        assert_eq!(sounds(&events), native_sounds(case), "row case {index}");
    }
}

#[test]
fn higher_portrait_powers_match_native_full_session_and_saved_continuations() {
    let content = support::content();
    let fixture: Value = support::fixture("learned-portrait-power.json");
    let armored = support::fixture("armored-portrait-heal.json");
    let full = support::fixture("full-portrait-power.json");
    for (index, case) in fixture
        .array("cases")
        .iter()
        .chain(armored.array("cases"))
        .chain(full.array("cases"))
        .enumerate()
    {
        let mut state = input(&fixture, &case["before"]);
        let initial = state.clone();
        let power = case.n::<i16>("power");
        let actor = current_combat(&state).active_body().unwrap() as usize;
        let kind = case.text("kind");
        let begin = Command::BeginLearnedHeal { power };
        if kind == "psi_below_cost" {
            assert_eq!(initial.warriors[actor].psi_points().unwrap(), 6 * power - 1);
            atomic(&initial, &content, begin);
            continue;
        }
        let mut restored = checkpoint(&state, &content, &format!("power-before-{index}"));
        let mut events = replay_command(begin, &mut state, &mut restored, &content);
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
        support::same_captured_session(&state, &expected);

        assert_eq!(state.globals.attack_count, power);
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
        let result = replay_command(command.clone(), &mut state, &mut restored, &content);
        events.extend(result);
        let mut expected = initial;
        project(&mut expected, &case["after"]);
        source_item_refresh(&mut expected, &content);
        expected.psi_menu_choices[actor] = Some(choice);
        support::same_captured_session(&state, &expected);
        assert_eq!(sounds(&events), native_sounds(case), "power case {index}");
        assert_native_damage(case, &events);
        atomic(&state, &content, command);
        checkpoint(&state, &content, &format!("power-after-{index}"));
        if power > 4 && kind == "confirmed_cancel" {
            apply(Command::BeginLearnedHeal { power: 1 }, &mut state, &content).unwrap();
        }
    }
}
