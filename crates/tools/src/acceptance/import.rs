use centurions_content::Content;
use centurions_core::{Command, Session, apply, load, original_save, save, trace};
use serde_json::{Value, json};
use std::{error::Error, fs, path::Path};

/// Imports an original saved game, saves and reloads it to check that nothing
/// changes, then applies the commands in order. Returns a JSON report with each
/// command's events and trace, stopping at the first rejected command. The work
/// folder must be inside the checkout's analysis directory, and an existing
/// round-trip save is never overwritten.
pub(crate) fn run(
    content: &Content,
    original: &Path,
    commands: &Path,
    work: &Path,
) -> Result<Value, Box<dyn Error>> {
    let area = std::env::current_dir()?.join("analysis").canonicalize()?;
    let work = work.canonicalize()?;
    if work == area || !work.starts_with(&area) {
        return Err("audit-import work must be within the checkout's analysis directory".into());
    }
    let roundtrip = work.join("import-roundtrip.json");
    if roundtrip.exists() {
        return Err("audit-import refuses an existing roundtrip save".into());
    }
    let commands: Vec<Command> = serde_json::from_slice(&fs::read(commands)?)?;
    let fresh = Session::new(content);
    let imported = original_save::import(
        original,
        content,
        &fresh.random,
        (fresh.library_random_seed, fresh.library_random_draws),
    );
    let mut state = match imported {
        Ok(state) => state,
        Err(error) => {
            return Ok(json!({"accepted":false,"stage":"import","error":error.to_string()}));
        }
    };
    save(&roundtrip, &state, content)?;
    let restored = load(&roundtrip, content)?;
    let equal = serde_json::to_value(&state)? == serde_json::to_value(&restored)?;
    if !equal {
        return Ok(json!({"accepted":false,"stage":"roundtrip","equal":false}));
    }
    state = restored;
    let mut steps = vec![];
    let mut accepted = true;
    for (index, command) in commands.into_iter().enumerate() {
        let before = state.clone();
        let result = apply(crate::replay::traced(command), &mut state, content);
        let entries = trace::take();
        match result {
            Ok(events) => steps.push(json!({"index":index,"events":events,"trace":entries})),
            Err(error) => {
                accepted = false;
                steps.push(
                    json!({"index":index,"error":error.to_string(),"trace":entries,
                    "state_unchanged":state==before}),
                );
                break;
            }
        }
    }
    Ok(
        json!({"accepted":accepted,"roundtrip_equal":equal,"steps":steps,"session":state,
        "qualification":"Original import and supplied command continuation only; no whole campaign or desktop acceptance."}),
    )
}
