use crate::{ContentError, span, word};

#[derive(Debug)]
pub struct Image {
    pub width: u16,
    pub height: u16,
    pub rgba: Vec<u8>,
}

pub fn decode_pattern(bytes: &[u8]) -> Result<Image, ContentError> {
    if word(bytes, 0)? != 1 {
        return Err(ContentError::Invalid(
            "unsupported pixel pattern type".into(),
        ));
    }
    let offset = |position| -> Result<usize, ContentError> {
        Ok(u32::from_be_bytes(span(bytes, position, 4)?.try_into().unwrap()) as usize)
    };
    let map = offset(2)?;
    span(bytes, map, 50)?;
    let (width, height) = bounds(bytes, map + 6)?;
    let row = (word(bytes, map + 4)? as u16 & 0x3fff) as usize;
    let bits = word(bytes, map + 32)? as usize;
    if word(bytes, map + 16)? != 0
        || word(bytes, map + 30)? != 0
        || ![1, 2, 4, 8].contains(&bits)
        || row * 8 < width * bits
    {
        return Err(ContentError::Invalid(
            "unsupported pixel pattern format".into(),
        ));
    }
    let pixels = span(bytes, offset(6)?, row * height)?;
    let table = offset(map + 42)?;
    let count = word(bytes, table + 6)?;
    if !(0..=255).contains(&count) {
        return Err(ContentError::Invalid(
            "pixel pattern palette exceeds bounds".into(),
        ));
    }
    let flags = word(bytes, table + 4)? as u16;
    let mut palette = [None; 256];
    for (index, entry) in span(bytes, table + 8, (count as usize + 1) * 8)?
        .as_chunks::<8>()
        .0
        .iter()
        .enumerate()
    {
        let id = if flags & 0x8000 != 0 {
            index
        } else {
            u16::from_be_bytes([entry[0], entry[1]]) as usize
        };
        if id > 255 || palette[id].is_some() {
            return Err(ContentError::Invalid(
                "invalid pixel pattern palette index".into(),
            ));
        }
        palette[id] = Some([entry[2], entry[4], entry[6], 255]);
    }
    let mut rgba = Vec::with_capacity(width * height * 4);
    for y in 0..height {
        for x in 0..width {
            let bit = x * bits;
            let index =
                (pixels[y * row + bit / 8] >> (8 - bits - bit % 8)) as usize & ((1 << bits) - 1);
            rgba.extend_from_slice(
                &palette[index]
                    .ok_or_else(|| ContentError::Invalid("missing pixel pattern color".into()))?,
            );
        }
    }
    Ok(Image {
        width: width as u16,
        height: height as u16,
        rgba,
    })
}

fn bounds(bytes: &[u8], offset: usize) -> Result<(usize, usize), ContentError> {
    let top = i32::from(word(bytes, offset)?);
    let left = i32::from(word(bytes, offset + 2)?);
    let bottom = i32::from(word(bytes, offset + 4)?);
    let right = i32::from(word(bytes, offset + 6)?);
    if !(1..=1024).contains(&(right - left)) || !(1..=1024).contains(&(bottom - top)) {
        return Err(ContentError::Invalid(
            "color icon dimensions out of bounds".into(),
        ));
    }
    Ok(((right - left) as usize, (bottom - top) as usize))
}

/// The original targeting overlay copies the cicn monochrome bitmap with srcXor.
pub fn decode_cicn_bitmap(bytes: &[u8]) -> Result<Image, ContentError> {
    span(bytes, 0, 82)?;
    let (width, height) = bounds(bytes, 70)?;
    let row = (word(bytes, 68)? as u16 & 0x3fff) as usize;
    let mask_row = (word(bytes, 54)? as u16 & 0x3fff) as usize;
    if row * 8 < width || bounds(bytes, 56)? != (width, height) {
        return Err(ContentError::Invalid("target bitmap dimensions".into()));
    }
    let bitmap = span(bytes, 82 + mask_row * height, row * height)?;
    let mut rgba = Vec::with_capacity(width * height * 4);
    for y in 0..height {
        for x in 0..width {
            let bit = bitmap[y * row + x / 8] & (0x80 >> (x % 8)) != 0;
            rgba.extend_from_slice(if bit {
                &[255, 255, 255, 255]
            } else {
                &[0, 0, 0, 0]
            });
        }
    }
    Ok(Image {
        width: width as u16,
        height: height as u16,
        rgba,
    })
}

pub fn decode_cicn(bytes: &[u8]) -> Result<Image, ContentError> {
    span(bytes, 0, 82)?;
    let (width, height) = bounds(bytes, 6)?;
    if bounds(bytes, 56)? != (width, height) {
        return Err(ContentError::Invalid(
            "color icon mask dimensions differ".into(),
        ));
    }
    let row = (word(bytes, 4)? as u16 & 0x3fff) as usize;
    let mask_row = (word(bytes, 54)? as u16 & 0x3fff) as usize;
    let bitmap_row = (word(bytes, 68)? as u16 & 0x3fff) as usize;
    let bits = word(bytes, 32)? as usize;
    if word(bytes, 30)? != 0 || ![1, 2, 4, 8].contains(&bits) {
        return Err(ContentError::Invalid(
            "unsupported color icon pixel format".into(),
        ));
    }
    if row * 8 < width * bits || mask_row * 8 < width {
        return Err(ContentError::Invalid("color icon row is too short".into()));
    }
    if bitmap_row != 0 && bounds(bytes, 70)? != (width, height) {
        return Err(ContentError::Invalid(
            "color icon bitmap dimensions differ".into(),
        ));
    }
    let mask = span(bytes, 82, mask_row * height)?;
    let table_offset = 82 + (mask_row + bitmap_row) * height;
    span(bytes, table_offset, 8)?;
    let count = word(bytes, table_offset + 6)?;
    if !(0..=255).contains(&count) {
        return Err(ContentError::Invalid(
            "invalid color icon palette size".into(),
        ));
    }
    let flags = word(bytes, table_offset + 4)? as u16;
    let entries = span(bytes, table_offset + 8, (count as usize + 1) * 8)?;
    let mut palette = [None; 256];
    for (i, entry) in entries.as_chunks::<8>().0.iter().enumerate() {
        let id = if flags & 0x8000 != 0 {
            i
        } else {
            u16::from_be_bytes([entry[0], entry[1]]) as usize
        };
        if id >= 256 || palette[id].is_some() {
            return Err(ContentError::Invalid(
                "invalid or duplicate palette index".into(),
            ));
        }
        palette[id] = Some([
            (u16::from_be_bytes([entry[2], entry[3]]) / 257) as u8,
            (u16::from_be_bytes([entry[4], entry[5]]) / 257) as u8,
            (u16::from_be_bytes([entry[6], entry[7]]) / 257) as u8,
        ]);
    }
    let pixels = span(bytes, table_offset + 8 + entries.len(), row * height)?;
    let mut rgba = Vec::with_capacity(width * height * 4);
    for y in 0..height {
        for x in 0..width {
            let bit = x * bits;
            let index = ((pixels[y * row + bit / 8] >> (8 - bits - bit % 8))
                & ((1u16 << bits) - 1) as u8) as usize;
            let rgb = palette[index]
                .ok_or_else(|| ContentError::Invalid(format!("missing palette index {index}")))?;
            rgba.extend_from_slice(&rgb);
            rgba.push(if mask[y * mask_row + x / 8] & (0x80 >> (x % 8)) != 0 {
                255
            } else {
                0
            });
        }
    }
    Ok(Image {
        width: width as u16,
        height: height as u16,
        rgba,
    })
}

/// Indexed terrain and interface pictures, and planar RGB interface pictures.
pub fn decode_pict(bytes: &[u8]) -> Result<Image, ContentError> {
    let (width, height) = bounds(bytes, 2)?;
    if span(bytes, 10, 4)? != [0, 0x11, 2, 0xff] {
        return Err(ContentError::Invalid("unsupported PICT version".into()));
    }
    let mut cursor = 14;
    let mut result = None;
    loop {
        cursor += cursor % 2;
        let opcode = word(bytes, cursor)? as u16;
        cursor += 2;
        match opcode {
            0 | 0x1e => {}
            0x0c00 => {
                span(bytes, cursor, 24)?;
                cursor += 24;
            }
            1 => {
                let size = word(bytes, cursor)? as u16 as usize;
                if size != 10 {
                    return Err(ContentError::Invalid("unsupported PICT clip region".into()));
                }
                span(bytes, cursor, size)?;
                cursor += size;
            }
            0xa0 => {
                span(bytes, cursor, 2)?;
                cursor += 2;
            }
            0xa1 => {
                let size = word(bytes, cursor + 2)? as u16 as usize;
                span(bytes, cursor + 4, size)?;
                cursor += 4 + size;
            }
            0x9a => {
                if result.is_some() {
                    return Err(ContentError::Invalid(
                        "multiple PICT pixel operations".into(),
                    ));
                }
                let header = cursor + 4;
                let row = (word(bytes, header)? as u16 & 0x3fff) as usize;
                let (pixmap_width, pixmap_height) = bounds(bytes, header + 2)?;
                if pixmap_width < width
                    || pixmap_height != height
                    || row != pixmap_width * 4
                    || word(bytes, header + 12)? != 4
                    || word(bytes, header + 26)? != 16
                    || word(bytes, header + 28)? != 32
                    || word(bytes, header + 30)? != 3
                    || word(bytes, header + 32)? != 8
                {
                    return Err(ContentError::Invalid(
                        "unsupported direct PICT pixmap".into(),
                    ));
                }
                cursor += 50;
                if bounds(bytes, cursor)? != (width, height)
                    || span(bytes, cursor, 8)? != span(bytes, cursor + 8, 8)?
                    || ![0, 0x40].contains(&word(bytes, cursor + 16)?)
                {
                    return Err(ContentError::Invalid(
                        "unsupported direct PICT transfer".into(),
                    ));
                }
                cursor += 18;
                let mut rgba = Vec::with_capacity(width * height * 4);
                for _ in 0..height {
                    let size = if row > 250 {
                        let size = word(bytes, cursor)? as u16 as usize;
                        cursor += 2;
                        size
                    } else {
                        let size = span(bytes, cursor, 1)?[0] as usize;
                        cursor += 1;
                        size
                    };
                    let planes = unpack_bits(span(bytes, cursor, size)?, pixmap_width * 3)?;
                    cursor += size;
                    for x in 0..width {
                        rgba.extend_from_slice(&[
                            planes[x],
                            planes[pixmap_width + x],
                            planes[2 * pixmap_width + x],
                            255,
                        ]);
                    }
                }
                result = Some(Image {
                    width: width as u16,
                    height: height as u16,
                    rgba,
                });
            }
            0x98 => {
                if result.is_some() {
                    return Err(ContentError::Invalid(
                        "multiple PICT pixel operations".into(),
                    ));
                }
                let header = cursor;
                let row = (word(bytes, header)? as u16 & 0x3fff) as usize;
                let bits = word(bytes, header + 28)? as usize;
                if word(bytes, header)? as u16 & 0x8000 == 0
                    || bounds(bytes, header + 2)? != (width, height)
                    || ![4, 8].contains(&bits)
                    || word(bytes, header + 26)? != 0
                    || (bits == 4
                        && (row < 8
                            || word(bytes, header + 12)? != 0
                            || word(bytes, header + 30)? != 1
                            || word(bytes, header + 32)? != 4))
                    || row * 8 < width * bits
                {
                    return Err(ContentError::Invalid("unsupported PICT pixmap".into()));
                }
                span(bytes, cursor, 46)?;
                cursor += 46;
                let count = word(bytes, cursor + 6)?;
                if !(0..=255).contains(&count) {
                    return Err(ContentError::Invalid("PICT palette count".into()));
                }
                let flags = word(bytes, cursor + 4)? as u16;
                let entries = span(bytes, cursor + 8, (count as usize + 1) * 8)?;
                let mut palette = [None; 256];
                for (index, entry) in entries.as_chunks::<8>().0.iter().enumerate() {
                    let id = if flags & 0x8000 != 0 {
                        index
                    } else {
                        u16::from_be_bytes([entry[0], entry[1]]) as usize
                    };
                    if id >= 256 || palette[id].is_some() {
                        return Err(ContentError::Invalid("PICT palette index".into()));
                    }
                    palette[id] = Some([
                        (u16::from_be_bytes([entry[2], entry[3]]) / 257) as u8,
                        (u16::from_be_bytes([entry[4], entry[5]]) / 257) as u8,
                        (u16::from_be_bytes([entry[6], entry[7]]) / 257) as u8,
                    ]);
                }
                cursor += 8 + entries.len();
                if bounds(bytes, cursor)? != (width, height)
                    || bounds(bytes, cursor + 8)? != (width, height)
                    || span(bytes, cursor, 8)? != span(bytes, cursor + 8, 8)?
                    || word(bytes, cursor + 16)? != 0
                {
                    return Err(ContentError::Invalid("unsupported PICT transfer".into()));
                }
                cursor += 18;
                let mut rgba = Vec::with_capacity(width * height * 4);
                for _ in 0..height {
                    let size = if row > 250 {
                        let n = word(bytes, cursor)? as u16 as usize;
                        cursor += 2;
                        n
                    } else {
                        let n = span(bytes, cursor, 1)?[0] as usize;
                        cursor += 1;
                        n
                    };
                    let packed = span(bytes, cursor, size)?;
                    cursor += size;
                    let mut position = 0;
                    let mut pixels = Vec::with_capacity(row);
                    while position < packed.len() {
                        let command = packed[position] as i8;
                        position += 1;
                        if command >= 0 {
                            let count = command as usize + 1;
                            pixels.extend_from_slice(span(packed, position, count)?);
                            position += count;
                        } else if command != -128 {
                            let value = span(packed, position, 1)?[0];
                            position += 1;
                            let count = (1 - i16::from(command)) as usize;
                            if pixels.len() + count > row {
                                return Err(ContentError::Invalid(
                                    "PICT PackBits row overflow".into(),
                                ));
                            }
                            pixels.resize(pixels.len() + count, value);
                        }
                        if pixels.len() > row {
                            return Err(ContentError::Invalid("PICT PackBits row overflow".into()));
                        }
                    }
                    if pixels.len() != row {
                        return Err(ContentError::Invalid("PICT PackBits row length".into()));
                    }
                    for x in 0..width {
                        let bit = x * bits;
                        let index =
                            (pixels[bit / 8] >> (8 - bits - bit % 8)) & ((1_u16 << bits) - 1) as u8;
                        let rgb = palette[usize::from(index)].ok_or_else(|| {
                            ContentError::Invalid("missing PICT palette index".into())
                        })?;
                        rgba.extend_from_slice(&rgb);
                        rgba.push(255);
                    }
                }
                result = Some(Image {
                    width: width as u16,
                    height: height as u16,
                    rgba,
                });
            }
            0xff => {
                return result.ok_or_else(|| ContentError::Invalid("PICT has no pixels".into()));
            }
            _ => {
                return Err(ContentError::Invalid(format!(
                    "unsupported PICT opcode {opcode:04x} at {}",
                    cursor - 2
                )));
            }
        }
    }
}

fn unpack_bits(packed: &[u8], expected: usize) -> Result<Vec<u8>, ContentError> {
    let mut cursor = 0;
    let mut output = Vec::with_capacity(expected);
    while cursor < packed.len() {
        let command = packed[cursor] as i8;
        cursor += 1;
        if command >= 0 {
            let count = command as usize + 1;
            output.extend_from_slice(span(packed, cursor, count)?);
            cursor += count;
        } else if command != -128 {
            let value = span(packed, cursor, 1)?[0];
            cursor += 1;
            let count = (1 - i16::from(command)) as usize;
            if output.len() + count > expected {
                return Err(ContentError::Invalid("PICT PackBits row overflow".into()));
            }
            output.resize(output.len() + count, value);
        }
        if output.len() > expected {
            return Err(ContentError::Invalid("PICT PackBits row overflow".into()));
        }
    }
    if output.len() != expected {
        return Err(ContentError::Invalid("PICT PackBits row length".into()));
    }
    Ok(output)
}

/// A colour cursor ('crsr', type 0x8001) with its hotspot as (h, v).
#[derive(Debug)]
pub struct Cursor {
    pub image: Image,
    pub hotspot: (i16, i16),
}

/// Decodes a 16x16 colour cursor: CCrsr header, PixMap, indexed pixels and colour
/// table. Pixels outside the mask are transparent; where the 1-bit data is set
/// outside the mask, the original inverts the screen, which is drawn black here.
pub fn decode_crsr(bytes: &[u8]) -> Result<Cursor, ContentError> {
    span(bytes, 0, 96)?;
    if word(bytes, 0)? as u16 != 0x8001 {
        return Err(ContentError::Invalid("unsupported cursor type".into()));
    }
    let long = |offset: usize| -> Result<usize, ContentError> {
        let raw = span(bytes, offset, 4)?;
        Ok(u32::from_be_bytes([raw[0], raw[1], raw[2], raw[3]]) as usize)
    };
    let map = long(2)?;
    let data = long(6)?;
    let one_bit = span(bytes, 20, 32)?;
    let mask = span(bytes, 52, 32)?;
    let hotspot = (word(bytes, 86)?, word(bytes, 84)?);
    span(bytes, map, 50)?;
    let row = (word(bytes, map + 4)? as u16 & 0x3fff) as usize;
    let (width, height) = bounds(bytes, map + 6)?;
    let bits = word(bytes, map + 32)? as usize;
    if (width, height) != (16, 16) || ![1, 2, 4, 8].contains(&bits) || row * 8 < width * bits {
        return Err(ContentError::Invalid("unsupported cursor pixel map".into()));
    }
    let table = long(map + 42)?;
    span(bytes, table, 8)?;
    let count = word(bytes, table + 6)?;
    if !(0..=255).contains(&count) {
        return Err(ContentError::Invalid("invalid cursor palette size".into()));
    }
    let flags = word(bytes, table + 4)? as u16;
    let entries = span(bytes, table + 8, (count as usize + 1) * 8)?;
    let mut palette = [None; 256];
    for (i, entry) in entries.as_chunks::<8>().0.iter().enumerate() {
        let id = if flags & 0x8000 != 0 {
            i
        } else {
            u16::from_be_bytes([entry[0], entry[1]]) as usize
        };
        if id >= 256 {
            return Err(ContentError::Invalid("invalid cursor palette index".into()));
        }
        palette[id] = Some([
            (u16::from_be_bytes([entry[2], entry[3]]) / 257) as u8,
            (u16::from_be_bytes([entry[4], entry[5]]) / 257) as u8,
            (u16::from_be_bytes([entry[6], entry[7]]) / 257) as u8,
        ]);
    }
    let pixels = span(bytes, data, row * height)?;
    let mut rgba = Vec::with_capacity(width * height * 4);
    for y in 0..height {
        for x in 0..width {
            let masked = mask[y * 2 + x / 8] & (0x80 >> (x % 8)) != 0;
            let set = one_bit[y * 2 + x / 8] & (0x80 >> (x % 8)) != 0;
            if !masked {
                rgba.extend_from_slice(if set { &[0, 0, 0, 255] } else { &[0, 0, 0, 0] });
                continue;
            }
            let bit = x * bits;
            let index = ((pixels[y * row + bit / 8] >> (8 - bits - bit % 8))
                & ((1u16 << bits) - 1) as u8) as usize;
            let [r, g, b] = palette[index]
                .ok_or_else(|| ContentError::Invalid("cursor pixel outside palette".into()))?;
            rgba.extend_from_slice(&[r, g, b, 255]);
        }
    }
    Ok(Cursor {
        image: Image {
            width: width as u16,
            height: height as u16,
            rgba,
        },
        hotspot,
    })
}
