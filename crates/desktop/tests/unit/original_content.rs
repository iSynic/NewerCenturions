// Desktop tests share only the source-frame builder.
#[allow(dead_code)]
#[path = "../../../core/tests/support/native_context.rs"]
mod native_context;
pub use native_context::source_frame;

pub fn load() -> centurions_content::Content {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    centurions_content::Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap()
}

pub fn fixture(name: &str) -> serde_json::Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../evidence/fixtures")
        .join(name);
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

/// Shared original mission setup for desktop command workflows.
pub fn enter_mission(state: &mut centurions_core::Session, content: &centurions_content::Content) {
    for command in [
        centurions_core::Command::ImportStarters,
        centurions_core::Command::EnterMission {
            name: "The Thunder of Guns".into(),
        },
    ] {
        centurions_core::apply(command, state, content).unwrap();
    }
}
