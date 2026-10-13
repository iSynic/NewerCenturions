use centurions_content::{Content, installation::Installation, read_resource};
use std::{collections::BTreeMap, fs, path::Path};

fn snapshot(root: &Path, files: &mut BTreeMap<String, String>) {
    for entry in fs::read_dir(root).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            snapshot(&entry.path(), files);
        } else {
            files.insert(
                entry.path().to_string_lossy().into_owned(),
                centurions_content::digest(&fs::read(entry.path()).unwrap()),
            );
        }
    }
}

#[test]
fn flat_installation_matches_prepared_content_and_does_not_write_originals() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let originals = root.join("NewCenturions");
    let extracted = root.join("analysis/extracted");
    let mut before = BTreeMap::new();
    snapshot(&originals, &mut before);
    let prepared = Content::load(&originals, &extracted).unwrap();
    let flat = Installation::open(&originals.join("TheNewCenturions")).unwrap();
    assert_eq!(flat.content.warriors, prepared.warriors);
    assert_eq!(flat.content.items, prepared.items);
    assert_eq!(flat.content.tutorials, prepared.tutorials);
    for (actual, expected) in flat
        .content
        .scenario
        .as_ref()
        .unwrap()
        .maps
        .iter()
        .zip(&prepared.scenario.as_ref().unwrap().maps)
    {
        assert_eq!(actual.header, expected.header);
        assert_eq!(actual.floor, expected.floor);
        assert_eq!(actual.objects, expected.objects);
        assert_eq!(actual.events, expected.events);
        assert_eq!(actual.shared.prologue, expected.shared.prologue);
    }
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(extracted.join("manifest.json")).unwrap()).unwrap();
    for actual in &flat.content.resources {
        let source = actual.source_fork.as_ref().unwrap();
        let path = format!("TheNewCenturions/{}", source.path);
        let expected = manifest["resources"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["input_path"] == path && r["type"] == actual.r#type && r["id"] == actual.id)
            .unwrap();
        let expected: centurions_content::Resource =
            serde_json::from_value(expected.clone()).unwrap();
        assert_eq!(
            read_resource(&flat.resource_root, actual).unwrap(),
            read_resource(&extracted, &expected).unwrap()
        );
    }
    let again = Installation::open(&originals.join("TheNewCenturions")).unwrap();
    assert_eq!(flat.content.identity, again.content.identity);
    let mut after = BTreeMap::new();
    snapshot(&originals, &mut after);
    assert_eq!(before, after);
}
