use centurions_content::{Content, graphics::decode_pict, read_resource};
use std::path::Path;

#[test]
fn supplied_startup_and_dialog_pictures_decode_without_blocking_startup() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let extracted = root.join("analysis/extracted");
    let content = Content::load(&root.join("NewCenturions"), &extracted).unwrap();
    // Independent full-raster cases own pixel/dimension fidelity; this checks startup admission.
    for id in [
        153, 156, 161, 176, 19, 187, 203, 132, 1, 7, 158, 160, 168, 179, 183, 216, 148, 170, 175,
        226, 227,
    ] {
        let resource = content
            .resources
            .iter()
            .find(|resource| {
                resource.r#type == "PICT"
                    && resource.id == id
                    && resource.original == "TheNewCenturions/Data Files/Cargo Pod"
            })
            .unwrap();
        decode_pict(&read_resource(&extracted, resource).unwrap()).unwrap();
    }
    // Newly connected dialogs must not make desktop startup fail on their original pictures.
    for dialog in [128, 131, 132, 133, 136, 158, 161, 166, 167, 177, 190] {
        let resource = content
            .resources
            .iter()
            .find(|r| r.r#type == "DITL" && r.id == dialog && r.original.ends_with("/Cargo Pod"))
            .unwrap();
        for item in
            centurions_content::ui::dialog_items(&read_resource(&extracted, resource).unwrap())
                .unwrap()
        {
            if item.kind & 0x7f == 64 && item.data.len() == 2 {
                let id = i16::from_be_bytes(item.data[..2].try_into().unwrap());
                if id > 0 {
                    let resource = content
                        .resources
                        .iter()
                        .find(|r| {
                            r.r#type == "PICT" && r.id == id && r.original.ends_with("/Cargo Pod")
                        })
                        .unwrap();
                    decode_pict(&read_resource(&extracted, resource).unwrap()).unwrap();
                }
            }
        }
    }
    for resource in content.resources.iter().filter(|resource| {
        resource.r#type == "PICT"
            && resource.original.contains("/Scenarios/")
            && resource.original.ends_with("/Scenario")
    }) {
        decode_pict(&read_resource(&extracted, resource).unwrap()).unwrap();
    }
}
