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

mod children;
mod gameplay_children;
mod spell_children;

/// Memory/effect transport for source-lifted register and frame operations.
pub(crate) trait SourceMemory {
    fn source_memory(&self) -> &RenderingMemory;
    fn source_memory_mut(&mut self) -> &mut RenderingMemory;
    fn source_write(&mut self, write: RecordWrite);
}
impl<S> SourceMemory for Execution<'_, S> {
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
pub(crate) enum Signal {
    Rule(RuleError),
    Stopped,
}
impl From<RuleError> for Signal {
    fn from(error: RuleError) -> Self {
        Self::Rule(error)
    }
}
pub(crate) type Result<T> = std::result::Result<T, Signal>;
pub(crate) struct Execution<'a, S> {
    pub(crate) plan: ApplicationControlPlan,
    pub(crate) services: &'a mut S,
    pub(crate) limits: ApplicationControlLimits,
    pub(crate) remaining: u32,
    pub(crate) draws: u64,
    pub(crate) registers: ApplicationRegisters,
    pub(crate) stack: u32,
    depth: u16,
}
impl<S: ApplicationControlServices> Execution<'_, S> {
    pub(crate) fn step(&mut self) -> Result<()> {
        self.remaining = self
            .remaining
            .checked_sub(1)
            .ok_or_else(|| RuleError::Invalid("application processing budget exhausted".into()))?;
        Ok(())
    }
    pub(crate) fn b(&self, address: u32) -> Result<u8> {
        Ok(self.plan.state.memory.byte(address)?)
    }
    pub(crate) fn sb(&self, address: u32) -> Result<i8> {
        Ok(self.b(address)? as i8)
    }
    pub(crate) fn w(&self, address: u32) -> Result<u16> {
        Ok(self.plan.state.memory.word(address)?)
    }
    pub(crate) fn sw(&self, address: u32) -> Result<i16> {
        Ok(self.w(address)? as i16)
    }
    pub(crate) fn l(&self, address: u32) -> Result<u32> {
        Ok(self.plan.state.memory.long(address)?)
    }
    pub(crate) fn sl(&self, address: u32) -> Result<i32> {
        Ok(self.l(address)? as i32)
    }
    pub(crate) fn bytes(&self, address: u32, count: usize) -> Result<Vec<u8>> {
        (0..count)
            .map(|n| self.b(address.wrapping_add(n as u32)))
            .collect()
    }
    pub(crate) fn write(&mut self, address: u32, bytes: &[u8]) -> Result<()> {
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
    pub(crate) fn byte(&mut self, address: u32, value: u8) -> Result<()> {
        self.write(address, &[value])
    }
    pub(crate) fn word(&mut self, address: u32, value: u16) -> Result<()> {
        self.write(address, &value.to_be_bytes())
    }
    pub(crate) fn long(&mut self, address: u32, value: u32) -> Result<()> {
        self.write(address, &value.to_be_bytes())
    }
    pub(crate) fn addw(&mut self, address: u32, value: i16) -> Result<()> {
        self.word(address, self.w(address)?.wrapping_add(value as u16))
    }
    pub(crate) fn addl(&mut self, address: u32, value: u32) -> Result<()> {
        self.long(address, self.l(address)?.wrapping_add(value))
    }
    pub(crate) fn copy(&mut self, source: u32, destination: u32, length: usize) -> Result<()> {
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
    pub(crate) fn full(&mut self, value: u32) {
        self.registers.data[0] = RegisterBits::full(value);
        self.plan.native_return = PromptReturn::Full(value);
    }
    pub(crate) fn low(&mut self, value: u16) {
        self.registers.data[0].assign_word(value);
        self.plan.native_return = if self.registers.data[0].known_mask == u32::MAX {
            PromptReturn::Full(self.registers.data[0].value)
        } else {
            PromptReturn::LowWord(value)
        };
    }
    pub(crate) fn unknown(&mut self) {
        self.registers.data[0] = RegisterBits::default();
        self.plan.native_return = PromptReturn::Unchanged;
        self.registers.ccr_known_mask = 0;
    }
    pub(crate) fn returned(&mut self, value: PromptReturn) {
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
    pub(crate) fn moveq_return(&mut self, value: i8) {
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
    pub(crate) fn d0_word(&self) -> Result<u16> {
        Ok(self.registers.data[0].word("D0")?)
    }
    pub(crate) fn d0_long(&self) -> Result<u32> {
        Ok(self.registers.data[0].long("D0")?)
    }
    pub(crate) fn active(&self) -> bool {
        self.plan.flow == LoadFlow::Returned
    }
    pub(crate) fn prompt_limits(&self) -> PromptLimits {
        PromptLimits {
            maximum_steps: self.remaining.min(self.limits.prompt.maximum_steps),
            ..self.limits.prompt
        }
    }
    pub(crate) fn trap(&mut self, opcode: u16, arguments: Vec<FeedbackArg>) -> Result<u32> {
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
    pub(crate) fn register_trap(
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
    pub(crate) fn lib(
        &mut self,
        entry: u32,
        arguments: Vec<FeedbackArg>,
    ) -> Result<LibraryObservation> {
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
    pub(crate) fn sane(&mut self, request: UpgradeSaneRequest) -> Result<u8> {
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
    pub(crate) fn rand(&mut self, bound: i16, stack: u32) -> Result<i16> {
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
    pub(crate) fn range(&mut self, min: i16, max: i16, stack: u32) -> Result<i16> {
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
    pub(crate) fn adopt(
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
    pub(crate) fn call(&mut self, entry: u32, frame: u32) -> Result<()> {
        self.step()?;
        if self.depth >= self.limits.maximum_call_depth {
            return Err(RuleError::Invalid("application call depth exhausted".into()).into());
        }
        let stack = self.stack;
        let saved = self.registers;
        let (data_mask, address_mask) = preserved_register_masks(entry);
        let framed = has_native_frame(entry);
        if framed && !owns_native_stack(entry) {
            self.registers.address[6] = RegisterBits::full(frame);
        }
        self.depth += 1;
        let result = dispatch(self, entry, frame);
        self.depth -= 1;
        if result.is_ok() {
            self.stack = stack;
            if !owns_native_stack(entry) {
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

macro_rules! entry {
    ($name:ident, $address:expr) => {
        #[doc = concat!("CODE 1 entry `", stringify!($address), "` through shared candidate execution.")]
        pub fn $name(
            state: &crate::recovered_application_control::ApplicationControlState,
            frame: u32,
            registers: crate::recovered_application_control::ApplicationRegisters,
            limits: crate::recovered_application_control::ApplicationControlLimits,
            services: &mut impl crate::recovered_application_control::ApplicationControlServices,
        ) -> std::result::Result<crate::recovered_application_control::ApplicationControlPlan, crate::RuleError> {
            crate::recovered_application_control::execute(state, $address, frame, registers, limits, services)
        }
    };
}
pub(crate) use entry;

pub fn execute(
    state: &ApplicationControlState,
    entry: u32,
    frame: u32,
    registers: ApplicationRegisters,
    limits: ApplicationControlLimits,
    services: &mut impl ApplicationControlServices,
) -> std::result::Result<ApplicationControlPlan, RuleError> {
    let mut execution = Execution {
        plan: ApplicationControlPlan { state: state.clone(), effects: vec![], native_return: PromptReturn::Unchanged, native_d0: registers.data[0],
            native_registers: registers, native_ccr: registers.ccr, native_ccr_known_mask: registers.ccr_known_mask, flow: LoadFlow::Returned, qualification: ApplicationControlQualification::SourcePortWithObservedBoundariesAndProjectedChildren },
        services, limits, remaining: limits.maximum_steps, draws: limits.maximum_random_draws, registers, stack: frame, depth: 0,
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
fn dispatch<S: ApplicationControlServices>(
    e: &mut Execution<'_, S>,
    entry: u32,
    frame: u32,
) -> Result<()> {
    match entry {
        0x100212 => crate::recovered_remaining_control::relocate_body(e, frame),
        0x100286 => crate::recovered_remaining_control::relocate_three_body(e, frame),
        0x1002b2 => crate::recovered_remaining_control::unrelocate_three_body(e, frame),
        0x1002ec => crate::recovered_remaining_control::runtime_transfer_body(e, frame),
        0x1004ae => crate::recovered_remaining_control::run_destructors_body(e, frame),
        0x1004d2 => crate::recovered_remaining_control::multiply_long_body(e, frame),
        0x100504 => crate::recovered_remaining_control::divide_unsigned_long_body(e, frame),
        0x100584 => crate::recovered_remaining_control::remainder_unsigned_long_body(e, frame),
        0x100a9a => crate::recovered_remaining_control::addtobooty_body(e, frame),
        0x10518a => crate::recovered_remaining_control::monsterupdate_body(e, frame),
        0x1056d4 => crate::recovered_remaining_control::updatebootymoney_body(e, frame),
        0x10b13a => crate::recovered_remaining_control::cansee2_body(e, frame),
        0x10b3fa => crate::recovered_remaining_control::cansee3_body(e, frame),
        0x10b814 => crate::recovered_remaining_control::updatetier_body(e, frame),
        0x10d3f6 => crate::recovered_remaining_control::calcbodyrect_body(e, frame),
        0x10f1c4 => crate::recovered_remaining_control::centerpictlarge_body(e, frame),
        0x10fc26 => crate::recovered_remaining_control::centerpictsmall_body(e, frame),
        0x11084a => crate::recovered_remaining_control::change_body(e, frame),
        0x112208 => crate::recovered_remaining_control::checklayout_body(e, frame),
        0x113988 => crate::recovered_remaining_control::caste_body(e, frame),
        0x120872 => crate::recovered_remaining_control::display_body(e, frame),
        0x123b56 => crate::recovered_remaining_control::flashrange_body(e, frame),
        0x123f96 => crate::recovered_remaining_control::flashrect_body(e, frame),
        0x125512 => crate::recovered_remaining_control::spell_count_body(e, frame),
        0x127264 => crate::recovered_remaining_control::gender_body(e, frame),
        0x12c874 => crate::recovered_remaining_control::installitem_complete_body(e, frame),
        0x12d558 => crate::recovered_remaining_control::randomitem_body(e, frame),
        0x12dc58 => crate::recovered_remaining_control::show_pack_body(e, frame),
        0x12ed42 => crate::recovered_remaining_control::level_up_body(e, frame),
        0x130c18 => crate::recovered_remaining_control::loaditemcompare_body(e, frame),
        0x130f6e => crate::recovered_remaining_control::loaditemnotype_body(e, frame),
        0x133886 => crate::recovered_remaining_control::update_shop_wings_body(e, frame),
        0x134ae0 => crate::recovered_remaining_control::update_edit_item_body(e, frame),
        0x13684a => crate::recovered_remaining_control::share_credits_body(e, frame),
        0x136f90 => crate::recovered_remaining_control::calcdamage_body(e, frame),
        0x1380ac => crate::recovered_remaining_control::getexp_body(e, frame),
        0x1382fc => crate::recovered_remaining_control::string_long_body(e, frame),
        0x13832a => crate::recovered_remaining_control::string_plus_body(e, frame),
        0x138368 => crate::recovered_remaining_control::string_no_zero_no_plus_body(e, frame),
        0x1383a6 => crate::recovered_remaining_control::string_no_zero_body(e, frame),
        0x1384d6 => crate::recovered_remaining_control::update_preparation_body(e, frame),
        0x138648 => crate::recovered_remaining_control::strength_adjustments_body(e, frame),
        0x1388aa => crate::recovered_remaining_control::agility_adjustment_body(e, frame),
        0x1389e4 => crate::recovered_remaining_control::get_dialog_number_body(e, frame),
        0x139244 => crate::recovered_remaining_control::update_shop_body(e, frame),
        0x139faa => crate::recovered_remaining_control::shop_credit_display_body(e, frame),
        0x13a068 => crate::recovered_remaining_control::eraseshopstats_body(e, frame),
        0x13a10a => crate::recovered_remaining_control::eraseshopname_body(e, frame),
        0x13a72a => crate::recovered_remaining_control::dialog_number_nonzero_body(e, frame),
        0x13aeb8 => crate::recovered_remaining_control::setverify_body(e, frame),
        0x13b0ea => crate::recovered_remaining_control::checkverify_body(e, frame),
        0x13b30c => crate::recovered_remaining_control::start_level_body(e, frame),
        0x13bac6 => crate::recovered_remaining_control::open_caste_body(e, frame),
        0x13c018 => crate::recovered_remaining_control::itemcost_body(e, frame),
        0x13f52c => crate::recovered_remaining_control::add_initial_items_body(e, frame),
        0x13f684 => crate::recovered_remaining_control::new_character_body(e, frame),
        0x14695e => crate::recovered_remaining_control::number_body(e, frame),
        0x146a50 => crate::recovered_remaining_control::copy_part_of_string_body(e, frame),
        0x146aa2 => crate::recovered_remaining_control::print_stat_body(e, frame),
        0x148f9e => crate::recovered_remaining_control::view_restrictions_body(e, frame),
        0x149fc6 => crate::recovered_remaining_control::ploticon2_body(e, frame),
        0x14a074 => crate::recovered_remaining_control::ploticon_body(e, frame),
        0x14a29e => crate::recovered_remaining_control::update_port_body(e, frame),
        0x14a918 => crate::recovered_remaining_control::plus_body(e, frame),
        0x14aa22 => crate::recovered_remaining_control::minus_body(e, frame),
        0x14d7cc => crate::recovered_remaining_control::race_body(e, frame),
        0x15498e => crate::recovered_remaining_control::show_charge_body(e, frame),
        0x1553bc => crate::recovered_remaining_control::showdamage_body(e, frame),
        0x15741a => crate::recovered_remaining_control::show_special_body(e, frame),
        0x157758 => crate::recovered_remaining_control::show_item_stats_body(e, frame),
        0x157ef8 => crate::recovered_remaining_control::showque_body(e, frame),
        0x159a96 => crate::recovered_remaining_control::show_stats_body(e, frame),
        0x15a49a => crate::recovered_remaining_control::show_upgrade_body(e, frame),
        0x161068 => crate::recovered_remaining_control::update_odds_body(e, frame),
        0x161d86 => crate::recovered_remaining_control::update_specialties_body(e, frame),
        0x162238 => crate::recovered_remaining_control::upgrade_warrior_body(e, frame),
        0x1634f8 => crate::recovered_remaining_control::apply_upgrade_body(e, frame),
        0x1636ec => crate::recovered_remaining_control::calculate_statistics_body(e, frame),
        0x1213f6 => crate::recovered_application_children::set_ind_string_body(e, frame),
        0x122b18 => crate::recovered_application_children::allocate_building_body(e, frame),
        0x122dc4 => crate::recovered_application_children::gcopy_body(e, frame),
        0x1241b4 => crate::recovered_application_children::is_color_gamma_available_body(e, frame),
        0x124212 => crate::recovered_application_children::get_monitor_state_body(e, frame),
        0x124242 => crate::recovered_application_children::release_monitor_state_body(e, frame),
        0x12427c => crate::recovered_application_children::capture_monitor_list_body(e, frame),
        0x124308 => crate::recovered_application_children::capture_monitor_gamma_body(e, frame),
        0x124576 => crate::recovered_application_children::restore_gamma_list_body(e, frame),
        0x1245cc => crate::recovered_application_children::dispose_gamma_list_body(e, frame),
        0x124632 => crate::recovered_application_children::fade_to_monitor_state_body(e, frame),
        0x124666 => crate::recovered_application_children::fade_to_constant_color_body(e, frame),
        0x1246a8 => crate::recovered_application_children::run_fade_sequence_body(e, frame),
        0x1247cc => crate::recovered_application_children::x_calc_fade_to_color_body(e, frame),
        0x12485e => crate::recovered_application_children::fade_gamma_list_body(e, frame),
        0x1248f0 => crate::recovered_application_children::calc_fade_to_color_body(e, frame),
        0x124a44 => crate::recovered_application_children::x_calc_fade_to_gamma_body(e, frame),
        0x124bba => crate::recovered_application_children::get_device_gamma_table_body(e, frame),
        0x124c18 => crate::recovered_application_children::set_device_gamma_table_body(e, frame),
        0x124ca2 => {
            crate::recovered_application_children::color_gamma_trap_available_body(e, frame)
        }
        0x124d04 => crate::recovered_application_children::color_gamma_memset_body(e, frame),
        0x1270c0 => crate::recovered_application_children::get_version_string_body(e, frame),
        0x127462 => crate::recovered_application_children::gworld_init_body(e, frame),
        0x1315a2 => crate::recovered_application_children::fade_in_out_body(e, frame),
        0x1366a8 => crate::recovered_application_children::pool_body(e, frame),
        0x136e04 => crate::recovered_application_children::get_save_filename_body(e, frame),
        0x13777a => crate::recovered_application_children::set_file_info_body(e, frame),
        0x13ba6c => crate::recovered_application_children::open_race_body(e, frame),
        0x13ef12 => crate::recovered_application_children::name_body(e, frame),
        0x14aba0 => crate::recovered_application_children::portrait_body(e, frame),
        0x14bca4 => crate::recovered_application_children::load_preferences_body(e, frame),
        0x14bfe8 => crate::recovered_application_children::application_name_body(e, frame),
        0x14c01c => crate::recovered_application_children::save_preferences_body(e, frame),
        0x14c318 => crate::recovered_application_children::check_for_mismatch_body(e, frame),
        0x14c440 => crate::recovered_application_children::create_preferences_body(e, frame),
        0x14c622 => crate::recovered_application_children::open_preferences_body(e, frame),
        0x14ee9e => crate::recovered_application_children::hide_menu_bar_body(e, frame),
        0x14ef36 => crate::recovered_application_children::show_menu_bar_body(e, frame),
        0x15311a => crate::recovered_application_children::set_free_body(e, frame),
        0x1531c0 => crate::recovered_application_children::set_free2_body(e, frame),
        0x153584 => crate::recovered_application_children::set_speaker_volume_body(e, frame),
        0x153cf4 => crate::recovered_application_children::dialog_number_long_body(e, frame),
        0x153d56 => crate::recovered_application_children::dialog_number_body(e, frame),
        0x153daa => crate::recovered_application_children::dialog_string_body(e, frame),
        0x157d38 => crate::recovered_application_children::show_logo_body(e, frame),
        0x157ea4 => crate::recovered_application_children::delay_interruptible_body(e, frame),
        0x105366 => crate::recovered_gameplay_leaves::restore_body_body(e, frame),
        0x105572 => crate::recovered_gameplay_leaves::bodyground_body(e, frame),
        0x10ae90 => crate::recovered_gameplay_leaves::cansee_body(e, frame),
        0x110d12 => crate::recovered_gameplay_leaves::checkformechanicalunits_body(e, frame),
        0x11235e => crate::recovered_gameplay_leaves::checkrange_body(e, frame),
        0x11279a => crate::recovered_gameplay_leaves::item_charges_body(e, frame),
        0x12063c => crate::recovered_gameplay_leaves::destroymonster_body(e, frame),
        0x125538 => crate::recovered_gameplay_leaves::getrange_body(e, frame),
        0x125960 => crate::recovered_gameplay_leaves::getclosest_body(e, frame),
        0x133bb8 => crate::recovered_gameplay_leaves::start_sound_cycle_body(e, frame),
        0x133c56 => crate::recovered_gameplay_leaves::sustain_sound_cycle_body(e, frame),
        0x136398 => crate::recovered_gameplay_leaves::nobrassnoammo_body(e, frame),
        0x1363e4 => crate::recovered_gameplay_leaves::best_choice_body(e, frame),
        0x1377f4 => crate::recovered_gameplay_leaves::drawspell_body(e, frame),
        0x1382d0 => crate::recovered_gameplay_leaves::string_body(e, frame),
        0x138a40 => crate::recovered_gameplay_leaves::xy_body(e, frame),
        0x13a3ae => crate::recovered_gameplay_leaves::save_against_body(e, frame),
        0x13f65e => crate::recovered_gameplay_leaves::pin_body(e, frame),
        0x149e0e => crate::recovered_gameplay_leaves::placemonster_body(e, frame),
        0x151f30 => crate::recovered_gameplay_leaves::direction_body(e, frame),
        0x153de6 => crate::recovered_gameplay_leaves::movement_cost_body(e, frame),
        0x153e44 => crate::recovered_gameplay_leaves::shift_body(e, frame),
        0x157c68 => crate::recovered_gameplay_leaves::save_against_attribute_body(e, frame),
        0x15b84c => crate::recovered_gameplay_leaves::spelleffectoutdoors_body(e, frame),
        0x15d210 => crate::recovered_gameplay_leaves::takegold_body(e, frame),
        0x15edfa => crate::recovered_gameplay_leaves::finddxdy_body(e, frame),
        0x15ef50 => crate::recovered_gameplay_leaves::to_weapon_body(e, frame),
        0x15efee => crate::recovered_gameplay_leaves::to_missile_body(e, frame),
        0x15f048 => crate::recovered_gameplay_leaves::missileset_body(e, frame),
        0x160fb2 => crate::recovered_gameplay_leaves::updatelight_body(e, frame),
        0x1124dc => crate::recovered_gameplay_leaves::getrangetobody_body(e, frame),
        0x122bec => crate::recovered_gameplay_leaves::fastplot_body(e, frame),
        0x100af0 => crate::recovered_gameplay_control::attack_body(e, frame),
        0x101eb6 => crate::recovered_gameplay_control::attack_secondary_body(e, frame),
        0x10950e => crate::recovered_gameplay_control::button_choice_body(e, frame),
        0x10d4fe => crate::recovered_gameplay_control::cast_body(e, frame),
        0x110e96 => crate::recovered_gameplay_control::check_for_enemy_body(e, frame),
        0x1140ae => crate::recovered_gameplay_control::collide_body(e, frame),
        0x114378 => crate::recovered_gameplay_control::combat_body(e, frame),
        0x117be2 => crate::recovered_gameplay_control::combat_choice_body(e, frame),
        0x12185a => crate::recovered_gameplay_control::encounter_body(e, frame),
        0x13e276 => crate::recovered_gameplay_control::move_monster_body(e, frame),
        0x1400e4 => crate::recovered_gameplay_control::new_land_body(e, frame),
        0x15e148 => crate::recovered_gameplay_control::advance_time_body(e, frame),
        0x1469f8 => {
            e.full(crate::recovered_verification::doreg3() as u32);
            Ok(())
        }
        0x153ca4 | 0x13849e => {
            let (minimum, bound) = if entry == 0x153ca4 {
                (0, e.sw(frame.wrapping_add(8))?)
            } else {
                let minimum = e.sw(frame.wrapping_add(8))?;
                let maximum = e.sw(frame.wrapping_add(10))?;
                (minimum, maximum.wrapping_sub(minimum).wrapping_add(1))
            };
            e.draws = e
                .draws
                .checked_sub(1)
                .ok_or_else(|| RuleError::Invalid("application random budget exhausted".into()))?;
            let mut context =
                runtime::RuntimeContext::new(e.plan.state.random.clone(), e.remaining as usize);
            let plan = runtime::rand(&mut context, bound)?;
            e.plan.state.random = context.random;
            e.plan.effects.extend(
                plan.effects
                    .into_iter()
                    .map(ApplicationControlEffect::Runtime),
            );
            if let RuntimeResult::Word { value, native_d0 } = plan.result {
                e.full(native_d0);
                if entry == 0x13849e {
                    e.low(minimum.wrapping_add(value).wrapping_sub(1) as u16);
                }
                Ok(())
            } else {
                Err(RuleError::Invalid("unexpected source Rand return".into()).into())
            }
        }
        0x11177e => crate::recovered_system_control::check_for_secret_body(e, frame),
        0x12f0cc => crate::recovered_system_control::load_body(e, frame),
        0x13164e => crate::recovered_system_control::toolbox_init_body(e, frame),
        0x132780 => crate::recovered_system_control::main_body(e, frame),
        0x13283e => crate::recovered_system_control::main_loop_body(e, frame),
        0x132886 => crate::recovered_system_control::handle_event_body(e, frame),
        0x134e82 => crate::recovered_system_control::show_map_body(e, frame),
        0x135354 => crate::recovered_system_control::menu_init_body(e, frame),
        0x135fc8 => crate::recovered_system_control::add_delete_scenario_body(e, frame),
        0x1366f8 => crate::recovered_system_control::select_scenario_body(e, frame),
        0x13bd52 => crate::recovered_system_control::modify_body(e, frame),
        0x14913e => crate::recovered_system_control::party_loss_body(e, frame),
        0x154016 => crate::recovered_system_control::update_scenario_availability_body(e, frame),
        0x1540c4 => crate::recovered_system_control::about_body(e, frame),
        0x15d2f0 => crate::recovered_system_control::temple_body(e, frame),
        0x129516 => {
            use crate::recovered_inventory_interaction::{
                self as inventory, InventoryInteractionEffect, ItemsRegisters,
            };
            let registers = ItemsRegisters {
                data: e.registers.data.map(|value| value.value),
                known_data_bits: e.registers.data.map(|value| value.known_mask),
                address: e
                    .registers
                    .address
                    .map(|value| (value.known_mask == u32::MAX).then_some(value.value)),
                condition_codes: e.registers.ccr,
                known_condition_codes: e.registers.ccr_known_mask,
            };
            let limits = ApplicationControlLimits {
                maximum_steps: e.remaining,
                maximum_random_draws: e.draws,
                maximum_call_depth: e.limits.maximum_call_depth.saturating_sub(e.depth),
                ..e.limits
            };
            let child =
                inventory::items_body_plan(&e.plan.state, frame, &registers, limits, e.services)?;
            let snapshot = child.effects.iter().rev().find_map(|effect| match effect {
                InventoryInteractionEffect::ItemsRegisters(snapshot) => Some(snapshot.clone()),
                _ => None,
            });
            e.adopt(
                child.state.memory,
                child.state.random,
                child.native_return,
                child.flow,
                child
                    .effects
                    .into_iter()
                    .map(ApplicationControlEffect::Interaction),
            )?;
            if let Some(snapshot) = snapshot {
                e.registers.data = std::array::from_fn(|n| RegisterBits {
                    value: snapshot.data[n],
                    known_mask: snapshot.known_data_bits[n],
                });
                e.registers.address = snapshot
                    .address
                    .map(|value| value.map_or(RegisterBits::default(), RegisterBits::full));
                e.registers.ccr = snapshot.condition_codes;
                e.registers.ccr_known_mask = snapshot.known_condition_codes;
            }
            Ok(())
        }
        0x12e4e4 => crate::recovered_core_control::kill_body_body(e, frame),
        0x121524 => crate::recovered_core_control::empty_queue_body(e, frame),
        0x109210 => crate::recovered_core_control::build_monster_body(e, frame),
        0x125a3a => crate::recovered_combat_setup::get_up_body(e, frame),
        0x11d406 => crate::recovered_combat_setup::combat_setup_body(e, frame),
        0x151022 => crate::recovered_spell_control::resolve_spell_body(e, frame),
        0x15b9d8 => crate::recovered_spell_control::spell_list_body(e, frame),
        0x15cb0e => crate::recovered_spell_control::spell_targets_body(e, frame),
        0x1287c2 => crate::recovered_spell_control::heal_body(e, frame),
        0x14de04 => crate::recovered_spell_control::reduce_body(e, frame),
        0x13b624 => crate::recovered_main_control::main_screen_body(e, frame),
        0x13bb20 => crate::recovered_main_control::update_main_body(e, frame),
        0x1279bc => crate::recovered_main_control::handle_menu_choice_body(e, frame),
        0x149452 => crate::recovered_main_control::main_screen_init_body(e, frame),
        0x14b63a => crate::recovered_main_control::preference_body(e, frame),
        0x112650 | 0x160a8e | 0x13822a | 0x150114 | 0x15075a | 0x15b2f6 | 0x158454 | 0x1600fc
        | 0x15f66c | 0x10e6b6 | 0x10f186 | 0x1107cc | 0x16043e | 0x1500d8 | 0x133a18 | 0x16474a
        | 0x123d1c | 0x13a498 | 0x13154c => spell_children::dispatch(e, entry, frame),
        _ => children::dispatch(e, entry, frame),
    }
}

pub(crate) fn owns_native_stack(entry: u32) -> bool {
    entry == 0x129516
        || crate::recovered_system_control::owns_frame(entry)
        || crate::recovered_gameplay_control::owns_frame(entry)
        || crate::recovered_gameplay_leaves::owns_frame(entry)
        || crate::recovered_application_children::owns_frame(entry)
        || crate::recovered_remaining_control::owns_frame(entry)
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
