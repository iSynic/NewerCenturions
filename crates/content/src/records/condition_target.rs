//! Conditions and saving throws of an attack's target, warrior or monster.
//! Monster durations are signed bytes; warrior durations are signed words.
//! The caller owns rule order and duration caps.
use super::warrior_layout::{CONDITIONS, REMAINING_VICTORY_POINTS, UPGRADE_POINTS};
use super::{Condition, DamageTargetKind, put_word};
use crate::bytes::add_long_wrapping;
use crate::{ContentError, span, word};

/// The nine kinds of attack a target gets a saving throw against.
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
    /// The saving throw numbered `index`, if there is one.
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

/// A warrior or monster record seen as the target of conditions and saving throws.
pub struct ConditionTarget<'a> {
    bytes: &'a mut [u8],
    kind: DamageTargetKind,
}

impl<'a> ConditionTarget<'a> {
    /// Views `bytes` as a target of the given kind; a record too short is refused.
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
    /// Whether the target is a monster.
    pub fn is_enemy(&self) -> bool {
        self.kind == DamageTargetKind::Enemy
    }
    /// Whether the target is a warrior of the Cyborg race.
    pub fn is_cyborg(&self) -> Result<bool, ContentError> {
        Ok(!self.is_enemy() && word(self.bytes, 0x32)? == 3)
    }
    /// The monster's resistance to scans (its Psi resistance); a warrior's lies elsewhere in its
    /// record.
    pub fn psi_resistance(&self) -> Result<i16, ContentError> {
        word(self.bytes, 0xa)
    }
    /// Which side the target fights on; zero is the party's side, nonzero the opposing side.
    pub fn faction(&self) -> Result<i16, ContentError> {
        word(self.bytes, if self.is_enemy() { 0xe } else { 0x10 })
    }
    /// Moves the target to a side.
    pub fn set_faction(&mut self, faction: i16) {
        let offset = if self.is_enemy() { 0xe } else { 0x10 };
        put_word(self.bytes, offset, faction);
    }
    /// The target's saving throw against one kind of attack.
    pub fn save(&self, save: SavingThrow) -> Result<i16, ContentError> {
        if self.is_enemy() {
            Ok(i16::from(self.bytes[0x8b + save as usize] as i8))
        } else {
            word(self.bytes, 0x112 + save as usize * 2)
        }
    }
    /// Remaining duration of `condition`; a negative duration is permanent.
    pub fn duration(&self, condition: Condition) -> Result<i16, ContentError> {
        self.slot_duration(condition as usize)
    }
    /// Sets how long `condition` lasts.
    pub fn set_duration(&mut self, condition: Condition, duration: i16) {
        self.set_slot_duration(condition as usize, duration);
    }
    /// A condition slot by index, for slots that data selects, such as weapon conditions.
    pub fn slot_duration(&self, slot: usize) -> Result<i16, ContentError> {
        if self.is_enemy() {
            Ok(i16::from(span(self.bytes, 0xb2 + slot, 1)?[0] as i8))
        } else {
            word(self.bytes, CONDITIONS + slot * 2)
        }
    }
    /// Sets how long condition slot `slot` lasts.
    pub fn set_slot_duration(&mut self, slot: usize, duration: i16) {
        if self.is_enemy() {
            self.bytes[0xb2 + slot] = duration as u8;
        } else {
            put_word(self.bytes, CONDITIONS + slot * 2, duration);
        }
    }
    /// Halves the movement and attacks the target has left this turn.
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
    /// Takes away the movement and attacks the target has left this turn.
    pub fn clear_remaining_turn(&mut self) {
        if self.is_enemy() {
            self.bytes[0x109] = 0;
            self.bytes[0x106] = 0;
        } else {
            put_word(self.bytes, 0x3c, 0);
            put_word(self.bytes, 0x40, 0);
        }
    }
    /// Whether the target is protected from damage of element `element`.
    pub fn elemental_protection(&self, element: usize) -> Result<bool, ContentError> {
        if self.is_enemy() {
            Ok(self.bytes[0xbb + element] != 0)
        } else {
            Ok(word(self.bytes, 0x70 + element * 2)? != 0)
        }
    }
    /// The target's current Psi points.
    pub fn psi_points(&self) -> Result<i16, ContentError> {
        word(self.bytes, if self.is_enemy() { 0x10e } else { 0x58 })
    }
    /// Sets the target's current Psi points.
    pub fn set_psi_points(&mut self, points: i16) {
        let offset = if self.is_enemy() { 0x10e } else { 0x58 };
        put_word(self.bytes, offset, points);
    }
    /// Drains a warrior's victory points: remaining victory points gain what upgrade points lose.
    pub fn shift_victory_points(&mut self, amount: i32) -> Result<(), ContentError> {
        span(self.bytes, REMAINING_VICTORY_POINTS, 8)?;
        for (offset, delta) in [
            (REMAINING_VICTORY_POINTS, amount),
            (UPGRADE_POINTS, amount.wrapping_neg()),
        ] {
            add_long_wrapping(self.bytes, offset, delta);
        }
        Ok(())
    }
}
