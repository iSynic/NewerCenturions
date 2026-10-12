//! Navigation calculations and coordinate display from CODE 1 instructions.

use crate::{
    RuleError,
    recovered_economy::ArithmeticQualification,
    recovered_geometry::BodyRect,
    recovered_install::InstallMemory,
    recovered_records::RecordWrite,
    recovered_text::{NumericServices, PascalString},
};
use centurions_content::WARRIOR_BYTES;

fn unavailable(action: &'static str) -> RuleError {
    RuleError::Unsupported {
        action,
        dependency: "reached original memory address is unavailable",
    }
}

fn offset(
    memory: &InstallMemory,
    address: u32,
    length: usize,
    action: &'static str,
) -> Result<usize, RuleError> {
    let start = address.checked_sub(memory.base_address).map(|n| n as usize);
    match start {
        Some(start)
            if start
                .checked_add(length)
                .is_some_and(|end| end <= memory.bytes.len()) =>
        {
            Ok(start)
        }
        _ => Err(unavailable(action)),
    }
}

fn read<'a>(
    memory: &'a InstallMemory,
    address: u32,
    length: usize,
    action: &'static str,
) -> Result<&'a [u8], RuleError> {
    let start = offset(memory, address, length, action)?;
    Ok(&memory.bytes[start..start + length])
}

fn word(memory: &InstallMemory, address: u32, action: &'static str) -> Result<i16, RuleError> {
    let bytes = read(memory, address, 2, action)?;
    Ok(i16::from_be_bytes([bytes[0], bytes[1]]))
}

fn long(memory: &InstallMemory, address: u32, action: &'static str) -> Result<i32, RuleError> {
    let bytes = read(memory, address, 4, action)?;
    Ok(i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn write(
    memory: &mut InstallMemory,
    address: u32,
    bytes: &[u8],
    action: &'static str,
) -> Result<RecordWrite, RuleError> {
    let start = offset(memory, address, bytes.len(), action)?;
    memory.bytes[start..start + bytes.len()].copy_from_slice(bytes);
    Ok(RecordWrite {
        address,
        bytes: bytes.to_vec(),
    })
}

fn truncate(value: f32, action: &'static str) -> Result<i32, RuleError> {
    let value = f64::from(value).trunc();
    if !value.is_finite() || value < f64::from(i32::MIN) || value > f64::from(i32::MAX) {
        return Err(RuleError::Unsupported {
            action,
            dependency: "original SANE exceptional or overflowing LONG conversion is unavailable",
        });
    }
    Ok(value as i32)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VisibilityResult {
    pub native_return: i32,
    pub samples: u16,
    pub last_pixel: (i16, i16),
    pub step_single_bits: (u32, u32),
    pub qualification: ArithmeticQualification,
}

/// cansee, 0x10ae90: 128 samples, WORD pixel coordinates, LONG map origins.
pub fn cansee(
    memory: &InstallMemory,
    from: (i16, i16),
    to: (i16, i16),
) -> Result<VisibilityResult, RuleError> {
    const ACTION: &str = "navigation.cansee";
    let divisor = word(memory, 0x439568, ACTION)?.wrapping_add(128);
    if divisor == 0 {
        return Err(RuleError::Unsupported {
            action: ACTION,
            dependency: "original SANE division by zero is unavailable",
        });
    }
    let mut pixel = (from.0.wrapping_mul(32), from.1.wrapping_mul(32));
    let end = (to.0.wrapping_mul(32), to.1.wrapping_mul(32));
    let step = (
        (f64::from(end.0.wrapping_sub(pixel.0)) / f64::from(divisor)) as f32,
        (f64::from(end.1.wrapping_sub(pixel.1)) / f64::from(divisor)) as f32,
    );
    let mut point = (f32::from(pixel.0), f32::from(pixel.1));
    for sample in 1..=128 {
        let row =
            i32::from(pixel.0.wrapping_add(16) / 32).wrapping_add(long(memory, 0x442780, ACTION)?);
        let column =
            i32::from(pixel.1.wrapping_add(16) / 32).wrapping_add(long(memory, 0x442784, ACTION)?);
        let address = 0x4085b8_u32
            .wrapping_add(row.wrapping_mul(180) as u32)
            .wrapping_add(column.wrapping_mul(2) as u32);
        let tile = word(memory, address, ACTION)?;
        if tile > 999 {
            let flags = 0x431688_u32.wrapping_add(u32::from(tile.wrapping_sub(1000) as u16) * 40);
            if word(memory, flags, ACTION)? != 0 {
                return Ok(VisibilityResult {
                    native_return: 0,
                    samples: sample,
                    last_pixel: pixel,
                    step_single_bits: (step.0.to_bits(), step.1.to_bits()),
                    qualification: ArithmeticQualification::SourceDerivedBinary32,
                });
            }
        }
        // The original rounds each addition to Single, then converts y before x.
        point.1 = (f64::from(point.1) + f64::from(step.1)) as f32;
        point.0 = (f64::from(point.0) + f64::from(step.0)) as f32;
        pixel.1 = truncate(point.1, ACTION)? as i16;
        pixel.0 = truncate(point.0, ACTION)? as i16;
    }
    Ok(VisibilityResult {
        native_return: 1,
        samples: 128,
        last_pixel: pixel,
        step_single_bits: (step.0.to_bits(), step.1.to_bits()),
        qualification: ArithmeticQualification::SourceDerivedBinary32,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MovementResult {
    pub memory: InstallMemory,
    pub writes: Vec<RecordWrite>,
    pub maximum: i16,
    pub movement: i16,
    pub ratio_single_bits: u32,
    pub result_single_bits: u32,
    pub native_return: i32,
    pub qualification: ArithmeticQualification,
}

/// movecalc, 0x136994: private record copy followed by the full ordered copy back.
pub fn movecalc(memory: &InstallMemory, member: u16) -> Result<MovementResult, RuleError> {
    const ACTION: &str = "navigation.movecalc";
    let address = match member {
        0xfffe => 0x447e5e,
        0xffff => 0x44721c,
        _ => 0x448aa0_u32.wrapping_add(u32::from(member) * WARRIOR_BYTES as u32),
    };
    let mut record = read(memory, address, WARRIOR_BYTES, ACTION)?.to_vec();
    let get = |at: usize| i16::from_be_bytes([record[at], record[at + 1]]);
    let maximum = get(0xace)
        .wrapping_mul(10)
        .wrapping_add(get(0x9e2).wrapping_mul(100))
        .wrapping_add(1000);
    let remaining = maximum.wrapping_sub(get(0x9d6)) as u16;
    let divisor = maximum as u16;
    if divisor == 0 {
        return Err(RuleError::Unsupported {
            action: ACTION,
            dependency: "original SANE division by zero is unavailable",
        });
    }
    let ratio = (f64::from(remaining) / f64::from(divisor)) as f32;
    let race_address = 0x435664_u32.wrapping_add(i32::from(get(0x32).wrapping_sub(1)) as u32);
    let race_movement = read(memory, race_address, 1, ACTION)?[0] as i8;
    let mut value = (f64::from(ratio) * f64::from(race_movement)) as f32;
    if get(0x62) != 0 {
        value = (f64::from(value) - f64::from(get(0x62))) as f32;
    }
    if get(0x6a) != 0 {
        value = (f64::from(value) / 2.0) as f32;
    }
    if get(0x8c) != 0 {
        value = (f64::from(value) * 2.0) as f32;
    }
    value = (f64::from(value) + f64::from(get(0x30))) as f32;
    value = (f64::from(value) + 0.5) as f32;
    if value < 5.0 {
        value = 5.0;
    }
    let movement = truncate(value, ACTION)? as i16;
    record[0x9d8..0x9da].copy_from_slice(&maximum.to_be_bytes());
    record[0x3e..0x40].copy_from_slice(&movement.to_be_bytes());
    let mut next = memory.clone();
    let mut writes = Vec::with_capacity(785);
    for (index, bytes) in record[..3136].as_chunks::<4>().0.iter().enumerate() {
        writes.push(write(
            &mut next,
            address.wrapping_add(index as u32 * 4),
            bytes,
            ACTION,
        )?);
    }
    writes.push(write(
        &mut next,
        address.wrapping_add(3136),
        &record[3136..3138],
        ACTION,
    )?);
    Ok(MovementResult {
        memory: next,
        writes,
        maximum,
        movement,
        ratio_single_bits: ratio.to_bits(),
        result_single_bits: value.to_bits(),
        native_return: 0,
        qualification: ArithmeticQualification::SourceDerivedBinary32,
    })
}

/// twixt, 0x13846a: signed inclusive WORD bounds, returning a zero-extended BYTE.
pub fn twixt(value: i16, lower: i16, upper: i16) -> i32 {
    i32::from(lower <= value && value <= upper)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CoordinateEffect {
    SetPort(u32),
    BackPixelPattern(u32),
    RgbForeColor([u16; 3]),
    TextFont(i16),
    TextSize(i16),
    TextMode(i16),
    TextFace(u8),
    EraseRect(BodyRect),
    MoveTo { horizontal: i16, vertical: i16 },
    NumberToString(i32),
    Write(RecordWrite),
    DrawString(PascalString),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoordinateResult {
    pub memory: InstallMemory,
    pub scratch: PascalString,
    pub effects: Vec<CoordinateEffect>,
    pub native_return: i32,
}

fn convert_coordinate(
    memory: &mut InstallMemory,
    value: i32,
    services: &mut impl NumericServices,
    effects: &mut Vec<CoordinateEffect>,
) -> Result<PascalString, RuleError> {
    let converted = services.number_to_string(value)?;
    effects.push(CoordinateEffect::NumberToString(value));
    effects.push(CoordinateEffect::Write(write(
        memory,
        0x479ed8,
        converted.raw(),
        "navigation.xy",
    )?));
    Ok(converted)
}

fn coordinate_literal(memory: &InstallMemory) -> Result<PascalString, RuleError> {
    let count = usize::from(read(memory, 0x40226a, 1, "navigation.xy")?[0]);
    PascalString::from_raw(read(memory, 0x40226a, count + 1, "navigation.xy")?)
}

/// xy, 0x138a40: retain numeric conversion even when the ordinary display is hidden.
pub fn xy(
    memory: &InstallMemory,
    mode: i16,
    services: &mut impl NumericServices,
) -> Result<CoordinateResult, RuleError> {
    const ACTION: &str = "navigation.xy";
    let mut next = memory.clone();
    let mut effects = vec![
        CoordinateEffect::SetPort(long(memory, 0x43878c, ACTION)? as u32),
        CoordinateEffect::BackPixelPattern(long(memory, 0x4358ce, ACTION)? as u32),
    ];
    let width = word(memory, 0x40856c, ACTION)?;
    let height = word(memory, 0x40856e, ACTION)?;
    let shift = word(memory, 0x439582, ACTION)?.wrapping_mul(32);
    let top = height.wrapping_add(304).wrapping_sub(shift);
    let first_rect = BodyRect {
        top,
        left: width.wrapping_add(350),
        bottom: top.wrapping_add(16),
        right: width.wrapping_add(375),
    };
    effects.extend([
        CoordinateEffect::RgbForeColor([
            word(memory, 0x44337a, ACTION)? as u16,
            word(memory, 0x44337c, ACTION)? as u16,
            word(memory, 0x44337e, ACTION)? as u16,
        ]),
        CoordinateEffect::TextFont(word(memory, 0x402f3c, ACTION)?),
        CoordinateEffect::TextSize(12),
        CoordinateEffect::TextMode(1),
        CoordinateEffect::TextFace(0),
        CoordinateEffect::EraseRect(first_rect),
        CoordinateEffect::EraseRect(BodyRect {
            left: first_rect.left.wrapping_add(50),
            right: first_rect.right.wrapping_add(50),
            ..first_rect
        }),
    ]);
    let vertical = height.wrapping_add(316).wrapping_sub(shift);
    effects.push(CoordinateEffect::MoveTo {
        horizontal: width.wrapping_add(352),
        vertical,
    });
    let first_value = if mode == 0 {
        long(&next, 0x442778, ACTION)?.wrapping_add(long(&next, 0x442764, ACTION)?)
    } else {
        i32::from(word(&next, 0x442814, ACTION)?)
    };
    let first = convert_coordinate(&mut next, first_value, services, &mut effects)?;
    let first_drawn = if mode == 0 && word(&next, 0x4354fe, ACTION)? != 0 {
        coordinate_literal(&next)?
    } else {
        first
    };
    effects.push(CoordinateEffect::DrawString(first_drawn));
    effects.push(CoordinateEffect::MoveTo {
        horizontal: width.wrapping_add(406),
        vertical,
    });
    let second_value = if mode == 0 {
        long(&next, 0x44277c, ACTION)?.wrapping_add(long(&next, 0x442768, ACTION)?)
    } else {
        i32::from(word(&next, 0x442816, ACTION)?)
    };
    let scratch = convert_coordinate(&mut next, second_value, services, &mut effects)?;
    let second_drawn = if mode == 0 && word(&next, 0x4354fe, ACTION)? != 0 {
        coordinate_literal(&next)?
    } else {
        scratch.clone()
    };
    effects.push(CoordinateEffect::DrawString(second_drawn));
    Ok(CoordinateResult {
        memory: next,
        scratch,
        effects,
        native_return: 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn memory(base_address: u32, end: u32) -> InstallMemory {
        InstallMemory {
            base_address,
            bytes: vec![0; (end - base_address) as usize],
        }
    }

    fn put(memory: &mut InstallMemory, address: u32, bytes: &[u8]) {
        let start = (address - memory.base_address) as usize;
        memory.bytes[start..start + bytes.len()].copy_from_slice(bytes);
    }

    #[test]
    fn visibility_keeps_word_pixels_long_origins_and_field_twelve() {
        let mut current = memory(0x4085b8, 0x442788);
        put(&mut current, 0x442780, &1_i32.to_be_bytes());
        put(&mut current, 0x442784, &2_i32.to_be_bytes());
        put(
            &mut current,
            0x4085b8 + 2 * 180 + 2 * 2,
            &1000_i16.to_be_bytes(),
        );
        put(&mut current, 0x431688, &1_i16.to_be_bytes());
        let original = current.clone();
        let blocked = cansee(&current, (0, 0), (3, 0)).unwrap();
        assert_eq!(
            (blocked.native_return, blocked.samples, blocked.last_pixel),
            (0, 23, (16, 0))
        );
        assert_eq!(blocked.step_single_bits, (0.75_f32.to_bits(), 0));
        assert_eq!(current, original);
        put(&mut current, 0x431688, &0_i16.to_be_bytes());
        assert_eq!(
            cansee(&current, (0, 0), (3, 0)).unwrap().last_pixel,
            (96, 0)
        );
        assert_eq!(
            cansee(&current, (2048, 0), (2048, 0))
                .unwrap()
                .native_return,
            1
        );
        put(&mut current, 0x439568, &(-128_i16).to_be_bytes());
        assert!(matches!(
            cansee(&current, (0, 0), (3, 0)),
            Err(RuleError::Unsupported { .. })
        ));
        assert!(cansee(&memory(0x439568, 0x43956a), (0, 0), (1, 1)).is_err());
        assert_eq!(twixt(i16::MIN, i16::MIN, -1), 1);
        assert_eq!(twixt(5, 5, 5), 1);
        assert_eq!(twixt(0, 1, -1), 0);
    }

    #[test]
    fn movement_copies_every_record_byte_and_uses_wrapped_unsigned_ratio() {
        for (member, address) in [
            (0xfffe, 0x447e5e),
            (0xffff, 0x44721c),
            (10, 0x448aa0 + 10 * 3138),
        ] {
            let mut current = memory(0x435663, address + 3138);
            put(&mut current, 0x435664, &[20]);
            let record: Vec<u8> = (0..3138).map(|i| (i * 17 + 9) as u8).collect();
            put(&mut current, address, &record);
            for (at, value) in [
                (0xace, 6_i16),
                (0x9e2, 2),
                (0x9d6, 100),
                (0x9d8, -99),
                (0x32, 1),
                (0x62, 3),
                (0x6a, 1),
                (0x8c, 1),
                (0x30, 2),
            ] {
                put(&mut current, address + at, &value.to_be_bytes());
            }
            let original = current.clone();
            let result = movecalc(&current, member).unwrap();
            assert_eq!(
                (result.maximum, result.movement, result.native_return),
                (1260, 17, 0)
            );
            let mut expected = original.clone();
            put(&mut expected, address + 0x9d8, &1260_i16.to_be_bytes());
            put(&mut expected, address + 0x3e, &17_i16.to_be_bytes());
            assert_eq!(result.memory, expected);
            assert_eq!(current, original);
            assert_eq!(result.writes.len(), 785);
            assert_eq!(
                (
                    result.writes[0].address,
                    result.writes[783].address,
                    result.writes[784].address
                ),
                (address, address + 3132, address + 3136)
            );
            assert!(
                result.writes[..784]
                    .iter()
                    .all(|write| write.bytes.len() == 4)
            );
            assert_eq!(result.writes[784].bytes.len(), 2);
            for at in [0xace, 0x9e2] {
                put(&mut current, address + at, &i16::MAX.to_be_bytes());
            }
            put(&mut current, address + 0x9d6, &1000_i16.to_be_bytes());
            let wrapped = movecalc(&current, member).unwrap();
            assert_eq!(
                (wrapped.maximum, wrapped.movement, wrapped.ratio_single_bits),
                (890, 1469, 0x42930654)
            );
            put(&mut current, address + 0xace, &(-100_i16).to_be_bytes());
            put(&mut current, address + 0x9e2, &0_i16.to_be_bytes());
            assert!(movecalc(&current, member).is_err());
        }
    }

    #[derive(Default)]
    struct Numbers(Vec<i32>);

    impl NumericServices for Numbers {
        fn number_to_string(&mut self, value: i32) -> Result<PascalString, RuleError> {
            self.0.push(value);
            PascalString::from_payload(value.to_string().as_bytes())
        }
        fn string_to_number(&mut self, _: &PascalString) -> Result<i32, RuleError> {
            unreachable!()
        }
    }

    #[test]
    fn coordinates_keep_toolbox_order_word_flag_and_conversion_scratch_tail() {
        let mut current = memory(0x40226a, 0x479fd8);
        for (at, value) in [
            (0x40856c, 10_i16),
            (0x40856e, 20),
            (0x439582, 2),
            (0x402f3c, 3),
            (0x44337a, -1),
            (0x44337c, 17),
            (0x44337e, 42),
            (0x442814, -7),
            (0x442816, -9),
        ] {
            put(&mut current, at, &value.to_be_bytes());
        }
        put(&mut current, 0x43878c, &0x11223344_u32.to_be_bytes());
        put(&mut current, 0x4358ce, &0x55667788_u32.to_be_bytes());
        put(&mut current, 0x442778, &i32::MAX.to_be_bytes());
        put(&mut current, 0x442764, &1_i32.to_be_bytes());
        put(&mut current, 0x44277c, &(-2_i32).to_be_bytes());
        put(&mut current, 0x442768, &5_i32.to_be_bytes());
        put(&mut current, 0x40226a, &[1, b'?']);
        put(&mut current, 0x4354fe, &[0, 0, 1, 0]);
        put(&mut current, 0x479ed8, &[0xaa; 256]);
        let original = current.clone();
        let mut service = Numbers::default();
        let result = xy(&current, 0, &mut service).unwrap();
        assert_eq!(service.0, [i32::MIN, 3]);
        assert_eq!(
            &result.effects[..10],
            &[
                CoordinateEffect::SetPort(0x11223344),
                CoordinateEffect::BackPixelPattern(0x55667788),
                CoordinateEffect::RgbForeColor([65535, 17, 42]),
                CoordinateEffect::TextFont(3),
                CoordinateEffect::TextSize(12),
                CoordinateEffect::TextMode(1),
                CoordinateEffect::TextFace(0),
                CoordinateEffect::EraseRect(BodyRect {
                    top: 260,
                    left: 360,
                    bottom: 276,
                    right: 385
                }),
                CoordinateEffect::EraseRect(BodyRect {
                    top: 260,
                    left: 410,
                    bottom: 276,
                    right: 435
                }),
                CoordinateEffect::MoveTo {
                    horizontal: 362,
                    vertical: 272
                },
            ]
        );
        let drawn: Vec<_> = result
            .effects
            .iter()
            .filter_map(|effect| {
                if let CoordinateEffect::DrawString(text) = effect {
                    Some(text.payload())
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(drawn, [b"-2147483648".as_slice(), b"3".as_slice()]);
        let mut expected = original.clone();
        put(&mut expected, 0x479ed8, b"\x0b-2147483648");
        put(&mut expected, 0x479ed8, b"\x013");
        assert_eq!(result.memory, expected);
        assert_eq!(current, original);
        put(&mut current, 0x4354fe, &1_i16.to_be_bytes());
        let hidden = xy(&current, 0, &mut Numbers::default()).unwrap();
        assert!(
            hidden
                .effects
                .iter()
                .filter_map(
                    |effect| if let CoordinateEffect::DrawString(text) = effect {
                        Some(text.payload())
                    } else {
                        None
                    }
                )
                .all(|text| text == b"?")
        );
        let mut special_service = Numbers::default();
        let special = xy(&current, -1, &mut special_service).unwrap();
        assert_eq!(special_service.0, [-7, -9]);
        assert_eq!(special.scratch.payload(), b"-9");
    }
}
