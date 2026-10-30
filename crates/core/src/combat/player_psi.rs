use crate::combat::{Combat, lifecycle, runtime, spells};
use crate::{Event, RandomState, RuleError, Session};
pub mod selection;

/// In battle the selected warrior casts.
///
/// Original: castspell
pub(crate) fn select_combat_caster(state: &mut Session) {
    if state.combat.is_some() {
        state.set_warrior_selector(state.char_select_new);
        // Showing the spell clears the item in use.
        state.globals.item_used = 0;
    }
}

/// The psi powers a warrior can cast in battle, by spell number.
pub const COMBAT_SPELLS: &[i16] = &[
    1101, 1106, 1201, 1202, 1203, 1204, 1205, 1206, 1207, 1301, 1302, 1303, 1304, 1305, 1306, 1307,
    1401, 1402, 1403, 1404, 1405, 1406, 1407, 1502, 1503, 1504, 1505, 1506, 1507, 1601, 1602, 1603,
    1604, 1606, 1607, 1701, 1702, 1703, 1704, 1705, 1706, 1707, 2101, 2102, 2103, 2104, 2105, 2106,
    2107, 2201, 2202, 2203, 2204, 2205, 2206, 2207, 2301, 2302, 2303, 2304, 2305, 2306, 2307, 2401,
    2402, 2403, 2404, 2405, 2406, 2407, 2501, 2502, 2503, 2504, 2505, 2506, 2507, 2601, 2602, 2603,
    2604, 2605, 2606, 2607, 2701, 2702, 2703, 2704, 2705, 2706, 2707, 3102, 3104, 3105, 3106, 3107,
    3201, 3202, 3203, 3204, 3205, 3206, 3207, 3301, 3302, 3303, 3304, 3305, 3306, 3307, 3402, 3403,
    3404, 3405, 3406, 3407, 3501, 3502, 3503, 3504, 3505, 3506, 3507, 3601, 3602, 3603, 3604, 3605,
    3607, 3701, 3702, 3703, 3704, 3705, 3706, 3707,
];

/// Further psi powers the party menu accepts out of combat, besides `OUTDOOR_TARGET_SPELLS` and
/// `PARTY_SPELLS`.
pub const OUTDOOR_PARTY_SPELLS: &[i16] = &[
    1104, 1105, 1107, 1301, 1302, 1303, 1304, 1305, 1306, 1307, 1505, 1706, 3401,
];
/// Psi powers the party menu casts out of combat that are aimed at a target.
pub const OUTDOOR_TARGET_SPELLS: &[i16] = &[
    1101, 1201, 1202, 1203, 1204, 1205, 1206, 1207, 1401, 1405, 1503, 1602, 1604, 1703, 1704, 1705,
    3201, 3202, 3203, 3204, 3301, 3303, 3304, 3305, 3505, 3603, 3605, 3703, 3704, 3705, 3707,
];

/// Psi powers the party menu casts that last on the whole party.
pub const PARTY_SPELLS: &[i16] = &[1102, 1103, 1407, 1501, 1507, 1605, 3101, 3103, 3406, 3606];
mod admission;
use admission::packet;
pub use admission::{
    CastAdmission, CastRequest, admission, can_cast, cost, in_range, mode_allowed, range,
};
mod party;
pub(crate) use party::prepare_party;
pub use party::{PartyCastRequest, PartyCastResult, cast_party, validate_party};
mod validation;
pub(crate) use validation::selected_body;
use validation::validate_target;
pub use validation::{validate, validate_warrior};
mod casting;
pub use casting::cast;
pub(crate) use casting::{
    CastContext, cast_item, cast_prepared_at, cast_selection, cast_with_buffs,
};
mod effects;
pub(crate) use effects::{MoveArena, land_mover, resolve_mask};
use effects::{cast_ray, move_caster};
