//! The two attack2 condition layouts. Enemy counters are signed bytes; warrior
//! counters are signed words. Rule order and duration caps belong to the caller.
use super::{Condition, DamageTargetKind, put_word};
use crate::{ContentError, span, word};

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum SavingThrow {
    Mental = 0,
    Chemical = 1,
    Heat = 2,
    Cold = 3,
    Electrical = 4,
    Psionic = 5,
    Atomic = 6,
    Biological = 7,
    Entanglement = 8,
}

impl SavingThrow {
    pub fn from_index(index: i16) -> Option<Self> {
        Some(match index {
            0 => Self::Mental,
            1 => Self::Chemical,
            2 => Self::Heat,
            3 => Self::Cold,
            4 => Self::Electrical,
            5 => Self::Psionic,
            6 => Self::Atomic,
            7 => Self::Biological,
            8 => Self::Entanglement,
            _ => return None,
        })
    }
}

pub struct ConditionTarget<'a> {
    bytes: &'a mut [u8],
    kind: DamageTargetKind,
}

impl<'a> ConditionTarget<'a> {
    pub fn new(bytes: &'a mut [u8], kind: DamageTargetKind) -> Result<Self, ContentError> {
        span(
            bytes,
            0,
            if kind == DamageTargetKind::Enemy {
                288
            } else {
                3138
            },
        )?;
        Ok(Self { bytes, kind })
    }
    pub fn is_enemy(&self) -> bool {
        self.kind == DamageTargetKind::Enemy
    }
    pub fn is_cyborg(&self) -> Result<bool, ContentError> {
        Ok(!self.is_enemy() && word(self.bytes, 0x32)? == 3)
    }
    pub fn magic_immunity(&self) -> Result<i16, ContentError> {
        word(self.bytes, 0xa)
    }
    pub fn faction(&self) -> Result<i16, ContentError> {
        word(self.bytes, if self.is_enemy() { 0xe } else { 0x10 })
    }
    pub fn set_faction(&mut self, faction: i16) {
        let offset = if self.is_enemy() { 0xe } else { 0x10 };
        put_word(self.bytes, offset, faction);
    }
    pub fn saving_throw(&self, save: SavingThrow) -> Result<i16, ContentError> {
        if self.is_enemy() {
            Ok(i16::from(self.bytes[0x8b + save as usize] as i8))
        } else {
            word(self.bytes, 0x112 + save as usize * 2)
        }
    }
    pub fn duration(&self, condition: Condition) -> Result<i16, ContentError> {
        self.slot_duration(condition as usize)
    }
    pub fn set_duration(&mut self, condition: Condition, duration: i16) {
        self.set_slot_duration(condition as usize, duration);
    }
    /// Condition slots by raw index, for data-selected slots such as weapon conditions.
    pub fn slot_duration(&self, slot: usize) -> Result<i16, ContentError> {
        if self.is_enemy() {
            Ok(i16::from(span(self.bytes, 0xb2 + slot, 1)?[0] as i8))
        } else {
            word(self.bytes, 0x5e + slot * 2)
        }
    }
    pub fn set_slot_duration(&mut self, slot: usize, duration: i16) {
        if self.is_enemy() {
            self.bytes[0xb2 + slot] = duration as u8;
        } else {
            put_word(self.bytes, 0x5e + slot * 2, duration);
        }
    }
    pub fn halve_remaining_turn(&mut self) -> Result<(), ContentError> {
        if self.is_enemy() {
            for offset in [0x109, 0x106] {
                self.bytes[offset] = ((self.bytes[offset] as i8) / 2) as u8;
            }
        } else {
            for offset in [0x3c, 0x40] {
                put_word(self.bytes, offset, word(self.bytes, offset)? / 2);
            }
        }
        Ok(())
    }
    pub fn clear_remaining_turn(&mut self) {
        if self.is_enemy() {
            self.bytes[0x109] = 0;
            self.bytes[0x106] = 0;
        } else {
            put_word(self.bytes, 0x3c, 0);
            put_word(self.bytes, 0x40, 0);
        }
    }
    pub fn elemental_protection(&self, element: usize) -> Result<bool, ContentError> {
        if self.is_enemy() {
            Ok(self.bytes[0xbb + element] != 0)
        } else {
            Ok(word(self.bytes, 0x70 + element * 2)? != 0)
        }
    }
    pub fn psi_points(&self) -> Result<i16, ContentError> {
        word(self.bytes, if self.is_enemy() { 0x10e } else { 0x58 })
    }
    pub fn set_psi_points(&mut self, points: i16) {
        let offset = if self.is_enemy() { 0x10e } else { 0x58 };
        put_word(self.bytes, offset, points);
    }
    /// Warrior experience drain: remaining experience gains what upgrade points lose.
    pub fn shift_experience(&mut self, amount: i32) -> Result<(), ContentError> {
        span(self.bytes, 0x9ce, 8)?;
        for (offset, delta) in [(0x9ce, amount), (0x9d2, amount.wrapping_neg())] {
            let value = i32::from_be_bytes(self.bytes[offset..offset + 4].try_into().unwrap());
            self.bytes[offset..offset + 4]
                .copy_from_slice(&value.wrapping_add(delta).to_be_bytes());
        }
        Ok(())
    }
}
