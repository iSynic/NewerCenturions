use serde_json::Value;

#[path = "../hex.rs"]
mod hex;
pub use hex::decode;

pub fn bytes(value: &Value) -> Vec<u8> {
    decode(value.as_str().unwrap())
}

pub fn fixture(name: &str) -> Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../evidence/fixtures")
        .join(name);
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

pub fn content() -> centurions_content::Content {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    centurions_content::Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap()
}

/// A command applied to a saved copy and to the live state must agree.
pub fn command_resumed(
    path: &std::path::Path,
    state: &mut crate::Session,
    content: &centurions_content::Content,
    command: crate::Command,
) -> Vec<crate::Event> {
    crate::save(path, state, content).unwrap();
    let mut resumed = crate::load(path, content).unwrap();
    let events = crate::apply(command.clone(), state, content).unwrap();
    assert_eq!(
        crate::apply(command, &mut resumed, content).unwrap(),
        events
    );
    assert_eq!(*state, resumed);
    events
}
