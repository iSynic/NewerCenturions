use centurions_content::{Content, digest, graphics::decode_pict, read_resource};
use std::path::Path;

#[test]
fn supplied_dialog_pictures_match_independent_full_raster_expansion() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let extracted = root.join("analysis/extracted");
    let content = Content::load(&root.join("NewCenturions"), &extracted).unwrap();
    // Expected rasters were independently expanded from the original PackBits bytes in Python.
    for (id, width, height, source_hash, rgba_hash) in [
        (
            2,
            70,
            35,
            "09d26ec7041be650cc3650df3f47f92c43c4995645579fff2c7bdaa0efc1edf1",
            "3920f460195265ece295899d8b21d6d0466ef2105e62bc239c367e78eede78ea",
        ),
        (
            4,
            70,
            35,
            "ff1afc3f6a2e888638c935829db92643f418f295898673681afae7631cad7624",
            "20c5f7d10d64bb8c54b5aeec687ebb2c655cb2c900fed442ae16175150f4ba60",
        ),
        (
            182,
            320,
            84,
            "15d33a36a42f18ef5dfacde53ee63a0a7f5164a0c0f5562b792fb080c3f6474b",
            "33e5d0f96574ce5b082068ab7a26e37e1ed00bec93b49a6fada9da7fb606151d",
        ),
        (
            192,
            206,
            129,
            "be46b01e0fa4d62d2df06cc65a91d1405c194f4d74285f3b1eb6b868ebaed4e8",
            "4d13551a082d95586f51307fe1cf0c281a686003fea137f8d0d076154846fa80",
        ),
        (
            214,
            640,
            460,
            "431fce6b18ef0316ec939b782cee5cbf5d6090c9743cf869a2cfea64f98871ff",
            "ab4ab49fa282d6ffc2b1cf514a4f3f0726467c7a1aa1a2f1963723c5f70e6f05",
        ),
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
        assert_eq!(digest(&bytes), source_hash);
        let image = decode_pict(&bytes).unwrap();
        assert_eq!((image.width, image.height), (width, height));
        assert_eq!(digest(&image.rgba), rgba_hash, "PICT {id}");
    }
}

fn four_bit_picture() -> Vec<u8> {
    let frame = [0, 0, 0, 0, 0, 1, 0, 15];
    let mut bytes = vec![0, 0];
    bytes.extend_from_slice(&frame);
    bytes.extend_from_slice(&[0, 0x11, 2, 0xff, 0, 0x98]);
    let mut pixmap = [0; 46];
    pixmap[..2].copy_from_slice(&0x8008_u16.to_be_bytes());
    pixmap[2..10].copy_from_slice(&frame);
    for (offset, value) in [(28, 4_i16), (30, 1), (32, 4)] {
        pixmap[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
    }
    bytes.extend_from_slice(&pixmap);
    bytes.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 5]);
    for index in 0_u16..6 {
        bytes.extend_from_slice(&index.to_be_bytes());
        for component in [index * 257, index * 514, index * 771] {
            bytes.extend_from_slice(&component.to_be_bytes());
        }
    }
    bytes.extend_from_slice(&frame);
    bytes.extend_from_slice(&frame);
    bytes.extend_from_slice(&[0, 0, 9, 7, 0x12, 0x34, 0x50, 0x12, 0x34, 0x50, 0x12, 0x3f]);
    bytes.extend_from_slice(&[0, 0xff]);
    bytes
}

#[test]
fn four_bit_pixels_use_high_nibble_first_and_ignore_row_padding() {
    let image = decode_pict(&four_bit_picture()).unwrap();
    assert_eq!((image.width, image.height), (15, 1));
    let expected: Vec<u8> = [1_u8, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 0, 1, 2, 3]
        .into_iter()
        .flat_map(|index| [index, index * 2, index * 3, 255])
        .collect();
    assert_eq!(image.rgba, expected);
}

#[test]
fn unsupported_four_bit_layouts_and_malformed_rows_return_diagnostics() {
    let bytes = four_bit_picture();
    for end in [0, 14, 16, 61, 69, 125, bytes.len() - 1] {
        assert!(decode_pict(&bytes[..end]).is_err());
    }
    for (offset, value) in [(28, 1_i16), (30, 2), (32, 2), (12, 1)] {
        let mut invalid = bytes.clone();
        invalid[16 + offset..18 + offset].copy_from_slice(&value.to_be_bytes());
        assert!(decode_pict(&invalid).is_err());
    }
    let mut short_row = bytes.clone();
    short_row[16..18].copy_from_slice(&0x8007_u16.to_be_bytes());
    assert!(decode_pict(&short_row).is_err());
    let mut missing_index = bytes.clone();
    missing_index[138] = 0x62;
    assert!(
        decode_pict(&missing_index)
            .unwrap_err()
            .to_string()
            .contains("missing PICT palette index")
    );
    let mut overflow = bytes;
    overflow[137] = 0xf8;
    assert!(
        decode_pict(&overflow)
            .unwrap_err()
            .to_string()
            .contains("PICT PackBits row overflow")
    );
}
