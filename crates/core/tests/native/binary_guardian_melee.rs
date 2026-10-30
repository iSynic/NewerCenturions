use crate::{native_common, prelude::*, support};

use centurions_core::combat::ally_turns;

fn verify(target_kind: i64) {
    let fixture: serde_json::Value = support::fixture("guardian-melee.json");

    let content = support::content();
    let attacks = &content.map(0).unwrap().combat.as_ref().unwrap().attacks;
    let weapon = &content.items[187];
    for (case_index, case) in fixture.array("cases").iter().enumerate() {
        if case.n::<i64>("target_kind") != target_kind {
            continue;
        }
        let context = format!("guardian case{case_index}");
        let source = case.n::<i16>("source");
        let target = case.n::<i16>("target");
        let actor = case.n::<i16>("actor");
        let index = case.n::<usize>("index");
        let mut warriors: Vec<_> = support::warriors(&case["warriors_before"]);
        let mut combat = combat(case, actor, (21, 20));
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let (result, packet) = if target_kind == 0 {
            enemy_weapons::melee_warrior_with_packet(
                &mut combat,
                &mut warriors,
                &mut random,
                (source, target, index, actor),
                weapon,
                0,
                attacks,
                &[0; 58],
            )
        } else {
            enemy_weapons::melee_enemy_with_packet(
                &mut combat,
                &mut random,
                (source, target, index, actor),
                weapon,
                0,
                attacks,
                &[0; 58],
            )
        }
        .unwrap_or_else(|error| panic!("{context}: {error:?}"));
        support::native_expect! {
            records &combat.roster.records => &case["records_after"], &format!("{context} records");
            records &warriors => &case["warriors_after"], &format!("{context} warriors");
            bytes packet => &case["packet_after"], &format!("{context} packet");
            rng random => (&case["seed_after"], &case["rng_draws"]), "binary_guardian_melee";
        }
        let mut sounds = result.sounds;
        sounds.push(
            ally_turns::result_audio(
                &combat.roster.records[(source - 10) as usize],
                index,
                result.damage,
                word(weapon, 14).unwrap(),
            )
            .unwrap(),
        );
        assert_eq!(serde_json::json!(sounds), case["sounds"], "{context}");
        assert!(!result.killed, "{context}");

        assert_eq!(combat.field.tiles, case.words("map_after"), "{context} map");
        support::native_expect! {
            positions combat.roster.mon_pos => &case["positions_after"], "{context} positions";
        };
    }
}

#[test]
fn guardian_melee_matches_native_warrior_poison_packet_audio_and_retained_actor() {
    verify(0);
}

#[test]
fn guardian_melee_matches_native_monster_poison_packet_audio_and_retained_actor() {
    verify(1);
}

fn combat(case: &Value, actor: i16, party_position: (i16, i16)) -> Combat {
    let mut combat = support::battle();
    combat.battle = 85;
    combat.field.tiles = case.words("map_before");
    combat.squad.pos.fill(party_position);
    combat.roster.records = support::monsters(&case["records_before"]);
    combat.roster.mon_pos = support::positions(&case["positions_before"]);
    combat.battle_order[1] = actor;
    combat.cast_identity = [2, 4, 6];
    combat
}

#[test]
fn guardian_reaction_caller_matches_native_retained_actor_reload_and_natural_attacks() {
    let fixture: serde_json::Value = support::fixture("guardian-melee.json");

    let content = support::content();
    let attacks = &content.map(0).unwrap().combat.as_ref().unwrap().attacks;
    let cases = fixture.array("caller_cases");

    for (case_index, case) in cases.iter().enumerate() {
        let context = format!("guardian caller{case_index}");
        let mut combat = combat(case, 22, (60, 60));
        let mut warriors: Vec<_> = support::warriors(&case["warriors_before"]);
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let events = centurions_core::combat::runtime::check_neighbors_with_context(
            &mut combat,
            &mut warriors,
            &mut random,
            22,
            &content.items,
            attacks,
        )
        .unwrap_or_else(|error| panic!("{context}: {error:?}"));
        native_common::assert_body_records(
            &warriors,
            &combat.roster.records,
            &case["warriors_after"],
            &case["records_after"],
        );
        support::native_expect! {
            bytes combat.spell_info => &case["packet_after"], &context;
        }
        assert_eq!(combat.field.tiles, case.words("map_after"), "{context}");
        assert_eq!(combat.battle_order, case.words("order_after"), "{context}");
        support::native_expect! {
            positions combat.roster.mon_pos => &case["positions_after"], "{context} positions";
        };
        support::assert_rng(&random, &case["seed_after"], &case["rng_draws"]);
        let sounds: Vec<_> = support::event_sounds(&events);
        assert_eq!(serde_json::json!(sounds), case["sounds"], "{context}");
        let calls: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                centurions_core::Event::AttackResolved {
                    attacker, target, ..
                } => Some((*attacker, *target)),
                _ => None,
            })
            .collect();
        let expected: Vec<(i16, i16)> = support::attack_calls(&case["calls"]);
        assert_eq!(calls, expected, "{context}");
        let steps = case.array("attack_steps");
        assert_eq!(steps[0]["actor_current_before"], 0);
        assert_eq!(steps[0]["actor_current_after"], 187);
        assert_eq!(steps[1]["actor_current_before"], 187);
        assert_eq!(steps[1]["actor_current_after"], 0);
    }
}
