use super::*;
#[path = "../native/audio_reference.rs"]
mod native_audio;

#[cfg(feature = "original-fixtures")]
#[path = "../../../core/tests/hex.rs"]
mod hex;
#[cfg(feature = "original-fixtures")]
use hex::decode as hex_bytes;

#[cfg(feature = "original-fixtures")]
#[test]
fn mace3_rejects_unsupported_headers_and_bounded_payload_errors() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../evidence/fixtures/sound-mace3.json"
    ))
    .unwrap();
    let original = hex_bytes(fixture["cases"][0]["input"].as_str().unwrap());
    for (offset, value) in [
        (20, 1i32),
        (24, 2),
        (32, 1),
        (36, 1),
        (42, 0),
        (42, -1),
        (42, i32::MAX),
        (42, 16_000_000 / 6 + 1),
        (56, 1),
        (60, 1),
        (64, 1),
        (68, 1),
        (72, 1),
    ] {
        let mut bytes = original.clone();
        bytes[offset..offset + 4].copy_from_slice(&value.to_be_bytes());
        assert!(
            decode_snd(&bytes).is_err(),
            "long offset {offset} value {value}"
        );
    }
    for (offset, value) in [(76, 4i16), (76, -1), (78, 8), (78, 0), (82, 16)] {
        let mut bytes = original.clone();
        bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
        assert!(
            decode_snd(&bytes).is_err(),
            "word offset {offset} value {value}"
        );
    }
    for rate in [0u32, 499 * 65536, 64001 * 65536, u32::MAX] {
        let mut bytes = original.clone();
        bytes[28..32].copy_from_slice(&rate.to_be_bytes());
        assert!(decode_snd(&bytes).is_err());
    }
    for length in [41, 83, original.len() - 2] {
        assert!(decode_snd(&original[..length]).is_err());
    }
    let mut no_padding = original.clone();
    assert_eq!(no_padding.pop(), Some(0));
    assert_eq!(
        decode_snd(&no_padding).unwrap().samples,
        decode_snd(&original).unwrap().samples
    );
    let mut extra = original.clone();
    extra.push(0);
    assert!(decode_snd(&extra).is_err());
    let mut padding = original.clone();
    *padding.last_mut().unwrap() = 0x80;
    assert_eq!(
        decode_snd(&padding).unwrap().samples,
        decode_snd(&original).unwrap().samples
    );
}

#[test]
fn rejects_truncated_command_and_sample_headers() {
    assert!(decode_snd(&[]).is_err());
    assert!(decode_snd(&[0; 20]).is_err());
    let mut bytes = vec![0; 42];
    for (offset, value) in [(0, 1u16), (2, 1), (4, 5), (10, 1), (12, 0x8051)] {
        bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
    }
    bytes[16..20].copy_from_slice(&20i32.to_be_bytes());
    bytes[24..28].copy_from_slice(&100i32.to_be_bytes());
    bytes[28..32].copy_from_slice(&(11025i32 * 65536).to_be_bytes());
    assert!(decode_snd(&bytes).is_err());
}

#[cfg(feature = "original-fixtures")]
#[test]
fn supplied_format_two_sounds_preserve_samples() {
    for mut bytes in native_audio::compare_format_two() {
        bytes.pop();
        assert!(decode_snd(&bytes).is_err());
    }
}
