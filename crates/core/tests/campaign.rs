use crate::prelude::*;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::PathBuf;

#[derive(Deserialize)]
struct Suite {
    verification_kind: String,
    content_identity: String,
    source_sha256: String,
    routes: Vec<Route>,
}

#[derive(Deserialize)]
struct Route {
    seed: u32,
    difficulty: i16,
    commands: PathBuf,
    commands_sha256: String,
    events_sha256: String,
    session_sha256: String,
    #[serde(default)]
    assistance: Vec<AssistedStep>,
}

#[derive(Deserialize)]
struct AssistedStep {
    command_index: usize,
    settings: Vec<campaign_assistance::Assistance>,
}

#[path = "support/campaign_assistance.rs"]
mod campaign_assistance;

#[path = "support/campaign_source.rs"]
mod campaign_source;
use campaign_source::source_digest;

fn replay(
    route: &Route,
    commands: &[Command],
    content: &Content,
    checkpoints: bool,
) -> (Session, Vec<Vec<Event>>) {
    let mut state = Session::new(content);
    state.random = RandomState::new(route.seed);
    apply(
        Command::SetDifficulty {
            level: route.difficulty,
        },
        &mut state,
        content,
    )
    .unwrap();
    let mut stages = BTreeSet::new();
    let mut conclusion = false;
    let mut reset = false;
    let mut ordered = vec![];
    for (index, command) in commands.iter().enumerate() {
        if let Some(step) = route
            .assistance
            .iter()
            .rev()
            .find(|step| step.command_index <= index)
        {
            campaign_assistance::assist(&mut state, &step.settings, content);
        }
        let before = state.clone();
        let events = match campaign_assistance::apply_recorded(command.clone(), &mut state, content)
        {
            Ok(events) => events,
            Err(error) => {
                assert_eq!(state, before, "campaign failure changed the session");
                let output = crate::support::root().join("analysis/thunder-completion");
                std::fs::create_dir_all(&output).unwrap();
                let failure = serde_json::json!({"seed":route.seed,"difficulty":route.difficulty,"index":index,"command":command,"error":error.to_string(),"state":state,"ordered_events":ordered});
                std::fs::write(
                    output.join(format!("frontier-{}-{}.json", route.seed, route.difficulty)),
                    serde_json::to_vec(&failure).unwrap(),
                )
                .unwrap();
                panic!(
                    "campaign seed{} difficulty{} command{index}: {error}",
                    route.seed, route.difficulty
                );
            }
        };
        if state.phase == Phase::Exploration {
            stages.insert(state.mission_stage);
        }
        conclusion |= events
            .iter()
            .any(|event| matches!(event, Event::Message { id: 1326, .. }))
            || state.exploration.as_ref().is_some_and(|position| {
                matches!(
                    position.presentation,
                    Some(Presentation::Dialogue { id: 1326, .. })
                )
            });
        reset |= conclusion && before.mission_stage == 5 && state.mission_stage == 0;
        if checkpoints
            && (index % 128 == 0
                || state.mission_stage != before.mission_stage
                || state.medical_continuation != before.medical_continuation
                || state.time_continuation != before.time_continuation
                || state.psi_selection != before.psi_selection
                || state.outdoor_psi != before.outdoor_psi
                || state.encounter_actions != before.encounter_actions
                || state.promotion != before.promotion
                || state.survivors != before.survivors
                || state.round_dispatch != before.round_dispatch
                || state.scripted_cleanup != before.scripted_cleanup
                || state
                    .combat
                    .as_ref()
                    .map(|c| (c.selection_pending, c.script_enemy))
                    != before
                        .combat
                        .as_ref()
                        .map(|c| (c.selection_pending, c.script_enemy))
                || state.combat.as_ref().map(|c| &c.round_scripts.pending)
                    != before.combat.as_ref().map(|c| &c.round_scripts.pending)
                || state.exploration.as_ref().map(|p| &p.presentation)
                    != before.exploration.as_ref().map(|p| &p.presentation))
        {
            state = crate::support::checkpoint(&state, content, "campaign");
        }
        ordered.push(events);
    }
    let report = serde_json::json!({"seed":route.seed,"difficulty":route.difficulty,"state":state,"ordered_events":ordered,"stages":stages,"conclusion":conclusion,"reset":reset});
    std::fs::write(
        crate::support::root().join(format!(
            "analysis/thunder-completion/prefix-{}-{}.json",
            route.seed, route.difficulty
        )),
        serde_json::to_vec(&report).unwrap(),
    )
    .unwrap();
    assert_eq!(stages, BTreeSet::from([0, 1, 2, 3, 4, 5]));
    assert!(
        conclusion && reset && state.phase == Phase::Exploration,
        "campaign did not reach the original conclusion and mission-one reset"
    );
    (state, ordered)
}

#[test]
#[ignore = "requires current campaign recordings with explicit assistance receipts; enforced by ExtendedScenarios and local packaging"]
fn complete_campaign_replays() {
    let root = crate::support::root();
    let path = std::env::var_os("CENTURIONS_CAMPAIGN_MANIFEST")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("analysis/thunder-completion/campaign-manifest.json"));
    let suite: Suite =
        serde_json::from_slice(&std::fs::read(&path).expect("missing campaign manifest")).unwrap();
    assert_eq!(suite.verification_kind, "assisted_functional");
    let content = content();
    assert_eq!(suite.content_identity, content.identity);
    assert_eq!(suite.source_sha256, source_digest(&root));
    assert_eq!(suite.routes.len(), 6);
    let actual: BTreeSet<_> = suite
        .routes
        .iter()
        .map(|route| (route.difficulty, route.seed))
        .collect();
    assert_eq!(
        actual,
        BTreeSet::from([(0, 12), (0, 1), (-2, 12), (-1, 12), (1, 12), (2, 12)])
    );
    for route in suite.routes {
        let bytes = std::fs::read(path.parent().unwrap().join(&route.commands)).unwrap();
        assert_eq!(digest(&bytes), route.commands_sha256);
        let commands: Vec<Command> = serde_json::from_slice(&bytes).unwrap();
        assert!(!commands.is_empty() && commands.len() <= 200_000);
        let (state, events) = replay(&route, &commands, &content, false);
        let resumed = replay(&route, &commands, &content, true);
        assert_eq!(state, resumed.0);
        assert_eq!(events, resumed.1);
        // Key-sorted, so moving a field between flattened owners keeps the hash. The
        // projection omits schema13 fields; full state and suffixes are compared above.
        let hash = |value: serde_json::Value| digest(&serde_json::to_vec(&value).unwrap());
        assert_eq!(
            hash(serde_json::to_value(&events).unwrap()),
            route.events_sha256
        );
        let session = serde_json::to_value(crate::support::captured_session(&state)).unwrap();
        assert_eq!(hash(session), route.session_sha256);
    }
}
