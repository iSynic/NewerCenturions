//! Player feedback references from the original executable and Cargo Pod resources.

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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeFeedback {
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
    Combat {
        kind: CombatRefusal,
    },
    CombatResult {
        body: i16,
        code: i16,
        amount: i16,
        audible: bool,
    },
    Sound {
        resource_id: i16,
    },
}

/// The camping flashes shared by buttonchoice and newland66: x 50, y 50, sound 6001.
pub fn camp_flash(address: u32) -> NativeFeedback {
    NativeFeedback::Flash {
        address,
        x: 50,
        y: 50,
        wait: 0,
        sound: 6001,
    }
}

/// newland144632: original Click Mouse flash before the script resumes.
pub fn script_pause() -> NativeFeedback {
    NativeFeedback::Flash {
        address: 0x401c9e,
        x: 100,
        y: 200,
        wait: 0,
        sound: 30005,
    }
}

/// showresults 0x158454 fixed result branches; addresses reference Pascal DATA strings.
pub fn combat_result(code: i16) -> Option<(u32, i16)> {
    Some(match code {
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
        -47 => (0x400d02, 631),
        -51 => (0x400d34, 629),
        -50 => (0x400d22, 629),
        -45 => (0x400d0e, 629),
        _ => return None,
    })
}

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
