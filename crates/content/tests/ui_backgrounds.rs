use centurions_content::{Content, Resource, graphics::decode_pict, read_resource};
use std::path::Path;

#[test]
fn supplied_startup_and_dialog_pictures_decode_without_blocking_startup() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let extracted = root.join("analysis/extracted");
    let content = Content::load(&root.join("NewCenturions"), &extracted).unwrap();
    // Independent full-raster cases own pixel/dimension fidelity; this checks startup admission.
    for id in [
        153, 156, 161, 176, 19, 187, 203, 132, 1, 7, 158, 160, 168, 179, 183, 216, 148, 170, 175,
        226, 227, -2101,
    ] {
        let picture =
            |r: &&Resource| r.r#type == "PICT" && r.id == id && r.original.ends_with("/Cargo Pod");
        let resource = content.resources.iter().find(picture).unwrap();
        decode_pict(&read_resource(&extracted, resource).unwrap()).unwrap();
    }
}
