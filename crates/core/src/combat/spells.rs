use crate::combat::{Combat, attack, damage, firearm, lifecycle, spell_conditions};
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
pub use selection::{
    SpellSelection, SpellTarget, select_firearm_targets, select_retained_targets, select_targets,
};
mod restrictions;
pub use restrictions::RestrictionMemoryView;
use restrictions::{monster_race_is_excluded, race_is_excluded, validate_direct};
mod results;
pub use results::{BodySpellResult, SupportSpellResult, WarriorSpellResult};
mod support;
pub(crate) use support::resolve_support_for_caster;
pub use support::{resolve_player_support, resolve_support_with_context};
mod support_rules;
pub(crate) use support_rules::resolve_psi_drain;
use support_rules::status_resisted;
mod contexts;
pub use contexts::{DirectSpell, FieldUsage, SpellContext, SpellMask};
mod fields;
pub use fields::{FieldCollision, collision_field, reset_used, tick_fields};
mod masks;
pub(crate) use masks::target_mask_rotated_with_origin;
pub use masks::{
    GlobalSpellTargets, global_targets, target_mask, target_mask_rotated, target_mask_with_origin,
};
mod party_damage;
use party_damage::resolve_party;
pub use party_damage::resolve_warriors;
mod body_damage;
pub use body_damage::{
    resolve_bodies, resolve_bodies_with_item_cache, resolve_bodies_with_memory, resolve_collision,
    resolve_collision_with_item_cache,
};
mod firearm_damage;
pub use firearm_damage::{resolve_enemy_firearm_body, resolve_enemy_firearm_body_after_duration};
