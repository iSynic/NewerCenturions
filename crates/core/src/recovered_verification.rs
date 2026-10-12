//! Warrior verification fields and the reconstruction's registered policy.

use crate::{RuleError, recovered_install::InstallMemory, recovered_records::RecordWrite};

/// doreg, 0013a6d4: select the recovered registered branch by project policy.
pub fn doreg() -> i32 {
    i32::from(crate::story_registration::registration_word_accepted(1))
}

/// doreg3, 001469f8: the same registered behavior, without imported-code gates.
pub fn doreg3() -> i32 {
    doreg()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerificationPlan {
    pub memory: InstallMemory,
    pub writes: Vec<RecordWrite>,
    pub native_return: u32,
}

fn unavailable() -> RuleError {
    RuleError::Unsupported {
        action: "warrior.verification_fields",
        dependency: "reached native WORD memory is unavailable",
    }
}

fn offset(memory: &InstallMemory, address: u32) -> Result<usize, RuleError> {
    let start = address.checked_sub(memory.base_address).map(|n| n as usize);
    match start {
        Some(start)
            if start
                .checked_add(2)
                .is_some_and(|end| end <= memory.bytes.len()) =>
        {
            Ok(start)
        }
        _ => Err(unavailable()),
    }
}

fn word(memory: &InstallMemory, address: u32) -> Result<u16, RuleError> {
    let start = offset(memory, address)?;
    Ok(u16::from_be_bytes([
        memory.bytes[start],
        memory.bytes[start + 1],
    ]))
}

fn put(plan: &mut VerificationPlan, address: u32, value: u16) -> Result<(), RuleError> {
    let start = offset(&plan.memory, address)?;
    let bytes = value.to_be_bytes();
    plan.memory.bytes[start..start + 2].copy_from_slice(&bytes);
    plan.writes.push(RecordWrite {
        address,
        bytes: bytes.to_vec(),
    });
    Ok(())
}

fn sum(memory: &InstallMemory, base: u32, fields: &[u32]) -> Result<u16, RuleError> {
    fields.iter().try_fold(0_u16, |value, field| {
        Ok(value.wrapping_add(word(memory, base.wrapping_add(*field))?))
    })
}

/// setverify, 0013aeb8. The ordinary and ffff formulas intentionally differ.
pub fn setverify(memory: &InstallMemory, member: u16) -> Result<VerificationPlan, RuleError> {
    let mut plan = VerificationPlan {
        memory: memory.clone(),
        writes: vec![],
        native_return: 0,
    };
    let base = if member == u16::MAX {
        0x44721c
    } else {
        0x448aa0 + u32::from(member) * 3138
    };
    for (destination, fields) in [
        (2, &[0x3a, 0x52, 0x1e][..]),
        (
            0x124,
            if member == u16::MAX {
                &[6, 0x5a, 0x9e2, 0x9e4][..]
            } else {
                &[6, 0x5a, 0x9e4, 0x9e6][..]
            },
        ),
        (0xa40, &[0xa, 0x52, 0x9e8, 0x52, 0x22][..]),
    ] {
        let value = sum(&plan.memory, base, fields)?;
        put(&mut plan, base + destination, value)?;
    }
    if member != u16::MAX {
        for entry in 0..10_u32 {
            for field in [0, 2, 4, 6] {
                let source = 0x43788a + u32::from(member) * 80 + entry * 8 + field;
                let value = word(&plan.memory, source)?;
                put(&mut plan, base + 0xa76 + entry * 8 + field, value)?;
            }
        }
    }
    Ok(plan)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VerificationResult {
    pub mismatch: bool,
    /// The source's last MULU leaves its upper WORD in D0.
    pub native_d0: u32,
}

/// checkverify, 0013b0ea. This diagnostic is never a registration requirement.
pub fn checkverify(memory: &InstallMemory, member: u16) -> Result<VerificationResult, RuleError> {
    let product = u32::from(member) * 3138;
    let base = 0x448aa0 + product;
    let mut mismatch = false;
    for (stored, fields) in [
        (2, &[0x3a, 0x52, 0x1e][..]),
        (0x124, &[6, 0x5a, 0x9e2, 0x9e4][..]),
        (0xa40, &[0xa, 0x52, 0x9e8, 0x52, 0x22][..]),
    ] {
        // All three comparisons execute even after an earlier mismatch.
        mismatch |= sum(memory, base, fields)? != word(memory, base + stored)?;
    }
    Ok(VerificationResult {
        mismatch,
        native_d0: (product & 0xffff0000) | u32::from(mismatch),
    })
}
