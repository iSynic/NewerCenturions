use crate::{RuleError, story_promotion::experience_threshold};
use centurions_content::{WARRIOR_BYTES, Warrior, mission::long};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArithmeticQualification {
    SourceDerivedBinary32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ItemCostInput {
    pub cost: i32,
    pub percent: i16,
    pub mode: i16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ItemCostOutput {
    pub cost: i32,
    pub applied_percent: i16,
    pub absolute_single_bits: u32,
    pub scaled_single_bits: u32,
    pub divided_single_bits: u32,
    pub qualification: ArithmeticQualification,
}

/// CODE 1 itemcost, 0x13c018: the mode and percentage are signed WORDs.
pub fn itemcost(input: ItemCostInput) -> Result<ItemCostOutput, RuleError> {
    let absolute = (input.cost as f32).abs();
    let applied_percent = if input.mode == 0 {
        input.percent
    } else {
        input.percent.min(100)
    };
    let scaled = (f64::from(absolute) * f64::from(applied_percent)) as f32;
    let divided = (f64::from(scaled) / 100.0) as f32;
    Ok(ItemCostOutput {
        cost: truncate_single(divided, "economy.itemcost")?,
        applied_percent,
        absolute_single_bits: absolute.to_bits(),
        scaled_single_bits: scaled.to_bits(),
        divided_single_bits: divided.to_bits(),
        qualification: ArithmeticQualification::SourceDerivedBinary32,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExperienceCounters {
    pub remaining: i32,
    pub total: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExperienceState {
    pub warriors: Vec<Warrior>,
    pub global: ExperienceCounters,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExperienceInput {
    pub level_count: i16,
    pub recipient: i16,
    pub update_total: i16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExperienceRecipient {
    Global,
    Warrior(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExperienceField {
    Remaining,
    Total,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExperienceWrite {
    pub iteration: i16,
    pub recipient: ExperienceRecipient,
    pub field: ExperienceField,
    pub increment: i32,
    pub before: i32,
    pub after: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExperienceOutput {
    pub state: ExperienceState,
    pub writes: Vec<ExperienceWrite>,
    pub qualification: ArithmeticQualification,
}

/// CODE 1 getexp, 0x1380ac: each increment writes remaining before optional total.
pub fn getexp(
    state: &ExperienceState,
    input: ExperienceInput,
) -> Result<ExperienceOutput, RuleError> {
    if input.level_count <= 0 {
        return Ok(ExperienceOutput {
            state: state.clone(),
            writes: vec![],
            qualification: ArithmeticQualification::SourceDerivedBinary32,
        });
    }
    let recipient = if input.recipient < 0 {
        ExperienceRecipient::Global
    } else {
        let index = input.recipient as usize;
        let warrior = state
            .warriors
            .get(index)
            .filter(|_| index < 6)
            .ok_or_else(|| {
                RuleError::Invalid("experience recipient outside party records".into())
            })?;
        if warrior.bytes.len() != WARRIOR_BYTES {
            return Err(RuleError::Invalid(
                "experience recipient has an incomplete warrior record".into(),
            ));
        }
        ExperienceRecipient::Warrior(index)
    };
    let increments = experience_increments(input.level_count)?;
    let mut next = state.clone();
    let mut writes = Vec::new();
    for (iteration, increment) in increments.into_iter().enumerate() {
        credit(
            &mut next,
            recipient,
            ExperienceField::Remaining,
            iteration as i16,
            increment,
            &mut writes,
        )?;
        if input.update_total != 0 {
            credit(
                &mut next,
                recipient,
                ExperienceField::Total,
                iteration as i16,
                increment,
                &mut writes,
            )?;
        }
    }
    Ok(ExperienceOutput {
        state: next,
        writes,
        qualification: ArithmeticQualification::SourceDerivedBinary32,
    })
}

fn experience_increments(level_count: i16) -> Result<Vec<i32>, RuleError> {
    let mut increments = Vec::new();
    let mut previous = 0_i32;
    for level in 1..=level_count.min(64) {
        let total = experience_threshold(level)?;
        increments.push(total.wrapping_sub(previous));
        previous = total;
    }
    if level_count > 64 {
        // By step 64 binary32 spacing exceeds one, so truncation loses no seed fraction.
        let mut running = increments[63] as f32;
        // The source clamps every factor in this suffix to SINGLE 0x3f933333.
        let factor = f32::from_bits(0x3f933333);
        for _ in 64..level_count {
            running = (f64::from(running) * f64::from(factor)) as f32;
            increments.push(truncate_single(running, "economy.getexp")?);
        }
    }
    Ok(increments)
}

fn credit(
    state: &mut ExperienceState,
    recipient: ExperienceRecipient,
    field: ExperienceField,
    iteration: i16,
    increment: i32,
    writes: &mut Vec<ExperienceWrite>,
) -> Result<(), RuleError> {
    let (before, after) = match recipient {
        ExperienceRecipient::Global => {
            let counter = match field {
                ExperienceField::Remaining => &mut state.global.remaining,
                ExperienceField::Total => &mut state.global.total,
            };
            let before = *counter;
            *counter = counter.wrapping_add(increment);
            (before, *counter)
        }
        ExperienceRecipient::Warrior(index) => {
            let offset = match field {
                ExperienceField::Remaining => 0x9ce,
                ExperienceField::Total => 0x9d2,
            };
            let bytes = &mut state.warriors[index].bytes;
            let before = long(bytes, offset)?;
            let after = before.wrapping_add(increment);
            bytes[offset..offset + 4].copy_from_slice(&after.to_be_bytes());
            (before, after)
        }
    };
    writes.push(ExperienceWrite {
        iteration,
        recipient,
        field,
        increment,
        before,
        after,
    });
    Ok(())
}

fn truncate_single(value: f32, action: &'static str) -> Result<i32, RuleError> {
    let wide = f64::from(value);
    if !wide.is_finite() || wide < f64::from(i32::MIN) || wide > f64::from(i32::MAX) {
        return Err(RuleError::Unsupported {
            action,
            dependency: "SANE FLOAT-to-LONG exception result outside signed 32-bit range",
        });
    }
    Ok(value as i32)
}
