use crate::prelude::*;

fn project(state: &mut Session, snapshot: &Value, owned: bool) {
    for (warrior, native) in state
        .warriors
        .iter_mut()
        .zip(crate::support::warriors(&snapshot["party"]))
    {
        warrior.bytes = native.bytes;
    }
    if owned {
        state.crate_items = crate::support::inventory(&snapshot.bytes("crate"));
    }
    state.ammunition = crate::support::longs(&snapshot.bytes("ammunition"));
    state.random = crate::native_common::snapshot_rng(snapshot);
    state.loaded_item_field32 = Some(snapshot.n::<i16>("item_context"));
    state.attack_count = snapshot.n::<i16>("attack_count");
    state.psi_memory = Some(snapshot.bytes("memory"));
    state.party_buffs = snapshot.bytes("party_buffs");
    state.weapon_work = crate::snapshots::weapon_work(&|key| {
        snapshot.bytes(if key == "weapon_work" {
            "weapon"
        } else {
            "ammo"
        })
    });
    let combat = state.combat.as_mut().unwrap();
    combat.map.shift = (
        snapshot["shift_x"].as_i64().unwrap_or(0) as i16,
        snapshot["shift_y"].as_i64().unwrap_or(0) as i16,
    );
    combat.enemies.records = crate::support::monsters(&snapshot.bytes("target"));
    crate::snapshots::geometry(
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
    combat.enemies.positions.truncate(1);
    combat.cursor = snapshot.n::<usize>("cursor");
    combat.round = snapshot.n::<i16>("round");
    combat.unavailable = snapshot.n::<i8>("unavailable");
    crate::snapshots::spell_work(combat, &|key| snapshot.bytes(key));
    combat.weapon_work = state.weapon_work;
    state.attack_packet = combat.attack_packet.clone();
    state.attack_identity = combat.attack_identity;
    let mut health = centurions_core::story_health::context(
        &state.warriors,
        state.selected_warrior,
        state.health_context.as_ref(),
    );
    if owned {
        health.affected = snapshot.words("markers")[..6].try_into().unwrap();
        health.effect = snapshot.bytes("packet")[18] as i8;
    } else {
        health.unavailable = combat.unavailable;
    }
    state.health_context = Some(health);
}

// Independent native low-AP fit/shoot receipts protect the automatic enemy-turn boundary.
#[test]
fn native_automatic_turns_preserve_complete_state_events_and_save_resume() {
    let content = content();
    for (name, owned, group) in [
        ("empty-rifle-advance.json", false, "cases"),
        ("owned-player-firearm-target.json", true, "fire_cases"),
    ] {
        let fixture = crate::support::fixture(name);
        for (index, case) in fixture.array(group).iter().enumerate() {
            let before = &case[if owned { "combat_before" } else { "before" }];
            let after = &case[if owned { "advanced" } else { "after" }];
            let mut state: Session =
                serde_json::from_value(fixture["foundation"]["session"].clone()).unwrap();
            state.phase = Phase::Combat;
            state.combat = Some(if owned {
                let mut combat = crate::support::battle();
                combat.battle = 85;
                combat
            } else {
                serde_json::from_value(
                    crate::support::fixture("player-psi.json")["route_foundation"]["combat"]
                        .clone(),
                )
                .unwrap()
            });
            project(&mut state, before, owned);
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
            let mut resumed = crate::support::checkpoint(&state, &content, "automatic-before");
            let events = apply(command.clone(), &mut state, &content).unwrap();
            assert_eq!(apply(command, &mut resumed, &content).unwrap(), events);
            crate::support::same_session(&state, &resumed);
            let mut expected = original;
            project(&mut expected, after, owned);
            expected.ammunition_work.item = Some(after.bytes("item_work"));
            expected.ammunition_work.ammo = Some(after.bytes("ammo"));
            if owned {
                expected.ammunition_work.weapon = Some(after.bytes("weapon"));
            }
            expected.bandage_ready = Some(1);
            expected.health_context = Some(centurions_core::story_health::context(
                &expected.warriors,
                expected.selected_warrior,
                expected.health_context.as_ref(),
            ));
            crate::support::same_session(&state, &expected);
            let mut expected_events: Vec<_> = after.n::<Vec<i16>>("sounds")
                [before.array("sounds").len()..]
                .iter()
                .map(|id| Event::Sound { resource_id: *id })
                .collect();
            if owned {
                expected_events.push(Event::AttackResolved {
                    attacker: case.n::<i16>("member"),
                    target: 10,
                    damage: word(&before.bytes("target"), 0x110)
                        .unwrap()
                        .wrapping_sub(word(&after.bytes("target"), 0x110).unwrap()),
                    defeated: false,
                });
            }
            expected_events.push(Event::TurnStarted { body: 10, round: 1 });
            assert_eq!(events, expected_events, "{name}:{index}");
            crate::support::checkpoint(&state, &content, "automatic-after");
        }
    }
}
