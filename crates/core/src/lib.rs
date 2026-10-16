use centurions_content::{Content, ContentError, InventoryRecord, Warrior, word};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::Path;
use thiserror::Error;

pub mod character_display;
pub mod confiscation;
pub mod creation;
pub mod feedback;
pub mod item_display;
pub mod native_save;
pub mod outdoor_psi;
pub mod reference_views;
pub mod roster;
pub mod shop;

pub mod rules {
    /// CODE 1 pin, 0x13f65e. Sequential comparisons also define reversed bounds.
    pub fn pin(mut value: i16, minimum: i16, maximum: i16) -> i16 {
        if value < minimum {
            value = minimum;
        }
        if value > maximum {
            value = maximum;
        }
        value
    }

    /// CODE 1 getnumspells, 0x125512: ADD.W leaves the product's upper word intact.
    pub fn spell_count(level: i16) -> i32 {
        let square = i32::from(level) * i32::from(level);
        (square & !0xffff) | i32::from((square as u16).wrapping_add(16))
    }

    /// CODE 1 getselection, 0x125a08: signed quotient, rounded toward zero.
    pub fn selection_base(selection: i16) -> i16 {
        (selection / 200).wrapping_mul(200)
    }

    pub fn inclusive_range(value: i16, minimum: i16, maximum: i16) -> bool {
        value >= minimum && value <= maximum
    }

    pub fn random_bound(bound: i16, quickdraw_value: i16) -> i16 {
        let magnitude = i32::from(quickdraw_value).abs();
        (magnitude.wrapping_mul(i32::from(bound)) / 32768).wrapping_add(1) as i16
    }

    pub fn random_range(minimum: i16, maximum: i16, quickdraw_value: i16) -> i16 {
        minimum
            .wrapping_add(random_bound(
                maximum.wrapping_sub(minimum).wrapping_add(1),
                quickdraw_value,
            ))
            .wrapping_sub(1)
    }

    pub fn movement_cost(tile_cost: i16, dx: i8, dy: i8) -> i16 {
        let sign = (tile_cost as u16 >> 15) as i16;
        let cost = (tile_cost.wrapping_add(sign) >> 1).wrapping_sub(1);
        let base = if cost < 1 { 0 } else { cost };
        base.wrapping_add(i16::from(dx != 0))
            .wrapping_add(i16::from(dy != 0))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RandomState {
    pub seed: u32,
    pub draws: u64,
}

impl RandomState {
    pub fn new(seed: u32) -> Self {
        Self { seed, draws: 0 }
    }
    pub fn quickdraw(&mut self) -> i16 {
        self.seed = (u64::from(self.seed) * 16807 % 2147483647) as u32;
        self.draws += 1;
        let result = self.seed as i16;
        if result == i16::MIN { 0 } else { result }
    }
    pub fn bound(&mut self, bound: i16) -> i16 {
        rules::random_bound(bound, self.quickdraw())
    }
    pub fn range(&mut self, min: i16, max: i16) -> i16 {
        recovered_ai::random_range(min, max, self)
    }
}

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
    command_preferences: CommandPreferences,
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MovementContinuation {
    RegionalReturn {
        header: bool,
    },
    Region {
        x: i16,
        y: i16,
        check: bool,
        header: bool,
    },
    Header,
    Secrets,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimeContinuation {
    pub remaining_ticks: u8,
    pub minutes_per_tick: i16,
    pub schedule_cursor: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MedicalContinuation {
    #[serde(default)]
    pub origin: MedicalOrigin,
    pub actor: usize,
    pub item: i16,
    pub spell: i16,
    pub power: i16,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum MedicalOrigin {
    #[default]
    OwnedNeedle,
    LearnedHeal,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PsiMenuChoice {
    pub row: u8,
    pub column: u8,
    pub power: i16,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MedicalTarget {
    LivingPoint { x: i16, y: i16 },
    Portrait { warrior: usize },
    ConfirmedCancel,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Exploration {
    pub map: i32,
    pub x: i32,
    pub y: i32,
    pub day: i16,
    pub hour: i16,
    pub minute: i16,
    pub fatigue: i16,
    pub facing: u8,
    #[serde(default)]
    pub last_move: [i8; 2],
    pub script: Option<ScriptState>,
    pub presentation: Option<Presentation>,
    pub event_headers: Vec<u8>,
    #[serde(default)]
    pub schedule: Vec<u8>,
    #[serde(default)]
    pub region_state: Vec<u8>,
    #[serde(default)]
    pub visibility: Vec<u8>,
    #[serde(default)]
    pub floor: Vec<i16>,
    #[serde(default)]
    pub objects: Vec<i16>,
    #[serde(default)]
    pub events: Vec<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapState {
    pub event_headers: Vec<u8>,
    pub regions: Vec<u8>,
    pub visibility: Vec<u8>,
    pub floor: Vec<i16>,
    pub objects: Vec<i16>,
    pub events: Vec<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LevelState {
    pub maps: std::collections::BTreeMap<i32, MapState>,
    pub schedule: Vec<u8>,
    pub menus: Vec<u8>,
    #[serde(default)]
    pub typed_encounters: Vec<u8>,
}

impl MapState {
    pub fn from_native(bytes: &[u8]) -> Result<Self, RuleError> {
        let bytes = centurions_content::span(bytes, 0, 77744)?;
        Ok(Self {
            event_headers: bytes[..4200].to_vec(),
            regions: bytes[4200..4844].to_vec(),
            visibility: bytes[4844..12944].to_vec(),
            floor: bytes[12944..29144]
                .as_chunks::<2>()
                .0
                .iter()
                .map(|v| i16::from_be_bytes(*v))
                .collect(),
            objects: bytes[29144..45344]
                .as_chunks::<2>()
                .0
                .iter()
                .map(|v| i16::from_be_bytes(*v))
                .collect(),
            events: bytes[45344..77744]
                .as_chunks::<4>()
                .0
                .iter()
                .map(|v| i32::from_be_bytes(*v))
                .collect(),
        })
    }
    pub fn native_bytes(&self) -> Result<Vec<u8>, RuleError> {
        if self.event_headers.len() != 4200
            || self.regions.len() != 644
            || self.visibility.len() != 8100
            || self.floor.len() != 8100
            || self.objects.len() != 8100
            || self.events.len() != 8100
        {
            return Err(RuleError::Invalid("invalid native map records".into()));
        }
        let mut bytes = self.event_headers.clone();
        bytes.extend(&self.regions);
        bytes.extend(&self.visibility);
        bytes.extend(self.floor.iter().flat_map(|v| v.to_be_bytes()));
        bytes.extend(self.objects.iter().flat_map(|v| v.to_be_bytes()));
        bytes.extend(self.events.iter().flat_map(|v| v.to_be_bytes()));
        Ok(bytes)
    }
    pub fn authored(map: &centurions_content::mission::OpeningMission) -> Self {
        Self {
            event_headers: map.header.clone(),
            regions: map.regions.clone(),
            visibility: vec![0; 8100],
            floor: map.floor.clone(),
            objects: map.objects.clone(),
            events: map.events.clone(),
        }
    }
    fn validate(&self, map: &centurions_content::mission::OpeningMission) -> Result<(), RuleError> {
        if self.event_headers.len() != map.header.len()
            || self.regions.len() != map.regions.len()
            || self.visibility.len() != 8100
            || self.floor.len() != map.floor.len()
            || self.objects.len() != map.objects.len()
            || self.events.len() != map.events.len()
        {
            return Err(RuleError::Invalid("invalid persistent map records".into()));
        }
        Ok(())
    }
}

impl Exploration {
    pub fn map_state(&self) -> MapState {
        MapState {
            event_headers: self.event_headers.clone(),
            regions: self.region_state.clone(),
            visibility: self.visibility.clone(),
            floor: self.floor.clone(),
            objects: self.objects.clone(),
            events: self.events.clone(),
        }
    }
    pub fn restore_map(&mut self, map: i32, records: MapState) {
        self.map = map;
        self.event_headers = records.event_headers;
        self.region_state = records.regions;
        self.visibility = records.visibility;
        self.floor = records.floor;
        self.objects = records.objects;
        self.events = records.events;
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScriptState {
    pub record: Vec<u8>,
    pub cursor: u8,
    pub map_slot: usize,
    #[serde(default)]
    pub scheduled: bool,
    /// The manual encounter caller decodes Items' negative script return.
    #[serde(default)]
    pub item_return: bool,
    #[serde(default)]
    pub menu: Option<MenuContinuation>,
    #[serde(default)]
    pub persist: bool,
    #[serde(default)]
    pub origin: Option<ScriptOrigin>,
}

impl ScriptState {
    fn valid(&self, content: &Content) -> bool {
        self.record.len() == 42
            && self.cursor <= 8
            && self.map_slot < 100
            && self.menu.as_ref().is_none_or(|menu| {
                menu.original.len() == 42
                    && menu.id >= 0
                    && content.map(0).is_some_and(|map| {
                        if menu.typed {
                            (menu.id as usize + 1) * typed_encounter::RECORD_BYTES
                                <= map.typed_encounters.len()
                        } else {
                            (menu.id as usize + 1) * story::MENU_RECORD_BYTES <= map.menus.len()
                        }
                    })
            })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScriptOrigin {
    pub map: i32,
    pub x: i32,
    pub y: i32,
    #[serde(default)]
    pub level: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MenuContinuation {
    pub id: i16,
    pub remaining: i16,
    pub original: Vec<u8>,
    #[serde(default)]
    pub typed: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Presentation {
    Messages {
        pages: Vec<(i16, String)>,
        next: Option<Box<Presentation>>,
    },
    Prologue {
        resource_id: i16,
    },
    Dialogue {
        id: i16,
        text: String,
    },
    Question {
        extra: i16,
        text: String,
    },
    Avoidance {
        region: u8,
        text: String,
    },
    Menu {
        id: i16,
        text: String,
        labels: Vec<String>,
    },
    Typed {
        id: i16,
        text: String,
    },
    Loot,
    Shop,
    Clinic {
        member: usize,
        percent: i16,
    },
    Survivors,
    StageChange {
        parameter: i16,
    },
    WarriorChoice {
        extra: i16,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandPreferences {
    pub auto_refit: bool,
    #[serde(default)]
    pub suppress_round_message_sound: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Command {
    ClinicControl {
        item: u16,
    },
    OpenShop,
    CloseShop,
    BuyItem {
        warrior: usize,
        stock: usize,
    },
    SellItem {
        warrior: usize,
        slot: usize,
    },
    SwapCombatants {
        body: i16,
    },
    AttackFriend {
        body: i16,
    },
    ViewCreature {
        index: usize,
        ally: bool,
    },
    ViewNote {
        index: usize,
    },
    WithPreferences {
        command: Box<Command>,
        preferences: CommandPreferences,
    },
    AutomaticBandage,
    CreationDialog,
    CreationEquipment,
    CreationScans,
    GenerateWarrior {
        request: creation::Request,
    },
    SelectWarriors {
        warriors: Vec<Warrior>,
    },
    ReplaceSquad {
        warriors: Vec<Warrior>,
    },
    ReorderSquad {
        destinations: Vec<usize>,
    },
    EndMission,
    RefitParty,
    DropItem {
        warrior: usize,
        slot: usize,
    },
    TransferItem {
        from: usize,
        to: usize,
        slot: usize,
    },
    SetWarriorAppearance {
        warrior: usize,
        portrait: bool,
        resource: i16,
    },
    ViewWarrior {
        warrior: usize,
    },
    CombatClick {
        x: i16,
        y: i16,
    },
    UpgradeWarrior {
        warrior: usize,
        selector: u16,
    },
    SetDifficulty {
        level: i16,
    },
    PoolCredits,
    ShareCredits,
    RecalculateStatistics {
        warrior: usize,
    },
    EditWarriorUpgrade {
        warrior: usize,
        selector: u16,
    },
    EditCarriedItem {
        warrior: usize,
        slot: usize,
        control: u16,
    },
    EditWarriorLevel {
        warrior: usize,
    },
    EditWarriorScans {
        warrior: usize,
        controls: Vec<u16>,
    },
    ImportStarters,
    SelectStarters {
        indices: Vec<usize>,
    },
    EnterMission {
        name: String,
    },
    Encounter,
    Move {
        dx: i8,
        dy: i8,
    },
    Continue,
    KeepCurrentSpells,
    BeginPromotion,
    PromotionControl {
        item: u16,
    },
    PromotionAnswer {
        yes: bool,
    },
    Choose {
        answer: bool,
    },
    CancelMenu,
    SelectMenuEntry {
        index: usize,
    },
    BeginEncounterActions,
    CastEncounterPsi {
        warrior: usize,
        spell: i16,
        power: i16,
    },
    EncounterActionControl {
        item: u16,
    },
    SubmitText {
        text: String,
    },
    ChooseWarrior {
        index: usize,
    },
    TakeLoot {
        slot: usize,
        warrior: usize,
    },
    StoreLoot {
        slot: usize,
    },
    TakeCrate {
        slot: usize,
        warrior: usize,
    },
    EquipItem {
        warrior: usize,
        slot: usize,
    },
    InspectInventory {
        members: [usize; 2],
        scroll: [usize; 2],
        selected: Option<(usize, usize)>,
    },
    UnequipItem {
        warrior: usize,
        slot: usize,
    },
    EndTurn,
    Guard,
    SelectSurvivor {
        row: usize,
    },
    RunEnemyTurn,
    ResumeCombatSelection,
    RunControlledTurn,
    AttackEnemy {
        body: i16,
    },
    ShootEnemy {
        body: i16,
    },
    ShootAt {
        x: i16,
        y: i16,
    },
    FireSelectedTargets,
    CancelFireTargets,
    CastPsi {
        #[serde(default)]
        rotation: u8,
        spell: i16,
        power: i16,
        target: Option<(i16, i16)>,
    },
    BeginPsiSelection {
        spell: i16,
        power: i16,
    },
    SelectPsiTarget {
        x: i16,
        y: i16,
    },
    FirePsiSelection,
    CancelPsiSelection,
    SelectOutdoorPsiTarget {
        warrior: usize,
    },
    FinishOutdoorPsi,
    CancelOutdoorPsi,
    CastPartyPsi {
        warrior: usize,
        spell: i16,
        power: i16,
    },
    Reload {
        warrior: usize,
    },
    FitAmmunition {
        warrior: usize,
        slot: usize,
        ammunition: i16,
    },
    SetFirearmMode {
        warrior: usize,
        enabled: bool,
    },
    UseAmmunitionPack {
        warrior: usize,
        slot: usize,
    },
    UseItem {
        warrior: usize,
        slot: usize,
    },
    BeginMedicalItem {
        warrior: usize,
        slot: usize,
    },
    BeginLearnedHeal {
        power: i16,
    },
    SelectMedicalTarget {
        selection: MedicalTarget,
    },
    StopBleeding {
        warrior: Option<usize>,
    },
    AddToCrate {
        item: InventoryRecord,
    },
    AddToBooty {
        item: InventoryRecord,
    },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Event {
    ItemUpgraded {
        warrior: usize,
        slot: usize,
    },
    ItemTransferred {
        from: usize,
        to: usize,
        item: i16,
    },
    ExchangeOffered {
        body: i16,
    },
    FriendAttackOffered {
        body: i16,
    },
    MissionEnded {
        defeated: bool,
    },
    CreatureViewed {
        index: usize,
        ally: bool,
        record: Vec<u8>,
        description: String,
    },
    NoteViewed {
        index: usize,
        record: Vec<u8>,
        text: String,
    },
    Feedback(feedback::NativeFeedback),
    CreditsPooled {
        total: i32,
    },
    CreditsShared {
        each: i16,
    },
    StatisticsCalculated {
        warrior: usize,
    },
    StartersImported {
        count: usize,
    },
    ItemAdded {
        slot: usize,
        crate_storage: bool,
    },
    Sound {
        resource_id: i16,
    },
    Delay {
        ticks: u32,
    },
    StorageFull,
    CollectionRejected {
        warning: i16,
    },
    MissionEntered {
        title: String,
        x: i32,
        y: i32,
    },
    Moved {
        x: i32,
        y: i32,
    },
    StoryPresented,
    Message {
        id: i16,
        text: String,
    },
    AllyAdded {
        definition: i16,
    },
    OutdoorEffect {
        effect: i16,
        x: i16,
        y: i16,
        size: i16,
        delay: i16,
    },
    CombatStarted {
        battle: i16,
        enemies: usize,
    },
    TacticalMoved {
        body: i16,
        x: i16,
        y: i16,
    },
    AttackResolved {
        attacker: i16,
        target: i16,
        damage: i16,
        defeated: bool,
    },
    SpellResolved {
        caster: Option<i16>,
        spell: i16,
        target: i16,
        damage: i16,
        defeated: bool,
    },
    CombatantAdded {
        body: i16,
        definition: i16,
    },
    CombatantRemoved {
        body: i16,
    },
    TurnStarted {
        body: i16,
        round: i16,
    },
    BattleEnded {
        victory: bool,
    },
    TimeAdvanced {
        day: i16,
        hour: i16,
        minute: i16,
    },
    ScheduleRescheduled {
        slot: usize,
        next_day: i16,
        dispatched: bool,
    },
    AmmunitionAdded {
        ammunition: i16,
        quantity: i16,
        reserve: i32,
    },
    LevelAdvanced {
        warrior: usize,
        level: i16,
    },
    EditorLevelAdvanced {
        warrior: usize,
        level: i16,
        information: bool,
    },
}

#[derive(Debug, Error)]
pub enum RuleError {
    #[error("action refused by the original rules")]
    Refused {
        feedback: Vec<feedback::NativeFeedback>,
    },
    /// A native refusal reached after deliberate rule-state effects.
    #[error("action refused by the original rules")]
    RefusedAfterEffects {
        feedback: Vec<feedback::NativeFeedback>,
    },
    #[error("unsupported action {action}: {dependency}")]
    Unsupported {
        action: &'static str,
        dependency: &'static str,
    },
    #[error("invalid session: {0}")]
    Invalid(String),
    #[error("{0}")]
    Content(#[from] ContentError),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Json(#[from] serde_json::Error),
}

impl From<String> for RuleError {
    fn from(message: String) -> Self {
        Self::Invalid(message)
    }
}
impl From<&str> for RuleError {
    fn from(message: &str) -> Self {
        Self::Invalid(message.into())
    }
}

impl Session {
    pub(crate) fn load_item_work(&mut self, content: &Content, id: i16) -> Result<(), RuleError> {
        self.loaded_item_field32 = Some(self.ammunition_work.load_item(content, id)?);
        Ok(())
    }
    pub fn new(content: &Content) -> Self {
        Self {
            content_identity: content.identity.clone(),
            difficulty: 0,
            command_preferences: CommandPreferences::default(),
            warriors: vec![],
            phase: Phase::SquadSelection,
            mission: None,
            scenario_level: 0,
            visited_levels: Default::default(),
            native_import: None,
            shops: Default::default(),
            shop_access: None,
            confiscated: None,
            suppress_allies: 0,
            loot_limit: None,
            typed_encounters: vec![],
            exploration: None,
            random: RandomState::new(12),
            crate_items: vec![InventoryRecord([0; 22]); 1000],
            booty_items: vec![InventoryRecord([0; 22]); 1000],
            ammunition: empty_ammunition(),
            combat: None,
            battle_treasure: 0,
            battle_after_message: 0,
            battle_group_script: None,
            battle_npc: None,
            party_mask: [0; 6],
            selected_warrior: 0,
            selected_enemy: 0,
            script_enemy: 0,
            loaded_item_field32: None,
            ammunition_work: ammunition::WorkBuffers::default(),
            inventory_record: None,
            loot_quality: None,
            attack_count: 0,
            attack_packet: Some(vec![0; 58]),
            attack_identity: Some([0; 3]),
            // CODE 1 startup initializes retained work words at 453776/4537f0/45375a to zero.
            weapon_work: combat::weapon_work::WeaponWork::zero(),
            psi_memory: Some(vec![0; 480]),
            party_buffs: empty_party_buffs(),
            scenario_duration_tail: None,
            psi_disabled: 0,
            enemy_spell_disabled: 0,
            energy_recovery_disabled: 0,
            health_context: None,
            party_currency: 0,
            library_random_seed: 1,
            library_random_draws: 0,
            combat_story: None,
            survivors: None,
            round_dispatch: None,
            scripted_cleanup: None,
            actor_replacement: None,
            movement_continuation: None,
            time_continuation: None,
            psi_selection: None,
            outdoor_psi: None,
            encounter_actions: None,
            promotion: None,
            medical_continuation: None,
            psi_menu_choices: Default::default(),
            bandage_ready: None,
            handled_deaths: vec![],
            menus: vec![],
            flags: initial_flags(),
            mission_score: 0,
            mission_stage: 0,
            map_picture: 0,
            notes: empty_notes(),
            visited_maps: std::collections::BTreeMap::new(),
            allies: empty_allies(),
            ally_count: 0,
        }
    }

    pub fn validate(&self, content: &Content) -> Result<(), RuleError> {
        shop::validate(self, content)?;
        if let Some(saved) = &self.confiscated {
            saved.validate()?;
        }
        if let Some(imported) = &self.native_import {
            imported.validate()?;
        }
        let context = content.context(self.mission.as_deref(), self.scenario_level)?;
        let content = context.as_ref();
        self.ammunition_work.validate()?;
        if self
            .loot_quality
            .as_ref()
            .is_some_and(|quality| quality.len() != 1000)
        {
            return Err(RuleError::Invalid("loot quality record length".into()));
        }
        if self.content_identity != content.identity {
            return Err(RuleError::Invalid("content identity mismatch".into()));
        }
        if self.warriors.len() > 6
            || !(-2..=2).contains(&self.difficulty)
            || self.crate_items.len() != 1000
            || self.booty_items.len() != 1000
            || self.ammunition.len() != 200
            || self.ammunition.iter().any(|v| *v < 0)
            || self.flags.len() != 128
            || self.notes.len() != 19
            || self.allies.len() != story_allies::ALLY_CAPACITY * story_allies::ALLY_BYTES
            || self.ally_count > story_allies::ALLY_CAPACITY
            || self.selected_warrior >= self.warriors.len().max(1)
            || self.selected_enemy >= 100
            || self.script_enemy >= 100
            || self.party_buffs.len() != 20
            || self.time_continuation.as_ref().is_some_and(|pending| {
                self.exploration
                    .as_ref()
                    .is_none_or(|position| pending.schedule_cursor > position.schedule.len() / 40)
                    || !(0..=60).contains(&pending.minutes_per_tick)
            })
            || self
                .psi_memory
                .as_ref()
                .is_some_and(|memory| memory.len() != 480)
            || self
                .attack_packet
                .as_ref()
                .is_some_and(|packet| packet.len() != 58)
            || (!self.menus.is_empty()
                && !menu_length(
                    self.menus.len(),
                    content.map(0).map_or(0, |map| map.menus.len()),
                ))
            || (!self.typed_encounters.is_empty()
                && self.typed_encounters.len()
                    != content.map(0).map_or(0, |map| map.typed_encounters.len()))
        {
            return Err(RuleError::Invalid(
                "record counts exceed original capacities".into(),
            ));
        }
        for warrior in &self.warriors {
            if Warrior::read(&warrior.bytes)?.name != warrior.name {
                return Err(RuleError::Invalid(
                    "warrior name differs from record".into(),
                ));
            }
            warrior.inventory()?;
        }
        if self.medical_continuation.as_ref().is_some_and(|pending| {
            let descriptor = match pending.origin {
                MedicalOrigin::OwnedNeedle => [0, 3, 0],
                MedicalOrigin::LearnedHeal => [0, 0, 0],
            };
            let valid_identity = match pending.origin {
                MedicalOrigin::OwnedNeedle => {
                    (700..=703).contains(&pending.item)
                        && pending.spell == 1401
                        && pending.power == 1
                }
                MedicalOrigin::LearnedHeal => {
                    pending.item == 0 && pending.spell == 1101 && (1..=7).contains(&pending.power)
                }
            };
            self.phase != Phase::Combat
                || pending.actor >= self.warriors.len()
                || !valid_identity
                || self.attack_count != pending.power
                || self.combat.as_ref().is_none_or(|arena| {
                    arena.active_body() != Some(pending.actor as i16)
                        || arena.attack_identity != Some(descriptor)
                        || arena.attack_packet.as_ref().is_none_or(|packet| {
                            packet.len() != 58
                                || centurions_content::word(packet, 40).ok() != Some(57)
                        })
                })
        }) {
            return Err(RuleError::Invalid("invalid medical continuation".into()));
        }
        if self
            .psi_menu_choices
            .iter()
            .flatten()
            .any(|choice| choice.row > 6 || choice.column > 6 || !(1..=7).contains(&choice.power))
        {
            return Err(RuleError::Invalid(
                "invalid remembered psi selection".into(),
            ));
        }
        if let Some(story) = &self.combat_story {
            story.validate(self, content)?;
        }
        if self.handled_deaths.iter().any(|i| {
            self.combat
                .as_ref()
                .is_none_or(|c| *i >= c.enemies.records.len())
        }) {
            return Err(RuleError::Invalid("invalid combat story state".into()));
        }
        if self.phase == Phase::Combat {
            if self
                .combat
                .as_ref()
                .is_some_and(|combat| combat.difficulty != self.difficulty)
            {
                return Err(RuleError::Invalid(
                    "combat difficulty differs from the session".into(),
                ));
            }
            self.combat
                .as_ref()
                .ok_or_else(|| RuleError::Invalid("combat state missing".into()))?
                .validate(&self.warriors)?;
        } else if self.combat.is_some() {
            return Err(RuleError::Invalid(
                "combat state present outside battle".into(),
            ));
        }
        if let Some(pending) = &self.promotion {
            pending.validate(self, content)?;
        }
        clinic::validate(self)?;
        story_cleanup::validate(self)?;
        story_actor::validate(self, content)?;
        if self.round_dispatch.as_ref().is_some_and(|frame| {
            frame
                .suspended_script
                .as_ref()
                .is_some_and(|script| !script.valid(content))
                || self
                    .exploration
                    .as_ref()
                    .is_none_or(|position| position.script.is_none())
                || (self.combat.is_none()
                    && self.scripted_cleanup != Some(story_cleanup::Stage::Complete))
                || self.combat.as_ref().is_some_and(|arena| {
                    arena
                        .round_scripts
                        .pending
                        .as_ref()
                        .is_none_or(|program| !program.delegated)
                })
        }) {
            return Err(RuleError::Invalid("invalid round dispatch frame".into()));
        }
        if let Some(selection) = &self.survivors {
            selection.validate(
                &self
                    .combat
                    .as_ref()
                    .ok_or("survivor roster missing")?
                    .enemies
                    .records,
            )?;
            if !matches!(
                self.exploration
                    .as_ref()
                    .and_then(|p| p.presentation.as_ref()),
                Some(Presentation::Survivors)
            ) {
                return Err(RuleError::Invalid("survivor dialog missing".into()));
            }
        }
        if let Some(combat) = &self.combat {
            if combat.selection_pending
                && self
                    .exploration
                    .as_ref()
                    .is_none_or(|position| position.presentation.is_some())
            {
                return Err(RuleError::Invalid(
                    "combat selection conflicts with a dialog".into(),
                ));
            }
            if let Some(program) = &combat.round_scripts.pending {
                program.validate(
                    self.map(
                        content,
                        self.exploration.as_ref().ok_or("round map missing")?.map,
                    )
                    .ok_or("round content missing")?,
                )?;
                if combat.cursor != 110
                    || (program.delegated && self.round_dispatch.is_none())
                    || (!program.delegated
                        && !matches!(self.exploration.as_ref().and_then(|p| p.presentation.as_ref()), Some(Presentation::Dialogue { id, .. }) if Some(*id) == program.text))
                {
                    return Err(RuleError::Invalid(
                        "invalid suspended round presentation".into(),
                    ));
                }
            } else if combat.cursor == 110
                && !combat.selection_pending
                && !matches!(
                    self.exploration
                        .as_ref()
                        .and_then(|p| p.presentation.as_ref()),
                    Some(Presentation::Survivors)
                )
            {
                return Err(RuleError::Invalid("round wrap has no continuation".into()));
            }
        }
        outdoor_psi::validate(self, content)?;
        typed_encounter::validate_actions(self, content)?;
        if let Some(pending) = &self.psi_selection {
            pending.validate(self)?;
        }
        for (index, records) in &self.visited_maps {
            records.validate(
                content
                    .map(*index)
                    .ok_or_else(|| RuleError::Invalid("visited map is unavailable".into()))?,
            )?;
            if self
                .exploration
                .as_ref()
                .is_some_and(|position| position.map == *index)
            {
                return Err(RuleError::Invalid(
                    "active map duplicated in visited state".into(),
                ));
            }
        }
        if matches!(self.phase, Phase::Exploration | Phase::Combat) {
            let position = self
                .exploration
                .as_ref()
                .ok_or_else(|| RuleError::Invalid("missing exploration state".into()))?;
            let map = content
                .map(position.map)
                .ok_or_else(|| RuleError::Invalid("active map is unavailable".into()))?;
            position.map_state().validate(map)?;
            if self.mission_stage < 0
                || self.mission_stage >= 20
                || !(1..=6).contains(&self.warriors.len())
                || self.mission.as_deref() != content.scenario.as_ref().map(|s| s.name.as_str())
                || !(0..90).contains(&position.x)
                || !(0..90).contains(&position.y)
                || !(0..24).contains(&position.hour)
                || !(0..60).contains(&position.minute)
                || position.day < 1
                || !(4..=135).contains(&position.fatigue)
                || position.facing > 1
                || position
                    .script
                    .as_ref()
                    .is_some_and(|script| !script.valid(content))
                || (position.presentation.is_some()
                    && position.script.is_none()
                    && !((self.combat_story.is_some()
                        || self
                            .combat
                            .as_ref()
                            .is_some_and(|combat| combat.round_scripts.pending.is_some()))
                        && matches!(position.presentation, Some(Presentation::Dialogue { .. })))
                    && !matches!(
                        position.presentation,
                        Some(
                            Presentation::Prologue { .. }
                                | Presentation::Messages { .. }
                                | Presentation::Survivors
                                | Presentation::Loot
                                | Presentation::Avoidance { .. }
                        )
                    ))
                || position.event_headers.len() != 4200
                || position.schedule.len() != content.map(0).map_or(0, |map| map.schedule.len())
                || !position.schedule.len().is_multiple_of(40)
                || position.region_state.len() != 644
                || matches!(
                    position.presentation,
                    Some(Presentation::Avoidance { region: 20.., .. })
                )
            {
                return Err(RuleError::Invalid("invalid mission state".into()));
            }
        } else if self.exploration.is_some()
            || self.mission.is_some()
            || self.movement_continuation.is_some()
        {
            return Err(RuleError::Invalid(
                "mission state present during squad selection".into(),
            ));
        }
        if self.visited_levels.contains_key(&self.scenario_level) {
            return Err(RuleError::Invalid(
                "active level duplicated in visited state".into(),
            ));
        }
        for (level, records) in &self.visited_levels {
            let context = content.context(self.mission.as_deref(), *level)?;
            let map = context.map(0).ok_or("visited level unavailable")?;
            if records.schedule.len() != map.schedule.len()
                || (!records.menus.is_empty() && !menu_length(records.menus.len(), map.menus.len()))
                || (!records.typed_encounters.is_empty()
                    && records.typed_encounters.len() != map.typed_encounters.len())
            {
                return Err(RuleError::Invalid("invalid persistent level tables".into()));
            }
            for (index, records) in &records.maps {
                records.validate(context.map(*index).ok_or("visited level map unavailable")?)?;
            }
        }
        Ok(())
    }

    pub fn map<'a>(
        &self,
        content: &'a Content,
        index: i32,
    ) -> Option<&'a centurions_content::mission::OpeningMission> {
        content.map_at(self.mission.as_deref(), self.scenario_level, index)
    }
}

pub fn apply(
    mut command: Command,
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    let selection = if let Command::EnterMission { name } = &command {
        Some(name.as_str())
    } else {
        state.mission.as_deref()
    };
    let context = content.context(selection, state.scenario_level)?;
    let content = context.as_ref();
    if let Command::WithPreferences {
        command,
        preferences,
    } = command
    {
        if matches!(*command, Command::WithPreferences { .. }) {
            return Err("nested command preferences".into());
        }
        let mut candidate = state.clone();
        let previous = candidate.command_preferences.clone();
        candidate.command_preferences = preferences;
        let result = apply(*command, &mut candidate, content);
        candidate.command_preferences = previous;
        if result.is_ok() || matches!(result, Err(RuleError::RefusedAfterEffects { .. })) {
            *state = candidate;
        }
        return result;
    }
    state.validate(content)?;
    if let Command::CombatClick { x, y } = command {
        let Some(resolved) = combat::click_command(state, x, y)? else {
            return Ok(vec![]);
        };
        command = resolved;
    }
    let mut candidate = state.clone();
    let editing = matches!(
        command,
        Command::EditWarriorUpgrade { .. }
            | Command::CreationDialog
            | Command::CreationEquipment
            | Command::CreationScans
            | Command::GenerateWarrior { .. }
            | Command::ReplaceSquad { .. }
            | Command::ReorderSquad { .. }
            | Command::EndMission
            | Command::RefitParty
            | Command::DropItem { .. }
            | Command::TransferItem { .. }
            | Command::OpenShop
            | Command::CloseShop
            | Command::BuyItem { .. }
            | Command::SellItem { .. }
            | Command::SetWarriorAppearance { .. }
            | Command::ViewWarrior { .. }
            | Command::InspectInventory { .. }
            | Command::ViewCreature { .. }
            | Command::ViewNote { .. }
            | Command::UpgradeWarrior { .. }
            | Command::EditCarriedItem { .. }
            | Command::EditWarriorLevel { .. }
            | Command::EditWarriorScans { .. }
    );
    let melee_input = matches!(
        command,
        Command::AttackEnemy { .. } | Command::AttackFriend { .. } | Command::Move { .. }
    );
    let physical_work_input = matches!(
        command,
        Command::AttackEnemy { .. }
            | Command::AttackFriend { .. }
            | Command::Move { .. }
            | Command::RunEnemyTurn
            | Command::RunControlledTurn
    );
    let unmodeled_inventory_write = matches!(
        command,
        Command::TakeLoot { .. }
            | Command::StoreLoot { .. }
            | Command::TakeCrate { .. }
            | Command::BuyItem { .. }
            | Command::SellItem { .. }
            | Command::RefitParty
            | Command::Reload { .. }
            | Command::FitAmmunition { .. }
            | Command::UseAmmunitionPack { .. }
    );
    let mut events = match apply_inner(command, &mut candidate, content) {
        Ok(events) => events,
        Err(error @ RuleError::RefusedAfterEffects { .. }) => {
            candidate.validate(content)?;
            *state = candidate;
            return Err(error);
        }
        Err(error) => return Err(error),
    };
    if unmodeled_inventory_write {
        candidate.inventory_record = None;
    }
    for event in &events {
        match event {
            Event::AttackResolved { damage, .. } if melee_input || *damage > 0 => {
                candidate.bandage_ready = Some(0);
            }
            Event::SpellResolved { spell, damage, .. } if *damage > 0 || *spell == 1305 => {
                candidate.bandage_ready = Some(0);
            }
            Event::TurnStarted { .. } => candidate.bandage_ready = Some(1),
            _ => {}
        }
    }
    if !editing
        && !events
            .iter()
            .any(|event| matches!(event, Event::ItemUpgraded { .. }))
        && !events
            .iter()
            .any(|event| matches!(event, Event::Message { .. }))
    {
        events.extend(exploration::resume_movement(&mut candidate, content)?);
    }
    if physical_work_input && let Some(item) = candidate.ammunition_work.item.as_ref() {
        candidate.loaded_item_field32 = Some(centurions_content::word(item, 32)?);
    }
    combat::runtime::retain_last_defeated(&mut candidate);
    if let Some(combat) = candidate.combat.as_ref() {
        candidate.selected_enemy = combat.selected_enemy;
        candidate.script_enemy = combat.script_enemy;
        candidate.attack_packet = combat.attack_packet.clone();
        candidate.attack_identity = combat.attack_identity;
        candidate.weapon_work = combat.weapon_work;
        let mut health = story_health::context(
            &candidate.warriors,
            candidate.selected_warrior,
            candidate.health_context.as_ref(),
        );
        health.unavailable = combat.unavailable;
        if let Some(packet) = combat.attack_packet.as_ref() {
            health.effect = packet[18] as i8;
        }
        candidate.health_context = Some(health);
    }
    let pages: Vec<_> = events
        .iter()
        .filter_map(|event| {
            if let Event::Message { id, text } = event {
                Some((*id, text.clone()))
            } else {
                None
            }
        })
        .collect();
    if !pages.is_empty()
        && let Some(position) = candidate.exploration.as_mut()
    {
        let next = position.presentation.take().map(Box::new);
        position.presentation = Some(Presentation::Messages { pages, next });
    }
    candidate.validate(content)?;
    *state = candidate;
    Ok(events)
}

fn apply_inner(
    command: Command,
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    if state
        .combat
        .as_ref()
        .is_some_and(|combat| combat.selection_pending)
        && !matches!(
            command,
            Command::ResumeCombatSelection
                | Command::ViewWarrior { .. }
                | Command::ViewCreature { .. }
        )
    {
        return Err(RuleError::Refused {
            feedback: vec![feedback::NativeFeedback::Silent],
        });
    }
    let free_upgrade = matches!(command, Command::EditWarriorUpgrade { .. });
    if state
        .exploration
        .as_ref()
        .is_some_and(|p| matches!(p.presentation, Some(Presentation::Clinic { .. })))
        && !matches!(
            command,
            Command::ClinicControl { .. }
                | Command::ViewWarrior { .. }
                | Command::UpgradeWarrior { .. }
                | Command::EditWarriorUpgrade { .. }
        )
    {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    if matches!(
        command,
        Command::SetWarriorAppearance { .. }
            | Command::ViewWarrior { .. }
            | Command::UpgradeWarrior { .. }
            | Command::EditWarriorUpgrade { .. }
            | Command::EditCarriedItem { .. }
            | Command::EditWarriorLevel { .. }
            | Command::EditWarriorScans { .. }
    ) && !warrior_management_available(state)
    {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    if state.medical_continuation.is_some()
        && !matches!(command, Command::SelectMedicalTarget { .. })
    {
        return Err(RuleError::Invalid(
            "finish choosing the medical target before acting".into(),
        ));
    }
    if state.promotion.is_some()
        && !matches!(
            command,
            Command::PromotionControl { .. } | Command::PromotionAnswer { .. }
        )
    {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    if state.encounter_actions.is_some()
        && !matches!(command, Command::EncounterActionControl { .. })
    {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    if state.outdoor_psi.is_some()
        && !matches!(
            command,
            Command::SelectOutdoorPsiTarget { .. }
                | Command::FinishOutdoorPsi
                | Command::CancelOutdoorPsi
        )
    {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    if state.psi_selection.is_some()
        && !matches!(
            command,
            Command::SelectPsiTarget { .. }
                | Command::FirePsiSelection
                | Command::CancelPsiSelection
        )
    {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    if state
        .combat
        .as_ref()
        .is_some_and(|c| c.firearm_selection.is_some())
        && !matches!(
            command,
            Command::ShootEnemy { .. } | Command::FireSelectedTargets | Command::CancelFireTargets
        )
    {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    if combat::runtime::controlled_warrior(state).is_some()
        && matches!(
            command,
            Command::Move { .. }
                | Command::AttackEnemy { .. }
                | Command::AttackFriend { .. }
                | Command::ShootEnemy { .. }
                | Command::ShootAt { .. }
                | Command::Guard
                | Command::CastPsi { .. }
        )
    {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    if state
        .exploration
        .as_ref()
        .is_some_and(|p| p.presentation == Some(Presentation::Shop))
        && !shop::command_allowed(&command)
    {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    if state.phase == Phase::Combat
        && state
            .exploration
            .as_ref()
            .is_some_and(|p| p.presentation.is_some())
        && !matches!(
            command,
            Command::Continue
                | Command::EndMission
                | Command::SelectSurvivor { .. }
                | Command::Choose { .. }
                | Command::ChooseWarrior { .. }
                | Command::SelectMenuEntry { .. }
                | Command::SubmitText { .. }
                | Command::BeginEncounterActions
                | Command::CastEncounterPsi { .. }
                | Command::EncounterActionControl { .. }
                | Command::UseItem { .. }
                | Command::InspectInventory { .. }
                | Command::CancelMenu
                | Command::TakeLoot { .. }
                | Command::StoreLoot { .. }
                | Command::KeepCurrentSpells
                | Command::BeginPromotion
                | Command::PromotionControl { .. }
                | Command::PromotionAnswer { .. }
        )
    {
        return Err(RuleError::Invalid(
            "continue the combat story before acting".into(),
        ));
    }
    match command {
        Command::ClinicControl { item } => clinic::control(state, content, item),
        Command::OpenShop => shop::open(state),
        Command::CloseShop => shop::close(state, content),
        Command::BuyItem { warrior, stock } => shop::buy(state, content, warrior, stock),
        Command::SellItem { warrior, slot } => shop::sell(state, content, warrior, slot),
        Command::TransferItem { from, to, slot } => {
            equipment::transfer(state, content, from, to, slot)
        }
        Command::SwapCombatants { body } => combat::interaction::exchange(state, body),
        Command::ViewCreature { index, ally } => reference_views::view(state, content, index, ally),
        Command::ViewNote { index } => reference_views::note(state, content, index),
        Command::WithPreferences { .. } => unreachable!("preferences unwrap at command boundary"),
        Command::AutomaticBandage => {
            let member = state
                .combat
                .as_ref()
                .and_then(combat::Combat::active_body)
                .filter(|n| (0..6).contains(n))
                .ok_or(RuleError::Refused { feedback: vec![] })? as usize;
            let record = &state.warriors[member].bytes;
            if centurions_content::word(record, 0x90)? <= 0
                || centurions_content::word(record, 0x10)? != 0
                || centurions_content::word(record, 0x5e)? != 0
            {
                return Err(RuleError::Refused { feedback: vec![] });
            }
            let target = state
                .warriors
                .iter()
                .position(|w| w.bytes[0x9de] != 0)
                .ok_or(RuleError::Refused { feedback: vec![] })?;
            state.warriors[target].bytes[0x9de] = 0;
            state.warriors[member].bytes[0x40..0x42].copy_from_slice(&0_i16.to_be_bytes());
            Ok(vec![Event::Feedback(feedback::NativeFeedback::Flash {
                address: 0x403cd6,
                x: 30,
                y: 100,
                wait: 120,
                sound: 10105,
            })])
        }
        Command::CreationDialog => {
            if state.phase != Phase::SquadSelection {
                return Err(RuleError::Refused { feedback: vec![] });
            }
            Ok(vec![Event::Sound {
                resource_id: state.random.bound(2).wrapping_add(200),
            }])
        }
        Command::CreationEquipment => {
            if state.phase != Phase::SquadSelection || state.warriors.len() != 1 {
                return Err(RuleError::Refused { feedback: vec![] });
            }
            creation::initial_items(state, content)
        }
        Command::CreationScans => {
            if state.phase != Phase::SquadSelection || state.warriors.len() != 1 {
                return Err(RuleError::Refused { feedback: vec![] });
            }
            state.warriors[0].bytes[0x9f0..0xa21].fill(0);
            state.psi_memory = Some(initial_psi_memory(&state.warriors)?);
            Ok(vec![])
        }
        Command::GenerateWarrior { request } => {
            if state.phase != Phase::SquadSelection || state.warriors.len() > 1 {
                return Err(RuleError::Refused { feedback: vec![] });
            }
            let warrior = creation::generate(
                &request,
                content,
                &mut state.random,
                &mut state.library_random_seed,
                &mut state.library_random_draws,
            )?;
            state.warriors = vec![warrior];
            state.psi_memory = Some(initial_psi_memory(&state.warriors)?);
            Ok(vec![Event::Sound { resource_id: 130 }])
        }
        Command::SelectWarriors { warriors } => {
            if state.phase != Phase::SquadSelection {
                return Err(RuleError::Refused { feedback: vec![] });
            }
            roster::validate_squad(&warriors)?;
            state.warriors = warriors;
            state.psi_memory = Some(initial_psi_memory(&state.warriors)?);
            for warrior in &state.warriors {
                state.loaded_item_field32 =
                    Some(combat::ally_turns::item_context_after_weight_calc(
                        warrior,
                        &content.items,
                        state.loaded_item_field32.unwrap_or(0),
                    )?);
            }
            state.ammunition_work.item = None;
            Ok(vec![Event::StartersImported {
                count: state.warriors.len(),
            }])
        }
        Command::ReplaceSquad { warriors } => {
            roster::admit_management(state)?;
            roster::validate_squad(&warriors)?;
            state.warriors = warriors;
            state.selected_warrior = 0;
            state.psi_memory = Some(initial_psi_memory(&state.warriors)?);
            state.psi_menu_choices = std::array::from_fn(|_| None);
            state.health_context = None;
            state.loaded_item_field32 = None;
            state.ammunition_work = ammunition::WorkBuffers::default();
            Ok(vec![])
        }
        Command::ReorderSquad { destinations } => {
            roster::admit_management(state)?;
            if destinations.len() != state.warriors.len()
                || destinations.iter().any(|i| *i >= destinations.len())
                || (0..destinations.len()).any(|i| destinations[..i].contains(&destinations[i]))
            {
                return Err(RuleError::Invalid(
                    "squad order is not a permutation".into(),
                ));
            }
            let before = state.warriors.clone();
            let choices = state.psi_menu_choices.clone();
            for (member, destination) in destinations.into_iter().enumerate() {
                state.warriors[destination] = before[member].clone();
                state.psi_menu_choices[destination] = choices[member].clone();
            }
            state.psi_memory = Some(initial_psi_memory(&state.warriors)?);
            state.health_context = None;
            Ok(vec![])
        }
        Command::EndMission => {
            if !matches!(state.phase, Phase::Exploration | Phase::Combat)
                || state.medical_continuation.is_some()
            {
                return Err(RuleError::Refused { feedback: vec![] });
            }
            let random = state.random.clone();
            let library = (state.library_random_seed, state.library_random_draws);
            let scenario_tail = state.scenario_duration_tail;
            *state = Session::new(content);
            state.random = random;
            state.scenario_duration_tail = scenario_tail;
            (state.library_random_seed, state.library_random_draws) = library;
            Ok(vec![])
        }
        Command::RefitParty => {
            if state.phase != Phase::Exploration {
                return Err(RuleError::Refused { feedback: vec![] });
            }
            let mut events = vec![];
            for member in 0..state.warriors.len() {
                let result = ammunition::refit(
                    &state.warriors[member],
                    &state.ammunition,
                    content,
                    ammunition::RefitFlags {
                        in_combat: false,
                        force: false,
                        silent: true,
                        warnings: false,
                    },
                    &state.ammunition_work,
                    &mut state.random,
                )?;
                events.extend(ammunition::commit_result(state, member, result)?);
            }
            Ok(events)
        }
        Command::DropItem { warrior, slot } => {
            if state.phase != Phase::Exploration {
                return Err(RuleError::Refused { feedback: vec![] });
            }
            let mission = state
                .exploration
                .as_ref()
                .and_then(|p| content.map(p.map))
                .ok_or("drop profiles missing")?;
            let record = state
                .warriors
                .get_mut(warrior)
                .ok_or("drop warrior outside squad")?;
            if state.inventory_record.is_some() {
                state.inventory_record = Some(
                    record
                        .inventory()?
                        .get(slot)
                        .cloned()
                        .ok_or("drop slot missing")?,
                );
            }
            equipment::drop(record, slot, content, mission)?;
            Ok(vec![Event::Sound { resource_id: 655 }])
        }
        Command::CombatClick { .. } => {
            unreachable!("combat clicks resolve at the shared command boundary")
        }
        Command::EditWarriorScans { warrior, controls } => {
            if !warrior_management_available(state) || controls.len() > 4096 {
                return Err(RuleError::Invalid(
                    "scan editing requires bounded exploration input".into(),
                ));
            }
            let map = content
                .map(0)
                .ok_or_else(|| RuleError::Invalid("editor profiles unavailable".into()))?;
            let record = state
                .warriors
                .get(warrior)
                .ok_or_else(|| RuleError::Invalid("editor warrior is outside the squad".into()))?;
            let mut selection = story_promotion::ScanSelection::new(record, &map.profiles)?;
            let mut events = vec![Event::Sound { resource_id: 3000 }];
            if !matches!(controls.last(), Some(1 | 32)) {
                return Err(RuleError::Invalid("scan selection requires Done".into()));
            }
            for control in &controls[..controls.len() - 1] {
                match control {
                    2..=8 => {
                        selection.select_tier(usize::from(*control - 2))?;
                        events.push(Event::Sound { resource_id: 137 });
                    }
                    10..=16 => {
                        if let Some(resource_id) = selection.toggle(usize::from(*control - 10))? {
                            events.push(Event::Sound { resource_id });
                        }
                    }
                    _ => return Err(RuleError::Invalid("unknown scan-selection control".into())),
                }
            }
            let attacks = &map
                .combat
                .as_ref()
                .ok_or_else(|| RuleError::Invalid("scan records unavailable".into()))?
                .attacks;
            state.warriors[warrior] = selection.finish(attacks)?;
            story_promotion::cache_scans(state, warrior, attacks)?;
            Ok(events)
        }
        Command::EditWarriorLevel { warrior } => {
            if !warrior_management_available(state) {
                return Err(RuleError::Invalid(
                    "warrior editing requires exploration".into(),
                ));
            }
            let map = content
                .map(0)
                .ok_or_else(|| RuleError::Invalid("editor profiles unavailable".into()))?;
            let record = state
                .warriors
                .get_mut(warrior)
                .ok_or_else(|| RuleError::Invalid("editor warrior is outside the squad".into()))?;
            let mut random = story_promotion::LibraryRandom {
                seed: state.library_random_seed,
                draws: state.library_random_draws,
            };
            let information =
                story_promotion::editor_level_up(record, &map.races, &map.profiles, &mut random)?;
            state.library_random_seed = random.seed;
            state.library_random_draws = random.draws;
            Ok(vec![
                Event::Sound { resource_id: 133 },
                Event::EditorLevelAdvanced {
                    warrior,
                    level: word(&record.bytes, 0x3a)?,
                    information,
                },
            ])
        }
        Command::EditCarriedItem {
            warrior,
            slot,
            control,
        } => editor_items::edit(state, content, warrior, slot, control),
        Command::SetWarriorAppearance {
            warrior,
            portrait,
            resource,
        } => {
            if !warrior_management_available(state) {
                return Err(RuleError::Refused { feedback: vec![] });
            }
            let owner = if portrait { "/Portraits" } else { "/Tacticals" };
            let first = if portrait { 257 } else { 9257 };
            if !(first..first + 120).contains(&resource)
                || !content
                    .resources
                    .iter()
                    .any(|r| r.original.ends_with(owner) && r.r#type == "cicn" && r.id == resource)
            {
                return Err(RuleError::Invalid(
                    "warrior appearance resource is unavailable".into(),
                ));
            }
            let record = state.warriors.get_mut(warrior).ok_or_else(|| {
                RuleError::Invalid("appearance warrior is outside the squad".into())
            })?;
            let offset = if portrait { 0x54 } else { 0x56 };
            record.bytes[offset..offset + 2].copy_from_slice(&resource.to_be_bytes());
            Ok(vec![Event::Sound { resource_id: 130 }])
        }
        Command::EditWarriorUpgrade { warrior, selector }
        | Command::UpgradeWarrior { warrior, selector } => {
            if !warrior_management_available(state) || selector > 32 {
                return Err(RuleError::Invalid(format!(
                    "warrior editing requires exploration and a selector from 0 through 32 (received {selector})"
                )));
            }
            let record = state
                .warriors
                .get_mut(warrior)
                .ok_or_else(|| RuleError::Invalid("editor warrior is outside the squad".into()))?;
            if (23..=31).contains(&selector)
                && word(&record.bytes, 0xf4 + usize::from(selector - 23) * 2)? < 1
                || selector == 21 && word(&record.bytes, 0x36)? == 0
            {
                return Err(RuleError::Refused { feedback: vec![] });
            }
            if word(&record.bytes, 0xace + usize::from(selector) * 2)? < 0 {
                return Err(RuleError::Refused {
                    feedback: vec![feedback::NativeFeedback::Warning { index: 149 }],
                });
            }
            let offset = 0xace + usize::from(selector) * 2;
            if !free_upgrade {
                let map = content
                    .map(state.exploration.as_ref().map_or(0, |p| p.map))
                    .ok_or_else(|| RuleError::Invalid("upgrade MOS tables missing".into()))?;
                let mos = word(&record.bytes, 0x34)?;
                let profile = centurions_content::span(
                    &map.profiles,
                    usize::try_from(mos.wrapping_sub(1))
                        .map_err(|_| RuleError::Invalid("upgrade MOS index".into()))?
                        * 576,
                    576,
                )?;
                let Some(points) = character::upgrade_payment(record, selector, profile)? else {
                    return Err(RuleError::Refused { feedback: vec![] });
                };
                record.bytes[0x9d2..0x9d6].copy_from_slice(&points.to_be_bytes());
            }
            let count = word(&record.bytes, offset)?.wrapping_add(1);
            record.bytes[offset..offset + 2].copy_from_slice(&count.to_be_bytes());
            character::apply_upgrade(record, selector, 1)?;
            character::calculate_statistics(record)?;
            Ok(vec![
                Event::Sound { resource_id: 10129 },
                Event::StatisticsCalculated { warrior },
            ])
        }
        Command::ViewWarrior { warrior } => {
            if state.phase != Phase::Exploration {
                return Err(RuleError::Refused { feedback: vec![] });
            }
            state.warriors.get(warrior).ok_or_else(|| {
                RuleError::Invalid("information warrior is outside the squad".into())
            })?;
            let mut presentation_random = state.random.clone();
            let first = presentation_random.bound(5) as usize - 1;
            let second = presentation_random.bound(5) as usize - 1;
            let sounds = [200, 201, 202, 207, 208];
            let mut events = vec![Event::Sound {
                resource_id: sounds[first],
            }];
            if first != second {
                events.push(Event::Sound {
                    resource_id: sounds[second],
                });
            }
            Ok(events)
        }
        Command::PoolCredits => {
            party::pool_credits(&mut state.warriors, &mut state.party_currency)?;
            Ok(vec![Event::CreditsPooled {
                total: state.party_currency,
            }])
        }
        Command::ShareCredits => {
            let each = party::share_credits(&mut state.warriors, &mut state.party_currency)?;
            Ok(vec![Event::CreditsShared { each }])
        }
        Command::RecalculateStatistics { warrior } => {
            let record = state.warriors.get_mut(warrior).ok_or_else(|| {
                RuleError::Invalid("statistics warrior is outside the squad".into())
            })?;
            character::calculate_statistics(record)?;
            Ok(vec![Event::StatisticsCalculated { warrior }])
        }
        Command::SetDifficulty { level } => {
            if state.phase != Phase::SquadSelection || !(-2..=2).contains(&level) {
                return Err(RuleError::Invalid(
                    "difficulty can only be selected before a mission".into(),
                ));
            }
            state.difficulty = level;
            Ok(vec![])
        }
        Command::ImportStarters | Command::SelectStarters { .. } => {
            if state.phase != Phase::SquadSelection {
                return Err(RuleError::Invalid("squad already entered a mission".into()));
            }
            let indices = match command {
                Command::SelectStarters { indices } => indices,
                _ => (0..content.warriors.len()).collect(),
            };
            if !(1..=6).contains(&indices.len())
                || indices.iter().any(|i| *i >= content.warriors.len())
                || (0..indices.len()).any(|i| indices[..i].contains(&indices[i]))
            {
                return Err(RuleError::Invalid(
                    "select one to six distinct supplied warriors".into(),
                ));
            }
            state.warriors = indices
                .iter()
                .map(|i| content.warriors[*i].clone())
                .collect();
            state.psi_memory = Some(initial_psi_memory(&state.warriors)?);
            for warrior in &state.warriors {
                state.loaded_item_field32 =
                    Some(combat::ally_turns::item_context_after_weight_calc(
                        warrior,
                        &content.items,
                        state.loaded_item_field32.unwrap_or(0),
                    )?);
            }
            state.ammunition_work.item = None;
            Ok(vec![Event::StartersImported {
                count: state.warriors.len(),
            }])
        }
        Command::EnterMission { name } => {
            if state.phase != Phase::SquadSelection || !(1..=6).contains(&state.warriors.len()) {
                return Err(RuleError::Invalid(
                    "mission entry requires one to six selected warriors".into(),
                ));
            }
            let scenario = content.scenario.as_ref().ok_or("scenario unavailable")?;
            let opening = content
                .map(scenario.opening_map)
                .ok_or_else(|| RuleError::Invalid("opening content unavailable".into()))?;
            state.phase = Phase::Exploration;
            state.mission = Some(name);
            state.scenario_level = scenario.level;
            state.exploration = Some(Exploration {
                map: scenario.opening_map,
                x: opening.start.0,
                y: opening.start.1,
                day: 1,
                hour: 12,
                minute: 0,
                facing: 0,
                last_move: [0; 2],
                script: None,
                presentation: None,
                fatigue: 4,
                event_headers: opening.header.clone(),
                schedule: opening.schedule.clone(),
                region_state: opening.regions.clone(),
                visibility: vec![0; 8100],
                floor: opening.floor.clone(),
                objects: opening.objects.clone(),
                events: opening.events.clone(),
            });
            Ok(vec![Event::MissionEntered {
                title: opening.name.clone(),
                x: opening.start.0,
                y: opening.start.1,
            }])
        }
        Command::Move { dx, dy } => {
            if state.phase == Phase::Combat {
                combat::move_warrior(state, content, dx, dy)
            } else {
                exploration::move_squad(state, content, dx, dy)
            }
        }
        Command::Continue => {
            if state.combat.as_ref().is_some_and(|c| {
                c.round_scripts
                    .pending
                    .as_ref()
                    .is_some_and(|program| !program.delegated)
            }) {
                return combat::runtime::continue_round(state, content);
            }
            if matches!(
                state
                    .exploration
                    .as_ref()
                    .and_then(|p| p.presentation.as_ref()),
                Some(Presentation::Loot)
            ) {
                exploration::continue_story(state, content)
            } else if matches!(
                state
                    .exploration
                    .as_ref()
                    .and_then(|p| p.presentation.as_ref()),
                Some(Presentation::Survivors)
            ) {
                if state.scripted_cleanup == Some(story_cleanup::Stage::Survivors) {
                    let mut events = story_cleanup::confirm(state, content)?;
                    events.extend(exploration::run_script(state, content)?);
                    Ok(events)
                } else {
                    combat::runtime::confirm_survivors(state, content)
                }
            } else if matches!(
                state
                    .exploration
                    .as_ref()
                    .and_then(|p| p.presentation.as_ref()),
                Some(Presentation::Messages { .. })
            ) {
                exploration::continue_messages(state, content)
            } else if state
                .combat_story
                .as_ref()
                .is_some_and(|story| !story.waiting_child || state.combat.is_none())
            {
                combat::runtime::continue_death(state, content)
            } else {
                exploration::continue_story(state, content)
            }
        }
        Command::KeepCurrentSpells => exploration::keep_current_spells(state, content),
        Command::BeginPromotion => story_promotion::begin(state, content),
        Command::PromotionControl { item } => story_promotion::control(state, content, item),
        Command::PromotionAnswer { yes } => story_promotion::answer(state, content, yes),
        Command::Choose { answer } => exploration::choose(state, content, answer),
        Command::CancelMenu => exploration::cancel_menu(state, content),
        Command::SelectMenuEntry { index } => exploration::select_menu(state, content, index),
        Command::SubmitText { text } => exploration::submit_text(state, content, &text),
        Command::BeginEncounterActions => typed_encounter::begin_actions(state, content),
        Command::CastEncounterPsi {
            warrior,
            spell,
            power,
        } => typed_encounter::cast_spell(state, content, warrior, spell, power),
        Command::EncounterActionControl { item } => {
            typed_encounter::action_control(state, content, item)
        }
        Command::ChooseWarrior { index } => exploration::choose_warrior(state, content, index),
        Command::TakeLoot { slot, warrior } => {
            exploration::take_loot(state, content, slot, Some(warrior))
        }
        Command::StoreLoot { slot } => exploration::take_loot(state, content, slot, None),
        Command::TakeCrate { slot, warrior } => {
            equipment::take_crate(state, content, slot, warrior)
        }
        Command::EquipItem { warrior, slot } => equipment::equip(state, content, warrior, slot),
        Command::InspectInventory {
            members,
            scroll,
            selected,
        } => equipment::inspect(state, members, scroll, selected),
        Command::UnequipItem { warrior, slot } => equipment::unequip(state, content, warrior, slot),
        Command::Guard => combat::runtime::player_guard(state, content),
        Command::SelectSurvivor { row } => {
            let combat = state.combat.as_ref().ok_or("survivor roster missing")?;
            state
                .survivors
                .as_mut()
                .ok_or("survivor selection missing")?
                .toggle(row, &combat.enemies.records, &mut state.random)
        }
        Command::EndTurn => {
            if let Some(body) = state.combat.as_ref().and_then(|arena| arena.active_body())
                && (0..6).contains(&body)
            {
                state.warriors[body as usize].bytes[0x16..0x18].fill(0);
                state.warriors[body as usize].bytes[0x3c..0x3e].fill(0);
            }
            combat::runtime::end_turn(state, content)
        }
        Command::RunEnemyTurn => combat::runtime::enemy_turn(state, content),
        Command::ResumeCombatSelection => combat::runtime::resume_selection(state, content),
        Command::RunControlledTurn => combat::runtime::controlled_turn(state, content),
        Command::AttackEnemy { body } => combat::runtime::player_attack(state, content, body),
        Command::AttackFriend { body } => {
            combat::runtime::player_attack_friend(state, content, body)
        }
        Command::ShootEnemy { body } => combat::runtime::player_shoot(state, content, body),
        Command::ShootAt { x, y } => combat::shooting::fire_at(state, content, (x, y)),
        Command::FireSelectedTargets => combat::shooting::fire_selected(state, content),
        Command::CancelFireTargets => combat::shooting::cancel(state),
        Command::CastPsi {
            rotation,
            spell,
            power,
            target,
        } => combat::runtime::player_psi(state, content, spell, power, target, rotation),
        Command::CastPartyPsi {
            warrior,
            spell,
            power,
        } => combat::runtime::party_psi(state, content, warrior, spell, power),
        Command::Reload { warrior }
            if state
                .exploration
                .as_ref()
                .is_some_and(|p| p.presentation == Some(Presentation::Shop)) =>
        {
            shop::reload(state, content, warrior)
        }
        Command::Reload { warrior } => combat::firearm::reload_command(state, content, warrior),
        Command::FitAmmunition {
            warrior,
            slot,
            ammunition,
        } => combat::runtime::fit_ammunition(state, content, warrior, slot, ammunition),
        Command::UseAmmunitionPack { warrior, slot } => {
            items::use_ammunition_pack(state, content, warrior, slot)
        }
        Command::UseItem { warrior, slot } => exploration::use_item(state, content, warrior, slot),
        Command::BeginMedicalItem { warrior, slot } => {
            combat::runtime::begin_medical_item(state, content, warrior, slot)
        }
        Command::BeginLearnedHeal { power } => {
            combat::runtime::begin_learned_heal(state, content, power)
        }
        Command::BeginPsiSelection { spell, power } => {
            combat::psi_selection::begin(state, content, spell, power)
        }
        Command::SelectPsiTarget { x, y } => combat::psi_selection::select(state, content, (x, y)),
        Command::FirePsiSelection => combat::psi_selection::fire(state, content),
        Command::CancelPsiSelection => combat::psi_selection::cancel(state, content),
        Command::SelectOutdoorPsiTarget { warrior } => outdoor_psi::select(state, content, warrior),
        Command::FinishOutdoorPsi => outdoor_psi::finish(state, content),
        Command::CancelOutdoorPsi => outdoor_psi::cancel(state, content),
        Command::SelectMedicalTarget { selection } => {
            combat::runtime::select_medical_target(state, content, selection)
        }
        Command::StopBleeding { warrior } => {
            combat::runtime::stop_bleeding(state, content, warrior)
        }
        Command::Encounter => exploration::manual_encounter(state, content),
        Command::SetFirearmMode { warrior, enabled } => {
            let w = state
                .warriors
                .get_mut(warrior)
                .ok_or_else(|| RuleError::Invalid("warrior outside squad".into()))?;
            w.bytes[0x9dd] = u8::from(enabled);
            Ok(vec![])
        }
        Command::AddToCrate { item } => insert(item, &mut state.crate_items, true),
        Command::AddToBooty { item } => insert(item, &mut state.booty_items, false),
    }
}

fn warrior_management_available(state: &Session) -> bool {
    state.medical_continuation.is_none()
        && (state.phase == Phase::SquadSelection && state.warriors.len() == 1
            || state.phase == Phase::Exploration
                && state.exploration.as_ref().is_some_and(|p| {
                    p.presentation.is_none()
                        || matches!(p.presentation, Some(Presentation::Clinic { .. }))
                }))
}

fn empty_ammunition() -> Vec<i32> {
    vec![0; 200]
}

fn empty_party_buffs() -> Vec<u8> {
    vec![0; 20]
}

fn initial_psi_memory(warriors: &[Warrior]) -> Result<Vec<u8>, RuleError> {
    if warriors.len() > 6 {
        return Err(RuleError::Invalid(
            "psi memory exceeds party capacity".into(),
        ));
    }
    let mut memory = vec![0; 480];
    for (index, warrior) in warriors.iter().enumerate() {
        memory[index * 80..(index + 1) * 80].copy_from_slice(centurions_content::span(
            &warrior.bytes,
            0xa76,
            80,
        )?);
    }
    Ok(memory)
}
fn empty_allies() -> Vec<u8> {
    vec![0; story_allies::ALLY_BYTES * story_allies::ALLY_CAPACITY]
}

fn empty_notes() -> Vec<u8> {
    vec![0; 19]
}
fn no_notes(notes: &[u8]) -> bool {
    notes.iter().all(|n| *n == 0)
}
fn zero_level(level: &u8) -> bool {
    *level == 0
}
fn zero_word(value: &i16) -> bool {
    *value == 0
}

fn initial_library_seed() -> u32 {
    1
}

fn initial_flags() -> Vec<u8> {
    let mut flags = vec![0; 128];
    flags[0] = 0xff;
    flags
}

pub mod ammunition;
pub mod character;
pub mod combat;
pub mod editor_items;
pub mod equipment;
mod exploration;
pub mod party;
pub mod reconstruction;
pub mod recovered_ai;
pub mod recovered_application_control;
pub mod recovered_attack;
pub mod recovered_audio;
pub mod recovered_battle_order;
pub mod recovered_bodies;
pub mod recovered_character_creation;
pub mod recovered_character_panels;
pub mod recovered_character_ui;
pub mod recovered_character_upgrade;
pub mod recovered_combat_center;
pub mod recovered_combat_panel;
pub mod recovered_combat_results;
pub mod recovered_combat_setup;
pub mod recovered_combat_ui;
pub mod recovered_detail_ui;
pub mod recovered_drawing;
pub mod recovered_economy;
pub mod recovered_equipment;
pub mod recovered_equipment_ui;
pub mod recovered_fade_ui;
pub mod recovered_feedback;
pub mod recovered_field_control;
pub mod recovered_files;
pub mod recovered_gamma;
pub mod recovered_gamma_control;
pub mod recovered_geometry;
pub mod recovered_install;
pub mod recovered_interface;
pub mod recovered_inventory;
pub mod recovered_inventory_edit;
pub mod recovered_inventory_interaction;
pub mod recovered_inventory_rules;
pub mod recovered_inventory_selection;
pub mod recovered_item_ui;
pub mod recovered_load_control;
pub mod recovered_loot_control;
pub mod recovered_native_services;
pub mod recovered_navigation;
pub mod recovered_order_form;
pub mod recovered_preferences;
pub mod recovered_preparation;
pub mod recovered_prompt_control;
pub mod recovered_random_item;
pub mod recovered_records;
pub mod recovered_rendering;
pub mod recovered_runtime;
pub mod recovered_scenario_menus;
pub mod recovered_selection_control;
pub mod recovered_session_control;
pub mod recovered_shop_ui;
pub mod recovered_spatial;
pub mod recovered_spell_animation;
pub mod recovered_spell_ui;
pub mod recovered_startup;
pub mod recovered_status_panels;
pub mod recovered_storage;
pub mod recovered_system_control;
pub mod recovered_text;
pub mod recovered_timing;
pub mod recovered_verification;
pub mod recovered_world_display;
pub use exploration::{needs_spell_confirmation, typed_record};
pub mod clinic;
mod item_modifiers;
pub mod items;
pub mod mission;
pub mod story;
pub mod story_actor;
pub mod story_allies;
pub mod story_cleanup;
pub mod story_collection;
mod story_combat;
pub mod story_delay;
pub mod story_destination;
pub mod story_health;
pub mod story_inventory;
pub mod story_item_use;
pub mod story_manual_encounter;
pub mod story_map_picture;
pub mod story_move_away;
pub mod story_npc_battle;
pub mod story_party;
pub mod story_promotion;
pub mod story_reduce;
pub mod story_registration;
pub mod story_remove_allies;
pub mod story_schedule;
pub mod story_skill;
mod story_spell;
pub mod story_treasure;
pub mod story_variables;
mod time;
pub mod travel;
pub mod typed_encounter;
pub mod walking_header;

fn insert(
    item: InventoryRecord,
    records: &mut [InventoryRecord],
    crate_storage: bool,
) -> Result<Vec<Event>, RuleError> {
    if let Some(slot) = records.iter().position(|entry| entry.id() == 0) {
        records[slot] = item;
        let mut events = vec![Event::ItemAdded {
            slot,
            crate_storage,
        }];
        if crate_storage {
            events.push(Event::Sound { resource_id: 663 });
        }
        Ok(events)
    } else {
        Ok(vec![Event::StorageFull])
    }
}

#[derive(Serialize, Deserialize)]
struct Save {
    schema_version: u32,
    session: Session,
}

pub(crate) fn normal_difficulty(level: &i16) -> bool {
    *level == 0
}

fn menu_length(actual: usize, authored: usize) -> bool {
    actual == authored || actual == authored / story::MENU_RECORD_BYTES * story::MENU_RECORD_BYTES
}

pub fn save(path: &Path, state: &Session, content: &Content) -> Result<(), RuleError> {
    state.validate(content)?;
    let bytes = serde_json::to_vec_pretty(&Save {
        schema_version: 6,
        session: state.clone(),
    })?;
    atomic_write(path, &bytes)
}

pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), RuleError> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;
    static SAVE_SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let (temporary, mut file) = (0..100)
        .find_map(|_| {
            let sequence = SAVE_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let temporary = parent.join(format!(".session-{}-{sequence}.tmp", std::process::id()));
            match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)
            {
                Ok(file) => Some(Ok((temporary, file))),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => None,
                Err(error) => Some(Err(error)),
            }
        })
        .unwrap_or_else(|| {
            Err(std::io::Error::other(
                "could not allocate a unique save temporary file after 100 attempts",
            ))
        })?;
    let written = file.write_all(bytes).and_then(|()| file.sync_all());
    drop(file);
    // std::fs::rename uses replacement on Windows and Unix, keeping the old save intact until this step.
    if let Err(error) = written.and_then(|()| fs::rename(&temporary, path)) {
        let _ = fs::remove_file(&temporary);
        return Err(error.into());
    }
    Ok(())
}

pub fn load(path: &Path, content: &Content) -> Result<Session, RuleError> {
    let metadata = fs::metadata(path)?;
    let map_count = content.scenario.as_ref().map_or(0, |scenario| {
        scenario
            .catalog
            .values()
            .map(|level| level.maps.len().min(20))
            .sum::<usize>()
            .max(scenario.maps.len().min(20))
            .min(400)
    });
    if metadata.len() > (2_000_000 + map_count * 1_500_000) as u64 {
        return Err(RuleError::Invalid("save exceeds size limit".into()));
    }
    let mut saved: Save = serde_json::from_slice(&fs::read(path)?)?;
    if saved.schema_version == 1 {
        if let Some(position) = saved.session.exploration.as_mut() {
            if position.day != 1 {
                return Err(RuleError::Invalid(
                    "legacy save after day one has unknown schedule history".into(),
                ));
            }
            position.schedule = content
                .map(0)
                .ok_or_else(|| RuleError::Invalid("opening schedule unavailable".into()))?
                .schedule
                .clone();
            position.region_state = content.map(0).unwrap().regions.clone();
        }
    } else if !matches!(saved.schema_version, 2..=6) {
        return Err(RuleError::Invalid(format!(
            "unsupported save version {}",
            saved.schema_version
        )));
    }
    if saved.schema_version < 6
        && (saved.session.time_continuation.is_some()
            || saved.session.scenario_duration_tail.is_some()
            || saved.session.energy_recovery_disabled != 0
            || saved.session.survivors.is_some()
            || saved.session.round_dispatch.is_some()
            || saved.session.scripted_cleanup.is_some()
            || saved.session.actor_replacement.is_some()
            || saved.session.inventory_record.is_some()
            || clinic::pending(&saved.session).is_some()
            || saved.session.psi_selection.is_some()
            || saved.session.outdoor_psi.is_some()
            || saved.session.encounter_actions.is_some()
            || saved.session.exploration.as_ref().is_some_and(|position| {
                position
                    .script
                    .as_ref()
                    .is_some_and(|script| script.item_return)
            })
            || saved
                .session
                .medical_continuation
                .as_ref()
                .is_some_and(|pending| {
                    pending.origin == MedicalOrigin::OwnedNeedle && pending.item != 700
                })
            || saved.session.combat.as_ref().is_some_and(|c| {
                c.round_scripts.pending.is_some()
                    || c.pending_last_defeated.is_some()
                    || c.inactive_rounds != 0
                    || c.selection_pending
                    || c.script_enemy != 0
            })
            || saved.session.promotion.is_some()
            || matches!(
                saved.session.movement_continuation,
                Some(
                    MovementContinuation::Region { .. }
                        | MovementContinuation::RegionalReturn { .. }
                )
            ))
    {
        return Err(RuleError::Invalid(
            "pending runtime context requires save version 6".into(),
        ));
    }
    if saved.schema_version < 5
        && (saved.session.scenario_level != 0
            || !saved.session.visited_levels.is_empty()
            || saved.session.native_import.is_some()
            || !saved.session.shops.is_empty()
            || saved.session.shop_access.is_some()
            || saved.session.confiscated.is_some()
            || saved.session.suppress_allies != 0
            || saved.session.loot_limit.is_some()
            || !saved.session.typed_encounters.is_empty()
            || saved.session.notes.iter().any(|value| *value != 0))
    {
        return Err(RuleError::Invalid(
            "level state requires save version 5".into(),
        ));
    }
    if saved.schema_version < 4 && saved.session.difficulty != 0 {
        return Err(RuleError::Invalid(
            "non-Normal difficulty requires save version 4".into(),
        ));
    }
    if saved.schema_version < 3
        && let Some(position) = saved.session.exploration.as_mut()
    {
        if position.map != 0 || !saved.session.visited_maps.is_empty() {
            return Err(RuleError::Invalid(
                "legacy save has unreconstructable map history".into(),
            ));
        }
        let map = content
            .map(0)
            .ok_or_else(|| RuleError::Invalid("legacy map is unavailable".into()))?;
        position.visibility = vec![0; 8100];
        position.floor = map.floor.clone();
        position.objects = map.objects.clone();
        position.events = map.events.clone();
        if let Some(Presentation::Menu { id, .. }) = position.presentation.as_ref() {
            let script = position
                .script
                .as_mut()
                .ok_or_else(|| RuleError::Invalid("legacy menu script missing".into()))?;
            if *id != 0 || !script.scheduled {
                return Err(RuleError::Invalid(
                    "legacy menu history is unavailable".into(),
                ));
            }
            script.menu = Some(MenuContinuation {
                id: *id,
                remaining: i16::from(map.menus[101] as i8).wrapping_sub(1),
                original: script.record.clone(),
                typed: false,
            });
        }
    }
    if saved.session.psi_memory.is_none() {
        saved.session.psi_memory = Some(initial_psi_memory(&saved.session.warriors)?);
    }
    saved.session.validate(content)?;
    Ok(saved.session)
}
