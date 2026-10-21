#[path = "../../../core/tests/support/native_context.rs"]
mod native_context;
pub use native_context::legacy_save;

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
