use crate::{prelude::*, support};

fn session(content: &Content) -> Session {
    let mut state = support::mission_session(content);
    state.random = centurions_core::RandomState::new(12);
    let p = state.exploration.as_mut().unwrap();
    p.x = 43;
    p.y = 84;
    p.script = Some(centurions_core::ScriptState::scheduled(
        content.map(0).unwrap().script(6).unwrap().to_vec(),
        0,
    ));
    state
}

#[test]
fn leader_death_stories_match_original_complete_enemy_map_and_order_records() {
    let content = content();
    for case in support::fixture("opening-stories.json")["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["script"] != 6)
    {
        let mut state = session(&content);
        state.phase = Phase::Combat;
        state.warriors[0].bytes[0x40..0x42].copy_from_slice(&4_i16.to_be_bytes());
        let positions = support::positions(case.text("positions"));
        let mut health = centurions_core::story_health::context(&state.warriors, 0, None);
        health.last_body = 10; // VerifyOpeningStories sets native DAT408574 to body10.
        state.native_work.health_context = health;
        let mut combat = support::battle();
        combat.battle = 1;
        combat.map.tiles = words(case.text("map_before"));
        combat.squad.positions.fill((1, 1));
        combat.enemies.records = support::monsters(&case["enemies"]);
        combat.enemies.positions = positions;
        combat.order = words(case.text("order_before"));
        state.combat = Some(combat);
        crate::native_common::source_victory_context(&mut state);
        state.combat_story = Some(runtime::DeathStory {
            body: 0,
            script: case.n::<i16>("script"),
            cursor: 0,
            actor: 0,
            entry: centurions_core::combat::death_scripts::DeathEntry {
                x: i32::from(current_combat(&state).enemies.positions[0].0) + 5,
                y: i32::from(current_combat(&state).enemies.positions[0].1) + 5,
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
        assert_eq!(support::event_sounds(&events), case.n::<Vec<i16>>("sounds"));
        let c = current_combat(&state);
        support::native_expect! {
            records &c.enemies.records => &case["after_enemies"], format!("script {} variant {}",case["script"],case["variant"]);
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
    let fixture = support::fixture("opening-stories.json");
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
