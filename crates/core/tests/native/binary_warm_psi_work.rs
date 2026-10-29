use crate::{native_common, prelude::*, support};

use native_common::{psi_snapshot as state, psi_work as native_work};

fn assert_snapshot(state: &Session, expected: &Value, events: &[Event], label: &str) {
    support::psi_frame::compare_session(state, expected, label);
    let sounds: Vec<_> = support::event_sounds(events);
    assert_eq!(
        serde_json::json!(sounds),
        expected["sounds"],
        "{label} audio"
    );
    let mut receipt = RandomState::new(12);
    support::replay_rng(&mut receipt, expected.array("rng_records"));
    assert_eq!(receipt, state.random, "{label} RNG receipt state");
}

#[test]
fn native_loading_and_preparation_preserve_independent_weapon_and_ammunition_fields() {
    let content = content();
    let fixture = support::fixture("warm-psi-work.json");
    for case in fixture.array("cases") {
        let mut work = native_work(&case["unwarmed"]);
        let weapon = &content.items[case.n::<usize>("warm_weapon")];
        let ammunition = &content.items[case.n::<usize>("warm_ammo")];
        work.load_pair(weapon, ammunition).unwrap();
        assert_eq!(work, native_work(&case["loaded"]));
        work.prepare(weapon, ammunition).unwrap();
        assert_eq!(work, native_work(&case["prepared"]));
        for phase in ["before", "charged", "targeted", "cast", "after", "collided"] {
            assert_eq!(work, native_work(&case[phase]));
        }
    }
}

#[test]
fn warmed_cast_automatic_initiative_and_field_collision_match_complete_native_cases() {
    let content = content();
    let mission = content.map(0).unwrap();
    let fixture = support::fixture("warm-psi-work.json");

    for case in fixture.array("cases") {
        let label = format!(
            "warm weapon{} ammo{}",
            case["warm_weapon"], case["warm_ammo"]
        );
        let mut state = state(&content, &case["before"]);
        let original = current_combat(&state).clone();
        let point = words(&case["casts"][0]["targets"]);
        let mut mask_rng = state.random.clone();
        let c = current_combat(&state);
        let packet = &c.attack_packet;
        let mask = spells::target_mask(
            c,
            &state.warriors,
            &mission.areas,
            &spells::SpellTarget {
                first: point[0],
                second: point[1],
                kind: 3,
            },
            &spells::SpellContext {
                packet,
                id: 2101,
                count: 1,
                caster: 1,
                initiative: c.cursor as i16,
                collision: false,
            },
            &c.spell_fields,
            &mut mask_rng,
        )
        .unwrap();
        assert_eq!(mask.mask.to_vec(), words(&case["resolutions"][0]["mask"]));
        assert_eq!(mask.fields, bytes(&case["field_events"][1]["fields"]));
        assert_eq!(mask_rng.draws, case["resolutions"][0]["draws_before"]);
        let mut events = player_psi::cast(
            state.combat.as_mut().unwrap(),
            &mut state.warriors,
            mission,
            &mut state.random,
            &mut state.native_work.psi_memory,
            0,
            CastRequest {
                rotation: 0,
                spell: 2101,
                power: 1,
                target: Some((point[0] - 5, point[1] - 5)),
            },
        )
        .unwrap_or_else(|error| panic!("{label}: {error:?}"));
        assert_snapshot(&state, &case["cast"], &events, &format!("{label} cast"));
        events.extend(runtime::end_turn(&mut state, &content).unwrap());
        assert_eq!(current_combat(&state).active_body(), Some(10));
        assert_snapshot(&state, &case["after"], &events, &format!("{label} advance"));
        events.extend(
            runtime::collide_fields(
                state.combat.as_mut().unwrap(),
                &mut state.warriors,
                mission,
                &mut state.native_work.attack_count,
                &mut state.random,
                (10, state.selected_warrior, &mut state.bandage_ready),
            )
            .unwrap(),
        );
        assert_snapshot(
            &state,
            &case["collided"],
            &events,
            &format!("{label} collision"),
        );
        let c = current_combat(&state);
        assert_eq!(c.death_queue, original.death_queue);
        assert_eq!(c.round_scripts, original.round_scripts);
        let expected_filter = i16::from(c.weapon_work.armor_filter());
        for call in case.array("damage_arguments") {
            assert_eq!(call["arguments"][4], expected_filter);
        }
    }
}
