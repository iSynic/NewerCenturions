//! The sounds, warnings and flashes the game uses to answer the player.

use serde::{Deserialize, Serialize};

/// Cargo Pod `snd ` 6000 "Error".
pub const ERROR_SOUND: i16 = 6000;
/// Cargo Pod `snd ` 20 "Booty", played when a battle hands over its loot.
pub const BOOTY_SOUND: i16 = 20;
/// Reloads add a `bound(2)` draw to this base, selecting Weapon Sounds `snd `
/// 404 "Clip Sound 0" or 405 "Clip Sound 1".
pub const CLIP_SOUND_BASE: i16 = 403;
/// Cargo Pod `STR#` 3 "Warnings", entry 25: the weapon has no ammunition left.
pub const OUT_OF_AMMUNITION_WARNING: i16 = 25;

/// What the game shows or sounds back to the player: warnings, tutorial pages, flashes, dialog
/// text, combat refusals and results.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameFeedback {
    Silent,
    DialogText {
        dialog: i16,
        item: u16,
        address: u32,
        wait: i16,
    },
    Tutorial {
        first: usize,
        last: usize,
    },
    Flash {
        address: u32,
        x: i16,
        y: i16,
        wait: i16,
        sound: i16,
    },
    ResourceFlash {
        resource_id: i16,
        index: usize,
        x: i16,
        y: i16,
        wait: i16,
        sound: i16,
    },
    Warning {
        index: i16,
    },
    EquipmentLevel {
        level: i16,
    },
    Combat {
        kind: CombatRefusal,
    },
    /// Outdoors, killbody shows that a warrior is dead or knocked out at
    /// (400, 30), then waits for the animation-delay preference.
    OutdoorBodyStatus {
        permanent: bool,
        wait: i16,
    },
    CombatResult {
        body: i16,
        code: i16,
        amount: i16,
        audible: bool,
    },
    /// Result code -40 shows the loaded item's condition label from STR# 134.
    CombatResourceResult {
        resource_id: i16,
        index: usize,
    },
    Sound {
        resource_id: i16,
    },
}

/// The camping flash shared by the camp button and the camping-ability script
/// operation: x 50, y 50, sound 6001.
pub fn camp_flash(address: u32) -> GameFeedback {
    GameFeedback::Flash {
        address,
        x: 50,
        y: 50,
        wait: 0,
        sound: 6001,
    }
}

/// The "Click Mouse" flash a script shows before it resumes.
///
/// Original: newland
pub fn script_pause() -> GameFeedback {
    GameFeedback::Flash {
        address: 0x401c9e,
        x: 100,
        y: 200,
        wait: 0,
        sound: 30005,
    }
}

/// The fixed text of showresults' result codes, as original string addresses.
///
/// Original: showresults
pub fn combat_result(code: i16) -> Option<(u32, i16)> {
    Some(match code {
        -8 => (0x400c98, 0),
        -7 => (0x400c8c, 0),
        -6 => (0x400c7c, 0),
        -5 => (0x400c72, 0),
        -34 => (0x400e9a, 104),
        -33 => (0x400e8e, 624),
        -32 => (0x400e84, 630),
        -30 => (0x400e76, 651),
        -29 => (0x400e6c, 659),
        -28 => (0x400e62, 667),
        -27 => (0x400e58, 644),
        -26 => (0x400e50, 664),
        -23 => (0x400e0e, 649),
        -22 => (0x400df8, 107),
        -21 => (0x400de2, 344),
        -20 => (0x400dd0, 431),
        -19 => (0x400dbe, 601),
        -18 => (0x400dac, 693),
        -17 => (0x400d9a, 693),
        -16 => (0x400d8a, 618),
        -15 => (0x400d76, 360),
        -14 => (0x400cc2, 0),
        -12 => (0x400ce2, 0),
        -9 => (0x400cac, 0),
        -47 => (0x400d02, 631),
        -51 => (0x400d34, 629),
        -50 => (0x400d22, 629),
        -45 => (0x400d0e, 629),
        _ => return None,
    })
}

/// Why a shot, throw or cast cannot be made on the chosen target.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatRefusal {
    NeedTarget,
    OutOfRange,
    Blocked,
    InvalidTarget,
    TooManyTargets,
    NotEnoughAmmo,
}

impl CombatRefusal {
    /// The text the game flashes for this refusal.
    pub fn address(self) -> u32 {
        match self {
            Self::NeedTarget => 0x403daa,
            Self::OutOfRange => 0x403d98,
            Self::Blocked => 0x403d86,
            Self::InvalidTarget => 0x403d70,
            Self::TooManyTargets => 0x403d5c,
            Self::NotEnoughAmmo => 0x403d4a,
        }
    }
}
