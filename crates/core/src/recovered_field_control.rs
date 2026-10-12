//! Field, scenario and warrior-file control from complete CODE 1 bodies.

use crate::RuleError;
use crate::recovered_attack::{
    self as attack, AttackEffect, AttackTables, AttackWork, PreparationContext,
};
use crate::recovered_combat_center::{self as center, CombatCenterEffect};
use crate::recovered_drawing::{
    self as drawing, DrawingEffect, DrawingServices, PictureObservation,
};
use crate::recovered_equipment::{
    self as equipment, EquipmentEffect, EquipmentLimits, EquipmentServices,
};
use crate::recovered_feedback::{
    self as feedback, FeedbackArg, FeedbackEffect, FeedbackObservation, FeedbackRequest,
    FeedbackServices,
};
use crate::recovered_files::{self as files, FileBuffer, FileContext, FileEffect, FileObservation};
use crate::recovered_geometry::BodyRect;
use crate::recovered_install::InstallMemory;
use crate::recovered_interface;
use crate::recovered_inventory_edit::{
    self as editing, InventoryEditEffect, InventoryEditLimits, InventoryEditServices,
};
use crate::recovered_load_control::{self as load, LoadEffect, LoadFlow, LoadLimits};
use crate::recovered_preferences::{self as preferences, PreferenceContext, PreferenceEffect};
use crate::recovered_prompt_control::{
    self as prompt, LibraryObservation, LibraryRequest, PromptControlEffect, PromptControlServices,
    PromptControlState, PromptLimits, PromptRegisterRequest, PromptReturn,
};
use crate::recovered_records::RecordWrite;
use crate::recovered_records::{self as records, ItemTables, SPELL_BYTES};
use crate::recovered_rendering::RenderingMemory;
use crate::recovered_runtime::{self as runtime, RuntimeContext, RuntimeEffect, RuntimeResult};
use crate::recovered_scenario_menus::{self as menus, ScenarioMenuEffect, ScenarioMenuServices};
use crate::recovered_spatial::{self as spatial, LayoutInput, LayoutServices};
use crate::recovered_startup::PreferenceInputs;
use crate::recovered_storage::{self as storage, StorageEffect, StorageLimits};
use crate::recovered_text::{self as text, PascalString};
use crate::recovered_timing::{self as timing, TimingServices};
use crate::recovered_verification as verification;
use crate::recovered_world_display::WorldDisplayState;
use FeedbackArg::{Byte as B, Long as L, Pointer as P, Word as W};
use centurions_content::{ITEM_BYTES, WARRIOR_BYTES};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldControlLimits {
    pub maximum_steps: u32,
    pub maximum_polls: usize,
    pub filename_capacity: usize,
    pub maximum_random_draws: u64,
    pub equipment: EquipmentLimits,
}
impl FieldControlLimits {
    fn prompt(self) -> PromptLimits {
        PromptLimits {
            maximum_steps: self.maximum_steps,
            maximum_polls: self.maximum_polls,
            filename_capacity: self.filename_capacity,
        }
    }
}

pub trait FilePrepServices: PromptControlServices {
    fn preference_inputs(
        &mut self,
        memory: &RenderingMemory,
    ) -> Result<PreferenceInputs, RuleError>;
    fn metadata_observations(
        &mut self,
        file_type: &[u8],
        name: &[u8],
        memory: &RenderingMemory,
    ) -> Result<Vec<FileObservation>, RuleError>;
}
pub trait FieldControlServices:
    FilePrepServices + InventoryEditServices + ScenarioMenuServices
{
    /// Required only when prepattack's actor >= 10 leaves the native D3.W input uninitialized.
    fn incoming_attack_slot(&mut self, memory: &RenderingMemory) -> Result<u16, RuleError>;
}

#[derive(Clone, Debug, PartialEq)]
pub enum FieldControlEffect {
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
    Runtime(RuntimeEffect),
    File(FileEffect),
    Preference(PreferenceEffect),
    Prompt(PromptControlEffect),
    Equipment(EquipmentEffect),
    Editing(InventoryEditEffect),
    Load(LoadEffect),
    Storage(StorageEffect),
    Menu(ScenarioMenuEffect),
    Center(CombatCenterEffect),
    Attack(AttackEffect),
    LayoutRead(Option<Vec<u8>>),
    Verification {
        member: u16,
        mismatch: bool,
        native_d0: u32,
    },
    LocationLog {
        argument: u16,
        native_return: i32,
    },
}
#[derive(Clone, Debug, PartialEq)]
pub struct FieldControlPlan {
    pub state: PromptControlState,
    pub effects: Vec<FieldControlEffect>,
    pub native_return: PromptReturn,
    pub flow: LoadFlow,
    pub qualification: FieldControlQualification,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldControlQualification {
    SourcePortWithObservedBoundariesAndCapturedChildren,
}

struct Execution<'a, S> {
    plan: FieldControlPlan,
    services: &'a mut S,
    limits: FieldControlLimits,
    remaining: u32,
    remaining_draws: u64,
    d0: Option<u32>,
}
impl<S: FilePrepServices> Execution<'_, S> {
    fn step(&mut self) -> Result<(), RuleError> {
        self.remaining = self
            .remaining
            .checked_sub(1)
            .ok_or(RuleError::Unsupported {
                action: "field_control.processing",
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
    fn sb(&self, a: u32) -> Result<i16, RuleError> {
        Ok(i16::from(self.b(a)? as i8))
    }
    fn byte(&mut self, a: u32, v: u8) -> Result<(), RuleError> {
        self.write(a, &[v])
    }
    fn copy(&mut self, source: u32, destination: u32, n: usize) -> Result<(), RuleError> {
        for offset in (0..n - n % 4).step_by(4) {
            self.lw(
                destination.wrapping_add(offset as u32),
                self.l(source.wrapping_add(offset as u32))?,
            )?;
        }
        if n % 4 == 2 {
            self.ww(
                destination.wrapping_add((n - 2) as u32),
                self.w(source.wrapping_add((n - 2) as u32))?,
            )?;
        } else if !n.is_multiple_of(4) {
            return Err(RuleError::Invalid(
                "unsupported native record-copy tail".into(),
            ));
        }
        Ok(())
    }
    fn sw(&self, a: u32) -> Result<i16, RuleError> {
        Ok(self.w(a)? as i16)
    }
    fn l(&self, a: u32) -> Result<u32, RuleError> {
        self.plan.state.memory.long(a)
    }
    fn bytes(&self, a: u32, n: usize) -> Result<Vec<u8>, RuleError> {
        (0..n).map(|i| self.b(a.wrapping_add(i as u32))).collect()
    }
    fn write(&mut self, a: u32, bytes: &[u8]) -> Result<(), RuleError> {
        self.bytes(a, bytes.len())?;
        self.plan.state.memory.insert_region(a, bytes);
        self.plan
            .effects
            .push(FieldControlEffect::Write(RecordWrite {
                address: a,
                bytes: bytes.to_vec(),
            }));
        Ok(())
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
            .push(FieldControlEffect::Feedback(FeedbackEffect::Service {
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
        self.plan.effects.push(FieldControlEffect::RegisterToolbox {
            request,
            observation,
        });
        Ok(value)
    }
    fn lib(&mut self, entry: u32, arguments: Vec<FeedbackArg>) -> Result<u32, RuleError> {
        self.step()?;
        let request = LibraryRequest { entry, arguments };
        let observation = self.services.library(&request, &self.plan.state.memory)?;
        let value = observation.value;
        self.d0 = Some(observation.d0);
        self.plan.state.memory = observation.memory.clone();
        self.plan.effects.push(FieldControlEffect::Library {
            request,
            observation,
        });
        Ok(value)
    }
    fn feedback(&mut self, child: feedback::FeedbackPlan) {
        self.plan.state.memory = child.memory;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(FieldControlEffect::Feedback));
        self.d0 = child.native_return;
    }
    fn sound(&mut self, id: i16) -> Result<(), RuleError> {
        let child = feedback::sound(&self.plan.state.memory, id, self.services)?;
        self.feedback(child);
        Ok(())
    }
    fn rand(&mut self, bound: i16) -> Result<i16, RuleError> {
        self.step()?;
        self.remaining_draws =
            self.remaining_draws
                .checked_sub(1)
                .ok_or(RuleError::Unsupported {
                    action: "field_control.random",
                    dependency: "the supplied random-draw budget is exhausted",
                })?;
        let mut context =
            RuntimeContext::new(self.plan.state.random.clone(), self.remaining as usize);
        let child = runtime::rand(&mut context, bound)?;
        self.plan.state.random = context.random;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(FieldControlEffect::Runtime));
        match child.result {
            RuntimeResult::Word { value, native_d0 } => {
                self.d0 = Some(native_d0);
                Ok(value)
            }
            _ => Err(RuleError::Invalid("unexpected native Rand result".into())),
        }
    }
    fn item(&mut self, dialog: u32, item: u16, rect: u32) -> Result<(), RuleError> {
        self.trap(
            0xa98d,
            vec![L(dialog), W(item), P(0x4434b0), P(0x4434dc), P(rect)],
        )?;
        Ok(())
    }
    fn di_str(&mut self, item: u16, pointer: u32) -> Result<(), RuleError> {
        self.trap(
            0xa98d,
            vec![
                L(self.l(0x4394b8)?),
                W(item),
                P(0x4434b0),
                P(0x4434dc),
                P(0x4434ba),
            ],
        )?;
        let string =
            PascalString::from_raw(&self.bytes(pointer, usize::from(self.b(pointer)?) + 1)?)?;
        let _child = text::dialog_string(item as i16, &string);
        self.trap(0xa98f, vec![L(self.l(0x4434dc)?), P(pointer)])?;
        self.d0 = Some(0);
        Ok(())
    }
    fn pict(&mut self, frame: u32, id: u16, rectangle: u32) -> Result<(), RuleError> {
        self.lw(frame.wrapping_add(14), self.l(rectangle.wrapping_add(4))?)?;
        self.lw(frame.wrapping_add(10), self.l(rectangle)?)?;
        self.ww(frame.wrapping_add(8), id)?;
        let handle = self.trap(0xa9bc, vec![W(id)])?;
        self.lw(frame.wrapping_sub(4), handle)?;
        let bounds = self.plan.state.memory.rect(frame.wrapping_add(10))?;
        let child = drawing::pict(id as i16, bounds, &mut FixedPicture(handle))?;
        if !matches!(
            child.as_slice(),
            [
                DrawingEffect::GetPicture(_),
                DrawingEffect::DrawPicture { .. }
            ]
        ) {
            return Err(RuleError::Invalid(
                "unexpected recovered pict effects".into(),
            ));
        }
        self.trap(
            0xa8f6,
            vec![L(self.l(frame.wrapping_sub(4))?), P(frame.wrapping_add(10))],
        )?;
        self.d0 = Some(0);
        Ok(())
    }
    fn press(&mut self, frame: u32, kind: u16, phase: u16, rect: u32) -> Result<(), RuleError> {
        self.lw(frame.wrapping_add(18), self.l(rect.wrapping_add(4))?)?;
        self.lw(frame.wrapping_add(14), self.l(rect)?)?;
        self.ww(frame.wrapping_add(12), 0)?;
        self.lw(
            frame.wrapping_add(8),
            (u32::from(kind) << 16) | u32::from(phase),
        )?;
        let child = feedback::pressbutton(
            &self.plan.state.memory,
            frame,
            kind as i16,
            phase as i16,
            0,
            self.services,
        )?;
        self.feedback(child);
        Ok(())
    }
    fn question(&mut self, frame: u32, packed: u32) -> Result<u16, RuleError> {
        self.lw(frame.wrapping_add(8), packed)?;
        let child = prompt::question(&self.plan.state, frame, self.limits.prompt(), self.services)?;
        self.plan.state = child.state;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(FieldControlEffect::Prompt));
        self.plan.flow = child.flow;
        match child.native_return {
            PromptReturn::Full(d) => {
                self.full(d);
                Ok(d as u16)
            }
            PromptReturn::LowWord(w) => {
                self.d0 = None;
                self.plan.native_return = PromptReturn::LowWord(w);
                Ok(w)
            }
            PromptReturn::NoReturn => {
                self.d0 = None;
                self.plan.native_return = PromptReturn::NoReturn;
                Ok(0)
            }
            PromptReturn::Unchanged => {
                Err(RuleError::Invalid("question has no returned WORD".into()))
            }
        }
    }
    fn metadata(&mut self, kind: u32, name: u32) -> Result<(), RuleError> {
        let file_type = c_string(&self.plan.state.memory, kind, self.limits.filename_capacity)?;
        if overlaps(kind, file_type.len(), name, self.limits.filename_capacity)
            || overlaps(kind, file_type.len(), 0x439cfc, 16)
            || overlaps(name, self.limits.filename_capacity, 0x439cfc, 16)
        {
            return Err(RuleError::Unsupported {
                action: "field_control.setfileinfo",
                dependency: "the captured metadata child does not support overlapping source buffers",
            });
        }
        let mut name_buffer = self.bytes(name, self.limits.filename_capacity)?;
        let observations = self.services.metadata_observations(
            &file_type,
            &name_buffer,
            &self.plan.state.memory,
        )?;
        let mut context = FileContext::default();
        let child =
            files::set_file_info(&mut context, &file_type, &mut name_buffer, &observations)?;
        for effect in child.effects {
            match &effect {
                FileEffect::Write {
                    destination: FileBuffer::MetadataFilename,
                    offset,
                    bytes,
                } => {
                    self.write(name.wrapping_add(*offset as u32), bytes)?;
                }
                FileEffect::Write {
                    destination: FileBuffer::FinderInfo,
                    offset,
                    bytes,
                } => {
                    self.write(0x439cfc_u32.wrapping_add(*offset as u32), bytes)?;
                }
                _ => self.plan.effects.push(FieldControlEffect::File(effect)),
            }
        }
        self.d0 = Some(0);
        Ok(())
    }
    fn save_pref(&mut self) -> Result<(), RuleError> {
        let inputs = self.services.preference_inputs(&self.plan.state.memory)?;
        let mut context = PreferenceContext::new(self.plan.state.random.clone());
        context.handles = inputs.handles;
        for (address, length) in [
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
                .insert(address, self.bytes(address, length)?);
        }
        for n in 0..20_u32 {
            context
                .globals
                .insert(0x44270e + n, vec![self.b(0x44270e + n)?]);
        }
        context.globals.insert(
            0x4385a0,
            self.bytes(0x4385a0, usize::from(self.b(0x4385a0)?) + 1)?,
        );
        let child = preferences::save_preferences(&mut context, &inputs.observations)?;
        for effect in child.effects {
            if let PreferenceEffect::GlobalWrite { address, bytes } = &effect {
                self.write(*address, bytes)?;
            } else {
                self.plan
                    .effects
                    .push(FieldControlEffect::Preference(effect));
            }
        }
        self.plan.state.random = context.random;
        self.d0 = Some(0);
        Ok(())
    }
    fn file_prep(&mut self, frame: u32) -> Result<(), RuleError> {
        self.step()?;
        let mode = self.w(frame.wrapping_add(8))?;
        if self.sw(0x439578)? < 4 {
            self.ww(0x439578, 4)?;
        }
        if self.sw(0x439578)? > 14 {
            self.ww(0x439578, 4)?;
        }
        let mut selected = self.b(0x439579)? as i8;
        if mode == u16::MAX {
            self.low(self.w(0x439578)?);
            return Ok(());
        }
        self.trap(0xaa1c, vec![L(self.l(0x43952c)?)])?;
        let dialog = self.trap(0xa97c, vec![W(139), L(0), L(u32::MAX)])?;
        self.trap(0xa873, vec![L(dialog)])?;
        self.trap(0xaa0b, vec![L(self.l(0x4358ce)?)])?;
        self.trap(0xa887, vec![W(self.w(0x402f44)?)])?;
        self.trap(0xa862, vec![L(69)])?;
        self.lw(0x4394b8, dialog)?;
        if self.trap(0xa924, vec![])? != self.l(0x479fd8)? {
            let h = half(self.sw(0x40856c)?)
                .wrapping_mul(self.sw(0x439582)?)
                .wrapping_add(self.sw(0x4395a4)?);
            let v = half(self.sw(0x40856e)?)
                .wrapping_mul(self.sw(0x439582)?)
                .wrapping_add(self.sw(0x4395a2)?);
            self.trap(0xa91b, vec![L(dialog), W(h as u16), W(v as u16), B(0)])?;
        }
        for opcode in [0xa915, 0xa981, 0xa922, 0xa923] {
            self.trap(opcode, vec![L(dialog)])?;
        }
        let sound = self.rand(2)?.wrapping_add(200);
        self.sound(sound)?;
        self.ww(0x438754, 0)?;
        self.item(dialog, 48, 0x4434ba)?;
        self.pict(frame.wrapping_sub(374), mode.wrapping_add(185), 0x4434ba)?;
        self.item(dialog, self.w(0x439578)?, 0x4434ba)?;
        self.trap(0xa8a9, vec![P(0x4434ba), L(0x20002)])?;
        self.trap(0xa8f6, vec![L(self.l(0x439550)?), P(0x4434ba)])?;
        loop {
            self.step()?;
            let handle = self.lib(0x400070, vec![P(0x4038ba), P(0x4038be)])?;
            if handle != 0 {
                let offset = (i32::from(selected).wrapping_sub(4) as u32)
                    .wrapping_shl(8)
                    .wrapping_add(40);
                self.lib(0x400078, vec![W(0), L(offset), L(handle)])?;
                self.lib(
                    0x400048,
                    vec![L(handle), L(1), L(256), P(frame.wrapping_sub(0x150))],
                )?;
                let read_result = self.d0.expect("observed fread result");
                self.di_str(
                    41,
                    if read_result != 0 {
                        frame.wrapping_sub(0x150)
                    } else {
                        0x4038a6
                    },
                )?;
                self.lib(0x400068, vec![L(handle)])?;
            } else {
                self.di_str(41, 0x4038a6)?;
                let created = self.lib(0x400070, vec![P(0x4038a2), P(0x4038be)])?;
                if created != 0 {
                    self.lib(0x400068, vec![L(created)])?;
                    self.metadata(0x40389c, 0x4038be)?;
                    continue;
                }
            }
            self.lib(0x400090, vec![P(0x403890), P(frame.wrapping_sub(0x50))])?;
            let offset = i16::from(selected).wrapping_sub(4) as u16;
            if offset <= 10 {
                let suffix = 0x40388c_u32.wrapping_sub(u32::from(offset) * 4);
                self.lib(0x400098, vec![P(suffix), P(frame.wrapping_sub(0x50))])?;
            }
            self.lib(0x400090, vec![P(frame.wrapping_sub(0x50)), P(0x4389a4)])?;
            self.lib(0x400098, vec![P(0x40385a), P(0x4389a4)])?;
            let game = self.lib(0x400070, vec![P(0x4038ba), P(0x4389a4)])?;
            if game == 0 {
                self.lib(0x400090, vec![P(0x403846), P(0x479ed8)])?;
                self.lib(0x400188, vec![P(0x479ed8)])?;
            } else {
                self.lib(
                    0x400048,
                    vec![L(game), L(1), L(2), P(frame.wrapping_sub(0x152))],
                )?;
                self.lib(0x400068, vec![L(game)])?;
                self.trap(
                    0xa946,
                    vec![
                        L(self.l(0x43870e)?),
                        W(self.w(frame.wrapping_sub(0x152))?),
                        P(0x479ed8),
                    ],
                )?;
            }
            self.di_str(43, 0x479ed8)?;
            let mut redisplay = false;
            loop {
                self.step()?;
                if self.w(0x438754)? != 0 {
                    self.ww(0x439578, i16::from(selected) as u16)?;
                    self.save_pref()?;
                    break;
                }
                self.registers(0xa032, Some(0xffff), None, vec![])?;
                self.trap(0xa991, vec![L(0), P(0x4434b2)])?;
                self.item(dialog, self.w(0x4434b2)?, 0x44292c)?;
                if self.w(0x4434b2)? == 2 {
                    self.press(frame.wrapping_sub(376), 1, 1, 0x44292c)?;
                    selected = 0;
                    break;
                }
                if (self.sw(0x4434b2)? > 2 && self.sw(0x4434b2)? < 14) || self.w(0x4434b2)? == 1 {
                    self.item(dialog, 41, 0x4434ba)?;
                    self.trap(
                        0xa990,
                        vec![L(self.l(0x4434dc)?), P(frame.wrapping_sub(0x150))],
                    )?;
                    let description = self.lib(0x400070, vec![P(0x403842), P(0x4038be)])?;
                    if description != 0 {
                        let offset = (i32::from(selected).wrapping_sub(4) as u32)
                            .wrapping_shl(8)
                            .wrapping_add(40);
                        self.lib(0x400078, vec![W(0), L(offset), L(description)])?;
                        self.lib(
                            0x400050,
                            vec![L(description), L(1), L(256), P(frame.wrapping_sub(0x150))],
                        )?;
                        self.lib(0x400068, vec![L(description)])?;
                    }
                }
                if self.w(0x4434b2)? == 14 {
                    self.press(frame.wrapping_sub(376), 1, 1, 0x44292c)?;
                    let answer = self.question(frame.wrapping_sub(380), 0x000c0001)?;
                    if self.plan.flow != LoadFlow::Returned {
                        return Ok(());
                    }
                    if answer != 0 {
                        self.ww(0x4434aa, 0)?;
                        while self.sw(0x4434aa)? < 3 {
                            self.step()?;
                            for suffix in [
                                0x403838, 0x40382e, 0x403824, 0x40381a, 0x403810, 0x403806,
                                0x4037fc, 0x4037f2,
                            ] {
                                self.lib(0x400090, vec![P(frame.wrapping_sub(0x50)), P(0x4389a4)])?;
                                self.lib(
                                    0x400168,
                                    vec![L(i32::from(self.sw(0x4434aa)?) as u32), P(0x479ed8)],
                                )?;
                                self.lib(0x400180, vec![P(0x479ed8)])?;
                                self.lib(0x400098, vec![P(0x479ed8), P(0x4389a4)])?;
                                self.lib(0x400098, vec![P(suffix), P(0x4389a4)])?;
                                self.lib(0x400058, vec![P(0x4389a4)])?;
                            }
                            self.addw(0x4434aa, 1)?;
                        }
                    }
                    self.trap(0xa873, vec![L(dialog)])?;
                    self.lw(0x4394b8, dialog)?;
                    self.item(dialog, 14, 0x44292c)?;
                    self.press(frame.wrapping_sub(376), 1, 2, 0x44292c)?;
                    redisplay = true;
                    break;
                }
                if self.sw(0x4434b2)? > 3 && self.sw(0x4434b2)? < 14 {
                    self.item(dialog, i16::from(selected) as u16, 0x4434ba)?;
                    self.pict(frame.wrapping_sub(372), 179, 0x4434ba)?;
                    self.sound(-144)?;
                    self.item(dialog, self.w(0x4434b2)?, 0x4434ba)?;
                    self.trap(0xa8a9, vec![P(0x4434ba), L(0x20002)])?;
                    self.trap(0xa8f6, vec![L(self.l(0x439550)?), P(0x4434ba)])?;
                    selected = self.b(0x4434b3)? as i8;
                    self.sound(145)?;
                    redisplay = true;
                    break;
                }
                if self.w(0x4434b2)? == 3 || self.w(0x4434b2)? == 1 {
                    self.item(dialog, 3, 0x4434ba)?;
                    self.press(frame.wrapping_sub(376), 1, 2, 0x4434ba)?;
                    self.ww(0x439578, i16::from(selected) as u16)?;
                    self.save_pref()?;
                    break;
                }
            }
            if !redisplay {
                break;
            }
        }
        self.sound(214)?;
        self.trap(0xa983, vec![L(dialog)])?;
        if self.trap(0xa924, vec![])? == self.l(0x479fd8)? {
            self.trap(0xa873, vec![L(self.l(0x479fd8)?)])?;
            self.lw(0x4394b8, self.l(0x479fd8)?)?;
            self.trap(0xa863, vec![L(33)])?;
        }
        self.ww(0x4434b2, 0)?;
        self.low(i16::from(selected) as u16);
        Ok(())
    }
}

/// fileprep, 0x122e24. A6+8 is the actual caller mode WORD.
pub fn file_prep(
    state: &PromptControlState,
    frame: u32,
    limits: FieldControlLimits,
    services: &mut impl FilePrepServices,
) -> Result<FieldControlPlan, RuleError> {
    let mut execution = Execution {
        plan: FieldControlPlan {
            state: state.clone(),
            effects: vec![],
            native_return: PromptReturn::Unchanged,
            flow: LoadFlow::Returned,
            qualification:
                FieldControlQualification::SourcePortWithObservedBoundariesAndCapturedChildren,
        },
        services,
        limits,
        remaining: limits.maximum_steps,
        remaining_draws: limits.maximum_random_draws,
        d0: None,
    };
    execution.file_prep(frame)?;
    Ok(execution.plan)
}
fn half(value: i16) -> i16 {
    value.wrapping_add(((value as u16) >> 15) as i16) >> 1
}
fn overlaps(a: u32, n: usize, b: u32, m: usize) -> bool {
    let end_a = u64::from(a) + n as u64;
    let end_b = u64::from(b) + m as u64;
    u64::from(a) < end_b && u64::from(b) < end_a
}
fn c_string(memory: &RenderingMemory, address: u32, limit: usize) -> Result<Vec<u8>, RuleError> {
    let mut bytes = vec![];
    for n in 0..limit {
        let b = memory.byte(address.wrapping_add(n as u32))?;
        bytes.push(b);
        if b == 0 {
            return Ok(bytes);
        }
    }
    Err(RuleError::Unsupported {
        action: "field_control.c_string",
        dependency: "the supplied string budget does not reach a terminator",
    })
}
struct FixedPicture(u32);
impl DrawingServices for FixedPicture {
    fn picture(&mut self, _: i16) -> Result<PictureObservation, RuleError> {
        Ok(PictureObservation {
            handle: self.0,
            bounds: None,
        })
    }
    fn color_icon(&mut self, _: i16) -> Result<u32, RuleError> {
        Err(RuleError::Invalid("unexpected picture color icon".into()))
    }
    fn indexed_string(&mut self, _: i16, _: i16) -> Result<PascalString, RuleError> {
        Err(RuleError::Invalid(
            "unexpected picture indexed string".into(),
        ))
    }
    fn current_world(&mut self) -> Result<(u32, u32), RuleError> {
        Err(RuleError::Invalid(
            "unexpected picture current world".into(),
        ))
    }
    fn new_world(
        &mut self,
        _: i16,
        _: BodyRect,
    ) -> Result<(i16, drawing::BuildingWorld), RuleError> {
        Err(RuleError::Invalid("unexpected picture new world".into()))
    }
}

impl<S: FieldControlServices> Execution<'_, S> {
    fn key_text(&mut self, frame: u32) -> Result<(), RuleError> {
        self.ww(frame.wrapping_add(16), 0)?;
        self.lw(frame.wrapping_add(12), 0)?;
        self.lw(frame.wrapping_add(8), 0x00030001)?;
        self.textbox(frame)
    }
    fn key_position(&mut self) -> Result<(), RuleError> {
        self.trap(
            0xa893,
            vec![
                W(30),
                W(self
                    .w(0x40856e)?
                    .wrapping_add(350)
                    .wrapping_sub(self.w(0x439582)?.wrapping_shl(5))),
            ],
        )?;
        Ok(())
    }
    fn check_keypad(&mut self, frame: u32) -> Result<(), RuleError> {
        self.step()?;
        let mut transition = false;
        let mut destination = -2_i16;
        self.byte(0x437a6a, 0)?;
        self.byte(0x437a6b, 0)?;
        self.byte(0x442737, 0)?;
        self.byte(0x442736, 0)?;
        let key = self.w(0x443316)?;
        match key {
            0x1231 => self.byte(0x437a6b, 1)?,
            0x1332 => self.byte(0x437a6b, 2)?,
            0x1433 => self.byte(0x437a6b, 3)?,
            0x1534 => self.byte(0x437a6b, 4)?,
            0x1735 => self.byte(0x437a6b, 5)?,
            0x1636 => self.byte(0x437a6b, 6)?,
            0x1a37 => self.byte(0x437a6b, 7)?,
            0x1c38 => self.byte(0x437a6b, 8)?,
            0x1939 => self.byte(0x437a6b, 9)?,
            0x1d30 => self.byte(0x437a6b, 10)?,
            0x240d => {
                self.ww(0x4471ee, self.sb(0x44273f)?.wrapping_mul(51) as u16)?;
                self.lw(0x439404, self.l(0x439440)?)?;
                self.ww(0x443316, 0)?;
            }
            0x5331 => {
                self.byte(0x442736, 255)?;
                self.byte(0x442737, 1)?;
            }
            0x5432 | 0x7d1f => self.byte(0x442737, 1)?,
            0x5533 => {
                self.byte(0x442736, 1)?;
                self.byte(0x442737, 1)?;
            }
            0x5634 | 0x7b1c => self.byte(0x442736, 255)?,
            0x5836 | 0x7c1d => self.byte(0x442736, 1)?,
            0x5937 => {
                self.byte(0x442736, 255)?;
                self.byte(0x442737, 255)?;
            }
            0x5b38 | 0x7e1e => self.byte(0x442737, 255)?,
            0x5c39 => {
                self.byte(0x442736, 1)?;
                self.byte(0x442737, 255)?;
            }
            _ => (),
        }
        if self.w(frame.wrapping_add(8))? != 0 {
            if self.w(0x438776)? == 0 && self.b(0x437a6b)? != 0 {
                if self.w(0x438776)? == 0 || self.control_order()? < 6 {
                    for offset in [0_u32, 2, 4, 6] {
                        let base =
                            0x43788a_u32.wrapping_add(u32::from(self.sb(0x44273f)? as u16) * 80);
                        let index = u16::from(self.b(0x437a6b)?).wrapping_sub(1) as i16;
                        self.ww(
                            0x4435c6 + offset,
                            self.w(base
                                .wrapping_add((i32::from(index) as u32).wrapping_mul(8))
                                .wrapping_add(offset))?,
                        )?;
                    }
                    self.spell(frame.wrapping_sub(24))?;
                    self.byte(0x435662, 1)?;
                }
                if self.w(0x438776)? != 0 {
                    let address = warrior(self.w(0x4435d2)?).wrapping_add(0xa66);
                    self.lw(address, self.l(address)?.wrapping_add(1))?;
                }
                let modifiers = self.trap(0xa858, vec![L(u32::from(self.w(0x447218)?)), L(256)])?;
                if modifiers != 0 {
                    self.ww(
                        frame.wrapping_sub(2),
                        self.w(warrior(self.sb(0x44273f)? as u16).wrapping_add(0x58))?,
                    )?;
                    let cost = u16::from(self.b(0x44376c)?).wrapping_mul(self.w(0x4435cc)?);
                    self.lib(0x400038, vec![W(cost)])?;
                    if (self.d0.expect("observed absolute result") as u16 as i16)
                        > self.sw(frame.wrapping_sub(2))?
                    {
                        self.trap(0xa873, vec![L(self.l(0x43878c)?)])?;
                        self.trap(0xaa0b, vec![L(self.l(0x4358ce)?)])?;
                        self.byte(0x442752, 1)?;
                        self.key_text(frame.wrapping_sub(28))?;
                        if self.plan.flow != LoadFlow::Returned {
                            return Ok(());
                        }
                        self.byte(0x442752, 0)?;
                        self.trap(0xa889, vec![W(1)])?;
                        self.trap(0xa88a, vec![W(20)])?;
                        self.trap(0xa862, vec![L(69)])?;
                        self.key_position()?;
                        self.trap(0xa884, vec![P(0x403ede)])?;
                        self.sound(-143)?;
                    } else if self.b(0x443764)? == 0 || self.w(0x4435cc)? == 0 {
                        self.sound(143)?;
                    } else {
                        let cost = u16::from(self.b(0x44376c)?).wrapping_mul(self.w(0x4435cc)?);
                        self.lib(0x400038, vec![W(cost)])?;
                        let cost = self.d0.expect("observed absolute result") as u16;
                        let mana = warrior(self.sb(0x44273f)? as u16).wrapping_add(0x58);
                        self.ww(mana, self.w(mana)?.wrapping_sub(cost))?;
                        self.byte(
                            0x442580_u32.wrapping_add(
                                (i32::from(self.sb(0x44273f)?) as u32).wrapping_mul(2),
                            ),
                            self.b(0x4435c9)?,
                        )?;
                        self.byte(
                            0x442581_u32.wrapping_add(
                                (i32::from(self.sb(0x44273f)?) as u32).wrapping_mul(2),
                            ),
                            self.b(0x4435cb)?,
                        )?;
                        self.ww(
                            0x437a7a_u32.wrapping_add(
                                (i32::from(self.sb(0x44273f)?) as u32).wrapping_mul(2),
                            ),
                            self.w(0x4435cc)?,
                        )?;
                        self.byte(0x437a6a, 1)?;
                        self.ww(
                            0x442798_u32.wrapping_add(
                                (i32::from(self.sb(0x44273f)?) as u32).wrapping_mul(2),
                            ),
                            1,
                        )?;
                        self.lw(0x439404, self.l(0x439470)?)?;
                    }
                } else {
                    self.trap(0xa873, vec![L(self.l(0x43878c)?)])?;
                    self.byte(0x442752, 1)?;
                    self.key_text(frame.wrapping_sub(28))?;
                    if self.plan.flow != LoadFlow::Returned {
                        return Ok(());
                    }
                    self.byte(0x442752, 0)?;
                    self.trap(0xa88a, vec![W(20)])?;
                    self.trap(0xa889, vec![W(1)])?;
                    self.trap(0xaa0b, vec![L(self.l(0x4358ce)?)])?;
                    self.trap(0xa862, vec![L(69)])?;
                    self.key_position()?;
                    let resource = self
                        .w(0x4435c6)?
                        .wrapping_add(1)
                        .wrapping_mul(1000)
                        .wrapping_add(self.w(0x4435c8)?);
                    self.lib(
                        0x400178,
                        vec![
                            P(0x479ed8),
                            W(resource),
                            W(self.w(0x4435ca)?.wrapping_add(1)),
                        ],
                    )?;
                    if self.w(0x4435cc)? != 0 {
                        self.trap(0xa884, vec![P(0x479ed8)])?;
                        self.trap(0xa884, vec![P(0x403ed8)])?;
                        self.lib(
                            0x400168,
                            vec![L(i32::from(self.sw(0x4435cc)?) as u32), P(0x479ed8)],
                        )?;
                        self.trap(0xa884, vec![P(0x479ed8)])?;
                    } else {
                        self.key_position()?;
                        self.trap(0xa884, vec![P(0x403ec8)])?;
                        self.sound(-143)?;
                        self.byte(0x442752, 1)?;
                        self.key_text(frame.wrapping_sub(30))?;
                        if self.plan.flow != LoadFlow::Returned {
                            return Ok(());
                        }
                        self.byte(0x442752, 0)?;
                    }
                }
            }
        } else {
            if self.w(0x438776)? == 0 {
                let control = match self.w(0x443316)? {
                    0x61 => Some(0x43944c),
                    0x173 => Some(0x439470),
                    0x468 => Some(0x439478),
                    0x567 if self.b(0x442753)? != 0 => Some(0x439430),
                    0x778 => Some(0x43947c),
                    0x863 => Some(0x439458),
                    0xe65 => Some(0x439438),
                    0xf72 => Some(0x439434),
                    0x1174 if self.b(0x44273a)? != 0 => Some(0x439444),
                    0x2269 => Some(0x43945c),
                    0x256c if self.b(0x442758)? != 0 => Some(0x439474),
                    _ => None,
                };
                if let Some(control) = control {
                    self.lw(0x439404, self.l(control)?)?;
                }
            }
            if self.w(0x438776)? == 0 {
                let row = self
                    .l(0x442764)?
                    .wrapping_add(self.l(0x442778)?)
                    .wrapping_add(i32::from(self.sb(0x442736)?) as u32)
                    as i32;
                if !(0..=89).contains(&row) {
                    destination = self.layout(frame.wrapping_sub(22), self.l(0x442774)?)? as i16;
                    if destination == -2 {
                        self.byte(0x442737, 0)?;
                        self.byte(0x442736, 0)?;
                    } else {
                        transition = true;
                    }
                }
                let column = self
                    .l(0x442768)?
                    .wrapping_add(self.l(0x44277c)?)
                    .wrapping_add(i32::from(self.sb(0x442737)?) as u32)
                    as i32;
                if !(0..=89).contains(&column) {
                    destination = self.layout(frame.wrapping_sub(22), self.l(0x442774)?)? as i16;
                    if destination == -2 {
                        self.byte(0x442737, 0)?;
                        self.byte(0x442736, 0)?;
                    } else {
                        transition = true;
                    }
                }
                if transition {
                    self.save_land(frame.wrapping_sub(22), self.l(0x442774)?)?;
                    if self.plan.flow != LoadFlow::Returned {
                        return Ok(());
                    }
                    self.lw(0x442774, i32::from(destination) as u32)?;
                    if self.b(0x442736)? != 0 {
                        if self.l(0x442764)? == 0 {
                            self.lw(0x442778, 89)?;
                            self.lw(0x442764, 9)?;
                        } else if self.l(0x442764)? == 9 {
                            self.lw(0x442778, 0)?;
                            self.lw(0x442764, 0)?;
                        }
                    }
                    if self.b(0x442737)? != 0 {
                        if self.l(0x442768)? == 0 {
                            self.lw(0x44277c, 89)?;
                            self.lw(0x442768, 9)?;
                        } else if self.l(0x442768)? == 9 {
                            self.lw(0x44277c, 0)?;
                            self.lw(0x442768, 0)?;
                        }
                    }
                    self.load_land(frame.wrapping_sub(22), self.l(0x442774)?)?;
                    if self.plan.flow != LoadFlow::Returned {
                        return Ok(());
                    }
                    self.center_pict(frame.wrapping_sub(22))?;
                    self.byte(0x442737, 0)?;
                    self.byte(0x442736, 0)?;
                }
            }
            if self.b(0x442736)? != 0 || self.b(0x442737)? != 0 {
                self.lw(0x439404, self.l(0x439454)?)?;
            }
        }
        self.full(u32::from(self.b(0x437a6a)?));
        Ok(())
    }
    fn control_order(&self) -> Result<i16, RuleError> {
        self.sw(0x4470b8_u32.wrapping_add((i32::from(self.sw(0x447194)?) as u32).wrapping_mul(2)))
    }
    fn layout(&mut self, frame: u32, argument: u32) -> Result<u16, RuleError> {
        self.lw(frame.wrapping_add(8), argument)?;
        self.ww(frame.wrapping_sub(2), 0)?;
        self.filename(0x403ec0, false)?;
        let handle = self.lib(0x400070, vec![P(0x403ebc), P(0x4389a4)])?;
        if handle == 0 {
            self.plan.effects.push(FieldControlEffect::LayoutRead(None));
            self.d0 = Some((-2_i32) as u32);
            return Ok((-2_i16) as u16);
        }
        self.lib(0x400048, vec![L(handle), L(1), L(256), P(0x435558)])?;
        self.lib(0x400068, vec![L(handle)])?;
        let layout: [u8; 256] = self.bytes(0x435558, 256)?.try_into().unwrap();
        self.plan
            .effects
            .push(FieldControlEffect::LayoutRead(Some(layout.to_vec())));
        let input = LayoutInput {
            argument: argument as i32,
            horizontal_transition: self.b(0x442736)?,
            vertical_transition: self.b(0x442737)?,
            first_coordinate: self.l(0x442764)? as i32,
            second_coordinate: self.l(0x442768)? as i32,
            previous_layout: layout,
        };
        let child = spatial::checklayout(input, &FixedLayout(Some(layout.to_vec())))
            .map_err(|e| RuleError::Invalid(e.to_string()))?;
        self.d0 = Some(child.native_return);
        Ok(child.native_return as u16)
    }
    fn spell(&mut self, frame: u32) -> Result<(), RuleError> {
        self.ww(frame.wrapping_add(12), self.w(0x4435ca)?)?;
        self.ww(frame.wrapping_add(10), self.w(0x4435c8)?)?;
        self.ww(frame.wrapping_add(8), self.w(0x4435c6)?)?;
        let school = self.b(frame.wrapping_add(9))? as i8;
        let level = self.b(frame.wrapping_add(11))? as i8;
        let index = self.b(frame.wrapping_add(13))? as i8;
        let offset = usize::from(i16::from(school) as u16) * 2842
            + usize::from(i16::from(level) as u16) * 406
            + usize::from(i16::from(index) as u16) * SPELL_BYTES;
        let length = offset
            .checked_add(SPELL_BYTES)
            .ok_or_else(|| RuleError::Invalid("spell view wraps".into()))?;
        if length > 1024 * 1024 {
            return Err(RuleError::Unsupported {
                action: "field_control.loadspell",
                dependency: "the captured record view exceeds one MiB",
            });
        }
        let table = self.bytes(0x443788, length)?;
        let child = records::loadspell(&table, school, level, index)?;
        for write in child.writes {
            self.write(write.address, &write.bytes)?;
        }
        self.d0 = Some(child.native_return as u32);
        Ok(())
    }
    fn prep_attack(&mut self, frame: u32) -> Result<(), RuleError> {
        let actor = self.sw(frame.wrapping_add(8))?;
        let active = self.w(0x4435d2)?;
        let incoming = if actor >= 10 {
            self.services
                .incoming_attack_slot(&self.plan.state.memory)?
        } else {
            0
        };
        let highest = u32::from((actor as u16).max(active));
        let warrior_length = (highest as usize + 1)
            .checked_mul(WARRIOR_BYTES)
            .ok_or_else(|| RuleError::Invalid("attack warrior view wraps".into()))?;
        if warrior_length > 1024 * 1024 {
            return Err(RuleError::Unsupported {
                action: "field_control.prepattack",
                dependency: "the captured warrior view exceeds one MiB",
            });
        }
        let selected = warrior(actor as u16);
        if [0x44374e, 0x45363c, 0x4536d2, 0x453768]
            .into_iter()
            .any(|a| overlaps(selected, WARRIOR_BYTES, a, ITEM_BYTES))
        {
            return Err(RuleError::Unsupported {
                action: "field_control.prepattack",
                dependency: "the selected captured warrior aliases an attack work buffer",
            });
        }
        let warrior_memory = self.bytes(0x448aa0, warrior_length)?;
        let mut families: [Vec<u8>; 5] = std::array::from_fn(|_| vec![]);
        for (n, a) in records::ITEM_TABLE_ADDRESSES.into_iter().enumerate() {
            families[n] = self.bytes(a, 200 * ITEM_BYTES)?;
        }
        let spells = self.bytes(0x443788, 0x2c68)?;
        let current = AttackWork {
            item: self.bytes(0x45363c, ITEM_BYTES)?.try_into().unwrap(),
            ammunition: self.bytes(0x4536d2, ITEM_BYTES)?.try_into().unwrap(),
            weapon: self.bytes(0x453768, ITEM_BYTES)?.try_into().unwrap(),
            packet: self.bytes(0x44374e, SPELL_BYTES)?.try_into().unwrap(),
            spell_coordinates: [self.sw(0x4435c6)?, self.sw(0x4435c8)?, self.sw(0x4435ca)?],
            attack_count: self.sw(0x4435cc)?,
            damage_total: self.sw(0x41f714)?,
        };
        let tables = AttackTables {
            warrior_memory: &warrior_memory,
            items: ItemTables {
                families: std::array::from_fn(|n| families[n].as_slice()),
            },
            spells: &spells,
        };
        let child = attack::prepattack(
            &current,
            &tables,
            actor,
            self.sw(frame.wrapping_add(10))?,
            PreparationContext {
                active_member: active,
                selected_item: self.sw(0x44719a)?,
                entry_inventory_slot: incoming,
                frame_return_word: self.sw(frame.wrapping_sub(6))?,
            },
        )?;
        for effect in child.effects {
            if let AttackEffect::Write(write) = &effect {
                self.write(write.address, &write.bytes)?;
            } else {
                self.plan.effects.push(FieldControlEffect::Attack(effect));
            }
        }
        self.d0 = None;
        Ok(())
    }
    fn do_reload(&mut self, frame: u32) -> Result<(), RuleError> {
        self.step()?;
        let preparation = frame.wrapping_sub(12);
        self.ww(preparation.wrapping_add(10), 0)?;
        self.ww(preparation.wrapping_add(8), self.w(frame.wrapping_add(8))?)?;
        self.prep_attack(preparation)?;
        self.ww(frame.wrapping_sub(6), 55)?;
        self.location(55);
        let charges = self.sw(warrior(self.w(0x4435d2)?).wrapping_add(0x134))?;
        if charges < self.sw(0x45377e)? {
            let refill = frame.wrapping_sub(16);
            self.ww(refill.wrapping_add(14), 1)?;
            self.lw(refill.wrapping_add(10), 0x00010001)?;
            self.ww(refill.wrapping_add(8), self.w(frame.wrapping_add(8))?)?;
            let child = equipment::refit(
                &self.plan.state.memory,
                &self.plan.state.random,
                refill,
                self.limits.equipment,
                self.services,
            )?;
            let returned = child.native_return.low_word;
            self.equipment(child);
            if returned == 0 {
                self.ww(warrior(self.w(0x4435d2)?).wrapping_add(0x90), 0)?;
                self.ww(0x438764, 0)?;
                self.ww(0x438760, 0)?;
                self.byte(
                    0x442742_u32.wrapping_add(i32::from(self.sw(0x4435d2)?) as u32),
                    0,
                )?;
                self.lw(0x439404, 0)?;
                self.trap(0xa873, vec![L(self.l(0x43879c)?)])?;
                self.trap(0xa862, vec![L(33)])?;
                self.trap(0xa863, vec![L(30)])?;
                self.warn(frame.wrapping_sub(10), 25)?;
                self.full(0);
                return Ok(());
            }
        }
        self.ww(frame.wrapping_sub(2), 60)?;
        self.location(60);
        self.full(0);
        Ok(())
    }
    fn setup_new_game(&mut self, frame: u32) -> Result<(), RuleError> {
        self.step()?;
        self.lw(frame.wrapping_sub(10), 0)?;
        self.lw(frame.wrapping_sub(14), 0)?;
        self.lw(frame.wrapping_sub(6), 0)?;
        let count = self.trap(0xa99c, vec![L(0x4e455743)])? as u16;
        self.ww(0x4434aa, count.wrapping_sub(1))?;
        if self.w(0x442560)? != self.w(0x4434aa)? && self.w(0x4434aa)? != 0 {
            self.warn(frame.wrapping_sub(28), 92)?;
            self.scratch(frame.wrapping_sub(30), 0xfe0a)?;
            if self.plan.flow != LoadFlow::Returned {
                return Ok(());
            }
        }
        self.trap(0xa981, vec![L(self.l(0x479fd8)?)])?;
        self.trap(0xa939, vec![L(self.l(0x43872a)?), W(0)])?;
        self.trap(0xa939, vec![L(self.l(0x43871e)?), W(0)])?;
        self.trap(0xa93a, vec![L(self.l(0x43871e)?), W(5)])?;
        self.ww(0x4434aa, 0)?;
        while self.sw(0x4434aa)? < 90 {
            self.step()?;
            self.ww(0x4434ac, 0)?;
            while self.sw(0x4434ac)? < 90 {
                self.step()?;
                let address = 0x4358e6_u32
                    .wrapping_add(u32::from(self.w(0x4434aa)?) * 90)
                    .wrapping_add(i32::from(self.sw(0x4434ac)?) as u32);
                self.byte(address, 0)?;
                self.addw(0x4434ac, 1)?;
            }
            self.addw(0x4434aa, 1)?;
        }
        self.byte(0x4386a9, 0)?;
        self.lib(
            0x400148,
            vec![W(0), P(0x479ed8), P(frame.wrapping_sub(2)), P(0x4427b6)],
        )?;
        if (self.l(0x4427b6)? as i32) < 0x7d000 {
            self.byte(0x4386a9, 1)?;
        }
        self.ww(0x43874c, 0)?;
        self.ww(0x43874a, 0)?;
        self.ww(0x4434aa, 0)?;
        while self.sw(0x4434aa)? < 10 {
            self.step()?;
            self.ww(
                0x442562_u32.wrapping_add((i32::from(self.sw(0x4434aa)?) as u32).wrapping_mul(2)),
                0,
            )?;
            self.byte(
                0x442722_u32.wrapping_add(i32::from(self.sw(0x4434aa)?) as u32),
                0,
            )?;
            self.byte(
                0x442722_u32.wrapping_add(i32::from(self.sw(0x4434aa)?.wrapping_add(10)) as u32),
                0,
            )?;
            self.addw(0x4434aa, 1)?;
        }
        self.menu(frame.wrapping_sub(26), 0x13b212)?;
        if self.plan.flow != LoadFlow::Returned {
            return Ok(());
        }
        let child = storage::reset_game_files(
            &self.plan.state.memory,
            frame.wrapping_sub(26),
            self.storage_limits(),
            self.services,
        )?;
        self.plan.state.memory = child.memory;
        self.plan.flow = child.flow;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(FieldControlEffect::Storage));
        if self.plan.flow != LoadFlow::Returned {
            self.plan.native_return = PromptReturn::NoReturn;
            return Ok(());
        }
        self.byte(0x44274c, 0)?;
        self.ww(0x4434aa, 0)?;
        while self.sw(0x4434aa)? < 6 {
            self.step()?;
            self.byte(
                0x442580_u32.wrapping_add((i32::from(self.sw(0x4434aa)?) as u32).wrapping_mul(2)),
                0,
            )?;
            self.byte(
                0x442581_u32.wrapping_add((i32::from(self.sw(0x4434aa)?) as u32).wrapping_mul(2)),
                0,
            )?;
            self.byte(warrior(self.w(0x4434aa)?).wrapping_add(0x9df), 1)?;
            self.addw(0x4434aa, 1)?;
        }
        self.byte(0x4386a9, 0)?;
        self.trap(
            0xa946,
            vec![L(self.l(0x43870e)?), W(self.w(0x442560)?), P(0x479ed8)],
        )?;
        self.lib(0x400180, vec![P(0x479ed8)])?;
        self.filename(0x479ed8, false)?;
        let handle = self.lib(0x400070, vec![P(0x401466), P(0x4389a4)])?;
        if handle == 0 {
            self.scratch(frame.wrapping_sub(28), 189)?;
            if self.plan.flow != LoadFlow::Returned {
                return Ok(());
            }
        }
        self.lib(0x400048, vec![L(handle), L(5), L(4), P(0x44276c)])?;
        self.lib(0x400048, vec![L(handle), L(1), L(20), P(0x435682)])?;
        self.lib(0x400048, vec![L(handle), L(1), L(20), P(0x435782)])?;
        self.lib(0x400068, vec![L(handle)])?;
        self.ww(0x4434aa, 0)?;
        while self.sw(0x4434aa)? < 128 {
            self.step()?;
            let value = if i32::from(self.sw(0x4434aa)?) > self.l(0x4427b6)? as i32 {
                255
            } else {
                0
            };
            self.byte(
                0x44258c_u32.wrapping_add(i32::from(self.sw(0x4434aa)?) as u32),
                value,
            )?;
            self.addw(0x4434aa, 1)?;
        }
        self.byte(0x44258c, 255)?;
        self.ww(frame.wrapping_sub(20), self.w(0x442560)?)?;
        self.menu(frame.wrapping_sub(28), 0x135bb2)?;
        if self.plan.flow != LoadFlow::Returned {
            return Ok(());
        }
        self.ww(0x442578, 4)?;
        self.byte(0x402f42, 1)?;
        self.byte(0x442754, 1)?;
        self.ww(0x4394b4, 0)?;
        self.ww(0x44257c, 0)?;
        self.byte(0x44274e, 0)?;
        self.byte(0x442739, 0)?;
        self.ww(0x438776, 0)?;
        self.byte(0x44274f, 0)?;
        self.ww(0x438760, 0)?;
        self.byte(0x442753, 0)?;
        self.byte(0x442759, 0)?;
        self.lw(0x442768, 0)?;
        self.byte(0x442748, 0)?;
        self.lw(0x442764, 0)?;
        self.lw(0x442788, 0)?;
        self.ww(0x408578, 0)?;
        self.ww(0x40857a, 0)?;
        self.lw(0x44278c, 0)?;
        self.ww(0x4394b6, 0)?;
        let answer = self.question(frame.wrapping_sub(32), 0x00120001)?;
        if self.plan.flow != LoadFlow::Returned {
            return Ok(());
        }
        if answer != 0 {
            self.ww(0x4434aa, 0)?;
            while self.sw(0x4434aa)? < 200 {
                self.step()?;
                self.lw(
                    0x42dbe4_u32
                        .wrapping_add((i32::from(self.sw(0x4434aa)?) as u32).wrapping_mul(4)),
                    0,
                )?;
                self.addw(0x4434aa, 1)?;
            }
            self.ww(0x4434aa, 0)?;
            while self.sw(0x4434aa)? < 1000 {
                self.step()?;
                self.copy(
                    0x41fc32,
                    0x4285f4_u32.wrapping_add(u32::from(self.w(0x4434aa)?) * 22),
                    22,
                )?;
                self.addw(0x4434aa, 1)?;
            }
        }
        self.ww(frame.wrapping_sub(20), 0)?;
        self.menu(frame.wrapping_sub(28), 0x133f3e)?;
        if self.plan.flow != LoadFlow::Returned {
            return Ok(());
        }
        self.load_land(frame.wrapping_sub(32), self.l(0x442774)?)?;
        if self.plan.flow != LoadFlow::Returned {
            return Ok(());
        }
        self.full(0);
        Ok(())
    }
    fn encounter(&mut self, frame: u32) -> Result<(), RuleError> {
        self.step()?;
        self.runtime(0x13e13c)?;
        self.trap(0xaa1c, vec![L(self.l(0x43952c)?)])?;
        if self.w(frame.wrapping_add(10))? != 0 {
            self.sound(20005)?;
            let handle = self.lib(0x400070, vec![P(0x403a88), P(0x403a8c)])?;
            if handle == 0 {
                self.scratch(frame.wrapping_sub(18), 53)?;
                if self.plan.flow != LoadFlow::Returned {
                    return Ok(());
                }
            }
            self.lib(
                0x400078,
                vec![
                    W(0),
                    L(u32::from(self.w(frame.wrapping_add(8))?) * 426),
                    L(handle),
                ],
            )?;
            self.lib(0x400048, vec![L(handle), L(1), L(106), P(0x443432)])?;
            self.ww(0x4434aa, 0)?;
            while self.sw(0x4434aa)? < 4 {
                self.step()?;
                let destination = 0x438af4_u32
                    .wrapping_add((i32::from(self.sw(0x4434aa)?) as u32).wrapping_shl(8));
                self.lib(0x400178, vec![P(destination), L(0x10003)])?;
                let destination = 0x438af4_u32
                    .wrapping_add((i32::from(self.sw(0x4434aa)?) as u32).wrapping_shl(8));
                self.lib(0x400048, vec![L(handle), L(1), L(80), P(destination)])?;
                self.addw(0x4434aa, 1)?;
            }
            self.lib(0x400068, vec![L(handle)])?;
        }
        self.ww(0x4434aa, 0)?;
        while self.sw(0x4434aa)? < 4 {
            self.step()?;
            let choice = 0x443492_u32.wrapping_add(i32::from(self.sw(0x4434aa)?) as u32);
            if self.b(choice)? == 0 {
                let destination = 0x438af4_u32
                    .wrapping_add((i32::from(self.sw(0x4434aa)?) as u32).wrapping_shl(8));
                self.lib(0x400178, vec![P(destination), L(0x10003)])?;
            }
            self.addw(0x4434aa, 1)?;
        }
        self.trap(
            0xa98b,
            vec![P(0x438af4), P(0x438bf4), P(0x438cf4), P(0x438df4)],
        )?;
        self.ww(frame.wrapping_sub(10), 0)?;
        self.lw(frame.wrapping_sub(14), 0)?;
        self.lib(0x400038, vec![W(self.w(0x44349a)?)])?;
        let index = (self.d0.expect("observed absolute result") as u16).wrapping_neg();
        self.ww(frame.wrapping_sub(16), index)?;
        self.ww(frame.wrapping_sub(18), u16::MAX)?;
        self.textbox(frame.wrapping_sub(26))?;
        if self.plan.flow != LoadFlow::Returned {
            return Ok(());
        }
        self.runtime(0x13a498)?;
        self.trap(0xaa1c, vec![L(self.l(0x43952c)?)])?;
        let dialog = self.trap(0xa97c, vec![W(147), L(0), L(u32::MAX)])?;
        self.trap(0xa873, vec![L(dialog)])?;
        self.trap(0xa887, vec![W(self.w(0x402f44)?)])?;
        self.trap(0xa862, vec![L(69)])?;
        let dimensions = if self.b(0x443496)? == 0 {
            0x010b0103
        } else {
            0x01400103
        };
        self.trap(0xa91d, vec![L(dialog), L(dimensions), B(0)])?;
        let horizontal = half(self.sw(0x40856c)?)
            .wrapping_add(self.sw(0x4395a4)?)
            .wrapping_add(30);
        let vertical = half(self.sw(0x40856e)?).wrapping_add(self.sw(0x4395a2)?);
        self.trap(
            0xa91b,
            vec![L(dialog), W(horizontal as u16), W(vertical as u16), B(0)],
        )?;
        self.trap(0xa915, vec![L(dialog)])?;
        self.trap(0xaa0b, vec![L(self.l(0x4358ce)?)])?;
        self.registers(0xa032, Some(0xffff), None, vec![])?;
        self.delay(15)?;
        loop {
            self.step()?;
            self.trap(0xa91f, vec![L(dialog)])?;
            self.trap(0xa873, vec![L(dialog)])?;
            self.registers(0xa032, Some(0xffff), None, vec![])?;
            self.trap(0xa991, vec![L(0), P(0x4434b2)])?;
            if self.w(0x4434b2)? == 9 {
                self.ww(0x4471ea, 0)?;
                break;
            }
            let address =
                0x443492_u32.wrapping_add(i32::from(self.sw(0x4434b2)?.wrapping_sub(1)) as u32);
            if self.b(address)? != 0 {
                self.ww(0x4471ea, self.w(0x4434b2)?)?;
                break;
            }
            self.sound(6000)?;
        }
        self.trap(0xa983, vec![L(dialog)])?;
        self.trap(0xaa1c, vec![L(self.l(0x43952c)?)])?;
        self.center_pict(frame.wrapping_sub(16))?;
        if self.w(0x4471ea)? == 0 {
            self.full(0);
        } else {
            let address =
                0x443492_u32.wrapping_add(i32::from(self.sw(0x4471ea)?.wrapping_sub(1)) as u32);
            self.low(i16::from(self.b(address)? as i8) as u16);
        }
        Ok(())
    }
    fn movie_copy(&mut self, frame: u32, edit: u32, window: u32) -> Result<(), RuleError> {
        self.trap(0xa8a3, vec![P(0x4471b4)])?;
        self.trap(0xa9d3, vec![P(frame.wrapping_sub(8)), L(edit)])?;
        self.trap(
            0xa8ec,
            vec![
                P(self.l(frame.wrapping_sub(12))?.wrapping_add(2)),
                P(window.wrapping_add(2)),
                P(0x4471b4),
                P(0x4471b4),
                W(0),
                L(0),
            ],
        )?;
        Ok(())
    }
    fn movie(&mut self, frame: u32) -> Result<(), RuleError> {
        self.step()?;
        let mut click_limit = 0_u32;
        self.lw(frame.wrapping_sub(8), self.l(0x4471b4)?)?;
        self.lw(frame.wrapping_sub(4), self.l(0x4471b8)?)?;
        self.registers(0xab1d, Some(0x80005), None, vec![P(0x4046da), P(0x4046de)])?;
        self.ww(
            0x4471ba,
            self.w(0x439582)?.wrapping_mul(160).wrapping_add(320),
        )?;
        self.ww(
            0x4471b8,
            self.w(0x439582)?
                .wrapping_shl(7)
                .wrapping_add(320)
                .wrapping_sub(self.w(0x439582)?.wrapping_shl(5)),
        )?;
        self.ww(0x4471b4, 0)?;
        self.ww(0x4471b6, 0)?;
        self.ww(frame.wrapping_add(10), 131)?;
        let window = self.trap(0xaa46, vec![W(133), L(0), L(u32::MAX)])?;
        self.trap(
            0xa91d,
            vec![L(window), W(self.w(0x4471ba)?), W(self.w(0x4471b8)?), B(0)],
        )?;
        self.trap(0xa873, vec![L(window)])?;
        self.trap(0xa862, vec![L(33)])?;
        self.trap(0xa863, vec![L(30)])?;
        let pattern = self.trap(0xaa0c, vec![W(self.w(frame.wrapping_add(10))?)])?;
        self.trap(0xaa0b, vec![L(pattern)])?;
        self.trap(0xa889, vec![W(1)])?;
        if self.trap(0xa924, vec![])? != self.l(0x479fd8)? {
            self.trap(
                0xa91b,
                vec![L(window), W(self.w(0x4395a4)?), W(self.w(0x4395a2)?), B(1)],
            )?;
        }
        self.trap(0xa915, vec![L(window)])?;
        self.trap(0xa8a3, vec![P(0x4471b4)])?;
        self.registers(
            0xab1d,
            Some(0x160000),
            None,
            vec![
                P(frame.wrapping_sub(12)),
                W(32),
                P(0x4471b4),
                L(0),
                L(0),
                L(0),
            ],
        )?;
        self.registers(
            0xab1d,
            Some(0x80006),
            None,
            vec![L(self.l(frame.wrapping_sub(12))?), L(0)],
        )?;
        let pixmap = self.l(self.l(frame.wrapping_sub(12))?.wrapping_add(2))?;
        self.registers(0xab1d, Some(0x40001), None, vec![L(pixmap)])?;
        self.trap(0xaa0b, vec![L(pattern)])?;
        self.trap(0xa862, vec![L(33)])?;
        self.trap(0xa863, vec![L(30)])?;
        self.trap(0xa889, vec![W(1)])?;
        self.trap(0xa8a3, vec![P(0x4471b4)])?;
        let edit = self.trap(
            0xa83e,
            vec![P(frame.wrapping_sub(8)), P(frame.wrapping_sub(8))],
        )?;
        let text_handle = self.trap(
            0xa9a0,
            vec![L(0x54455854), W(self.w(frame.wrapping_add(8))?)],
        )?;
        self.registers(0xa029, None, Some(text_handle), vec![])?;
        let style = self.trap(
            0xa9a0,
            vec![L(0x7374796c), W(self.w(frame.wrapping_add(8))?)],
        )?;
        let text_pointer = self.l(text_handle)?;
        let text_size = self.trap(0xa9a5, vec![L(text_handle)])?;
        self.trap(
            0xa83d,
            vec![P(text_pointer), L(text_size), L(style), L(edit), W(7)],
        )?;
        self.trap(0xa9df, vec![W(self.w(frame.wrapping_add(12))?), L(edit)])?;
        self.registers(0xa02a, None, Some(text_handle), vec![])?;
        self.trap(0xa9d3, vec![P(frame.wrapping_sub(8)), L(edit)])?;
        self.trap(0xa8a9, vec![P(frame.wrapping_sub(8)), L(0xfffffffb)])?;
        self.trap(0xa915, vec![L(window)])?;
        loop {
            self.step()?;
            while self.trap(0xa974, vec![])? as u8 != 0 {
                self.step()?;
            }
            let tick = self.trap(0xa975, vec![])?;
            self.trap(0xa9dd, vec![L(0xffff0000), L(edit)])?;
            self.movie_copy(frame, edit, window)?;
            loop {
                self.step()?;
                self.lw(frame.wrapping_sub(18), tick.wrapping_add(4))?;
                if self.trap(0xa975, vec![])? >= self.l(frame.wrapping_sub(18))? {
                    break;
                }
            }
            self.trap(0xa9b4, vec![])?;
            self.music(frame.wrapping_sub(60))?;
            if self.plan.flow != LoadFlow::Returned {
                return Ok(());
            }
            let available = self.trap(0xa970, vec![W(u16::MAX), P(0x44720a)])? as u8;
            self.ww(frame.wrapping_sub(14), u16::from(available))?;
            match self.w(0x44720a)? {
                1 if click_limit >= self.l(0x447210)? => {
                    self.registers(0xab1d, Some(0x40002), None, vec![L(pixmap)])?;
                    self.registers(
                        0xab1d,
                        Some(0x40004),
                        None,
                        vec![L(self.l(frame.wrapping_sub(12))?)],
                    )?;
                    self.trap(0xaa08, vec![L(pattern)])?;
                    self.trap(0xa9cd, vec![L(edit)])?;
                    self.trap(0xa914, vec![L(window)])?;
                    self.registers(
                        0xab1d,
                        Some(0x80006),
                        None,
                        vec![L(self.l(0x4046da)?), L(self.l(0x4046de)?)],
                    )?;
                    self.center_pict(frame.wrapping_sub(58))?;
                    self.full(0);
                    return Ok(());
                }
                2 => click_limit = self.l(0x2f0)?.wrapping_add(self.l(0x447210)?),
                _ => (),
            }
            while self.trap(0xa973, vec![])? as u8 != 0 {
                self.step()?;
                self.trap(0xa972, vec![P(0x4471ee)])?;
                let current = self.sw(0x4471ee)?;
                let prior = self.sw(0x447214)?;
                if current < prior {
                    self.trap(0xa9dd, vec![L(0xffe70000), L(edit)])?;
                } else if current > prior {
                    self.trap(0xa9dd, vec![L(0x00190000), L(edit)])?;
                }
                self.ww(0x447214, self.w(0x4471ee)?)?;
                self.movie_copy(frame, edit, window)?;
            }
        }
    }
    fn save_character(&mut self, frame: u32) -> Result<(), RuleError> {
        self.step()?;
        let member = self.w(frame.wrapping_add(8))?;
        if (member as i16) < 0 {
            self.ww(frame.wrapping_sub(0x1892), u16::MAX)?;
            self.verify(u16::MAX, false)?;
            self.copy(0x44721c, frame.wrapping_sub(0xc42), WARRIOR_BYTES)?;
        } else {
            self.copy(warrior(member), frame.wrapping_sub(0x1884), WARRIOR_BYTES)?;
            let mut slot = self.w(warrior(member).wrapping_add(0x1a))?.wrapping_sub(1) as i16;
            while slot > -1 {
                self.step()?;
                let node = warrior(member)
                    .wrapping_add(u32::from(slot as u16) * 22)
                    .wrapping_add(0x132);
                self.load_item_no_type(self.sw(node)?)?;
                let kind = self.sw(0x453656)?;
                if kind == 23 || kind == 24 || kind == 25 || kind < 0 {
                    self.lib(0x400038, vec![W(kind as u16)])?;
                    self.ww(0x453656, self.d0.expect("observed absolute result") as u16)?;
                    let child_frame = frame.wrapping_sub(0x18a2);
                    self.ww(child_frame.wrapping_add(14), 0)?;
                    self.ww(child_frame.wrapping_add(12), slot as u16)?;
                    self.ww(child_frame.wrapping_add(10), self.w(node)?)?;
                    self.ww(child_frame.wrapping_add(8), member)?;
                    let child = equipment::dropitem(
                        &self.plan.state.memory,
                        &self.plan.state.random,
                        child_frame,
                        self.limits.equipment,
                        self.services,
                    )?;
                    self.equipment(child);
                }
                slot = slot.wrapping_sub(1);
            }
            self.ww(frame.wrapping_sub(0x1892), member)?;
            self.verify(member, false)?;
            self.copy(warrior(member), frame.wrapping_sub(0xc42), WARRIOR_BYTES)?;
            self.copy(frame.wrapping_sub(0x1884), warrior(member), WARRIOR_BYTES)?;
        }
        let minus = frame.wrapping_sub(0x189e);
        self.ww(minus.wrapping_add(12), 1)?;
        self.lw(minus.wrapping_add(8), frame.wrapping_sub(0x221))?;
        self.edit(minus, false)?;
        if self.plan.flow != LoadFlow::Returned {
            return Ok(());
        }
        let plus = frame.wrapping_sub(0x18a4);
        self.ww(plus.wrapping_add(12), self.w(frame.wrapping_sub(0xc08))?)?;
        self.lw(plus.wrapping_add(8), frame.wrapping_sub(0x221))?;
        self.edit(plus, true)?;
        if self.plan.flow != LoadFlow::Returned {
            return Ok(());
        }
        let registered = verification::doreg();
        self.d0 = Some(registered as u32);
        if registered != 0 || self.sw(frame.wrapping_sub(0xc08))? < 4 {
            self.lib(0x400090, vec![P(0x40246a), P(0x41f608)])?;
            self.lib(0x400098, vec![P(frame.wrapping_sub(0x221)), P(0x41f608)])?;
            let handle = self.lib(0x400070, vec![P(0x402466), P(0x41f608)])?;
            if handle == 0 {
                self.scratch(frame.wrapping_sub(0x189a), 2)?;
                if self.plan.flow != LoadFlow::Returned {
                    return Ok(());
                }
            }
            self.lib(
                0x400050,
                vec![L(handle), L(1), L(3138), P(frame.wrapping_sub(0xc42))],
            )?;
            self.lib(0x400068, vec![L(handle)])?;
            let profile_frame = frame.wrapping_sub(0x18b0);
            self.ww(
                profile_frame.wrapping_add(10),
                self.w(frame.wrapping_sub(0xc0e))?,
            )?;
            self.ww(profile_frame.wrapping_add(8), 0)?;
            self.profile(
                self.sw(profile_frame.wrapping_add(8))?,
                self.sw(profile_frame.wrapping_add(10))?,
            )?;
            let caste = self.w(0x41faf0)?;
            if (1..=7).contains(&caste) {
                self.metadata(
                    0x402460_u32.wrapping_sub(u32::from(caste - 1) * 6),
                    0x41f608,
                )?;
            }
        }
        self.full(0);
        Ok(())
    }
    fn load_shop(&mut self, frame: u32) -> Result<(), RuleError> {
        self.step()?;
        let mode = self.w(frame.wrapping_add(8))?;
        if mode != 0 {
            self.byte(0x44273b, mode as u8)?;
        }
        self.filename(0x4031b2, true)?;
        let handle = self.lib(0x400070, vec![P(0x40327a), P(0x4389a4)])?;
        if handle != 0 {
            self.lib(
                0x400078,
                vec![
                    W(0),
                    L(u32::from(self.sb(0x44273b)? as u16) * 0xbba),
                    L(handle),
                ],
            )?;
            self.lib(
                0x400048,
                vec![L(handle), L(1), L(0xbba), P(frame.wrapping_sub(0xbba))],
            )?;
            self.lib(0x400068, vec![L(handle)])?;
        }
        if mode == 0 {
            self.byte(0x442753, 1)?;
        }
        if self.l(0x43879c)? != 0 && mode == 0 {
            self.controls(frame.wrapping_sub(0xbd2))?;
            if self.plan.flow != LoadFlow::Returned {
                return Ok(());
            }
            self.sound(30005)?;
        }
        self.ww(0x478e40, self.w(frame.wrapping_sub(2))?)?;
        for family in 0..5_i16 {
            let first = family * 200;
            let mut destination = first;
            let mut source = first;
            while source < first + 200 {
                self.step()?;
                let count = frame
                    .wrapping_sub(0xbba)
                    .wrapping_add(source as u32)
                    .wrapping_add(0x7d0);
                if self.b(count)? != 0 {
                    self.byte(
                        0x478a58_u32.wrapping_add(destination as u32),
                        self.b(count)?,
                    )?;
                    let target = destination;
                    destination = destination.wrapping_add(1);
                    self.ww(
                        0x478288_u32.wrapping_add(u32::from(target as u16) * 2),
                        self.w(frame
                            .wrapping_sub(0xbba)
                            .wrapping_add(u32::from(source as u16) * 2))?,
                    )?;
                }
                source = source.wrapping_add(1);
            }
        }
        self.full(0);
        Ok(())
    }
    fn band_aid(&mut self, frame: u32) -> Result<(), RuleError> {
        self.step()?;
        if self.sb(0x44273a)? < 0 {
            self.full(0);
            return Ok(());
        }
        self.menu(frame.wrapping_sub(20), 0x13b212)?;
        if self.plan.flow != LoadFlow::Returned {
            return Ok(());
        }
        let mut invalid = false;
        let mut member = 0_i16;
        while member <= self.sb(0x44273a)? {
            self.step()?;
            self.ww(frame.wrapping_sub(14), member as u16)?;
            self.verify(member as u16, true)?;
            let profile_frame = frame.wrapping_sub(26);
            self.ww(
                profile_frame.wrapping_add(10),
                self.w(warrior(member as u16).wrapping_add(0x34))?,
            )?;
            self.ww(
                profile_frame.wrapping_add(8),
                self.w(warrior(member as u16).wrapping_add(0x32))?,
            )?;
            self.profile(
                self.sw(profile_frame.wrapping_add(8))?,
                self.sw(profile_frame.wrapping_add(10))?,
            )?;
            if self.sw(warrior(member as u16))? < 1 {
                invalid = true;
            }
            member = member.wrapping_add(1);
        }
        if invalid {
            self.flash(frame.wrapping_sub(32), 0x401e70, 0x00320028, 6000)?;
            self.flash(frame.wrapping_sub(44), 0x401e22, 0x00320028, 6000)?;
            self.lib(0x400028, vec![W(0)])?;
            self.plan.flow = LoadFlow::Terminated(0);
            self.d0 = None;
            self.plan.native_return = PromptReturn::NoReturn;
            return Ok(());
        }
        self.full(0);
        Ok(())
    }
    fn enter_capture_mode(&mut self, frame: u32) -> Result<(), RuleError> {
        self.step()?;
        self.trap(0xaa1c, vec![L(self.l(0x43953c)?)])?;
        self.byte(0x43565f, 1)?;
        let field = frame.wrapping_sub(12);
        self.ww(field.wrapping_add(10), self.w(0x408572)?)?;
        self.ww(field.wrapping_add(8), self.w(0x408570)?)?;
        self.center_field(field)?;
        self.flash(frame.wrapping_sub(24), 0x4024de, 0x001e012c, 0xffff1388)?;
        self.trap(0xa873, vec![L(self.l(0x43879c)?)])?;
        let tutorial = frame.wrapping_sub(28);
        self.lw(tutorial.wrapping_add(8), 0x00710073)?;
        self.tutorial(tutorial)?;
        if self.plan.flow != LoadFlow::Returned {
            return Ok(());
        }
        self.full(0);
        Ok(())
    }
    fn move_party(&mut self, frame: u32) -> Result<(), RuleError> {
        self.step()?;
        let distance = self.sw(frame.wrapping_add(8))?;
        if distance != 0 {
            let row = self
                .l(0x442764)?
                .wrapping_add(self.l(0x442778)?)
                .wrapping_add(i32::from(self.sb(0x442736)?) as u32);
            let column = self
                .l(0x442768)?
                .wrapping_add(self.l(0x44277c)?)
                .wrapping_add(i32::from(self.sb(0x442737)?) as u32);
            let tile = 0x4085b8_u32
                .wrapping_add(row.wrapping_mul(180))
                .wrapping_add(column.wrapping_mul(2));
            self.lib(0x400038, vec![W(self.w(tile)?)])?;
            self.ww(0x4434b6, self.d0.expect("observed absolute result") as u16)?;
            self.byte(0x442754, 1)?;
            if self.b(0x442753)? != 0 || self.b(0x442758)? != 0 {
                for a in [0x442755, 0x44273b, 0x442756, 0x442753, 0x442757, 0x442758] {
                    self.byte(a, 0)?;
                }
                self.controls(frame.wrapping_sub(16))?;
                if self.plan.flow != LoadFlow::Returned {
                    return Ok(());
                }
            }
            for a in [0x442755, 0x44273b, 0x442756, 0x442753, 0x442757, 0x442758] {
                self.byte(a, 0)?;
            }
        }
        let x = self.sb(0x442736)?.wrapping_mul(distance);
        self.lw(
            0x442764,
            self.l(0x442764)?.wrapping_add(i32::from(x) as u32),
        )?;
        let y = self.sb(0x442737)?.wrapping_mul(distance);
        self.lw(
            0x442768,
            self.l(0x442768)?.wrapping_add(i32::from(y) as u32),
        )?;
        self.byte(0x4358c3, self.b(0x442736)?)?;
        self.byte(0x4358c4, self.b(0x442737)?)?;
        self.ww(frame.wrapping_sub(10), 1)?;
        self.center_pict(frame.wrapping_sub(18))?;
        self.lw(frame.wrapping_sub(4), self.l(0x41f716)?.wrapping_add(2))?;
        let tick = self.trap(0xa975, vec![])?;
        if tick < self.l(frame.wrapping_sub(4))? {
            self.delay(1)?;
        }
        let tick = self.trap(0xa975, vec![])?;
        self.lw(0x41f716, tick)?;
        self.trap(0xa873, vec![L(self.l(0x43878c)?)])?;
        self.full(0);
        Ok(())
    }
    fn storage_limits(&self) -> StorageLimits {
        StorageLimits {
            filename_capacity: self.limits.filename_capacity,
            maximum_iterations: self.remaining as usize,
            maximum_polls: self.limits.maximum_polls,
        }
    }
    fn prompt_child(&mut self, child: prompt::PromptControlPlan) {
        self.plan.state = child.state;
        self.plan.flow = child.flow;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(FieldControlEffect::Prompt));
        match child.native_return {
            PromptReturn::Full(d) => self.full(d),
            PromptReturn::LowWord(w) => {
                self.d0 = None;
                self.plan.native_return = PromptReturn::LowWord(w);
            }
            PromptReturn::NoReturn => {
                self.d0 = None;
                self.plan.native_return = PromptReturn::NoReturn;
            }
            PromptReturn::Unchanged => (),
        }
    }
    fn controls(&mut self, frame: u32) -> Result<(), RuleError> {
        let child =
            prompt::update_controls(&self.plan.state, frame, self.limits.prompt(), self.services)?;
        self.prompt_child(child);
        Ok(())
    }
    fn textbox(&mut self, frame: u32) -> Result<(), RuleError> {
        let child = prompt::text_box(&self.plan.state, frame, self.limits.prompt(), self.services)?;
        self.prompt_child(child);
        Ok(())
    }
    fn tutorial(&mut self, frame: u32) -> Result<(), RuleError> {
        let child =
            prompt::first_tutorial(&self.plan.state, frame, self.limits.prompt(), self.services)?;
        self.prompt_child(child);
        Ok(())
    }
    fn music(&mut self, frame: u32) -> Result<(), RuleError> {
        self.ww(frame.wrapping_add(8), 0)?;
        let child =
            prompt::submit_music(&self.plan.state, frame, self.limits.prompt(), self.services)?;
        self.prompt_child(child);
        Ok(())
    }
    fn scratch(&mut self, frame: u32, code: u16) -> Result<(), RuleError> {
        self.ww(frame.wrapping_add(8), code)?;
        let child = load::scratch(&self.plan.state.memory, frame, self.services)?;
        self.plan.state.memory = child.memory;
        self.plan.flow = child.flow;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(FieldControlEffect::Load));
        self.d0 = child.native_return;
        if self.plan.flow != LoadFlow::Returned {
            self.plan.native_return = PromptReturn::NoReturn;
        }
        Ok(())
    }
    fn warn(&mut self, frame: u32, code: u16) -> Result<(), RuleError> {
        self.ww(frame.wrapping_add(8), code)?;
        let child = feedback::warn(
            &self.plan.state.memory,
            frame,
            code as i16,
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
        position: u32,
        flags: u32,
    ) -> Result<(), RuleError> {
        self.lw(frame.wrapping_add(16), flags)?;
        self.lw(frame.wrapping_add(12), position)?;
        self.lw(frame.wrapping_add(8), pointer)?;
        let child = feedback::flashmessage(
            &self.plan.state.memory,
            frame,
            pointer,
            (position >> 16) as u16 as i16,
            position as u16 as i16,
            (flags >> 16) as u16 as i16,
            flags as u16 as i16,
            self.limits.maximum_polls,
            self.services,
        )?;
        self.feedback(child);
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
            0x13e13c => runtime::compact_heap(&mut context, &observations)?,
            0x13a498 => runtime::copy_in(&mut context, &observations)?,
            _ => return Err(RuleError::Invalid("unknown field runtime child".into())),
        };
        self.plan.state.memory.bytes = context.memory.bytes;
        self.plan.state.random = context.random;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(FieldControlEffect::Runtime));
        self.d0 = match child.result {
            RuntimeResult::Long(d) => Some(d),
            _ => None,
        };
        Ok(())
    }
    fn center_pict(&mut self, frame: u32) -> Result<(), RuleError> {
        let state = WorldDisplayState {
            memory: self.plan.state.memory.clone(),
            random: self.plan.state.random.clone(),
        };
        let child = center::centerpict(&state, frame, self.remaining, self.services)?;
        self.plan.state.memory = child.memory;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(FieldControlEffect::Center));
        if let Some(d) = child.native_return {
            self.d0 = Some(d);
        }
        Ok(())
    }
    fn center_field(&mut self, frame: u32) -> Result<(), RuleError> {
        let child = center::centerfield(
            &self.plan.state.memory,
            frame,
            self.remaining,
            self.services,
        )?;
        self.plan.state.memory = child.memory;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(FieldControlEffect::Center));
        if let Some(d) = child.native_return {
            self.d0 = Some(d);
        }
        Ok(())
    }
    fn menu(&mut self, frame: u32, entry: u32) -> Result<(), RuleError> {
        let limits = self.storage_limits();
        let child = match entry {
            0x13b212 => {
                menus::update_map_menu(&self.plan.state.memory, frame, limits, self.services)?
            }
            0x135bb2 => {
                menus::update_monster_menu(&self.plan.state.memory, frame, limits, self.services)?
            }
            0x133f3e => {
                menus::display_mission_name(&self.plan.state.memory, frame, limits, self.services)?
            }
            _ => return Err(RuleError::Invalid("unknown field menu child".into())),
        };
        self.plan.state.memory = child.memory;
        self.plan.flow = child.flow;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(FieldControlEffect::Menu));
        self.d0 = child.native_return;
        if self.plan.flow != LoadFlow::Returned {
            self.plan.native_return = PromptReturn::NoReturn;
        }
        Ok(())
    }
    fn filename(&mut self, address: u32, temporary: bool) -> Result<(), RuleError> {
        let mut context = FileContext {
            scenario_directory: c_string(
                &self.plan.state.memory,
                0x44260c,
                self.limits.filename_capacity,
            )?,
            variant: self.b(0x44274c)?,
            filename: self.bytes(0x4389a4, self.limits.filename_capacity)?,
            ..FileContext::default()
        };
        let name = c_string(
            &self.plan.state.memory,
            address,
            self.limits.filename_capacity,
        )?;
        let number = self
            .services
            .filename_number(context.variant, &self.plan.state.memory)?;
        let observations = [FileObservation {
            request: files::FileRequest::NumberToString(i32::from(context.variant)),
            reply: files::FileReply::Pascal(number),
        }];
        let child = if temporary {
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
                self.plan.effects.push(FieldControlEffect::File(effect));
            }
        }
        self.d0 = Some(0);
        Ok(())
    }
    fn load_land(&mut self, frame: u32, index: u32) -> Result<(), RuleError> {
        self.lw(frame.wrapping_add(8), index)?;
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
            .extend(child.effects.into_iter().map(FieldControlEffect::Load));
        self.d0 = child.native_return;
        if self.plan.flow != LoadFlow::Returned {
            self.plan.native_return = PromptReturn::NoReturn;
        }
        Ok(())
    }
    fn save_land(&mut self, frame: u32, index: u32) -> Result<(), RuleError> {
        self.lw(frame.wrapping_add(8), index)?;
        let child = storage::save_land(
            &self.plan.state.memory,
            frame,
            self.storage_limits(),
            self.services,
        )?;
        self.plan.state.memory = child.memory;
        self.plan.flow = child.flow;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(FieldControlEffect::Storage));
        self.d0 = child.native_return;
        if self.plan.flow != LoadFlow::Returned {
            self.plan.native_return = PromptReturn::NoReturn;
        }
        Ok(())
    }
    fn profile(&mut self, race: i16, caste: i16) -> Result<(), RuleError> {
        if race == 0 && caste == 0 {
            self.d0 = Some(0);
            return Ok(());
        }
        let observations = EquipmentServices::profile_observations(
            self.services,
            race,
            caste,
            &self.plan.state.memory,
        )?;
        let mut context = FileContext {
            scenario_directory: c_string(
                &self.plan.state.memory,
                0x44260c,
                self.limits.filename_capacity,
            )?,
            variant: self.b(0x44274c)?,
            filename: self.bytes(0x4389a4, self.limits.filename_capacity)?,
            file_handle: self.l(0x4434a6)?,
            race_profile: if race != 0 {
                self.bytes(0x41f85a, 408)?
            } else {
                vec![]
            },
            caste_profile: if caste != 0 {
                self.bytes(0x41f9f2, 576)?
            } else {
                vec![]
            },
            ..FileContext::default()
        };
        let child = files::load_profile(&mut context, race, caste, &observations)?;
        for effect in child.effects {
            match &effect {
                FileEffect::Write {
                    destination,
                    offset,
                    bytes,
                } => {
                    let address: u32 = match destination {
                        FileBuffer::Filename => 0x4389a4,
                        FileBuffer::RaceProfile => 0x41f85a,
                        FileBuffer::CasteProfile => 0x41f9f2,
                        _ => return Err(RuleError::Invalid("unexpected profile output".into())),
                    };
                    self.write(address.wrapping_add(*offset as u32), bytes)?;
                }
                FileEffect::FileHandle(handle) => self.lw(0x4434a6, *handle)?,
                _ => self.plan.effects.push(FieldControlEffect::File(effect)),
            }
        }
        self.d0 = Some(0);
        Ok(())
    }
    fn verify(&mut self, member: u16, check: bool) -> Result<(), RuleError> {
        let base = if member == u16::MAX {
            0x44721c
        } else {
            warrior(member)
        };
        let start = if !check && member != u16::MAX {
            (0x43788a + u32::from(member) * 80).min(base)
        } else {
            base
        };
        let end = base
            .checked_add(WARRIOR_BYTES as u32)
            .ok_or_else(|| RuleError::Invalid("verification view wraps".into()))?;
        let memory = InstallMemory {
            base_address: start,
            bytes: self.bytes(start, (end - start) as usize)?,
        };
        if check {
            let child = verification::checkverify(&memory, member)?;
            self.plan.effects.push(FieldControlEffect::Verification {
                member,
                mismatch: child.mismatch,
                native_d0: child.native_d0,
            });
            self.d0 = Some(child.native_d0);
        } else {
            let child = verification::setverify(&memory, member)?;
            for write in child.writes {
                self.write(write.address, &write.bytes)?;
            }
            self.d0 = Some(child.native_return);
        }
        Ok(())
    }
    fn load_item_no_type(&mut self, id: i16) -> Result<(), RuleError> {
        let backing = equipment::item_tables(&self.plan.state.memory, id)?;
        let tables = ItemTables {
            families: std::array::from_fn(|n| backing[n].as_slice()),
        };
        let current = self.bytes(0x45363c, ITEM_BYTES)?.try_into().unwrap();
        let child = records::loaditemnotype(&tables, &current, id)?;
        for write in child.writes {
            self.write(write.address, &write.bytes)?;
        }
        self.d0 = Some(child.native_return as u32);
        Ok(())
    }
    fn equipment(&mut self, child: equipment::EquipmentPlan) {
        self.plan.state.memory = child.memory;
        self.plan.state.random = child.random;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(FieldControlEffect::Equipment));
        self.d0 = child.native_return.known_long();
    }
    fn edit(&mut self, frame: u32, plus: bool) -> Result<(), RuleError> {
        let limits = InventoryEditLimits {
            maximum_steps: self.remaining,
            equipment: self.limits.equipment,
            maximum_random_draws: self.remaining_draws,
        };
        let child = if plus {
            editing::plus(
                &self.plan.state.memory,
                &self.plan.state.random,
                frame,
                limits,
                self.services,
            )?
        } else {
            editing::minus(
                &self.plan.state.memory,
                &self.plan.state.random,
                frame,
                limits,
                self.services,
            )?
        };
        self.plan.state.memory = child.memory;
        self.plan.state.random = child.random;
        self.plan.flow = child.flow;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(FieldControlEffect::Editing));
        self.d0 = child.native_return;
        if self.plan.flow != LoadFlow::Returned {
            self.plan.native_return = PromptReturn::NoReturn;
        }
        Ok(())
    }
    fn location(&mut self, argument: u16) {
        let native_return = recovered_interface::locationlog();
        self.plan.effects.push(FieldControlEffect::LocationLog {
            argument,
            native_return,
        });
        self.d0 = Some(native_return as u32);
    }
    fn delay(&mut self, duration: i16) -> Result<(), RuleError> {
        let maximum = self.limits.maximum_polls;
        let child = timing::delay(
            duration,
            0,
            &mut Clock {
                execution: self,
                remaining: maximum,
            },
        )?;
        self.d0 = Some(child.native_result);
        Ok(())
    }
}

fn warrior(member: u16) -> u32 {
    0x448aa0_u32.wrapping_add(u32::from(member) * WARRIOR_BYTES as u32)
}

/// Addressed children already composed by field control. Void working D0 is
/// intentionally unavailable to an external native caller.
pub(crate) fn existing_child_plan(
    state: &PromptControlState,
    entry: u32,
    frame: u32,
    limits: FieldControlLimits,
    services: &mut impl FieldControlServices,
) -> Result<FieldControlPlan, RuleError> {
    let mut execution = Execution {
        plan: FieldControlPlan {
            state: state.clone(),
            effects: vec![],
            native_return: PromptReturn::Unchanged,
            flow: LoadFlow::Returned,
            qualification:
                FieldControlQualification::SourcePortWithObservedBoundariesAndCapturedChildren,
        },
        services,
        limits,
        remaining: limits.maximum_steps,
        remaining_draws: limits.maximum_random_draws,
        d0: None,
    };
    match entry {
        0x136cfa | 0x136d86 => {
            let pointer = execution.l(frame.wrapping_add(8))?;
            execution.filename(pointer, entry == 0x136d86)?;
        }
        0x13a7c4 => {
            let race = execution.sw(frame.wrapping_add(8))?;
            let caste = execution.sw(frame.wrapping_add(10))?;
            execution.profile(race, caste)?;
        }
        0x14c6c2 => execution.prep_attack(frame)?,
        _ => {
            return Err(RuleError::Unsupported {
                action: "field_control.existing_child",
                dependency: "the reached entry has no existing field child adapter",
            });
        }
    }
    // These captured plans expose effects, not the callee's final CPU registers.
    execution.plan.native_return = PromptReturn::Unchanged;
    Ok(execution.plan)
}
macro_rules! entry {
    ($name:ident) => {
        pub fn $name(state: &PromptControlState, frame: u32, limits: FieldControlLimits, services: &mut impl FieldControlServices) -> Result<FieldControlPlan, RuleError> {
            let mut execution = Execution {
                plan: FieldControlPlan { state: state.clone(), effects: vec![], native_return: PromptReturn::Unchanged, flow: LoadFlow::Returned, qualification: FieldControlQualification::SourcePortWithObservedBoundariesAndCapturedChildren },
                services, limits, remaining: limits.maximum_steps, remaining_draws: limits.maximum_random_draws, d0: None,
            };
            execution.$name(frame)?;
            Ok(execution.plan)
        }
    };
}
entry!(save_character);
entry!(load_shop);
entry!(setup_new_game);
entry!(check_keypad);
entry!(encounter);
entry!(move_party);
entry!(movie);
entry!(band_aid);
entry!(do_reload);
entry!(enter_capture_mode);
struct FixedLayout(Option<Vec<u8>>);
impl LayoutServices for FixedLayout {
    fn read_layout(&self) -> Result<Option<Vec<u8>>, spatial::SpatialError> {
        Ok(self.0.clone())
    }
}
struct Clock<'a, 'b, S> {
    execution: &'a mut Execution<'b, S>,
    remaining: usize,
}
impl<S: FilePrepServices> TimingServices for Clock<'_, '_, S> {
    fn tick_count(&mut self) -> Result<u32, RuleError> {
        self.remaining = self
            .remaining
            .checked_sub(1)
            .ok_or(RuleError::Unsupported {
                action: "field_control.delay",
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
    use crate::RandomState;
    use crate::recovered_audio::{AudioContext, AudioObservation};
    use crate::recovered_character_panels::{
        CharacterPanelObservation, CharacterPanelRequest, CharacterPanelServices,
    };
    use crate::recovered_combat_center::CombatCenterServices;
    use crate::recovered_combat_ui::{CombatUiObservation, CombatUiRequest, CombatUiServices};
    use crate::recovered_equipment_ui::EquipmentUiServices;
    use crate::recovered_gamma::{
        GammaError, GammaMemory, GammaObservation, GammaRequest, GammaServices,
    };
    use crate::recovered_gamma_control::{GammaControlRequest, GammaControlServices};
    use crate::recovered_interface::{InterfaceServices, Point};
    use crate::recovered_inventory_edit::{InventoryEditObservation, InventoryEditRequest};
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

    const FRAME: u32 = base::FRAME;
    fn unused() -> RuleError {
        RuleError::Invalid("unexpected focused field-control observation".into())
    }
    fn put_word(m: &mut RenderingMemory, a: u32, value: u16) {
        m.insert_region(a, &value.to_be_bytes());
    }
    fn put_long(m: &mut RenderingMemory, a: u32, value: u32) {
        m.insert_region(a, &value.to_be_bytes());
    }
    fn value(argument: &FeedbackArg) -> u32 {
        match *argument {
            W(v) => u32::from(v),
            B(v) => u32::from(v),
            L(v) | P(v) => v,
        }
    }
    fn state() -> PromptControlState {
        let mut state = base::state();
        state.memory.insert_region(FRAME - 0x2000, &vec![0; 0x2040]);
        put_long(&mut state.memory, 0x479fd8, 0xa000);
        put_long(&mut state.memory, 0x41f716, 0);
        state.memory.insert_region(0x44273a, &[255]);
        put_word(&mut state.memory, 0x438776, 0);
        put_long(&mut state.memory, 0x442764, 0);
        put_long(&mut state.memory, 0x442768, 0);
        put_long(&mut state.memory, 0x442778, 0);
        put_long(&mut state.memory, 0x44277c, 0);
        state.memory.insert_region(0x44260c, b":Campaign\0");
        state.memory.insert_region(0x4389a4, &[0; 64]);
        state.memory.insert_region(0x4038a6, &[0]);
        state.memory.insert_region(0x403890, b"Base\0");
        for n in 0..11_u32 {
            state
                .memory
                .insert_region(0x40388c - n * 4, &[b'A' + n as u8, b':', 0]);
        }
        state.memory.insert_region(0x40385a, b"I1\0");
        state.memory.insert_region(0x403846, b"missing\0");
        state.random = RandomState::new(7);
        state
    }
    fn limits() -> FieldControlLimits {
        FieldControlLimits {
            maximum_steps: 2000,
            maximum_polls: 20,
            filename_capacity: 64,
            maximum_random_draws: 2,
            equipment: EquipmentLimits {
                maximum_steps: 500,
                maximum_polls: 20,
                filename_capacity: 64,
            },
        }
    }
    struct Observed {
        base: base::Observed,
        requests: RefCell<Vec<FeedbackRequest>>,
        registers: Vec<PromptRegisterRequest>,
        libraries: Vec<LibraryRequest>,
        dialogs: RefCell<VecDeque<u16>>,
        opens: VecDeque<u32>,
        names: VecDeque<Vec<u8>>,
        writes: Vec<(u32, Vec<u8>)>,
        ticks: Cell<u32>,
        change_preference: bool,
        movie: bool,
        bit_and: u32,
    }
    impl Default for Observed {
        fn default() -> Self {
            Self {
                base: base::Observed::new(),
                requests: RefCell::new(vec![]),
                registers: vec![],
                libraries: vec![],
                dialogs: RefCell::new(VecDeque::from([2])),
                opens: VecDeque::new(),
                names: VecDeque::new(),
                writes: vec![],
                ticks: Cell::new(0),
                change_preference: false,
                movie: false,
                bit_and: 0,
            }
        }
    }
    impl FeedbackServices for Observed {
        fn observe(
            &self,
            request: &FeedbackRequest,
            memory: &RenderingMemory,
        ) -> Result<FeedbackObservation, RuleError> {
            self.requests.borrow_mut().push(request.clone());
            let mut memory = memory.clone();
            let mut result = 0;
            if let FeedbackRequest::Toolbox { opcode, arguments } = request {
                let at = |n: usize| value(&arguments[n]);
                match opcode {
                    0xa991 => {
                        put_word(
                            &mut memory,
                            at(1),
                            self.dialogs.borrow_mut().pop_front().unwrap_or(2),
                        );
                    }
                    0xa975 => {
                        result = self.ticks.get();
                        self.ticks.set(result + 3);
                    }
                    0xa97c if self.change_preference => {
                        result = 0xd000;
                        put_word(&mut memory, 0x439578, 7);
                    }
                    0xaa46 => result = 0xe100,
                    0xaa0c => result = 0xe200,
                    0xa83e => result = 0xe300,
                    0xa9a0 if self.movie => {
                        result = if at(0) == 0x54455854 { 0xe400 } else { 0xe500 }
                    }
                    0xa9a5 if self.movie => {
                        result = 32;
                        put_long(&mut memory, 0xe400, 0xe420);
                    }
                    0xa974 | 0xa973 => result = 0,
                    0xa858 => result = self.bit_and,
                    0xa970 if self.movie => {
                        put_word(&mut memory, 0x44720a, 1);
                        put_long(&mut memory, 0x447210, 0);
                    }
                    _ => return FeedbackServices::observe(&self.base, request, &memory),
                }
            } else {
                return FeedbackServices::observe(&self.base, request, &memory);
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
            self.base.sound_observations(id, c, m)
        }
    }
    impl PromptControlServices for Observed {
        fn observe_registers(
            &mut self,
            request: &PromptRegisterRequest,
            memory: &RenderingMemory,
        ) -> Result<FeedbackObservation, RuleError> {
            self.registers.push(request.clone());
            if self.movie {
                let mut memory = memory.clone();
                let at = |n: usize| value(&request.arguments[n]);
                match (request.opcode, request.d0_input) {
                    (0xab1d, Some(0x80005)) => {
                        put_long(&mut memory, at(0), 0xe600);
                        put_long(&mut memory, at(1), 0xe700);
                    }
                    (0xab1d, Some(0x160000)) => {
                        put_long(&mut memory, at(0), 0xe800);
                    }
                    (0xab1d, Some(0x80006 | 0x40001 | 0x40002 | 0x40004)) => (),
                    (0xa029 | 0xa02a, None) => {
                        assert_eq!(request.a0_input, Some(0xe400));
                        assert!(request.arguments.is_empty());
                    }
                    _ => {
                        return PromptControlServices::observe_registers(
                            &mut self.base,
                            request,
                            &memory,
                        );
                    }
                }
                return Ok(FeedbackObservation {
                    memory,
                    value: 0,
                    d0: 0xbeef1234,
                });
            }
            PromptControlServices::observe_registers(&mut self.base, request, memory)
        }
        fn library(
            &mut self,
            request: &LibraryRequest,
            memory: &RenderingMemory,
        ) -> Result<LibraryObservation, RuleError> {
            self.libraries.push(request.clone());
            let mut memory = memory.clone();
            let a: Vec<u32> = request.arguments.iter().map(value).collect();
            let mut result = 0;
            match request.entry {
                0x400070 => result = self.opens.pop_front().unwrap_or(0),
                0x400090 => {
                    let bytes = c_string(&memory, a[0], 256)?;
                    memory.insert_region(a[1], &bytes);
                }
                0x400098 => {
                    let mut bytes = c_string(&memory, a[1], 256)?;
                    bytes.pop();
                    bytes.extend(c_string(&memory, a[0], 256)?);
                    memory.insert_region(a[1], &bytes);
                }
                0x400188 => {
                    let mut bytes = c_string(&memory, a[0], 256)?;
                    bytes.pop();
                    memory.insert_region(a[0], PascalString::from_payload(&bytes)?.raw());
                }
                0x400180 => {
                    let mut bytes =
                        equipment::read(&memory, a[0] + 1, usize::from(memory.byte(a[0])?))?;
                    bytes.push(0);
                    memory.insert_region(a[0], &bytes);
                }
                0x400048 if a[2] == 30 => {
                    let mut bytes = self.names.pop_front().unwrap_or_default();
                    bytes.resize(30, 0);
                    memory.insert_region(a[3], &bytes);
                    result = 1;
                }
                0x400048 if a[2] == 2 => {
                    put_word(&mut memory, a[3], 0x4321);
                    result = 1;
                }
                0x400048 => result = 0,
                0x400050 => {
                    self.writes
                        .push((a[3], equipment::read(&memory, a[3], a[2] as usize)?));
                    result = 1;
                }
                0x4000a8 => {
                    result =
                        u32::from(c_string(&memory, a[0], 256)? != c_string(&memory, a[1], 256)?)
                }
                0x400078 | 0x400068 | 0x400058 => (),
                _ => return PromptControlServices::library(&mut self.base, request, &memory),
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
            self.base.quiet_observations(s, c, m)
        }
        fn incoming_mechanical_mode(&mut self, m: &RenderingMemory) -> Result<i16, RuleError> {
            self.base.incoming_mechanical_mode(m)
        }
    }
    impl FilePrepServices for Observed {
        fn preference_inputs(
            &mut self,
            _: &RenderingMemory,
        ) -> Result<PreferenceInputs, RuleError> {
            Err(unused())
        }
        fn metadata_observations(
            &mut self,
            _: &[u8],
            _: &[u8],
            _: &RenderingMemory,
        ) -> Result<Vec<FileObservation>, RuleError> {
            Err(unused())
        }
    }
    impl FieldControlServices for Observed {
        fn incoming_attack_slot(&mut self, _: &RenderingMemory) -> Result<u16, RuleError> {
            Err(unused())
        }
    }
    impl RenderingServices for Observed {
        fn observe(
            &self,
            r: &RenderingRequest,
            m: &RenderingMemory,
        ) -> Result<RenderingObservation, RuleError> {
            RenderingServices::observe(&self.base, r, m)
        }
    }
    impl CombatUiServices for Observed {
        fn observe(
            &self,
            r: &CombatUiRequest,
            m: &RenderingMemory,
        ) -> Result<CombatUiObservation, RuleError> {
            CombatUiServices::observe(&self.base, r, m)
        }
    }
    impl WorldDisplayServices for Observed {
        fn observe(
            &mut self,
            r: &WorldDisplayRequest,
            m: &RenderingMemory,
        ) -> Result<WorldDisplayObservation, RuleError> {
            WorldDisplayServices::observe(&mut self.base, r, m)
        }
    }
    impl CombatCenterServices for Observed {}
    impl CharacterPanelServices for Observed {
        fn panel_observe(
            &mut self,
            r: &CharacterPanelRequest,
            m: &RenderingMemory,
        ) -> Result<CharacterPanelObservation, RuleError> {
            self.base.panel_observe(r, m)
        }
        fn runtime_observations(
            &mut self,
            e: u32,
            m: &RenderingMemory,
        ) -> Result<Vec<RuntimeObservation>, RuleError> {
            self.base.runtime_observations(e, m)
        }
    }
    impl InventoryServices for Observed {
        fn sane(
            &self,
            r: &SaneRequest,
            m: &RenderingMemory,
        ) -> Result<Vec<RecordWrite>, RuleError> {
            self.base.sane(r, m)
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
    impl LoadServices for Observed {
        fn observe_load(
            &self,
            r: &LoadRequest,
            m: &RenderingMemory,
        ) -> Result<LoadObservation, RuleError> {
            self.base.observe_load(r, m)
        }
        fn version_inputs(
            &self,
            r: i16,
            f: i16,
            m: &RenderingMemory,
        ) -> Result<VersionInputs, RuleError> {
            self.base.version_inputs(r, f, m)
        }
        fn filename_number(&self, v: u8, _: &RenderingMemory) -> Result<PascalString, RuleError> {
            PascalString::from_payload(v.to_string().as_bytes())
        }
    }
    impl StorageServices for Observed {
        fn observe_storage(
            &self,
            _: &StorageRequest,
            _: &RenderingMemory,
        ) -> Result<StorageObservation, RuleError> {
            Err(unused())
        }
    }
    impl ScenarioMenuServices for Observed {}
    impl GammaServices for Observed {
        fn observe(
            &self,
            r: &GammaRequest,
            m: &GammaMemory,
        ) -> Result<GammaObservation, GammaError> {
            GammaServices::observe(&self.base, r, m)
        }
    }
    impl GammaControlServices for Observed {
        fn observe_control(
            &self,
            r: &GammaControlRequest,
            m: &GammaMemory,
        ) -> Result<GammaObservation, GammaError> {
            self.base.observe_control(r, m)
        }
    }
    impl DrawingServices for Observed {
        fn color_icon(&mut self, id: i16) -> Result<u32, RuleError> {
            self.base.color_icon(id)
        }
        fn indexed_string(&mut self, r: i16, i: i16) -> Result<PascalString, RuleError> {
            self.base.indexed_string(r, i)
        }
        fn current_world(&mut self) -> Result<(u32, u32), RuleError> {
            self.base.current_world()
        }
        fn picture(&mut self, id: i16) -> Result<PictureObservation, RuleError> {
            self.base.picture(id)
        }
        fn new_world(
            &mut self,
            d: i16,
            r: BodyRect,
        ) -> Result<(i16, drawing::BuildingWorld), RuleError> {
            self.base.new_world(d, r)
        }
    }
    impl NumericServices for Observed {
        fn number_to_string(&mut self, n: i32) -> Result<PascalString, RuleError> {
            self.base.number_to_string(n)
        }
        fn string_to_number(&mut self, p: &PascalString) -> Result<i32, RuleError> {
            self.base.string_to_number(p)
        }
    }
    impl ItemUiServices for Observed {
        fn observe(
            &mut self,
            r: &ItemUiRequest,
            m: &RenderingMemory,
        ) -> Result<ItemUiReply, RuleError> {
            ItemUiServices::observe(&mut self.base, r, m)
        }
    }
    impl EquipmentUiServices for Observed {
        fn pen_position(
            &mut self,
            d: u32,
            m: &RenderingMemory,
        ) -> Result<Vec<RecordWrite>, RuleError> {
            self.base.pen_position(d, m)
        }
    }
    impl InterfaceServices for Observed {
        fn word(&mut self, a: u32) -> Result<i16, RuleError> {
            self.base.word(a)
        }
        fn current_port(&mut self) -> Result<u32, RuleError> {
            self.base.current_port()
        }
        fn pen_state(&mut self) -> Result<[u8; 18], RuleError> {
            self.base.pen_state()
        }
        fn local_to_global(&mut self, p: Point) -> Result<Point, RuleError> {
            self.base.local_to_global(p)
        }
        fn point_in_rect(&mut self, p: Point, r: BodyRect) -> Result<u8, RuleError> {
            self.base.point_in_rect(p, r)
        }
        fn new_region(&mut self) -> Result<u32, RuleError> {
            self.base.new_region()
        }
        fn main_device_rect(&mut self) -> Result<BodyRect, RuleError> {
            self.base.main_device_rect()
        }
    }
    #[test]
    fn fileprep_query_clamps_and_cancel_keeps_captured_slot_with_real_draw() {
        let mut s = state();
        put_word(&mut s.memory, FRAME + 8, u16::MAX);
        put_word(&mut s.memory, 0x439578, 3);
        let query = file_prep(&s, FRAME, limits(), &mut Observed::default()).unwrap();
        assert_eq!(query.native_return, PromptReturn::LowWord(4));
        assert_eq!(query.state.random, s.random);
        put_word(&mut s.memory, FRAME + 8, 0);
        put_word(&mut s.memory, 0x439578, 5);
        let before = s.clone();
        let mut o = Observed {
            change_preference: true,
            ..Observed::default()
        };
        let result = file_prep(&s, FRAME, limits(), &mut o).unwrap();
        assert_eq!(result.state.memory.word(0x439578).unwrap(), 7);
        assert_eq!(
            c_string(&result.state.memory, FRAME - 0x50, 64).unwrap(),
            b"BaseB:\0"
        );
        assert_eq!(result.state.random.draws, 1);
        assert_eq!(result.native_return, PromptReturn::Full(0xfeed0000));
        assert_eq!(result.state.memory.word(0x4434b2).unwrap(), 0);
        let mut limited = limits();
        limited.maximum_random_draws = 0;
        assert!(file_prep(&s, FRAME, limited, &mut Observed::default()).is_err());
        assert_eq!(s, before);
    }
    #[test]
    fn savecharacter_exports_verified_copy_and_restores_original_warrior() {
        let mut s = state();
        let old = s.memory.bytes.clone();
        s.memory.insert_region(
            0x43788a,
            &vec![0; (0x448aa0 + WARRIOR_BYTES as u32 - 0x43788a) as usize],
        );
        s.memory.bytes.extend(old);
        put_word(&mut s.memory, FRAME + 8, 0);
        let base = warrior(0);
        put_word(&mut s.memory, base + 0x1a, 0);
        put_word(&mut s.memory, base + 0x3a, 5);
        for (offset, n) in [
            (6, 1),
            (0x5a, 2),
            (0x9e4, 3),
            (0x9e6, 4),
            (0x124, 777),
            (0x34, 0),
        ] {
            put_word(&mut s.memory, base + offset, n);
        }
        s.memory.insert_region(base + 0xa21, b"warrior\0");
        s.memory.insert_region(0x401a66, b"Warriors:\0");
        s.memory.insert_region(0x401a71, &[0]);
        s.memory.insert_region(0x40246a, b"W:\0");
        put_word(&mut s.memory, 0x41faf0, 0);
        let warrior_before = equipment::read(&s.memory, base, WARRIOR_BYTES).unwrap();
        let mut o = Observed {
            opens: VecDeque::from([99, 99, 99]),
            names: VecDeque::from([b"warrior\0".to_vec(), vec![]]),
            ..Observed::default()
        };
        let result = save_character(&s, FRAME, limits(), &mut o).unwrap();
        let exported = &o
            .writes
            .iter()
            .find(|(_, b)| b.len() == WARRIOR_BYTES)
            .unwrap()
            .1;
        assert_eq!(
            u16::from_be_bytes(exported[0x124..0x126].try_into().unwrap()),
            10
        );
        assert_eq!(
            equipment::read(&result.state.memory, base, WARRIOR_BYTES).unwrap(),
            warrior_before
        );
        assert!(
            o.writes
                .iter()
                .any(|(a, b)| *a == FRAME - 0x18a4 + 12 && *b == vec![0, 5])
        );
        assert_eq!(result.native_return, PromptReturn::Full(0));
    }
    #[test]
    fn loadshop_uses_stale_local_after_zero_read_and_compacts_each_family() {
        let mut s = state();
        put_word(&mut s.memory, FRAME + 8, 257);
        s.memory.insert_region(0x4031b2, b":Shop\0");
        s.memory.insert_region(0x478288, &vec![0x5a; 0xbba]);
        let ids = [0x8000, 0x1234, 0x2345, 0x3456, 0x4567];
        let counts = [1, 128, 255, 2, 3];
        for n in 0..5_u32 {
            let at = 200 * n + 1;
            put_word(&mut s.memory, FRAME - 0xbba + at * 2, ids[n as usize]);
            s.memory
                .insert_region(FRAME - 0xbba + 2000 + at, &[counts[n as usize]]);
        }
        put_word(&mut s.memory, FRAME - 2, 0x3210);
        let mut o = Observed {
            opens: VecDeque::from([8]),
            ..Observed::default()
        };
        let result = load_shop(&s, FRAME, limits(), &mut o).unwrap();
        for n in 0..5_u32 {
            assert_eq!(
                result.state.memory.word(0x478288 + n * 400).unwrap(),
                ids[n as usize]
            );
            assert_eq!(
                result.state.memory.byte(0x478a58 + n * 200).unwrap(),
                counts[n as usize]
            );
            assert_eq!(
                result.state.memory.word(0x478288 + n * 400 + 2).unwrap(),
                0x5a5a
            );
        }
        assert_eq!(result.state.memory.word(0x478e40).unwrap(), 0x3210);
        assert_eq!(result.state.memory.byte(0x44273b).unwrap(), 1);
        assert!(o.libraries.contains(&LibraryRequest {
            entry: 0x400078,
            arguments: vec![W(0), L(0xbba), L(8)]
        }));
    }
    #[test]
    fn moveparty_sign_extends_low_product_word_before_long_addition() {
        let mut s = state();
        put_word(&mut s.memory, FRAME + 8, 0x8000);
        put_long(&mut s.memory, 0x442764, 100);
        s.memory.insert_region(0x442736, &[255]);
        put_word(&mut s.memory, 0x4085b8 + 99 * 180, 1);
        let result = move_party(&s, FRAME, limits(), &mut Observed::default()).unwrap();
        assert_eq!(result.state.memory.long(0x442764).unwrap(), 0xffff8064);
        assert_eq!(result.state.memory.long(0x41f716).unwrap(), 9);
        assert_eq!(result.native_return, PromptReturn::Full(0));
    }
    #[test]
    fn keypad_executes_hotkey_spell_child_and_preserves_word_selection_flags() {
        let mut s = state();
        put_word(&mut s.memory, FRAME + 8, 1);
        put_word(&mut s.memory, 0x443316, 0x1231);
        s.memory.insert_region(0x44273f, &[0]);
        s.memory.insert_region(0x43788a, &[0, 0, 0, 0, 0, 0, 0, 3]);
        s.memory.insert_region(0x437a6a, &[0; 18]);
        s.memory.insert_region(0x442580, &[0; 18]);
        let mut packet = [0_u8; SPELL_BYTES];
        packet[22] = 1;
        packet[30] = 2;
        s.memory.insert_region(0x443788, &packet);
        put_word(&mut s.memory, warrior(0) + 0x58, 10);
        put_long(&mut s.memory, 0x439470, 88);
        let mut o = Observed {
            bit_and: 256,
            ..Observed::default()
        };
        let result = check_keypad(&s, FRAME, limits(), &mut o).unwrap();
        assert_eq!(result.state.memory.word(warrior(0) + 0x58).unwrap(), 4);
        assert_eq!(result.state.memory.word(0x437a7a).unwrap(), 3);
        assert_eq!(result.state.memory.word(0x442798).unwrap(), 1);
        assert_eq!(result.state.memory.long(0x439404).unwrap(), 88);
        assert_eq!(result.native_return, PromptReturn::Full(1));
        assert_eq!(result.state.random, s.random);
    }
    #[test]
    fn movie_keeps_captured_text_pointer_and_separates_world_register_selectors() {
        let mut s = state();
        put_word(&mut s.memory, FRAME + 8, 100);
        put_word(&mut s.memory, FRAME + 12, 1);
        put_word(&mut s.memory, 0x439594, 0);
        put_long(&mut s.memory, 0xe400, 0xe410);
        put_long(&mut s.memory, 0xe802, 0xe900);
        let mut o = Observed {
            movie: true,
            ..Observed::default()
        };
        let result = movie(&s, FRAME, limits(), &mut o).unwrap();
        assert_eq!(result.state.memory.long(0xe400).unwrap(), 0xe420);
        assert!(o.requests.borrow().contains(&FeedbackRequest::Toolbox {
            opcode: 0xa83d,
            arguments: vec![P(0xe410), L(32), L(0xe500), L(0xe300), W(7)]
        }));
        assert_eq!(
            o.registers[0],
            PromptRegisterRequest {
                opcode: 0xab1d,
                d0_input: Some(0x80005),
                a0_input: None,
                arguments: vec![P(0x4046da), P(0x4046de)]
            }
        );
        assert!(
            o.registers
                .iter()
                .any(|r| r.d0_input == Some(0x40004) && r.arguments == vec![L(0xe800)])
        );
        assert_eq!(result.state.memory.word(FRAME + 10).unwrap(), 131);
        assert_eq!(result.native_return, PromptReturn::Full(0));
    }
}
