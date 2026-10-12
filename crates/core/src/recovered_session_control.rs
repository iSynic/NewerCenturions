//! Session, save and confirmation control from complete CODE 1 bodies.

use crate::recovered_character_creation::{
    self as creation, CharacterCreationEffect, CharacterCreationServices,
};
use crate::recovered_character_ui::{self as character_ui, CharacterUiEffect, CharacterUiState};
use crate::recovered_combat_center::{self as center, CombatCenterEffect};
use crate::recovered_equipment::{EquipmentLimits, read};
use crate::recovered_feedback::{
    self as feedback, FeedbackArg, FeedbackEffect, FeedbackObservation, FeedbackRequest,
    FeedbackServices,
};
use crate::recovered_field_control::{
    self as field, FieldControlEffect, FieldControlLimits, FieldControlServices,
};
use crate::recovered_files::{
    self as files, FileBuffer, FileContext, FileEffect, FileObservation, FileReply, FileRequest,
};
use crate::recovered_gamma::{GammaMemory, GammaReturn};
use crate::recovered_gamma_control::GammaControlEffect;
use crate::recovered_install::InstallMemory;
use crate::recovered_inventory_edit::{self as editing, InventoryEditEffect, InventoryEditLimits};
use crate::recovered_inventory_selection::{
    self as selection, InventorySelectionEffect, InventorySelectionLimits,
};
use crate::recovered_load_control::{self as load, LoadEffect, LoadFlow, LoadLimits};
use crate::recovered_preferences::{self as preferences, PreferenceContext, PreferenceEffect};
use crate::recovered_prompt_control::{
    self as prompt, LibraryObservation, LibraryRequest, PromptControlEffect, PromptControlState,
    PromptLimits, PromptRegisterRequest, PromptReturn,
};
use crate::recovered_records::RecordWrite;
use crate::recovered_rendering::RenderingMemory;
use crate::recovered_runtime::{self as runtime, RuntimeContext, RuntimeEffect, RuntimeResult};
use crate::recovered_scenario_menus::{self as menus, ScenarioMenuEffect};
use crate::recovered_startup::StartupServices;
use crate::recovered_storage::{self as storage, StorageEffect, StorageLimits};
use crate::recovered_timing::{self as timing, TimingServices};
use crate::recovered_verification as verification;
use crate::recovered_world_display::WorldDisplayState;
use crate::{RuleError, recovered_fade_ui};
use FeedbackArg::{Byte as B, Long as L, Pointer as P, Word as W};
use centurions_content::{WARRIOR_BYTES, Warrior};

pub trait SessionControlServices:
    FieldControlServices + CharacterCreationServices + StartupServices
{
}
impl<T: FieldControlServices + CharacterCreationServices + StartupServices> SessionControlServices
    for T
{
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SessionControlLimits {
    pub maximum_steps: u32,
    pub maximum_polls: usize,
    pub filename_capacity: usize,
    pub maximum_random_draws: u64,
    pub equipment: EquipmentLimits,
}
impl SessionControlLimits {
    fn prompt(self) -> PromptLimits {
        PromptLimits {
            maximum_steps: self.maximum_steps,
            maximum_polls: self.maximum_polls,
            filename_capacity: self.filename_capacity,
        }
    }
    fn field(self) -> FieldControlLimits {
        FieldControlLimits {
            maximum_steps: self.maximum_steps,
            maximum_polls: self.maximum_polls,
            filename_capacity: self.filename_capacity,
            maximum_random_draws: self.maximum_random_draws,
            equipment: self.equipment,
        }
    }
    fn editing(self) -> InventoryEditLimits {
        InventoryEditLimits {
            maximum_steps: self.maximum_steps,
            equipment: self.equipment,
            maximum_random_draws: self.maximum_random_draws,
        }
    }
    fn selection(self) -> InventorySelectionLimits {
        InventorySelectionLimits {
            maximum_steps: self.maximum_steps,
            prompt: self.prompt(),
            equipment: self.equipment,
            maximum_random_draws: self.maximum_random_draws,
        }
    }
    fn storage(self) -> StorageLimits {
        StorageLimits {
            filename_capacity: self.filename_capacity,
            maximum_iterations: self.maximum_steps as usize,
            maximum_polls: self.maximum_polls,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GenerationRegisters {
    pub d3_word: i16,
    pub d6_word: i16,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegistrationStage {
    ScenarioCode,
    ConfirmationName,
    ReservedName,
}
#[derive(Clone, Debug, PartialEq)]
pub enum SessionControlEffect {
    Write(RecordWrite),
    Feedback(FeedbackEffect),
    RegisterToolbox {
        request: PromptRegisterRequest,
        observation: FeedbackObservation,
    },
    Library {
        request: LibraryRequest,
        observation: LibraryObservation,
    },
    Creation(CharacterCreationEffect),
    CharacterUi(CharacterUiEffect),
    Prompt(PromptControlEffect),
    Field(FieldControlEffect),
    Editing(InventoryEditEffect),
    Selection(InventorySelectionEffect),
    Storage(StorageEffect),
    Load(LoadEffect),
    Menu(ScenarioMenuEffect),
    Center(CombatCenterEffect),
    Runtime(RuntimeEffect),
    Fade(GammaControlEffect),
    Preference(PreferenceEffect),
    File(FileEffect),
    Verification {
        member: u16,
        native_return: u32,
    },
    SharedCredits {
        member_count: usize,
        pool_before: i32,
        each: i16,
    },
    RegisteredByPolicy {
        stage: RegistrationStage,
    },
    RegisteredAdmission {
        stage: RegistrationStage,
        native_match: bool,
    },
    ExitToShell,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionControlQualification {
    SourcePortWithObservedBoundariesProjectedChildrenAndRegisteredPolicy,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SessionControlPlan {
    pub state: PromptControlState,
    pub effects: Vec<SessionControlEffect>,
    pub flow: LoadFlow,
    pub native_return: PromptReturn,
    pub qualification: SessionControlQualification,
}
struct Execution<'a, S: ?Sized> {
    plan: SessionControlPlan,
    services: &'a mut S,
    limits: SessionControlLimits,
    remaining: u32,
    initial_draws: u64,
    d0: Option<u32>,
}
impl<S: SessionControlServices + ?Sized> Execution<'_, S> {
    fn step(&mut self) -> Result<(), RuleError> {
        self.remaining = self
            .remaining
            .checked_sub(1)
            .ok_or(RuleError::Unsupported {
                action: "session_control.processing",
                dependency: "the supplied processing budget is exhausted",
            })?;
        Ok(())
    }
    fn b(&self, a: u32) -> Result<u8, RuleError> {
        self.plan.state.memory.byte(a)
    }
    fn w(&self, a: u32) -> Result<u16, RuleError> {
        self.plan.state.memory.word(a)
    }
    fn sw(&self, a: u32) -> Result<i16, RuleError> {
        Ok(self.w(a)? as i16)
    }
    fn sb(&self, a: u32) -> Result<i16, RuleError> {
        Ok(i16::from(self.b(a)? as i8))
    }
    fn l(&self, a: u32) -> Result<u32, RuleError> {
        self.plan.state.memory.long(a)
    }
    fn write(&mut self, a: u32, bytes: &[u8]) -> Result<(), RuleError> {
        read(&self.plan.state.memory, a, bytes.len())?;
        self.plan.state.memory.insert_region(a, bytes);
        self.plan
            .effects
            .push(SessionControlEffect::Write(RecordWrite {
                address: a,
                bytes: bytes.to_vec(),
            }));
        Ok(())
    }
    fn byte(&mut self, a: u32, v: u8) -> Result<(), RuleError> {
        self.write(a, &[v])
    }
    fn ww(&mut self, a: u32, v: u16) -> Result<(), RuleError> {
        self.write(a, &v.to_be_bytes())
    }
    fn lw(&mut self, a: u32, v: u32) -> Result<(), RuleError> {
        self.write(a, &v.to_be_bytes())
    }
    fn addw(&mut self, a: u32, v: u16) -> Result<(), RuleError> {
        self.ww(a, self.w(a)?.wrapping_add(v))
    }
    fn full(&mut self, v: u32) {
        self.d0 = Some(v);
        self.plan.native_return = PromptReturn::Full(v);
    }
    fn low(&mut self, v: u16) {
        self.d0 = self.d0.map(|d| (d & 0xffff0000) | u32::from(v));
        self.plan.native_return = self.d0.map_or(PromptReturn::LowWord(v), PromptReturn::Full);
    }
    fn returned(&mut self, value: PromptReturn) {
        match value {
            PromptReturn::Full(d) => self.full(d),
            PromptReturn::LowWord(w) => {
                self.d0 = None;
                self.plan.native_return = PromptReturn::LowWord(w);
            }
            PromptReturn::Unchanged => {
                self.d0 = None;
                self.plan.native_return = PromptReturn::Unchanged;
            }
            PromptReturn::NoReturn => {
                self.d0 = None;
                self.plan.native_return = PromptReturn::NoReturn;
            }
        }
    }
    fn active(&self) -> bool {
        self.plan.flow == LoadFlow::Returned
    }
    fn draw_budget(&self) -> Result<(), RuleError> {
        if self
            .plan
            .state
            .random
            .draws
            .checked_sub(self.initial_draws)
            .is_none_or(|n| n > self.limits.maximum_random_draws)
        {
            return Err(RuleError::Unsupported {
                action: "session_control.random",
                dependency: "the supplied random-draw budget is exhausted",
            });
        }
        Ok(())
    }
    fn trap(&mut self, opcode: u16, arguments: Vec<FeedbackArg>) -> Result<u32, RuleError> {
        self.step()?;
        let request = FeedbackRequest::Toolbox { opcode, arguments };
        let observation =
            FeedbackServices::observe(self.services, &request, &self.plan.state.memory)?;
        let value = observation.value;
        self.d0 = Some(observation.d0);
        self.plan.state.memory = observation.memory.clone();
        self.plan
            .effects
            .push(SessionControlEffect::Feedback(FeedbackEffect::Service {
                request,
                observation,
            }));
        Ok(value)
    }
    fn registers(
        &mut self,
        opcode: u16,
        d0_input: Option<u32>,
        a0_input: Option<u32>,
        arguments: Vec<FeedbackArg>,
    ) -> Result<u32, RuleError> {
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
        self.d0 = Some(observation.d0);
        self.plan.state.memory = observation.memory.clone();
        self.plan
            .effects
            .push(SessionControlEffect::RegisterToolbox {
                request,
                observation,
            });
        Ok(value)
    }
    fn flush(&mut self) -> Result<(), RuleError> {
        self.registers(0xa032, Some(0xffff), None, vec![])?;
        Ok(())
    }
    fn lib(&mut self, entry: u32, arguments: Vec<FeedbackArg>) -> Result<u32, RuleError> {
        self.step()?;
        let request = LibraryRequest { entry, arguments };
        let observation = self.services.library(&request, &self.plan.state.memory)?;
        let value = observation.value;
        self.d0 = Some(observation.d0);
        self.plan.state.memory = observation.memory.clone();
        self.plan.effects.push(SessionControlEffect::Library {
            request,
            observation,
        });
        Ok(value)
    }
    fn native_d0(&self) -> Result<u32, RuleError> {
        self.d0.ok_or(RuleError::Unsupported {
            action: "session_control.native_return",
            dependency: "the reached native D0 value is not known",
        })
    }
    fn port(&mut self, a: u32) -> Result<(), RuleError> {
        self.trap(0xa873, vec![L(a)])?;
        Ok(())
    }
    fn feedback(&mut self, child: feedback::FeedbackPlan) {
        self.plan.state.memory = child.memory;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(SessionControlEffect::Feedback),
        );
        self.d0 = child.native_return;
    }
    fn sound(&mut self, id: i16) -> Result<(), RuleError> {
        let child = feedback::sound(&self.plan.state.memory, id, self.services)?;
        self.feedback(child);
        Ok(())
    }
    fn warn(&mut self, frame: u32, id: i16) -> Result<(), RuleError> {
        self.ww(frame.wrapping_add(8), id as u16)?;
        let child = feedback::warn(
            &self.plan.state.memory,
            frame,
            id,
            self.limits.maximum_polls,
            self.services,
        )?;
        self.feedback(child);
        Ok(())
    }
    fn flash(
        &mut self,
        frame: u32,
        pointer: u32,
        coordinates: u32,
        modes: u32,
    ) -> Result<(), RuleError> {
        self.lw(frame.wrapping_add(16), modes)?;
        self.lw(frame.wrapping_add(12), coordinates)?;
        self.lw(frame.wrapping_add(8), pointer)?;
        let child = feedback::flashmessage(
            &self.plan.state.memory,
            frame,
            pointer,
            (coordinates >> 16) as u16 as i16,
            coordinates as u16 as i16,
            (modes >> 16) as u16 as i16,
            modes as u16 as i16,
            self.limits.maximum_polls,
            self.services,
        )?;
        self.feedback(child);
        Ok(())
    }
    fn scratch(&mut self, frame: u32, id: i16) -> Result<(), RuleError> {
        self.ww(frame.wrapping_add(8), id as u16)?;
        let child = load::scratch(&self.plan.state.memory, frame, self.services)?;
        self.plan.state.memory = child.memory;
        self.plan.flow = child.flow;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(SessionControlEffect::Load));
        self.d0 = child.native_return;
        if !self.active() {
            self.plan.native_return = PromptReturn::NoReturn;
        }
        Ok(())
    }
    fn runtime(&mut self, entry: u32) -> Result<(), RuleError> {
        let observations = self
            .services
            .runtime_observations(entry, &self.plan.state.memory)?;
        let mut context =
            RuntimeContext::new(self.plan.state.random.clone(), self.remaining as usize);
        context.memory.bytes = self.plan.state.memory.bytes.clone();
        let child = match entry {
            0x13a498 => runtime::copy_in(&mut context, &observations)?,
            0x13e13c => runtime::compact_heap(&mut context, &observations)?,
            _ => return Err(RuleError::Invalid("unknown session runtime child".into())),
        };
        self.plan.state.memory.bytes = context.memory.bytes;
        self.plan.state.random = context.random;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(SessionControlEffect::Runtime));
        self.d0 = match child.result {
            RuntimeResult::Long(d) => Some(d),
            _ => None,
        };
        self.draw_budget()
    }
    fn prompt(&mut self, frame: u32, entry: u32) -> Result<(), RuleError> {
        let child = match entry {
            0x14cff6 => {
                prompt::question(&self.plan.state, frame, self.limits.prompt(), self.services)?
            }
            0x12382a => prompt::first_tutorial(
                &self.plan.state,
                frame,
                self.limits.prompt(),
                self.services,
            )?,
            _ => return Err(RuleError::Invalid("unknown session prompt child".into())),
        };
        self.plan.state = child.state;
        self.plan.flow = child.flow;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(SessionControlEffect::Prompt));
        self.returned(child.native_return);
        self.draw_budget()
    }
    fn creation(
        &mut self,
        frame: u32,
        entry: u32,
        registers: Option<GenerationRegisters>,
    ) -> Result<(), RuleError> {
        let child = match entry {
            0x13ef12 => {
                creation::name(&self.plan.state, frame, self.limits.prompt(), self.services)?
            }
            0x13b30c => {
                creation::start_level(&self.plan.state, frame, self.limits.prompt(), self.services)?
            }
            0x113988 => {
                creation::caste(&self.plan.state, frame, self.limits.prompt(), self.services)?
            }
            0x14d7cc => {
                creation::race(&self.plan.state, frame, self.limits.prompt(), self.services)?
            }
            0x127264 => {
                creation::gender(&self.plan.state, frame, self.limits.prompt(), self.services)?
            }
            0x13f684 => creation::new_character(
                &self.plan.state,
                frame,
                self.limits.prompt(),
                self.services,
            )?,
            0x14aba0 => creation::portrait(
                &self.plan.state,
                frame,
                registers
                    .ok_or(RuleError::Unsupported {
                        action: "session_control.generate",
                        dependency: "the reached incoming D3.W has not been supplied",
                    })?
                    .d3_word,
                self.limits.prompt(),
                self.services,
            )?,
            0x1289fa => creation::icon_picture(
                &self.plan.state,
                frame,
                registers
                    .ok_or(RuleError::Unsupported {
                        action: "session_control.generate",
                        dependency: "the reached incoming D6.W has not been supplied",
                    })?
                    .d6_word,
                self.limits.prompt(),
                self.services,
            )?,
            _ => {
                return Err(RuleError::Invalid(
                    "unknown character creation child".into(),
                ));
            }
        };
        self.plan.state = child.state;
        self.plan.flow = child.flow;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(SessionControlEffect::Creation),
        );
        self.returned(child.native_return);
        self.draw_budget()
    }
    fn field(&mut self, frame: u32, entry: u32) -> Result<(), RuleError> {
        let child = match entry {
            0x1374bc => {
                field::save_character(&self.plan.state, frame, self.limits.field(), self.services)?
            }
            0x122e24 => {
                field::file_prep(&self.plan.state, frame, self.limits.field(), self.services)?
            }
            _ => return Err(RuleError::Invalid("unknown session field child".into())),
        };
        self.plan.state = child.state;
        self.plan.flow = child.flow;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(SessionControlEffect::Field));
        self.returned(child.native_return);
        self.draw_budget()
    }
    fn editing(&mut self, frame: u32, initial: bool) -> Result<(), RuleError> {
        let child = if initial {
            editing::add_initial_items(
                &self.plan.state.memory,
                &self.plan.state.random,
                frame,
                self.limits.editing(),
                self.services,
            )?
        } else {
            editing::minus(
                &self.plan.state.memory,
                &self.plan.state.random,
                frame,
                self.limits.editing(),
                self.services,
            )?
        };
        self.plan.state.memory = child.memory;
        self.plan.state.random = child.random;
        self.plan.flow = child.flow;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(SessionControlEffect::Editing));
        self.d0 = child.native_return;
        if !self.active() {
            self.plan.native_return = PromptReturn::NoReturn;
        }
        self.draw_budget()
    }
    fn selection(&mut self, frame: u32, scan: bool) -> Result<(), RuleError> {
        let child = if scan {
            selection::scan_select(
                &self.plan.state,
                frame,
                self.limits.selection(),
                self.services,
            )?
        } else {
            selection::update_fat(
                &self.plan.state,
                frame,
                self.limits.selection(),
                self.services,
            )?
        };
        self.plan.state = child.state;
        self.plan.flow = child.flow;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(SessionControlEffect::Selection),
        );
        self.returned(child.native_return);
        self.draw_budget()
    }
    fn show_stats(&mut self, frame: u32) -> Result<(), RuleError> {
        self.ww(frame.wrapping_add(8), 0)?;
        let state = CharacterUiState {
            memory: self.plan.state.memory.clone(),
            files: FileContext {
                filename: vec![0; self.limits.filename_capacity],
                race_profile: vec![0; 408],
                caste_profile: vec![0; 576],
                application_name: vec![0; 256],
                ..FileContext::default()
            },
        };
        let child = character_ui::show_stats(&state, 0, frame, self.services)?;
        self.plan.state.memory = child.state.memory;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(SessionControlEffect::CharacterUi),
        );
        self.d0 = child.native_return.map(|v| v as u32);
        Ok(())
    }
    fn center(&mut self, frame: u32) -> Result<(), RuleError> {
        self.ww(frame.wrapping_add(8), 0)?;
        let state = WorldDisplayState {
            memory: self.plan.state.memory.clone(),
            random: self.plan.state.random.clone(),
        };
        let child = center::centerpict(&state, frame, self.remaining, self.services)?;
        self.plan.state.memory = child.memory;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(SessionControlEffect::Center));
        if let Some(d) = child.native_return {
            self.d0 = Some(d);
        }
        Ok(())
    }
    fn save_land(&mut self, frame: u32) -> Result<(), RuleError> {
        self.lw(frame.wrapping_add(8), self.l(0x442774)?)?;
        let child = storage::save_land(
            &self.plan.state.memory,
            frame,
            self.limits.storage(),
            self.services,
        )?;
        self.plan.state.memory = child.memory;
        self.plan.flow = child.flow;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(SessionControlEffect::Storage));
        self.d0 = child.native_return;
        if !self.active() {
            self.plan.native_return = PromptReturn::NoReturn;
        }
        Ok(())
    }
    fn save_shop(&mut self, _frame: u32) -> Result<(), RuleError> {
        let child = storage::save_shop(
            &self.plan.state.memory,
            self.limits.storage(),
            self.services,
        )?;
        self.plan.state.memory = child.memory;
        self.plan.flow = child.flow;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(SessionControlEffect::Storage));
        self.d0 = child.native_return;
        if !self.active() {
            self.plan.native_return = PromptReturn::NoReturn;
        }
        Ok(())
    }
    fn load_land(&mut self, frame: u32) -> Result<(), RuleError> {
        self.lw(frame.wrapping_add(8), self.l(0x442774)?)?;
        let child = load::load_land(
            &self.plan.state.memory,
            frame,
            LoadLimits {
                filename_capacity: self.limits.filename_capacity,
                maximum_polls: self.limits.maximum_polls,
            },
            self.services,
        )?;
        self.plan.state.memory = child.memory;
        self.plan.flow = child.flow;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(SessionControlEffect::Load));
        self.d0 = child.native_return;
        if !self.active() {
            self.plan.native_return = PromptReturn::NoReturn;
        }
        Ok(())
    }
    fn filename(
        &mut self,
        frame: u32,
        name: u32,
        selector: Option<u16>,
        temporary: bool,
    ) -> Result<(), RuleError> {
        if let Some(slot) = selector {
            self.ww(frame.wrapping_add(12), slot)?;
        }
        self.lw(frame.wrapping_add(8), name)?;
        let mut context = FileContext {
            scenario_directory: if temporary || selector.is_some() {
                vec![]
            } else {
                c_string(
                    &self.plan.state.memory,
                    0x44260c,
                    self.limits.filename_capacity,
                )?
            },
            variant: self.b(0x44274c)?,
            filename: read(
                &self.plan.state.memory,
                0x4389a4,
                self.limits.filename_capacity,
            )?,
            ..FileContext::default()
        };
        let name = c_string(&self.plan.state.memory, name, self.limits.filename_capacity)?;
        let number = self
            .services
            .filename_number(context.variant, &self.plan.state.memory)?;
        let observations = [FileObservation {
            request: FileRequest::NumberToString(i32::from(context.variant)),
            reply: FileReply::Pascal(number),
        }];
        let child = if let Some(slot) = selector {
            files::get_save_filename(&mut context, &name, slot as i16, &observations)?
        } else if temporary {
            files::get_temp_filename(&mut context, &name, &observations)?
        } else {
            files::get_filename(&mut context, &name, &observations)?
        };
        for effect in child.effects {
            if let FileEffect::Write {
                destination: FileBuffer::Filename,
                offset,
                bytes,
            } = &effect
            {
                self.write(0x4389a4_u32.wrapping_add(*offset as u32), bytes)?;
            } else {
                self.plan.effects.push(SessionControlEffect::File(effect));
            }
        }
        self.full(0);
        Ok(())
    }
    fn metadata(&mut self, frame: u32, name: u32, kind: u32) -> Result<(), RuleError> {
        self.lw(frame.wrapping_add(12), name)?;
        self.lw(frame.wrapping_add(8), kind)?;
        self.lib(0x400188, vec![P(name)])?;
        self.lib(0x400150, vec![P(name), W(0), P(0x439cfc)])?;
        self.lib(
            0x400090,
            vec![P(self.l(frame.wrapping_add(8))?), P(0x439cfc)],
        )?;
        self.lib(0x400090, vec![P(0x402436), P(0x439d00)])?;
        self.lib(0x400158, vec![P(name), W(0), P(0x439cfc)])?;
        self.lib(0x400180, vec![P(name)])?;
        self.full(0);
        Ok(())
    }
    fn read_file(&mut self, handle: u32, size: u32, destination: u32) -> Result<u32, RuleError> {
        self.lib(0x400048, vec![L(handle), L(1), L(size), P(destination)])?;
        self.native_d0()
    }
    fn write_file(
        &mut self,
        handle: u32,
        size: u32,
        count: u32,
        source: u32,
    ) -> Result<u32, RuleError> {
        self.lib(0x400050, vec![L(handle), L(count), L(size), P(source)])?;
        self.native_d0()
    }
    fn open_file(&mut self, mode: u32, name: u32) -> Result<u32, RuleError> {
        let handle = self.lib(0x400070, vec![P(mode), P(name)])?;
        self.full(handle);
        Ok(handle)
    }
    fn fade(&mut self, frame: u32, direction: u16) -> Result<(), RuleError> {
        self.lw(frame.wrapping_add(8), (25 << 16) | u32::from(direction))?;
        let memory = GammaMemory {
            bytes: self.plan.state.memory.bytes.clone(),
        };
        let child =
            recovered_fade_ui::fadeinout(&memory, frame, 25, direction as i16, self.services)
                .map_err(|e| RuleError::Invalid(e.to_string()))?;
        self.plan.state.memory.bytes = child.memory.bytes;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(SessionControlEffect::Fade));
        self.d0 = match child.native_return {
            GammaReturn::DataRegister(d) => Some(d),
            _ => None,
        };
        Ok(())
    }
    fn delay(&mut self, frame: u32, duration: i16) -> Result<(), RuleError> {
        self.ww(frame.wrapping_add(8), duration as u16)?;
        let polls = self.limits.maximum_polls;
        let child = timing::delay(
            duration,
            0,
            &mut Clock {
                execution: self,
                remaining: polls,
            },
        )?;
        self.full(child.native_result);
        Ok(())
    }
    fn preferences(&mut self) -> Result<(), RuleError> {
        let inputs = StartupServices::preference_inputs(self.services, &self.plan.state.memory)?;
        let mut context = PreferenceContext::new(self.plan.state.random.clone());
        context.handles = inputs.handles;
        for (a, n) in [
            (0x439568, 2),
            (0x43956a, 2),
            (0x43956c, 4),
            (0x439570, 4),
            (0x439574, 2),
            (0x439578, 2),
            (0x43957a, 2),
            (0x43957c, 2),
            (0x43957e, 2),
            (0x439580, 2),
            (0x439582, 2),
            (0x439584, 2),
            (0x439586, 2),
            (0x439588, 2),
            (0x43958a, 2),
            (0x43958c, 2),
            (0x43958e, 2),
            (0x439590, 2),
            (0x439592, 2),
            (0x438589, 1),
            (0x402f44, 2),
            (0x442560, 2),
            (0x438598, 4),
            (0x439596, 2),
            (0x4046ea, 2),
        ] {
            context
                .globals
                .insert(a, read(&self.plan.state.memory, a, n)?);
        }
        for i in 0..20 {
            context
                .globals
                .insert(0x44270e + i, vec![self.b(0x44270e + i)?]);
        }
        context.globals.insert(
            0x4385a0,
            read(
                &self.plan.state.memory,
                0x4385a0,
                usize::from(self.b(0x4385a0)?) + 1,
            )?,
        );
        let child = preferences::save_preferences(&mut context, &inputs.observations)?;
        for effect in child.effects {
            if let PreferenceEffect::GlobalWrite { address, bytes } = &effect {
                self.write(*address, bytes)?;
            } else {
                self.plan
                    .effects
                    .push(SessionControlEffect::Preference(effect));
            }
        }
        self.plan.state.random = context.random;
        self.full(0);
        self.draw_budget()
    }
    fn verify(&mut self, member: u16) -> Result<(), RuleError> {
        let body = 0x448aa0_u32.wrapping_add(u32::from(member) * 3138);
        let base = 0x43788a_u32.wrapping_add(u32::from(member) * 80);
        let end = body.wrapping_add(3138);
        let memory = InstallMemory {
            base_address: base,
            bytes: read(
                &self.plan.state.memory,
                base,
                end.wrapping_sub(base) as usize,
            )?,
        };
        let child = verification::setverify(&memory, member)?;
        for write in child.writes {
            self.write(write.address, &write.bytes)?;
        }
        self.plan.effects.push(SessionControlEffect::Verification {
            member,
            native_return: child.native_return,
        });
        self.full(child.native_return);
        Ok(())
    }
    fn share(&mut self) -> Result<(), RuleError> {
        let last = self.sb(0x44273a)?;
        if last < 0 {
            return Err(RuleError::Invalid(
                "credit sharing requires a reached party".into(),
            ));
        }
        let mut warriors = Vec::new();
        for member in 0..=last {
            warriors.push(Warrior {
                name: String::new(),
                original_sha256: String::new(),
                bytes: read(
                    &self.plan.state.memory,
                    warrior(member as u16),
                    WARRIOR_BYTES,
                )?,
            });
        }
        let pool_before = self.l(0x442788)? as i32;
        let mut pool = pool_before;
        let each = crate::party::share_credits(&mut warriors, &mut pool)?;
        for member in 0..=last {
            self.step()?;
            self.lw(
                0x442788,
                self.l(0x442788)?
                    .wrapping_add(u32::from(self.w(warrior(member as u16) + 0x9da)?)),
            )?;
        }
        self.lw(0x442788, 0)?;
        for member in 0..=last {
            self.ww(warrior(member as u16) + 0x9da, each as u16)?;
        }
        self.plan.effects.push(SessionControlEffect::SharedCredits {
            member_count: warriors.len(),
            pool_before,
            each,
        });
        self.full(0);
        Ok(())
    }
}
impl<S: SessionControlServices + ?Sized> Execution<'_, S> {
    fn generate(
        &mut self,
        frame: u32,
        registers: Option<GenerationRegisters>,
    ) -> Result<(), RuleError> {
        self.trap(0xa938, vec![W(0)])?;
        self.ww(frame.wrapping_sub(4), 0)?;
        self.creation(frame.wrapping_sub(12), 0x13ef12, registers)?;
        if !self.active() {
            return Ok(());
        }
        if self.result_word()? == 0 {
            self.port(self.l(0x479fd8)?)?;
            self.trap(0xa863, vec![L(33)])?;
            self.trap(0xa981, vec![L(self.l(0x479fd8)?)])?;
            self.full(0);
            return Ok(());
        }
        self.creation(frame.wrapping_sub(10), 0x13b30c, registers)?;
        if !self.active() {
            return Ok(());
        }
        self.ww(frame.wrapping_sub(2), self.result_word()?)?;
        self.ww(frame.wrapping_sub(4), 0)?;
        self.creation(frame.wrapping_sub(12), 0x113988, registers)?;
        if !self.active() {
            return Ok(());
        }
        self.ww(frame.wrapping_sub(6), 0)?;
        self.creation(frame.wrapping_sub(14), 0x14d7cc, registers)?;
        if !self.active() {
            return Ok(());
        }
        if self.w(0x41f91c)? != 0 {
            self.creation(frame.wrapping_sub(10), 0x127264, registers)?;
            if !self.active() {
                return Ok(());
            }
        }
        self.ww(0x438756, 1)?;
        self.byte(0x44273f, 0)?;
        self.ww(
            0x447270,
            self.w(0x44724e)?.wrapping_mul(6).wrapping_add(251),
        )?;
        self.ww(frame.wrapping_sub(4), 1)?;
        self.creation(frame.wrapping_sub(12), 0x14aba0, registers)?;
        if !self.active() {
            return Ok(());
        }
        self.ww(0x447272, self.w(0x447270)?.wrapping_add(9000))?;
        self.ww(frame.wrapping_sub(6), 1)?;
        self.creation(frame.wrapping_sub(14), 0x1289fa, registers)?;
        if !self.active() {
            return Ok(());
        }
        self.ww(frame.wrapping_sub(8), self.w(frame.wrapping_sub(2))?)?;
        self.creation(frame.wrapping_sub(16), 0x13f684, registers)?;
        if !self.active() {
            return Ok(());
        }
        if self.w(0x447276)? != 0 {
            self.ww(0x4434aa, 0)?;
            while self.sw(0x4434aa)? < 7 {
                self.step()?;
                self.ww(0x4434ac, 0)?;
                while self.sw(0x4434ac)? < 7 {
                    self.step()?;
                    self.byte(
                        index(
                            0x447c0c_u32.wrapping_add(u32::from(self.w(0x4434aa)?) * 7),
                            self.sw(0x4434ac)?,
                            1,
                        ),
                        0,
                    )?;
                    self.addw(0x4434ac, 1)?;
                }
                self.addw(0x4434aa, 1)?;
            }
            self.selection(frame.wrapping_sub(10), true)?;
            if !self.active() {
                return Ok(());
            }
            self.lw(0x4394b8, self.l(0x4394bc)?)?;
            self.show_stats(frame.wrapping_sub(12))?;
        }
        self.editing(frame.wrapping_sub(10), true)?;
        if !self.active() {
            return Ok(());
        }
        self.lw(frame.wrapping_sub(6), 0x00020001)?;
        self.prompt(frame.wrapping_sub(14), 0x14cff6)?;
        if !self.active() {
            return Ok(());
        }
        if self.result_word()? != 0 {
            self.ww(frame.wrapping_sub(4), u16::MAX)?;
            self.field(frame.wrapping_sub(12), 0x1374bc)?;
        } else {
            self.ww(frame.wrapping_sub(4), 0)?;
            self.lw(frame.wrapping_sub(8), 0x447c3d)?;
            self.editing(frame.wrapping_sub(16), false)?;
        }
        if !self.active() {
            return Ok(());
        }
        self.port(self.l(0x479fd8)?)?;
        self.trap(0xa862, vec![L(69)])?;
        self.trap(0xa863, vec![L(33)])?;
        self.trap(0xa88a, vec![W(12)])?;
        self.trap(0xa85f, vec![P(0x43859c), L(6)])?;
        self.lib(0x400168, vec![L(self.l(0x43859c)?), P(0x479ed8)])?;
        self.trap(
            0xa98b,
            vec![P(0x479ed8), P(0x401d08), P(0x4388a4), P(0x401d08)],
        )?;
        self.trap(0xa983, vec![L(self.l(0x4394bc)?)])?;
        self.trap(0xa981, vec![L(self.l(0x479fd8)?)])?;
        self.full(0);
        Ok(())
    }
    fn result_word(&self) -> Result<u16, RuleError> {
        match self.plan.native_return {
            PromptReturn::Full(d) => Ok(d as u16),
            PromptReturn::LowWord(w) => Ok(w),
            _ => Err(RuleError::Unsupported {
                action: "session_control.child_result",
                dependency: "the reached child WORD result is not known",
            }),
        }
    }
    fn quit_game(&mut self, frame: u32) -> Result<(), RuleError> {
        self.preferences()?;
        self.ww(frame.wrapping_sub(6), 208)?;
        self.sound(208)?;
        self.ww(
            0x4395a2,
            half(self.sw(0x4395a0)?.wrapping_sub(480)).wrapping_add(20) as u16,
        )?;
        self.ww(0x4395a4, half(self.sw(0x43959e)?.wrapping_sub(640)) as u16)?;
        let created = self.l(0x479fd8)? == 0;
        if created {
            self.fade(frame.wrapping_sub(16), 1)?;
            loop {
                self.step()?;
                if self.trap(0xa924, vec![])? == 0 {
                    break;
                }
                let window = self.trap(0xa924, vec![])?;
                self.trap(0xa914, vec![L(window)])?;
            }
            let dialog = self.trap(0xa97c, vec![W(129), L(0), L(u32::MAX)])?;
            self.lw(0x479fd8, dialog)?;
        } else {
            for a in [0x4386ae, 0x43879c, 0x43878c] {
                if self.l(a)? != 0 {
                    self.trap(0xa914, vec![L(self.l(a)?)])?;
                }
            }
        }
        self.ww(frame.wrapping_sub(6), 203)?;
        self.sound(203)?;
        let dialog = self.trap(0xa97c, vec![W(175), L(0), L(u32::MAX)])?;
        self.lw(0x41f604, dialog)?;
        self.trap(0xa921, vec![L(self.l(0x41f604)?), L(0)])?;
        if self.l(0x438794)? != 0 {
            self.trap(0xa921, vec![L(self.l(0x438794)?), L(0)])?;
        }
        if created {
            self.fade(frame.wrapping_sub(16), 0)?;
            self.trap(0xa921, vec![L(self.l(0x479fd8)?), L(0)])?;
            self.trap(0xa921, vec![L(self.l(0x41f604)?), L(0)])?;
            self.move_quit_window(0x479fd8)?;
            self.move_quit_window(0x41f604)?;
            self.trap(0xa915, vec![L(self.l(0x479fd8)?)])?;
        } else {
            self.move_quit_window(0x41f604)?;
        }
        self.trap(0xa91d, vec![L(self.l(0x41f604)?), L(0x01cc0280), B(0)])?;
        self.port(self.l(0x479fd8)?)?;
        self.trap(0xa981, vec![L(self.l(0x479fd8)?)])?;
        if created {
            self.fade(frame.wrapping_sub(20), 0)?;
        }
        self.ww(0x4434aa, 1)?;
        while self.sw(0x4434aa)? < 12 {
            self.step()?;
            self.trap(
                0xa877,
                vec![
                    W(self
                        .w(0x4434aa)?
                        .wrapping_shl(5)
                        .wrapping_add(self.w(0x4395a4)?)),
                    W(self
                        .w(0x4434aa)?
                        .wrapping_mul(23)
                        .wrapping_add(self.w(0x4395a2)?)),
                ],
            )?;
            self.trap(
                0xa91d,
                vec![
                    L(self.l(0x479fd8)?),
                    W(640_u16.wrapping_sub(self.w(0x4434aa)?.wrapping_shl(6))),
                    W(460_u16.wrapping_sub(self.w(0x4434aa)?.wrapping_mul(46))),
                    B(0),
                ],
            )?;
            self.trap(
                0xa978,
                vec![
                    L(self.l(0x41f604)?),
                    L(self.l(self.l(0x41f604)?.wrapping_add(28))?),
                ],
            )?;
            self.delay(frame.wrapping_sub(14), 3)?;
            self.addw(0x4434aa, 1)?;
        }
        self.trap(0xa914, vec![L(self.l(0x479fd8)?)])?;
        self.trap(
            0xa978,
            vec![
                L(self.l(0x41f604)?),
                L(self.l(self.l(0x41f604)?.wrapping_add(28))?),
            ],
        )?;
        self.ww(frame.wrapping_sub(6), 205)?;
        self.sound(205)?;
        self.flush()?;
        for pointer in [
            0x40217c, 0x402166, 0x402152, 0x402140, 0x40212e, 0x40211c, 0x40210a, 0x4020f8,
            0x4020e6, 0x4020d4, 0x4020be, 0x4020a8, 0x402094, 0x402082, 0x402070, 0x40205e,
            0x40204c, 0x40203a, 0x402028, 0x402016, 0x402000, 0x401fea, 0x401fd6, 0x401fc4,
            0x401fb2, 0x401fa0, 0x401f8e, 0x401f7c, 0x401f6a, 0x401f58,
        ] {
            self.lib(0x400058, vec![P(pointer)])?;
        }
        self.metadata(frame.wrapping_sub(142), 0x401f40, 0x401f52)?;
        let handle = self.open_file(0x402466, 0x401f2e)?;
        self.lw(0x4434a6, handle)?;
        if handle != 0 {
            self.write_file(self.l(0x4434a6)?, 800, 1, 0x42dbe4)?;
            self.write_file(self.l(0x4434a6)?, 22000, 1, 0x4285f4)?;
            self.lib(0x400068, vec![L(self.l(0x4434a6)?)])?;
            self.metadata(frame.wrapping_sub(56), 0x401f2e, 0x401f28)?;
        }
        if self.sb(0x44273a)? > -1 {
            self.share()?;
            self.ww(0x4434aa, 0)?;
            while self.sw(0x4434aa)? <= self.sb(0x44273a)? {
                self.step()?;
                self.ww(frame.wrapping_sub(6), self.w(0x4434aa)?)?;
                self.field(frame.wrapping_sub(14), 0x1374bc)?;
                if !self.active() {
                    return Ok(());
                }
                self.addw(0x4434aa, 1)?;
            }
        }
        self.ww(0x4434aa, 0)?;
        while self.sw(0x4434aa)? < 60 {
            self.step()?;
            if self.trap(0xa924, vec![])? != 0 {
                let window = self.trap(0xa924, vec![])?;
                self.trap(0xa914, vec![L(window)])?;
            }
            self.addw(0x4434aa, 1)?;
        }
        let device = self.trap(0xaa2a, vec![])?;
        self.lw(0x4046e2, device)?;
        if self.b(0x43873f)? != self.b(0x43873e)? {
            self.fade(frame.wrapping_sub(16), 1)?;
            self.registers(
                0xaaa2,
                Some(0x000a0013),
                None,
                vec![
                    L(self.l(0x4046e2)?),
                    W(self.sb(0x43873e)? as u16),
                    L(0x00080004),
                ],
            )?;
            self.fade(frame.wrapping_sub(18), 0)?;
        } else {
            self.delay(frame.wrapping_sub(14), 80)?;
        }
        self.trap(0xa9f4, vec![])?;
        self.plan.effects.push(SessionControlEffect::ExitToShell);
        self.plan.flow = LoadFlow::Terminated(0);
        self.plan.native_return = PromptReturn::NoReturn;
        self.d0 = None;
        Ok(())
    }
    fn move_quit_window(&mut self, global: u32) -> Result<(), RuleError> {
        self.trap(
            0xa91b,
            vec![
                L(self.l(global)?),
                W(self.w(0x4395a4)?.wrapping_add(1)),
                W(self.w(0x4395a2)?.wrapping_add(1)),
                B(0),
            ],
        )?;
        Ok(())
    }
}
impl<S: SessionControlServices + ?Sized> Execution<'_, S> {
    fn down_button(&mut self, frame: u32) -> Result<(), RuleError> {
        self.ww(frame.wrapping_add(8), 0)?;
        self.trap(0xa874, vec![P(frame.wrapping_sub(22))])?;
        self.trap(0xa898, vec![P(frame.wrapping_sub(18))])?;
        if self.b(frame.wrapping_add(9))? != 0 {
            self.port(self.l(0x43878c)?)?;
        }
        self.trap(0xa89c, vec![W(0)])?;
        self.trap(0xa89b, vec![W(1), W(1)])?;
        self.trap(0xaa0a, vec![L(self.l(0x4358de)?)])?;
        self.trap(
            0xa893,
            vec![W(self.w(0x44292e)?), W(self.w(0x442930)?.wrapping_sub(1))],
        )?;
        self.trap(0xa891, vec![W(self.w(0x44292e)?), W(self.w(0x44292c)?)])?;
        self.trap(
            0xa891,
            vec![W(self.w(0x442932)?.wrapping_sub(1)), W(self.w(0x44292c)?)],
        )?;
        self.trap(0xaa0a, vec![L(self.l(0x4358e2)?)])?;
        self.trap(
            0xa893,
            vec![
                W(self.w(0x442932)?.wrapping_sub(1)),
                W(self.w(0x44292c)?.wrapping_add(1)),
            ],
        )?;
        self.trap(
            0xa891,
            vec![
                W(self.w(0x442932)?.wrapping_sub(1)),
                W(self.w(0x442930)?.wrapping_sub(1)),
            ],
        )?;
        self.trap(
            0xa891,
            vec![
                W(self.w(0x44292e)?.wrapping_add(1)),
                W(self.w(0x442930)?.wrapping_sub(1)),
            ],
        )?;
        self.port(self.l(frame.wrapping_sub(22))?)?;
        self.trap(0xa899, vec![P(frame.wrapping_sub(18))])?;
        self.full(0);
        Ok(())
    }
    fn question_port(&mut self, dialog: u32) -> Result<(), RuleError> {
        self.port(dialog)?;
        self.trap(0xaa0b, vec![L(self.l(0x4358ce)?)])?;
        self.trap(0xa887, vec![W(self.w(0x402f44)?)])?;
        self.trap(0xa862, vec![L(69)])?;
        self.trap(
            0xa91b,
            vec![
                L(dialog),
                W(self
                    .w(0x4395a4)?
                    .wrapping_add(322)
                    .wrapping_add(self.w(0x40856c)?)),
                W(self
                    .w(0x4395a2)?
                    .wrapping_add(301)
                    .wrapping_add(self.w(0x40856e)?)
                    .wrapping_sub(self.w(0x439582)?.wrapping_shl(5))),
                B(0),
            ],
        )?;
        self.trap(0xa915, vec![L(dialog)])?;
        Ok(())
    }
    fn question_finish(
        &mut self,
        frame: u32,
        base: u32,
        port_offset: u32,
        dialog: u32,
        third: bool,
        keyboard: u8,
    ) -> Result<(), RuleError> {
        self.trap(0xaa1c, vec![L(self.l(0x43952c)?)])?;
        self.trap(
            0xa98d,
            vec![
                L(dialog),
                W(self.w(0x4434b2)?),
                P(0x4434b0),
                P(0x4434dc),
                P(0x44292c),
            ],
        )?;
        self.down_button(base.wrapping_sub(10))?;
        self.lw(0x4434ba, self.l(self.l(0x3fe302)?.wrapping_add(16))?)?;
        self.lw(0x4434be, self.l(self.l(0x3fe302)?.wrapping_add(20))?)?;
        self.trap(0xa983, vec![L(dialog)])?;
        self.port(self.l(frame.wrapping_sub(port_offset))?)?;
        let child = base.wrapping_sub(16);
        if self.w(0x438776)? == 0 {
            self.lw(child.wrapping_add(12), 0)?;
            self.lw(child.wrapping_add(8), 0x00010000)?;
        } else {
            self.lw(
                child.wrapping_add(12),
                if third { 0x00010000 } else { 0x00010001 },
            )?;
            self.lw(child.wrapping_add(8), 0)?;
        }
        self.selection(child, false)?;
        if !self.active() {
            return Ok(());
        }
        if !third || keyboard != 0 {
            self.low(self.w(0x4434b2)?.wrapping_sub(1));
        } else if self.w(0x4434b2)? == 1 {
            self.ww(0x4434b2, 2)?;
            self.full(1);
        } else {
            self.ww(0x4434b2, 3)?;
            self.full(2);
        }
        Ok(())
    }
    fn question_events(
        &mut self,
        frame: u32,
        third: bool,
        yes: u8,
        no: u8,
    ) -> Result<u8, RuleError> {
        let mut keyboard = 0;
        loop {
            self.step()?;
            self.trap(0xa860, vec![W(u16::MAX), P(0x44720a), L(0), L(0)])?;
            if self.trap(0xa97f, vec![P(0x44720a)])? as u8 != 0
                && self.trap(
                    0xa980,
                    vec![P(0x44720a), P(frame.wrapping_sub(4)), P(0x4434b2)],
                )? as u8
                    != 0
            {
                return Ok(keyboard);
            }
            if third {
                keyboard = 0;
            }
            if self.w(0x44720a)? != 3 {
                continue;
            }
            keyboard = self.trap(0xa858, vec![L(self.l(0x44720c)?), L(255)])? as u8;
            if third && keyboard == 27 {
                self.ww(0x4434b2, 1)?;
                return Ok(keyboard);
            }
            if keyboard == no {
                self.ww(0x4434b2, if third { 2 } else { 1 })?;
                return Ok(keyboard);
            }
            if keyboard == yes {
                self.ww(0x4434b2, if third { 3 } else { 2 })?;
                return Ok(keyboard);
            }
        }
    }
    fn question2(&mut self, frame: u32) -> Result<(), RuleError> {
        let first = self.w(frame.wrapping_add(8))?;
        let base = frame.wrapping_sub(548);
        let left = frame.wrapping_sub(260);
        let right = frame.wrapping_sub(516);
        let mut kind = 200;
        let mut yes = b'y';
        let mut no = b'n';
        self.runtime(0x13e13c)?;
        self.ww(base.wrapping_sub(2), 20005)?;
        self.sound(20005)?;
        self.trap(0xa874, vec![P(frame.wrapping_sub(520))])?;
        self.trap(0xaa1c, vec![L(self.l(0x439530)?)])?;
        if first != 0 {
            self.lib(0x400178, vec![P(left), L(0x00010003)])?;
            self.lib(0x400178, vec![P(right), L(0x00010003)])?;
            self.filename(base.wrapping_sub(12), 0x401990, None, false)?;
            let mut handle = self.open_file(0x40198c, 0x4389a4)?;
            let wide = handle == 0;
            if wide {
                self.filename(base.wrapping_sub(12), 0x401982, None, false)?;
                handle = self.open_file(0x40198c, 0x4389a4)?;
                if handle == 0 {
                    self.scratch(base.wrapping_sub(10), 134)?;
                    if !self.active() {
                        return Ok(());
                    }
                }
            }
            for (number, destination) in [(Some(first), left), (None, right)] {
                let number = match number {
                    Some(number) => number,
                    None => self.w(frame.wrapping_add(10))?,
                };
                self.lib(0x400038, vec![W(number)])?;
                let absolute = self.native_d0()? as u16 as i16;
                let offset = if wide {
                    (i32::from(absolute) as u32).wrapping_shl(8)
                } else {
                    absolute.wrapping_mul(25) as i32 as u32
                };
                self.lib(0x400078, vec![W(0), L(offset), L(handle)])?;
                self.read_file(handle, if wide { 256 } else { 25 }, destination)?;
            }
            self.lib(0x400068, vec![L(handle)])?;
            self.trap(0xa98b, vec![P(left), P(right), P(0x401998), P(0x401998)])?;
            kind = 136;
        }
        let dialog = self.trap(0xa97c, vec![W(kind), L(0), L(u32::MAX)])?;
        self.question_port(dialog)?;
        if first != 0 {
            self.lib(0x400040, vec![W(u16::from(self.b(left.wrapping_add(1))?))])?;
            yes = self.native_d0()? as u8;
            self.lib(0x400040, vec![W(u16::from(self.b(right.wrapping_add(1))?))])?;
            no = self.native_d0()? as u8;
        }
        for opcode in [0xa981, 0xa922, 0xa923] {
            self.trap(opcode, vec![L(dialog)])?;
        }
        self.lw(base.wrapping_sub(4), 0x006a006a)?;
        self.prompt(base.wrapping_sub(12), 0x12382a)?;
        if !self.active() {
            return Ok(());
        }
        self.flush()?;
        let keyboard = self.question_events(frame, false, yes, no)?;
        self.question_finish(frame, base, 520, dialog, false, keyboard)
    }
    fn question3(&mut self, frame: u32) -> Result<(), RuleError> {
        let left = self.l(frame.wrapping_add(8))?;
        let right = self.l(frame.wrapping_add(12))?;
        let base = frame.wrapping_sub(40);
        self.lw(frame.wrapping_sub(12), 0)?;
        self.runtime(0x13e13c)?;
        self.ww(base.wrapping_sub(2), 20005)?;
        self.sound(20005)?;
        self.trap(0xa874, vec![P(frame.wrapping_sub(8))])?;
        self.trap(0xaa1c, vec![L(self.l(0x439530)?)])?;
        if left != 0 {
            self.trap(0xa98b, vec![L(left), L(right), P(0x401998), P(0x401998)])?;
        }
        let dialog = self.trap(
            0xa97c,
            vec![W(if left == 0 { 200 } else { 136 }), L(0), L(u32::MAX)],
        )?;
        self.question_port(dialog)?;
        self.lib(0x400040, vec![W(u16::from(self.b(left.wrapping_add(1))?))])?;
        let yes = self.native_d0()? as u8;
        self.lib(0x400040, vec![W(u16::from(self.b(right.wrapping_add(1))?))])?;
        let no = self.native_d0()? as u8;
        for opcode in [0xa981, 0xa922, 0xa923] {
            self.trap(opcode, vec![L(dialog)])?;
        }
        self.lw(base.wrapping_sub(8), 0x006a006a)?;
        self.prompt(base.wrapping_sub(16), 0x12382a)?;
        if !self.active() {
            return Ok(());
        }
        self.flush()?;
        let keyboard = self.question_events(frame, true, yes, no)?;
        self.question_finish(frame, base, 8, dialog, true, keyboard)
    }
}
impl<S: SessionControlServices + ?Sized> Execution<'_, S> {
    fn get_word(&mut self, frame: u32) -> Result<(), RuleError> {
        let child = menus::get_word(
            &self.plan.state.memory,
            frame,
            self.limits.storage(),
            self.services,
        )?;
        self.plan.state.memory = child.memory;
        self.plan.flow = child.flow;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(SessionControlEffect::Menu));
        self.d0 = child.native_return;
        if !self.active() {
            self.plan.native_return = PromptReturn::NoReturn;
        }
        Ok(())
    }
    fn registration_bits(
        &mut self,
        accumulator: u32,
        source: u32,
        divisors: [i16; 3],
        increments: [i16; 3],
        signed: bool,
    ) -> Result<(), RuleError> {
        self.ww(0x4434ac, 1)?;
        loop {
            self.step()?;
            let length = if signed {
                self.sb(source)?
            } else {
                i16::from(self.b(source)?)
            };
            if self.sw(0x4434ac)? > length {
                break;
            }
            let character = if signed {
                self.sb(index(source, self.sw(0x4434ac)?, 1))?
            } else {
                i16::from(self.b(index(source, self.sw(0x4434ac)?, 1))?)
            };
            let test = character % divisors[0] + increments[0];
            let set = self.trap(0xa85d, vec![P(accumulator), L(test as i32 as u32)])? as u8 != 0;
            let character = if signed {
                self.sb(index(source, self.sw(0x4434ac)?, 1))?
            } else {
                i16::from(self.b(index(source, self.sw(0x4434ac)?, 1))?)
            };
            let (opcode, divisor, increment) = if set {
                (0xa85f, divisors[1], increments[1])
            } else {
                (0xa85e, divisors[2], increments[2])
            };
            let bit = character % divisor + increment;
            self.trap(opcode, vec![P(accumulator), L(bit as i32 as u32)])?;
            self.addw(0x4434ac, 1)?;
        }
        Ok(())
    }
    fn registered_name(&mut self, frame: u32) -> Result<(), RuleError> {
        self.ww(frame.wrapping_add(8), 0)?;
        self.lib(0x400180, vec![P(0x438aa4)])?;
        for i in 0..40_u32 {
            self.step()?;
            self.lib(0x400040, vec![W(self.sb(0x438aa4 + i)? as u16)])?;
            self.byte(0x438aa4 + i, self.native_d0()? as u8)?;
        }
        let mut matched = false;
        for right in [
            0x4015ce, 0x4015b6, 0x4015ac, 0x401598, 0x401592, 0x40158a, 0x401580, 0x401572,
            0x401566, 0x40155c, 0x40154e, 0x401544, 0x40153c, 0x401532, 0x40152a, 0x401522,
            0x40151a, 0x401512, 0x401544, 0x401508, 0x4014fc, 0x4014f2, 0x4014e8, 0x4014e2,
            0x4014d4, 0x4014cc, 0x4014bc, 0x4014ae,
        ] {
            self.lib(0x4000a8, vec![P(right), P(0x438aa4)])?;
            matched |= self.native_d0()? as u16 == 0;
        }
        self.plan
            .effects
            .push(SessionControlEffect::RegisteredAdmission {
                stage: RegistrationStage::ReservedName,
                native_match: matched,
            });
        self.lib(0x400188, vec![P(0x438aa4)])?;
        self.full(0);
        Ok(())
    }
    fn reg_scen(&mut self, frame: u32) -> Result<(), RuleError> {
        if crate::recovered_verification::doreg() != 0 {
            self.step()?;
            self.plan
                .effects
                .push(SessionControlEffect::RegisteredByPolicy {
                    stage: RegistrationStage::ScenarioCode,
                });
            self.full(1);
            return Ok(());
        }
        let base = frame.wrapping_sub(286);
        let initial = frame.wrapping_sub(264);
        let accumulator = frame.wrapping_sub(4);
        let local_name = frame.wrapping_sub(260);
        self.ww(frame.wrapping_sub(266), 180)?;
        self.lw(initial, 0)?;
        if self.w(0x442560)? == 8 || self.sw(0x442560)? > 9 {
            self.lw(initial, self.l(0x438598)?)?;
            self.trap(0xa85e, vec![P(initial), L(6)])?;
            let divisor = self.sw(0x442560)?.wrapping_mul(666) as i32;
            let quotient =
                (self.l(initial)? as i32)
                    .checked_div(divisor)
                    .ok_or(RuleError::Unsupported {
                        action: "session_control.regscen",
                        dependency: "the reached native signed division traps or overflows",
                    })?;
            self.lw(initial, quotient as u32)?;
        }
        self.flash(base.wrapping_sub(20), 0x402406, 0x001e0028, 0xffff0000)?;
        self.get_word(base.wrapping_sub(20))?;
        if !self.active() {
            return Ok(());
        }
        self.flash(base.wrapping_sub(32), 0x40250d, 0x001e0028, 0xffff0000)?;
        self.lib(0x400090, vec![P(0x438aa4), P(0x438acc)])?;
        self.filename(base.wrapping_sub(44), 0x4023fe, None, false)?;
        let handle = self.open_file(0x40250e, 0x4389a4)?;
        self.lw(0x4434a6, handle)?;
        if handle == 0 {
            self.scratch(base.wrapping_sub(10), 700)?;
            if !self.active() {
                return Ok(());
            }
        }
        self.lib(0x400078, vec![W(0), L(20), L(self.l(0x4434a6)?)])?;
        self.read_file(self.l(0x4434a6)?, 20, 0x438010)?;
        self.read_file(self.l(0x4434a6)?, 20, 0x438110)?;
        self.lib(0x400068, vec![L(self.l(0x4434a6)?)])?;
        self.ww(0x4434aa, 0)?;
        while self.sw(0x4434aa)? < 20 {
            self.step()?;
            let i = self.sw(0x4434aa)?;
            self.byte(
                index(0x435782, i, 1),
                self.b(index(0x435782, i, 1))?
                    .wrapping_sub(self.b(index(0x438010, i, 1))?),
            )?;
            self.byte(
                index(0x435682, self.sw(0x4434aa)?, 1),
                self.b(index(0x435682, self.sw(0x4434aa)?, 1))?
                    .wrapping_sub(self.b(index(0x435782, self.sw(0x4434aa)?, 1))?),
            )?;
            self.addw(0x4434aa, 1)?;
        }
        self.lw(initial, self.l(0x43859c)?)?;
        self.lw(accumulator, self.l(0x438598)?)?;
        self.lib(0x400088, vec![P(0x435682)])?;
        self.ww(0x4434b6, self.native_d0()? as u16)?;
        self.lib(0x400088, vec![P(0x435782)])?;
        if (self.native_d0()? as u16 as i16) < self.sw(0x4434b6)? {
            self.lib(0x400088, vec![P(0x435782)])?;
            self.ww(0x4434b6, self.native_d0()? as u16)?;
        }
        self.ww(0x4434aa, 0)?;
        while self.sw(0x4434aa)? < self.sw(0x4434b6)? {
            self.step()?;
            self.lw(
                accumulator,
                self.l(accumulator)?.wrapping_add(u32::from(self.b(index(
                    0x435682,
                    self.sw(0x4434aa)?,
                    1,
                ))?)),
            )?;
            self.lw(
                accumulator,
                self.l(accumulator)?.wrapping_sub(u32::from(self.b(index(
                    0x435782,
                    self.sw(0x4434aa)?,
                    1,
                ))?)),
            )?;
            self.addw(0x4434aa, 1)?;
        }
        self.lib(0x400160, vec![P(0x438acc), P(0x4471dc)])?;
        self.lw(
            accumulator,
            self.l(accumulator)?.wrapping_add(self.l(0x4471dc)?),
        )?;
        self.trap(
            0xa946,
            vec![L(self.l(0x43870e)?), W(self.w(0x442560)?), P(0x479ed8)],
        )?;
        self.lib(0x400160, vec![P(0x479ed8), P(0x4471dc)])?;
        self.lw(
            accumulator,
            self.l(accumulator)?.wrapping_add(self.l(0x4471dc)?),
        )?;
        let ticks = self.trap(0xa975, vec![])?;
        self.lw(frame.wrapping_sub(270), ticks)?;
        self.lib(0x400168, vec![L(self.l(0x43859c)?), P(0x479ed8)])?;
        self.lib(0x400188, vec![P(0x435682)])?;
        self.lib(0x400188, vec![P(0x435782)])?;
        let d3 = 32_i16.wrapping_sub(self.sb(0x438acc)? % 4);
        let d4 = 31_i16.wrapping_sub(i16::from(self.b(0x435682)?) % 3);
        let d5 = 30_i16.wrapping_sub(i16::from(self.b(0x435782)?) % 5);
        self.ww(0x4434ac, 1)?;
        while self.sw(0x4434ac)? <= self.sb(0x438acc)? {
            self.step()?;
            self.lib(
                0x400040,
                vec![W(self.sb(index(0x438acc, self.sw(0x4434ac)?, 1))? as u16)],
            )?;
            self.byte(
                index(0x438acc, self.sw(0x4434ac)?, 1),
                self.native_d0()? as u8,
            )?;
            self.addw(0x4434ac, 1)?;
        }
        self.registration_bits(accumulator, 0x438acc, [d4, d4, d4], [0, 2, 1], true)?;
        self.registration_bits(accumulator, 0x479ed8, [d3, d3, d3], [2, 1, 0], false)?;
        self.registration_bits(accumulator, 0x435682, [d5, d5, d5], [2, 0, 1], false)?;
        self.registration_bits(accumulator, 0x435782, [d3, d4, d5], [0, 0, 0], false)?;
        self.trap(
            0xa946,
            vec![L(self.l(0x43870e)?), W(self.w(0x442560)?), P(0x479ed8)],
        )?;
        self.lib(0x400180, vec![P(0x479ed8)])?;
        self.filename(base.wrapping_sub(16), 0x479ed8, None, false)?;
        let handle = self.open_file(0x40250e, 0x4389a4)?;
        self.lw(0x4434a6, handle)?;
        if handle == 0 {
            self.scratch(base.wrapping_sub(10), 700)?;
            if !self.active() {
                return Ok(());
            }
        }
        self.lib(0x400078, vec![W(0), L(20), L(self.l(0x4434a6)?)])?;
        self.read_file(self.l(0x4434a6)?, 20, 0x435682)?;
        self.read_file(self.l(0x4434a6)?, 20, 0x435782)?;
        self.lib(0x400068, vec![L(self.l(0x4434a6)?)])?;
        let ticks = self.trap(0xa975, vec![])?;
        self.lw(frame.wrapping_sub(274), ticks)?;
        if (self.l(accumulator)? as i32) < 0 {
            self.lw(
                accumulator,
                (self.l(accumulator)? as i32).wrapping_neg() as u32,
            )?;
        }
        self.flash(base.wrapping_sub(20), 0x4023c8, 0x001e0028, 0xffff0000)?;
        self.get_word(base.wrapping_sub(20))?;
        if !self.active() {
            return Ok(());
        }
        self.flash(base.wrapping_sub(32), 0x40250d, 0x001e0028, 0xffff0000)?;
        self.lib(0x400160, vec![P(0x438aa4), P(0x4471dc)])?;
        self.lib(0x400090, vec![P(0x4385a0), P(local_name)])?;
        self.ww(0x4434ac, 1)?;
        while self.sw(0x4434ac)? <= i16::from(self.b(local_name)?) {
            self.step()?;
            self.lib(
                0x400040,
                vec![W(u16::from(self.b(index(
                    local_name,
                    self.sw(0x4434ac)?,
                    1,
                ))?))],
            )?;
            self.byte(
                index(local_name, self.sw(0x4434ac)?, 1),
                self.native_d0()? as u8,
            )?;
            self.addw(0x4434ac, 1)?;
        }
        self.lib(0x400180, vec![P(local_name)])?;
        self.lib(0x400180, vec![P(0x438acc)])?;
        self.lib(0x4000a8, vec![P(0x438acc), P(local_name)])?;
        let compare = self.native_d0()? as u16 as i16;
        self.lw(
            0x4471dc,
            self.l(0x4471dc)?.wrapping_add(compare as i32 as u32),
        )?;
        self.plan
            .effects
            .push(SessionControlEffect::RegisteredAdmission {
                stage: RegistrationStage::ScenarioCode,
                native_match: self.l(0x4471dc)? == self.l(accumulator)?,
            });
        self.flush()?;
        self.trap(0xa860, vec![W(u16::MAX), P(0x44720a), L(0), L(0)])?;
        if self.w(frame.wrapping_add(8))? == 0 {
            for (n, (pointer, mode)) in [
                (0x40236a, 1101),
                (0x402308, 1100),
                (0x4022a8, 1102),
                (0x40227c, 1102),
            ]
            .into_iter()
            .enumerate()
            {
                self.flash(
                    base.wrapping_sub(20 + n as u32 * 12),
                    pointer,
                    0x00320096,
                    mode,
                )?;
            }
            self.lib(0x4000a8, vec![P(0x438acc), P(local_name)])?;
            self.plan
                .effects
                .push(SessionControlEffect::RegisteredAdmission {
                    stage: RegistrationStage::ConfirmationName,
                    native_match: self.native_d0()? as u16 == 0,
                });
        }
        self.registered_name(base.wrapping_sub(10))?;
        self.byte(0x442604, 98)?;
        self.full(1);
        Ok(())
    }
}
impl<S: SessionControlServices + ?Sized> Execution<'_, S> {
    fn save_open(
        &mut self,
        base: u32,
        name: u32,
        slot: Option<u16>,
        error: Option<i16>,
        metadata_retained: bool,
    ) -> Result<u32, RuleError> {
        let filename_frame = base.wrapping_sub(if slot.is_some() {
            14
        } else if metadata_retained {
            28
        } else {
            12
        });
        self.filename(filename_frame, name, slot, slot.is_none())?;
        let handle = self.open_file(if slot.is_some() { 0x4017ac } else { 0x4017b0 }, 0x4389a4)?;
        if handle == 0
            && let Some(error) = error
        {
            self.scratch(base.wrapping_sub(10), error)?;
        }
        Ok(handle)
    }
    fn save_copy(
        &mut self,
        source: u32,
        destination: u32,
        header: (u32, u32),
        tails: &[(u32, u32)],
    ) -> Result<(), RuleError> {
        loop {
            self.step()?;
            if self.read_file(source, header.1, header.0)? != 1 {
                break;
            }
            for (address, size) in tails {
                self.read_file(source, *size, *address)?;
            }
            self.write_file(destination, header.1, 1, header.0)?;
            for (address, size) in tails {
                self.write_file(destination, *size, 1, *address)?;
            }
        }
        Ok(())
    }
    fn save_close(&mut self, base: u32, source: u32, destination: u32) -> Result<(), RuleError> {
        self.lib(0x400068, vec![L(source)])?;
        self.lib(0x400068, vec![L(destination)])?;
        self.metadata(base.wrapping_sub(24), 0x4389a4, 0x4017a6)
    }
    fn save(&mut self, frame: u32) -> Result<(), RuleError> {
        let base = frame.wrapping_sub(22);
        let resource_count = self.trap(0xa99c, vec![L(0x4e455743)])? as u16;
        let count = resource_count.wrapping_sub(1);
        if count != self.w(0x442560)? && count != 0 {
            self.warn(base.wrapping_sub(10), 92)?;
            self.scratch(base.wrapping_sub(12), -501)?;
            if !self.active() {
                return Ok(());
            }
        }
        self.byte(0x4386a9, 0)?;
        self.lib(
            0x400148,
            vec![W(0), P(0x479ed8), P(frame.wrapping_sub(6)), P(0x4427b6)],
        )?;
        if (self.l(0x4427b6)? as i32) < 1_000_000 {
            self.byte(0x4386a9, 1)?;
        }
        self.save_land(base.wrapping_sub(12))?;
        if !self.active() {
            return Ok(());
        }
        if self.b(0x442756)? != 0 {
            self.save_shop(base.wrapping_sub(8))?;
            if !self.active() {
                return Ok(());
            }
        }
        self.runtime(0x13a498)?;
        self.ww(base.wrapping_sub(2), self.w(frame.wrapping_add(8))?)?;
        self.field(base.wrapping_sub(10), 0x122e24)?;
        if !self.active() {
            return Ok(());
        }
        let slot = self.result_word()?;
        self.center(base.wrapping_sub(12))?;
        if slot == 0 {
            if let Some(d0) = self.d0 {
                self.full(d0);
            }
            return Ok(());
        }
        self.ww(0x44255e, u16::from(self.b(0x44274c)?))?;
        self.flash(base.wrapping_sub(20), 0x4017fc, 0x00190096, 0xffff0000)?;
        self.byte(0x44274c, 0)?;
        while self.b(0x44274c)? < 3 {
            self.step()?;
            for (n, name) in [
                0x4017f4, 0x4017ec, 0x4017e4, 0x4017dc, 0x4017d4, 0x4017cc, 0x4017c4, 0x4017ba,
            ]
            .into_iter()
            .enumerate()
            {
                self.filename(
                    base.wrapping_sub(14 + n as u32 * 10),
                    name,
                    Some(slot),
                    false,
                )?;
                self.lib(0x400058, vec![P(0x4389a4)])?;
            }
            self.byte(0x44274c, self.b(0x44274c)?.wrapping_add(1))?;
        }
        self.byte(0x44274c, 0)?;
        while self.b(0x44274c)? < 3 {
            self.step()?;
            let source = self.save_open(base, 0x4017b4, None, None, false)?;
            if !self.active() {
                return Ok(());
            }
            if source == 0 {
                self.byte(0x44274c, self.b(0x44274c)?.wrapping_add(1))?;
                continue;
            }
            let destination = self.save_open(base, 0x4017ba, Some(slot), Some(139), false)?;
            if !self.active() {
                return Ok(());
            }
            self.save_copy(source, destination, (0x439d14, 40), &[])?;
            self.save_close(base, source, destination)?;

            let source = self.save_open(base, 0x4017a2, None, Some(144), true)?;
            if !self.active() {
                return Ok(());
            }
            let destination = self.save_open(base, 0x4017dc, Some(slot), Some(143), false)?;
            if !self.active() {
                return Ok(());
            }
            let tails: Vec<_> = (0..4).map(|i| (0x438af4 + i * 256, 80)).collect();
            self.save_copy(source, destination, (0x443432, 106), &tails)?;
            self.save_close(base, source, destination)?;

            let source = self.save_open(base, 0x40179e, None, Some(146), true)?;
            if !self.active() {
                return Ok(());
            }
            let destination = self.save_open(base, 0x4017d4, Some(slot), Some(145), false)?;
            if !self.active() {
                return Ok(());
            }
            let tails: Vec<_> = (0..9).map(|i| (0x438af4 + i * 256, 40)).collect();
            self.save_copy(source, destination, (0x443396, 156), &tails)?;
            self.save_close(base, source, destination)?;
            let mut checksum = self.l(0x442764)?;
            for a in [0x442768, 0x442778, 0x44277c, 0x442774] {
                checksum = checksum.wrapping_add(self.l(a)?);
            }
            self.lw(frame.wrapping_sub(4), checksum)?;
            let mut member = 0_i16;
            while member <= self.sb(0x44273a)? {
                self.step()?;
                self.ww(base.wrapping_sub(2), member as u16)?;
                self.verify(member as u16)?;
                member = member.wrapping_add(1);
            }
            let handle = self.save_open(base, 0x4017c4, Some(slot), Some(147), false)?;
            if !self.active() {
                return Ok(());
            }
            for (address, size, count) in [
                (0x435502, 2, 1),
                (0x442562, 20, 1),
                (0x442576, 2, 5),
                (0x442580, 1, 438),
                (0x442736, 1, 24),
                (0x442742, 1, 29),
                (0x44275c, 4, 14),
                (0x442790, 4, 2),
                (0x448aa0, 0x498c, 1),
                (0x4427c6, 18, 1),
                (0x440edc, 0x1680, 1),
                (0x43788a, 480, 1),
                (0x42e2ec, 0x3390, 1),
                (0x438acc, 40, 1),
                (frame.wrapping_sub(4), 4, 1),
                (0x435500, 2, 1),
                (0x40857a, 2, 1),
                (0x408578, 2, 1),
                (0x402f42, 1, 1),
                (0x4354fe, 2, 1),
                (0x44255e, 2, 2),
                (0x42dbe4, 800, 1),
                (0x4285f4, 22000, 1),
            ] {
                self.write_file(handle, size, count, address)?;
            }
            self.lib(0x400068, vec![L(handle)])?;
            self.metadata(base.wrapping_sub(388), 0x4389a4, 0x4017a6)?;
            self.filename(base.wrapping_sub(392), 0x40181c, None, true)?;
            let source = self.open_file(0x4017b0, 0x4389a4)?;
            if source == 0 {
                self.scratch(base.wrapping_sub(10), 150)?;
                if !self.active() {
                    return Ok(());
                }
            }
            let destination = self.save_open(base, 0x4017f4, Some(slot), Some(151), false)?;
            if !self.active() {
                return Ok(());
            }
            self.save_copy(
                source,
                destination,
                (0x478e42, 0x1068),
                &[
                    (0x443086, 0x284),
                    (0x4358e6, 0x1fa4),
                    (0x4085b8, 0x3f48),
                    (0x40c500, 0x3f48),
                    (0x414390, 0x7e90),
                ],
            )?;
            self.save_close(base, source, destination)?;

            let source = self.save_open(base, 0x40179a, None, Some(152), true)?;
            if !self.active() {
                return Ok(());
            }
            let destination = self.save_open(base, 0x4017e4, Some(slot), Some(153), false)?;
            if !self.active() {
                return Ok(());
            }
            loop {
                self.step()?;
                if self.read_file(source, 0xbba, 0x478288)? == 0 {
                    break;
                }
                if self.write_file(destination, 0xbba, 1, 0x478288)? == 0 {
                    self.scratch(base.wrapping_sub(10), 154)?;
                    if !self.active() {
                        return Ok(());
                    }
                }
            }
            self.save_close(base, source, destination)?;
            self.byte(0x44274c, self.b(0x44274c)?.wrapping_add(1))?;
        }
        self.byte(0x44274c, self.b(0x44255f)?)?;
        self.load_land(base.wrapping_sub(12))?;
        if !self.active() {
            return Ok(());
        }
        self.flash(base.wrapping_sub(24), 0x401799, 0x00190096, 0xffff0000)?;
        self.byte(0x4386a9, 0)?;
        self.plan.native_return = self.d0.map_or(PromptReturn::Unchanged, PromptReturn::Full);
        Ok(())
    }
}
fn execution<'a, S: SessionControlServices + ?Sized>(
    state: &PromptControlState,
    limits: SessionControlLimits,
    services: &'a mut S,
) -> Execution<'a, S> {
    Execution { plan: SessionControlPlan { state: state.clone(), effects: vec![], flow: LoadFlow::Returned, native_return: PromptReturn::Unchanged, qualification: SessionControlQualification::SourcePortWithObservedBoundariesProjectedChildrenAndRegisteredPolicy }, services, limits, remaining: limits.maximum_steps, initial_draws: state.random.draws, d0: None }
}
macro_rules! entry {
    ($name:ident, $body:ident) => {
        pub fn $name(
            state: &PromptControlState,
            frame: u32,
            limits: SessionControlLimits,
            services: &mut (impl SessionControlServices + ?Sized),
        ) -> Result<SessionControlPlan, RuleError> {
            let mut e = execution(state, limits, services);
            e.$body(frame)?;
            Ok(e.plan)
        }
    };
}
entry!(quit_game, quit_game);
entry!(register_scenario, reg_scen);
entry!(save, save);
entry!(question2, question2);
entry!(question3, question3);
pub fn generate(
    state: &PromptControlState,
    frame: u32,
    registers: Option<GenerationRegisters>,
    limits: SessionControlLimits,
    services: &mut (impl SessionControlServices + ?Sized),
) -> Result<SessionControlPlan, RuleError> {
    let mut e = execution(state, limits, services);
    e.generate(frame, registers)?;
    Ok(e.plan)
}
fn warrior(member: u16) -> u32 {
    0x448aa0_u32.wrapping_add(u32::from(member) * 3138)
}
fn c_string(memory: &RenderingMemory, address: u32, limit: usize) -> Result<Vec<u8>, RuleError> {
    let mut bytes = Vec::new();
    for i in 0..limit {
        let byte = memory.byte(address.wrapping_add(i as u32))?;
        bytes.push(byte);
        if byte == 0 {
            return Ok(bytes);
        }
    }
    Err(RuleError::Unsupported {
        action: "session_control.string",
        dependency: "the reached C string exceeds the supplied capacity",
    })
}
fn half(value: i16) -> i16 {
    value.wrapping_add(((value as u16) >> 15) as i16) >> 1
}
fn index(base: u32, value: i16, stride: i32) -> u32 {
    base.wrapping_add(i32::from(value).wrapping_mul(stride) as u32)
}
struct Clock<'a, 'b, S: ?Sized> {
    execution: &'a mut Execution<'b, S>,
    remaining: usize,
}
impl<S: SessionControlServices + ?Sized> TimingServices for Clock<'_, '_, S> {
    fn tick_count(&mut self) -> Result<u32, RuleError> {
        self.remaining = self
            .remaining
            .checked_sub(1)
            .ok_or(RuleError::Unsupported {
                action: "session_control.delay",
                dependency: "the supplied tick-poll budget is exhausted",
            })?;
        self.execution.trap(0xa975, vec![])
    }
    fn button(&mut self) -> Result<bool, RuleError> {
        Err(RuleError::Invalid(
            "the delay child does not query Button".into(),
        ))
    }
}
