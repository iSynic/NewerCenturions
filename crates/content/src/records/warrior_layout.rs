//! Warrior record (`c[n]`, 0xc42 bytes) fields shared by more than one view.
//!
//! Each constant is the byte offset of a big-endian field in the saved warrior
//! record. Views keep the width and signedness given here; a field read by a
//! single view stays in that view. Rules that reach a warrior field through a
//! monster indexed as a warrior name the field with these offsets.

/// Signed word item count (`numitems`, 0..=100), kept as items are installed
/// and dropped.
pub const ITEM_COUNT: usize = 0x1a;
/// Signed word Psi school; zero means none, which hides upgrade selector 21.
pub const PSI_SCHOOL: usize = 0x36;
/// Signed word current stamina (`stamina`).
pub const STAMINA: usize = 0x50;
/// Signed word maximum stamina (`staminamax`).
pub const STAMINA_MAX: usize = 0x52;
/// Forty signed word condition durations (`condition`); a negative duration is
/// permanent.
pub const CONDITIONS: usize = 0x5e;
/// End of the condition durations (exclusive).
pub const CONDITIONS_END: usize = 0xae;
/// Signed word condition 25, Computer Control: the warrior fights on its own.
pub const COMPUTER_CONTROL: usize = 0x90;
/// One hundred inventory records (`items`) of `INVENTORY_BYTES` each.
pub const ITEMS: usize = 0x132;
/// Signed long remaining victory points.
pub const REMAINING_VICTORY_POINTS: usize = 0x9ce;
/// Signed long upgrade points.
pub const UPGRADE_POINTS: usize = 0x9d2;
/// The firing toggle byte: nonzero selects the missile weapon.
pub const TOGGLE: usize = 0x9dd;
/// Bleeding flag byte (`bleeding`).
pub const BLEEDING: usize = 0x9de;
/// Seven by seven learned-scan bytes (`cspells`, row * 7 + column), ending at
/// the name.
pub const KNOWN_SCANS: usize = 0x9f0;
/// Number of learned-scan bytes.
pub const LEARNED_SCAN_BYTES: usize = 49;
/// NUL-terminated Mac Roman name.
pub const NAME: usize = 0xa21;
/// Ten eight-byte scan shortcuts (`definespells`), kept when choosing scans.
pub const DEFINE_SPELLS: usize = 0xa76;
/// End of the scan shortcuts (exclusive).
pub const PSI_MEMORY_END: usize = 0xac6;
