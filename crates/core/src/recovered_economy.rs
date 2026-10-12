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

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> ExperienceState {
        ExperienceState {
            warriors: (0..6)
                .map(|index| {
                    let mut bytes = vec![0x40 + index; WARRIOR_BYTES];
                    bytes[0x9ce..0x9d2].copy_from_slice(&(-1035_i32).to_be_bytes());
                    bytes[0x9d2..0x9d6].copy_from_slice(&2147481647_i32.to_be_bytes());
                    Warrior {
                        name: format!("Warrior {index}"),
                        original_sha256: format!("synthetic-{index}"),
                        bytes,
                    }
                })
                .collect(),
            global: ExperienceCounters {
                remaining: 2147483547,
                total: i32::MIN,
            },
        }
    }

    #[test]
    fn itemcost_preserves_absolute_signed_rate_and_nonzero_mode_branches() {
        for (cost, percent, mode, applied, expected) in [
            (-12345, 25, 0, 25, 3086),
            (12345, 125, 0, 125, 15431),
            (12345, 125, 1, 100, 12345),
            (12345, 125, i16::MIN, 100, 12345),
            (12345, -25, 1, -25, -3086),
            (100, i16::MIN, 1, i16::MIN, -32768),
            (i32::MIN, -100, 1, -100, i32::MIN),
            (100, 33, 0, 33, 33),
        ] {
            let result = itemcost(ItemCostInput {
                cost,
                percent,
                mode,
            })
            .unwrap();
            assert_eq!((result.cost, result.applied_percent), (expected, applied));
            assert_eq!(
                result.qualification,
                ArithmeticQualification::SourceDerivedBinary32
            );
        }
    }

    #[test]
    fn itemcost_keeps_single_stages_and_diagnoses_integer_conversion_overflow() {
        let result = itemcost(ItemCostInput {
            cost: -12345,
            percent: 25,
            mode: 0,
        })
        .unwrap();
        assert_eq!(
            (
                result.absolute_single_bits,
                result.scaled_single_bits,
                result.divided_single_bits
            ),
            (0x4640e400, 0x4896b220, 0x4540e400)
        );
        assert_eq!(
            itemcost(ItemCostInput {
                cost: 16777217,
                percent: 100,
                mode: 0
            })
            .unwrap()
            .cost,
            16777216
        );
        assert_eq!(
            itemcost(ItemCostInput {
                cost: i32::MAX,
                percent: 1,
                mode: 0
            })
            .unwrap()
            .cost,
            21474836
        );
        assert!(matches!(
            itemcost(ItemCostInput {
                cost: i32::MAX,
                percent: 100,
                mode: 1
            }),
            Err(RuleError::Unsupported {
                action: "economy.itemcost",
                ..
            })
        ));
        let zero = itemcost(ItemCostInput {
            cost: 0,
            percent: -1,
            mode: 0,
        })
        .unwrap();
        assert_eq!(
            (zero.cost, zero.scaled_single_bits, zero.divided_single_bits),
            (0, 0x80000000, 0x80000000)
        );
    }

    #[test]
    fn getexp_writes_selected_party_record_in_native_order_with_long_wrap() {
        let before = state();
        let output = getexp(
            &before,
            ExperienceInput {
                level_count: 3,
                recipient: 2,
                update_total: 1,
            },
        )
        .unwrap();
        let mut expected = before.clone();
        expected.warriors[2].bytes[0x9ce..0x9d2].copy_from_slice(&3621_i32.to_be_bytes());
        expected.warriors[2].bytes[0x9d2..0x9d6].copy_from_slice(&(-2147480993_i32).to_be_bytes());
        assert_eq!(output.state, expected);
        assert_eq!(before, state());
        let projected: Vec<_> = output
            .writes
            .iter()
            .map(|write| {
                (
                    write.iteration,
                    write.field,
                    write.increment,
                    write.before,
                    write.after,
                )
            })
            .collect();
        assert_eq!(
            projected,
            [
                (0, ExperienceField::Remaining, 1035, -1035, 0),
                (0, ExperienceField::Total, 1035, 2147481647, 2147482682),
                (1, ExperienceField::Remaining, 1490, 0, 1490),
                (1, ExperienceField::Total, 1490, 2147482682, -2147483124),
                (2, ExperienceField::Remaining, 2131, 1490, 3621),
                (2, ExperienceField::Total, 2131, -2147483124, -2147480993),
            ]
        );
        assert!(
            output
                .writes
                .iter()
                .all(|write| write.recipient == ExperienceRecipient::Warrior(2))
        );
    }

    #[test]
    fn getexp_negative_recipient_and_raw_word_flag_select_global_counters() {
        let before = state();
        let output = getexp(
            &before,
            ExperienceInput {
                level_count: 1,
                recipient: i16::MIN,
                update_total: -2,
            },
        )
        .unwrap();
        assert_eq!(output.state.warriors, before.warriors);
        assert_eq!(
            output.state.global,
            ExperienceCounters {
                remaining: -2147482714,
                total: -2147482613
            }
        );
        assert_eq!(output.writes.len(), 2);
        assert!(
            output
                .writes
                .iter()
                .all(|write| write.recipient == ExperienceRecipient::Global)
        );
        let output = getexp(
            &before,
            ExperienceInput {
                level_count: 1,
                recipient: -1,
                update_total: 0,
            },
        )
        .unwrap();
        assert_eq!(output.state.global.total, i32::MIN);
        assert_eq!(output.writes.len(), 1);
        let output = getexp(
            &before,
            ExperienceInput {
                level_count: 1,
                recipient: 0,
                update_total: 0,
            },
        )
        .unwrap();
        let mut expected = before.clone();
        expected.warriors[0].bytes[0x9ce..0x9d2].copy_from_slice(&0_i32.to_be_bytes());
        assert_eq!(output.state, expected);
        assert_eq!(output.writes.len(), 1);
    }

    #[test]
    fn getexp_extends_the_source_floor_tail_and_keeps_wrapping_totals() {
        let mut before = state();
        before.global = ExperienceCounters {
            remaining: 0,
            total: 0,
        };
        let output = getexp(
            &before,
            ExperienceInput {
                level_count: 66,
                recipient: -1,
                update_total: 1,
            },
        )
        .unwrap();
        assert_eq!(
            output.state.global,
            ExperienceCounters {
                remaining: -1991713572,
                total: -1991713572
            }
        );
        assert_eq!(output.writes.len(), 132);
        assert_eq!(output.writes[128].increment, 261617360);
        assert_eq!(output.writes[128].after, 2002393756);
        assert_eq!(output.writes[130].increment, 300859968);
        let output = getexp(
            &before,
            ExperienceInput {
                level_count: 80,
                recipient: -1,
                update_total: 0,
            },
        )
        .unwrap();
        assert_eq!(output.state.global.remaining, -862438500);
        assert_eq!(output.writes.last().unwrap().increment, 2128795776);
    }

    #[test]
    fn getexp_no_iteration_skips_target_validation_and_errors_leave_input_unchanged() {
        let before = state();
        for level_count in [0, -1, i16::MIN] {
            let output = getexp(
                &before,
                ExperienceInput {
                    level_count,
                    recipient: i16::MAX,
                    update_total: 1,
                },
            )
            .unwrap();
            assert_eq!(output.state, before);
            assert!(output.writes.is_empty());
        }
        for (level_count, recipient) in [(1, 6), (1, i16::MAX), (81, 0), (i16::MAX, -1)] {
            assert!(
                getexp(
                    &before,
                    ExperienceInput {
                        level_count,
                        recipient,
                        update_total: 1
                    }
                )
                .is_err()
            );
            assert_eq!(before, state());
        }
        let mut incomplete = before;
        incomplete.warriors[0].bytes.truncate(0x9d6);
        let snapshot = incomplete.clone();
        assert!(
            getexp(
                &incomplete,
                ExperienceInput {
                    level_count: 1,
                    recipient: 0,
                    update_total: 1
                }
            )
            .is_err()
        );
        assert_eq!(incomplete, snapshot);
    }
}
