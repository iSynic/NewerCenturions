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

#[cfg(test)]
mod tests {
    use super::*;

    fn memory() -> InstallMemory {
        InstallMemory {
            base_address: 0x437000,
            bytes: vec![0xa5; 0x13000],
        }
    }
    fn seed_word(memory: &mut InstallMemory, address: u32, value: u16) {
        let start = offset(memory, address).unwrap();
        memory.bytes[start..start + 2].copy_from_slice(&value.to_be_bytes());
    }
    fn seed_fields(memory: &mut InstallMemory, base: u32) {
        for (at, value) in [
            (0x3a, 0xffff),
            (0x52, 4),
            (0x1e, 3),
            (6, 8),
            (0x5a, 9),
            (0x9e2, 10),
            (0x9e4, 11),
            (0x9e6, 12),
            (0xa, 13),
            (0x9e8, 14),
            (0x22, 15),
        ] {
            seed_word(memory, base + at, value);
        }
    }

    #[test]
    fn ordinary_formulas_preserve_native_set_check_difference_and_ordered_profile_copy() {
        let mut memory = memory();
        seed_fields(&mut memory, 0x448aa0);
        for index in 0..40 {
            seed_word(&mut memory, 0x43788a + index * 2, index as u16);
        }
        let original = memory.clone();
        let plan = setverify(&memory, 0).unwrap();
        assert_eq!(memory, original);
        assert_eq!(plan.writes.len(), 43);
        assert_eq!(word(&plan.memory, 0x448aa2).unwrap(), 6);
        assert_eq!(word(&plan.memory, 0x448bc4).unwrap(), 40);
        assert_eq!(word(&plan.memory, 0x4494e0).unwrap(), 50);
        assert_eq!(word(&plan.memory, 0x449516).unwrap(), 0);
        assert_eq!(word(&plan.memory, 0x449564).unwrap(), 39);
        assert!(checkverify(&plan.memory, 0).unwrap().mismatch);
        let mut corrected = plan.memory.clone();
        seed_word(&mut corrected, 0x448bc4, 38);
        assert_eq!(checkverify(&corrected, 0).unwrap().native_d0, 0);
    }

    #[test]
    fn sentinel_only_writes_three_words_and_missing_copy_source_is_atomic() {
        let mut memory = memory();
        seed_fields(&mut memory, 0x44721c);
        let plan = setverify(&memory, u16::MAX).unwrap();
        assert_eq!(plan.writes.len(), 3);
        assert_eq!(word(&plan.memory, 0x447340).unwrap(), 38);
        let mut missing = InstallMemory {
            base_address: 0x448aa0,
            bytes: vec![0; 3138],
        };
        seed_fields(&mut missing, 0x448aa0);
        let before = missing.clone();
        assert!(setverify(&missing, 0).is_err());
        assert_eq!(missing, before);
    }
}
