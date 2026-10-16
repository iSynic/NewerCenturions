use centurions_content::{Content, digest};
use centurions_core::{Command, RandomState, Session};
use serde::Deserialize;
use std::path::PathBuf;
mod campaign_source;
use campaign_source::source_digest;

mod campaign_assistance;
use campaign_assistance::{Assistance, apply_recorded, assist};

fn main() {
    use std::io::Write;
    use std::time::{Duration, Instant};
    #[derive(Deserialize)]
    struct Request {
        owner: u32,
        id: u64,
        commands: Vec<Command>,
        #[serde(default)]
        finish: bool,
        #[serde(default)]
        assistance: Vec<Assistance>,
    }
    let directory = PathBuf::from(
        std::env::var_os("CENTURIONS_CAMPAIGN_DRIVER").expect("candidate mailbox required"),
    );
    std::fs::create_dir_all(&directory).unwrap();
    let owner = std::process::id();
    let mut log = std::fs::File::create(directory.join(format!("commands-{owner}.jsonl"))).unwrap();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let content = Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap();
    let mut state = Session::new(&content);
    let seed = std::env::var("CENTURIONS_CAMPAIGN_SEED")
        .map(|seed| seed.parse::<u32>().unwrap())
        .unwrap_or(12);
    assert!(matches!(seed, 1 | 12));
    state.random = RandomState::new(seed);
    let mut id = 0;
    let publish = |id: u64, value: serde_json::Value| {
        let temporary = directory.join(format!("response-{id}.tmp"));
        std::fs::write(&temporary, serde_json::to_vec(&value).unwrap()).unwrap();
        std::fs::rename(temporary, directory.join(format!("response-{id}.json"))).unwrap();
    };
    publish(
        id,
        serde_json::json!({
            "owner": owner,
            "id": id,
            "session": state,
            "source_sha256": source_digest(&root),
            "accepted_complete_campaign": false
        }),
    );
    let started = Instant::now();
    while started.elapsed() < Duration::from_secs(600) {
        let Ok(bytes) = std::fs::read(directory.join("request.json")) else {
            std::thread::sleep(Duration::from_millis(25));
            continue;
        };
        let request: Request = serde_json::from_slice(&bytes).unwrap();
        if request.owner != owner || request.id <= id {
            std::thread::sleep(Duration::from_millis(25));
            continue;
        }
        assert_eq!(request.id, id + 1);
        assert!(request.commands.len() <= 200_000);
        id = request.id;
        let mut ordered = vec![];
        let mut error = None;
        for command in request.commands {
            let unassisted_sha256 = digest(&serde_json::to_vec(&state).unwrap());
            assist(&mut state, &request.assistance, &content);
            let context = content
                .context(state.mission.as_deref(), state.scenario_level)
                .unwrap();
            state.validate(context.as_ref()).unwrap();
            let before = state.clone();
            let result = apply_recorded(command.clone(), &mut state, &content);
            let state_sha256 = digest(&serde_json::to_vec(&state).unwrap());
            let row = match result {
                Ok(events) => {
                    let row = serde_json::json!({
                        "command": command,
                        "events": events,
                        "state_sha256": state_sha256,
                        "random": state.random,
                        "assistance": request.assistance,
                        "unassisted_sha256": unassisted_sha256
                    });
                    ordered.push(events);
                    row
                }
                Err(failure) => {
                    assert_eq!(state, before, "failed candidate command changed state");
                    error = Some(failure.to_string());
                    serde_json::json!({
                        "command": command,
                        "error": error,
                        "state_sha256": state_sha256,
                        "random": state.random
                    })
                }
            };
            serde_json::to_writer(&mut log, &row).unwrap();
            writeln!(log).unwrap();
            if error.is_some() {
                break;
            }
        }
        log.flush().unwrap();
        publish(
            id,
            serde_json::json!({
                "owner": owner,
                "id": id,
                "session": state,
                "events": ordered,
                "error": error,
                "accepted_complete_campaign": false
            }),
        );
        if request.finish {
            return;
        }
    }
    panic!("candidate recording exceeded its owned time limit");
}
