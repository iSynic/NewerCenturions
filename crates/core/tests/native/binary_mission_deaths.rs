use crate::prelude::*;

use centurions_core::combat::mission_deaths;
use centurions_core::combat::mission_deaths::DeathContext;

use crate::native_common::script_mission as mission;
use crate::support::field_bytes as bytes;
fn input(case: &serde_json::Value) -> (Combat, Vec<Warrior>, i8) {
    (
        Combat {
            battle: 146,
            map: TacticalMap {
                tiles: words(&bytes(case, "map_before")),
                edge_word: 0,
                shift: (0, 0),
            },
            squad: SquadPlacement {
                positions: vec![(40, 40); 6],
                under_tiles: vec![1155; 6],
            },
            enemies: EnemySetup {
                records: crate::support::monsters(&bytes(case, "records_before")),
                positions: vec![(20, 20); 100],
            },
            order: words(&bytes(case, "order_before")),
            ..crate::support::battle()
        },
        crate::support::warriors(&case.bytes("warriors_before")),
        case.n::<i8>("unavailable_before"),
    )
}
fn context(case: &serde_json::Value) -> DeathContext {
    DeathContext {
        script: if case.n::<i64>("monster_id") == 119 {
            336
        } else {
            337
        },
        actor: case.n::<usize>("actor_before"),
        deferred_faction: (case.n::<i64>("processing_before") != 0)
            .then(|| case.n::<i16>("faction")),
    }
}

#[test]
fn original_direct_and_queued_death_story_phases_match_complete_native_state() {
    let fixture = crate::support::fixture("mission-deaths.json");
    let mission = mission(&fixture);
    let extras = bytes(&fixture, "extras");
    for (index, case) in fixture.array("cases").iter().enumerate() {
        let (mut combat, mut warriors, mut unavailable) = input(case);
        let result = mission_deaths::apply(
            &mut combat,
            &mut warriors,
            &mission,
            &extras,
            context(case),
            &mut unavailable,
        )
        .unwrap();
        let records: Vec<u8> = crate::support::record_bytes(&combat.enemies.records);
        let party: Vec<u8> = crate::support::record_bytes(&warriors);
        assert_eq!(records, bytes(case, "records_after"), "records case{index}");
        assert_eq!(party, bytes(case, "warriors_after"), "party case{index}");
        assert_eq!(
            combat.map.tiles,
            words(&bytes(case, "map_after")),
            "map case{index}"
        );
        assert_eq!(
            combat.order,
            words(&bytes(case, "order_after")),
            "order case{index}"
        );
        crate::support::expect(unavailable as i64, &case["unavailable_after"]);
        assert_eq!(
            result.ended_early,
            case.n::<i64>("return_value") == -1,
            "return case{index}"
        );
        assert_eq!(
            result.text,
            if context(case).script == 336 {
                1247
            } else {
                1248
            }
        );

        let queued = case.n::<u64>("queued");
        assert_eq!(context(case).actor, if queued == 0 { 3 } else { 0 });
        assert_eq!(case.n::<u64>("queued_count"), u64::from(queued != 0));
    }
}

#[test]
fn altered_death_script_and_invalid_actor_are_transactional_diagnostics() {
    let fixture = crate::support::fixture("mission-deaths.json");
    let mission = mission(&fixture);
    let extras = bytes(&fixture, "extras");
    let case = &fixture["cases"][0];
    for variant in 0..3 {
        let (mut combat, mut warriors, mut unavailable) = input(case);
        let mut context = context(case);
        match variant {
            0 => context.script = 999,
            1 => context.actor = 100,
            _ => warriors[0].bytes.clear(),
        }
        let before = (combat.clone(), warriors.clone(), unavailable);
        assert!(
            mission_deaths::apply(
                &mut combat,
                &mut warriors,
                &mission,
                &extras,
                context,
                &mut unavailable
            )
            .is_err()
        );
        assert_eq!((combat, warriors, unavailable), before);
    }
}
