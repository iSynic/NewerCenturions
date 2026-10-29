use centurions_content::{Content, graphics::decode_pict, mission::ScenarioContent, read_resource};
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

#[test]
fn level_custom_properties_survive_switching_and_cloned_map_caches() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../NewCenturions");
    let mut custom = vec![0; 16064];
    custom[1] = 155;
    custom[3] = 1;
    let level = ScenarioContent::read_level(
        |path| {
            if path.ends_with("/ BD") {
                assert_eq!(path, "TheNewCenturions/Scenarios/The Thunder of Guns/0/ BD");
                Ok(custom.clone())
            } else {
                std::fs::read(root.join(path)).map_err(Into::into)
            }
        },
        "The Thunder of Guns",
        0,
        Default::default(),
    )
    .unwrap();
    for terrain in [15, 19] {
        let land = level.land(0, terrain).unwrap();
        assert_eq!(
            (land.terrain, land.default_tile, land.minutes_per_tick),
            (terrain, 155, 1)
        );
        assert_eq!(land.tile_word(399, 38).unwrap(), 0);
    }
    let mut other = level.clone();
    std::sync::Arc::make_mut(&mut other.tile_sets)[0].default_tile = 166;
    assert_eq!(other.land(0, 15).unwrap().default_tile, 166);
    assert_eq!(level.land(0, 15).unwrap().default_tile, 155);
}
