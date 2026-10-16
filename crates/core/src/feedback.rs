//! Player feedback references from the original executable and Cargo Pod resources.

use serde::{Deserialize, Serialize};

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
    Sound {
        resource_id: i16,
    },
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
