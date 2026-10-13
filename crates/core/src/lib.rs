use centurions_content::{Content, ContentError, InventoryRecord, Warrior, word};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::Path;
use thiserror::Error;

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
    pub warriors: Vec<Warrior>,
    pub phase: Phase,
    pub mission: Option<String>,
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
    #[serde(default)]
    pub movement_continuation: Option<MovementContinuation>,
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
    #[serde(default)]
    pub loaded_item_field32: Option<i16>,
    #[serde(default)]
    pub ammunition_work: ammunition::WorkBuffers,
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
    #[serde(default)]
    pub psi_disabled: u8,
    #[serde(default)]
    pub enemy_spell_disabled: u8,
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
    Header,
    Secrets,
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
    #[serde(default)]
    pub menu: Option<MenuContinuation>,
    #[serde(default)]
    pub persist: bool,
    #[serde(default)]
    pub origin: Option<ScriptOrigin>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScriptOrigin {
    pub map: i32,
    pub x: i32,
    pub y: i32,
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
    Survivors,
    StageChange {
        parameter: i16,
    },
    WarriorChoice {
        extra: i16,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Command {
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
    Choose {
        answer: bool,
    },
    CancelMenu,
    SelectMenuEntry {
        index: usize,
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
    UnequipItem {
        warrior: usize,
        slot: usize,
    },
    EndTurn,
    Guard,
    RunEnemyTurn,
    AttackEnemy {
        body: i16,
    },
    ShootEnemy {
        body: i16,
    },
    CastPsi {
        spell: i16,
        power: i16,
        target: Option<(i16, i16)>,
    },
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

impl Session {
    pub(crate) fn load_item_work(&mut self, content: &Content, id: i16) -> Result<(), RuleError> {
        self.loaded_item_field32 = Some(self.ammunition_work.load_item(content, id)?);
        Ok(())
    }
    pub fn new(content: &Content) -> Self {
        Self {
            content_identity: content.identity.clone(),
            difficulty: 0,
            warriors: vec![],
            phase: Phase::SquadSelection,
            mission: None,
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
            loaded_item_field32: None,
            ammunition_work: ammunition::WorkBuffers::default(),
            loot_quality: None,
            attack_count: 0,
            attack_packet: Some(vec![0; 58]),
            attack_identity: Some([0; 3]),
            weapon_work: combat::weapon_work::WeaponWork::default(),
            psi_memory: Some(vec![0; 480]),
            party_buffs: empty_party_buffs(),
            psi_disabled: 0,
            enemy_spell_disabled: 0,
            health_context: None,
            party_currency: 0,
            library_random_seed: 1,
            library_random_draws: 0,
            combat_story: None,
            movement_continuation: None,
            medical_continuation: None,
            psi_menu_choices: Default::default(),
            bandage_ready: None,
            handled_deaths: vec![],
            menus: vec![],
            flags: initial_flags(),
            mission_score: 0,
            mission_stage: 0,
            map_picture: 0,
            visited_maps: std::collections::BTreeMap::new(),
            allies: empty_allies(),
            ally_count: 0,
        }
    }

    pub fn validate(&self, content: &Content) -> Result<(), RuleError> {
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
            || self.allies.len() != story_allies::ALLY_CAPACITY * story_allies::ALLY_BYTES
            || self.ally_count > story_allies::ALLY_CAPACITY
            || self.selected_warrior >= self.warriors.len().max(1)
            || self.party_buffs.len() != 20
            || self
                .psi_memory
                .as_ref()
                .is_some_and(|memory| memory.len() != 480)
            || self
                .attack_packet
                .as_ref()
                .is_some_and(|packet| packet.len() != 58)
            || (!self.menus.is_empty()
                && self.menus.len() != content.map(0).map_or(0, |map| map.menus.len()))
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
                    (pending.item, pending.spell, pending.power) == (700, 1401, 1)
                }
                MedicalOrigin::LearnedHeal => {
                    pending.item == 0 && pending.spell == 1101 && (1..=4).contains(&pending.power)
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
        if self.combat_story.as_ref().is_some_and(|story| {
            story.cursor > 8
                || !(0..110).contains(&story.actor)
                || (self.phase != Phase::Combat && !story.waiting_child)
                || content
                    .map(self.exploration.as_ref().map_or(0, |p| p.map))
                    .is_none_or(|map| map.script(story.script).is_err())
        }) || self.handled_deaths.iter().any(|i| {
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
                || self.mission.as_deref() != Some("The Thunder of Guns")
                || !(0..90).contains(&position.x)
                || !(0..90).contains(&position.y)
                || !(0..24).contains(&position.hour)
                || !(0..60).contains(&position.minute)
                || position.day < 1
                || !(4..=135).contains(&position.fatigue)
                || position.facing > 1
                || position.script.as_ref().is_some_and(|script| {
                    script.record.len() != 42
                        || script.cursor > 8
                        || script.map_slot >= 100
                        || script.menu.as_ref().is_some_and(|menu| {
                            menu.original.len() != 42
                                || menu.id < 0
                                || content.map(0).is_none_or(|map| {
                                    if menu.typed {
                                        (menu.id as usize + 1) * typed_encounter::RECORD_BYTES
                                            > map.typed_encounters.len()
                                    } else {
                                        (menu.id as usize + 1) * story::MENU_RECORD_BYTES
                                            > map.menus.len()
                                    }
                                })
                        })
                })
                || (position.presentation.is_some()
                    && position.script.is_none()
                    && !matches!(
                        position.presentation,
                        Some(
                            Presentation::Prologue { .. }
                                | Presentation::Messages { .. }
                                | Presentation::Survivors
                                | Presentation::Loot
                        )
                    ))
                || position.event_headers.len() != 4200
                || position.schedule.len() != content.map(0).map_or(0, |map| map.schedule.len())
                || !position.schedule.len().is_multiple_of(40)
                || position.region_state.len() != 644
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
        Ok(())
    }
}

pub fn apply(
    command: Command,
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    state.validate(content)?;
    let mut candidate = state.clone();
    let editing = matches!(
        command,
        Command::EditWarriorUpgrade { .. }
            | Command::EditCarriedItem { .. }
            | Command::EditWarriorLevel { .. }
            | Command::EditWarriorScans { .. }
    );
    let melee_input = matches!(command, Command::AttackEnemy { .. } | Command::Move { .. });
    let physical_work_input = matches!(
        command,
        Command::AttackEnemy { .. } | Command::Move { .. } | Command::RunEnemyTurn
    );
    let mut events = apply_inner(command, &mut candidate, content)?;
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
            .any(|event| matches!(event, Event::Message { .. }))
    {
        events.extend(exploration::resume_movement(&mut candidate, content)?);
    }
    if physical_work_input && let Some(item) = candidate.ammunition_work.item.as_ref() {
        candidate.loaded_item_field32 = Some(centurions_content::word(item, 32)?);
    }
    if let Some(combat) = candidate.combat.as_ref() {
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
    if state.medical_continuation.is_some()
        && !matches!(command, Command::SelectMedicalTarget { .. })
    {
        return Err(RuleError::Invalid(
            "finish choosing the medical target before acting".into(),
        ));
    }
    if state.phase == Phase::Combat
        && state
            .exploration
            .as_ref()
            .is_some_and(|p| p.presentation.is_some())
        && !matches!(command, Command::Continue)
    {
        return Err(RuleError::Invalid(
            "continue the combat story before acting".into(),
        ));
    }
    match command {
        Command::EditWarriorScans { warrior, controls } => {
            if state.phase != Phase::Exploration || controls.len() > 4096 {
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
            if state.psi_memory.is_none() {
                state.psi_memory = Some(initial_psi_memory(&state.warriors)?);
            }
            state.psi_memory.as_mut().unwrap()[warrior * 80..(warrior + 1) * 80]
                .copy_from_slice(&state.warriors[warrior].bytes[0xa76..0xac6]);
            let record = &state.warriors[warrior];
            if let Some(index) = record.bytes[0x9f0..0xa21]
                .iter()
                .rposition(|selected| *selected != 0)
            {
                let school = word(&record.bytes, 0x36)? as usize - 1;
                state.attack_packet = Some(
                    centurions_content::span(attacks, (school * 49 + index) * 58, 58)?.to_vec(),
                );
            }
            Ok(events)
        }
        Command::EditWarriorLevel { warrior } => {
            if state.phase != Phase::Exploration {
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
        Command::EditWarriorUpgrade { warrior, selector } => {
            if state.phase != Phase::Exploration || selector > 32 {
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
                || word(&record.bytes, 0xace + usize::from(selector) * 2)? < 0
            {
                return Err(RuleError::Invalid(
                    "this warrior cannot upgrade that attribute".into(),
                ));
            }
            let offset = 0xace + usize::from(selector) * 2;
            let count = word(&record.bytes, offset)?.wrapping_add(1);
            record.bytes[offset..offset + 2].copy_from_slice(&count.to_be_bytes());
            character::apply_upgrade(record, selector, 1)?;
            character::calculate_statistics(record)?;
            Ok(vec![
                Event::Sound { resource_id: 10129 },
                Event::StatisticsCalculated { warrior },
            ])
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
            if name != "The Thunder of Guns" {
                return Err(RuleError::Unsupported {
                    action: "mission.enter",
                    dependency: "only the supplied opening scenario is recovered",
                });
            }
            if state.phase != Phase::SquadSelection || !(1..=6).contains(&state.warriors.len()) {
                return Err(RuleError::Invalid(
                    "mission entry requires one to six selected warriors".into(),
                ));
            }
            let opening = content
                .map(0)
                .ok_or_else(|| RuleError::Invalid("opening content unavailable".into()))?;
            state.phase = Phase::Exploration;
            state.mission = Some(name);
            state.exploration = Some(Exploration {
                map: 0,
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
                combat::runtime::confirm_survivors(state, content)
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
        Command::Choose { answer } => exploration::choose(state, content, answer),
        Command::CancelMenu => exploration::cancel_menu(state, content),
        Command::SelectMenuEntry { index } => exploration::select_menu(state, content, index),
        Command::SubmitText { text } => exploration::submit_text(state, content, &text),
        Command::ChooseWarrior { index } => exploration::choose_warrior(state, content, index),
        Command::TakeLoot { slot, warrior } => {
            exploration::take_loot(state, content, slot, Some(warrior))
        }
        Command::StoreLoot { slot } => exploration::take_loot(state, content, slot, None),
        Command::TakeCrate { slot, warrior } => {
            equipment::take_crate(state, content, slot, warrior)
        }
        Command::EquipItem { warrior, slot } => equipment::equip(state, content, warrior, slot),
        Command::UnequipItem { warrior, slot } => equipment::unequip(state, content, warrior, slot),
        Command::Guard => combat::runtime::player_guard(state, content),
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
        Command::AttackEnemy { body } => combat::runtime::player_attack(state, content, body),
        Command::ShootEnemy { body } => combat::runtime::player_shoot(state, content, body),
        Command::CastPsi {
            spell,
            power,
            target,
        } => combat::runtime::player_psi(state, content, spell, power, target),
        Command::CastPartyPsi {
            warrior,
            spell,
            power,
        } => combat::runtime::party_psi(state, content, warrior, spell, power),
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
pub use exploration::needs_spell_confirmation;
pub mod items;
pub mod mission;
pub mod story;
pub mod story_allies;
pub mod story_collection;
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

pub fn save(path: &Path, state: &Session, content: &Content) -> Result<(), RuleError> {
    state.validate(content)?;
    let bytes = serde_json::to_vec_pretty(&Save {
        schema_version: if state.difficulty == 0 { 3 } else { 4 },
        session: state.clone(),
    })?;
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
    let written = file.write_all(&bytes).and_then(|()| file.sync_all());
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
    let map_count = content
        .scenario
        .as_ref()
        .map_or(0, |scenario| scenario.maps.len().min(20));
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
    } else if !matches!(saved.schema_version, 2..=4) {
        return Err(RuleError::Invalid(format!(
            "unsupported save version {}",
            saved.schema_version
        )));
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
