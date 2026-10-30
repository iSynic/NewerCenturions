use crate::{
    Exploration, Globals, LevelState, MapState, MedicalContinuation, MovementContinuation,
    PsiMenuChoice, RandomState, TimeContinuation, combat, confiscation, empty_allies,
    empty_ammunition, empty_notes, empty_party_buffs, field_controls, initial_flags,
    initial_library_seed, no_notes, normal_difficulty, original_save, outdoor_psi, shop,
    story_actor, story_cleanup, story_npc_battle, story_promotion, zero_level, zero_word,
};
use centurions_content::{InventoryRecord, Warrior};
use serde::{Deserialize, Serialize};

/// Where the game stands: choosing the squad, exploring, or in a battle.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Phase {
    /// The player is choosing the squad.
    SquadSelection,
    /// The party is exploring a land.
    Exploration,
    /// The party is in a battle.
    Combat,
}

/// A game in progress: the squad and phase, the mission and land the party is in, and everything
/// the party carries and the game remembers.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Session {
    pub content_identity: String,
    #[serde(default, skip_serializing_if = "normal_difficulty")]
    pub difficulty: i16,
    #[serde(skip)]
    pub(crate) command_preferences: CommandPreferences,
    /// Per-command control flow, cleared before committing a public action.
    #[serde(skip)]
    pub(crate) command_return: bool,
    pub warriors: Vec<Warrior>,
    pub phase: Phase,
    pub mission: Option<String>,
    #[serde(default, skip_serializing_if = "zero_level")]
    pub scenario_level: u8,
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub visited_levels: std::collections::BTreeMap<u8, LevelState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "native_import")]
    pub imported: Option<original_save::ImportRecords>,
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub shops: std::collections::BTreeMap<u8, Vec<u8>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shop_access: Option<shop::Access>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confiscated: Option<confiscation::ConfiscatedEquipment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recovery_return: Option<crate::loot_recovery::RecoveryReturn>,
    /// Keeps the held-over allies out of the next battle while set.
    #[serde(default, skip_serializing_if = "zero_word")]
    #[serde(rename = "suppress_allies")]
    pub suspend_allies: i16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loot_limit: Option<i16>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub typed_encounters: Vec<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field_controls: Option<field_controls::State>,
    pub exploration: Option<Exploration>,
    pub random: RandomState,
    pub crate_items: Vec<InventoryRecord>,
    #[serde(rename = "booty_items")]
    pub booty_list: Vec<InventoryRecord>,
    #[serde(default = "empty_ammunition")]
    pub ammunition: Vec<i32>,
    #[serde(default)]
    pub combat: Option<combat::Combat>,
    #[serde(default)]
    pub battle_treasure: i16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ordinary_battle_rewards: Option<combat::runtime::OrdinaryBattleRewards>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_battle_treasure: Option<combat::runtime::SelectedBattleTreasure>,
    #[serde(default)]
    pub battle_after_message: i16,
    #[serde(default)]
    pub battle_group_script: Option<i16>,
    #[serde(default)]
    pub battle_npc: Option<story_npc_battle::NpcBattleStart>,
    #[serde(default)]
    pub combat_story: Option<combat::runtime::DeathStory>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "retained_walk_choice")]
    pub out_of_combat_walk_choice: Option<combat::runtime::OutOfCombatWalkChoice>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "npc_action")]
    pub monster_action: Option<Box<combat::runtime::MonsterActionFrame>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub survivors: Option<combat::allies::SurvivorSelection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub round_dispatch: Option<combat::round_stories::RoundDispatch>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_dispatch: Option<combat::runtime::TurnDispatch>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scripted_cleanup: Option<story_cleanup::Stage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor_replacement: Option<story_actor::Replacement>,
    #[serde(default)]
    pub movement_continuation: Option<MovementContinuation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_continuation: Option<TimeContinuation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zero_time: Option<crate::ZeroTimeReturn>,
    /// getup reached booty's early ending while a dialogue or survivor window
    /// was open (save schema 57).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub deferred_getup: bool,
    /// Omitted while zero; cleared when the loot window closes.
    #[serde(default, skip_serializing_if = "crate::BootyTally::is_empty")]
    pub booty_tally: crate::BootyTally,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub psi_selection: Option<combat::player_psi::selection::Selection>,
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
    pub bandage_ready: i16,
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
    /// The map, x and y the party-position operation saved, kept in the original
    /// save and never reset by a new mission. Omitted while zero.
    #[serde(default, skip_serializing_if = "no_saved_position")]
    pub saved_position: [u8; 3],
    #[serde(default)]
    pub visited_maps: std::collections::BTreeMap<i32, MapState>,
    /// The allies held over between battles, one monster record each.
    #[serde(default = "empty_allies")]
    #[serde(rename = "allies")]
    pub holdover: Vec<u8>,
    /// How many allies are held over.
    #[serde(default)]
    #[serde(rename = "ally_count")]
    pub held_over: usize,
    #[serde(default)]
    #[serde(rename = "selected_warrior")]
    pub char_select_new: usize,
    /// Game values that outlast a single battle.
    #[serde(flatten)]
    pub globals: Globals,
    #[serde(default = "empty_party_buffs")]
    #[serde(rename = "party_buffs")]
    pub party_condition: Vec<u8>,
    /// Effect 110 writes into the unused end of the scenario directory name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scenario_duration_tail: Option<i16>,
    #[serde(default)]
    pub psi_disabled: u8,
    #[serde(default)]
    pub enemy_spell_disabled: u8,
    #[serde(default)]
    pub energy_recovery_disabled: u8,
    #[serde(default)]
    pub party_currency: i32,
    #[serde(default = "initial_library_seed")]
    pub library_random_seed: u32,
    #[serde(default)]
    pub library_random_draws: u64,
}

/// The player's choices that change how a command plays out: automatic refitting, message sounds,
/// animation speed and the combat layout.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandPreferences {
    /// Whether ammunition is refitted automatically.
    pub auto_refit: bool,
    /// Whether the round message sound is left out.
    #[serde(default)]
    pub suppress_round_message_sound: bool,
    /// The animation delay the player chose, in steps of 5 up to 50.
    #[serde(default)]
    pub animation_delay: i16,
    /// Preference 22 selects the fixed combat layout. It is a command input
    /// because a script's position read can reach its rectangles.
    #[serde(default, skip_serializing_if = "large_combat_field")]
    pub small_combat_field: bool,
    /// Records `trace` entries for this command only; never serialized.
    #[serde(skip)]
    pub trace: bool,
}

/// Whether the combat field is the large one, so the setting can be left out of a save.
fn large_combat_field(small: &bool) -> bool {
    !*small
}

/// Whether no position has been saved, so it can be left out of a save.
fn no_saved_position(position: &[u8; 3]) -> bool {
    *position == [0; 3]
}
