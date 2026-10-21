use super::put_word;
use crate::{ContentError, word};

/// Item fields consumed by prepattack14c6c2. These are modifiers, not a prepared
/// attack: ammunition overrides and burst/enhancement arithmetic belong to rules.
pub struct WeaponAttackModifiers {
    pub count: u8,
    pub target_filter: u8,
    pub health_effect: u8,
    pub range: i16,
    pub duration: u8,
    pub physical_maximum: i16,
    pub elemental_maximum: [i16; 7],
    pub launch_sound: i16,
    pub impact_sound: i16,
    pub target_mode: i16,
    pub area: i16,
    pub rounds: i16,
    pub enhancement: i8,
}

impl WeaponAttackModifiers {
    pub fn read(bytes: &[u8]) -> Result<Self, ContentError> {
        let mut elemental_maximum = [0; 7];
        for (value, offset) in elemental_maximum
            .iter_mut()
            .zip([140, 64, 66, 68, 70, 72, 74])
        {
            *value = word(bytes, offset)?;
        }
        Ok(Self {
            count: bytes[77],
            target_filter: bytes[55],
            health_effect: bytes[53],
            range: word(bytes, 60)?,
            duration: bytes[81],
            physical_maximum: word(bytes, 62)?,
            elemental_maximum,
            launch_sound: word(bytes, 12)?,
            impact_sound: word(bytes, 14)?,
            target_mode: word(bytes, 56)?,
            area: word(bytes, 58)?,
            rounds: word(bytes, 22)?,
            enhancement: bytes[85] as i8,
        })
    }
}

/// Only the fields written by weapon preparation. Encoding updates the supplied
/// template in place, preserving all other attack bytes and their saved meaning.
pub struct WeaponAttackParameters {
    pub count: u8,
    pub target_filter: u8,
    pub health_effect: u8,
    pub range: u8,
    pub duration: u8,
    pub physical: (i16, i16),
    pub elemental_maximum: [u8; 7],
    pub launch_sound: i16,
    pub impact_sound: i16,
    pub target_mode: u8,
    pub area: u8,
}

impl WeaponAttackParameters {
    pub fn read(packet: &[u8]) -> Result<Self, ContentError> {
        Ok(Self {
            count: packet[0],
            target_mode: packet[1],
            range: packet[2],
            duration: packet[5],
            target_filter: packet[17],
            health_effect: packet[18],
            elemental_maximum: packet[23..30].try_into().unwrap(),
            area: packet[31],
            physical: (word(packet, 32)?, word(packet, 34)?),
            launch_sound: word(packet, 36)?,
            impact_sound: word(packet, 38)?,
        })
    }

    pub fn write(&self, packet: &mut [u8]) {
        packet[0] = self.count;
        packet[1] = self.target_mode;
        packet[2] = self.range;
        packet[5] = self.duration;
        packet[17] = self.target_filter;
        packet[18] = self.health_effect;
        packet[23..30].copy_from_slice(&self.elemental_maximum);
        packet[31] = self.area;
        put_word(packet, 32, self.physical.0);
        put_word(packet, 34, self.physical.1);
        put_word(packet, 36, self.launch_sound);
        put_word(packet, 38, self.impact_sound);
    }
}
