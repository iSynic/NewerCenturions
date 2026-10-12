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
struct Execution<'a, S> {
    plan: SessionControlPlan,
    services: &'a mut S,
    limits: SessionControlLimits,
    remaining: u32,
    initial_draws: u64,
    d0: Option<u32>,
}
impl<S: SessionControlServices> Execution<'_, S> {
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
impl<S: SessionControlServices> Execution<'_, S> {
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
impl<S: SessionControlServices> Execution<'_, S> {
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
impl<S: SessionControlServices> Execution<'_, S> {
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
impl<S: SessionControlServices> Execution<'_, S> {
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
fn execution<'a, S: SessionControlServices>(
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
            services: &mut impl SessionControlServices,
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
    services: &mut impl SessionControlServices,
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
struct Clock<'a, 'b, S> {
    execution: &'a mut Execution<'b, S>,
    remaining: usize,
}
impl<S: SessionControlServices> TimingServices for Clock<'_, '_, S> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recovered_audio::{
        AudioContext, AudioObservation, AudioReply, AudioRequest, SoundCommand,
    };
    use crate::recovered_character_panels::{
        CharacterPanelObservation, CharacterPanelRequest, CharacterPanelServices,
    };
    use crate::recovered_combat_center::CombatCenterServices;
    use crate::recovered_combat_ui::{CombatUiObservation, CombatUiRequest, CombatUiServices};
    use crate::recovered_equipment::EquipmentServices;
    use crate::recovered_equipment_ui::EquipmentUiServices;
    use crate::recovered_gamma::{
        GammaError, GammaMemory, GammaObservation, GammaRequest, GammaServices,
    };
    use crate::recovered_gamma_control::{GammaControlRequest, GammaControlServices};
    use crate::recovered_interface::{InterfaceServices, Point};
    use crate::recovered_inventory_edit::{
        InventoryEditObservation, InventoryEditRequest, InventoryEditServices,
    };
    use crate::recovered_inventory_rules::{InventoryServices, SaneRequest};
    use crate::recovered_item_ui::{ItemUiReply, ItemUiRequest, ItemUiServices};
    use crate::recovered_load_control::{
        LoadObservation, LoadRequest, LoadServices, VersionInputs,
    };
    use crate::recovered_prompt_control::tests as base;
    use crate::recovered_rendering::{RenderingObservation, RenderingRequest, RenderingServices};
    use crate::recovered_runtime::RuntimeObservation;
    use crate::recovered_storage::{StorageObservation, StorageRequest, StorageServices};
    use crate::recovered_text::NumericServices;
    use crate::recovered_world_display::{
        WorldDisplayObservation, WorldDisplayRequest, WorldDisplayServices,
    };
    use std::cell::{Cell, RefCell};
    use std::collections::VecDeque;

    use crate::recovered_bodies::BodyState;
    use crate::recovered_character_ui::{
        CharacterUiObservation, CharacterUiRequest, CharacterUiServices,
    };
    use crate::recovered_combat_panel::{
        CombatPanelObservation, CombatPanelRequest, CombatPanelServices,
    };
    use crate::recovered_detail_ui::{DetailReply, DetailRequest, DetailServices};
    use crate::recovered_drawing::{BuildingWorld, DrawingServices, PictureObservation};
    use crate::recovered_field_control::FilePrepServices;
    use crate::recovered_geometry::BodyRect;
    use crate::recovered_prompt_control::PromptControlServices;
    use crate::recovered_runtime::{RuntimeReply, RuntimeRequest};
    use crate::recovered_scenario_menus::ScenarioMenuServices;
    use crate::recovered_startup::{PreferenceInputs, StartupObservation, StartupRequest};
    use crate::recovered_status_panels::{
        StatusPanelObservation, StatusPanelRequest, StatusPanelServices,
    };
    use crate::recovered_text::PascalString;

    impl RenderingServices for Observed {
        fn observe(
            &self,
            _: &RenderingRequest,
            _: &RenderingMemory,
        ) -> Result<RenderingObservation, RuleError> {
            Err(unused())
        }
    }
    impl CombatUiServices for Observed {
        fn observe(
            &self,
            _: &CombatUiRequest,
            _: &RenderingMemory,
        ) -> Result<CombatUiObservation, RuleError> {
            Err(unused())
        }
    }
    impl WorldDisplayServices for Observed {
        fn observe(
            &mut self,
            _: &WorldDisplayRequest,
            _: &RenderingMemory,
        ) -> Result<WorldDisplayObservation, RuleError> {
            Err(unused())
        }
    }
    impl CombatCenterServices for Observed {}
    impl InventoryServices for Observed {
        fn sane(
            &self,
            _: &SaneRequest,
            _: &RenderingMemory,
        ) -> Result<Vec<RecordWrite>, RuleError> {
            Err(unused())
        }
    }
    impl EquipmentUiServices for Observed {
        fn pen_position(
            &mut self,
            _: u32,
            _: &RenderingMemory,
        ) -> Result<Vec<RecordWrite>, RuleError> {
            Err(unused())
        }
    }
    impl InterfaceServices for Observed {
        fn word(&mut self, _: u32) -> Result<i16, RuleError> {
            Err(unused())
        }
        fn current_port(&mut self) -> Result<u32, RuleError> {
            Err(unused())
        }
        fn pen_state(&mut self) -> Result<[u8; 18], RuleError> {
            Err(unused())
        }
        fn local_to_global(&mut self, _: Point) -> Result<Point, RuleError> {
            Err(unused())
        }
        fn point_in_rect(&mut self, _: Point, _: BodyRect) -> Result<u8, RuleError> {
            Err(unused())
        }
        fn new_region(&mut self) -> Result<u32, RuleError> {
            Err(unused())
        }
        fn main_device_rect(&mut self) -> Result<BodyRect, RuleError> {
            Err(unused())
        }
    }
    impl EquipmentServices for Observed {
        fn profile_observations(
            &mut self,
            _: i16,
            _: i16,
            _: &RenderingMemory,
        ) -> Result<Vec<FileObservation>, RuleError> {
            Err(unused())
        }
    }
    impl InventoryEditServices for Observed {
        fn observe_edit(
            &mut self,
            r: &InventoryEditRequest,
            m: &RenderingMemory,
        ) -> Result<InventoryEditObservation, RuleError> {
            let o = match r {
                InventoryEditRequest::Imported { address, arguments } => self.library(
                    &LibraryRequest {
                        entry: *address,
                        arguments: arguments.clone(),
                    },
                    m,
                )?,
                InventoryEditRequest::Toolbox {
                    opcode,
                    arguments,
                    d0_input: Some(d0),
                } => {
                    let o = self.observe_registers(
                        &PromptRegisterRequest {
                            opcode: *opcode,
                            d0_input: Some(*d0),
                            a0_input: None,
                            arguments: arguments.clone(),
                        },
                        m,
                    )?;
                    LibraryObservation {
                        memory: o.memory,
                        value: o.value,
                        d0: o.d0,
                    }
                }
                InventoryEditRequest::Toolbox {
                    opcode, arguments, ..
                } => {
                    let o = FeedbackServices::observe(
                        self,
                        &FeedbackRequest::Toolbox {
                            opcode: *opcode,
                            arguments: arguments.clone(),
                        },
                        m,
                    )?;
                    LibraryObservation {
                        memory: o.memory,
                        value: o.value,
                        d0: o.d0,
                    }
                }
            };
            Ok(InventoryEditObservation {
                memory: o.memory,
                value: o.value,
                d0: o.d0,
            })
        }
    }
    impl FieldControlServices for Observed {
        fn incoming_attack_slot(&mut self, _: &RenderingMemory) -> Result<u16, RuleError> {
            Err(unused())
        }
    }
    impl DetailServices for Observed {
        fn observe(
            &mut self,
            _: &DetailRequest,
            _: &RenderingMemory,
        ) -> Result<DetailReply, RuleError> {
            Err(unused())
        }
    }
    impl CombatPanelServices for Observed {
        fn observe_panel(
            &mut self,
            _: &CombatPanelRequest,
            _: &RenderingMemory,
        ) -> Result<CombatPanelObservation, RuleError> {
            Err(unused())
        }
        fn visibility_memory(&mut self, _: &RenderingMemory) -> Result<InstallMemory, RuleError> {
            Err(unused())
        }
    }
    impl StatusPanelServices for Observed {
        fn observe_status(
            &self,
            _: &StatusPanelRequest,
            _: &RenderingMemory,
        ) -> Result<StatusPanelObservation, RuleError> {
            Err(unused())
        }
        fn body_ground_state(&self, _: &RenderingMemory) -> Result<BodyState, RuleError> {
            Err(unused())
        }
        fn quiet_observations(
            &self,
            _: i16,
            _: &AudioContext,
            _: &RenderingMemory,
        ) -> Result<Vec<AudioObservation>, RuleError> {
            Err(unused())
        }
        fn heap_observations(
            &self,
            _: &runtime::RuntimeContext,
        ) -> Result<Vec<RuntimeObservation>, RuleError> {
            Err(unused())
        }
    }
    impl CharacterUiServices for Observed {
        fn observe(
            &mut self,
            request: &CharacterUiRequest,
            memory: &RenderingMemory,
        ) -> Result<CharacterUiObservation, RuleError> {
            let mut library = None;
            let (opcode, args) = match *request {
                CharacterUiRequest::SetPort(p) => (0xa873, vec![L(p)]),
                CharacterUiRequest::BackPixPat(p) => (0xaa0b, vec![L(p)]),
                CharacterUiRequest::ForeColor(c) => (0xa862, vec![L(c as u32)]),
                CharacterUiRequest::TextSize(n) => (0xa88a, vec![W(n as u16)]),
                CharacterUiRequest::TextFont(n) => (0xa887, vec![W(n as u16)]),
                CharacterUiRequest::RgbForeColor(p) => (0xaa14, vec![P(p)]),
                CharacterUiRequest::GetDialog {
                    id,
                    storage,
                    behind,
                } => (0xa97c, vec![W(id as u16), L(storage), L(behind)]),
                CharacterUiRequest::DrawControls(p) => (0xa915, vec![L(p)]),
                CharacterUiRequest::DrawDialog(p) => (0xa981, vec![L(p)]),
                CharacterUiRequest::BeginUpdate(p) => (0xa922, vec![L(p)]),
                CharacterUiRequest::EndUpdate(p) => (0xa923, vec![L(p)]),
                CharacterUiRequest::DisposeDialog(p) => (0xa983, vec![L(p)]),
                CharacterUiRequest::FlushEvents(_) => (0xa032, vec![]),
                CharacterUiRequest::ModalDialog {
                    filter,
                    item_address,
                } => (0xa991, vec![L(filter), P(item_address)]),
                CharacterUiRequest::GetDialogItem {
                    dialog,
                    item,
                    type_address,
                    handle_address,
                    rect_address,
                } => (
                    0xa98d,
                    vec![
                        L(dialog),
                        W(item as u16),
                        P(type_address),
                        P(handle_address),
                        P(rect_address),
                    ],
                ),
                CharacterUiRequest::SetControlValue { handle, value } => {
                    (0xa963, vec![L(handle), W(value as u16)])
                }
                CharacterUiRequest::SetDialogItemText {
                    handle,
                    text_address,
                } => (0xa98f, vec![L(handle), P(text_address)]),
                CharacterUiRequest::NumberToString { value, destination } => {
                    library = Some(LibraryRequest {
                        entry: 0x400168,
                        arguments: vec![L(value as u32), P(destination)],
                    });
                    (0, vec![])
                }
                CharacterUiRequest::GetIndexedString {
                    destination,
                    resource,
                    index,
                } => {
                    library = Some(LibraryRequest {
                        entry: 0x400178,
                        arguments: vec![P(destination), W(resource as u16), W(index as u16)],
                    });
                    (0, vec![])
                }
                CharacterUiRequest::CopyCString {
                    destination,
                    source,
                } => {
                    library = Some(LibraryRequest {
                        entry: 0x400090,
                        arguments: vec![P(source), P(destination)],
                    });
                    (0, vec![])
                }
                CharacterUiRequest::AppendCString {
                    destination,
                    source,
                    count,
                } => {
                    library = Some(LibraryRequest {
                        entry: 0x4000a0,
                        arguments: vec![W(count), P(source), P(destination)],
                    });
                    (0, vec![])
                }
                CharacterUiRequest::CToPascal(p) => {
                    library = Some(LibraryRequest {
                        entry: 0x400188,
                        arguments: vec![P(p)],
                    });
                    (0, vec![])
                }
                CharacterUiRequest::PascalToC(p) => {
                    library = Some(LibraryRequest {
                        entry: 0x400180,
                        arguments: vec![P(p)],
                    });
                    (0, vec![])
                }
            };
            if let Some(r) = library {
                let o = PromptControlServices::library(self, &r, memory)?;
                Ok(CharacterUiObservation {
                    value: o.value,
                    memory: o.memory,
                })
            } else {
                let o = FeedbackServices::observe(
                    self,
                    &FeedbackRequest::Toolbox {
                        opcode,
                        arguments: args,
                    },
                    memory,
                )?;
                Ok(CharacterUiObservation {
                    value: o.value,
                    memory: o.memory,
                })
            }
        }
        fn profile_observations(
            &mut self,
            _: i16,
            _: i16,
            _: &FileContext,
        ) -> Result<Vec<FileObservation>, RuleError> {
            Err(unused())
        }
    }
    impl GammaServices for Observed {
        fn observe(
            &self,
            r: &GammaRequest,
            m: &GammaMemory,
        ) -> Result<GammaObservation, GammaError> {
            GammaServices::observe(&self.startup, r, m)
        }
    }
    impl GammaControlServices for Observed {
        fn observe_control(
            &self,
            r: &GammaControlRequest,
            m: &GammaMemory,
        ) -> Result<GammaObservation, GammaError> {
            self.startup.observe_control(r, m)
        }
    }
    impl LoadServices for Observed {
        fn observe_load(
            &self,
            r: &LoadRequest,
            m: &RenderingMemory,
        ) -> Result<LoadObservation, RuleError> {
            let mut o = self.startup.observe_load(r, m)?;
            if let LoadRequest::NumberToString { value, destination } = r {
                o.memory.insert_region(
                    *destination,
                    PascalString::from_payload(value.to_string().as_bytes())?.raw(),
                );
            }
            Ok(o)
        }
        fn version_inputs(
            &self,
            id: i16,
            f: i16,
            m: &RenderingMemory,
        ) -> Result<VersionInputs, RuleError> {
            self.startup.version_inputs(id, f, m)
        }
        fn filename_number(&self, v: u8, m: &RenderingMemory) -> Result<PascalString, RuleError> {
            self.startup.filename_number(v, m)
        }
    }
    impl StorageServices for Observed {
        fn observe_storage(
            &self,
            r: &StorageRequest,
            m: &RenderingMemory,
        ) -> Result<StorageObservation, RuleError> {
            self.startup.observe_storage(r, m)
        }
    }
    impl NumericServices for Observed {
        fn number_to_string(&mut self, v: i32) -> Result<PascalString, RuleError> {
            self.startup.number_to_string(v)
        }
        fn string_to_number(&mut self, p: &PascalString) -> Result<i32, RuleError> {
            self.startup.string_to_number(p)
        }
    }
    impl DrawingServices for Observed {
        fn color_icon(&mut self, id: i16) -> Result<u32, RuleError> {
            self.startup.color_icon(id)
        }
        fn indexed_string(&mut self, r: i16, i: i16) -> Result<PascalString, RuleError> {
            self.startup.indexed_string(r, i)
        }
        fn current_world(&mut self) -> Result<(u32, u32), RuleError> {
            self.startup.current_world()
        }
        fn picture(&mut self, id: i16) -> Result<PictureObservation, RuleError> {
            self.startup.picture(id)
        }
        fn new_world(&mut self, d: i16, b: BodyRect) -> Result<(i16, BuildingWorld), RuleError> {
            self.startup.new_world(d, b)
        }
    }
    impl ItemUiServices for Observed {
        fn observe(
            &mut self,
            r: &ItemUiRequest,
            _: &RenderingMemory,
        ) -> Result<ItemUiReply, RuleError> {
            match r {
                ItemUiRequest::GetDialogItem {
                    item_type,
                    handle,
                    rectangle,
                    ..
                } => Ok(ItemUiReply::Writes(vec![
                    crate::recovered_records::RecordWrite {
                        address: *item_type,
                        bytes: 8_i16.to_be_bytes().to_vec(),
                    },
                    crate::recovered_records::RecordWrite {
                        address: *handle,
                        bytes: 77_u32.to_be_bytes().to_vec(),
                    },
                    crate::recovered_records::RecordWrite {
                        address: *rectangle,
                        bytes: vec![0; 8],
                    },
                ])),
                ItemUiRequest::GetDialogItemText { .. } => {
                    Ok(ItemUiReply::Pascal(PascalString::from_payload(b"A\rB")?))
                }
                _ => Err(RuleError::Invalid("unexpected fixture item request".into())),
            }
        }
    }
    impl TimingServices for Observed {
        fn tick_count(&mut self) -> Result<u32, RuleError> {
            let v = self.startup.ticks.get();
            self.startup.ticks.set(v + 100);
            Ok(v)
        }
        fn button(&mut self) -> Result<bool, RuleError> {
            Ok(false)
        }
    }
    impl StartupServices for Observed {
        fn observe_startup(
            &self,
            r: &StartupRequest,
            m: &RenderingMemory,
        ) -> Result<StartupObservation, RuleError> {
            self.startup.observe_startup(r, m)
        }
        fn preference_inputs(&self, m: &RenderingMemory) -> Result<PreferenceInputs, RuleError> {
            StartupServices::preference_inputs(&self.startup, m)
        }
    }
    impl FilePrepServices for Observed {
        fn preference_inputs(
            &mut self,
            m: &RenderingMemory,
        ) -> Result<PreferenceInputs, RuleError> {
            StartupServices::preference_inputs(&self.startup, m)
        }
        fn metadata_observations(
            &mut self,
            _: &[u8],
            _: &[u8],
            _: &RenderingMemory,
        ) -> Result<Vec<FileObservation>, RuleError> {
            Ok(vec![])
        }
    }
    impl ScenarioMenuServices for Observed {}
    impl crate::recovered_inventory_selection::InventorySelectionServices for Observed {}

    impl CharacterPanelServices for Observed {
        fn panel_observe(
            &mut self,
            request: &CharacterPanelRequest,
            memory: &RenderingMemory,
        ) -> Result<CharacterPanelObservation, RuleError> {
            let (value, memory) = match request {
                CharacterPanelRequest::Toolbox {
                    selector,
                    arguments,
                } => {
                    let args = arguments
                        .iter()
                        .map(|a| match a {
                            crate::recovered_character_panels::PanelArgument::Byte(b) => B(*b),
                            crate::recovered_character_panels::PanelArgument::Word(w) => W(*w),
                            crate::recovered_character_panels::PanelArgument::Long(l) => L(*l),
                            crate::recovered_character_panels::PanelArgument::Pointer(p) => P(*p),
                        })
                        .collect();
                    let o = FeedbackServices::observe(
                        self,
                        &FeedbackRequest::Toolbox {
                            opcode: *selector,
                            arguments: args,
                        },
                        memory,
                    )?;
                    (o.value, o.memory)
                }
                CharacterPanelRequest::RuntimeString { .. } => return Err(unused()),
            };
            Ok(CharacterPanelObservation { value, memory })
        }
        fn runtime_observations(
            &mut self,
            entry: u32,
            memory: &RenderingMemory,
        ) -> Result<Vec<RuntimeObservation>, RuleError> {
            if entry == 0x13a498 {
                assert_eq!(memory.long(0x43879c)?, 0);
                return Ok(vec![]);
            }
            assert_eq!(entry, 0x13e13c);
            let mut trace = vec![];
            for a in [0x4386d8, 0x4386e0, 0x4386e4, 0x4386e8, 0x4386dc] {
                trace.push(RuntimeObservation {
                    request: RuntimeRequest::UnlockPixels(memory.long(a)?),
                    reply: RuntimeReply::Ignored,
                });
            }
            trace.push(RuntimeObservation {
                request: RuntimeRequest::MaxMem,
                reply: RuntimeReply::MaxMem { d0: 100, a0: 200 },
            });
            for (a, id) in [0x4386b4, 0x4386bc, 0x4386c0, 0x4386c4, 0x4386b8]
                .into_iter()
                .zip(1..=5)
            {
                trace.push(RuntimeObservation {
                    request: RuntimeRequest::GetGWorldPixMap(memory.long(a)?),
                    reply: RuntimeReply::Pointer(id),
                });
                trace.push(RuntimeObservation {
                    request: RuntimeRequest::LockPixels(id),
                    reply: RuntimeReply::LockPixels {
                        boolean: 1,
                        native_d0: 0xaabbcc00,
                    },
                });
            }
            Ok(trace)
        }
    }
    const FRAME: u32 = 0x10000;
    fn unused() -> RuleError {
        RuleError::Invalid("unexpected focused session observation".into())
    }
    fn put_word(m: &mut RenderingMemory, a: u32, v: u16) {
        m.insert_region(a, &v.to_be_bytes());
    }
    fn put_long(m: &mut RenderingMemory, a: u32, v: u32) {
        m.insert_region(a, &v.to_be_bytes());
    }
    fn value(a: &FeedbackArg) -> u32 {
        match *a {
            W(v) => u32::from(v),
            B(v) => u32::from(v),
            L(v) | P(v) => v,
        }
    }
    fn limits() -> SessionControlLimits {
        SessionControlLimits {
            maximum_steps: 5000,
            maximum_polls: 100,
            filename_capacity: 256,
            maximum_random_draws: 4,
            equipment: EquipmentLimits {
                maximum_steps: 1000,
                maximum_polls: 100,
                filename_capacity: 256,
            },
        }
    }
    fn state() -> PromptControlState {
        let mut s = base::state();
        // Explicit fixture regions; production requires reached bytes from its caller.
        for (a, n) in [
            (FRAME - 0x3000, 0x3040),
            (0x401400, 0x1400),
            (0x41f604, 0x200),
            (0x41d1c0, 4),
            (0x442500, 0x2200),
            (0x438000, 0x140),
            (0x4046d0, 64),
            (0x439cf0, 64),
        ] {
            s.memory.insert_region(a, &vec![0; n]);
        }
        put_long(&mut s.memory, 0x479fd8, 0xa000);
        put_long(&mut s.memory, 0x43879c, 0xa100);
        put_long(&mut s.memory, 0x4386ae, 0xa200);
        put_word(&mut s.memory, 0x439576, u16::MAX);
        put_word(&mut s.memory, 0x439596, 0);
        s.memory.insert_region(0x44273a, &[255]);
        for (a, t) in [
            (0x401990, b"DQ\0".as_slice()),
            (0x401982, b"DT\0"),
            (0x40198c, b"rb\0"),
            (0x401998, b"\0"),
            (0x401d9d, b"\0"),
            (0x402436, b"NEWC\0"),
            (0x401f52, b"GI\0"),
            (0x401f40, b"GI\0"),
            (0x401f2e, b"GI\0"),
            (0x401f28, b"GI\0"),
            (0x402466, b"wb\0"),
        ] {
            s.memory.insert_region(a, t);
        }
        s
    }
    struct Observed {
        base: base::Observed,
        startup: crate::recovered_startup::test_support::Services,
        requests: RefCell<Vec<FeedbackRequest>>,
        libraries: Vec<LibraryRequest>,
        registers: Vec<PromptRegisterRequest>,
        events: RefCell<VecDeque<(u8, u16)>>,
        modal: RefCell<VecDeque<u16>>,
        event: Cell<u8>,
        key: Cell<u8>,
        ticks: Cell<u32>,
        opens: VecDeque<u32>,
        reads: VecDeque<(u32, Vec<u8>)>,
        question_records: Vec<(u32, Vec<u8>)>,
        question_offset: u32,
        second_argument: Option<u16>,
        restore_variant: Option<u8>,
        force_strcmp_match: bool,
    }
    impl Default for Observed {
        fn default() -> Self {
            Self {
                base: base::Observed::new(),
                startup: Default::default(),
                requests: RefCell::new(vec![]),
                libraries: vec![],
                registers: vec![],
                events: RefCell::new(VecDeque::from([(0, 2)])),
                modal: RefCell::new(VecDeque::from([2])),
                event: Cell::new(0),
                key: Cell::new(0),
                ticks: Cell::new(0),
                opens: VecDeque::new(),
                reads: VecDeque::new(),
                question_records: vec![],
                question_offset: 0,
                second_argument: None,
                restore_variant: None,
                force_strcmp_match: false,
            }
        }
    }
    impl FeedbackServices for Observed {
        fn observe(
            &self,
            r: &FeedbackRequest,
            m: &RenderingMemory,
        ) -> Result<FeedbackObservation, RuleError> {
            self.requests.borrow_mut().push(r.clone());
            let mut memory = m.clone();
            let mut result = 0;
            if let FeedbackRequest::Toolbox { opcode, arguments } = r {
                let a: Vec<_> = arguments.iter().map(value).collect();
                match opcode {
                    0xa975 => {
                        result = self.ticks.get();
                        self.ticks.set(result + 100);
                    }
                    0xa924 => result = 0,
                    0xa860 => {
                        let (key, item) = self.events.borrow_mut().pop_front().unwrap_or((0, 2));
                        self.event.set(u8::from(key != 0));
                        self.key.set(key);
                        put_word(&mut memory, 0x44720a, if key == 0 { 1 } else { 3 });
                        put_word(&mut memory, 0x4434b2, item);
                        result = 1;
                    }
                    0xa97f => result = u32::from(self.event.get() == 0),
                    0xa980 => result = 1,
                    0xa858 => result = u32::from(self.key.get()),
                    0xa991 => {
                        put_word(
                            &mut memory,
                            a[1],
                            self.modal.borrow_mut().pop_front().unwrap_or(2),
                        );
                    }
                    0xa990 => memory.insert_region(a[1], b"\0"),
                    0xa88c => result = 0,
                    0xa99c => result = 1,
                    0xa98d => {
                        put_word(&mut memory, a[2], 4);
                        put_long(&mut memory, a[3], 5);
                        memory.insert_region(a[4], &[0, 1, 0, 2, 0, 3, 0, 4]);
                    }
                    0xa85d..=0xa85f => {
                        let bit = a[1];
                        let v = memory.long(a[0])?;
                        let mask = 1u32.wrapping_shl(31 - bit);
                        result = u32::from(v & mask != 0);
                        if *opcode != 0xa85d {
                            put_long(
                                &mut memory,
                                a[0],
                                if *opcode == 0xa85e {
                                    v | mask
                                } else {
                                    v & !mask
                                },
                            );
                        }
                    }
                    _ => return FeedbackServices::observe(&self.base, r, m),
                }
            } else {
                match r {
                    FeedbackRequest::BlockMove {
                        source,
                        destination,
                        count,
                    } => {
                        let bytes = read(m, *source, *count as usize)?;
                        memory.insert_region(*destination, &bytes);
                    }
                    FeedbackRequest::CToPascal(p) => {
                        let mut bytes = c_string(m, *p, 256)?;
                        bytes.pop();
                        memory.insert_region(*p, PascalString::from_payload(&bytes)?.raw());
                    }
                    FeedbackRequest::Button => result = 0,
                    _ => return FeedbackServices::observe(&self.base, r, m),
                }
            }
            Ok(FeedbackObservation {
                memory,
                value: result,
                d0: 0xbeef1234,
            })
        }
        fn sound_observations(
            &self,
            id: i16,
            c: &AudioContext,
            m: &RenderingMemory,
        ) -> Result<Vec<AudioObservation>, RuleError> {
            FeedbackServices::sound_observations(&self.startup, id, c, m)
        }
    }
    impl PromptControlServices for Observed {
        fn observe_registers(
            &mut self,
            r: &PromptRegisterRequest,
            m: &RenderingMemory,
        ) -> Result<FeedbackObservation, RuleError> {
            self.registers.push(r.clone());
            self.base.observe_registers(r, m)
        }
        fn library(
            &mut self,
            r: &LibraryRequest,
            m: &RenderingMemory,
        ) -> Result<LibraryObservation, RuleError> {
            self.libraries.push(r.clone());
            let a: Vec<_> = r.arguments.iter().map(value).collect();
            let mut memory = m.clone();
            let mut result = 0;
            match r.entry {
                0x400038 => result = i32::from((a[0] as u16 as i16).wrapping_abs()) as u32,
                0x400040 => result = u32::from((a[0] as u8).to_ascii_uppercase()),
                0x400070 => {
                    result = self.opens.pop_front().unwrap_or(0);
                    if let Some(v) = self.restore_variant.take() {
                        memory.insert_region(0x44255f, &[v]);
                    }
                }
                0x400048 => {
                    let (d, bytes) = if self.question_records.is_empty() {
                        self.reads.pop_front().unwrap_or((0, vec![]))
                    } else {
                        let bytes = self
                            .question_records
                            .iter()
                            .find(|(offset, _)| *offset == self.question_offset)
                            .ok_or_else(|| RuleError::Invalid("missing question record".into()))?
                            .1
                            .clone();
                        (1, bytes)
                    };
                    memory.insert_region(a[3], &bytes);
                    result = d;
                    if let Some(w) = self.second_argument {
                        put_word(&mut memory, FRAME + 10, w);
                        self.second_argument = None;
                    }
                }
                0x400050 => {
                    let _ = read(&memory, a[3], (a[1] * a[2]) as usize)?;
                    result = 1;
                }
                0x400078 => self.question_offset = a[1],
                0x400088 => result = (c_string(&memory, a[0], 256)?.len() - 1) as u32,
                0x400090 => {
                    let b = c_string(&memory, a[0], 256)?;
                    memory.insert_region(a[1], &b);
                }
                0x400098 | 0x4000a0 => {
                    let (from, to, n) = if r.entry == 0x4000a0 {
                        (a[1], a[2], a[0] as usize)
                    } else {
                        (a[0], a[1], 256)
                    };
                    let mut b = c_string(&memory, to, 256)?;
                    b.pop();
                    let mut tail = c_string(&memory, from, 256)?;
                    tail.truncate(n);
                    b.extend(tail);
                    if b.last() != Some(&0) {
                        b.push(0);
                    }
                    memory.insert_region(to, &b);
                }
                0x400180 => {
                    let n = usize::from(memory.byte(a[0])?);
                    let mut b = read(&memory, a[0] + 1, n)?;
                    b.push(0);
                    memory.insert_region(a[0], &b);
                }
                0x400188 => {
                    let mut b = c_string(&memory, a[0], 256)?;
                    b.pop();
                    memory.insert_region(a[0], PascalString::from_payload(&b)?.raw());
                }
                0x400178 => memory.insert_region(a[0], b"\x01?"),
                0x400168 => memory.insert_region(
                    a[1],
                    PascalString::from_payload((a[0] as i32).to_string().as_bytes())?.raw(),
                ),
                0x400160 => put_long(&mut memory, a[1], 123),
                0x400148 => {
                    put_long(&mut memory, a[3], 999999);
                    put_word(&mut memory, a[2], 0);
                }
                0x4000a8 => {
                    result = if self.force_strcmp_match {
                        0
                    } else {
                        u32::from(c_string(&memory, a[0], 256)? != c_string(&memory, a[1], 256)?)
                    }
                }
                0x400068 | 0x400058 | 0x400150 | 0x400158 => (),
                _ => {
                    return Err(RuleError::Invalid(format!(
                        "unobserved session import {:x}",
                        r.entry
                    )));
                }
            }
            Ok(LibraryObservation {
                memory,
                value: result,
                d0: result,
            })
        }
        fn quiet_observations(
            &mut self,
            s: i16,
            c: &AudioContext,
            m: &RenderingMemory,
        ) -> Result<Vec<AudioObservation>, RuleError> {
            let _ = m;
            let mut trace = vec![];
            if c.last_channel < 0 {
                return Ok(trace);
            }
            let indices: Vec<i16> = if s > 0 {
                vec![s]
            } else {
                (1..c.last_channel).collect()
            };
            for i in indices {
                for code in [3, 4] {
                    trace.push(AudioObservation {
                        request: AudioRequest::Immediate {
                            channel: c.channels[i as usize],
                            command: SoundCommand {
                                code,
                                param1: 0,
                                param2: 0,
                            },
                        },
                        reply: AudioReply::Error(0),
                    });
                }
            }
            Ok(trace)
        }
        fn incoming_mechanical_mode(&mut self, m: &RenderingMemory) -> Result<i16, RuleError> {
            self.base.incoming_mechanical_mode(m)
        }
    }
    #[test]
    fn question_files_choose_from_signed_offsets_and_reread_the_second_argument() {
        for (opens, second, updated_second, offsets, key, answer) in [
            (vec![99], 1, Some(3), [0xffff8000, 75], b'Y', 1),
            (vec![0, 99], 2, None, [0xff800000, 512], b'N', 0),
        ] {
            let mut input = state();
            put_word(&mut input.memory, FRAME + 8, 0x8000);
            put_word(&mut input.memory, FRAME + 10, second);
            let mut services = Observed {
                opens: VecDeque::from(opens),
                question_records: vec![
                    (offsets[0], b"\x01y".to_vec()),
                    (offsets[1], b"\x01n".to_vec()),
                ],
                second_argument: updated_second,
                events: RefCell::new(VecDeque::from([(key, 0)])),
                ..Observed::default()
            };
            let before = input.clone();
            let result = question2(&input, FRAME, limits(), &mut services).unwrap();
            assert_eq!(result.state.memory.byte(FRAME - 259).unwrap(), b'y');
            assert_eq!(result.state.memory.byte(FRAME - 515).unwrap(), b'n');
            assert_eq!(result.native_return, PromptReturn::LowWord(answer));
            assert_eq!(result.state.random, input.random);
            assert_eq!(input, before);
        }
    }
    #[test]
    fn question3_retains_bad_key_across_accepted_mouse_and_reads_null_plus_one() {
        let mut input = state();
        put_long(&mut input.memory, FRAME + 8, 0);
        put_long(&mut input.memory, FRAME + 12, 0);
        input.memory.insert_region(1, b"y");
        let mut s = Observed {
            events: RefCell::new(VecDeque::from([(b'x', 0), (0, 1)])),
            ..Observed::default()
        };
        let p = question3(&input, FRAME, limits(), &mut s).unwrap();
        assert_eq!(p.native_return, PromptReturn::LowWord(0));
        assert_eq!(p.state.memory.word(0x4434b2).unwrap(), 1);
        assert!(
            !s.requests
                .borrow()
                .iter()
                .any(|r| matches!(r, FeedbackRequest::Toolbox { opcode: 0xa98b, .. }))
        );
        let mut missing = input.clone();
        missing.memory.bytes.remove(&1);
        let before = missing.clone();
        assert!(question3(&missing, FRAME, limits(), &mut Observed::default()).is_err());
        assert_eq!(missing, before);
    }
    #[test]
    fn matched_registered_name_is_uppercase_and_admitted() {
        let mut input = state();
        input.memory.insert_region(0x438aa4, b"\x03bob\0");
        let mut s = Observed {
            force_strcmp_match: true,
            ..Observed::default()
        };
        let mut e = execution(&input, limits(), &mut s);
        e.registered_name(FRAME).unwrap();
        let p = e.plan;
        assert_eq!(p.state.memory.byte(0x438aa4).unwrap(), 3);
        assert_eq!(read(&p.state.memory, 0x438aa5, 3).unwrap(), b"BOB");
        assert_eq!(p.flow, LoadFlow::Returned);
        assert!(p.effects.iter().any(|e| matches!(
            e,
            SessionControlEffect::RegisteredAdmission {
                stage: RegistrationStage::ReservedName,
                native_match: true
            }
        )));
        assert!(!s.libraries.iter().any(|r| r.entry == 0x400028));
        assert_eq!(p.state.random, input.random);
    }
    #[test]
    fn save_copy_uses_exact_header_one_and_preserves_partial_tail_before_write() {
        let mut input = state();
        input.memory.insert_region(0xe000, &[1; 40]);
        input.memory.insert_region(0xe100, &[2; 8]);
        let mut s = Observed {
            reads: VecDeque::from([(1, vec![9; 40]), (0, vec![7; 2]), (0, vec![])]),
            ..Observed::default()
        };
        let mut e = execution(&input, limits(), &mut s);
        e.save_copy(10, 20, (0xe000, 40), &[(0xe100, 8)]).unwrap();
        let p = e.plan;
        assert_eq!(
            read(&p.state.memory, 0xe100, 8).unwrap(),
            [7, 7, 2, 2, 2, 2, 2, 2]
        );
        let calls: Vec<_> = s
            .libraries
            .iter()
            .filter(|r| matches!(r.entry, 0x400048 | 0x400050))
            .map(|r| r.entry)
            .collect();
        assert_eq!(calls, [0x400048, 0x400048, 0x400050, 0x400050, 0x400048]);
        let mut s = Observed {
            reads: VecDeque::from([(0x10001, vec![])]),
            ..Observed::default()
        };
        let mut e = execution(&input, limits(), &mut s);
        e.save_copy(10, 20, (0xe000, 40), &[]).unwrap();
        assert!(!s.libraries.iter().any(|r| r.entry == 0x400050));
        assert_eq!(input, state_with_buffers());
    }
    fn state_with_buffers() -> PromptControlState {
        let mut s = state();
        s.memory.insert_region(0xe000, &[1; 40]);
        s.memory.insert_region(0xe100, &[2; 8]);
        s
    }
    #[test]
    fn generation_name_cancel_keeps_native_prefix_and_one_draw() {
        let mut input = state();
        put_word(&mut input.memory, 0x439576, 2);
        put_long(&mut input.memory, 0x43879c, 0);
        let mut s = Observed {
            modal: RefCell::new(VecDeque::from([1])),
            ..Observed::default()
        };
        let p = generate(&input, FRAME, None, limits(), &mut s).unwrap();
        assert_eq!(p.native_return, PromptReturn::Full(0));
        assert_eq!(p.state.random.draws, 1);
        assert!(!s.requests.borrow().iter().any(|r|matches!(r,FeedbackRequest::Toolbox{opcode:0xa97c,arguments} if arguments.first()==Some(&W(139)))));
        assert!(
            p.effects
                .iter()
                .any(|e| matches!(e, SessionControlEffect::Creation(_)))
        );
        let mut bounded = limits();
        bounded.maximum_random_draws = 0;
        let before = input.clone();
        let mut bounded_services = Observed {
            modal: RefCell::new(VecDeque::from([1])),
            ..Observed::default()
        };
        assert!(matches!(
            generate(&input, FRAME, None, bounded, &mut bounded_services),
            Err(RuleError::Unsupported {
                action: "session_control.random",
                ..
            })
        ));
        assert_eq!(input, before);
    }
    #[test]
    fn quit_existing_main_cleans_up_and_models_exit_as_terminal() {
        let mut input = state();
        put_word(&mut input.memory, 0x439576, 2);
        put_long(&mut input.memory, 0x438794, 0);
        put_word(&mut input.memory, 0x43959e, 640);
        put_word(&mut input.memory, 0x4395a0, 480);
        input.memory.insert_region(0x4385a0, b"\x01P");
        input
            .memory
            .insert_region(0x41f604, &0xd000_u32.to_be_bytes());
        put_long(&mut input.memory, 0xd01c, 0xdddd);
        let mut s = Observed::default();
        let p = quit_game(&input, FRAME, limits(), &mut s).unwrap();
        assert_eq!(p.flow, LoadFlow::Terminated(0));
        assert_eq!(p.native_return, PromptReturn::NoReturn);
        assert!(matches!(
            p.effects.last(),
            Some(SessionControlEffect::ExitToShell)
        ));
        assert_eq!(p.state.random, input.random);
    }
    #[test]
    fn save_query_composes_land_three_empty_variants_and_fresh_restore_without_draw() {
        let mut input = state();
        let extra = crate::recovered_startup::test_support::fixture();
        for (a, b) in extra.bytes {
            input.memory.bytes.entry(a).or_insert(b);
        }
        for (a, n) in [
            (0x478e42, 0x1068),
            (0x443086, 0x284),
            (0x4358e6, 0x1fa4),
            (0x4085b8, 0x3f48),
            (0x40c500, 0x3f48),
            (0x414390, 0x7e90),
        ] {
            for i in 0..n {
                input.memory.bytes.entry(a + i).or_insert(0);
            }
        }
        put_word(&mut input.memory, FRAME + 8, u16::MAX);
        put_word(&mut input.memory, 0x439578, 3);
        put_long(&mut input.memory, 0x43879c, 0);
        put_long(&mut input.memory, 0x442774, 0);
        input.memory.insert_region(0x44274c, &[1]);
        input.memory.insert_region(0x442756, &[0]);
        for a in [
            0x4017fc, 0x4017f4, 0x4017ec, 0x4017e4, 0x4017dc, 0x4017d4, 0x4017cc, 0x4017c4,
            0x4017ba, 0x4017b4, 0x401799,
        ] {
            input.memory.insert_region(a, b"S\0");
        }
        let mut s = Observed {
            restore_variant: Some(2),
            ..Observed::default()
        };
        let before = input.clone();
        let p = save(&input, FRAME, limits(), &mut s).unwrap();
        assert_eq!(p.flow, LoadFlow::Returned);
        assert_eq!(p.state.random, input.random);
        assert_eq!(p.state.memory.word(0x439578).unwrap(), 4);
        assert_eq!(p.state.memory.byte(0x44274c).unwrap(), 2);
        assert_eq!(p.state.memory.byte(0x4386a9).unwrap(), 0);
        assert!(
            p.effects
                .iter()
                .any(|e| matches!(e, SessionControlEffect::Storage(_)))
        );
        assert!(
            p.effects
                .iter()
                .any(|e| matches!(e, SessionControlEffect::Load(_)))
        );
        assert_eq!(input, before);
    }
}
