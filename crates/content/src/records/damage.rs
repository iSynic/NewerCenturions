use super::put_word;
use crate::{ContentError, word};

/// The damage fields of a spell info packet. Reading them leaves the packet
/// unchanged.
pub struct DamageDefinition {
    pub physical: (i16, i16),
    pub per_attack: (i16, i16),
    pub elemental_maximum: [u8; 7],
    pub scale_by_target_size: bool,
    pub healing: bool,
    /// The saving-throw mode: above one, the target gets no saving throw.
    pub cannot: i8,
    pub saving_throw_bonus: i8,
    pub saving_throw_bonus_per_attack: i8,
}

impl DamageDefinition {
    /// Reads the damage fields of a 58-byte spell info packet.
    pub fn read(packet: &[u8]) -> Result<Self, ContentError> {
        crate::span(packet, 0, 58)?;
        Ok(Self {
            physical: (word(packet, 32)?, word(packet, 34)?),
            per_attack: (i16::from(packet[11] as i8), i16::from(packet[12] as i8)),
            elemental_maximum: packet[23..30].try_into().unwrap(),
            scale_by_target_size: packet[42] != 0,
            healing: word(packet, 40)? == 57,
            cannot: packet[9] as i8,
            saving_throw_bonus: packet[6] as i8,
            saving_throw_bonus_per_attack: packet[8] as i8,
        })
    }
}

/// Whether the target of damage is a warrior or a monster.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DamageTargetKind {
    Warrior,
    Enemy,
}

/// A target's defenses, read from a warrior or monster record. Values are
/// signed and unbounded; the damage rules apply caps and exceptions. Elements
/// run chemical, heat, cold, electrical, Psi, atomic, biological, as in the
/// spell info packet.
pub struct DamageDefenses {
    pub kind: DamageTargetKind,
    pub size: i16,
    pub armor: i16,
    pub shield_available: bool,
    pub shield_chance: i16,
    pub shield_charge: i16,
    pub shield_cost: i16,
    pub saves: [i16; 7],
    pub protected: [bool; 7],
    pub biological_immunity: bool,
    pub race: Option<i16>,
}

/// A warrior or monster record seen as the target of damage.
pub struct DamageTargetRecord<'a> {
    bytes: &'a mut [u8],
    kind: DamageTargetKind,
}

impl<'a> DamageTargetRecord<'a> {
    /// Views `bytes` as a target of the given kind.
    pub fn new(bytes: &'a mut [u8], kind: DamageTargetKind) -> Self {
        Self { bytes, kind }
    }

    /// Reads the target's defenses.
    pub fn defenses(&self) -> Result<DamageDefenses, ContentError> {
        let enemy = self.kind == DamageTargetKind::Enemy;
        let mut saves = [0; 7];
        let mut protected = [false; 7];
        for element in 0..7 {
            saves[element] = if enemy {
                i16::from(self.bytes[0x8c + element] as i8)
            } else {
                word(self.bytes, 0x114 + element * 2)?
            };
            protected[element] = if enemy {
                self.bytes[0xbb + element] != 0
            } else {
                word(self.bytes, 0x70 + element * 2)? != 0
            };
        }
        Ok(DamageDefenses {
            kind: self.kind,
            size: if enemy { word(self.bytes, 0x10)? } else { 0 },
            armor: word(self.bytes, if enemy { 8 } else { 0x2c })?,
            shield_available: !enemy || self.bytes[0xb0] != 0xff,
            shield_chance: word(self.bytes, if enemy { 0x56 } else { 0x24 })?,
            shield_charge: word(self.bytes, if enemy { 0x58 } else { 0x26 })?,
            shield_cost: if enemy {
                i16::from(self.bytes[0x10a] as i8)
            } else {
                word(self.bytes, 0x2a)?
            },
            saves,
            protected,
            biological_immunity: enemy && self.bytes[0xab] != 0,
            race: if enemy {
                None
            } else {
                Some(word(self.bytes, 0x32)?)
            },
        })
    }

    /// Sets the charge left in the target's power shield; a warrior's worn battery
    /// follows it.
    pub fn set_shield_charge(&mut self, charge: i16) {
        match self.kind {
            DamageTargetKind::Enemy => put_word(self.bytes, 0x58, charge),
            DamageTargetKind::Warrior => {
                put_word(self.bytes, 0x26, charge);
                // The worn battery's charge follows the shield.
                put_word(self.bytes, 0x160, charge);
            }
        }
    }

    /// Marks the target as attacked this round.
    pub fn mark_been_attacked(&mut self) {
        match self.kind {
            DamageTargetKind::Enemy => self.bytes[0x105] = 1,
            DamageTargetKind::Warrior => put_word(self.bytes, 0x14, 1),
        }
    }
}
