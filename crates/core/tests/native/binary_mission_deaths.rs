use crate::{native_common, prelude::*, support};

use centurions_core::combat::mission_deaths;
use centurions_core::combat::mission_deaths::DeathContext;

use native_common::script_mission as mission;
fn input(case: &serde_json::Value) -> (Combat, Vec<Warrior>, i8) {
    let mut combat = support::battle();
    combat.battle = 146;
    combat.map.tiles = case.words("map_before");
    combat.squad.positions = vec![(40, 40); 6];
    combat.enemies.records = support::monsters(&case["records_before"]);
    combat.enemies.positions = vec![(20, 20); 100];
    combat.order = case.words("order_before");
    (
        combat,
        support::warriors(&case["warriors_before"]),
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
    let fixture = support::fixture("mission-deaths.json");
    let mission = mission(&fixture);
    let extras = fixture.bytes("extras");
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
        let label = format!("mission death case{index}");
        support::native_expect! {
            records &combat.enemies.records => &case["records_after"], &label;
            records &warriors => &case["warriors_after"], &label;
            words combat.map.tiles => &case["map_after"], &label;
            words combat.order => &case["order_after"], &label;
            scalar unavailable => &case["unavailable_after"], &label;
            scalar result.ended_early => &(case.n::<i64>("return_value") == -1).into(), &label;
        }
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
    let fixture = support::fixture("mission-deaths.json");
    let mission = mission(&fixture);
    let extras = fixture.bytes("extras");
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

#[test]
fn child_roster_retains_native_story_actor_and_saved_inactive_record() {
    let content = content();
    let fixture = support::fixture("mission-deaths.json");
    let case = fixture
        .array("cases")
        .iter()
        .find(|case| {
            case["monster_id"] == 121
                && case["queued"] == 0
                && case["faction"] == 0
                && case["unavailable_before"] == 0
        })
        .unwrap();
    let mut state = support::mission_session(&content);
    let mut parent = support::battle();
    parent.enemies.records = support::monsters(&case.bytes("records_before"))[..4].to_vec();
    parent.enemies.positions = vec![(20, 20); 4];
    state.combat = Some(parent);
    centurions_core::combat::begin_battle(&mut state, &content, 14, 0).unwrap();
    let active = current_combat(&state).enemies.records.clone();
    assert_eq!(active.len(), 1);
    state.combat.as_mut().unwrap().script_enemy = 3;
    let position = state.exploration.as_mut().unwrap();
    position.script = Some(centurions_core::ScriptState::scheduled(
        content.map(0).unwrap().script(337).unwrap().to_vec(),
        1,
    ));
    position.script.as_mut().unwrap().scheduled = true;
    position.presentation = Some(Presentation::Dialogue {
        id: 1248,
        text: content.map(0).unwrap().dialogue(1248).unwrap(),
    });
    support::saved_command(Command::Continue, &mut state, &content);
    let arena = current_combat(&state);
    assert_eq!(arena.enemies.records, active);
    assert_eq!(
        arena.retained_npcs.slots[3].as_ref().unwrap().record.bytes,
        case.bytes("records_after")[3 * MONSTER_BYTES..4 * MONSTER_BYTES]
    );
    support::checkpoint(&state, &content, "inactive-npc-after");
}
