//! Shared candidate execution for mutually dependent application control.

use crate::recovered_character_upgrade::{
    CharacterUpgradeEffect, UpgradeSaneObservation, UpgradeSaneRequest,
};
use crate::recovered_combat_results::{CombatResultEffect, CombatResultServices};
use crate::recovered_equipment::{EquipmentEffect, EquipmentLimits};
use crate::recovered_feedback::{
    FeedbackArg, FeedbackEffect, FeedbackObservation, FeedbackRequest, FeedbackServices,
};
use crate::recovered_load_control::{LoadEffect, LoadFlow};
use crate::recovered_loot_control::{LootControlEffect, LootControlServices};
use crate::recovered_prompt_control::{
    LibraryObservation, LibraryRequest, PromptControlEffect, PromptControlState, PromptLimits,
    PromptRegisterRequest, PromptReturn,
};
use crate::recovered_records::RecordWrite;
use crate::recovered_rendering::RenderingMemory;
use crate::recovered_runtime::{self as runtime, RuntimeEffect, RuntimeResult};
use crate::recovered_session_control::{SessionControlEffect, SessionControlServices};
use crate::{RandomState, RuleError};

/// Memory/effect transport for source-lifted register and frame operations.
pub trait SourceMemory {
    fn source_memory(&self) -> &RenderingMemory;
    fn source_memory_mut(&mut self) -> &mut RenderingMemory;
    fn source_write(&mut self, write: RecordWrite);
}
impl SourceMemory for Execution<'_> {
    fn source_memory(&self) -> &RenderingMemory {
        &self.plan.state.memory
    }
    fn source_memory_mut(&mut self) -> &mut RenderingMemory {
        &mut self.plan.state.memory
    }
    fn source_write(&mut self, write: RecordWrite) {
        self.plan
            .effects
            .push(ApplicationControlEffect::Write(write));
    }
}

pub trait ApplicationControlServices:
    LootControlServices
    + SessionControlServices
    + CombatResultServices
    + crate::recovered_inventory_interaction::ItemsServices
    + crate::recovered_native_services::NativeServices
{
}
impl<
    T: LootControlServices
        + SessionControlServices
        + CombatResultServices
        + crate::recovered_inventory_interaction::ItemsServices
        + crate::recovered_native_services::NativeServices,
> ApplicationControlServices for T
{
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RegisterBits {
    pub value: u32,
    pub known_mask: u32,
}
impl RegisterBits {
    pub const fn full(value: u32) -> Self {
        Self {
            value,
            known_mask: u32::MAX,
        }
    }
    pub fn word(self, name: &str) -> std::result::Result<u16, RuleError> {
        if self.known_mask & 0xffff != 0xffff {
            return Err(RuleError::Invalid(format!(
                "missing reached application register {name}.W"
            )));
        }
        Ok(self.value as u16)
    }
    pub fn long(self, name: &str) -> std::result::Result<u32, RuleError> {
        if self.known_mask != u32::MAX {
            return Err(RuleError::Invalid(format!(
                "missing reached application register {name}.L"
            )));
        }
        Ok(self.value)
    }
    pub fn assign_word(&mut self, value: u16) {
        self.value = (self.value & 0xffff0000) | u32::from(value);
        self.known_mask |= 0xffff;
    }
    pub fn assign_byte(&mut self, value: u8) {
        self.value = (self.value & 0xffffff00) | u32::from(value);
        self.known_mask |= 0xff;
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ApplicationRegisters {
    pub data: [RegisterBits; 8],
    pub address: [RegisterBits; 8],
    pub ccr: u8,
    pub ccr_known_mask: u8,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ApplicationControlLimits {
    pub maximum_steps: u32,
    pub prompt: PromptLimits,
    pub equipment: EquipmentLimits,
    pub maximum_random_draws: u64,
    pub maximum_call_depth: u16,
}
pub type ApplicationControlState = PromptControlState;
#[derive(Clone, Debug, PartialEq)]
pub enum ApplicationControlEffect {
    Write(RecordWrite),
    Feedback(FeedbackEffect),
    Registers {
        request: PromptRegisterRequest,
        observation: FeedbackObservation,
    },
    Library {
        request: LibraryRequest,
        observation: LibraryObservation,
    },
    Sane {
        request: UpgradeSaneRequest,
        observation: UpgradeSaneObservation,
    },
    Runtime(RuntimeEffect),
    Prompt(PromptControlEffect),
    Load(LoadEffect),
    Loot(LootControlEffect),
    Session(SessionControlEffect),
    Upgrade(CharacterUpgradeEffect),
    Interaction(crate::recovered_inventory_interaction::InventoryInteractionEffect),
    Selection(crate::recovered_inventory_selection::InventorySelectionEffect),
    Survivor(crate::recovered_selection_control::SelectionControlEffect),
    Creation(crate::recovered_character_creation::CharacterCreationEffect),
    InventoryEdit(crate::recovered_inventory_edit::InventoryEditEffect),
    Field(crate::recovered_field_control::FieldControlEffect),
    CombatResult(CombatResultEffect),
    CombatPanel(crate::recovered_combat_panel::CombatPanelEffect),
    CombatUi(crate::recovered_combat_ui::CombatUiEffect),
    CombatCenter(crate::recovered_combat_center::CombatCenterEffect),
    CharacterPanel(crate::recovered_character_panels::CharacterPanelEffect),
    CharacterUi(crate::recovered_character_ui::CharacterUiEffect),
    Equipment(EquipmentEffect),
    EquipmentUi(crate::recovered_equipment_ui::EquipmentUiEffect),
    Audio(crate::recovered_audio::AudioEffect),
    File(crate::recovered_files::FileEffect),
    Interface(crate::recovered_interface::InterfaceEffect),
    Text(crate::recovered_text::TextEffect),
    Drawing(crate::recovered_drawing::DrawingEffect),
    Rendering(crate::recovered_rendering::RenderingEffect),
    Body(crate::recovered_bodies::BodyEffect),
    Timing(crate::recovered_timing::TimingEffect),
    SpellUi(crate::recovered_spell_ui::SpellUiEffect),
    ScenarioMenu(crate::recovered_scenario_menus::ScenarioMenuEffect),
    Startup(crate::recovered_startup::StartupEffect),
    Preference(crate::recovered_preferences::PreferenceEffect),
    Storage(crate::recovered_storage::StorageEffect),
    Shop(crate::recovered_shop_ui::ShopEffect),
    ItemUi(crate::recovered_item_ui::ItemUiEffect),
    Detail(crate::recovered_detail_ui::DetailEffect),
    NativeService {
        request: crate::recovered_native_services::NativeRequest,
        observation: Box<crate::recovered_native_services::NativeObservation>,
    },
    Movement(crate::recovered_navigation::MovementResult),
    Order(crate::recovered_battle_order::OrderEffect),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApplicationControlQualification {
    /// Native frame/register values are modeled; hidden LINK/MOVEM save storage
    /// is not projected into memory by the shared execution wrapper.
    SourcePortWithObservedBoundariesAndProjectedChildren,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ApplicationControlPlan {
    pub state: ApplicationControlState,
    pub effects: Vec<ApplicationControlEffect>,
    pub native_return: PromptReturn,
    pub native_d0: RegisterBits,
    /// Partial source projection; terminal flow is not a native CPU return receipt.
    pub native_registers: ApplicationRegisters,
    pub native_ccr: u8,
    pub native_ccr_known_mask: u8,
    pub flow: LoadFlow,
    pub qualification: ApplicationControlQualification,
}
#[derive(Debug)]
pub enum Signal {
    Rule(RuleError),
    Stopped,
}
impl From<RuleError> for Signal {
    fn from(error: RuleError) -> Self {
        Self::Rule(error)
    }
}
pub type Result<T> = std::result::Result<T, Signal>;
pub struct Execution<'a> {
    pub plan: ApplicationControlPlan,
    pub services: &'a mut dyn ApplicationControlServices,
    pub resolver: &'a dyn RoutineResolver,
    pub limits: ApplicationControlLimits,
    pub remaining: u32,
    pub draws: u64,
    pub registers: ApplicationRegisters,
    pub stack: u32,
    pub depth: u16,
}
impl Execution<'_> {
    pub fn step(&mut self) -> Result<()> {
        self.remaining = self
            .remaining
            .checked_sub(1)
            .ok_or_else(|| RuleError::Invalid("application processing budget exhausted".into()))?;
        Ok(())
    }
    pub fn b(&self, address: u32) -> Result<u8> {
        Ok(self.plan.state.memory.byte(address)?)
    }
    pub fn sb(&self, address: u32) -> Result<i8> {
        Ok(self.b(address)? as i8)
    }
    pub fn w(&self, address: u32) -> Result<u16> {
        Ok(self.plan.state.memory.word(address)?)
    }
    pub fn sw(&self, address: u32) -> Result<i16> {
        Ok(self.w(address)? as i16)
    }
    pub fn l(&self, address: u32) -> Result<u32> {
        Ok(self.plan.state.memory.long(address)?)
    }
    pub fn sl(&self, address: u32) -> Result<i32> {
        Ok(self.l(address)? as i32)
    }
    pub fn bytes(&self, address: u32, count: usize) -> Result<Vec<u8>> {
        (0..count)
            .map(|n| self.b(address.wrapping_add(n as u32)))
            .collect()
    }
    pub fn write(&mut self, address: u32, bytes: &[u8]) -> Result<()> {
        self.bytes(address, bytes.len())?;
        self.plan.state.memory.insert_region(address, bytes);
        self.plan
            .effects
            .push(ApplicationControlEffect::Write(RecordWrite {
                address,
                bytes: bytes.to_vec(),
            }));
        Ok(())
    }
    pub fn byte(&mut self, address: u32, value: u8) -> Result<()> {
        self.write(address, &[value])
    }
    pub fn word(&mut self, address: u32, value: u16) -> Result<()> {
        self.write(address, &value.to_be_bytes())
    }
    pub fn long(&mut self, address: u32, value: u32) -> Result<()> {
        self.write(address, &value.to_be_bytes())
    }
    pub fn addw(&mut self, address: u32, value: i16) -> Result<()> {
        self.word(address, self.w(address)?.wrapping_add(value as u16))
    }
    pub fn addl(&mut self, address: u32, value: u32) -> Result<()> {
        self.long(address, self.l(address)?.wrapping_add(value))
    }
    pub fn copy(&mut self, source: u32, destination: u32, length: usize) -> Result<()> {
        let mut offset = 0;
        while offset + 4 <= length {
            self.step()?;
            self.long(
                destination.wrapping_add(offset as u32),
                self.l(source.wrapping_add(offset as u32))?,
            )?;
            offset += 4;
        }
        if offset + 2 <= length {
            self.word(
                destination.wrapping_add(offset as u32),
                self.w(source.wrapping_add(offset as u32))?,
            )?;
            offset += 2;
        }
        if offset < length {
            self.byte(
                destination.wrapping_add(offset as u32),
                self.b(source.wrapping_add(offset as u32))?,
            )?;
        }
        Ok(())
    }
    pub fn full(&mut self, value: u32) {
        self.registers.data[0] = RegisterBits::full(value);
        self.plan.native_return = PromptReturn::Full(value);
    }
    pub fn low(&mut self, value: u16) {
        self.registers.data[0].assign_word(value);
        self.plan.native_return = if self.registers.data[0].known_mask == u32::MAX {
            PromptReturn::Full(self.registers.data[0].value)
        } else {
            PromptReturn::LowWord(value)
        };
    }
    pub fn unknown(&mut self) {
        self.registers.data[0] = RegisterBits::default();
        self.plan.native_return = PromptReturn::Unchanged;
        self.registers.ccr_known_mask = 0;
    }
    pub fn returned(&mut self, value: PromptReturn) {
        match value {
            PromptReturn::Full(v) => self.full(v),
            PromptReturn::LowWord(v) => {
                self.registers.data[0] = RegisterBits {
                    value: u32::from(v),
                    known_mask: 0xffff,
                };
                self.plan.native_return = PromptReturn::LowWord(v);
            }
            PromptReturn::Unchanged => self.unknown(),
            PromptReturn::NoReturn => self.plan.native_return = PromptReturn::NoReturn,
        }
        self.registers.ccr_known_mask = 0;
    }
    pub fn moveq_return(&mut self, value: i8) {
        self.full(i32::from(value) as u32);
        self.registers.ccr = (self.registers.ccr & 0x10)
            | if value == 0 {
                4
            } else if value < 0 {
                8
            } else {
                0
            };
        self.registers.ccr_known_mask |= 0xf;
    }
    pub fn d0_word(&self) -> Result<u16> {
        Ok(self.registers.data[0].word("D0")?)
    }
    pub fn d0_long(&self) -> Result<u32> {
        Ok(self.registers.data[0].long("D0")?)
    }
    pub fn active(&self) -> bool {
        self.plan.flow == LoadFlow::Returned
    }
    pub fn prompt_limits(&self) -> PromptLimits {
        PromptLimits {
            maximum_steps: self.remaining.min(self.limits.prompt.maximum_steps),
            ..self.limits.prompt
        }
    }
    pub fn trap(&mut self, opcode: u16, arguments: Vec<FeedbackArg>) -> Result<u32> {
        self.step()?;
        let request = FeedbackRequest::Toolbox { opcode, arguments };
        let observation =
            FeedbackServices::observe(self.services, &request, &self.plan.state.memory)?;
        let value = observation.value;
        self.full(observation.d0);
        self.registers.ccr_known_mask = 0;
        self.plan.state.memory = observation.memory.clone();
        self.plan.effects.push(ApplicationControlEffect::Feedback(
            FeedbackEffect::Service {
                request,
                observation,
            },
        ));
        Ok(value)
    }
    pub fn register_trap(
        &mut self,
        opcode: u16,
        d0_input: Option<u32>,
        a0_input: Option<u32>,
        arguments: Vec<FeedbackArg>,
    ) -> Result<u32> {
        self.step()?;
        let request = PromptRegisterRequest {
            opcode,
            d0_input,
            a0_input,
            arguments,
        };
        let observation = self
            .services
            .observe_registers(&request, &self.plan.state.memory)?;
        let value = observation.value;
        self.full(observation.d0);
        self.registers.ccr_known_mask = 0;
        self.plan.state.memory = observation.memory.clone();
        self.plan.effects.push(ApplicationControlEffect::Registers {
            request,
            observation,
        });
        Ok(value)
    }
    pub fn lib(&mut self, entry: u32, arguments: Vec<FeedbackArg>) -> Result<LibraryObservation> {
        self.step()?;
        let request = LibraryRequest { entry, arguments };
        let observation = self.services.library(&request, &self.plan.state.memory)?;
        self.full(observation.d0);
        self.registers.ccr_known_mask = 0;
        self.plan.state.memory = observation.memory.clone();
        self.plan.effects.push(ApplicationControlEffect::Library {
            request,
            observation: observation.clone(),
        });
        Ok(observation)
    }
    pub fn sane(&mut self, request: UpgradeSaneRequest) -> Result<u8> {
        self.step()?;
        let observation = self
            .services
            .observe_sane(&request, &self.plan.state.memory)?;
        self.full(observation.d0);
        self.registers.ccr = observation.ccr;
        self.registers.ccr_known_mask = 0x1f;
        self.plan.state.memory = observation.memory.clone();
        let ccr = observation.ccr;
        self.plan.effects.push(ApplicationControlEffect::Sane {
            request,
            observation,
        });
        Ok(ccr)
    }
    pub fn rand(&mut self, bound: i16, stack: u32) -> Result<i16> {
        self.step()?;
        self.draws = self
            .draws
            .checked_sub(1)
            .ok_or_else(|| RuleError::Invalid("application random budget exhausted".into()))?;
        self.word(stack.wrapping_sub(2), bound as u16)?;
        let mut context =
            runtime::RuntimeContext::new(self.plan.state.random.clone(), self.remaining as usize);
        let child = runtime::rand(&mut context, bound)?;
        self.plan.state.random = context.random;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(ApplicationControlEffect::Runtime),
        );
        match child.result {
            RuntimeResult::Word { value, native_d0 } => {
                self.full(native_d0);
                self.registers.ccr_known_mask = 0;
                Ok(value)
            }
            _ => Err(RuleError::Invalid("unexpected application Rand return".into()).into()),
        }
    }
    pub fn range(&mut self, min: i16, max: i16, stack: u32) -> Result<i16> {
        self.word(stack.wrapping_sub(2), max as u16)?;
        self.word(stack.wrapping_sub(4), min as u16)?;
        let value = min
            .wrapping_add(self.rand(
                max.wrapping_sub(min).wrapping_add(1),
                stack.wrapping_sub(20),
            )?)
            .wrapping_sub(1);
        self.low(value as u16);
        Ok(value)
    }
    pub fn adopt(
        &mut self,
        memory: RenderingMemory,
        random: RandomState,
        native_return: PromptReturn,
        flow: LoadFlow,
        effects: impl IntoIterator<Item = ApplicationControlEffect>,
    ) -> Result<()> {
        let used = random
            .draws
            .checked_sub(self.plan.state.random.draws)
            .ok_or_else(|| {
                RuleError::Invalid("application child reset its random draw counter".into())
            })?;
        self.draws = self
            .draws
            .checked_sub(used)
            .ok_or_else(|| RuleError::Invalid("application random budget exhausted".into()))?;
        self.plan.state = PromptControlState { memory, random };
        self.returned(native_return);
        self.plan.effects.extend(effects);
        self.plan.flow = flow;
        if flow != LoadFlow::Returned {
            self.plan.native_return = PromptReturn::NoReturn;
            return Err(Signal::Stopped);
        }
        Ok(())
    }
    pub fn call(&mut self, entry: u32, frame: u32) -> Result<()> {
        self.step()?;
        if self.depth >= self.limits.maximum_call_depth {
            return Err(RuleError::Invalid("application call depth exhausted".into()).into());
        }
        let stack = self.stack;
        let saved = self.registers;
        let (data_mask, address_mask) = preserved_register_masks(entry);
        let framed = has_native_frame(entry);
        if framed && !self.resolver.owns_native_stack(entry) {
            self.registers.address[6] = RegisterBits::full(frame);
        }
        self.depth += 1;
        let resolver = self.resolver;
        let result = resolver.dispatch(self, entry, frame);
        self.depth -= 1;
        if result.is_ok() {
            self.stack = stack;
            if !self.resolver.owns_native_stack(entry) {
                // Items projects its own symbolic LINK/MOVEM restore, including
                // reached save-slot aliases. Other children expose typed returns.
                for index in 0..8 {
                    if data_mask & (1 << index) != 0 {
                        self.registers.data[index] = saved.data[index];
                    } else if index == 1 && entry == 0x130afe {
                        // Complete helper/abs_short source leaves D1 upper bits untouched.
                        self.registers.data[1] = RegisterBits {
                            value: saved.data[1].value & 0xffff0000,
                            known_mask: saved.data[1].known_mask & 0xffff0000,
                        };
                    } else if index != 0 {
                        self.registers.data[index] = RegisterBits::default();
                    }
                    if address_mask & (1 << index) != 0 {
                        self.registers.address[index] = saved.address[index];
                    } else {
                        self.registers.address[index] = RegisterBits::default();
                    }
                }
                if framed {
                    self.registers.address[6] = saved.address[6];
                }
                // Projected bodies expose D0 and source-preserved registers. Other
                // working registers and epilogue flags are not observed returns.
                self.registers.ccr_known_mask = 0;
            }
            let d0 = self.registers.data[0];
            self.plan.native_return = if d0.known_mask == u32::MAX {
                PromptReturn::Full(d0.value)
            } else if d0.known_mask & 0xffff == 0xffff {
                PromptReturn::LowWord(d0.value as u16)
            } else {
                PromptReturn::Unchanged
            };
        }
        result
    }
}

pub fn execute(
    state: &ApplicationControlState,
    entry: u32,
    frame: u32,
    registers: ApplicationRegisters,
    limits: ApplicationControlLimits,
    services: &mut dyn ApplicationControlServices,
    resolver: &dyn RoutineResolver,
) -> std::result::Result<ApplicationControlPlan, RuleError> {
    let mut execution = Execution {
        plan: ApplicationControlPlan { state: state.clone(), effects: vec![], native_return: PromptReturn::Unchanged, native_d0: registers.data[0],
            native_registers: registers, native_ccr: registers.ccr, native_ccr_known_mask: registers.ccr_known_mask, flow: LoadFlow::Returned, qualification: ApplicationControlQualification::SourcePortWithObservedBoundariesAndProjectedChildren },
        services, resolver, limits, remaining: limits.maximum_steps, draws: limits.maximum_random_draws, registers, stack: frame, depth: 0,
    };
    match execution.call(entry, frame) {
        Ok(()) => {}
        Err(Signal::Stopped) if !execution.active() => {}
        Err(Signal::Stopped) => {
            return Err(RuleError::Invalid(
                "application stopped without terminal flow".into(),
            ));
        }
        Err(Signal::Rule(error)) => return Err(error),
    }
    execution.plan.native_ccr = execution.registers.ccr;
    execution.plan.native_d0 = execution.registers.data[0];
    execution.plan.native_registers = execution.registers;
    execution.plan.native_ccr_known_mask = execution.registers.ccr_known_mask;
    Ok(execution.plan)
}

fn has_native_frame(entry: u32) -> bool {
    (0x100000..0x200000).contains(&entry)
        && !matches!(
            entry,
            0x100212 | 0x100286 | 0x1002b2 | 0x1002ec | 0x1004ae | 0x1004d2 | 0x100504 | 0x100584
        )
}

fn preserved_register_masks(entry: u32) -> (u8, u8) {
    match entry {
        // Complete native helpers plus the resolved eight-instruction abs_short.
        0x130afe | 0x149e0e | 0x105366 => (0xfc, 0x7c),
        // Provisional SANE/compiler preservation ceiling; D2 remains unknown.
        // This numeric child carries SourceDerivedBinary32 qualification.
        0x136994 => (0xf8, 0x7c),
        // Existing Rand arithmetic plus the inferred QuickDraw/import ceiling.
        0x153ca4 | 0x13849e => (0xf8, 0x7c),
        _ => saved_register_masks(entry),
    }
}

fn saved_register_masks(entry: u32) -> (u8, u8) {
    match entry {
        0x100286 | 0x1002b2 | 0x1004ae | 0x123c60 | 0x127462 | 0x13006e | 0x13107a | 0x13164e
        | 0x133bb8 | 0x133c56 | 0x13777a | 0x1383e6 | 0x13bd52 | 0x148f9e | 0x14ee9e | 0x151e24
        | 0x152dc4 | 0x1538ba | 0x15f210 => (0x0, 0x4),
        0x121596 | 0x124576 | 0x1245cc | 0x127132 => (0x0, 0xc),
        0x12427c => (0x0, 0x1c),
        0x1002ec => (0x0, 0x21),
        0x100a38 | 0x100a9a | 0x1046e0 | 0x10518a | 0x105366 | 0x109210 | 0x10d3f6 | 0x1107cc
        | 0x11084a | 0x11f1a8 | 0x12006e | 0x121524 | 0x122dc4 | 0x124bba | 0x12946c | 0x13145c
        | 0x133886 | 0x133a18 | 0x133f3e | 0x134ae0 | 0x134d9e | 0x136994 | 0x138648 | 0x138efe
        | 0x13a1dc | 0x13a2c4 | 0x13a87c | 0x13ea82 | 0x14a074 | 0x14c318 | 0x152ec2 | 0x15498e
        | 0x156b26 | 0x159192 | 0x15947c | 0x15a49a | 0x15ef50 | 0x15f4b8 | 0x160a8e | 0x160fb2
        | 0x161b80 | 0x162238 | 0x1636ec => (0x8, 0x0),
        0x113988 | 0x122b18 | 0x127264 | 0x1279bc | 0x135fc8 | 0x1366f8 | 0x13b212 | 0x14913e
        | 0x14a29e | 0x14b63a | 0x15741a | 0x157d38 | 0x162084 | 0x16474a => (0x8, 0x4),
        0x14a918 | 0x14aa22 => (0x8, 0xc),
        0x124c18 => (0x8, 0x1c),
        0x1004d2 | 0x100504 | 0x100584 => (0xc, 0x0),
        0x10ae90 | 0x13b0ea => (0x10, 0x0),
        0x10b6ba | 0x110644 | 0x111984 | 0x1205f8 | 0x130afe | 0x130c18 | 0x130d38 | 0x130e54
        | 0x130f6e | 0x135354 | 0x13849e | 0x139192 | 0x13aeb8 | 0x13b56c | 0x13beea | 0x13f52c
        | 0x14ce2a | 0x152188 | 0x153ca4 | 0x153cf4 | 0x1540c4 | 0x157c68 | 0x157ea4 | 0x15fa20
        | 0x1600fc | 0x1634f8 => (0x18, 0x0),
        0x100642 | 0x117a54 | 0x1374bc | 0x139244 | 0x13b30c | 0x14cff6 | 0x1551a2 => (0x18, 0x4),
        0x122e24 | 0x131182 | 0x13ef12 | 0x14d7cc | 0x152378 => (0x18, 0xc),
        0x14f96e => (0x18, 0x1c),
        0x100212 => (0x20, 0x0),
        0x10f1c4 | 0x10fc26 | 0x11177e | 0x11235e | 0x1124dc | 0x12063c | 0x1212a8 | 0x122bec
        | 0x123b56 | 0x12dc58 | 0x1378a2 | 0x1388aa | 0x13a5a6 | 0x13a72a | 0x13add4 | 0x13b624
        | 0x149e0e | 0x14c622 | 0x14c6c2 | 0x14ef76 | 0x15311a | 0x1531c0 | 0x15d2f0 | 0x160d08 => {
            (0x38, 0x0)
        }
        0x1270c0 | 0x12fe54 | 0x14c440 => (0x38, 0x4),
        0x1047f8 | 0x1247cc | 0x12485e | 0x14bca4 => (0x38, 0xc),
        0x14c01c => (0x38, 0x1c),
        0x13846a | 0x146a50 => (0x60, 0x0),
        0x11279a => (0x70, 0x0),
        0x112820 | 0x1140ae | 0x120872 | 0x121152 | 0x125a3a | 0x12c46e | 0x133d46 | 0x136f90
        | 0x13a3ae | 0x150114 | 0x1553bc | 0x157586 | 0x15f66c => (0x78, 0x0),
        0x112208 | 0x1174f0 | 0x12382a | 0x12ed42 | 0x134034 | 0x151022 => (0x78, 0x4),
        0x134e82 | 0x14d596 => (0x78, 0x1c),
        0x1380ac => (0xe0, 0x0),
        0x10b13a | 0x10b3fa => (0xf0, 0xc),
        0x124d04 => (0xf0, 0x10),
        0x101eb6 | 0x105572 | 0x10950e | 0x10d4fe | 0x10e6fe | 0x10ec52 | 0x110d12 | 0x110e96
        | 0x112650 | 0x11ce8e | 0x11edc4 | 0x123d1c | 0x124d9e | 0x125538 | 0x125960 | 0x1287c2
        | 0x12c874 | 0x12d558 | 0x12e4e4 | 0x1363e4 | 0x13c1ee | 0x13e276 | 0x13f684 | 0x146aa2
        | 0x14a3e6 | 0x14de04 | 0x14e5e4 | 0x15075a | 0x153584 | 0x153e44 | 0x157758 | 0x157ef8
        | 0x1580be | 0x158454 | 0x159a96 | 0x15b2f6 | 0x15b84c | 0x15b9d8 | 0x15cb0e | 0x15d210
        | 0x15edfa | 0x161068 | 0x164a0e => (0xf8, 0x0),
        0x100af0 | 0x103ebe | 0x10b89e | 0x114378 | 0x11d406 | 0x120c14 | 0x12512a | 0x1289fa
        | 0x1300ec | 0x132b0c | 0x135bb2 | 0x1476cc | 0x14aba0 | 0x154a46 | 0x15e148 | 0x16399c => {
            (0xf8, 0x4)
        }
        0x117be2 | 0x12185a | 0x14d21a | 0x1555f2 | 0x15a9cc | 0x15dc34 => (0xf8, 0xc),
        0x105748 | 0x1213f6 | 0x124308 | 0x1246a8 | 0x1248f0 | 0x124a44 | 0x129516 | 0x12f0cc
        | 0x13ebde | 0x1400e4 | 0x146b6a => (0xf8, 0x1c),
        _ => (0, 0),
    }
}

/// Explicit routine set used by source execution. Test sets remain independently scoped.
pub trait RoutineResolver {
    fn dispatch(&self, execution: &mut Execution<'_>, entry: u32, frame: u32) -> Result<()>;
    fn owns_native_stack(&self, entry: u32) -> bool;
}

#[macro_export]
macro_rules! translated_entry {
    ($name:ident, $address:expr) => {
        #[doc = concat!("CODE 1 entry `", stringify!($address), "` through shared candidate execution.")]
        pub fn $name(
            state: &$crate::recovered_application_control::ApplicationControlState,
            frame: u32,
            registers: $crate::recovered_application_control::ApplicationRegisters,
            limits: $crate::recovered_application_control::ApplicationControlLimits,
            services: &mut dyn $crate::recovered_application_control::ApplicationControlServices,
            resolver: &dyn $crate::recovered_application_control::RoutineResolver,
        ) -> std::result::Result<$crate::recovered_application_control::ApplicationControlPlan, $crate::RuleError> {
            $crate::recovered_application_control::execute(state, $address, frame, registers, limits, services, resolver)
        }
    };
}
pub use crate::translated_entry as entry;
