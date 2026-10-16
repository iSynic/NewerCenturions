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
