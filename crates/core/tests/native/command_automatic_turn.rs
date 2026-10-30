use crate::{native_common, prelude::*, snapshots, support};

fn project(state: &mut Session, snapshot: &Value, owned: bool) {
    for (warrior, native) in state
        .warriors
        .iter_mut()
        .zip(support::warriors(&snapshot["party"]))
    {
        warrior.bytes = native.bytes;
    }
    if owned {
        state.crate_items = support::inventory(&snapshot.bytes("crate"));
    }
    state.ammunition = support::longs(&snapshot.bytes("ammunition"));
    state.random = native_common::snapshot_rng(snapshot);
    state.globals.loaded_item_field32 = snapshot.n::<i16>("item_context");
    state.globals.attack_count = snapshot.n::<i16>("attack_count");
    state.globals.define_spells = snapshot.bytes("memory");
    state.party_condition = snapshot.bytes("party_buffs");
    state.globals.weapon_traits = snapshots::weapon_traits(&|key| {
        snapshot.bytes(if key == "weapon_work" {
            "weapon"
        } else {
            "ammo"
        })
    });
    let combat = state.combat.as_mut().unwrap();
    combat.field.map_shift = (
        snapshot["shift_x"].as_i64().unwrap_or(0) as i16,
        snapshot["shift_y"].as_i64().unwrap_or(0) as i16,
    );
    combat.roster.records = support::monsters(&snapshot.bytes("target"));
    snapshots::geometry(
        combat,
        &|key| {
            if key == "under_tiles" {
                [0x04, 0x83].repeat(6)
            } else {
                snapshot.bytes(key)
            }
        },
        (0, 0),
    );
    combat.roster.mon_pos.truncate(1);
    combat.battle_order_index = snapshot.n::<usize>("cursor");
    combat.combat_round = snapshot.n::<i16>("round");
    combat.kill_party = snapshot.n::<i8>("unavailable");
    snapshots::spell_work(combat, &|key| snapshot.bytes(key));
    combat.weapon_traits = state.globals.weapon_traits;
    state.globals.spell_info = combat.spell_info.clone();
    state.globals.cast_identity = combat.cast_identity;
    let mut health = centurions_core::story_health::context(
        &state.warriors,
        state.char_select_new,
        Some(&state.globals.health_context),
    );
    if owned {
        health.affected = snapshot.words("markers")[..6].try_into().unwrap();
        health.effect = snapshot.bytes("packet")[18] as i8;
    } else {
        health.kill_party = combat.kill_party;
    }
    state.globals.health_context = health;
}

// Independent native low-AP fit/shoot receipts protect the automatic enemy-turn boundary.
#[test]
fn native_automatic_turns_preserve_complete_state_events_and_save_resume() {
    let content = content();
    for (name, owned, group) in [
        ("empty-rifle-advance.json", false, "cases"),
        ("owned-player-firearm-target.json", true, "fire_cases"),
    ] {
        let fixture = support::fixture(name);
        for (index, case) in fixture.array(group).iter().enumerate() {
            let before = &case[if owned { "combat_before" } else { "before" }];
            let after = &case[if owned { "advanced" } else { "after" }];
            let mut state: Session =
                serde_json::from_value(fixture["foundation"]["session"].clone()).unwrap();
            state.phase = Phase::Combat;
            state.combat = Some(if owned {
                let mut combat = support::battle();
                combat.battle = 85;
                combat
            } else {
                serde_json::from_value(
                    support::fixture("player-psi.json")["route_foundation"]["combat"].clone(),
                )
                .unwrap()
            });
            project(&mut state, before, owned);
            native_common::source_victory_context(&mut state);
            let original = state.clone();
            let command = if owned {
                Command::ShootEnemy { body: 10 }
            } else {
                Command::FitAmmunition {
                    warrior: 0,
                    slot: case.n::<usize>("inventory_slot"),
                    ammunition: 203,
                }
            };
            let mut resumed = support::checkpoint(&state, &content, "automatic-before");
            let events = replay_command(command, &mut state, &mut resumed, &content);
            let mut expected = original;
            project(&mut expected, after, owned);
            expected.globals.loaded_items.item = Some(after.bytes("item_work"));
            expected.globals.loaded_items.ammo = Some(after.bytes("ammo"));
            if owned {
                expected.globals.loaded_items.weapon = Some(after.bytes("weapon"));
            }
            expected.bandage_ready = 1;
            expected.globals.health_context = centurions_core::story_health::context(
                &expected.warriors,
                expected.char_select_new,
                Some(&expected.globals.health_context),
            );
            support::same_captured_session(&state, &expected);
            let mut expected_events: Vec<_> = after.n::<Vec<i16>>("sounds")
                [before.array("sounds").len()..]
                .iter()
                .map(|id| Event::Sound { resource_id: *id })
                .collect();
            if owned {
                // The target key's firstt(76, 77) (combatchoice 11b026).
                expected_events.insert(
                    0,
                    Event::Feedback(centurions_core::feedback::GameFeedback::Tutorial {
                        first: 76,
                        last: 77,
                    }),
                );
                expected_events.push(Event::AttackResolved {
                    attacker: case.n::<i16>("member"),
                    target: 10,
                    damage: word(&before.bytes("target"), 0x110)
                        .unwrap()
                        .wrapping_sub(word(&after.bytes("target"), 0x110).unwrap()),
                    defeated: false,
                    icon: Default::default(),
                    flight: Default::default(),
                });
            }
            expected_events.push(Event::TurnStarted { body: 10, round: 1 });
            if expected.warriors[case.n::<usize>("member")]
                .record()
                .guarding()
                .unwrap()
                != 0
            {
                expected_events =
                    native_common::turn_guard_events(expected_events, case.n("member"));
            }
            assert_eq!(events, expected_events, "{name}:{index}");
            support::checkpoint(&state, &content, "automatic-after");
        }
    }
}
