mod preparation;
pub(super) use preparation::compose_weapon_attack;
pub use preparation::{prepare, prepare_enemy};
pub(crate) use preparation::{prepare_packet, prepare_packet_with_enhancement};
mod reload;
mod resolution;
pub mod weapon_traits;
pub use reload::{FitAmmunitionResult, fit_ammunition, reload, reload_command};
pub use resolution::{resolve, resolve_enemy, resolve_enemy_after_duration};
pub(crate) use resolution::{resolve_after_duration, resolve_friendly_after_duration};

/// The point-target attack an enemy laser fires.
pub(crate) const POINT_TARGET_ATTACK: i16 = 4101;
/// Data ID items whose `STR#` 601 "Devices Short" names are "Small Laser" and
/// "Medium Laser"; the only enemy primary weapons `prepare_enemy` accepts.
pub(crate) const SMALL_LASER: i16 = 680;
/// Data ID of the medium laser, the other enemy primary weapon `prepare_enemy` accepts.
pub(crate) const MEDIUM_LASER: i16 = 681;

/// A firearm ready to fire: the attack packet, the inventory slot of the weapon, shots per firing,
/// rounds spent per shot, the weight each spent round takes off the load, the impact sound, and
/// whether the weapon or its ammunition skips the hit roll and has its damage filtered by armor.
pub struct PreparedFirearm {
    pub packet: Vec<u8>,
    pub slot: usize,
    pub shots: i16,
    pub rounds: i16,
    pub weight: i16,
    pub sound: i16,
    pub armor_filter: bool,
}
