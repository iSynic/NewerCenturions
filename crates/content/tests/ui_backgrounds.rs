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
}
