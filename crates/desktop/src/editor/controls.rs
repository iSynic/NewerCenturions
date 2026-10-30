//! The Upgrade Warrior dialog's items, decoded once when input arrives.
//! These are desktop actions, not serialized command or save variants.
/// The Upgrade Warrior dialog's controls.
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
    /// The control for dialog item `item`.
    pub(super) fn from_item(item: usize) -> Self {
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
