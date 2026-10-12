//! CODE 1 text routines with explicit Pascal bytes and ordered Toolbox requests.

use crate::RuleError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PascalString(Vec<u8>);

impl Default for PascalString {
    fn default() -> Self {
        Self(vec![0])
    }
}

impl PascalString {
    pub fn from_raw(bytes: &[u8]) -> Result<Self, RuleError> {
        let length = bytes
            .first()
            .copied()
            .ok_or_else(|| RuleError::Invalid("missing Pascal string length".into()))?;
        let raw = bytes.get(..usize::from(length) + 1).ok_or_else(|| {
            RuleError::Invalid("Pascal string exceeds its supplied byte buffer".into())
        })?;
        Ok(Self(raw.to_vec()))
    }

    pub fn from_payload(bytes: &[u8]) -> Result<Self, RuleError> {
        let length = u8::try_from(bytes.len())
            .map_err(|_| RuleError::Invalid("Pascal string exceeds 255 bytes".into()))?;
        let mut raw = Vec::with_capacity(bytes.len() + 1);
        raw.push(length);
        raw.extend_from_slice(bytes);
        Ok(Self(raw))
    }

    pub fn raw(&self) -> &[u8] {
        &self.0
    }

    pub fn payload(&self) -> &[u8] {
        &self.0[1..]
    }
}

/// CODE 2 Pack7 selector 0/1. The adapter supplies actual conversion results.
pub trait NumericServices {
    fn number_to_string(&mut self, value: i32) -> Result<PascalString, RuleError>;
    fn string_to_number(&mut self, value: &PascalString) -> Result<i32, RuleError>;
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TextContext {
    /// Original conversion/string scratch at 0x479ed8.
    pub scratch: PascalString,
    /// Original LONG conversion result at 0x4471dc.
    pub parsed_number: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TextEffect {
    NumberToString(i32),
    StringToNumber(PascalString),
    DrawString(PascalString),
    GetDialogItem { item: i16 },
    GetDialogItemText { item: i16 },
    SetDialogItemText { item: i16, text: PascalString },
    CurrentResourceFile,
    UseResourceFile(i16),
    GetResourceInCurrentFile { resource_type: [u8; 4], id: i16 },
    MoveHandleHigh,
    LockHandle,
    CopyVersionBytes { offset: usize, count: u16 },
    UnlockHandle,
    ReleaseResource,
    TextSize(i16),
    ForeColor(i32),
    TextFace(u8),
    TextFont(i16),
    MoveTo { x: i16, y: i16 },
}

fn format_number(
    context: &mut TextContext,
    value: i32,
    services: &mut (impl NumericServices + ?Sized),
) -> Result<Vec<TextEffect>, RuleError> {
    context.scratch = services.number_to_string(value)?;
    Ok(vec![TextEffect::NumberToString(value)])
}

/// string, 0x1382d0: sign-extend WORD, convert, then DrawString.
pub fn string(
    context: &mut TextContext,
    value: i16,
    services: &mut (impl NumericServices + ?Sized),
) -> Result<Vec<TextEffect>, RuleError> {
    string_long(context, i32::from(value), services)
}

/// stringlong, 0x1382fc: retain the full signed LONG conversion input.
pub fn string_long(
    context: &mut TextContext,
    value: i32,
    services: &mut (impl NumericServices + ?Sized),
) -> Result<Vec<TextEffect>, RuleError> {
    let mut effects = format_number(context, value, services)?;
    effects.push(TextEffect::DrawString(context.scratch.clone()));
    Ok(effects)
}

/// stringplus, 0x13832a: positive WORDs draw a separate '+' before the number.
pub fn string_plus(
    context: &mut TextContext,
    value: i16,
    services: &mut (impl NumericServices + ?Sized),
) -> Result<Vec<TextEffect>, RuleError> {
    let mut effects = format_number(context, i32::from(value), services)?;
    if value > 0 {
        effects.push(TextEffect::DrawString(PascalString(vec![1, b'+'])));
    }
    effects.push(TextEffect::DrawString(context.scratch.clone()));
    Ok(effects)
}

/// stringnozeronoplus, 0x138368: conversion always occurs, drawing only if >0.
pub fn string_no_zero_no_plus(
    context: &mut TextContext,
    value: i16,
    services: &mut (impl NumericServices + ?Sized),
) -> Result<Vec<TextEffect>, RuleError> {
    let mut effects = format_number(context, i32::from(value), services)?;
    if value > 0 {
        effects.push(TextEffect::DrawString(context.scratch.clone()));
    }
    Ok(effects)
}

/// stringnozero, 0x1383a6: both zero and negative WORDs suppress drawing.
pub fn string_no_zero(
    context: &mut TextContext,
    value: i16,
    services: &mut (impl NumericServices + ?Sized),
) -> Result<Vec<TextEffect>, RuleError> {
    let mut effects = format_number(context, i32::from(value), services)?;
    if value > 0 {
        effects.push(TextEffect::DrawString(PascalString(vec![1, b'+'])));
        effects.push(TextEffect::DrawString(context.scratch.clone()));
    }
    Ok(effects)
}

/// Copy_Part_Of_String, 0x146a50. Indices include the source length byte at zero.
pub fn copy_part_of_string(
    destination: &mut [u8],
    source: &[u8],
    first: i16,
    last: i16,
) -> Result<(), RuleError> {
    if destination.is_empty() {
        return Err(RuleError::Invalid("missing destination length byte".into()));
    }
    let range = if first <= last {
        if first < 0 || last == i16::MAX {
            return Err(RuleError::Unsupported {
                action: "text.copy_part_of_string",
                dependency: "negative source addresses or nonterminating signed WORD loop",
            });
        }
        let range = usize::try_from(first).unwrap()..usize::try_from(last).unwrap() + 1;
        if range.end > source.len() || range.len() >= destination.len() {
            return Err(RuleError::Invalid(
                "string copy exceeds supplied buffers".into(),
            ));
        }
        Some(range)
    } else {
        None
    };
    if let Some(range) = range {
        let count = range.len();
        destination[1..count + 1].copy_from_slice(&source[range]);
    }
    destination[0] = last.wrapping_sub(first).wrapping_add(1) as u8;
    Ok(())
}

/// GetVersStr, 0x1270c0: the caller supplies the resource selected by Get1Resource.
pub fn get_version_string(
    destination: &mut PascalString,
    resource_id: i16,
    resource_file: i16,
    current_resource_file: i16,
    resource: &[u8],
) -> Result<(u16, Vec<TextEffect>), RuleError> {
    let text = PascalString::from_raw(resource.get(6..).ok_or_else(|| {
        RuleError::Invalid("vers resource lacks its short-version string".into())
    })?)?;
    let count = text.raw().len() as u16;
    let mut effects = vec![TextEffect::CurrentResourceFile];
    if resource_file != 0 {
        effects.push(TextEffect::UseResourceFile(resource_file));
    }
    effects.extend([
        TextEffect::GetResourceInCurrentFile {
            resource_type: *b"vers",
            id: resource_id,
        },
        TextEffect::MoveHandleHigh,
        TextEffect::LockHandle,
        TextEffect::CopyVersionBytes { offset: 6, count },
        TextEffect::UnlockHandle,
        TextEffect::ReleaseResource,
        TextEffect::UseResourceFile(current_resource_file),
    ]);
    *destination = text;
    Ok((count, effects))
}

/// GetDialogNum, 0x1389e4: preserve LONG result, return its low WORD.
pub fn get_dialog_number(
    context: &mut TextContext,
    item: i16,
    item_text: &PascalString,
    services: &mut (impl NumericServices + ?Sized),
) -> Result<(i16, Vec<TextEffect>), RuleError> {
    let value = services.string_to_number(item_text)?;
    context.scratch = item_text.clone();
    context.parsed_number = value;
    Ok((
        value as i16,
        vec![
            TextEffect::GetDialogItem { item },
            TextEffect::GetDialogItemText { item },
            TextEffect::StringToNumber(item_text.clone()),
        ],
    ))
}

/// DialogNumLong, 0x153cf4: GetDialogItem precedes conversion and SetIText.
pub fn dialog_number_long(
    context: &mut TextContext,
    item: i16,
    value: i32,
    services: &mut (impl NumericServices + ?Sized),
) -> Result<Vec<TextEffect>, RuleError> {
    let mut effects = vec![TextEffect::GetDialogItem { item }];
    effects.extend(format_number(context, value, services)?);
    effects.push(TextEffect::SetDialogItemText {
        item,
        text: context.scratch.clone(),
    });
    Ok(effects)
}

/// DialogNum, 0x153d56: the conversion argument is a sign-extended WORD.
pub fn dialog_number(
    context: &mut TextContext,
    item: i16,
    value: i16,
    services: &mut (impl NumericServices + ?Sized),
) -> Result<Vec<TextEffect>, RuleError> {
    dialog_number_long(context, item, i32::from(value), services)
}

/// DiStr, 0x153daa: text bytes are passed through without numeric conversion.
pub fn dialog_string(item: i16, text: &PascalString) -> Vec<TextEffect> {
    vec![
        TextEffect::GetDialogItem { item },
        TextEffect::SetDialogItemText {
            item,
            text: text.clone(),
        },
    ]
}

/// DialogNumNZ, 0x13a72a: zero clears the item without changing numeric scratch.
pub fn dialog_number_nonzero(
    context: &mut TextContext,
    item: i16,
    value: i16,
    services: &mut (impl NumericServices + ?Sized),
) -> Result<Vec<TextEffect>, RuleError> {
    let mut effects = vec![TextEffect::GetDialogItem { item }];
    let text = if value == 0 {
        PascalString::default()
    } else {
        let input = if value > 0 {
            value.wrapping_neg()
        } else {
            value
        };
        let mut scratch = services.number_to_string(i32::from(input))?;
        if value > 0 {
            let sign = scratch.0.get_mut(1).ok_or_else(|| {
                RuleError::Invalid("positive dialog number conversion has no sign byte".into())
            })?;
            *sign = b'+';
        }
        effects.push(TextEffect::NumberToString(i32::from(input)));
        context.scratch = scratch;
        context.scratch.clone()
    };
    effects.push(TextEffect::SetDialogItemText { item, text });
    Ok(effects)
}

/// WORD globals read by number, 0x14695e; presentation owns the label bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NumberContext {
    pub suppressed: i16,
    pub origin_x: i16,
    pub origin_y: i16,
    pub line: i16,
    pub font: i16,
    pub enemy_total: i16,
    pub enemy_removed: i16,
    pub label: PascalString,
}

pub fn number(
    context: &mut TextContext,
    inputs: &NumberContext,
    services: &mut (impl NumericServices + ?Sized),
) -> Result<Vec<TextEffect>, RuleError> {
    if inputs.suppressed != 0 {
        return Ok(vec![]);
    }
    let y = inputs
        .origin_y
        .wrapping_add(0x1bd)
        .wrapping_sub(inputs.line.wrapping_shl(5));
    let mut effects = vec![
        TextEffect::TextSize(20),
        TextEffect::ForeColor(69),
        TextEffect::TextFace(1),
        TextEffect::TextFont(inputs.font),
        TextEffect::MoveTo {
            x: inputs.origin_x.wrapping_add(0x1ae),
            y,
        },
        TextEffect::DrawString(inputs.label.clone()),
        TextEffect::MoveTo {
            x: inputs.origin_x.wrapping_add(0x244),
            y,
        },
    ];
    effects.extend(string(
        context,
        inputs.enemy_total.wrapping_sub(inputs.enemy_removed),
        services,
    )?);
    Ok(effects)
}
