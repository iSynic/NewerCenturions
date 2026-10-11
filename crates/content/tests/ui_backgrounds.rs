use centurions_content::{Content, graphics::decode_pict, read_resource, span};
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

#[test]
fn supplied_textile_unicode_cmap_contains_the_apple_glyph() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let extracted = root.join("analysis/extracted");
    let content = Content::load(&root.join("NewCenturions"), &extracted).unwrap();
    let resource = content
        .resources
        .iter()
        .find(|resource| {
            resource.r#type == "sfnt"
                && resource.id == 30600
                && resource.original == "TheNewCenturions/Data Files/Cargo Pod"
        })
        .unwrap();
    let bytes = read_resource(&extracted, resource).unwrap();
    let u16_at = |offset| u16::from_be_bytes(span(&bytes, offset, 2).unwrap().try_into().unwrap());
    let u32_at = |offset| u32::from_be_bytes(span(&bytes, offset, 4).unwrap().try_into().unwrap());
    let directory = (0..usize::from(u16_at(4)))
        .map(|index| 12 + index * 16)
        .find(|&offset| span(&bytes, offset, 4).unwrap() == b"cmap")
        .unwrap();
    let cmap = u32_at(directory + 8) as usize;
    let unicode = (0..usize::from(u16_at(cmap + 2)))
        .map(|index| cmap + 4 + index * 8)
        .find(|&offset| u16_at(offset) == 0 && u16_at(offset + 2) == 3)
        .unwrap();
    let subtable = cmap + u32_at(unicode + 4) as usize;
    assert_eq!(u16_at(subtable), 4);
    let segments = usize::from(u16_at(subtable + 6)) / 2;
    let end_codes = subtable + 14;
    let start_codes = end_codes + 2 * segments + 2;
    let deltas = start_codes + 2 * segments;
    let range_offsets = deltas + 2 * segments;
    let glyph = |code: u16| {
        let index = (0..segments)
            .find(|&index| {
                u16_at(start_codes + 2 * index) <= code && code <= u16_at(end_codes + 2 * index)
            })
            .unwrap();
        let delta = u16_at(deltas + 2 * index);
        let range = u16_at(range_offsets + 2 * index);
        if range == 0 {
            code.wrapping_add(delta)
        } else {
            let address = range_offsets
                + 2 * index
                + usize::from(range)
                + 2 * usize::from(code - u16_at(start_codes + 2 * index));
            match u16_at(address) {
                0 => 0,
                id => id.wrapping_add(delta),
            }
        }
    };
    assert_eq!(glyph(0xf8ff), 210);
    assert_eq!(glyph(0x14), 0);
}
