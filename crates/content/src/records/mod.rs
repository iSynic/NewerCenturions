//! Borrowed views over original records. These never change their serialized storage.
//!
//! Field names follow their callers in Centurions, not the superficially similar
//! Realmz layouts. Reads retain signed big-endian words; writes touch only the
//! specified bytes. Unknown fields stay opaque.
mod attack;
mod character;
mod condition_target;
mod conditions;
mod damage;
mod development;
mod equipment;
mod equipment_eligibility;
mod exploration_extras;
mod exploration_tables;
mod inventory_slots;
mod item;
mod item_panel;
mod learned_scans;
pub mod loot_cell;
mod monster;
mod party_restrictions;
mod profiles;
mod psi_memory;
mod shop;
mod spell_field;
mod spell_map;
mod statistics;
mod story;
mod story_destination;
mod story_effects;
mod story_inventory;
mod story_party;
mod story_promotion;
mod story_schedule;
mod story_variables;
mod time_schedule;
mod travel_events;
mod tutorial;
pub use spell_map::SpellMapAliases;
mod typed_encounter;
mod upgrade;
mod warrior;
mod weapon_attack;

pub use attack::{AttackPacket, AttackPacketMut};
pub use character::{CharacterIdentityMut, CharacterInformation};
pub use condition_target::{ConditionTarget, SavingThrow};
pub use conditions::Condition;
pub use damage::{DamageDefenses, DamageDefinition, DamageTargetKind, DamageTargetRecord};
pub use development::{ATTRIBUTES, Attribute, DevelopmentField};
pub use equipment::{
    EquipmentModifiers, EquipmentStat, InventoryModifier, ItemUpgrade, RetainedEquipmentModifiers,
};
pub use equipment_eligibility::{EquipmentEligibility, ProfessionEquipment, RaceEquipment};
pub use exploration_extras::{
    AllyClassBranchExtra, ChanceBranchExtra, ClinicExtra, ExtraTable, LimitedTreasureExtra,
    MapEffectExtra, MapMutationExtra, PaymentBranchExtra, RandomBranchExtra, RegionChangeExtra,
};
pub use exploration_tables::{
    EncounterMenu, EventHeaderTable, EventHeaderTableMut, RegionTable, RegionTableMut,
    TreasureTable,
};
pub use inventory_slots::{InventorySlots, InventorySlotsMut};
pub use item::{ItemDefinition, WeaponCondition, item_class};
pub use item_panel::{ItemPanelRecord, ItemPanelWorkMut};
pub use learned_scans::{LearnedScans, LearnedScansMut};
pub use monster::{EnemyRecord, EnemyRecordMut, FootprintCell, NaturalAttack};
pub use party_restrictions::PartyRestrictions;
pub use profiles::{ProfessionProfile, RaceProfile};
pub use psi_memory::PsiSelectionMemoryMut;
pub use shop::{SHOP_BYTES, SHOP_SLOTS, ShopRecord, ShopRecordMut};
pub use spell_field::{SPELL_FIELD_BYTES, SpellField, SpellFieldMut};
pub use statistics::{CharacterStatistics, CharacterStatisticsMut, Statistic};
pub use story::{
    FatigueChange, ObjectRectangleExtra, RecoveryRestrictions, StoryRecord, StoryRecordMut,
    TravelExtra,
};
pub use story_effects::{
    ActorReplacementExtra, NpcBattleExtra, PartyHealthExtra, RetreatClassesExtra,
    ScriptedSpellExtra, SkillCheckExtra, TreasureRecord, TreasureRecordMut,
};
pub use story_inventory::{ItemConsumptionExtra, ItemPrerequisiteExtra};
pub use story_party::{PartyFilterExtra, PartySelectionExtra, StatisticAdjustmentExtra};
pub use story_schedule::ScheduleEditExtra;
pub use story_variables::{PartyItemConditionExtra, VariablePredicateExtra, VariableUpdateExtra};
pub use time_schedule::{ScheduleEntry, ScheduleEntryMut, ScheduleRegion, ScheduledScripts};
pub use travel_events::{EventChanceExtra, EventReplacementExtra, TileMutationExtra};
pub use tutorial::TutorialRectangles;
pub use typed_encounter::{QuestionExtra, TYPED_ENCOUNTER_BYTES, TypedEncounterRecord};
pub use upgrade::UpgradeRecord;
pub use warrior::{WarriorRecord, WarriorRecordMut};
pub use weapon_attack::{WeaponAttackModifiers, WeaponAttackParameters};

fn put_word(bytes: &mut [u8], offset: usize, value: i16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
}
