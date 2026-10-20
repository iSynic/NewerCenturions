use crate::{
    Exploration, LevelState, MapState, MedicalContinuation, MovementContinuation, PsiMenuChoice,
    RandomState, TimeContinuation, ammunition, combat, confiscation, empty_allies,
    empty_ammunition, empty_notes, empty_party_buffs, field_controls, initial_flags,
    initial_library_seed, native_save, no_notes, normal_difficulty, outdoor_psi, shop, story_actor,
    story_cleanup, story_health, story_npc_battle, story_promotion, zero_level, zero_word,
};
use centurions_content::{InventoryRecord, Warrior};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Phase {
    SquadSelection,
    Exploration,
    Combat,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Session {
    pub content_identity: String,
    #[serde(default, skip_serializing_if = "normal_difficulty")]
    pub difficulty: i16,
    #[serde(skip)]
    pub(crate) command_preferences: CommandPreferences,
    pub warriors: Vec<Warrior>,
    pub phase: Phase,
    pub mission: Option<String>,
    #[serde(default, skip_serializing_if = "zero_level")]
    pub scenario_level: u8,
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub visited_levels: std::collections::BTreeMap<u8, LevelState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_import: Option<native_save::ImportRecords>,
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub shops: std::collections::BTreeMap<u8, Vec<u8>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shop_access: Option<shop::Access>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confiscated: Option<confiscation::ConfiscatedEquipment>,
    #[serde(default, skip_serializing_if = "zero_word")]
    pub suppress_allies: i16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loot_limit: Option<i16>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub typed_encounters: Vec<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field_controls: Option<field_controls::State>,
    pub exploration: Option<Exploration>,
    pub random: RandomState,
    pub crate_items: Vec<InventoryRecord>,
    pub booty_items: Vec<InventoryRecord>,
    #[serde(default = "empty_ammunition")]
    pub ammunition: Vec<i32>,
    #[serde(default)]
    pub combat: Option<combat::Combat>,
    #[serde(default)]
    pub battle_treasure: i16,
    #[serde(default)]
    pub battle_after_message: i16,
    #[serde(default)]
    pub battle_group_script: Option<i16>,
    #[serde(default)]
    pub battle_npc: Option<story_npc_battle::NpcBattleStart>,
    #[serde(default)]
    pub combat_story: Option<combat::runtime::DeathStory>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub survivors: Option<combat::allies::SurvivorSelection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub round_dispatch: Option<combat::round_stories::RoundDispatch>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scripted_cleanup: Option<story_cleanup::Stage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor_replacement: Option<story_actor::Replacement>,
    #[serde(default)]
    pub movement_continuation: Option<MovementContinuation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_continuation: Option<TimeContinuation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub psi_selection: Option<combat::psi_selection::Selection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_quantity: Option<combat::targeting::FirearmQuantity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outdoor_psi: Option<outdoor_psi::Selection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encounter_actions: Option<[bool; 8]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub promotion: Option<story_promotion::Progress>,
    #[serde(default)]
    pub medical_continuation: Option<MedicalContinuation>,
    #[serde(default)]
    pub psi_menu_choices: [Option<PsiMenuChoice>; 6],
    #[serde(default)]
    pub bandage_ready: Option<i16>,
    #[serde(default)]
    pub handled_deaths: Vec<usize>,
    #[serde(default)]
    pub menus: Vec<u8>,
    #[serde(default = "initial_flags")]
    pub flags: Vec<u8>,
    #[serde(default)]
    pub mission_score: i16,
    #[serde(default)]
    pub mission_stage: i16,
    #[serde(default)]
    pub map_picture: i16,
    #[serde(default = "empty_notes", skip_serializing_if = "no_notes")]
    pub notes: Vec<u8>,
    #[serde(default)]
    pub visited_maps: std::collections::BTreeMap<i32, MapState>,
    #[serde(default = "empty_allies")]
    pub allies: Vec<u8>,
    #[serde(default)]
    pub ally_count: usize,
    #[serde(default)]
    pub party_mask: [i16; 6],
    #[serde(default)]
    pub selected_warrior: usize,
    /// Retained combat globals survive replacement of the active combat roster.
    #[serde(default)]
    pub selected_enemy: usize,
    #[serde(default)]
    pub script_enemy: usize,
    #[serde(default)]
    pub loaded_item_field32: Option<i16>,
    #[serde(default)]
    pub ammunition_work: ammunition::WorkBuffers,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inventory_record: Option<InventoryRecord>,
    #[serde(default)]
    pub npc_backing: combat::retained::NpcBacking,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retained_death_queue: Option<combat::death_scripts::DeathQueue>,
    #[serde(default)]
    pub loot_quality: Option<Vec<u8>>,
    #[serde(default)]
    pub attack_count: i16,
    #[serde(default)]
    pub attack_packet: Option<Vec<u8>>,
    #[serde(default)]
    pub attack_identity: Option<[i16; 3]>,
    #[serde(default)]
    pub weapon_work: combat::weapon_work::WeaponWork,
    #[serde(default)]
    pub psi_memory: Option<Vec<u8>>,
    #[serde(default = "empty_party_buffs")]
    pub party_buffs: Vec<u8>,
    /// WORD44263e: effect110 aliases the unused scenario-directory tail.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scenario_duration_tail: Option<i16>,
    #[serde(default)]
    pub psi_disabled: u8,
    #[serde(default)]
    pub enemy_spell_disabled: u8,
    #[serde(default)]
    pub energy_recovery_disabled: u8,
    #[serde(default)]
    pub health_context: Option<story_health::HealthContext>,
    #[serde(default)]
    pub party_currency: i32,
    #[serde(default = "initial_library_seed")]
    pub library_random_seed: u32,
    #[serde(default)]
    pub library_random_draws: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandPreferences {
    pub auto_refit: bool,
    #[serde(default)]
    pub suppress_round_message_sound: bool,
    #[serde(default)]
    pub animation_delay: i16,
}
