use super::Sample;
use crate::{ContentError, mission::long, span, word};

const STEPS: [i32; 89] = [
    7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 19, 21, 23, 25, 28, 31, 34, 37, 41, 45, 50, 55, 60, 66,
    73, 80, 88, 97, 107, 118, 130, 143, 157, 173, 190, 209, 230, 253, 279, 307, 337, 371, 408, 449,
    494, 544, 598, 658, 724, 796, 876, 963, 1060, 1166, 1282, 1411, 1552, 1707, 1878, 2066, 2272,
    2499, 2749, 3024, 3327, 3660, 4026, 4428, 4871, 5358, 5894, 6484, 7132, 7845, 8630, 9493,
    10442, 11487, 12635, 13899, 15289, 16818, 18500, 20350, 22385, 24623, 27086, 29794, 32767,
];

pub(super) fn decode(bytes: &[u8], offset: usize, rate_fixed: u32) -> Result<Sample, ContentError> {
    span(bytes, offset, 64)?;
    let channels = long(bytes, offset + 4)?;
    let packets = long(bytes, offset + 22)?;
    if !(1..=2).contains(&channels)
        || !(1..=250_000).contains(&packets)
        || !(500 * 65536..=64000u32 * 65536).contains(&rate_fixed)
        || long(bytes, offset)? != 0
        || word(bytes, offset + 56)? != -1
        || word(bytes, offset + 58)? != 34
        || word(bytes, offset + 62)? != 16
    {
        return Err(ContentError::Invalid(
            "unsupported IMA4 sound header".into(),
        ));
    }
    let channels = channels as usize;
    let size = packets as usize * channels * 34;
    let payload = span(bytes, offset + 64, size)?;
    // Two supplied stereo resources retain a final unpaired channel packet.
    let tail = bytes.len() - offset - 64 - size;
    if tail != 0 && !(channels == 2 && tail == 34) {
        return Err(ContentError::Invalid(
            "unexpected IMA4 trailing data".into(),
        ));
    }
    let mut samples = Vec::with_capacity(packets as usize * channels * 128);
    let mut predictors = [0_i32; 2];
    let mut indices = [0_usize; 2];
    for packet in payload.chunks_exact(channels * 34) {
        let mut decoded = [[0_i16; 64]; 2];
        for (channel, block) in packet.as_chunks::<34>().0.iter().enumerate() {
            let header = i16::from_be_bytes([block[0], block[1]]);
            let index = (header as u16 & 127) as usize;
            if index > 88 {
                return Err(ContentError::Invalid("IMA4 step index exceeds 88".into()));
            }
            let predictor = i32::from(header & !127);
            if index != indices[channel] || (predictor - predictors[channel]).abs() > 127 {
                indices[channel] = index;
                predictors[channel] = predictor;
            }
            for (sample, nibble) in block[2..]
                .iter()
                .flat_map(|byte| [byte & 15, byte >> 4])
                .enumerate()
            {
                let step = STEPS[indices[channel]];
                let mut delta = step >> 3;
                if nibble & 1 != 0 {
                    delta += step >> 2;
                }
                if nibble & 2 != 0 {
                    delta += step >> 1;
                }
                if nibble & 4 != 0 {
                    delta += step;
                }
                predictors[channel] = (predictors[channel]
                    + if nibble & 8 != 0 { -delta } else { delta })
                .clamp(-32768, 32767);
                indices[channel] = (indices[channel] as i32
                    + if nibble & 7 < 4 {
                        -1
                    } else {
                        2 * i32::from(nibble & 7) - 6
                    })
                .clamp(0, 88) as usize;
                decoded[channel][sample] = predictors[channel] as i16;
            }
        }
        for sample in 0..64 {
            for channel in decoded.iter().take(channels) {
                samples.extend_from_slice(&channel[sample].to_le_bytes());
            }
        }
    }
    Ok(Sample {
        channels: channels as u16,
        rate_fixed,
        bits_per_sample: 16,
        samples,
    })
}
