pub fn load() -> centurions_content::Content {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    centurions_content::Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap()
}
