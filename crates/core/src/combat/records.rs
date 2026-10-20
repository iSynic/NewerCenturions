//! Borrowed native records used by maintained combat interaction rules.
use crate::RuleError;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct CombatBodyId(i16);

pub(crate) enum BodySlot {
    Warrior(usize),
    Enemy(usize),
}

impl CombatBodyId {
    pub(crate) fn from_native(body: i16) -> Self {
        Self(body)
    }

    pub(crate) fn native(self) -> i16 {
        self.0
    }

    pub(crate) fn is_present(self, warriors: usize, enemies: usize) -> bool {
        (0..warriors as i16).contains(&self.0) || (10..10 + enemies as i16).contains(&self.0)
    }

    /// Native IDs reserve 0..5 for the squad and begin enemies at 10.
    /// Classify storage only; living/active requirements belong to each caller.
    pub(crate) fn slot(self, warriors: usize, enemies: usize) -> Option<BodySlot> {
        if (0..warriors as i16).contains(&self.0) {
            Some(BodySlot::Warrior(self.0 as usize))
        } else if self.0 >= 10 && ((self.0 - 10) as usize) < enemies {
            Some(BodySlot::Enemy((self.0 - 10) as usize))
        } else {
            None
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct CombatPosition {
    pub(crate) x: i16,
    pub(crate) y: i16,
}

impl CombatPosition {
    pub(crate) fn from_native((x, y): (i16, i16)) -> Self {
        Self { x, y }
    }

    pub(crate) fn step_toward(self, target: Self) -> (i8, i8) {
        (
            target.x.wrapping_sub(self.x).signum() as i8,
            target.y.wrapping_sub(self.y).signum() as i8,
        )
    }

    pub(crate) fn map_index(self, shift: (i16, i16)) -> Result<usize, RuleError> {
        Ok(centurions_content::mission::OpeningMission::index(
            i32::from(self.x) + i32::from(shift.0) + 5,
            i32::from(self.y) + i32::from(shift.1) + 5,
        )?)
    }
}
