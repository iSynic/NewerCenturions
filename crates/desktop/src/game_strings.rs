//! The game's own strings, read through `GameText::text`.
//!
//! Each value is where the string lives in the game's data; the comment gives
//! its text.

/// Pascal "Activate Warrior Edit" (Character menu item 8 while editing is off).
pub const ACTIVATE_WARRIOR_EDIT: u32 = 0x403670;
/// Pascal "Deactivate Warrior Edit" (Character menu item 8 while editing is on).
pub const DEACTIVATE_WARRIOR_EDIT: u32 = 0x403658;
/// Pascal "Start Music" (Music menu item 1 while music is disabled).
pub const START_MUSIC: u32 = 0x40355e;
/// Pascal " ---" (a maximum Psi of zero).
pub const NO_PSI: u32 = 0x400a76;
/// Pascal " ---" (a maximum Psi of zero in a squad row beside the map).
pub const NO_PSI_SHORT: u32 = 0x400a70;
/// Pascal "" (an empty readout in the Items window's warrior panel).
pub const BLANK: u32 = 0x400a75;
/// Pascal "---" (the capture button's second special skill when the warrior has none).
pub const NO_SPECIAL_SKILL: u32 = 0x400a6c;
/// Pascal "No Special Attacks".
pub const NO_SPECIAL_ATTACKS: u32 = 0x404046;
/// C "This warrior does not meet the minimum skill level to use this item.  Minimum
/// Skill Level = ".
pub const MINIMUM_SKILL_LEVEL: u32 = 0x400548;
/// Pascal " X" (item quantity prefix).
pub const QUANTITY_PREFIX: u32 = 0x401152;
/// Pascal " ? ?" (hidden map coordinates).
pub const HIDDEN_COORDINATES: u32 = 0x40226a;
/// C "      |  Duration = ".
pub const DURATION_LABEL: u32 = 0x40113c;
/// C "The charge can't be regained once used. If you abort you will not regain the
/// charge.  Abort anyway?"
pub const ABORT_CHARGE_QUESTION: u32 = 0x403c72;
/// C "Select Target(s). ... Hit 'A' To Abort. ... Hit Space To Activate." (space-padded).
pub const SELECT_TARGETS: u32 = 0x4033f6;
/// Pascal "Attack Friend".
pub const ATTACK_FRIEND: u32 = 0x403bfa;
/// Pascal "Swap Positions".
pub const SWAP_POSITIONS: u32 = 0x403c08;
/// Pascal "Do Not Attack".
pub const DO_NOT_ATTACK: u32 = 0x403bd8;
/// Pascal "Attack Your Friend".
pub const ATTACK_YOUR_FRIEND: u32 = 0x403be6;
/// Pascal "Stay and Fight".
pub const STAY_AND_FIGHT: u32 = 0x403df4;
/// Pascal "Embrace Cowardice".
pub const EMBRACE_COWARDICE: u32 = 0x403e04;
/// Pascal "No" (a race that does not regenerate).
pub const REGENERATES_NO: u32 = 0x40197a;
/// Pascal "Yes" (a race that regenerates).
pub const REGENERATES_YES: u32 = 0x40197e;
/// C "If you play a saved game which has a character by this name, this one may change to
/// match that one." (flashed when a new warrior's name is already kept).
pub const NAME_KEPT_ALREADY: u32 = 0x401d2a;
/// Pascal "Range = " (the readout of the distance to the body in view).
pub const RANGE_LABEL: u32 = 0x403786;
/// Pascal " Blocked" (follows the range when the body in view cannot be seen).
pub const BLOCKED: u32 = 0x403aa8;
/// Pascal "Enemies Left" (the label before the count of monsters still to defeat).
pub const ENEMIES_LEFT: u32 = 0x401b6a;
/// Pascal "---" (the weapon in view holds no charge).
pub const NO_CHARGE: u32 = 0x403aa4;
/// Pascal "??" (the charge of a monster's weapon is unknown).
pub const UNKNOWN_CHARGE: u32 = 0x403aa0;
