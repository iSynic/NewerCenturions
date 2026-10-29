use crate::combat::{Combat, attack, damage, firearm};
use crate::{RandomState, RuleError};
use attack::AttackResult;
use damage::{DamageFlags, DamagePacket, resolve_damage, resolve_damage_with_amount};
mod packets;
#[cfg(all(test, feature = "original-fixtures"))]
#[path = "../../tests/unit/spells_restriction_tests.rs"]
mod restriction_tests;
pub use packets::{load_packet, select_packet};
mod selection;
use selection::position;
pub(crate) use selection::{
    SelectionView, select_firearm_from_view, select_firearm_targets_beside,
    select_retained_from_view, select_spell_from_view, select_targets_beside,
};
pub use selection::{
    SpellSelection, SpellTarget, select_firearm_targets, select_retained_targets, select_targets,
};
mod restrictions;
pub use restrictions::RestrictionMemoryView;
pub(crate) use restrictions::{is_support, retained_item_alias};
use restrictions::{monster_race_is_excluded, race_is_excluded, validate_direct};
mod results;
pub use results::{BodySpellResult, SupportSpellResult, WarriorSpellResult};
mod support;
use support::resolve_support_inner;
pub use support::{
    resolve_player_support, resolve_support_for_caster, resolve_support_with_context,
};
mod support_rules;
pub(crate) use support_rules::resolve_psi_drain;
use support_rules::status_resisted;
mod contexts;
pub(crate) mod detached;
pub use contexts::{DirectSpell, FieldUsage, SpellContext, SpellMask};
mod fields;
pub use fields::{FieldCollision, collision_field, reset_used, tick_fields};
mod masks;
pub use masks::{
    GlobalSpellTargets, global_targets, target_mask, target_mask_rotated,
    target_mask_with_map_aliases, target_mask_with_origin,
};
pub(crate) use masks::{global_target_mask, summon_field, target_mask_rotated_with_origin};
mod ray;
pub(crate) use ray::{
    RayStep, RayWalk, body_pixel, crossings, flight, flight_between, flight_jitter, mark_crossings,
    mark_step, ray_walks, stop_flight,
};
mod party_damage;
pub use party_damage::resolve_warriors;
mod body_damage;
pub(crate) use body_damage::resolve_collision_with_memory;
pub use body_damage::{
    resolve_bodies, resolve_bodies_with_item_cache, resolve_bodies_with_memory, resolve_collision,
    resolve_collision_with_item_cache,
};
mod firearm_damage;
pub use firearm_damage::{resolve_enemy_firearm_body, resolve_enemy_firearm_body_after_duration};
