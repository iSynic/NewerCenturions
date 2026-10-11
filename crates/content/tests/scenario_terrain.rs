use centurions_content::{Content, graphics::decode_pict, read_resource};
use std::path::Path;

#[test]
fn supplied_terrain_pixmaps_decode_to_the_original_picture_frame() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let extracted = root.join("analysis/extracted");
    let content = Content::load(&root.join("NewCenturions"), &extracted).unwrap();
    let mut ids = Vec::new();
    for resource in content.resources.iter().filter(|r| {
        r.r#type == "PICT" && (300..=308).contains(&r.id) && r.original.ends_with("/Cargo Pod")
    }) {
        let source = read_resource(&extracted, resource).unwrap();
        let image = decode_pict(&source).unwrap();
        let frame_height = centurions_content::word(&source, 6).unwrap()
            - centurions_content::word(&source, 2).unwrap();
        let frame_width = centurions_content::word(&source, 8).unwrap()
            - centurions_content::word(&source, 4).unwrap();
        assert_eq!(
            (image.width, image.height),
            (frame_width as u16, frame_height as u16)
        );
        assert_eq!(
            image.rgba.len(),
            usize::from(image.width) * usize::from(image.height) * 4
        );
        assert!(
            image
                .rgba
                .as_chunks::<4>()
                .0
                .iter()
                .all(|pixel| pixel[3] == 255)
        );
        ids.push(resource.id);
    }
    ids.sort();
    assert_eq!(ids, [300, 301, 302, 303, 304, 307, 308]);
}
