use crate::{ContentError, mission::long, span, word};

mod ima4;
mod mace3;

#[derive(Debug)]
pub struct Sample {
    pub channels: u16,
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
        if span(bytes, offset + 40, 4)? == b"ima4" {
            return ima4::decode(bytes, offset, rate_fixed);
        }
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
        channels: 1,
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
        channels: 1,
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
        let block_align = self.bits_per_sample / 8 * self.channels;
        let mut wave = Vec::with_capacity(44 + count as usize + padding as usize);
        wave.extend_from_slice(b"RIFF");
        wave.extend_from_slice(&(36 + count + padding).to_le_bytes());
        wave.extend_from_slice(b"WAVEfmt ");
        wave.extend_from_slice(&16u32.to_le_bytes());
        wave.extend_from_slice(&1u16.to_le_bytes());
        wave.extend_from_slice(&self.channels.to_le_bytes());
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
#[cfg(all(test, feature = "original-fixtures"))]
#[path = "../tests/unit/audio_reference.rs"]
mod tests;
