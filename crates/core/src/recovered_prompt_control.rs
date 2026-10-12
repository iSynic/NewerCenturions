//! Dialog, text, tutorial, music and selection control from CODE 1.

use crate::recovered_audio::{self as audio, AudioContext, AudioEffect, AudioObservation};
use crate::recovered_character_panels::{
    self as character, CharacterPanelEffect, CharacterPanelServices,
};
use crate::recovered_combat_center::{self as center, CombatCenterEffect, CombatCenterServices};
use crate::recovered_combat_ui::{self as combat_ui, CombatUiEffect};
use crate::recovered_drawing::{
    self as drawing, BuildingWorld, DrawingContext, DrawingEffect, DrawingServices,
    PictureObservation,
};
use crate::recovered_equipment::read;
use crate::recovered_feedback::{
    self as feedback, FeedbackArg, FeedbackEffect, FeedbackObservation, FeedbackRequest,
    FeedbackServices,
};
use crate::recovered_files::{
    self as files, FileBuffer, FileContext, FileEffect, FileObservation, FileReply, FileRequest,
};
use crate::recovered_geometry::BodyRect;
use crate::recovered_inventory_rules::{self as inventory, InventoryEffect, InventoryServices};
use crate::recovered_load_control::{self as load, LoadEffect, LoadFlow, LoadServices};
use crate::recovered_records::RecordWrite;
use crate::recovered_rendering::RenderingMemory;
use crate::recovered_runtime::{self as runtime, RuntimeEffect, RuntimeResult};
use crate::recovered_spatial::{self as spatial, SpatialServices};
use crate::recovered_text::{self as text, NumericServices, PascalString, TextContext};
use crate::recovered_timing::{self as timing, TimingServices};
use crate::recovered_world_display::WorldDisplayState;
use crate::{RandomState, RuleError};
use FeedbackArg::{Byte as B, Long as L, Pointer as P, Word as W};
use centurions_content::ITEM_BYTES;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PromptControlState {
    pub memory: RenderingMemory,
    pub random: RandomState,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PromptLimits {
    pub maximum_steps: u32,
    pub maximum_polls: usize,
    pub filename_capacity: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PromptReturn {
    Full(u32),
    LowWord(u16),
    Unchanged,
    NoReturn,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LibraryRequest {
    pub entry: u32,
    pub arguments: Vec<FeedbackArg>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LibraryObservation {
    pub memory: RenderingMemory,
    pub value: u32,
    pub d0: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PromptRegisterRequest {
    pub opcode: u16,
    pub d0_input: Option<u32>,
    pub a0_input: Option<u32>,
    pub arguments: Vec<FeedbackArg>,
}
pub trait PromptControlServices:
    CombatCenterServices + CharacterPanelServices + InventoryServices + LoadServices
{
    fn observe_registers(
        &mut self,
        _: &PromptRegisterRequest,
        _: &RenderingMemory,
    ) -> Result<FeedbackObservation, RuleError> {
        Err(RuleError::Unsupported {
            action: "prompt_control.register_toolbox",
            dependency: "actual register-input Toolbox observation",
        })
    }
    /// Actual imported-library outputs and complete resulting memory, without I/O.
    fn library(
        &mut self,
        request: &LibraryRequest,
        memory: &RenderingMemory,
    ) -> Result<LibraryObservation, RuleError>;
    fn quiet_observations(
        &mut self,
        selector: i16,
        context: &AudioContext,
        memory: &RenderingMemory,
    ) -> Result<Vec<AudioObservation>, RuleError>;
    /// Reached only when the mechanical child leaves incoming D5.W untouched.
    fn incoming_mechanical_mode(&mut self, memory: &RenderingMemory) -> Result<i16, RuleError>;
}
#[derive(Clone, Debug, PartialEq)]
pub enum PromptControlEffect {
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
    Audio(AudioEffect),
    Runtime(RuntimeEffect),
    File(FileEffect),
    Load(LoadEffect),
    Character(CharacterPanelEffect),
    Center(CombatCenterEffect),
    Combat(CombatUiEffect),
    Inventory(InventoryEffect),
    Random {
        bound: i16,
        value: i16,
        before: RandomState,
        after: RandomState,
    },
    Mechanical {
        active: i16,
        origin: (i32, i32),
        incoming_mode: i16,
        result: bool,
    },
}
#[derive(Clone, Debug, PartialEq)]
pub struct PromptControlPlan {
    pub state: PromptControlState,
    pub effects: Vec<PromptControlEffect>,
    pub native_return: PromptReturn,
    pub flow: LoadFlow,
}
struct Execution<'a, S> {
    plan: PromptControlPlan,
    s: &'a mut S,
    limits: PromptLimits,
    remaining: u32,
    d0: Option<u32>,
}
impl<S: PromptControlServices> Execution<'_, S> {
    fn step(&mut self) -> Result<(), RuleError> {
        self.remaining = self.remaining.checked_sub(1).ok_or_else(|| {
            RuleError::Invalid("prompt control processing budget exhausted".into())
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
            .push(PromptControlEffect::Write(RecordWrite {
                address: a,
                bytes: bytes.to_vec(),
            }));
        Ok(())
    }
    fn byte(&mut self, a: u32, v: u8) -> Result<(), RuleError> {
        self.write(a, &[v])
    }
    fn word(&mut self, a: u32, v: u16) -> Result<(), RuleError> {
        self.write(a, &v.to_be_bytes())
    }
    fn long(&mut self, a: u32, v: u32) -> Result<(), RuleError> {
        self.write(a, &v.to_be_bytes())
    }
    fn add(&mut self, a: u32, v: i16) -> Result<(), RuleError> {
        self.word(a, self.w(a)?.wrapping_add(v as u16))
    }
    fn low(&mut self, v: u16) {
        self.d0 = self.d0.map(|d| (d & 0xffff0000) | u32::from(v));
        self.plan.native_return = self.d0.map_or(PromptReturn::LowWord(v), PromptReturn::Full);
    }
    fn full(&mut self, v: u32) {
        self.d0 = Some(v);
        self.plan.native_return = PromptReturn::Full(v);
    }
    fn returned(&mut self, value: PromptReturn) {
        match value {
            PromptReturn::Full(d) => self.full(d),
            PromptReturn::LowWord(w) => self.low(w),
            PromptReturn::Unchanged => {}
            PromptReturn::NoReturn => {
                self.plan.native_return = value;
                self.d0 = None;
            }
        }
    }
    fn observe(&mut self, request: FeedbackRequest) -> Result<u32, RuleError> {
        self.step()?;
        let observation = FeedbackServices::observe(self.s, &request, &self.plan.state.memory)?;
        let value = observation.value;
        self.d0 = Some(observation.d0);
        self.plan.state.memory = observation.memory.clone();
        self.plan
            .effects
            .push(PromptControlEffect::Feedback(FeedbackEffect::Service {
                request,
                observation,
            }));
        Ok(value)
    }
    fn trap(&mut self, opcode: u16, arguments: Vec<FeedbackArg>) -> Result<u32, RuleError> {
        self.observe(FeedbackRequest::Toolbox { opcode, arguments })
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
            .s
            .observe_registers(&request, &self.plan.state.memory)?;
        let value = observation.value;
        self.d0 = Some(observation.d0);
        self.plan.state.memory = observation.memory.clone();
        self.plan
            .effects
            .push(PromptControlEffect::RegisterToolbox {
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
        let observation = self.s.library(&request, &self.plan.state.memory)?;
        let value = observation.value;
        self.d0 = Some(observation.d0);
        self.plan.state.memory = observation.memory.clone();
        self.plan.effects.push(PromptControlEffect::Library {
            request,
            observation,
        });
        Ok(value)
    }
    fn port(&mut self, pointer: u32) -> Result<(), RuleError> {
        self.trap(0xa873, vec![L(pointer)])?;
        Ok(())
    }
    fn feedback(&mut self, child: feedback::FeedbackPlan) {
        self.plan.state.memory = child.memory;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(PromptControlEffect::Feedback));
        if let Some(d) = child.native_return {
            self.full(d);
        } else {
            self.d0 = None;
        }
    }
    fn sound(&mut self, id: i16) -> Result<(), RuleError> {
        let child = feedback::sound(&self.plan.state.memory, id, self.s)?;
        self.feedback(child);
        Ok(())
    }
    fn warn(&mut self, frame: u32, id: i16) -> Result<(), RuleError> {
        self.word(frame.wrapping_add(8), id as u16)?;
        let child = feedback::warn(
            &self.plan.state.memory,
            frame,
            id,
            self.limits.maximum_polls,
            self.s,
        )?;
        self.feedback(child);
        Ok(())
    }
    fn delay(&mut self, duration: i16) -> Result<(), RuleError> {
        let limit = self.limits.maximum_polls;
        let child = timing::delay(
            duration,
            0,
            &mut Clock {
                execution: self,
                remaining: limit,
            },
        )?;
        self.full(child.native_result);
        Ok(())
    }
    fn quiet(&mut self, selector: i16) -> Result<(), RuleError> {
        let last = self.sw(0x439576)?;
        let mut context = AudioContext {
            last_channel: last,
            ..AudioContext::default()
        };
        if last >= 0 {
            let reached: Vec<i16> = if selector > 0 {
                vec![selector]
            } else {
                (1..last).collect()
            };
            if let Some(maximum) = reached.iter().copied().max() {
                context.channels.resize(maximum as usize + 1, 0);
            }
            for n in reached {
                context.channels[n as usize] = self.l(index(0x4436b6, n, 4))?;
            }
        }
        let observations =
            self.s
                .quiet_observations(selector, &context, &self.plan.state.memory)?;
        let child = audio::quiet(&mut context, selector, &observations)?;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(PromptControlEffect::Audio));
        self.full(0);
        Ok(())
    }
    fn runtime(&mut self, entry: u32) -> Result<(), RuleError> {
        let observations =
            CharacterPanelServices::runtime_observations(self.s, entry, &self.plan.state.memory)?;
        let mut context =
            runtime::RuntimeContext::new(self.plan.state.random.clone(), self.remaining as usize);
        context.memory.bytes = self.plan.state.memory.bytes.clone();
        let child = match entry {
            0x13a498 => runtime::copy_in(&mut context, &observations)?,
            0x13e13c => runtime::compact_heap(&mut context, &observations)?,
            _ => return Err(RuleError::Invalid("unknown prompt runtime child".into())),
        };
        self.plan.state.memory.bytes = context.memory.bytes;
        self.plan.state.random = context.random;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(PromptControlEffect::Runtime));
        match child.result {
            RuntimeResult::Long(v) => self.full(v),
            _ => self.d0 = None,
        }
        Ok(())
    }
    fn press(
        &mut self,
        frame: u32,
        kind: u16,
        phase: u16,
        sound: u16,
        rect: u32,
    ) -> Result<(), RuleError> {
        self.long(frame.wrapping_add(18), self.l(rect.wrapping_add(4))?)?;
        self.long(frame.wrapping_add(14), self.l(rect)?)?;
        self.word(frame.wrapping_add(12), sound)?;
        self.long(
            frame.wrapping_add(8),
            (u32::from(kind) << 16) | u32::from(phase),
        )?;
        let child = feedback::pressbutton(
            &self.plan.state.memory,
            frame,
            kind as i16,
            phase as i16,
            sound as i16,
            self.s,
        )?;
        self.feedback(child);
        Ok(())
    }
    fn flash(
        &mut self,
        frame: u32,
        pointer: u32,
        h: u16,
        v: u16,
        wait: u16,
        sound: u16,
    ) -> Result<(), RuleError> {
        self.long(
            frame.wrapping_add(16),
            (u32::from(wait) << 16) | u32::from(sound),
        )?;
        self.long(frame.wrapping_add(12), (u32::from(h) << 16) | u32::from(v))?;
        self.long(frame.wrapping_add(8), pointer)?;
        let child = feedback::flashmessage(
            &self.plan.state.memory,
            frame,
            pointer,
            h as i16,
            v as i16,
            wait as i16,
            sound as i16,
            self.limits.maximum_polls,
            self.s,
        )?;
        self.feedback(child);
        Ok(())
    }
    fn pict(&mut self, frame: u32, id: u16, rect: u32) -> Result<(), RuleError> {
        self.long(frame.wrapping_add(14), self.l(rect.wrapping_add(4))?)?;
        self.long(frame.wrapping_add(10), self.l(rect)?)?;
        self.word(frame.wrapping_add(8), id)?;
        let handle = self.trap(0xa9bc, vec![W(id)])?;
        self.long(frame.wrapping_sub(4), handle)?;
        let bounds = self.plan.state.memory.rect(frame.wrapping_add(10))?;
        let child = drawing::pict(id as i16, bounds, &mut FixedDrawing { handle })?;
        if !matches!(
            child.as_slice(),
            [
                DrawingEffect::GetPicture(_),
                DrawingEffect::DrawPicture { .. }
            ]
        ) {
            return Err(RuleError::Invalid("unexpected pict child plan".into()));
        }
        self.trap(
            0xa8f6,
            vec![L(self.l(frame.wrapping_sub(4))?), P(frame.wrapping_add(10))],
        )?;
        Ok(())
    }
    fn pict_handle(&mut self, frame: u32, id: u16, global: u32) -> Result<(), RuleError> {
        let handle = self.l(global)?;
        let data = self.l(handle)?;
        self.long(frame.wrapping_add(14), self.l(data.wrapping_add(12))?)?;
        self.long(frame.wrapping_add(10), self.l(data.wrapping_add(8))?)?;
        self.word(frame.wrapping_add(8), id)?;
        let icon = self.trap(0xa9bc, vec![W(id)])?;
        self.long(frame.wrapping_sub(4), icon)?;
        let bounds = self.plan.state.memory.rect(frame.wrapping_add(10))?;
        let _child = drawing::pict(id as i16, bounds, &mut FixedDrawing { handle: icon })?;
        self.trap(
            0xa8f6,
            vec![L(self.l(frame.wrapping_sub(4))?), P(frame.wrapping_add(10))],
        )?;
        Ok(())
    }
    fn icon2(&mut self, id: i16) -> Result<(), RuleError> {
        let rect = self.plan.state.memory.rect(0x4471f2)?;
        let mut context = icon_context(rect);
        let child = drawing::ploticon2(&mut context, id, &mut FixedDrawing { handle: 1 })?;
        let mut obtained = None;
        for effect in child.effects {
            match effect {
                DrawingEffect::Write(w) if w.address == 0x439598 && obtained.is_some() => {
                    self.long(w.address, obtained.expect("obtained icon"))?
                }
                DrawingEffect::Write(w) => self.write(w.address, &w.bytes)?,
                DrawingEffect::GetColorIcon(id) => {
                    obtained = Some(self.trap(0xaa1e, vec![W(id as u16)])?)
                }
                DrawingEffect::PlotColorIcon { .. } if obtained != Some(0) => {
                    self.trap(0xaa1f, vec![P(0x4471f2), L(self.l(0x439598)?)])?;
                }
                DrawingEffect::DisposeColorIcon(_) if obtained != Some(0) => {
                    self.trap(0xaa25, vec![L(self.l(0x439598)?)])?;
                }
                DrawingEffect::PlotColorIcon { .. } | DrawingEffect::DisposeColorIcon(_) => {}
                _ => {
                    return Err(RuleError::Invalid(
                        "unexpected ploticon2 child effect".into(),
                    ));
                }
            }
        }
        self.full(0);
        Ok(())
    }
    fn numeric(&mut self, value: i16) -> Result<(), RuleError> {
        self.lib(0x400168, vec![L(value as i32 as u32), P(0x479ed8)])?;
        let scratch = PascalString::from_raw(&read(
            &self.plan.state.memory,
            0x479ed8,
            usize::from(self.b(0x479ed8)?) + 1,
        )?)?;
        let _child = text::string(
            &mut TextContext::default(),
            value,
            &mut FixedNumber(scratch),
        )?;
        self.trap(0xa884, vec![P(0x479ed8)])?;
        Ok(())
    }
    fn filename(&mut self, address: u32) -> Result<(), RuleError> {
        let mut context = FileContext {
            scenario_directory: c_string(&self.plan.state.memory, 0x44260c)?,
            variant: self.b(0x44274c)?,
            filename: read(
                &self.plan.state.memory,
                0x4389a4,
                self.limits.filename_capacity,
            )?,
            ..FileContext::default()
        };
        let number = self
            .s
            .filename_number(context.variant, &self.plan.state.memory)?;
        let observations = [FileObservation {
            request: FileRequest::NumberToString(i32::from(context.variant)),
            reply: FileReply::Pascal(number),
        }];
        let name = c_string(&self.plan.state.memory, address)?;
        let child = files::get_filename(&mut context, &name, &observations)?;
        for effect in child.effects {
            match effect {
                FileEffect::Write {
                    destination: FileBuffer::Filename,
                    offset,
                    bytes,
                } => self.write(0x4389a4_u32.wrapping_add(offset as u32), &bytes)?,
                e => self.plan.effects.push(PromptControlEffect::File(e)),
            }
        }
        self.d0 = None;
        Ok(())
    }
    fn character_box(&mut self, frame: u32, member: u16, args: u32) -> Result<(), RuleError> {
        self.long(frame.wrapping_add(10), args)?;
        self.word(frame.wrapping_add(8), member)?;
        let child =
            character::update_picture_box(&self.plan.state.memory, frame, self.remaining, self.s)?;
        self.plan.state.memory = child.memory;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(PromptControlEffect::Character),
        );
        self.full(child.native_return);
        Ok(())
    }
    fn merge_center(&mut self, child: center::CombatCenterPlan) {
        self.plan.state.memory = child.memory;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(PromptControlEffect::Center));
        if let Some(d) = child.native_return {
            self.full(d);
        }
    }
    fn cast(&mut self, frame: u32, member: u16) -> Result<bool, RuleError> {
        self.word(frame.wrapping_add(10), 1)?;
        self.word(frame.wrapping_add(8), member)?;
        let child = inventory::cancast(
            &self.plan.state.memory,
            frame,
            self.d0.unwrap_or(0),
            self.limits.maximum_polls,
            self.s,
        )?;
        let allowed = child.native_return as u16 != 0;
        self.plan.state.memory = child.memory;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(PromptControlEffect::Inventory),
        );
        // Only the low WORD is consumed by the parent; an unknown upper WORD stays unknown.
        self.low(child.native_return as u16);
        Ok(allowed)
    }
    fn control_order(&self) -> Result<i16, RuleError> {
        self.sw(index(0x4470b8, self.sw(0x447194)?, 2))
    }
    fn position(&mut self, horizontal: u16, vertical: u16) -> Result<(), RuleError> {
        self.trap(0xa893, vec![W(horizontal), W(vertical)])?;
        Ok(())
    }
    fn controls(&mut self, frame: u32) -> Result<(), RuleError> {
        self.step()?;
        self.port(self.l(0x43878c)?)?;
        self.trap(0xa862, vec![L(33)])?;
        self.trap(0xa8f6, vec![L(self.l(0x439544)?), P(0x447202)])?;
        self.word(0x447204, self.w(0x40856c)?.wrapping_add(330))?;
        self.word(0x447208, self.w(0x447204)?.wrapping_add(50))?;
        self.word(frame.wrapping_sub(6), 0)?;
        self.word(
            frame.wrapping_sub(8),
            self.w(0x40856e)?
                .wrapping_add(320)
                .wrapping_sub(self.w(0x439582)?.wrapping_shl(5)),
        )?;
        self.word(frame.wrapping_sub(2), self.w(0x40856c)?.wrapping_add(308))?;
        self.word(
            frame.wrapping_sub(4),
            self.w(0x40856e)?
                .wrapping_add(460)
                .wrapping_sub(self.w(0x439582)?.wrapping_shl(5)),
        )?;
        self.press(frame.wrapping_sub(30), 1, 1, 0, 0x447202)?;
        if self.w(0x438776)? != 0 && self.control_order()? > 5 {
            self.full(0);
            return Ok(());
        }
        if self.w(0x438776)? == 0 {
            self.word(0x447202, self.sb(0x44273f)?.wrapping_mul(50) as u16)?;
        } else if self.control_order()? < 9 {
            self.word(0x447202, self.control_order()?.wrapping_mul(50) as u16)?;
        }
        self.word(0x447206, self.w(0x447202)?.wrapping_add(50))?;
        self.word(0x447204, self.w(0x40856c)?.wrapping_add(330))?;
        self.word(0x447208, self.w(0x447204)?.wrapping_add(50))?;
        self.press(frame.wrapping_sub(30), 1, 2, 0, 0x447202)?;
        self.word(0x447204, self.w(0x40856c)?.wrapping_add(321))?;
        self.word(0x447208, self.w(0x40856c)?.wrapping_add(330))?;
        self.trap(0xa8f6, vec![L(self.l(0x43955c)?), P(0x447202)])?;
        self.word(0x4435d0, self.sb(0x44273f)? as u16)?;
        let member = if self.w(0x438776)? != 0 && self.control_order()? < 9 {
            self.w(0x4435d2)?
        } else {
            self.sb(0x44273f)? as u16
        };
        let allowed = self.cast(frame.wrapping_sub(20), member)?;
        let available = allowed && self.w(0x438760)? == 0;
        self.pict_handle(
            frame.wrapping_sub(26),
            if available { 162 } else { 154 },
            0x439470,
        )?;
        if self.w(0x438776)? == 0 {
            self.pict(
                frame.wrapping_sub(26),
                223_u16.wrapping_sub(self.w(0x439582)?),
                frame.wrapping_sub(8),
            )?;
            self.trap(0xa862, vec![L(205)])?;
            self.trap(0xa88a, vec![W(14)])?;
            self.position(
                half(self.sw(0x40856c)?).wrapping_add(135) as u16,
                self.w(0x40856e)?
                    .wrapping_add(400)
                    .wrapping_sub(self.w(0x439582)?.wrapping_shl(5)),
            )?;
            self.lib(
                0x400168,
                vec![
                    L(self.l(record(0x448aa0, self.sb(0x44273f)?, 3138).wrapping_add(0x9d2))?),
                    P(0x479ed8),
                ],
            )?;
            self.trap(0xa884, vec![P(0x479ed8)])?;
            self.trap(0xa862, vec![L(33)])?;
            if self.b(0x442758)? != 0 {
                for global in [0x439450, 0x439430, 0x43947c] {
                    self.trap(0xa958, vec![L(self.l(global)?)])?;
                }
                self.trap(0xa957, vec![L(self.l(0x439474)?)])?;
                self.pict_handle(frame.wrapping_sub(26), 200, 0x439474)?;
            } else {
                self.trap(0xa958, vec![L(self.l(0x439474)?)])?;
                self.pict_handle(frame.wrapping_sub(26), 154, 0x439474)?;
                if self.b(0x442753)? != 0 {
                    for global in [0x439474, 0x439450, 0x43947c] {
                        self.trap(0xa958, vec![L(self.l(global)?)])?;
                    }
                    self.trap(0xa957, vec![L(self.l(0x439430)?)])?;
                    self.pict_handle(frame.wrapping_sub(26), 225, 0x439430)?;
                } else {
                    self.trap(0xa958, vec![L(self.l(0x439430)?)])?;
                    self.pict_handle(frame.wrapping_sub(26), 154, 0x439430)?;
                }
                if self.b(0x442753)? == 0 && self.b(0x442758)? == 0 {
                    for global in [0x439474, 0x439450, 0x439430] {
                        self.trap(0xa958, vec![L(self.l(global)?)])?;
                    }
                    self.trap(0xa957, vec![L(self.l(0x43947c)?)])?;
                    self.pict_handle(frame.wrapping_sub(26), 208, 0x43947c)?;
                } else {
                    self.trap(0xa958, vec![L(self.l(0x43947c)?)])?;
                }
            }
            if self.b(0x44273a)? == 0 {
                self.trap(0xa958, vec![L(self.l(0x439444)?)])?;
                self.pict_handle(frame.wrapping_sub(26), 154, 0x439444)?;
            }
        } else {
            let qualifies = self.control_order()? < 9
                && self.sw(record(0x448aa0, self.control_order()?, 3138).wrapping_add(244))? > 0
                && self.sw(record(0x448aa0, self.control_order()?, 3138).wrapping_add(64))? > 1;
            let mechanical = if qualifies {
                let active = self.sw(0x4435d2)?;
                let origin = (self.l(0x442780)? as i32, self.l(0x442784)? as i32);
                let base = record(0x448aa0, active, 3138);
                let mode = if self.w(base.wrapping_add(246))? != 0
                    || self.w(base.wrapping_add(244))? != 0
                {
                    0
                } else {
                    self.s.incoming_mechanical_mode(&self.plan.state.memory)?
                };
                let result = spatial::checkformechanicalunits(
                    active,
                    origin,
                    mode,
                    &MemorySpatial(&self.plan.state.memory),
                )
                .map_err(|error| RuleError::Invalid(error.to_string()))?;
                self.plan.effects.push(PromptControlEffect::Mechanical {
                    active,
                    origin,
                    incoming_mode: mode,
                    result,
                });
                result
            } else {
                false
            };
            self.trap(
                if mechanical { 0xa957 } else { 0xa958 },
                vec![L(self.l(0x439450)?)],
            )?;
            self.pict_handle(
                frame.wrapping_sub(26),
                if mechanical { 161 } else { 154 },
                0x439450,
            )?;
            if mechanical {
                self.trap(0xa88a, vec![W(10)])?;
                self.trap(0xa862, vec![L(69)])?;
                self.position(
                    self.w(0x40856c)?.wrapping_add(388),
                    self.w(0x40856e)?
                        .wrapping_add(445)
                        .wrapping_sub(self.w(0x439582)?.wrapping_shl(5)),
                )?;
                let active = self.sw(0x4435d2)?;
                self.numeric(
                    self.sw(record(0x448aa0, active, 3138).wrapping_add(244))?
                        .wrapping_add(self.sw(record(0x448aa0, active, 3138).wrapping_add(0xb62))?),
                )?;
                self.position(
                    self.w(0x40856c)?.wrapping_add(388),
                    self.w(0x40856e)?
                        .wrapping_add(454)
                        .wrapping_sub(self.w(0x439582)?.wrapping_shl(5)),
                )?;
                if self.sw(record(0x448aa0, self.sw(0x4435d2)?, 3138).wrapping_add(246))? > 0 {
                    let active = self.sw(0x4435d2)?;
                    self.numeric(
                        self.sw(record(0x448aa0, active, 3138).wrapping_add(246))?
                            .wrapping_add(
                                self.sw(record(0x448aa0, active, 3138).wrapping_add(0xb64))?,
                            ),
                    )?;
                } else {
                    self.trap(0xa884, vec![P(0x400a6c)])?;
                }
                self.long(frame.wrapping_sub(12), 0x006b0070)?;
                self.first(frame.wrapping_sub(20))?;
                if self.plan.flow != LoadFlow::Returned {
                    return Ok(());
                }
            }
        }
        self.full(0);
        Ok(())
    }
    fn reset(&mut self, frame: u32) -> Result<(), RuleError> {
        self.step()?;
        self.port(self.l(0x43878c)?)?;
        let mode = self.w(frame.wrapping_add(8))?;
        self.low(mode);
        if mode == 0 {
            self.controls(frame.wrapping_sub(8))?;
        } else {
            self.low(mode.wrapping_sub(1));
            if mode == 1 {
                self.trap(0xa8a3, vec![P(0x4471cc)])?;
                if let Some(d0) = self.d0 {
                    self.full(d0);
                }
            }
        }
        Ok(())
    }
    fn question(&mut self, frame: u32) -> Result<(), RuleError> {
        self.step()?;
        let mut resource = self.sw(frame.wrapping_add(8))?;
        let copy_back = resource >= 0;
        self.trap(0xa874, vec![P(frame.wrapping_sub(16))])?;
        self.trap(0xaa1c, vec![L(self.l(0x439530)?)])?;
        self.word(0x4434aa, 0)?;
        while self.sw(0x4434aa)? <= self.sw(0x439576)? {
            self.step()?;
            self.quiet(self.sw(0x4434aa)?)?;
            self.add(0x4434aa, 1)?;
        }
        self.sound(20005)?;
        if copy_back && self.trap(0xa924, vec![])? == self.l(0x43879c)? {
            self.runtime(0x13a498)?;
        }
        self.lib(0x400038, vec![W(resource as u16)])?;
        resource = self.d0.expect("observed absolute result") as u16 as i16;
        let dialog = self.trap(0xa97c, vec![W(137), L(0), L(u32::MAX)])?;
        self.port(dialog)?;
        self.trap(0xaa0b, vec![L(self.l(0x4358ce)?)])?;
        self.trap(0xa887, vec![W(self.w(0x402f44)?)])?;
        self.trap(0xa862, vec![L(69)])?;
        self.observe(FeedbackRequest::GetIndexedString {
            destination: 0x479ed8,
            resource: 135,
            index: resource,
        })?;
        self.trap(
            0xa98b,
            vec![P(0x479ed8), P(0x401998), P(0x401998), P(0x401998)],
        )?;
        self.w(0x439582)?;
        self.trap(
            0xa91b,
            vec![L(dialog), W(self.w(0x4395a4)?), W(self.w(0x4395a2)?), B(0)],
        )?;
        self.trap(0xa915, vec![L(dialog)])?;
        self.flush()?;
        loop {
            self.step()?;
            self.trap(0xa860, vec![W(65535), P(0x44720a), L(0), L(0)])?;
            if self.trap(0xa97f, vec![P(0x44720a)])? as u8 != 0
                && self.trap(
                    0xa980,
                    vec![P(0x44720a), P(frame.wrapping_sub(4)), P(0x4434b2)],
                )? as u8
                    != 0
            {
                break;
            }
            if self.w(0x44720a)? == 3 {
                match self.l(0x44720c)? {
                    0x21079 => {
                        self.word(0x4434b2, 2)?;
                        break;
                    }
                    0x22d6e => {
                        self.word(0x4434b2, 1)?;
                        break;
                    }
                    _ => {}
                }
            }
        }
        self.trap(0xaa1c, vec![L(self.l(0x43952c)?)])?;
        self.trap(
            0xa98d,
            vec![
                L(dialog),
                W(self.w(0x4434b2)?),
                P(0x4434b0),
                P(0x4434dc),
                P(frame.wrapping_sub(12)),
            ],
        )?;
        self.press(frame.wrapping_sub(50), 2, 2, 0, frame.wrapping_sub(12))?;
        self.delay(2)?;
        self.press(frame.wrapping_sub(66), 2, 1, 0, frame.wrapping_sub(12))?;
        self.trap(0xa983, vec![L(dialog)])?;
        self.port(self.l(frame.wrapping_sub(16))?)?;
        if copy_back {
            let child = character::copy_out(
                &self.plan.state.memory,
                frame.wrapping_sub(36),
                self.remaining,
                self.s,
            )?;
            self.plan.state.memory = child.memory;
            self.plan.effects.extend(
                child
                    .effects
                    .into_iter()
                    .map(PromptControlEffect::Character),
            );
            self.full(child.native_return);
        }
        if self.w(frame.wrapping_add(10))? == 0 {
            let state = WorldDisplayState {
                memory: self.plan.state.memory.clone(),
                random: self.plan.state.random.clone(),
            };
            let child = center::centerpict(&state, frame.wrapping_sub(36), self.remaining, self.s)?;
            self.merge_center(child);
            self.long(frame.wrapping_sub(32), 0x50005)?;
            let child = center::centerfield(
                &self.plan.state.memory,
                frame.wrapping_sub(40),
                self.remaining,
                self.s,
            )?;
            self.merge_center(child);
        }
        self.low(self.w(0x4434b2)?.wrapping_sub(1));
        Ok(())
    }
    fn rand(&mut self, bound: i16) -> Result<i16, RuleError> {
        let mut context =
            runtime::RuntimeContext::new(self.plan.state.random.clone(), self.remaining as usize);
        let child = runtime::rand(&mut context, bound)?;
        self.plan.state.random = context.random;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(PromptControlEffect::Runtime));
        let RuntimeResult::Word { value, native_d0 } = child.result else {
            return Err(RuleError::Invalid("unexpected native Rand result".into()));
        };
        self.full(native_d0);
        Ok(value)
    }
    fn music(&mut self, frame: u32) -> Result<(), RuleError> {
        self.step()?;
        let override_id = self.w(frame.wrapping_add(8))?;
        if self.w(0x439594)? != 0 {
            for command in [4, 3] {
                self.word(frame.wrapping_sub(8), command)?;
                self.word(frame.wrapping_sub(6), 0)?;
                self.long(frame.wrapping_sub(4), 0)?;
                let status =
                    self.trap(0xa804, vec![L(self.l(0x4436b6)?), P(frame.wrapping_sub(8))])?;
                self.word(0x4386ac, status as u16)?;
            }
            self.full(0);
            return Ok(());
        }
        let status = self.registers(
            0xa800,
            Some(0x05100008),
            None,
            vec![L(self.l(0x4436b6)?), W(24), P(frame.wrapping_sub(32))],
        )?;
        self.word(0x4386ac, status as u16)?;
        if self.w(0x4386ac)? != 0 {
            self.long(frame.wrapping_sub(314), 0x70001)?;
            self.question(frame.wrapping_sub(322))?;
            if self.w_return()? == 0 {
                self.word(0x439594, 1)?;
                self.flash(frame.wrapping_sub(330), 0x402912, 30, 300, 180, 6000)?;
                self.full(0);
                return Ok(());
            }
        }
        if self.b(frame.wrapping_sub(20))? == 1 {
            self.full(0);
            return Ok(());
        }
        let mut attempts = 0_u16;
        let mut selected;
        loop {
            self.step()?;
            let prior = attempts;
            attempts = attempts.wrapping_add(1);
            if prior as i16 > 1000 {
                self.word(0x439594, 1)?;
                self.trap(0xa947, vec![L(self.l(0x4386a0)?), W(1), P(0x402906)])?;
                self.trap(0xa947, vec![L(self.l(0x4386a0)?), W(4), P(0x4028ce)])?;
                self.warn(frame.wrapping_sub(320), 155)?;
                self.full(0);
                return Ok(());
            }
            selected = self.rand(13)?.wrapping_add(20127) as u16;
            self.low(selected);
            if self.b(index(0x44270e, selected.wrapping_sub(20128) as i16, 1))? != 0 {
                break;
            }
        }
        if override_id != 0 {
            selected = override_id;
        }
        if self.l(0x439570)? == 0 || self.sw(0x439576)? < 0 {
            self.full(0);
            return Ok(());
        }
        self.word(0x402f3a, 0)?;
        self.registers(0xa02a, None, Some(self.l(0x4394dc)?), vec![])?;
        if self.l(0x4394dc)? != 0 {
            self.trap(0xa9a3, vec![L(self.l(0x4394dc)?)])?;
            self.runtime(0x13e13c)?;
        }
        let handle = self.trap(0xa9a0, vec![L(0x736e6420), W(selected)])?;
        self.long(0x4394dc, handle)?;
        self.registers(0xa029, None, Some(self.l(0x4394dc)?), vec![])?;
        if self.l(0x4394dc)? != 0 {
            self.trap(
                0xa9a8,
                vec![
                    L(self.l(0x4394dc)?),
                    P(frame.wrapping_sub(290)),
                    P(frame.wrapping_sub(294)),
                    P(frame.wrapping_sub(288)),
                ],
            )?;
            self.trap(
                0xa947,
                vec![L(self.l(0x4386a0)?), W(4), P(frame.wrapping_sub(288))],
            )?;
            self.trap(
                0xa805,
                vec![L(self.l(0x4436b6)?), L(self.l(0x4394dc)?), B(1)],
            )?;
        }
        self.full(0);
        Ok(())
    }
    fn w_return(&self) -> Result<u16, RuleError> {
        match self.plan.native_return {
            PromptReturn::Full(d) => Ok(d as u16),
            PromptReturn::LowWord(w) => Ok(w),
            _ => Err(RuleError::Invalid(
                "prompt child has no returned WORD".into(),
            )),
        }
    }
    fn textbox(&mut self, frame: u32) -> Result<(), RuleError> {
        self.step()?;
        let mut text_position = -1_i16;
        self.long(frame.wrapping_sub(14), self.l(0x44337a)?)?;
        self.word(frame.wrapping_sub(10), self.w(0x44337e)?)?;
        self.word(frame.wrapping_sub(16), 72)?;
        if self.w(frame.wrapping_add(10))? == 0 {
            self.full(0);
            return Ok(());
        }
        if self.sw(frame.wrapping_add(8))? == -1 {
            self.word(
                frame.wrapping_add(12),
                u16::from(self.sw(frame.wrapping_add(10))? >= 0),
            )?;
        }
        self.trap(0xa874, vec![P(frame.wrapping_sub(24))])?;
        if self.w(frame.wrapping_add(14))? != 0 {
            self.long(frame.wrapping_sub(8), self.l(frame.wrapping_add(16))?)?;
            self.long(frame.wrapping_sub(4), self.l(frame.wrapping_add(20))?)?;
        } else {
            self.long(frame.wrapping_sub(8), self.l(0x4471cc)?)?;
            self.long(frame.wrapping_sub(4), self.l(0x4471d0)?)?;
            self.port(self.l(0x43878c)?)?;
        }
        self.trap(0xa862, vec![L(69)])?;
        self.trap(0xaa1c, vec![L(self.l(0x43952c)?)])?;
        match self.sw(frame.wrapping_add(8))? {
            -2 | -1 => {
                let tutorial = self.sw(frame.wrapping_add(8))? == -2;
                self.observe(FeedbackRequest::GetIndexedString {
                    destination: 0x479ed8,
                    resource: 1,
                    index: 3,
                })?;
                let (name, mode) = if tutorial {
                    (0x400a9c, 0x400a98)
                } else {
                    self.filename(0x400a8e)?;
                    (0x4389a4, 0x400a8c)
                };
                let handle = self.lib(0x400070, vec![P(mode), P(name)])?;
                if handle == 0 {
                    if tutorial {
                        self.full(0);
                        return Ok(());
                    }
                    let child_frame = frame.wrapping_sub(62);
                    self.word(child_frame.wrapping_add(8), 200)?;
                    let child = load::scratch(&self.plan.state.memory, child_frame, self.s)?;
                    self.plan.state.memory = child.memory;
                    self.plan
                        .effects
                        .extend(child.effects.into_iter().map(PromptControlEffect::Load));
                    self.plan.flow = child.flow;
                    if self.plan.flow != LoadFlow::Returned {
                        self.returned(PromptReturn::NoReturn);
                        return Ok(());
                    }
                    self.d0 = child.native_return;
                }
                self.lib(0x400038, vec![W(self.w(frame.wrapping_add(10))?)])?;
                let absolute = self.d0.expect("observed absolute result") as u16 as i16;
                self.lib(
                    0x400078,
                    vec![W(0), L((absolute as i32 as u32).wrapping_shl(8)), L(handle)],
                )?;
                self.lib(0x400048, vec![L(handle), L(1), L(256), P(0x479ed8)])?;
                self.lib(0x400068, vec![L(handle)])?;
                self.lib(0x400038, vec![W(self.w(frame.wrapping_add(10))?)])?;
                self.word(0x4427c4, self.d0.expect("observed absolute result") as u16)?;
            }
            _ => {
                self.observe(FeedbackRequest::GetIndexedString {
                    destination: 0x479ed8,
                    resource: 1,
                    index: 3,
                })?;
                self.observe(FeedbackRequest::GetIndexedString {
                    destination: 0x479ed8,
                    resource: self.sw(frame.wrapping_add(8))?,
                    index: self.sw(frame.wrapping_add(10))?,
                })?;
            }
        }
        self.trap(0xa887, vec![W(self.w(0x402f44)?)])?;
        self.trap(0xa88a, vec![W(12)])?;
        self.trap(0xa888, vec![W(0)])?;
        self.trap(0xaa0b, vec![L(self.l(0x4358ce)?)])?;
        let edit = self.trap(
            0xa9d2,
            vec![P(frame.wrapping_sub(8)), P(frame.wrapping_sub(8))],
        )?;
        self.trap(0xa9cf, vec![P(0x479ed8), L(255), L(edit)])?;
        self.trap(0xa9d1, vec![L(0), L(1), L(edit)])?;
        if self.w(0x438776)? == 0 {
            self.trap(0xa8a3, vec![P(frame.wrapping_sub(8))])?;
        } else {
            self.word(0x4434bc, 0)?;
            self.word(
                0x4434ba,
                self.w(0x40856e)?
                    .wrapping_add(320)
                    .wrapping_sub(self.w(0x439582)?.wrapping_shl(5)),
            )?;
            self.word(0x4434c0, self.w(0x40856c)?.wrapping_add(308))?;
            self.word(
                0x4434be,
                self.w(0x40856e)?
                    .wrapping_add(460)
                    .wrapping_sub(self.w(0x439582)?.wrapping_shl(5)),
            )?;
            self.pict(
                frame.wrapping_sub(70),
                221_u16.wrapping_sub(self.w(0x439582)?),
                0x4434ba,
            )?;
        }
        if self.w(frame.wrapping_add(14))? == 0 {
            self.word(frame.wrapping_sub(4), 333)?;
        }
        self.trap(0xa9d7, vec![L(edit)])?;
        let mut source = 0_i16;
        while source < i16::from(self.b(0x479ed8)?) {
            self.step()?;
            text_position = text_position.wrapping_add(1);
            if self.b(index(0x479ed8, source, 1))? == b'^' {
                let mut member = self
                    .b(index(0x479ed8, source.wrapping_add(1), 1))?
                    .wrapping_sub(b'1') as i8;
                if self.sw(record(0x448aa0, i16::from(member), 3138).wrapping_add(80))? < 1 {
                    let mut candidate = self.sb(0x44273a)?;
                    while candidate > -1 {
                        self.step()?;
                        if self.sw(record(0x448aa0, candidate, 3138).wrapping_add(80))? > 0 {
                            member = candidate as i8;
                            break;
                        }
                        candidate = candidate.wrapping_sub(1);
                    }
                }
                if member == 8 {
                    if self.sw(0x402f48)? <= -1 {
                        for candidate in 0..7_i16 {
                            if self.w(index(0x4434e0, candidate, 2))? != 0 {
                                self.word(0x402f48, candidate as u16)?;
                                member = candidate as i8;
                                break;
                            }
                        }
                    } else {
                        member = self.b(0x402f49)? as i8;
                    }
                } else if member < 0 || member > self.b(0x44273a)? as i8 || member > 5 {
                    member = 0;
                }
                let name = record(0x448aa0, i16::from(member), 3138).wrapping_add(0xa21);
                self.lib(0x400088, vec![P(name)])?;
                self.word(0x4434b6, self.d0.expect("observed strlen result") as u16)?;
                self.long(0x4427b6, self.sw(0x4434b6)? as i32 as u32)?;
                self.trap(
                    0xa9d1,
                    vec![
                        L(text_position.wrapping_sub(1) as i32 as u32),
                        L(text_position.wrapping_add(1) as i32 as u32),
                        L(edit),
                    ],
                )?;
                self.trap(0xa9d7, vec![L(edit)])?;
                let name = record(0x448aa0, i16::from(member), 3138).wrapping_add(0xa21);
                self.trap(0xa9de, vec![P(name), L(self.l(0x4427b6)?), L(edit)])?;
                self.trap(
                    0xa83d,
                    vec![W(15), P(frame.wrapping_sub(20)), B(1), L(edit), W(1)],
                )?;
                text_position =
                    text_position.wrapping_add(self.w(0x4434b6)?.wrapping_sub(2) as i16);
            }
            source = source.wrapping_add(1);
        }
        if self.w(frame.wrapping_add(14))? == 0 {
            self.word(
                frame.wrapping_sub(4),
                448_u16.wrapping_sub(self.w(0x439582)?.wrapping_mul(11)),
            )?;
        }
        self.trap(0xa9d3, vec![P(frame.wrapping_sub(8)), L(edit)])?;
        self.trap(0xa9cd, vec![L(edit)])?;
        if self.w(0x438760)? == 0 && self.b(0x442752)? == 0 {
            self.sound(5000)?;
        }
        if self.w(frame.wrapping_add(12))? != 0 {
            self.trap(0xaa1c, vec![L(self.l(0x439538)?)])?;
            self.delay(10)?;
            self.long(0x44720c, 0)?;
            self.word(0x447218, 0)?;
            self.word(0x44720a, 0)?;
            self.flush()?;
            loop {
                self.step()?;
                self.trap(0xa860, vec![W(65535), P(0x44720a), L(0), L(0)])?;
                if self.w(0x438776)? != 0 {
                    let mut member = 0_i16;
                    while member < 6 && self.l(index(0x43949c, member, 4))? != 0 {
                        member += 1;
                    }
                    if member == 6 {
                        self.delay(120)?;
                        break;
                    }
                }
                self.word(frame.wrapping_sub(54), 0)?;
                self.music(frame.wrapping_sub(62))?;
                if self.plan.flow != LoadFlow::Returned {
                    return Ok(());
                }
                if matches!(self.w(0x44720a)?, 1 | 3) {
                    break;
                }
            }
            self.trap(0xa8a3, vec![P(frame.wrapping_sub(8))])?;
            self.trap(0xaa1c, vec![L(self.l(0x43952c)?)])?;
        }
        self.flush()?;
        self.word(frame.wrapping_add(14), 0)?;
        self.trap(0xa887, vec![W(self.w(0x402f3c)?)])?;
        self.port(self.l(frame.wrapping_sub(24))?)?;
        self.full(0);
        Ok(())
    }
    fn first(&mut self, frame: u32) -> Result<(), RuleError> {
        self.step()?;
        let mut current = self.sw(frame.wrapping_add(8))?;
        let last = self.sw(frame.wrapping_add(10))?;
        if self.w(0x439596)? == 0 {
            return Ok(());
        }
        loop {
            self.step()?;
            if self.b(index(0x4042f2, current, 1))? == 0 {
                break;
            }
            current = current.wrapping_add(1);
            if current > last {
                return Ok(());
            }
        }
        self.trap(0xa874, vec![P(frame.wrapping_sub(4))])?;
        self.trap(0xaa19, vec![P(frame.wrapping_sub(26))])?;
        self.trap(0xaa1a, vec![P(frame.wrapping_sub(32))])?;
        self.trap(0xaa1c, vec![L(self.l(0x43952c)?)])?;
        self.trap(0xa862, vec![L(33)])?;
        self.trap(0xa863, vec![L(30)])?;
        let first = current;
        let dialog = self.trap(0xa97c, vec![W(130), L(0), L(u32::MAX)])?;
        self.port(dialog)?;
        self.trap(
            0xa91b,
            vec![
                L(dialog),
                W(self.w(0x4395a4)?.wrapping_add(5)),
                W(self.w(0x4395a2)?.wrapping_add(5)),
                B(0),
            ],
        )?;
        self.long(frame.wrapping_sub(12), self.l(dialog.wrapping_add(16))?)?;
        self.long(frame.wrapping_sub(8), self.l(dialog.wrapping_add(20))?)?;
        self.long(frame.wrapping_sub(20), self.l(dialog.wrapping_add(16))?)?;
        self.long(frame.wrapping_sub(16), self.l(dialog.wrapping_add(20))?)?;
        self.add(frame.wrapping_sub(14), 15)?;
        self.add(frame.wrapping_sub(16), 40)?;
        self.add(frame.wrapping_sub(12), self.sw(0x4395a2)?)?;
        self.add(frame.wrapping_sub(8), self.sw(0x4395a2)?.wrapping_add(40))?;
        self.add(frame.wrapping_sub(10), self.sw(0x4395a4)?)?;
        self.add(frame.wrapping_sub(6), self.sw(0x4395a4)?.wrapping_add(15))?;
        self.trap(0xa862, vec![L(33)])?;
        self.trap(0xa863, vec![L(30)])?;
        self.trap(
            0xa8ec,
            vec![
                P(0x3fe288),
                P(self.l(0x4386b4)?.wrapping_add(2)),
                P(frame.wrapping_sub(12)),
                P(frame.wrapping_sub(20)),
                W(0),
                L(0),
            ],
        )?;
        self.trap(0xaa0b, vec![L(self.l(0x4358ce)?)])?;
        self.trap(0xa862, vec![L(69)])?;
        for selector in [0xa915, 0xa981, 0xa922, 0xa923] {
            self.trap(selector, vec![L(dialog)])?;
        }
        for (offset, value) in [(40, 17), (38, 17), (36, 135), (34, 305)] {
            self.word(frame.wrapping_sub(offset), value)?;
        }
        loop {
            self.step()?;
            self.trap(0xa8a3, vec![P(frame.wrapping_sub(40))])?;
            let child = frame.wrapping_sub(84);
            self.long(child.wrapping_add(20), self.l(frame.wrapping_sub(36))?)?;
            self.long(child.wrapping_add(16), self.l(frame.wrapping_sub(40))?)?;
            self.long(child.wrapping_add(12), 1)?;
            self.word(child.wrapping_add(10), current as u16)?;
            self.word(child.wrapping_add(8), (-2_i16) as u16)?;
            self.textbox(child)?;
            if self.plan.flow != LoadFlow::Returned {
                return Ok(());
            }
            self.byte(index(0x4042f2, current, 1), 1)?;
            loop {
                self.step()?;
                self.trap(0xa9b4, vec![])?;
                self.word(frame.wrapping_sub(62), 0)?;
                self.music(frame.wrapping_sub(70))?;
                if self.plan.flow != LoadFlow::Returned {
                    return Ok(());
                }
                self.trap(0xa970, vec![W(65535), P(0x44720a)])?;
                self.trap(0xa853, vec![])?;
                let flash_frame = frame.wrapping_sub(70);
                self.word(flash_frame.wrapping_add(8), current as u16)?;
                let child =
                    combat_ui::flashrect(&self.plan.state.memory, flash_frame, current, self.s)?;
                self.plan.state.memory = child.memory;
                self.plan
                    .effects
                    .extend(child.effects.into_iter().map(PromptControlEffect::Combat));
                if let Some(d0) = child.native_return {
                    self.full(d0);
                } else {
                    self.d0 = None;
                }
                if self.trap(0xa97f, vec![P(0x44720a)])? as u8 != 0
                    && self.trap(0xa980, vec![P(0x44720a), P(0x4394c0), P(0x4434b2)])? as u8 != 0
                {
                    break;
                }
            }
            current = current.wrapping_add(1);
            if self.w(0x4434b2)? == 6 {
                self.word(0x439596, 0)?;
                self.sound(141)?;
                break;
            }
            if self.w(0x4434b2)? == 1 || self.w(0x4434b2)? == 3 {
                self.sound(148)?;
                let mut n = first;
                while n <= last {
                    self.step()?;
                    self.byte(index(0x4042f2, n, 1), 1)?;
                    n = n.wrapping_add(1);
                }
                break;
            }
            self.sound(141)?;
            if self.w(0x4434b2)? == 4 {
                current = current.wrapping_sub(2);
                if current < first {
                    current = first;
                }
            } else if current > last {
                break;
            }
        }
        self.trap(0xa983, vec![L(dialog)])?;
        self.trap(0xaa1c, vec![L(self.l(0x43952c)?)])?;
        self.port(self.l(frame.wrapping_sub(4))?)?;
        self.trap(0xa862, vec![L(33)])?;
        self.trap(0xa863, vec![L(30)])?;
        self.trap(
            0xa8ec,
            vec![
                P(self.l(0x4386b4)?.wrapping_add(2)),
                P(0x3fe288),
                P(frame.wrapping_sub(20)),
                P(frame.wrapping_sub(12)),
                W(0),
                L(0),
            ],
        )?;
        self.trap(0xaa15, vec![P(frame.wrapping_sub(32))])?;
        self.trap(0xaa14, vec![P(frame.wrapping_sub(26))])?;
        self.trap(0xa922, vec![L(self.l(frame.wrapping_sub(4))?)])?;
        self.trap(0xa923, vec![L(self.l(frame.wrapping_sub(4))?)])?;
        if self.w(0x4434b2)? == 6 {
            self.flash(frame.wrapping_sub(80), 0x4037aa, 50, 250, 0, 6000)?;
        }
        let handle = self.lib(0x400070, vec![P(0x403790), P(0x403794)])?;
        self.long(0x4434a6, handle)?;
        if handle != 0 {
            self.lib(
                0x400050,
                vec![L(self.l(0x4434a6)?), L(1), L(1000), P(0x4042f2)],
            )?;
            self.lib(
                0x400050,
                vec![L(self.l(0x4434a6)?), L(1), L(16000), P(0x4046ec)],
            )?;
            self.lib(0x400068, vec![L(self.l(0x4434a6)?)])?;
        }
        self.flush()?;
        if let Some(d0) = self.d0 {
            self.full(d0);
        }
        Ok(())
    }
    fn cursor(&mut self, id: u16) -> Result<(), RuleError> {
        let cursor = self.trap(0xaa1b, vec![W(id)])?;
        self.trap(0xaa1c, vec![L(cursor)])?;
        self.trap(0xaa26, vec![L(cursor)])?;
        Ok(())
    }
    fn choice(&mut self, frame: u32) -> Result<(), RuleError> {
        self.step()?;
        self.word(frame.wrapping_sub(8), 0)?;
        self.word(
            frame.wrapping_sub(4),
            self.sb(0x44273a)?.wrapping_add(1).wrapping_mul(50) as u16,
        )?;
        self.word(frame.wrapping_sub(6), self.w(0x40856c)?.wrapping_add(332))?;
        self.word(
            frame.wrapping_sub(2),
            self.w(frame.wrapping_sub(6))?.wrapping_add(60),
        )?;
        self.word(0x402f48, u16::MAX)?;
        let excluded = if self.w(frame.wrapping_add(12))? == 0 {
            self.sb(0x442739)?
        } else {
            0
        };
        let admitted = self.sb(0x44273a)?.wrapping_sub(excluded);
        if admitted < self.sw(frame.wrapping_add(8))? {
            self.word(frame.wrapping_add(8), admitted as u16)?;
        }
        let mut remaining = self.sw(frame.wrapping_add(8))?.wrapping_add(1);
        for n in 0..8_i16 {
            self.word(index(0x4434e0, n, 2), 0)?;
        }
        let tutorial = frame.wrapping_sub(42);
        self.word(tutorial.wrapping_add(8), 105)?;
        // The source pushes only this first WORD; its second argument is caller storage.
        self.first(tutorial)?;
        if self.plan.flow != LoadFlow::Returned {
            return Ok(());
        }
        self.cursor(remaining.wrapping_add(147) as u16)?;
        loop {
            self.step()?;
            self.trap(0xa860, vec![W(65535), P(0x44720a), L(0), L(0)])?;
            if self.trap(0xa858, vec![L(u32::from(self.w(0x447218)?)), L(1024)])? != 0 {
                self.warn(frame.wrapping_sub(42), 21)?;
            }
            match self.w(0x44720a)? {
                3 => {
                    let key = self.w(0x44720e)?;
                    if key == 0x61 {
                        self.word(0x4434b2, u16::MAX)?;
                        for n in 0..8_i16 {
                            self.word(index(0x4434e0, n, 2), 0)?;
                        }
                        self.word(0x438760, 0)?;
                        break;
                    }
                    if key == 0x3120 && self.w(0x438760)? != 0 {
                        break;
                    }
                    continue;
                }
                1 => {}
                _ => continue,
            }
            self.long(0x4471ee, self.l(0x447214)?)?;
            self.trap(0xa871, vec![P(0x4471ee)])?;
            if self.trap(0xa8ad, vec![L(self.l(0x4471ee)?), P(frame.wrapping_sub(8))])? as u8 == 0 {
                self.sound(6000)?;
                continue;
            }
            self.word(0x4434b2, (self.sw(0x4471ee)? / 50) as u16)?;
            self.word(0x4434ba, self.sw(0x4434b2)?.wrapping_mul(50) as u16)?;
            self.word(0x4434bc, self.w(0x40856c)?.wrapping_add(330))?;
            self.word(0x4434c0, self.w(0x4434bc)?.wrapping_add(50))?;
            self.word(0x4434be, self.w(0x4434ba)?.wrapping_add(50))?;
            self.long(0x44292c, self.l(0x4434ba)?)?;
            self.long(0x442930, self.l(0x4434be)?)?;
            if self.w(frame.wrapping_add(12))? == 0
                && self.sw(record(0x448aa0, self.sw(0x4434b2)?, 3138).wrapping_add(80))? < 1
            {
                self.sound(6000)?;
                continue;
            }
            self.press(frame.wrapping_sub(54), 1, 1, 0, 0x44292c)?;
            let slot = index(0x4434e0, self.sw(0x4434b2)?, 2);
            if self.w(slot)? == 0 {
                remaining = remaining.wrapping_sub(1);
                self.sound(141)?;
                self.cursor(remaining.wrapping_add(147) as u16)?;
                self.word(
                    index(0x4434e0, self.sw(0x4434b2)?, 2),
                    remaining.wrapping_add(1) as u16,
                )?;
                self.word(0x4471f2, self.w(0x4434ba)?.wrapping_add(9))?;
                self.word(0x4471f4, self.w(0x4434bc)?.wrapping_add(9))?;
                self.icon2(remaining.wrapping_add(2201))?;
            } else {
                remaining = remaining.wrapping_add(1);
                self.sound(144)?;
                let removed = self.sw(index(0x4434e0, self.sw(0x4434b2)?, 2))?;
                self.word(index(0x4434e0, self.sw(0x4434b2)?, 2), 0)?;
                let mut n = 0_i16;
                while n <= self.sb(0x44273a)? {
                    self.step()?;
                    let slot = index(0x4434e0, n, 2);
                    if removed > self.sw(slot)? && self.w(slot)? != 0 {
                        self.add(slot, 1)?;
                    }
                    n = n.wrapping_add(1);
                }
                self.cursor(remaining.wrapping_add(147) as u16)?;
                let mut n = 0_i16;
                while n <= self.sb(0x44273a)? {
                    self.step()?;
                    self.character_box(frame.wrapping_sub(46), n as u16, 1)?;
                    if self.w(index(0x4434e0, n, 2))? != 0 {
                        self.icon2(self.sw(index(0x4434e0, n, 2))?.wrapping_add(2200))?;
                    }
                    n = n.wrapping_add(1);
                }
                self.controls(frame.wrapping_sub(40))?;
                if self.plan.flow != LoadFlow::Returned {
                    return Ok(());
                }
            }
            self.press(frame.wrapping_sub(54), 1, 2, 0, 0x44292c)?;
            if remaining == 0 {
                break;
            }
        }
        self.trap(0xaa1c, vec![L(self.l(0x43952c)?)])?;
        if self.w(frame.wrapping_add(10))? == 0 {
            if self.w(0x438776)? == 0 {
                self.word(frame.wrapping_sub(34), 0)?;
                self.reset(frame.wrapping_sub(42))?;
                if self.plan.flow != LoadFlow::Returned {
                    return Ok(());
                }
            }
            let mut n = 0_i16;
            while n <= self.sb(0x44273a)? {
                self.step()?;
                self.character_box(frame.wrapping_sub(46), n as u16, 0x10001)?;
                n = n.wrapping_add(1);
            }
        }
        self.controls(frame.wrapping_sub(40))?;
        if self.plan.flow != LoadFlow::Returned {
            return Ok(());
        }
        self.low(self.w(0x4434b2)?.wrapping_add(1));
        Ok(())
    }
}

fn index(base: u32, n: i16, stride: i32) -> u32 {
    base.wrapping_add((i32::from(n).wrapping_mul(stride)) as u32)
}
fn record(base: u32, n: i16, stride: u32) -> u32 {
    base.wrapping_add(u32::from(n as u16) * stride)
}
fn half(value: i16) -> i16 {
    value.wrapping_add(((value as u16) >> 15) as i16) >> 1
}
fn c_string(memory: &RenderingMemory, address: u32) -> Result<Vec<u8>, RuleError> {
    let mut bytes = vec![];
    for n in 0..=memory.bytes.len() {
        let byte = memory.byte(address.wrapping_add(n as u32))?;
        bytes.push(byte);
        if byte == 0 {
            return Ok(bytes);
        }
    }
    Err(RuleError::Invalid("unterminated prompt C string".into()))
}
struct FixedDrawing {
    handle: u32,
}
impl DrawingServices for FixedDrawing {
    fn color_icon(&mut self, _: i16) -> Result<u32, RuleError> {
        Ok(self.handle)
    }
    fn picture(&mut self, _: i16) -> Result<PictureObservation, RuleError> {
        Ok(PictureObservation {
            handle: self.handle,
            bounds: None,
        })
    }
    fn indexed_string(&mut self, _: i16, _: i16) -> Result<PascalString, RuleError> {
        Err(RuleError::Invalid("unused prompt drawing string".into()))
    }
    fn current_world(&mut self) -> Result<(u32, u32), RuleError> {
        Err(RuleError::Invalid("unused prompt drawing world".into()))
    }
    fn new_world(&mut self, _: i16, _: BodyRect) -> Result<(i16, BuildingWorld), RuleError> {
        Err(RuleError::Invalid(
            "unused prompt drawing allocation".into(),
        ))
    }
}
struct FixedNumber(PascalString);
impl NumericServices for FixedNumber {
    fn number_to_string(&mut self, _: i32) -> Result<PascalString, RuleError> {
        Ok(self.0.clone())
    }
    fn string_to_number(&mut self, _: &PascalString) -> Result<i32, RuleError> {
        Err(RuleError::Invalid("unused prompt numeric parse".into()))
    }
}
fn icon_context(rect: BodyRect) -> DrawingContext {
    DrawingContext {
        rect,
        banner_rect: rect,
        source_rect: rect,
        source_row: 0,
        source_column: 0,
        icon_handle: 0,
        item_work: [0; ITEM_BYTES],
        text_work: PascalString::default(),
        window: 0,
        item_window: 0,
        half_width_input: 0,
        active_member: 0,
        omit_banner: 0,
        show_item_background: 0,
        text_in_other_port: 0,
        erase_pattern: 0,
        default_font: 0,
        selection: 0,
        map_port: 0,
        text_port: 0,
        secondary_port: 0,
        backing_port: 0,
        tile_port: 0,
        depth: 0,
        old_world: 0,
        old_device: 0,
        buildings: vec![],
    }
}
struct Clock<'a, 'b, S> {
    execution: &'a mut Execution<'b, S>,
    remaining: usize,
}
impl<S: PromptControlServices> TimingServices for Clock<'_, '_, S> {
    fn tick_count(&mut self) -> Result<u32, RuleError> {
        self.remaining = self.remaining.checked_sub(1).ok_or_else(|| {
            RuleError::Invalid("prompt delay observation budget exhausted".into())
        })?;
        self.execution.observe(FeedbackRequest::TickCount)
    }
    fn button(&mut self) -> Result<bool, RuleError> {
        Ok(self.execution.observe(FeedbackRequest::Button)? as u8 != 0)
    }
}
struct MemorySpatial<'a>(&'a RenderingMemory);
impl SpatialServices for MemorySpatial<'_> {
    fn warrior_position(&self, n: i16) -> Option<(i16, i16)> {
        Some((
            self.0.word(index(0x44709a, n, 4)).ok()? as i16,
            self.0.word(index(0x44709c, n, 4)).ok()? as i16,
        ))
    }
    fn enemy_position(&self, n: i16) -> Option<(i16, i16)> {
        Some((
            self.0.word(index(0x446f0a, n, 4)).ok()? as i16,
            self.0.word(index(0x446f0c, n, 4)).ok()? as i16,
        ))
    }
    fn warrior_byte(&self, n: i16, offset: usize) -> Option<u8> {
        self.0
            .byte(record(0x448aa0, n, 3138).wrapping_add(offset as u32))
            .ok()
    }
    fn warrior_word(&self, n: i16, offset: usize) -> Option<i16> {
        self.0
            .word(record(0x448aa0, n, 3138).wrapping_add(offset as u32))
            .ok()
            .map(|v| v as i16)
    }
    fn enemy_byte(&self, n: i16, offset: usize) -> Option<u8> {
        self.0
            .byte(record(0x439d3c, n, 288).wrapping_add(offset as u32))
            .ok()
    }
    fn enemy_word(&self, n: i16, offset: usize) -> Option<i16> {
        self.0
            .word(record(0x439d3c, n, 288).wrapping_add(offset as u32))
            .ok()
            .map(|v| v as i16)
    }
    fn map_word(&self, x: i32, y: i32) -> Option<i16> {
        self.0
            .word(
                0x4085b8_u32
                    .wrapping_add(x.wrapping_mul(180).wrapping_add(y.wrapping_mul(2)) as u32),
            )
            .ok()
            .map(|v| v as i16)
    }
}

macro_rules! entry {
    ($public:ident, $body:ident) => {
        pub fn $public(
            state: &PromptControlState,
            frame: u32,
            limits: PromptLimits,
            services: &mut impl PromptControlServices,
        ) -> Result<PromptControlPlan, RuleError> {
            let mut execution = Execution {
                plan: PromptControlPlan {
                    state: state.clone(),
                    effects: vec![],
                    native_return: PromptReturn::Unchanged,
                    flow: LoadFlow::Returned,
                },
                s: services,
                limits,
                remaining: limits.maximum_steps,
                d0: None,
            };
            execution.$body(frame)?;
            Ok(execution.plan)
        }
    };
}
entry!(question, question);
entry!(submit_music, music);
entry!(text_box, textbox);
entry!(first_tutorial, first);
entry!(update_controls, controls);
entry!(reset_text_area, reset);
entry!(get_choice, choice);

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::recovered_audio::AudioObservation;
    use crate::recovered_character_panels::{CharacterPanelObservation, CharacterPanelRequest};
    use crate::recovered_combat_ui::{CombatUiObservation, CombatUiRequest, CombatUiServices};
    use crate::recovered_equipment_ui::EquipmentUiServices;
    use crate::recovered_feedback::FeedbackObservation;
    use crate::recovered_gamma::{
        GammaError, GammaMemory, GammaObservation, GammaRequest, GammaServices,
    };
    use crate::recovered_gamma_control::{GammaControlRequest, GammaControlServices};
    use crate::recovered_interface::{InterfaceServices, Point};
    use crate::recovered_inventory_rules::SaneRequest;
    use crate::recovered_item_ui::{ItemUiReply, ItemUiRequest, ItemUiServices};
    use crate::recovered_load_control::{LoadObservation, LoadRequest, VersionInputs};
    use crate::recovered_rendering::{RenderingObservation, RenderingRequest, RenderingServices};
    use crate::recovered_runtime::RuntimeObservation;
    use crate::recovered_world_display::{
        WorldDisplayObservation, WorldDisplayRequest, WorldDisplayServices,
    };
    use std::cell::Cell;
    pub(crate) const FRAME: u32 = 0x8000;
    fn unused() -> RuleError {
        RuleError::Invalid("unexpected focused prompt observation".into())
    }
    pub(crate) fn limits() -> PromptLimits {
        PromptLimits {
            maximum_steps: 300,
            maximum_polls: 20,
            filename_capacity: 64,
        }
    }
    pub(crate) fn state() -> PromptControlState {
        let mut memory = RenderingMemory::default();
        for (a, n) in [
            (0x3fe280, 0x100),
            (0x402f30, 0x20),
            (0x4042f2, 1000),
            (0x408560, 0x30),
            (0x435600, 0x400),
            (0x438500, 0x1200),
            (0x442700, 0x1300),
            (0x447000, 0x300),
            (0x448aa0, 0xc42),
            (0x479ed8, 256),
            (0x7800, 0x820),
            (0xa000, 0x100),
            (0xd000, 0x40),
        ] {
            memory.insert_region(a, &vec![0; n]);
        }
        memory.insert_region(0x3fe302, &0xa000_u32.to_be_bytes());
        memory.insert_region(0x43878c, &0xa000_u32.to_be_bytes());
        memory.insert_region(0x43879c, &0xa100_u32.to_be_bytes());
        memory.insert_region(0x40856c, &100_u16.to_be_bytes());
        memory.insert_region(0x40856e, &200_u16.to_be_bytes());
        memory.insert_region(0x439576, &u16::MAX.to_be_bytes());
        memory.insert_region(0xd010, &[0, 0, 0, 0, 0, 100, 0, 100]);
        memory.insert_region(0x447202, &[0, 8, 0, 0, 0, 58, 0, 0]);
        PromptControlState {
            memory,
            random: RandomState::new(4),
        }
    }
    pub(crate) fn argument(state: &mut PromptControlState, offset: u32, value: u16) {
        state
            .memory
            .insert_region(FRAME + offset, &value.to_be_bytes());
    }
    fn value(arg: &FeedbackArg) -> u32 {
        match arg {
            W(w) => u32::from(*w),
            L(l) | P(l) => *l,
            B(b) => u32::from(*b),
        }
    }
    pub(crate) struct Observed {
        pub(crate) port_d0: u32,
        pub(crate) reset_alias: Option<u16>,
        pub(crate) busy: u8,
        pub(crate) choice_abort: bool,
        pub(crate) ticks: Cell<u32>,
    }
    impl Observed {
        pub(crate) fn new() -> Self {
            Self {
                port_d0: 0xabcd5555,
                reset_alias: None,
                busy: 1,
                choice_abort: false,
                ticks: Cell::new(0),
            }
        }
    }
    impl FeedbackServices for Observed {
        fn observe(
            &self,
            request: &FeedbackRequest,
            memory: &RenderingMemory,
        ) -> Result<FeedbackObservation, RuleError> {
            let mut memory = memory.clone();
            let mut result = 0;
            let mut d0 = 0xfeed1234;
            match request {
                FeedbackRequest::Toolbox { opcode, arguments } => {
                    let at = |n: usize| value(&arguments[n]);
                    match opcode {
                        0xa874 => {
                            let port = memory.long(0x3fe302)?;
                            memory.insert_region(at(0), &port.to_be_bytes());
                        }
                        0xa873 => {
                            memory.insert_region(0x3fe302, &at(0).to_be_bytes());
                            d0 = self.port_d0;
                            if let Some(mode) = self.reset_alias {
                                memory.insert_region(FRAME + 8, &mode.to_be_bytes());
                            }
                        }
                        0xa8a3 => d0 = 0xdeadbeef,
                        0xaa19 | 0xaa1a => memory.insert_region(at(0), &[1, 2, 3, 4, 5, 6]),
                        0xa97c => result = 0xd000,
                        0xaa1b => result = 0xe000 + at(0),
                        0xa9d2 => result = 0xb000,
                        0xa9bc => result = 0,
                        0xaa1e => result = 0,
                        0xa97f => result = 1,
                        0xa980 => {
                            result = 1;
                            memory.insert_region(at(2), &2_u16.to_be_bytes());
                        }
                        0xa98d => memory.insert_region(at(4), &[0, 1, 0, 2, 0, 3, 0, 4]),
                        0xa804 => result = u32::from(memory.word(at(1))?) + 20,
                        0xa860 | 0xa970 if self.choice_abort => {
                            memory.insert_region(0x44720a, &3_u16.to_be_bytes());
                            memory.insert_region(0x44720e, &0x61_u16.to_be_bytes());
                        }
                        0xa860 | 0xa970 => memory.insert_region(0x44720a, &1_u16.to_be_bytes()),
                        0xa858 => result = 0,
                        0xa924 => result = 0xa000,
                        _ => {}
                    }
                }
                FeedbackRequest::GetIndexedString {
                    destination,
                    resource,
                    index,
                } => {
                    memory.insert_region(
                        *destination,
                        if (*resource, *index) == (133, 1) {
                            b"\x05^1abc"
                        } else {
                            b"\0"
                        },
                    );
                }
                FeedbackRequest::TickCount => {
                    result = self.ticks.get();
                    self.ticks.set(result + 3);
                    d0 = result;
                }
                _ => return Err(unused()),
            }
            Ok(FeedbackObservation {
                value: result,
                d0,
                memory,
            })
        }
        fn sound_observations(
            &self,
            _: i16,
            _: &AudioContext,
            _: &RenderingMemory,
        ) -> Result<Vec<AudioObservation>, RuleError> {
            Err(unused())
        }
    }
    impl PromptControlServices for Observed {
        fn observe_registers(
            &mut self,
            request: &PromptRegisterRequest,
            memory: &RenderingMemory,
        ) -> Result<FeedbackObservation, RuleError> {
            let mut memory = memory.clone();
            match request.opcode {
                0xa800 => {
                    assert_eq!(request.d0_input, Some(0x05100008));
                    assert_eq!(request.a0_input, None);
                    assert_eq!(request.arguments.len(), 3);
                    assert_eq!(request.arguments[1], W(24));
                    let destination = value(&request.arguments[2]);
                    memory.insert_region(destination, &[0; 24]);
                    memory.insert_region(destination + 12, &[self.busy]);
                }
                0xa032 => {
                    assert_eq!(request.d0_input, Some(0xffff));
                    assert_eq!(request.a0_input, None);
                    assert!(request.arguments.is_empty());
                }
                0xa029 | 0xa02a => {
                    assert_eq!(request.d0_input, None);
                    assert_eq!(request.a0_input, Some(memory.long(0x4394dc)?));
                    assert!(request.arguments.is_empty());
                }
                _ => return Err(unused()),
            }
            Ok(FeedbackObservation {
                value: 0,
                d0: 0xfeed1234,
                memory,
            })
        }
        fn library(
            &mut self,
            request: &LibraryRequest,
            memory: &RenderingMemory,
        ) -> Result<LibraryObservation, RuleError> {
            let mut memory = memory.clone();
            let mut result = 0;
            let at = |n: usize| value(&request.arguments[n]);
            match request.entry {
                0x400038 => result = i32::from((at(0) as u16 as i16).wrapping_abs()) as u32,
                0x400070 => {}
                0x400088 => result = (c_string(&memory, at(0))?.len() - 1) as u32,
                0x400168 => {
                    let p = PascalString::from_payload((at(0) as i32).to_string().as_bytes())?;
                    memory.insert_region(at(1), p.raw());
                }
                _ => return Err(unused()),
            }
            Ok(LibraryObservation {
                memory,
                value: result,
                d0: result,
            })
        }
        fn quiet_observations(
            &mut self,
            _: i16,
            context: &AudioContext,
            _: &RenderingMemory,
        ) -> Result<Vec<AudioObservation>, RuleError> {
            if context.last_channel < 1 {
                Ok(vec![])
            } else {
                Err(unused())
            }
        }
        fn incoming_mechanical_mode(&mut self, _: &RenderingMemory) -> Result<i16, RuleError> {
            Err(unused())
        }
    }
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
    impl CharacterPanelServices for Observed {
        fn panel_observe(
            &mut self,
            _: &CharacterPanelRequest,
            _: &RenderingMemory,
        ) -> Result<CharacterPanelObservation, RuleError> {
            Err(unused())
        }
        fn runtime_observations(
            &mut self,
            _: u32,
            _: &RenderingMemory,
        ) -> Result<Vec<RuntimeObservation>, RuleError> {
            Err(unused())
        }
    }
    impl InventoryServices for Observed {
        fn sane(
            &self,
            _: &SaneRequest,
            _: &RenderingMemory,
        ) -> Result<Vec<RecordWrite>, RuleError> {
            Err(unused())
        }
    }
    impl LoadServices for Observed {
        fn observe_load(
            &self,
            _: &LoadRequest,
            _: &RenderingMemory,
        ) -> Result<LoadObservation, RuleError> {
            Err(unused())
        }
        fn version_inputs(
            &self,
            _: i16,
            _: i16,
            _: &RenderingMemory,
        ) -> Result<VersionInputs, RuleError> {
            Err(unused())
        }
        fn filename_number(&self, _: u8, _: &RenderingMemory) -> Result<PascalString, RuleError> {
            Err(unused())
        }
    }
    impl GammaServices for Observed {
        fn observe(
            &self,
            _: &GammaRequest,
            _: &GammaMemory,
        ) -> Result<GammaObservation, GammaError> {
            Err(GammaError::MissingService("unused prompt gamma".into()))
        }
    }
    impl GammaControlServices for Observed {
        fn observe_control(
            &self,
            _: &GammaControlRequest,
            _: &GammaMemory,
        ) -> Result<GammaObservation, GammaError> {
            Err(GammaError::MissingService(
                "unused prompt gamma control".into(),
            ))
        }
    }
    impl DrawingServices for Observed {
        fn color_icon(&mut self, _: i16) -> Result<u32, RuleError> {
            Err(unused())
        }
        fn indexed_string(&mut self, _: i16, _: i16) -> Result<PascalString, RuleError> {
            Err(unused())
        }
        fn current_world(&mut self) -> Result<(u32, u32), RuleError> {
            Err(unused())
        }
        fn picture(&mut self, _: i16) -> Result<PictureObservation, RuleError> {
            Err(unused())
        }
        fn new_world(&mut self, _: i16, _: BodyRect) -> Result<(i16, BuildingWorld), RuleError> {
            Err(unused())
        }
    }
    impl NumericServices for Observed {
        fn number_to_string(&mut self, _: i32) -> Result<PascalString, RuleError> {
            Err(unused())
        }
        fn string_to_number(&mut self, _: &PascalString) -> Result<i32, RuleError> {
            Err(unused())
        }
    }
    impl ItemUiServices for Observed {
        fn observe(
            &mut self,
            _: &ItemUiRequest,
            _: &RenderingMemory,
        ) -> Result<ItemUiReply, RuleError> {
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
    fn calls(plan: &PromptControlPlan, opcode: u16) -> Vec<&[FeedbackArg]> {
        plan.effects
            .iter()
            .filter_map(|e| match e {
                PromptControlEffect::Feedback(FeedbackEffect::Service {
                    request:
                        FeedbackRequest::Toolbox {
                            opcode: o,
                            arguments,
                        },
                    ..
                }) if *o == opcode => Some(arguments.as_slice()),
                _ => None,
            })
            .collect()
    }
    #[test]
    fn reset_preserves_observed_upper_word_and_rereads_mode_after_port() {
        let mut input = state();
        argument(&mut input, 8, 1);
        let result = reset_text_area(&input, FRAME, limits(), &mut Observed::new()).unwrap();
        assert_eq!(result.native_return, PromptReturn::Full(0xdeadbeef));
        assert_eq!(calls(&result, 0xa8a3), vec![vec![P(0x4471cc)].as_slice()]);
        let mut s = Observed::new();
        s.reset_alias = Some(2);
        let before = input.clone();
        let result = reset_text_area(&input, FRAME, limits(), &mut s).unwrap();
        assert_eq!(result.native_return, PromptReturn::Full(0xabcd0001));
        assert!(calls(&result, 0xa8a3).is_empty());
        assert_eq!(input, before);
    }
    #[test]
    fn controls_combat_order_gate_still_draws_native_prefix_button() {
        let mut input = state();
        input.memory.insert_region(0x438776, &1_u16.to_be_bytes());
        input.memory.insert_region(0x4470b8, &6_u16.to_be_bytes());
        let result = update_controls(&input, FRAME, limits(), &mut Observed::new()).unwrap();
        assert_eq!(result.native_return, PromptReturn::Full(0));
        assert_eq!(result.state.memory.word(0x447204).unwrap(), 430);
        assert_eq!(result.state.memory.word(0x447208).unwrap(), 480);
        assert_eq!(result.state.memory.word(FRAME - 30 + 8).unwrap(), 1);
        assert_eq!(calls(&result, 0xa870).len(), 2);
        assert!(calls(&result, 0xa9bc).is_empty());
        assert_eq!(
            result.state.memory.rect(FRAME - 8).unwrap(),
            BodyRect {
                top: 520,
                left: 0,
                bottom: 660,
                right: 408
            }
        );
    }
    #[test]
    fn textbox_zero_index_and_missing_tutorial_retain_early_return_writes() {
        let mut input = state();
        argument(&mut input, 10, 0);
        let result = text_box(&input, FRAME, limits(), &mut Observed::new()).unwrap();
        assert_eq!(result.effects.len(), 3);
        assert!(calls(&result, 0xa874).is_empty());
        assert_eq!(result.state.memory.word(FRAME - 16).unwrap(), 72);
        argument(&mut input, 8, (-2_i16) as u16);
        argument(&mut input, 10, 5);
        argument(&mut input, 14, 1);
        let result = text_box(&input, FRAME, limits(), &mut Observed::new()).unwrap();
        assert_eq!(result.native_return, PromptReturn::Full(0));
        assert_eq!(result.state.memory.word(FRAME + 14).unwrap(), 1);
        assert!(calls(&result, 0xa9d2).is_empty());
        assert!(calls(&result, 0xa873).is_empty());
    }
    #[test]
    fn textbox_name_replacement_uses_native_indices_and_clears_rect_argument() {
        let mut input = state();
        argument(&mut input, 8, 133);
        argument(&mut input, 10, 1);
        argument(&mut input, 14, 1);
        input
            .memory
            .insert_region(FRAME + 16, &[0, 10, 0, 20, 0, 30, 0, 40]);
        input.memory.insert_region(0x448af0, &10_u16.to_be_bytes());
        input.memory.insert_region(0x4494c1, b"NAME\0");
        let result = text_box(&input, FRAME, limits(), &mut Observed::new()).unwrap();
        assert_eq!(
            calls(&result, 0xa9d1),
            vec![
                vec![L(0), L(1), L(0xb000)].as_slice(),
                vec![L(0), L(2), L(0xb000)].as_slice()
            ]
        );
        assert_eq!(
            calls(&result, 0xa9de),
            vec![vec![P(0x4494c1), L(4), L(0xb000)].as_slice()]
        );
        assert_eq!(result.state.memory.word(0x4434b6).unwrap(), 4);
        assert_eq!(result.state.memory.long(0x4427b6).unwrap(), 4);
        assert_eq!(result.state.memory.word(FRAME + 14).unwrap(), 0);
        assert_eq!(result.native_return, PromptReturn::Full(0));
        assert_eq!(result.state.random, input.random);
    }
    #[test]
    fn music_busy_disable_and_forced_track_keep_real_rng_consumption() {
        let input = state();
        let result = submit_music(&input, FRAME, limits(), &mut Observed::new()).unwrap();
        assert_eq!(result.state.random, input.random);
        let mut disabled = input.clone();
        disabled
            .memory
            .insert_region(0x439594, &1_u16.to_be_bytes());
        let result = submit_music(&disabled, FRAME, limits(), &mut Observed::new()).unwrap();
        let commands: Vec<_> = result
            .effects
            .iter()
            .filter_map(|e| match e {
                PromptControlEffect::Write(w) if w.address == FRAME - 8 => Some(w.bytes.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(commands, [vec![0, 4], vec![0, 3]]);
        assert_eq!(result.state.memory.word(0x4386ac).unwrap(), 23);
        let mut forced = input.clone();
        argument(&mut forced, 8, 21000);
        forced.memory.insert_region(0x44270e, &[1; 13]);
        let mut s = Observed::new();
        s.busy = 0;
        let result = submit_music(&forced, FRAME, limits(), &mut s).unwrap();
        assert_eq!(result.state.random.draws, 1);
        assert_eq!(result.state.random.seed, 67228);
        assert!(result.effects.iter().any(|e| matches!(
            e,
            PromptControlEffect::Runtime(RuntimeEffect::Random { bound: 13, .. })
        )));
        assert!(calls(&result, 0xa9a0).is_empty());
        let mut enabled = forced.clone();
        enabled.memory.insert_region(0x439570, &1_u32.to_be_bytes());
        enabled.memory.insert_region(0x439576, &0_u16.to_be_bytes());
        let loaded = submit_music(&enabled, FRAME, limits(), &mut s).unwrap();
        let register_calls: Vec<_> = loaded
            .effects
            .iter()
            .filter_map(|effect| match effect {
                PromptControlEffect::RegisterToolbox { request, .. } => Some(request),
                _ => None,
            })
            .collect();
        assert_eq!(register_calls.len(), 3);
        assert_eq!(register_calls[0].d0_input, Some(0x05100008));
        assert_eq!(register_calls[1].opcode, 0xa02a);
        assert_eq!(register_calls[2].opcode, 0xa029);
        assert_eq!(register_calls[1].a0_input, Some(0));
        assert_eq!(register_calls[2].a0_input, Some(0));
        assert!(register_calls[1].arguments.is_empty());
        assert!(register_calls[2].arguments.is_empty());
        forced.memory.insert_region(0x44270e, &[0; 13]);
        let before = forced.clone();
        let short = PromptLimits {
            maximum_steps: 3,
            ..limits()
        };
        assert!(submit_music(&forced, FRAME, short, &mut s).is_err());
        assert_eq!(forced, before);
    }
    #[test]
    fn negative_question_preserves_actual_upper_d0_and_retained_button_frames() {
        let mut input = state();
        argument(&mut input, 8, (-7_i16) as u16);
        argument(&mut input, 10, 1);
        let result = question(&input, FRAME, limits(), &mut Observed::new()).unwrap();
        assert_eq!(result.native_return, PromptReturn::Full(0xabcd0001));
        assert_eq!(result.state.memory.word(FRAME - 50 + 8).unwrap(), 2);
        assert_eq!(result.state.memory.word(FRAME - 66 + 10).unwrap(), 1);
        assert_eq!(
            calls(&result, 0xa97c),
            vec![vec![W(137), L(0), L(u32::MAX)].as_slice()]
        );
        assert_eq!(result.state.random, input.random);
        assert!(!result.effects.iter().any(|e| matches!(
            e,
            PromptControlEffect::Character(_) | PromptControlEffect::Center(_)
        )));
    }
    #[test]
    fn tutorial_gate_and_accept_mark_source_range_without_writing_host_files() {
        let mut input = state();
        argument(&mut input, 8, 105);
        argument(&mut input, 10, 106);
        let result = first_tutorial(&input, FRAME, limits(), &mut Observed::new()).unwrap();
        assert_eq!(result.native_return, PromptReturn::Unchanged);
        assert!(result.effects.is_empty());
        input.memory.insert_region(0x439596, &1_u16.to_be_bytes());
        let result = first_tutorial(&input, FRAME, limits(), &mut Observed::new()).unwrap();
        assert_eq!(result.state.memory.byte(0x4042f2 + 105).unwrap(), 1);
        assert_eq!(result.state.memory.byte(0x4042f2 + 106).unwrap(), 1);
        assert_eq!(result.state.memory.word(0x439596).unwrap(), 1);
        assert_eq!(result.native_return, PromptReturn::Full(0xfeed1234));
        assert_eq!(result.state.random, input.random);
        assert_eq!(result.state.memory.long(0x4434a6).unwrap(), 0);
        assert!(
            result
                .effects
                .iter()
                .filter_map(|e| match e {
                    PromptControlEffect::Library { request, .. } => Some(request.entry),
                    _ => None,
                })
                .all(|entry| entry != 0x400050)
        );
    }
    #[test]
    fn choice_abort_clamps_caller_word_clears_flags_and_returns_full_zero() {
        let mut input = state();
        argument(&mut input, 8, 4);
        argument(&mut input, 10, 1);
        argument(&mut input, 12, 1);
        input.memory.insert_region(0x44273a, &[1]);
        input.memory.insert_region(0x438776, &1_u16.to_be_bytes());
        input.memory.insert_region(0x4470b8, &6_u16.to_be_bytes());
        let before = input.clone();
        let mut s = Observed::new();
        s.choice_abort = true;
        let result = get_choice(&input, FRAME, limits(), &mut s).unwrap();
        assert_eq!(result.state.memory.word(FRAME + 8).unwrap(), 1);
        assert_eq!(result.state.memory.word(0x4434b2).unwrap(), u16::MAX);
        assert_eq!(result.state.memory.word(0x438760).unwrap(), 0);
        assert_eq!(read(&result.state.memory, 0x4434e0, 16).unwrap(), [0; 16]);
        assert_eq!(result.state.memory.word(FRAME - 42 + 8).unwrap(), 105);
        assert_eq!(result.native_return, PromptReturn::Full(0));
        assert_eq!(input, before);
    }
}
