use crate::{ContentError, mission::long, span, word};

mod mace3;

#[derive(Debug)]
pub struct Sample {
    pub rate_fixed: u32,
    pub bits_per_sample: u16,
    pub samples: Vec<u8>,
}

pub fn decode_snd(bytes: &[u8]) -> Result<Sample, ContentError> {
    let (command, header_minimum) = match word(bytes, 0)? {
        1 if word(bytes, 2)? == 1 && word(bytes, 4)? == 5 && word(bytes, 10)? == 1 => (12, 20),
        2 if word(bytes, 4)? == 1 => (6, 14),
        _ => {
            return Err(ContentError::Invalid(
                "unsupported sound resource command list".into(),
            ));
        }
    };
    if word(bytes, command)? as u16 != 0x8051 {
        return Err(ContentError::Invalid(
            "unsupported sound resource command list".into(),
        ));
    }
    let mut offset = long(bytes, command + 4)?;
    // Supplied format-2 sounds 105/134 retain the format-1 offset after its six-byte header removal.
    if header_minimum == 14 && offset == 20 && bytes.len() >= 36 {
        let count = long(bytes, 18)?;
        if count > 0 && count as usize == bytes.len() - 36 && bytes[34] == 0 {
            offset = 14;
        }
    }
    if offset < header_minimum {
        return Err(ContentError::Invalid("invalid sound header offset".into()));
    }
    let offset = offset as usize;
    span(bytes, offset, 22)?;
    let rate_fixed = long(bytes, offset + 8)? as u32;
    if bytes[offset + 20] == 0xfe {
        return decode_mace3(bytes, offset, rate_fixed);
    }
    let (count, bits_per_sample, sample_offset) = match bytes[offset + 20] {
        0 => (long(bytes, offset + 4)?, 8, offset + 22),
        0xff => {
            span(bytes, offset, 64)?;
            if long(bytes, offset)? != 0
                || long(bytes, offset + 4)? != 1
                || word(bytes, offset + 48)? != 16
                || span(bytes, offset + 36, 12)?.iter().any(|&b| b != 0)
                || span(bytes, offset + 50, 14)?.iter().any(|&b| b != 0)
            {
                return Err(ContentError::Invalid(
                    "unsupported extended sound header".into(),
                ));
            }
            (long(bytes, offset + 22)?, 16, offset + 64)
        }
        _ => {
            return Err(ContentError::Invalid("unsupported sound encoding".into()));
        }
    };
    if !(1..=16_000_000).contains(&count) || !(500 * 65536..=64000u32 * 65536).contains(&rate_fixed)
    {
        return Err(ContentError::Invalid(
            "sound size or sample rate outside bounds".into(),
        ));
    }
    let mut samples = span(
        bytes,
        sample_offset,
        count as usize * (bits_per_sample / 8) as usize,
    )?
    .to_vec();
    if bits_per_sample == 16 {
        for sample in samples.as_chunks_mut::<2>().0 {
            sample.swap(0, 1);
        }
    }
    Ok(Sample {
        rate_fixed,
        bits_per_sample,
        samples,
    })
}

fn decode_mace3(bytes: &[u8], offset: usize, rate_fixed: u32) -> Result<Sample, ContentError> {
    span(bytes, offset, 64)?;
    if long(bytes, offset)? != 0
        || long(bytes, offset + 4)? != 1
        || long(bytes, offset + 12)? != 0
        || long(bytes, offset + 16)? != 0
        || span(bytes, offset + 36, 20)?.iter().any(|&byte| byte != 0)
        || word(bytes, offset + 56)? != 3
        || word(bytes, offset + 58)? != 16
        || word(bytes, offset + 62)? != 8
    {
        return Err(ContentError::Invalid(
            "unsupported compressed sound header".into(),
        ));
    }
    let packets = long(bytes, offset + 22)?;
    if !(1..=16_000_000 / 6).contains(&packets)
        || !(500 * 65536..=64000u32 * 65536).contains(&rate_fixed)
    {
        return Err(ContentError::Invalid(
            "sound size or sample rate outside bounds".into(),
        ));
    }
    let sample_offset = offset + 64;
    let payload = span(bytes, sample_offset, packets as usize * 2)?;
    let tail = &bytes[sample_offset + payload.len()..];
    if tail.len() > 1 {
        return Err(ContentError::Invalid(
            "unexpected compressed sound trailing data".into(),
        ));
    }
    Ok(Sample {
        rate_fixed,
        bits_per_sample: 16,
        samples: mace3::decode(payload),
    })
}

impl Sample {
    /// WAV uses integral sample rates; retain the original 16.16 rate in conversion receipts.
    pub fn wave(&self) -> Vec<u8> {
        let rate = (u64::from(self.rate_fixed) + 32768) / 65536;
        let count = self.samples.len() as u32;
        let padding = count % 2;
        let block_align = self.bits_per_sample / 8;
        let mut wave = Vec::with_capacity(44 + count as usize + padding as usize);
        wave.extend_from_slice(b"RIFF");
        wave.extend_from_slice(&(36 + count + padding).to_le_bytes());
        wave.extend_from_slice(b"WAVEfmt ");
        wave.extend_from_slice(&16u32.to_le_bytes());
        wave.extend_from_slice(&1u16.to_le_bytes());
        wave.extend_from_slice(&1u16.to_le_bytes());
        wave.extend_from_slice(&(rate as u32).to_le_bytes());
        wave.extend_from_slice(&(rate as u32 * u32::from(block_align)).to_le_bytes());
        wave.extend_from_slice(&block_align.to_le_bytes());
        wave.extend_from_slice(&self.bits_per_sample.to_le_bytes());
        wave.extend_from_slice(b"data");
        wave.extend_from_slice(&count.to_le_bytes());
        wave.extend_from_slice(&self.samples);
        if padding != 0 {
            wave.push(0);
        }
        wave
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../../../evidence/fixtures/sound-mace3.json"))
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
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../../../evidence/fixtures/sound-mace3.json"))
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
            "../../../evidence/fixtures/sound-format-two.json"
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
            "../../../evidence/fixtures/sound-extended.json"
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
}
