use centurions_content::{Content, graphics::decode_pict, read_resource};
use std::path::Path;

#[test]
fn supplied_startup_and_squad_pictures_decode_at_their_original_dimensions() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let extracted = root.join("analysis/extracted");
    let content = Content::load(&root.join("NewCenturions"), &extracted).unwrap();
    for (id, width, height) in [
        (19, 640, 460),
        (187, 640, 460),
        (203, 640, 460),
        (132, 50, 50),
        (1, 60, 50),
        (2, 70, 35),
        (4, 70, 35),
        (214, 640, 460),
        (182, 320, 84),
        (7, 50, 50),
        (158, 50, 50),
        (160, 50, 50),
        (168, 50, 50),
        (179, 13, 13),
        (183, 50, 50),
        (216, 70, 35),
        (148, 640, 73),
        (170, 114, 387),
        (175, 50, 37),
        (226, 800, 120),
        (227, 80, 73),
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
        let bytes = read_resource(&extracted, resource).unwrap();
        let image = decode_pict(&bytes).unwrap();
        assert_eq!((image.width, image.height), (width, height), "PICT {id}");
        assert_eq!(
            image.rgba.len(),
            usize::from(width) * usize::from(height) * 4
        );
        assert!(
            image
                .rgba
                .as_chunks::<4>()
                .0
                .iter()
                .all(|pixel| pixel[3] == 255)
        );
    }
    // Newly connected dialogs must not make desktop startup fail on their original pictures.
    for dialog in [128, 131, 132, 133, 136, 158, 161, 167, 177, 190] {
        let resource = content
            .resources
            .iter()
            .find(|r| r.r#type == "DITL" && r.id == dialog && r.original.ends_with("/Cargo Pod"))
            .unwrap();
        for item in
            centurions_content::ui::dialog_items(&read_resource(&extracted, resource).unwrap())
                .unwrap()
        {
            if item.kind == 64 && item.data.len() == 2 {
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
