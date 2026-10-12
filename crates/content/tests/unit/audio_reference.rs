use super::*;

#[test]
fn original_ima4_music_matches_independent_ffmpeg_pcm_and_rejects_corruption() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let installation =
        crate::installation::Installation::open(&root.join("NewCenturions/TheNewCenturions"))
            .unwrap();
    // FFmpeg ADPCM_IMA_QT, complete declared packets, stereo s16le output.
    for (id, size, digest) in [
        (
            20128,
            7454976,
            "2b4c9737809c3c92535c01c17662462ae0e065adcaa01bf64e57faa3bbfc1e12",
        ),
        (
            20130,
            8662784,
            "29d85dc137a151b06965aae4a0fbcd5961389c46fb44c86816bfd4d5c2b887ef",
        ),
        (
            20137,
            3362304,
            "bba6dc8ea9f445e5b1da72feab26c5a72f7af22a9616fd4b867caf3afb92ac95",
        ),
        (
            20138,
            7321344,
            "0034e9d3c67d8f4e81b3bacede4d37eb3c6a7bd80446756e79c41c74c5553772",
        ),
        (
            20140,
            8849664,
            "6b67b5c594439acef2998cf4a3f280c4ccb0e59319787acdac8d83307198dae1",
        ),
    ] {
        let resource = installation
            .content
            .resources
            .iter()
            .find(|r| r.r#type == "snd " && r.id == id)
            .unwrap();
        let bytes = crate::read_resource(&installation.resource_root, resource).unwrap();
        let sample = decode_snd(&bytes).unwrap();
        assert_eq!(sample.samples.len(), size);
        assert_eq!(crate::digest(&sample.samples), digest, "track {id}");
        assert_eq!(&sample.wave()[22..24], &2_u16.to_le_bytes());
        assert!(decode_snd(&bytes[..bytes.len() - 68]).is_err());
        let mut corrupt = bytes;
        corrupt[85] = 127;
        assert!(decode_snd(&corrupt).is_err());
    }
}

#[cfg(feature = "original-fixtures")]
fn hex_bytes(text: &str) -> Vec<u8> {
    (0..text.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&text[index..index + 2], 16).unwrap())
        .collect()
}

#[cfg(feature = "original-fixtures")]
#[test]
fn mace3_matches_complete_independent_reference_pcm() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../evidence/fixtures/sound-mace3.json"
    ))
    .unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let expected = hex_bytes(case["pcm_le"].as_str().unwrap());
        assert_eq!(crate::digest(&expected), case["pcm_le_sha256"], "{name}");
        let actual = if let Some(input) = case["input"].as_str() {
            let bytes = hex_bytes(input);
            assert_eq!(crate::digest(&bytes), case["input_sha256"], "{name}");
            let sample = decode_snd(&bytes).unwrap();
            assert_eq!(sample.bits_per_sample, 16);
            assert_eq!(
                u64::from(sample.rate_fixed),
                case["rate_fixed"].as_u64().unwrap()
            );
            let wave = sample.wave();
            assert_eq!(&wave[44..], expected);
            assert_eq!(u16::from_le_bytes(wave[32..34].try_into().unwrap()), 2);
            assert_eq!(u16::from_le_bytes(wave[34..36].try_into().unwrap()), 16);
            sample.samples
        } else {
            mace3::decode(&hex_bytes(case["payload"].as_str().unwrap()))
        };
        assert_eq!(actual.len(), case["samples"].as_u64().unwrap() as usize * 2);
        assert_eq!(actual, expected, "{name}");
    }
}

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
    let cases: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../evidence/fixtures/sound-format-two.json"
    ))
    .unwrap();
    for case in cases["cases"].as_array().unwrap() {
        let hex = case["input"].as_str().unwrap();
        let bytes: Vec<u8> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        let sample = decode_snd(&bytes).unwrap();
        assert_eq!(sample.samples, bytes[36..]);
        assert_eq!(
            crate::digest(&sample.samples),
            case["sample_sha256"].as_str().unwrap()
        );
        assert_eq!(
            sample.rate_fixed as u64,
            case["rate_fixed"].as_u64().unwrap()
        );
        let mut truncated = bytes.clone();
        truncated.pop();
        assert!(decode_snd(&truncated).is_err());
    }
}

#[cfg(feature = "original-fixtures")]
#[test]
fn supplied_extended_sound_preserves_signed_pcm_and_wave_layout() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../evidence/fixtures/sound-extended.json"
    ))
    .unwrap();
    let hex = fixture["input"].as_str().unwrap();
    let bytes: Vec<u8> = (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect();
    assert_eq!(
        crate::digest(&bytes),
        fixture["input_sha256"].as_str().unwrap()
    );
    let sample = decode_snd(&bytes).unwrap();
    assert_eq!(sample.bits_per_sample, 16);
    assert_eq!(
        sample.samples.len(),
        fixture["frames"].as_u64().unwrap() as usize * 2
    );
    assert_eq!(
        u64::from(sample.rate_fixed),
        fixture["rate_fixed"].as_u64().unwrap()
    );
    assert_eq!(
        crate::digest(&sample.samples),
        fixture["pcm_le_sha256"].as_str().unwrap()
    );
    let wave = sample.wave();
    assert_eq!(&wave[..4], b"RIFF");
    assert_eq!(&wave[8..16], b"WAVEfmt ");
    assert_eq!(u16::from_le_bytes(wave[22..24].try_into().unwrap()), 1);
    assert_eq!(u16::from_le_bytes(wave[32..34].try_into().unwrap()), 2);
    assert_eq!(u16::from_le_bytes(wave[34..36].try_into().unwrap()), 16);
    assert_eq!(&wave[44..], sample.samples);
    for (offset, replacement) in [(20 + 4, 2i32), (20, 1)] {
        let mut unsupported = bytes.clone();
        unsupported[offset..offset + 4].copy_from_slice(&replacement.to_be_bytes());
        assert!(decode_snd(&unsupported).is_err());
    }
    let mut compressed = bytes.clone();
    compressed[40] = 0xfe;
    assert!(decode_snd(&compressed).is_err());
    for length in [41, 83, bytes.len() - 1] {
        assert!(decode_snd(&bytes[..length]).is_err());
    }
}
