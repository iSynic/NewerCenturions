//! Views over the game's records, borrowed in their saved form.
//!
//! Field names use the author's vocabulary where New Centurions confirms it
//! (see docs/glossary.md); Realmz layouts differ, so no field is named from its
//! position alone. Reads keep signed big-endian words; writes touch only the
//! named bytes. Unknown fields stay opaque.
use crate::bytes::put_word;

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
pub mod warrior_layout;
mod weapon_attack;

pub use attack::{SpellInfoRecord, SpellInfoRecordMut};
pub use character::{CharacterIdentityMut, CharacterInformation};
pub use condition_target::{ConditionTarget, SavingThrow};
pub use conditions::Condition;
pub use damage::{DamageDefenses, DamageDefinition, DamageTargetKind, DamageTargetRecord};
pub use development::{ATTRIBUTES, Attribute, DevelopmentField};
pub use equipment::{
    EquipmentModifiers, EquipmentStat, InstanceModifiers, InventoryModifier, ItemUpgrade,
};
pub use equipment_eligibility::{EquipmentEligibility, ProfessionEquipment, RaceEquipment};
pub use exploration_extras::{
    AllyClassBranchExtra, BoatCampingExtra, ChanceBranchExtra, ClinicExtra, DisableChoiceExtra,
    DropCreditsExtra, ExtraTable, GiveConditionExtra, LandLightExtra, LimitedTreasureExtra,
    MapEffectExtra, MapMutationExtra, PaymentBranchExtra, PickByCasteExtra, PickByTraitExtra,
    RandomBranchExtra, RandomTreasureExtra, RegionChangeExtra, ShiftPartyExtra, VictoryPointsExtra,
};
pub use exploration_tables::{
    EncounterMenu, EncounterMenuMut, EventHeaderTable, EventHeaderTableMut, RegionTable,
    RegionTableMut, TreasureTable,
};
pub use inventory_slots::{InventorySlots, InventorySlotsMut};
pub use item::{ItemDefinition, WeaponCondition, item_class};
pub use item_panel::{ItemPanelMut, ItemPanelRecord};
pub use learned_scans::{LearnedScans, LearnedScansMut};
pub use monster::{EnemyRecord, EnemyRecordMut, FootprintCell, NaturalAttack};
pub use party_restrictions::PartyRestrictions;
pub use profiles::{
    PROFESSION_PROFILE_BYTES, ProfessionProfile, ProfileTable, RACE_PROFILE_BYTES, RaceProfile,
};
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
