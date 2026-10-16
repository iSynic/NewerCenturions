use centurions_content::Content;
use std::path::Path;

fn bytes(value: &serde_json::Value) -> Vec<u8> {
    let text = value.as_str().unwrap();
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect()
}

pub fn write(directory: &Path, _content: &Content) -> (centurions_core::Session, Vec<u8>, Vec<u8>) {
    let receipt: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../evidence/fixtures/native-save.json"
    ))
    .unwrap();
    let original: centurions_core::Session =
        serde_json::from_value(receipt["session"].clone()).unwrap();
    let info = bytes(&receipt["data_i1"]);
    let files: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../evidence/fixtures/native-game-files.json"
    ))
    .unwrap();
    let maps = bytes(&files["files"]["output/0/CL"]);
    let index = original.exploration.as_ref().unwrap().map as usize;
    let native = maps[index * 77744..(index + 1) * 77744].to_vec();
    std::fs::write(directory.join("Data I1"), &info).unwrap();
    std::fs::write(directory.join("Data A1"), &maps).unwrap();
    for (name, native) in [
        ("Data TD3", "CTD3"),
        ("Data F1", "CE"),
        ("Data G1", "CE2"),
        ("Data E1", "CS"),
    ] {
        std::fs::write(
            directory.join(name),
            bytes(&files["files"][format!("output/0/{native}")]),
        )
        .unwrap();
    }
    (original, info, native)
}

pub fn remove(directory: &Path) {
    for name in [
        "Data I1", "Data A1", "Data TD3", "Data F1", "Data G1", "Data E1",
    ] {
        std::fs::remove_file(directory.join(name)).unwrap();
    }
}
