use crate::prelude::*;

use centurions_core::combat::*;

fn session(content: &Content) -> Session {
    let mut state = Session::new(content);
    apply(Command::ImportStarters, &mut state, content).unwrap();
    apply(
        Command::EnterMission {
            name: "The Thunder of Guns".into(),
        },
        &mut state,
        content,
    )
    .unwrap();
    state.random = centurions_core::RandomState::new(12);
    let p = state.exploration.as_mut().unwrap();
    p.x = 43;
    p.y = 84;
    p.script = Some(ScriptState {
        record: content.map(0).unwrap().script(6).unwrap().to_vec(),
        cursor: 0,
        map_slot: 0,
        scheduled: true,
        menu: None,
        persist: false,
        origin: None,
    });
    state
}
fn fixture() -> serde_json::Value {
    crate::support::fixture("opening-stories.json")
}

#[test]
fn leader_death_stories_match_original_complete_enemy_map_and_order_records() {
    let content = content();
    for case in fixture()["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["script"] != 6)
    {
        let mut state = session(&content);
        state.phase = Phase::Combat;
        state.warriors[0].bytes[0x40..0x42].copy_from_slice(&4_i16.to_be_bytes());
        let positions = words(case.text("positions"))
            .as_chunks::<2>()
            .0
            .iter()
            .map(|v| (v[0], v[1]))
            .collect();
        state.combat = Some(Combat {
            battle: 1,
            map: TacticalMap {
                tiles: words(case.text("map_before")),
                shift: (0, 0),
                edge_word: 0,
            },
            squad: SquadPlacement {
                positions: vec![(1, 1); 6],
                under_tiles: vec![1155; 6],
            },
            enemies: EnemySetup {
                records: crate::support::monsters(&case["enemies"]),
                positions,
            },
            order: words(case.text("order_before")),
            ..crate::support::battle()
        });
        state.combat_story = Some(runtime::DeathStory {
            body: 0,
            script: case.n::<i16>("script"),
            cursor: 0,
            actor: 0,
            entry: centurions_core::combat::death_scripts::DeathEntry {
                x: i32::from(state.combat.as_ref().unwrap().enemies.positions[0].0) + 5,
                y: i32::from(state.combat.as_ref().unwrap().enemies.positions[0].1) + 5,
                ..Default::default()
            },
            ..Default::default()
        });
        state.handled_deaths.push(0);
        runtime::continue_death(&mut state, &content).unwrap();
        let p = state.exploration.as_ref().unwrap();
        assert!(
            matches!(&p.presentation,Some(Presentation::Dialogue { id, .. }) if *id==case["texts"][0].as_i64().unwrap() as i16)
        );
        let path = std::env::temp_dir().join(format!(
            "centurions-death-{}-{}-{}.json",
            std::process::id(),
            case["script"],
            case["variant"]
        ));
        save(&path, &state, &content).unwrap();
        let mut restored = load(&path, &content).unwrap();
        std::fs::remove_file(path).unwrap();
        let events = apply(Command::Continue, &mut state, &content).unwrap();
        assert_eq!(
            apply(Command::Continue, &mut restored, &content).unwrap(),
            events
        );
        assert_eq!(state, restored);
        assert_eq!(events, vec![Event::Sound { resource_id: 409 }]);
        let c = state.combat.as_ref().unwrap();
        crate::support::native_expect! {
        bytes crate::support::record_bytes(&c.enemies.records) => &case["after_enemies"], format!("script {} variant {}",case["script"],case["variant"]);
        words &(c.map.tiles) => &case["map"], "binary_opening_stories";
        words &(c.order) => &case["order"], "binary_opening_stories";
        }
        assert_eq!(state.random.seed, 12);
        assert_eq!(state.random.draws, 0);
        assert!(state.combat_story.is_none());
        assert_eq!(
            state
                .exploration
                .as_ref()
                .unwrap()
                .script
                .as_ref()
                .unwrap()
                .cursor,
            0
        );
    }
}

#[test]
fn peaceful_camp_choice_teleport_and_cancel_match_original_newland() {
    let content = content();
    let fixture = fixture();
    let case = fixture
        .array("cases")
        .iter()
        .find(|c| c["script"] == 6)
        .unwrap();
    let mut state = session(&content);
    state.exploration.as_mut().unwrap().presentation = Some(Presentation::Dialogue {
        id: 0,
        text: String::new(),
    });
    let mut texts = vec![];
    let mut sounds = vec![];
    let mut finished = false;
    for _ in 0..100 {
        let p = state.exploration.as_ref().unwrap();
        let command = match p.presentation.as_ref().unwrap() {
            Presentation::Question { .. } => Command::Choose { answer: true },
            Presentation::Menu { .. } => Command::CancelMenu,
            _ => Command::Continue,
        };
        let events = apply(command, &mut state, &content).unwrap();
        for event in events {
            if let Event::Sound { resource_id } = event {
                sounds.push(resource_id);
            }
        }
        let p = state.exploration.as_ref().unwrap();
        if let Some(Presentation::Dialogue { id, .. }) = p.presentation {
            texts.push(id);
        }
        if p.presentation.is_none() {
            finished = true;
            break;
        }
    }
    assert!(finished);
    assert_eq!(serde_json::json!(texts), case["texts"]);
    assert_eq!(serde_json::json!(sounds), case["sounds"]);
    let p = state.exploration.as_ref().unwrap();
    assert_eq!((p.x, p.y), (50, 82));
    assert_eq!(p.region_state, case.bytes("regions"));
    assert_eq!(state.random.seed, 12);
    assert_eq!(state.random.draws, 0);
    assert!(p.script.is_none());
}
