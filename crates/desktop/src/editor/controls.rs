//! Original Upgrade Warrior DITL identities, decoded once at input admission.
//! These are desktop actions, not serialized command or save variants.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum UpgradeControl {
    ScanSelection,
    ApplyUpgrade,
    SelectAttribute(u16),
    EditLevel,
    UndoLast,
    UndoAll,
    PreviousWarrior,
    NextWarrior,
    Unknown,
}

impl UpgradeControl {
    pub(super) fn from_native(item: usize) -> Self {
        match item {
            39 => Self::ScanSelection,
            45 => Self::ApplyUpgrade,
            48..=80 => Self::SelectAttribute((item - 48) as u16),
            81 => Self::EditLevel,
            82 => Self::UndoLast,
            83 => Self::UndoAll,
            84 => Self::PreviousWarrior,
            85 => Self::NextWarrior,
            _ => Self::Unknown,
        }
    }
}
