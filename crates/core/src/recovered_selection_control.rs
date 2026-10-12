//! Spell, creature and survivor selection from CODE 1.

use crate::RuleError;
use crate::recovered_audio::{self as audio, AudioContext, AudioEffect};
use crate::recovered_bodies::{self as bodies, BodyEffect, BodyState, ColorIconServices};
use crate::recovered_character_panels::{
    self as character, CharacterPanelEffect, CharacterPanelServices,
};
use crate::recovered_drawing::{
    self as drawing, BuildingWorld, DrawingContext, DrawingEffect, DrawingServices,
    PictureObservation, TierContext, TierServices,
};
use crate::recovered_feedback::{
    self as feedback, FeedbackArg, FeedbackEffect, FeedbackObservation, FeedbackRequest,
    FeedbackServices,
};
use crate::recovered_files::{
    self as files, FileBuffer, FileContext, FileEffect, FileObservation, FileReply, FileRequest,
};
use crate::recovered_geometry::BodyRect;
use crate::recovered_interface::{self as interface, ButtonInput, InterfaceEffect};
use crate::recovered_inventory_rules::{self as inventory, InventoryEffect};
use crate::recovered_load_control::{self as load, LoadEffect, LoadFlow};
use crate::recovered_prompt_control::{
    self as prompt, LibraryObservation, LibraryRequest, PromptControlEffect, PromptControlServices,
    PromptControlState, PromptLimits, PromptRegisterRequest, PromptReturn,
};
use crate::recovered_records::{self as records, RecordWrite};
use crate::recovered_rendering::RenderingMemory;
use crate::recovered_runtime::{self as runtime, RuntimeEffect, RuntimeResult};
use crate::recovered_shop_ui::{self as shop, NpcMenuContext, NpcMenuEffect, NpcNameServices};
use crate::recovered_spell_ui::{self as spell, SpellUiEffect, SpellUiServices, SpellUiState};
use crate::recovered_text::{
    self as text, NumberContext, NumericServices, PascalString, TextContext, TextEffect,
};
use crate::recovered_timing::{self as timing, TimingEffect, TimingServices};
use FeedbackArg::{Byte as B, Long as L, Pointer as P, Word as W};
use centurions_content::combat::MonsterRecord;

pub type SelectionControlState = PromptControlState;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectionControlLimits {
    pub prompt: PromptLimits,
    pub maximum_random_draws: u64,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SelectionRegisters {
    /// Inherited until castspell's caster-cycle branch assigns D3.
    pub d3_byte: Option<u8>,
    /// Inherited in beast's non-file branch, until navigation changes D4.
    pub d4_word: Option<i16>,
}
pub trait SelectionControlServices:
    PromptControlServices + SpellUiServices + NpcNameServices
{
}
#[derive(Clone, Debug, PartialEq)]
pub enum SelectionControlEffect {
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
    Audio(AudioEffect),
    Character(CharacterPanelEffect),
    Prompt(PromptControlEffect),
    Inventory(InventoryEffect),
    Load(LoadEffect),
    Spell(SpellUiEffect),
    Drawing(DrawingEffect),
    Interface(InterfaceEffect),
    Text(TextEffect),
    Timing(TimingEffect),
    File(FileEffect),
    NpcMenu(NpcMenuEffect),
}
#[derive(Clone, Debug, PartialEq)]
pub struct SelectionControlPlan {
    pub state: SelectionControlState,
    pub effects: Vec<SelectionControlEffect>,
    pub native_return: PromptReturn,
    pub flow: LoadFlow,
    pub qualification: SelectionControlQualification,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionControlQualification {
    SourcePortWithObservedBoundariesAndCapturedChildren,
}
pub const QUALIFICATION: SelectionControlQualification =
    SelectionControlQualification::SourcePortWithObservedBoundariesAndCapturedChildren;
struct Execution<'a, S: ?Sized> {
    plan: SelectionControlPlan,
    services: &'a mut S,
    limits: SelectionControlLimits,
    remaining: u32,
    draws: u64,
    d0: Option<u32>,
}
impl<S: SelectionControlServices + ?Sized> Execution<'_, S> {
    fn step(&mut self) -> Result<(), RuleError> {
        self.remaining = self.remaining.checked_sub(1).ok_or_else(|| {
            RuleError::Invalid("selection control processing budget exhausted".into())
        })?;
        Ok(())
    }
    fn b(&self, a: u32) -> Result<u8, RuleError> {
        self.plan.state.memory.byte(a)
    }
    fn w(&self, a: u32) -> Result<i16, RuleError> {
        Ok(self.plan.state.memory.word(a)? as i16)
    }
    fn l(&self, a: u32) -> Result<u32, RuleError> {
        self.plan.state.memory.long(a)
    }
    fn bytes(&self, a: u32, n: usize) -> Result<Vec<u8>, RuleError> {
        read(&self.plan.state.memory, a, n)
    }
    fn write(&mut self, a: u32, bytes: &[u8]) -> Result<(), RuleError> {
        self.bytes(a, bytes.len())?;
        self.plan.state.memory.insert_region(a, bytes);
        self.plan
            .effects
            .push(SelectionControlEffect::Write(RecordWrite {
                address: a,
                bytes: bytes.to_vec(),
            }));
        Ok(())
    }
    fn ww(&mut self, a: u32, v: i16) -> Result<(), RuleError> {
        self.write(a, &v.to_be_bytes())
    }
    fn lw(&mut self, a: u32, v: u32) -> Result<(), RuleError> {
        self.write(a, &v.to_be_bytes())
    }
    fn addw(&mut self, a: u32, v: i16) -> Result<(), RuleError> {
        self.ww(a, self.w(a)?.wrapping_add(v))
    }
    fn copy_longs(&mut self, mut from: u32, mut to: u32, count: usize) -> Result<(), RuleError> {
        for _ in 0..count {
            self.step()?;
            self.lw(to, self.l(from)?)?;
            from = from.wrapping_add(4);
            to = to.wrapping_add(4);
        }
        Ok(())
    }
    fn full(&mut self, d0: u32) {
        self.d0 = Some(d0);
        self.plan.native_return = PromptReturn::Full(d0);
    }
    fn observe(&mut self, request: FeedbackRequest) -> Result<u32, RuleError> {
        self.step()?;
        let observation =
            FeedbackServices::observe(self.services, &request, &self.plan.state.memory)?;
        let value = observation.value;
        self.d0 = Some(observation.d0);
        self.plan.state.memory = observation.memory.clone();
        self.plan
            .effects
            .push(SelectionControlEffect::Feedback(FeedbackEffect::Service {
                request,
                observation,
            }));
        Ok(value)
    }
    fn trap(&mut self, opcode: u16, arguments: Vec<FeedbackArg>) -> Result<u32, RuleError> {
        self.observe(FeedbackRequest::Toolbox { opcode, arguments })
    }
    fn flush_events(&mut self) -> Result<(), RuleError> {
        self.step()?;
        let request = PromptRegisterRequest {
            opcode: 0xa032,
            d0_input: Some(0xffff),
            a0_input: None,
            arguments: vec![],
        };
        let observation = self
            .services
            .observe_registers(&request, &self.plan.state.memory)?;
        self.d0 = Some(observation.d0);
        self.plan.state.memory = observation.memory.clone();
        self.plan
            .effects
            .push(SelectionControlEffect::RegisterToolbox {
                request,
                observation,
            });
        Ok(())
    }
    fn lib(
        &mut self,
        entry: u32,
        arguments: Vec<FeedbackArg>,
    ) -> Result<LibraryObservation, RuleError> {
        self.step()?;
        let request = LibraryRequest { entry, arguments };
        let observation = self.services.library(&request, &self.plan.state.memory)?;
        self.d0 = Some(observation.d0);
        self.plan.state.memory = observation.memory.clone();
        self.plan.effects.push(SelectionControlEffect::Library {
            request,
            observation: observation.clone(),
        });
        Ok(observation)
    }
    fn abs(&mut self, value: i16, stack: u32) -> Result<i16, RuleError> {
        self.ww(stack.wrapping_sub(2), value)?;
        Ok(self.lib(0x400038, vec![W(value as u16)])?.d0 as i16)
    }
    fn rand(&mut self, bound: i16, stack: u32) -> Result<i16, RuleError> {
        self.step()?;
        self.draws = self.draws.checked_sub(1).ok_or_else(|| {
            RuleError::Invalid("selection control random budget exhausted".into())
        })?;
        self.ww(stack.wrapping_sub(2), bound)?;
        let mut context =
            runtime::RuntimeContext::new(self.plan.state.random.clone(), self.remaining as usize);
        let child = runtime::rand(&mut context, bound)?;
        self.plan.state.random = context.random;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(SelectionControlEffect::Runtime),
        );
        match child.result {
            RuntimeResult::Word { value, native_d0 } => {
                self.d0 = Some(native_d0);
                Ok(value)
            }
            _ => Err(RuleError::Invalid(
                "unexpected selection Rand result".into(),
            )),
        }
    }
    fn sound(&mut self, id: i16, stack: u32) -> Result<(), RuleError> {
        self.ww(stack.wrapping_sub(2), id)?;
        let child = feedback::sound(&self.plan.state.memory, id, self.services)?;
        self.plan.state.memory = child.memory;
        self.d0 = child.native_return;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(SelectionControlEffect::Feedback),
        );
        Ok(())
    }
    fn quiet(&mut self, selector: i16, stack: u32) -> Result<(), RuleError> {
        self.ww(stack.wrapping_sub(2), selector)?;
        let last = self.w(0x439576)?;
        let mut context = AudioContext {
            last_channel: last,
            ..AudioContext::default()
        };
        if last >= 0 {
            let indices: Vec<i16> = if selector > 0 {
                vec![selector]
            } else {
                (1..last).collect()
            };
            if let Some(n) = indices.iter().copied().max() {
                context.channels.resize(n as usize + 1, 0);
            }
            for n in indices {
                context.channels[n as usize] = self.l(index(0x4436b6, n, 4))?;
            }
        }
        let observations =
            self.services
                .quiet_observations(selector, &context, &self.plan.state.memory)?;
        let child = audio::quiet(&mut context, selector, &observations)?;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(SelectionControlEffect::Audio));
        self.d0 = Some(0);
        Ok(())
    }
    fn runtime(&mut self, entry: u32) -> Result<(), RuleError> {
        self.step()?;
        let observations = CharacterPanelServices::runtime_observations(
            self.services,
            entry,
            &self.plan.state.memory,
        )?;
        let mut context =
            runtime::RuntimeContext::new(self.plan.state.random.clone(), self.remaining as usize);
        context.memory.bytes = self.plan.state.memory.bytes.clone();
        let child = match entry {
            0x13e13c => runtime::compact_heap(&mut context, &observations)?,
            0x13a498 => runtime::copy_in(&mut context, &observations)?,
            _ => return Err(RuleError::Invalid("unknown selection runtime entry".into())),
        };
        self.plan.state.memory.bytes = context.memory.bytes;
        self.plan.state.random = context.random;
        self.d0 = if let RuntimeResult::Long(value) = child.result {
            Some(value)
        } else {
            None
        };
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(SelectionControlEffect::Runtime),
        );
        Ok(())
    }
    fn out(&mut self, stack: u32) -> Result<(), RuleError> {
        let child = character::copy_out(
            &self.plan.state.memory,
            stack.wrapping_sub(8),
            self.remaining,
            self.services,
        )?;
        self.plan.state.memory = child.memory;
        self.d0 = Some(child.native_return);
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(SelectionControlEffect::Character),
        );
        Ok(())
    }
    fn prompt(&mut self, entry: u32, frame: u32) -> Result<(), RuleError> {
        self.step()?;
        let limits = PromptLimits {
            maximum_steps: self.remaining,
            ..self.limits.prompt
        };
        let child = match entry {
            0x12382a => prompt::first_tutorial(&self.plan.state, frame, limits, self.services)?,
            0x133d46 => prompt::submit_music(&self.plan.state, frame, limits, self.services)?,
            _ => return Err(RuleError::Invalid("unknown selection prompt entry".into())),
        };
        self.plan.state = child.state;
        self.plan.flow = child.flow;
        self.d0 = if let PromptReturn::Full(v) = child.native_return {
            Some(v)
        } else {
            None
        };
        if self.plan.flow != LoadFlow::Returned {
            self.plan.native_return = PromptReturn::NoReturn;
        }
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(SelectionControlEffect::Prompt),
        );
        Ok(())
    }
    fn first_text(&mut self, stack: u32, packed: u32) -> Result<(), RuleError> {
        self.lw(stack.wrapping_sub(4), packed)?;
        self.prompt(0x12382a, stack.wrapping_sub(12))
    }
    fn music(&mut self, stack: u32) -> Result<(), RuleError> {
        self.ww(stack.wrapping_sub(2), 0)?;
        self.prompt(0x133d46, stack.wrapping_sub(10))
    }
    fn warn(&mut self, stack: u32, id: i16) -> Result<(), RuleError> {
        self.ww(stack.wrapping_sub(2), id)?;
        let child = feedback::warn(
            &self.plan.state.memory,
            stack.wrapping_sub(10),
            id,
            self.limits.prompt.maximum_polls,
            self.services,
        )?;
        self.plan.state.memory = child.memory;
        self.d0 = child.native_return;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(SelectionControlEffect::Feedback),
        );
        Ok(())
    }
    fn scratch(&mut self, stack: u32, id: i16) -> Result<(), RuleError> {
        self.ww(stack.wrapping_sub(2), id)?;
        let child = load::scratch(
            &self.plan.state.memory,
            stack.wrapping_sub(10),
            self.services,
        )?;
        self.plan.state.memory = child.memory;
        self.plan.flow = child.flow;
        self.d0 = child.native_return;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(SelectionControlEffect::Load));
        if self.plan.flow != LoadFlow::Returned {
            self.plan.native_return = PromptReturn::NoReturn;
        }
        Ok(())
    }
    fn delay(&mut self, stack: u32, duration: i16) -> Result<(), RuleError> {
        self.ww(stack.wrapping_sub(2), duration)?;
        let default = if duration == 0 { self.w(0x439568)? } else { 0 };
        let polls = self.limits.prompt.maximum_polls;
        let child = timing::delay(
            duration,
            default,
            &mut Clock {
                execution: self,
                remaining: polls,
            },
        )?;
        self.d0 = Some(child.native_result);
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(SelectionControlEffect::Timing),
        );
        Ok(())
    }
    fn get_item(&mut self, dialog: u32, item: i16, rect: u32) -> Result<(), RuleError> {
        self.trap(
            0xa98d,
            vec![L(dialog), W(item as u16), P(0x4434b0), P(0x4434dc), P(rect)],
        )?;
        Ok(())
    }
    fn numeric(&mut self, value: i16, stack: u32) -> Result<(), RuleError> {
        self.ww(stack.wrapping_sub(2), value)?;
        self.lib(0x400168, vec![L(i32::from(value) as u32), P(0x479ed8)])?;
        let scratch = pascal(&self.plan.state.memory, 0x479ed8)?;
        let _child = text::string(
            &mut TextContext::default(),
            value,
            &mut FixedNumber(scratch),
        )?;
        self.trap(0xa884, vec![P(0x479ed8)])?;
        Ok(())
    }
    fn di_str(&mut self, stack: u32, item: i16, pointer: u32) -> Result<(), RuleError> {
        self.lw(stack.wrapping_sub(4), pointer)?;
        self.ww(stack.wrapping_sub(6), item)?;
        let frame = stack.wrapping_sub(14);
        self.get_item(self.l(0x4394b8)?, self.w(frame.wrapping_add(8))?, 0x4434ba)?;
        let pointer = self.l(frame.wrapping_add(10))?;
        let _child = text::dialog_string(item, &pascal(&self.plan.state.memory, pointer)?);
        self.trap(0xa98f, vec![L(self.l(0x4434dc)?), P(pointer)])?;
        self.d0 = Some(0);
        Ok(())
    }
    fn dialog_number(&mut self, stack: u32, item: i16, value: i16) -> Result<(), RuleError> {
        self.ww(stack.wrapping_sub(2), value)?;
        self.ww(stack.wrapping_sub(4), item)?;
        let frame = stack.wrapping_sub(12);
        self.get_item(self.l(0x4394b8)?, self.w(frame.wrapping_add(8))?, 0x4434ba)?;
        let value = self.w(frame.wrapping_add(10))?;
        self.lib(0x400168, vec![L(i32::from(value) as u32), P(0x479ed8)])?;
        let _child = text::dialog_number(
            &mut TextContext::default(),
            item,
            value,
            &mut FixedNumber(pascal(&self.plan.state.memory, 0x479ed8)?),
        )?;
        self.trap(0xa98f, vec![L(self.l(0x4434dc)?), P(0x479ed8)])?;
        self.d0 = Some(0);
        Ok(())
    }
    fn filename(&mut self, stack: u32, name_pointer: u32) -> Result<(), RuleError> {
        self.lw(stack.wrapping_sub(4), name_pointer)?;
        let mut context = FileContext {
            scenario_directory: c_string(&self.plan.state.memory, 0x44260c)?,
            variant: self.b(0x44274c)?,
            filename: self.bytes(0x4389a4, self.limits.prompt.filename_capacity)?,
            ..FileContext::default()
        };
        let converted = self
            .services
            .filename_number(context.variant, &self.plan.state.memory)?;
        let observations = [FileObservation {
            request: FileRequest::NumberToString(i32::from(context.variant)),
            reply: FileReply::Pascal(converted),
        }];
        let name = c_string(&self.plan.state.memory, name_pointer)?;
        let child = files::get_filename(&mut context, &name, &observations)?;
        for effect in child.effects {
            match effect {
                FileEffect::Write {
                    destination: FileBuffer::Filename,
                    offset,
                    bytes,
                } => self.write(0x4389a4_u32.wrapping_add(offset as u32), &bytes)?,
                other => self.plan.effects.push(SelectionControlEffect::File(other)),
            }
        }
        self.d0 = None;
        Ok(())
    }
    fn pict(&mut self, stack: u32, id: i16, rect: u32) -> Result<(), RuleError> {
        self.lw(stack.wrapping_sub(4), self.l(rect.wrapping_add(4))?)?;
        self.lw(stack.wrapping_sub(8), self.l(rect)?)?;
        self.ww(stack.wrapping_sub(10), id)?;
        let handle = self.trap(0xa9bc, vec![W(id as u16)])?;
        self.lw(stack.wrapping_sub(22), handle)?;
        let _child = drawing::pict(
            id,
            self.plan.state.memory.rect(stack.wrapping_sub(8))?,
            &mut FixedDrawing(handle),
        )?;
        self.trap(
            0xa8f6,
            vec![L(self.l(stack.wrapping_sub(22))?), P(stack.wrapping_sub(8))],
        )?;
        Ok(())
    }
    fn icon(&mut self, stack: u32, id: i16, rect: u32) -> Result<(), RuleError> {
        self.lw(stack.wrapping_sub(4), self.l(rect.wrapping_add(4))?)?;
        self.lw(stack.wrapping_sub(8), self.l(rect)?)?;
        self.ww(stack.wrapping_sub(10), id)?;
        self.lw(0x439598, 0)?;
        let handle = self.trap(0xaa1e, vec![W(id as u16)])?;
        self.lw(0x439598, handle)?;
        let rect_pointer = stack.wrapping_sub(8);
        let rect = self.plan.state.memory.rect(rect_pointer)?;
        let _child = drawing::ploticon3(
            &mut drawing_context(rect),
            id,
            rect,
            &mut FixedDrawing(handle),
        )?;
        if self.l(0x439598)? != 0 {
            self.trap(0xaa1f, vec![P(rect_pointer), L(self.l(0x439598)?)])?;
            self.trap(0xaa25, vec![L(self.l(0x439598)?)])?;
        }
        self.d0 = Some(0);
        Ok(())
    }
    fn portrait(
        &mut self,
        stack: u32,
        flags: u32,
        race: i16,
        icon: i16,
        rect: u32,
    ) -> Result<(), RuleError> {
        self.lw(stack.wrapping_sub(4), flags)?;
        self.ww(stack.wrapping_sub(6), race)?;
        self.lw(stack.wrapping_sub(10), self.l(rect.wrapping_add(4))?)?;
        self.lw(stack.wrapping_sub(14), self.l(rect)?)?;
        self.ww(stack.wrapping_sub(16), icon)?;
        let child = character::plot_portrait(
            &self.plan.state.memory,
            stack.wrapping_sub(24),
            self.remaining,
            self.services,
        )?;
        self.plan.state.memory = child.memory;
        self.d0 = Some(child.native_return);
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(SelectionControlEffect::Character),
        );
        Ok(())
    }
    fn press(
        &mut self,
        stack: u32,
        kind: i16,
        phase: i16,
        sound: i16,
        rect: u32,
    ) -> Result<(), RuleError> {
        self.lw(stack.wrapping_sub(4), self.l(rect.wrapping_add(4))?)?;
        self.lw(stack.wrapping_sub(8), self.l(rect)?)?;
        self.ww(stack.wrapping_sub(10), sound)?;
        self.lw(
            stack.wrapping_sub(14),
            (u32::from(kind as u16) << 16) | u32::from(phase as u16),
        )?;
        let child = feedback::pressbutton(
            &self.plan.state.memory,
            stack.wrapping_sub(22),
            kind,
            phase,
            sound,
            self.services,
        )?;
        self.plan.state.memory = child.memory;
        self.d0 = child.native_return;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(SelectionControlEffect::Feedback),
        );
        Ok(())
    }
    fn button(&mut self, stack: u32, pressed: bool, pushed: bool) -> Result<(), RuleError> {
        let switch = if pushed {
            self.ww(stack.wrapping_sub(2), 0)?;
            0
        } else {
            self.b(stack.wrapping_add(1))? as i8
        };
        let input = ButtonInput {
            rect: self.plan.state.memory.rect(0x44292c)?,
            target_port: if switch == 0 { 0 } else { self.l(0x43878c)? },
            dark_pattern: self.l(0x4358de)?,
            light_pattern: self.l(0x4358e2)?,
        };
        let child = if pressed {
            interface::downbutton(input, switch, self.services)?
        } else {
            interface::upbutton(input, switch, self.services)?
        };
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(SelectionControlEffect::Interface),
        );
        self.d0 = Some(0);
        Ok(())
    }
    fn tier(&mut self, stack: u32, value: i16) -> Result<(), RuleError> {
        self.ww(stack.wrapping_sub(2), value)?;
        let mut tier = TierContext {
            dialog: self.l(0x4394d4)?,
            selected_tier: self.w(0x437a92)?,
            item_type: 0,
            item_handle: 0,
            rect: BodyRect {
                top: 0,
                left: 0,
                bottom: 0,
                right: 0,
            },
        };
        let mut context = drawing_context(tier.rect);
        let child = drawing::updatetier(
            &mut context,
            &mut tier,
            value,
            &mut TierBridge(self.services),
        )?;
        for effect in child.effects {
            if let DrawingEffect::Write(ref write) = effect {
                self.write(write.address, &write.bytes)?;
            } else {
                self.plan
                    .effects
                    .push(SelectionControlEffect::Drawing(effect));
            }
        }
        self.d0 = Some(0);
        Ok(())
    }
    fn cast(&mut self, stack: u32, member: i16) -> Result<bool, RuleError> {
        self.ww(stack.wrapping_sub(2), 1)?;
        self.ww(stack.wrapping_sub(4), member)?;
        let incoming = (self.d0.unwrap_or(0) & 0xffff0000) | u32::from(member as u16);
        let child = inventory::cancast(
            &self.plan.state.memory,
            stack.wrapping_sub(12),
            incoming,
            self.limits.prompt.maximum_polls,
            self.services,
        )?;
        let allowed = child.native_return as u16 != 0;
        self.plan.state.memory = child.memory;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(SelectionControlEffect::Inventory),
        );
        self.d0 = if member < 6 || self.d0.is_some() {
            Some(child.native_return)
        } else {
            None
        };
        Ok(allowed)
    }
    fn load_spell(
        &mut self,
        stack: u32,
        school: i16,
        tier: i16,
        slot: i16,
    ) -> Result<(), RuleError> {
        self.ww(stack.wrapping_sub(2), slot)?;
        self.ww(stack.wrapping_sub(4), tier)?;
        self.ww(stack.wrapping_sub(6), school)?;
        let coordinates = [school as i8, tier as i8, slot as i8];
        let source = 0x443788_u32
            .wrapping_add(u32::from(i16::from(coordinates[0]) as u16) * 2842)
            .wrapping_add(u32::from(i16::from(coordinates[1]) as u16) * 406)
            .wrapping_add(
                u32::from(i16::from(coordinates[2]) as u16) * records::SPELL_BYTES as u32,
            );
        let bytes = self.bytes(source, records::SPELL_BYTES)?;
        let child = records::loadspell(&bytes, 0, 0, 0)?;
        for write in child.writes {
            self.write(write.address, &write.bytes)?;
        }
        self.d0 = Some(0);
        Ok(())
    }
    fn spell_ui(&mut self, entry: u32) -> Result<(), RuleError> {
        self.step()?;
        let state = SpellUiState {
            memory: self.plan.state.memory.clone(),
            drawing: drawing_context(BodyRect {
                top: 0,
                left: 0,
                bottom: 0,
                right: 0,
            }),
        };
        let child = match entry {
            0x10b6ba => spell::showspelllist(&state, self.services)?,
            0x15947c => spell::spellinfoupdate(&state, self.services)?,
            0x159192 => spell::showspellinfo(&state, self.services)?,
            0x162084 => spell::updatespell(&state, 1, self.services)?,
            _ => return Err(RuleError::Invalid("unknown selection spell child".into())),
        };
        self.plan.state.memory = child.state.memory;
        self.d0 = child.native_return.map(|v| v as u32);
        self.plan
            .effects
            .extend(child.effects.into_iter().map(SelectionControlEffect::Spell));
        Ok(())
    }
    fn npc_menu(&mut self) -> Result<(), RuleError> {
        let count = self.w(0x44257c)?;
        let length = usize::from(count.max(0) as u16)
            .checked_mul(288)
            .ok_or_else(|| RuleError::Invalid("NPC menu projection length overflow".into()))?;
        let mut context = NpcMenuContext {
            menu: self.l(0x438722)?,
            count,
            record_bytes: self.bytes(0x440edc, length)?,
            placeholder: pascal(&self.plan.state.memory, 0x404044)?,
            scratch: pascal(&self.plan.state.memory, 0x479ed8)?,
        };
        let child = shop::updatenpcmenu(&mut context, self.remaining as usize, self.services)?;
        for effect in child.effects {
            if let NpcMenuEffect::Write(ref write) = effect {
                self.write(write.address, &write.bytes)?;
            } else {
                self.plan
                    .effects
                    .push(SelectionControlEffect::NpcMenu(effect));
            }
        }
        self.d0 = Some(child.native_return as u32);
        Ok(())
    }
    fn number(&mut self) -> Result<(), RuleError> {
        if self.w(0x438760)? != 0 {
            return Ok(());
        }
        let inputs = NumberContext {
            suppressed: 0,
            origin_x: self.w(0x40856c)?,
            origin_y: self.w(0x40856e)?,
            line: self.w(0x439582)?,
            font: self.w(0x402f3c)?,
            enemy_total: self.w(0x4435c4)?,
            enemy_removed: self.w(0x44374c)?,
            label: pascal(&self.plan.state.memory, 0x401b6a)?,
        };
        let mut context = TextContext::default();
        let child = text::number(&mut context, &inputs, self.services)?;
        for effect in child {
            match effect {
                TextEffect::TextSize(v) => {
                    self.trap(0xa88a, vec![W(v as u16)])?;
                }
                TextEffect::ForeColor(v) => {
                    self.trap(0xa862, vec![L(v as u32)])?;
                }
                TextEffect::TextFace(v) => {
                    self.trap(0xa889, vec![W(u16::from(v))])?;
                }
                TextEffect::TextFont(v) => {
                    self.trap(0xa887, vec![W(v as u16)])?;
                }
                TextEffect::MoveTo { x, y } => {
                    self.trap(0xa893, vec![W(x as u16), W(y as u16)])?;
                }
                TextEffect::NumberToString(v) => {
                    self.plan
                        .effects
                        .push(SelectionControlEffect::Text(TextEffect::NumberToString(v)));
                    self.write(0x479ed8, context.scratch.raw())?;
                }
                TextEffect::DrawString(value) => {
                    self.plan
                        .effects
                        .push(SelectionControlEffect::Text(TextEffect::DrawString(value)));
                }
                _ => return Err(RuleError::Invalid("unexpected number child effect".into())),
            }
        }
        self.d0 = None;
        Ok(())
    }
    fn icon2(&mut self, id: i16) -> Result<(), RuleError> {
        let rect = self.plan.state.memory.rect(0x4471f2)?;
        let child = drawing::ploticon2(&mut drawing_context(rect), id, &mut FixedDrawing(1))?;
        let mut actual = 0;
        for effect in child.effects {
            match effect {
                DrawingEffect::Write(write) if write.address == 0x439598 => {
                    self.lw(write.address, actual)?
                }
                DrawingEffect::Write(write) => self.write(write.address, &write.bytes)?,
                DrawingEffect::GetColorIcon(resource) => {
                    actual = self.trap(0xaa1e, vec![W(resource as u16)])?;
                }
                DrawingEffect::PlotColorIcon { .. } if self.l(0x439598)? != 0 => {
                    self.trap(0xaa1f, vec![P(0x4471f2), L(self.l(0x439598)?)])?;
                }
                DrawingEffect::DisposeColorIcon(_) if self.l(0x439598)? != 0 => {
                    self.trap(0xaa25, vec![L(self.l(0x439598)?)])?;
                }
                DrawingEffect::PlotColorIcon { .. } | DrawingEffect::DisposeColorIcon(_) => {}
                _ => {
                    return Err(RuleError::Invalid(
                        "unexpected selection ploticon2 effect".into(),
                    ));
                }
            }
        }
        self.d0 = Some(0);
        Ok(())
    }
    fn monster(&mut self, stack: u32, enemy: i16, rect: u32) -> Result<(), RuleError> {
        self.lw(stack.wrapping_sub(4), self.l(rect.wrapping_add(4))?)?;
        self.lw(stack.wrapping_sub(8), self.l(rect)?)?;
        self.ww(stack.wrapping_sub(10), enemy)?;
        let frame = stack.wrapping_sub(18);
        let record = self.bytes(npc(enemy), 288)?;
        let view = monster_view(enemy as u16, record);
        let child = bodies::monsterupdate(
            &view,
            enemy,
            self.plan.state.memory.rect(frame.wrapping_add(10))?,
            &IconMarker,
        )?;
        for effect in child {
            match effect {
                BodyEffect::ForeColor(v) => {
                    self.trap(0xa862, vec![L(v as u32)])?;
                }
                BodyEffect::MovePen {
                    horizontal,
                    vertical,
                } => {
                    self.trap(0xa893, vec![W(horizontal as u16), W(vertical as u16)])?;
                }
                BodyEffect::DrawNumber(v) => self.numeric(v, frame.wrapping_sub(20))?,
                BodyEffect::GlobalRect(write) => {
                    let offset = match write.field {
                        crate::recovered_geometry::RectField::Top => 0,
                        crate::recovered_geometry::RectField::Left => 2,
                        crate::recovered_geometry::RectField::Bottom => 4,
                        crate::recovered_geometry::RectField::Right => 6,
                    };
                    self.ww(0x4471f2 + offset, write.value)?;
                }
                BodyEffect::PlotIcon2(id) => self.icon2(id)?,
                BodyEffect::IconHandle(0) => self.lw(0x439598, 0)?,
                BodyEffect::GetColorIcon { resource, .. } => {
                    let handle = self.trap(0xaa1e, vec![W(resource as u16)])?;
                    self.lw(0x439598, handle)?;
                }
                BodyEffect::IconHandle(_) => {}
                BodyEffect::InsetRect { rect, .. } => {
                    self.write(frame.wrapping_add(10), &rect_bytes(rect))?
                }
                BodyEffect::PlotColorIcon { .. } if self.l(0x439598)? != 0 => {
                    self.trap(
                        0xaa1f,
                        vec![P(frame.wrapping_add(10)), L(self.l(0x439598)?)],
                    )?;
                }
                BodyEffect::DisposeColorIcon(_) if self.l(0x439598)? != 0 => {
                    self.trap(0xaa25, vec![L(self.l(0x439598)?)])?;
                }
                BodyEffect::PlotColorIcon { .. } | BodyEffect::DisposeColorIcon(_) => {}
                _ => {
                    return Err(RuleError::Invalid(
                        "unexpected monsterupdate child effect".into(),
                    ));
                }
            }
        }
        self.d0 = None;
        Ok(())
    }
}
fn execution<'a, S: ?Sized>(
    state: &SelectionControlState,
    limits: SelectionControlLimits,
    services: &'a mut S,
) -> Execution<'a, S> {
    Execution {
        plan: SelectionControlPlan {
            state: state.clone(),
            effects: vec![],
            native_return: PromptReturn::Unchanged,
            flow: LoadFlow::Returned,
            qualification: QUALIFICATION,
        },
        services,
        limits,
        remaining: limits.prompt.maximum_steps,
        draws: limits.maximum_random_draws,
        d0: None,
    }
}

pub fn existing_child_plan(
    state: &SelectionControlState,
    entry: u32,
    limits: SelectionControlLimits,
    services: &mut (impl SelectionControlServices + ?Sized),
) -> Result<SelectionControlPlan, RuleError> {
    let mut e = execution(state, limits, services);
    match entry {
        0x1046e0 => e.npc_menu()?,
        0x10b6ba | 0x159192 | 0x15947c | 0x162084 => e.spell_ui(entry)?,
        _ => {
            return Err(RuleError::Unsupported {
                action: "selection_control.existing_child",
                dependency: "the reached entry has no existing selection child adapter",
            });
        }
    }
    // The typed presentation children do not report complete native CPU returns.
    e.plan.native_return = PromptReturn::Unchanged;
    Ok(e.plan)
}
fn read(memory: &RenderingMemory, a: u32, n: usize) -> Result<Vec<u8>, RuleError> {
    (0..n)
        .map(|i| memory.byte(a.wrapping_add(i as u32)))
        .collect()
}
fn index(a: u32, n: i16, stride: i32) -> u32 {
    a.wrapping_add((i32::from(n) * stride) as u32)
}
fn warrior(n: i16) -> u32 {
    0x448aa0_u32.wrapping_add(u32::from(n as u16) * 3138)
}
fn npc(n: i16) -> u32 {
    0x439d3c_u32.wrapping_add(u32::from(n as u16) * 288)
}
fn pascal(memory: &RenderingMemory, a: u32) -> Result<PascalString, RuleError> {
    PascalString::from_raw(&read(memory, a, usize::from(memory.byte(a)?) + 1)?)
}
fn c_string(memory: &RenderingMemory, a: u32) -> Result<Vec<u8>, RuleError> {
    let mut bytes = vec![];
    for i in 0..=memory.bytes.len() {
        let b = memory.byte(a.wrapping_add(i as u32))?;
        bytes.push(b);
        if b == 0 {
            return Ok(bytes);
        }
    }
    Err(RuleError::Invalid(
        "selection C string lacks a reached terminator".into(),
    ))
}
fn half(value: i16) -> i16 {
    (value.wrapping_add((value as u16 >> 15) as i16)) >> 1
}
fn rect_bytes(rect: BodyRect) -> Vec<u8> {
    [rect.top, rect.left, rect.bottom, rect.right]
        .into_iter()
        .flat_map(i16::to_be_bytes)
        .collect()
}
fn monster_view(enemy: u16, bytes: Vec<u8>) -> BodyState {
    let mut records = vec![MonsterRecord { bytes: vec![] }; usize::from(enemy) + 1];
    records[usize::from(enemy)] = MonsterRecord { bytes };
    BodyState {
        combat: crate::combat::Combat {
            battle: 0,
            map: crate::combat::TacticalMap {
                tiles: vec![],
                edge_word: 0,
                shift: (0, 0),
            },
            squad: crate::combat::SquadPlacement {
                positions: vec![],
                under_tiles: vec![],
            },
            enemies: crate::combat::EnemySetup {
                records,
                positions: vec![],
            },
            order: vec![],
            cursor: 0,
            round: 0,
            round_scripts: Default::default(),
            death_queue: Default::default(),
            spell_fields: vec![],
            field_used: vec![],
            attack_packet: None,
            attack_identity: None,
            weapon_work: Default::default(),
            unavailable: 0,
        },
        native_origin: (0, 0),
        enemy_count: 0,
        killed_word: 0,
    }
}
struct IconMarker;
impl ColorIconServices for IconMarker {
    fn get_color_icon(&self, _: i16) -> Result<u32, RuleError> {
        Ok(1)
    }
}
fn drawing_context(rect: BodyRect) -> DrawingContext {
    DrawingContext {
        rect,
        banner_rect: rect,
        source_rect: rect,
        source_row: 0,
        source_column: 0,
        icon_handle: 0,
        item_work: [0; centurions_content::ITEM_BYTES],
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
struct FixedNumber(PascalString);
impl NumericServices for FixedNumber {
    fn number_to_string(&mut self, _: i32) -> Result<PascalString, RuleError> {
        Ok(self.0.clone())
    }
    fn string_to_number(&mut self, _: &PascalString) -> Result<i32, RuleError> {
        Err(RuleError::Invalid("unused selection numeric parse".into()))
    }
}
struct FixedDrawing(u32);
impl DrawingServices for FixedDrawing {
    fn color_icon(&mut self, _: i16) -> Result<u32, RuleError> {
        Ok(self.0)
    }
    fn picture(&mut self, _: i16) -> Result<PictureObservation, RuleError> {
        Ok(PictureObservation {
            handle: self.0,
            bounds: None,
        })
    }
    fn indexed_string(&mut self, _: i16, _: i16) -> Result<PascalString, RuleError> {
        Err(RuleError::Invalid("unused selection indexed string".into()))
    }
    fn current_world(&mut self) -> Result<(u32, u32), RuleError> {
        Err(RuleError::Invalid("unused selection world".into()))
    }
    fn new_world(&mut self, _: i16, _: BodyRect) -> Result<(i16, BuildingWorld), RuleError> {
        Err(RuleError::Invalid("unused selection allocation".into()))
    }
}
struct TierBridge<'a, S: ?Sized>(&'a mut S);
impl<S: SpellUiServices + ?Sized> TierServices for TierBridge<'_, S> {
    fn dialog_item(&mut self, dialog: u32, item: i16) -> Result<(i16, u32, BodyRect), RuleError> {
        let value = SpellUiServices::dialog_item(self.0, dialog, item)?;
        Ok((value.item_type, value.handle, value.rect))
    }
}
impl<S: SpellUiServices + ?Sized> DrawingServices for TierBridge<'_, S> {
    fn color_icon(&mut self, id: i16) -> Result<u32, RuleError> {
        DrawingServices::color_icon(self.0, id)
    }
    fn picture(&mut self, id: i16) -> Result<PictureObservation, RuleError> {
        DrawingServices::picture(self.0, id)
    }
    fn indexed_string(&mut self, resource: i16, item: i16) -> Result<PascalString, RuleError> {
        DrawingServices::indexed_string(self.0, resource, item)
    }
    fn current_world(&mut self) -> Result<(u32, u32), RuleError> {
        DrawingServices::current_world(self.0)
    }
    fn new_world(
        &mut self,
        depth: i16,
        bounds: BodyRect,
    ) -> Result<(i16, BuildingWorld), RuleError> {
        DrawingServices::new_world(self.0, depth, bounds)
    }
}
struct Clock<'a, 'b, S: ?Sized> {
    execution: &'a mut Execution<'b, S>,
    remaining: usize,
}
impl<S: SelectionControlServices + ?Sized> TimingServices for Clock<'_, '_, S> {
    fn tick_count(&mut self) -> Result<u32, RuleError> {
        self.remaining = self
            .remaining
            .checked_sub(1)
            .ok_or_else(|| RuleError::Invalid("selection delay polling budget exhausted".into()))?;
        self.execution.observe(FeedbackRequest::TickCount)
    }
    fn button(&mut self) -> Result<bool, RuleError> {
        Ok(self.execution.observe(FeedbackRequest::Button)? as u8 != 0)
    }
}

/// combaterrors, 0x117356. The argument is tested as an unsigned WORD.
pub fn combat_errors(
    state: &SelectionControlState,
    frame: u32,
    limits: SelectionControlLimits,
    services: &mut (impl SelectionControlServices + ?Sized),
) -> Result<SelectionControlPlan, RuleError> {
    let mut e = execution(state, limits, services);
    let stack = frame;
    e.trap(0xa873, vec![L(e.l(0x43878c)?)])?;
    e.trap(0xa88a, vec![W(18)])?;
    e.trap(0xa889, vec![W(1)])?;
    let x = e.w(0x40856c)?.wrapping_add(420);
    let y = e
        .w(0x40856e)?
        .wrapping_add(440)
        .wrapping_sub(e.w(0x439582)?.wrapping_shl(5));
    e.trap(0xa893, vec![W(x as u16), W(y as u16)])?;
    e.trap(0xa862, vec![L(69)])?;
    e.pict(stack, 156, 0x44370a)?;
    if let Some(pointer) = match e.w(frame.wrapping_add(8))? as u16 {
        1 => Some(0x403daa),
        2 => Some(0x403d98),
        3 => Some(0x403d86),
        4 => Some(0x403d70),
        5 => Some(0x403d5c),
        6 => Some(0x403d4a),
        _ => None,
    } {
        e.trap(0xa884, vec![P(pointer)])?;
    }
    e.sound(-143, stack)?;
    e.pict(
        stack.wrapping_sub(2),
        e.w(0x438760)?.wrapping_mul(18).wrapping_add(156),
        0x44370a,
    )?;
    if e.w(0x438760)? == 0 {
        e.number()?;
    } else {
        e.ww(stack.wrapping_sub(2), 1)?;
        e.spell_ui(0x162084)?;
    }
    e.button(stack, false, false)?;
    if let Some(value) = match e.w(frame.wrapping_add(8))? as u16 {
        1 => Some(90),
        2 => Some(86),
        3 => Some(91),
        4 => Some(88),
        5 => Some(87),
        6 => Some(89),
        _ => None,
    } {
        e.first_text(stack, (value << 16) | value)?;
        if e.plan.flow != LoadFlow::Returned {
            return Ok(e.plan);
        }
    }
    e.full(0);
    Ok(e.plan)
}

fn list_address(frame: u32, n: i16) -> u32 {
    index(frame.wrapping_sub(220), n, 2)
}
fn caster<S: SelectionControlServices + ?Sized>(e: &Execution<'_, S>) -> Result<i16, RuleError> {
    Ok(i16::from(e.b(0x44273f)? as i8))
}
fn school<S: SelectionControlServices + ?Sized>(e: &Execution<'_, S>) -> Result<i16, RuleError> {
    Ok(e.w(warrior(caster(e)?) + 0x36)?.wrapping_sub(1))
}
fn spell_available<S: SelectionControlServices + ?Sized>(
    e: &Execution<'_, S>,
) -> Result<bool, RuleError> {
    Ok((e.w(0x438776)? != 0 && e.b(0x443763)? != 0)
        || (e.w(0x438776)? == 0 && e.b(0x443764)? != 0)
        || e.w(0x447198)? != 0)
}
fn spell_cost<S: SelectionControlServices + ?Sized>(
    e: &Execution<'_, S>,
    quantity: i16,
) -> Result<i16, RuleError> {
    Ok(i16::from(e.b(0x44376c)?).wrapping_mul(quantity))
}
fn spell_highlight<S: SelectionControlServices + ?Sized>(
    e: &mut Execution<'_, S>,
    frame: u32,
) -> Result<(), RuleError> {
    e.get_item(e.l(0x4394d4)?, 17, frame.wrapping_sub(14))?;
    e.addw(
        frame.wrapping_sub(14),
        e.w(0x4435ca)?.wrapping_mul(24).wrapping_add(2),
    )?;
    e.addw(frame.wrapping_sub(12), 2)?;
    e.addw(frame.wrapping_sub(8), -2)?;
    e.ww(
        frame.wrapping_sub(10),
        e.w(frame.wrapping_sub(14))?.wrapping_add(9),
    )?;
    Ok(())
}
fn spell_name<S: SelectionControlServices + ?Sized>(
    e: &mut Execution<'_, S>,
    stack: u32,
) -> Result<(), RuleError> {
    e.lib(0x400188, vec![P(warrior(caster(e)?) + 0xa21)])?;
    e.di_str(stack.wrapping_sub(4), 43, warrior(caster(e)?) + 0xa21)?;
    e.lib(0x400180, vec![P(warrior(caster(e)?) + 0xa21)])?;
    e.dialog_number(stack.wrapping_sub(14), 44, e.w(warrior(caster(e)?) + 0x58)?)?;
    Ok(())
}
fn spell_description<S: SelectionControlServices + ?Sized>(
    e: &mut Execution<'_, S>,
    secondary: u32,
    record: i16,
) -> Result<(), RuleError> {
    e.ww(0x4434b6, record)?;
    if e.w(0x4434b6)? < 0 {
        e.ww(0x4434b6, 0)?;
    }
    let file = e.lib(0x400070, vec![P(0x403f04), P(0x403f08)])?.value;
    e.lw(0x4434a6, file)?;
    e.lib(
        0x400078,
        vec![
            W(0),
            L((i32::from(e.w(0x4434b6)?) as u32).wrapping_shl(8)),
            L(e.l(0x4434a6)?),
        ],
    )?;
    e.lib(0x400048, vec![L(e.l(0x4434a6)?), L(1), L(256), P(0x479ed8)])?;
    e.lib(0x400068, vec![L(e.l(0x4434a6)?)])?;
    e.get_item(secondary, 2, 0x4434ba)?;
    Ok(())
}
fn shortcut_address<S: SelectionControlServices + ?Sized>(
    e: &Execution<'_, S>,
    offset: u32,
) -> Result<u32, RuleError> {
    Ok(0x43788a_u32
        .wrapping_add(u32::from(caster(e)? as u16) * 80)
        .wrapping_add((i32::from(i16::from(e.b(0x437a6b)?).wrapping_sub(1)) * 8) as u32)
        .wrapping_add(offset))
}
#[derive(Clone, Copy)]
enum CastPhase {
    Redraw,
    Prepare,
    Poll,
    Dispatch,
    Key,
    Class,
    ClassCleanup,
    Slot,
    Quantity,
    Clamp,
    ApplyQuantity,
    Cleanup,
}

/// castspell, 0x10b89e. Dialog and keyboard paths share the same source state machine.
pub fn cast_spell(
    state: &SelectionControlState,
    frame: u32,
    registers: SelectionRegisters,
    limits: SelectionControlLimits,
    services: &mut (impl SelectionControlServices + ?Sized),
) -> Result<SelectionControlPlan, RuleError> {
    let mut e = execution(state, limits, services);
    let stack = frame.wrapping_sub(60);
    let mut secondary = 0_u32;
    let mut gray = false;
    let mut direction = 1_i16;
    let mut inherited_d3 = registers.d3_byte;
    for offset in [23, 33, 15] {
        e.write(frame.wrapping_sub(offset), &[0])?;
    }
    for offset in [22, 20, 6, 4] {
        e.ww(frame.wrapping_sub(offset), 0)?;
    }
    for offset in [18, 2] {
        e.ww(frame.wrapping_sub(offset), 1)?;
    }
    e.ww(0x437a92, 0)?;
    e.ww(0x4434aa, 0)?;
    while e.w(0x4434aa)? < 110 {
        e.step()?;
        e.ww(index(0x4434e0, e.w(0x4434aa)?, 2), 0)?;
        e.addw(0x4434aa, 1)?;
    }
    e.ww(0x4434aa, 0)?;
    while e.w(0x4434aa)? <= e.w(0x439576)? {
        e.step()?;
        e.quiet(e.w(0x4434aa)?, stack)?;
        e.addw(0x4434aa, 1)?;
    }
    e.runtime(0x13e13c)?;
    e.runtime(0x13a498)?;
    if e.w(0x438776)? != 0 {
        e.write(0x44273f, &[e.b(0x4435d3)?])?;
    }
    e.ww(0x4435cc, 1)?;
    loop {
        e.step()?;
        if e.cast(stack, caster(&e)?)? {
            break;
        }
        e.write(0x44273f, &[e.b(0x44273f)?.wrapping_add(1)])?;
        if caster(&e)? > i16::from(e.b(0x44273a)? as i8).wrapping_add(1) {
            e.write(0x44273f, &[0])?;
            e.write(
                frame.wrapping_sub(33),
                &[e.b(frame.wrapping_sub(33))?.wrapping_add(1)],
            )?;
            if e.b(frame.wrapping_sub(33))? > 2 {
                e.warn(stack, 13)?;
                e.full(1100);
                return Ok(e.plan);
            }
        }
    }
    e.ww(0x438760, 1)?;
    let dialog = e.trap(0xa97c, vec![W(144), L(0), L(u32::MAX)])?;
    e.lw(0x4394d4, dialog)?;
    e.sound(20002, stack)?;
    e.lw(0x4394b8, e.l(0x4394d4)?)?;
    e.trap(0xa873, vec![L(e.l(0x4394d4)?)])?;
    e.trap(0xaa0b, vec![L(e.l(0x4358ce)?)])?;
    e.trap(0xa887, vec![W(e.w(0x402f44)? as u16)])?;
    e.trap(
        0xa91b,
        vec![
            L(e.l(0x4394d4)?),
            W(e.w(0x4395a4)?
                .wrapping_add(320)
                .wrapping_add(e.w(0x40856c)?) as u16),
            W(e.w(0x4395a2)? as u16),
            B(0),
        ],
    )?;
    e.trap(0xa915, vec![L(e.l(0x4394d4)?)])?;
    e.trap(0xa981, vec![L(e.l(0x4394d4)?)])?;
    let mut phase = CastPhase::Redraw;
    let mut selected_school = 0_i16;
    let mut polls = 0_usize;
    loop {
        e.step()?;
        match phase {
            CastPhase::Redraw => {
                if e.w(0x438776)? != 0 || e.b(0x44273a)? == 0 {
                    e.get_item(e.l(0x4394d4)?, 45, 0x4434ba)?;
                    e.icon(stack, 0, 0x4434ba)?;
                }
                e.get_item(e.l(0x4394d4)?, 49, 0x4434ba)?;
                e.portrait(
                    stack,
                    0xffff0000,
                    e.w(warrior(caster(&e)?) + 0x34)?,
                    e.w(warrior(caster(&e)?) + 0x54)?,
                    0x4434ba,
                )?;
                e.trap(0xa862, vec![L(69)])?;
                for (offset, value) in [(14, -50), (10, -20), (12, -20), (8, -10)] {
                    e.ww(frame.wrapping_sub(offset), value)?;
                }
                e.trap(0xaa1c, vec![L(e.l(0x43952c)?)])?;
                e.trap(0xa862, vec![L(69)])?;
                spell_name(&mut e, stack.wrapping_sub(16))?;
                e.trap(0xa8a7, vec![P(frame.wrapping_sub(32)), L(0), L(0)])?;
                e.trap(0xa922, vec![L(e.l(0x4394d4)?)])?;
                e.trap(0xa923, vec![L(e.l(0x4394d4)?)])?;
                selected_school = school(&e)?;
                if e.b(frame.wrapping_sub(23))? == 0 {
                    e.ww(
                        0x4435c8,
                        i16::from(e.b(index(0x442580, caster(&e)?, 2))? as i8),
                    )?;
                    e.ww(
                        0x4435ca,
                        i16::from(e.b(index(0x442581, caster(&e)?, 2))? as i8),
                    )?;
                    e.ww(0x4435cc, e.w(index(0x437a7a, caster(&e)?, 2))?)?;
                }
                e.write(frame.wrapping_sub(23), &[0])?;
                for (offset, address) in [(4, 0x4435ca), (6, 0x4435c8), (2, 0x4435cc)] {
                    e.ww(frame.wrapping_sub(offset), e.w(address)?)?;
                }
                if e.w(0x4435cc)? == 0 {
                    e.ww(0x4435cc, 1)?;
                }
                e.get_item(e.l(0x4394d4)?, e.w(0x4435cc)?.wrapping_add(29), 0x44292c)?;
                e.button(stack, true, true)?;
                if e.w(0x4435ca)? > -1 || i16::from(e.b(0x44275b)?) != selected_school {
                    if e.w(0x4435ca)? > -1 {
                        e.trap(0xa873, vec![L(e.l(0x4394d4)?)])?;
                    }
                    e.spell_ui(0x10b6ba)?;
                    if e.w(0x4435ca)? > -1 {
                        e.load_spell(stack, selected_school, e.w(0x4435c8)?, e.w(0x4435ca)?)?;
                        spell_highlight(&mut e, frame)?;
                        if !spell_available(&e)? {
                            e.trap(0xa8f6, vec![L(e.l(0x439558)?), P(frame.wrapping_sub(14))])?;
                            gray = true;
                        } else {
                            e.trap(0xa8f6, vec![L(e.l(0x439550)?), P(frame.wrapping_sub(14))])?;
                            e.trap(0xaa14, vec![P(0x44337a)])?;
                            e.get_item(e.l(0x4394d4)?, e.w(0x4435ca)?.wrapping_add(1), 0x4434ba)?;
                            e.trap(0xa990, vec![L(e.l(0x4434dc)?), P(0x479ed8)])?;
                            e.di_str(stack, e.w(0x4435ca)?.wrapping_add(1), 0x479ed8)?;
                            e.trap(0xa862, vec![L(69)])?;
                        }
                        e.get_item(e.l(0x4394d4)?, e.w(0x4435c8)?.wrapping_add(23), 0x44292c)?;
                        e.button(stack, true, true)?;
                        e.trap(0xa862, vec![L(69)])?;
                        let cost = e.abs(spell_cost(&e, e.w(0x4435cc)?)?, stack)?;
                        e.dialog_number(stack, 41, cost)?;
                        let cost = e.abs(spell_cost(&e, e.w(0x4435cc)?)?, stack.wrapping_sub(4))?;
                        e.dialog_number(stack.wrapping_sub(4), 42, cost)?;
                        e.spell_ui(0x15947c)?;
                    }
                }
                if e.w(0x439586)? != 0 && e.w(0x4435ca)? > -1 {
                    if secondary == 0 {
                        secondary = e.trap(0xa97c, vec![W(169), L(0), L(u32::MAX)])?;
                    }
                    e.trap(0xa873, vec![L(secondary)])?;
                    e.trap(0xa88a, vec![W(10)])?;
                    e.trap(0xa887, vec![W(e.w(0x402f44)? as u16)])?;
                    e.trap(0xaa0b, vec![L(e.l(0x4358ce)?)])?;
                    e.trap(0xa862, vec![L(69)])?;
                    let record = e
                        .w(0x4435c8)?
                        .wrapping_mul(7)
                        .wrapping_add(selected_school.wrapping_mul(49));
                    spell_description(&mut e, secondary, record)?;
                    let y = e
                        .w(0x40856e)?
                        .wrapping_add(e.w(0x4395a2)?)
                        .wrapping_add(320)
                        .wrapping_sub(e.w(0x439582)?.wrapping_shl(5));
                    e.trap(
                        0xa91b,
                        vec![L(secondary), W(e.w(0x4395a4)? as u16), W(y as u16), B(0)],
                    )?;
                    e.trap(0xa915, vec![L(secondary)])?;
                    e.trap(0xa981, vec![L(secondary)])?;
                    e.trap(0xa98f, vec![L(e.l(0x4434dc)?), P(0x479ed8)])?;
                    e.trap(0xa922, vec![L(secondary)])?;
                    e.trap(0xa923, vec![L(secondary)])?;
                }
                e.trap(0xa873, vec![L(e.l(0x4394d4)?)])?;
                e.trap(0xa920, vec![L(e.l(0x4394d4)?)])?;
                phase = CastPhase::Prepare;
            }
            CastPhase::Prepare => {
                e.flush_events()?;
                e.tier(stack, 1)?;
                e.first_text(stack.wrapping_sub(2), 0x1a001c)?;
                if e.plan.flow != LoadFlow::Returned {
                    return Ok(e.plan);
                }
                phase = CastPhase::Poll;
            }
            CastPhase::Poll => {
                polls = polls
                    .checked_add(1)
                    .ok_or_else(|| RuleError::Invalid("castspell poll counter overflow".into()))?;
                if polls > limits.prompt.maximum_polls {
                    return Err(RuleError::Invalid(
                        "castspell dialog polling budget exhausted".into(),
                    ));
                }
                e.trap(0xa9b4, vec![])?;
                e.music(stack)?;
                if e.plan.flow != LoadFlow::Returned {
                    return Ok(e.plan);
                }
                let got = e.trap(0xa970, vec![W(0xffff), P(0x44720a)])?;
                e.ww(0x4434aa, i16::from(got as u8))?;
                if e.w(0x44720a)? == 3 {
                    phase = CastPhase::Key;
                    continue;
                }
                let is_dialog = e.trap(0xa97f, vec![P(0x44720a)])? as u8 != 0;
                if !is_dialog {
                    phase = if e.w(0x44720a)? == 3 {
                        CastPhase::Key
                    } else {
                        CastPhase::Poll
                    };
                    continue;
                }
                let selected =
                    e.trap(0xa980, vec![P(0x44720a), P(0x4394c0), P(0x4434b2)])? as u8 != 0;
                if !selected {
                    continue;
                }
                e.get_item(e.l(0x4394d4)?, e.w(0x4434b2)?, 0x44292c)?;
                selected_school = school(&e)?;
                phase = CastPhase::Dispatch;
            }
            CastPhase::Dispatch => {
                let item = e.w(0x4434b2)?;
                if item == 37 {
                    e.get_item(e.l(0x4394d4)?, 37, 0x44292c)?;
                    e.load_spell(stack, selected_school, e.w(0x4435c8)?, e.w(0x4435ca)?)?;
                    if e.w(0x4435ca)? == -1 {
                        e.warn(stack, 12)?;
                        e.get_item(e.l(0x4394d4)?, 37, 0x44292c)?;
                        e.trap(0xa981, vec![L(e.l(0x4394d4)?)])?;
                        phase = CastPhase::Redraw;
                        continue;
                    }
                    e.press(stack, 1, 2, 0, 0x44292c)?;
                    e.ww(frame.wrapping_sub(36), e.w(warrior(caster(&e)?) + 0x58)?)?;
                    let cost = e.abs(spell_cost(&e, e.w(0x4435cc)?)?, stack.wrapping_sub(14))?;
                    if e.w(frame.wrapping_sub(36))? < cost {
                        e.warn(stack, 18)?;
                        e.get_item(e.l(0x4394d4)?, 37, 0x44292c)?;
                        e.trap(0xa981, vec![L(e.l(0x4394d4)?)])?;
                        phase = CastPhase::Redraw;
                        continue;
                    }
                    let warning = if (e.b(0x443764)? as i8) < 0 && e.w(0x447198)? == 0 {
                        Some(91)
                    } else if e.w(0x438776)? != 0 && e.b(0x443763)? == 0 && e.w(0x447198)? == 0 {
                        Some(23)
                    } else if e.w(0x438776)? == 0 && e.b(0x443764)? == 0 && e.w(0x447198)? == 0 {
                        Some(22)
                    } else {
                        None
                    };
                    if let Some(id) = warning {
                        e.warn(stack, id)?;
                        e.trap(0xa981, vec![L(e.l(0x4394d4)?)])?;
                        phase = CastPhase::Redraw;
                        continue;
                    }
                    e.ww(0x4435c6, selected_school)?;
                    phase = CastPhase::Cleanup;
                } else if item == 38 {
                    e.press(stack, 1, 2, 0, 0x44292c)?;
                    e.ww(0x4435c6, 0)?;
                    e.ww(0x4435c8, 0)?;
                    e.ww(0x438760, 0)?;
                    e.ww(0x4435ca, -1)?;
                    phase = CastPhase::Cleanup;
                } else if matches!(item, 39 | 40)
                    && e.w(0x438776)? == 0
                    && i16::from(e.b(0x44273a)? as i8).wrapping_sub(i16::from(e.b(0x442739)? as i8))
                        > 0
                {
                    gray = false;
                    e.write(frame.wrapping_sub(15), &[1])?;
                    e.get_item(e.l(0x4394d4)?, e.w(0x4435c8)?.wrapping_add(23), 0x44292c)?;
                    e.button(stack, false, true)?;
                    e.get_item(e.l(0x4394d4)?, e.w(0x4435cc)?.wrapping_add(29), 0x44292c)?;
                    e.button(stack, false, true)?;
                    e.get_item(e.l(0x4394d4)?, e.w(0x4434b2)?, 0x4434ba)?;
                    e.press(stack, 3, 2, 0, 0x4434ba)?;
                    loop {
                        e.step()?;
                        if e.w(0x4434b2)? == 39 {
                            e.write(0x44273f, &[e.b(0x44273f)?.wrapping_sub(1)])?;
                            if caster(&e)? < 0 {
                                e.write(0x44273f, &[e.b(0x44273a)?])?;
                            }
                        } else {
                            e.write(0x44273f, &[e.b(0x44273f)?.wrapping_add(1)])?;
                            if caster(&e)? > i16::from(e.b(0x44273a)? as i8) {
                                e.write(0x44273f, &[0])?;
                            }
                        }
                        if e.cast(stack, caster(&e)?)? {
                            inherited_d3 = Some(0);
                            break;
                        }
                    }
                    e.sound(141, stack)?;
                    e.get_item(e.l(0x4394d4)?, 55, 0x4434ba)?;
                    e.pict(stack.wrapping_sub(2), 154, 0x4434ba)?;
                    e.get_item(e.l(0x4394d4)?, 49, 0x4434ba)?;
                    e.portrait(
                        stack.wrapping_sub(12),
                        0xffff0000,
                        e.w(warrior(caster(&e)?) + 0x34)?,
                        e.w(warrior(caster(&e)?) + 0x54)?,
                        0x4434ba,
                    )?;
                    e.trap(0xa862, vec![L(69)])?;
                    spell_name(&mut e, stack.wrapping_sub(28))?;
                    e.ww(0x4435c8, -1)?;
                    e.get_item(e.l(0x4394d4)?, e.w(0x4434b2)?, 0x4434ba)?;
                    e.press(stack.wrapping_sub(46), 3, 1, 0, 0x4434ba)?;
                    e.ww(0x4434b2, 23)?;
                    phase = CastPhase::ClassCleanup;
                } else if item > 22 && item < 37 {
                    phase = if item < 30 {
                        CastPhase::Class
                    } else {
                        CastPhase::Quantity
                    };
                } else if item < 13 {
                    e.get_item(e.l(0x4394d4)?, item.wrapping_add(57), 0x44292c)?;
                    e.button(stack, true, true)?;
                    e.delay(stack.wrapping_sub(2), 6)?;
                    if e.w(0x437a92)? != 1 {
                        e.tier(stack, 0)?;
                        e.ww(0x437a92, 1)?;
                        e.tier(stack.wrapping_sub(2), 1)?;
                    }
                    phase = CastPhase::Slot;
                } else {
                    phase = CastPhase::Poll;
                }
            }
            CastPhase::Class => {
                if e.w(0x437a92)? != 0 {
                    e.tier(stack, 0)?;
                    e.ww(0x437a92, 0)?;
                    e.tier(stack.wrapping_sub(2), 1)?;
                }
                if e.w(0x4434b2)?.wrapping_sub(23) == e.w(0x4435c8)? {
                    phase = CastPhase::Prepare;
                    continue;
                }
                e.get_item(e.l(0x4394d4)?, e.w(0x4435c8)?.wrapping_add(23), 0x44292c)?;
                e.button(stack, false, true)?;
                e.sound(10129, stack.wrapping_sub(2))?;
                phase = CastPhase::ClassCleanup;
            }
            CastPhase::ClassCleanup => {
                e.trap(0xa8f6, vec![L(e.l(0x439554)?), P(frame.wrapping_sub(14))])?;
                for (i, item) in [41, 42, 13, 14, 50, 15, 18, 19].into_iter().enumerate() {
                    e.di_str(stack.wrapping_sub((i * 6) as u32), item, 0x403f22)?;
                }
                e.trap(0xa8f6, vec![L(e.l(0x439554)?), P(frame.wrapping_sub(32))])?;
                if e.b(frame.wrapping_sub(15))? != 0 {
                    e.write(frame.wrapping_sub(15), &[0])?;
                    phase = CastPhase::Redraw;
                    continue;
                }
                e.ww(0x4435ca, -1)?;
                e.ww(frame.wrapping_sub(4), 0)?;
                e.ww(0x4435c8, e.w(0x4434b2)?.wrapping_sub(23))?;
                e.ww(frame.wrapping_sub(6), e.w(0x4435c8)?)?;
                e.get_item(e.l(0x4394d4)?, e.w(0x4434b2)?, 0x44292c)?;
                e.button(stack, true, true)?;
                e.spell_ui(0x10b6ba)?;
                e.ww(0x4434b2, 1)?;
                e.write(frame.wrapping_sub(16), &[1])?;
                phase = CastPhase::Slot;
            }
            CastPhase::Quantity => {
                if e.w(0x4435ca)? == -1 || e.w(0x4434b2)?.wrapping_sub(29) == e.w(0x4435cc)? {
                    phase = CastPhase::Prepare;
                    continue;
                }
                e.ww(frame.wrapping_sub(36), e.w(warrior(caster(&e)?) + 0x58)?)?;
                let cost = e.abs(spell_cost(&e, e.w(0x4434b2)?.wrapping_sub(29))?, stack)?;
                phase = if cost > e.w(frame.wrapping_sub(36))? {
                    CastPhase::Clamp
                } else {
                    CastPhase::ApplyQuantity
                };
                if matches!(phase, CastPhase::ApplyQuantity) && e.w(0x437a92)? != 2 {
                    e.tier(stack, 0)?;
                    e.ww(0x437a92, 2)?;
                    e.tier(stack.wrapping_sub(2), 1)?;
                }
            }
            CastPhase::Clamp => {
                e.sound(612, stack)?;
                let divisor = e.abs(i16::from(e.b(0x44376c)?), stack.wrapping_sub(2))?;
                if divisor == 0 {
                    return Err(RuleError::Invalid(
                        "castspell reached native DIVS by zero".into(),
                    ));
                }
                let dividend = i32::from(e.w(warrior(caster(&e)?) + 0x58)?);
                let quotient = dividend / i32::from(divisor);
                let quantity = i16::try_from(quotient)
                    .map_err(|_| {
                        RuleError::Invalid("castspell reached native DIVS quotient overflow".into())
                    })?
                    .wrapping_add(29);
                e.ww(0x4434b2, quantity)?;
                if e.w(0x4434b2)? < 30 {
                    e.ww(0x4434b2, 30)?;
                }
                phase = CastPhase::ApplyQuantity;
            }
            CastPhase::ApplyQuantity => {
                e.trap(0xa862, vec![L(69)])?;
                e.sound(137, stack)?;
                e.ww(0x4435cc, e.w(0x4434b2)?.wrapping_sub(29))?;
                e.ww(frame.wrapping_sub(2), e.w(0x4435cc)?)?;
                if e.w(0x4435ca)? > -1 {
                    let cost = e.abs(spell_cost(&e, e.w(0x4435cc)?)?, stack)?;
                    e.dialog_number(stack, 42, cost)?;
                }
                e.get_item(e.l(0x4394d4)?, 48, 0x4434ba)?;
                e.pict(stack, 129, 0x4434ba)?;
                e.get_item(e.l(0x4394d4)?, e.w(0x4435cc)?.wrapping_add(29), 0x44292c)?;
                e.button(stack.wrapping_sub(10), true, true)?;
                e.tier(stack, 1)?;
                e.spell_ui(0x15947c)?;
                phase = CastPhase::Poll;
            }
            CastPhase::Slot => {
                e.ww(frame.wrapping_sub(4), e.w(0x4434b2)?.wrapping_sub(1))?;
                e.get_item(e.l(0x4394d4)?, e.w(0x4434b2)?, 0x4434ba)?;
                e.trap(0xa990, vec![L(e.l(0x4434dc)?), P(0x479ed8)])?;
                let width = e.trap(0xa88c, vec![P(0x479ed8)])? as i16;
                if width == 0 {
                    phase = scan_spell_slot(&mut e, &mut direction)?;
                    continue;
                }
                e.get_item(e.l(0x4394d4)?, e.w(0x4434b2)?.wrapping_add(57), 0x44292c)?;
                e.load_spell(
                    stack,
                    selected_school,
                    e.w(0x4435c8)?,
                    e.w(0x4434b2)?.wrapping_sub(1),
                )?;
                if e.w(0x439586)? != 0 && secondary != 0 {
                    let record = e
                        .w(0x4434b2)?
                        .wrapping_add(e.w(0x4435c8)?.wrapping_mul(7))
                        .wrapping_add(selected_school.wrapping_mul(49))
                        .wrapping_sub(1);
                    spell_description(&mut e, secondary, record)?;
                    e.trap(0xa98f, vec![L(e.l(0x4434dc)?), P(0x479ed8)])?;
                }
                let available = spell_available(&e)?;
                if !available && e.w(0x438776)? != 0 {
                    phase = scan_spell_slot(&mut e, &mut direction)?;
                    continue;
                }
                if available {
                    if e.b(frame.wrapping_sub(16))? != 1 {
                        e.sound(141, stack)?;
                    }
                    e.write(frame.wrapping_sub(16), &[0])?;
                    e.get_item(e.l(0x4394d4)?, e.w(0x4435ca)?.wrapping_add(1), 0x4434ba)?;
                    e.trap(0xa990, vec![L(e.l(0x4434dc)?), P(0x479ed8)])?;
                    if gray {
                        e.trap(0xaa14, vec![P(0x44337a)])?;
                    } else {
                        e.trap(0xa862, vec![L(69)])?;
                    }
                    gray = false;
                    if e.w(0x4435ca)?.wrapping_add(1) != e.w(0x4434b2)? {
                        e.di_str(stack, e.w(0x4435ca)?.wrapping_add(1), 0x479ed8)?;
                    }
                    e.ww(0x4435ca, e.w(0x4434b2)?.wrapping_sub(1))?;
                    e.trap(0xa8f6, vec![L(e.l(0x439554)?), P(frame.wrapping_sub(14))])?;
                    spell_highlight(&mut e, frame)?;
                    e.button(stack, true, true)?;
                    e.trap(0xa862, vec![L(69)])?;
                    e.trap(0xa8f6, vec![L(e.l(0x439550)?), P(frame.wrapping_sub(14))])?;
                    e.trap(0xaa14, vec![P(0x44337a)])?;
                    let resource = e
                        .w(0x4435c8)?
                        .wrapping_add(selected_school.wrapping_add(1).wrapping_mul(1000));
                    e.lib(
                        0x400178,
                        vec![P(0x479ed8), W(resource as u16), W(e.w(0x4434b2)? as u16)],
                    )?;
                    e.di_str(stack.wrapping_sub(2), e.w(0x4434b2)?, 0x479ed8)?;
                    e.trap(0xa862, vec![L(69)])?;
                }
                let cost = e.abs(i16::from(e.b(0x44376c)?), stack)?;
                e.dialog_number(stack, 41, cost)?;
                let cost = e.abs(spell_cost(&e, e.w(0x4435cc)?)?, stack.wrapping_sub(4))?;
                e.dialog_number(stack.wrapping_sub(4), 42, cost)?;
                if spell_available(&e)? {
                    e.button(stack, false, true)?;
                }
                e.ww(frame.wrapping_sub(36), e.w(warrior(caster(&e)?) + 0x58)?)?;
                let cost = e.abs(spell_cost(&e, e.w(0x4435cc)?)?, stack)?;
                if cost > e.w(frame.wrapping_sub(36))? {
                    phase = CastPhase::Clamp;
                } else {
                    e.spell_ui(0x15947c)?;
                    phase = CastPhase::Poll;
                }
            }
            CastPhase::Key => {
                e.ww(0x443316, e.w(0x44720e)?)?;
                let tier = e.w(0x437a92)?;
                e.ww(
                    index(frame.wrapping_sub(22), tier, 2),
                    e.w(index(frame.wrapping_sub(6), e.w(0x437a92)?, 2))?,
                )?;
                e.write(frame.wrapping_sub(23), &[0])?;
                e.write(0x437a6b, &[0])?;
                let key = e.w(0x443316)? as u16;
                if key == 0x61 {
                    e.ww(0x4434b2, 38)?;
                    phase = CastPhase::Dispatch;
                    continue;
                }
                if matches!(key, 0x863 | 0x240d) {
                    e.ww(0x4434b2, 37)?;
                    phase = CastPhase::Dispatch;
                    continue;
                }
                if let Some(n) = match key {
                    0x1231 => Some(1),
                    0x1332 => Some(2),
                    0x1433 => Some(3),
                    0x1534 => Some(4),
                    0x1735 => Some(5),
                    0x1636 => Some(6),
                    0x1a37 => Some(7),
                    0x1c38 => Some(8),
                    0x1939 => Some(9),
                    0x1d30 => Some(10),
                    _ => None,
                } {
                    e.write(0x437a6b, &[n])?;
                } else if matches!(key, 0x5432 | 0x7d1f | 0x5b38 | 0x7e1e) {
                    direction = if matches!(key, 0x5432 | 0x7d1f) {
                        1
                    } else {
                        -1
                    };
                    e.addw(index(frame.wrapping_sub(6), e.w(0x437a92)?, 2), direction)?;
                } else if matches!(key, 0x5634 | 0x7b1c | 0x5836 | 0x7c1d) {
                    e.tier(stack, 0)?;
                    e.addw(
                        0x437a92,
                        if matches!(key, 0x5634 | 0x7b1c) {
                            -1
                        } else {
                            1
                        },
                    )?;
                }
                if e.b(0x437a6b)? != 0 {
                    let mask =
                        e.trap(0xa858, vec![L(u32::from(e.w(0x447218)? as u16)), L(0x100)])?;
                    if mask == 0 {
                        e.write(frame.wrapping_sub(23), &[1])?;
                        e.sound(145, stack)?;
                        for (offset, address) in
                            [(0, 0x4435c6), (2, 0x4435c8), (4, 0x4435ca), (6, 0x4435cc)]
                        {
                            e.ww(address, e.w(shortcut_address(&e, offset)?)?)?;
                        }
                        e.trap(0xa981, vec![L(e.l(0x4394d4)?)])?;
                        phase = CastPhase::Redraw;
                        continue;
                    }
                    e.sound(144, stack)?;
                    e.ww(0x4435c6, selected_school)?;
                    for (offset, address) in
                        [(0, 0x4435c6), (2, 0x4435c8), (4, 0x4435ca), (6, 0x4435cc)]
                    {
                        e.ww(shortcut_address(&e, offset)?, e.w(address)?)?;
                    }
                }
                if e.w(0x437a92)? < 0 || e.w(0x437a92)? > 2 {
                    e.addw(0x437a92, if e.w(0x437a92)? < 0 { 1 } else { -1 })?;
                }
                e.sound(130, stack)?;
                e.tier(stack.wrapping_sub(2), 1)?;
                let a = index(frame.wrapping_sub(6), e.w(0x437a92)?, 2);
                let value = e.w(a)?;
                if !(0..=6).contains(&value) && e.w(0x437a92)? == 0 {
                    e.ww(
                        frame.wrapping_sub(6),
                        if e.w(frame.wrapping_sub(6))? < 0 {
                            0
                        } else {
                            6
                        },
                    )?;
                }
                let value = e.w(index(frame.wrapping_sub(6), e.w(0x437a92)?, 2))?;
                if !(1..=7).contains(&value) && e.w(0x437a92)? == 2 {
                    e.ww(
                        frame.wrapping_sub(2),
                        if e.w(frame.wrapping_sub(2))? < 1 {
                            1
                        } else {
                            7
                        },
                    )?;
                }
                if e.w(0x437a92)? == 1 && !(0..7).contains(&e.w(frame.wrapping_sub(4))?) {
                    e.ww(
                        frame.wrapping_sub(4),
                        if e.w(frame.wrapping_sub(4))? < 0 {
                            0
                        } else {
                            6
                        },
                    )?;
                }
                if e.w(index(frame.wrapping_sub(22), e.w(0x437a92)?, 2))?
                    == e.w(index(frame.wrapping_sub(6), e.w(0x437a92)?, 2))?
                {
                    phase = CastPhase::Poll;
                    continue;
                }
                match e.w(0x437a92)? {
                    0 => {
                        e.ww(0x4434b2, e.w(frame.wrapping_sub(6))?.wrapping_add(23))?;
                        phase = CastPhase::Class;
                    }
                    1 => {
                        e.ww(0x4434b2, e.w(frame.wrapping_sub(4))?.wrapping_add(1))?;
                        phase = CastPhase::Slot;
                    }
                    2 => {
                        e.ww(0x4434b2, e.w(frame.wrapping_sub(2))?.wrapping_add(29))?;
                        if e.w(0x437a92)? != 2 {
                            e.tier(stack, 0)?;
                            e.ww(0x437a92, 2)?;
                            e.tier(stack.wrapping_sub(2), 1)?;
                        }
                        phase = CastPhase::ApplyQuantity;
                    }
                    _ => phase = CastPhase::Poll,
                }
            }
            CastPhase::Cleanup => {
                e.trap(0xa983, vec![L(e.l(0x4394d4)?)])?;
                e.out(stack)?;
                if e.w(0x447198)? != 0 {
                    let n = inherited_d3.ok_or_else(|| {
                        RuleError::Invalid(
                            "castspell special target cleanup needs inherited D3.B".into(),
                        )
                    })?;
                    e.ww(0x4434aa, 0)?;
                    while e.w(0x4434aa)? < 8 {
                        e.step()?;
                        e.ww(index(0x4434e0, i16::from(n), 2), 0)?;
                        e.addw(0x4434aa, 1)?;
                    }
                    e.ww(index(0x4434e0, caster(&e)?, 2), 1)?;
                }
                if e.w(0x4435ca)? > -1 {
                    let cost = e.abs(spell_cost(&e, e.w(0x4435cc)?)?, stack)?;
                    let address = warrior(caster(&e)?) + 0x58;
                    e.ww(address, e.w(address)?.wrapping_sub(cost))?;
                    e.write(index(0x442580, caster(&e)?, 2), &[e.w(0x4435c8)? as u8])?;
                    e.write(index(0x442581, caster(&e)?, 2), &[e.w(0x4435ca)? as u8])?;
                    e.ww(index(0x437a7a, caster(&e)?, 2), e.w(0x4435cc)?)?;
                    e.ww(index(0x442798, caster(&e)?, 2), 1)?;
                }
                e.lw(0x4394d4, 0)?;
                if secondary != 0 {
                    e.trap(0xa983, vec![L(secondary)])?;
                }
                let identifier = e
                    .w(0x4435c8)?
                    .wrapping_mul(100)
                    .wrapping_add(e.w(0x4435c6)?.wrapping_mul(1000))
                    .wrapping_add(e.w(0x4435ca)?)
                    .wrapping_add(1101);
                if identifier > 1100 {
                    e.write(0x435662, &[1])?;
                    e.write(0x43565e, &[0])?;
                    if e.w(0x438776)? != 0 {
                        let address = warrior(e.w(0x4435d2)?) + 0xa66;
                        e.lw(address, e.l(address)?.wrapping_add(1))?;
                        e.addw(warrior(e.w(0x4435d2)?) + 0xac8, 1)?;
                    }
                } else {
                    e.write(0x435662, &[0])?;
                }
                let product = i32::from(e.w(0x4435c8)?) * 100;
                let identifier = e
                    .w(0x4435ca)?
                    .wrapping_add(product as i16)
                    .wrapping_add(e.w(0x4435c6)?.wrapping_mul(1000))
                    .wrapping_add(1101);
                e.full((product as u32 & 0xffff0000) | u32::from(identifier as u16));
                return Ok(e.plan);
            }
        }
    }
}

fn scan_spell_slot<S: SelectionControlServices + ?Sized>(
    e: &mut Execution<'_, S>,
    direction: &mut i16,
) -> Result<CastPhase, RuleError> {
    e.addw(0x4434b2, *direction)?;
    if e.w(0x4434b2)? < 1 || e.w(0x4434b2)? > 12 {
        if *direction == -1 {
            return Ok(CastPhase::Prepare);
        }
        *direction = direction.wrapping_neg();
        return Ok(CastPhase::Slot);
    }
    Ok(CastPhase::Slot)
}

fn current_list<S: SelectionControlServices + ?Sized>(
    e: &Execution<'_, S>,
    frame: u32,
) -> Result<i16, RuleError> {
    e.w(list_address(frame, e.w(0x4434aa)?))
}
fn selected_list<S: SelectionControlServices + ?Sized>(
    e: &Execution<'_, S>,
    frame: u32,
) -> Result<i16, RuleError> {
    e.w(list_address(frame, e.w(0x4434b2)?.wrapping_sub(3)))
}
fn read_creature<S: SelectionControlServices + ?Sized>(
    e: &mut Execution<'_, S>,
    stack: u32,
    selection: i16,
    extra: i16,
) -> Result<(), RuleError> {
    e.filename(stack, 0x404080)?;
    let file = e.lib(0x400070, vec![P(0x40407c), P(0x4389a4)])?.value;
    let record = if extra != 0 {
        extra
    } else {
        e.w(index(0x4395a6, selection, 2))?.wrapping_sub(1)
    };
    e.lib(
        0x400078,
        vec![W(0), L(u32::from(record as u16) * 288), L(file)],
    )?;
    e.lib(0x400048, vec![L(file), L(1), L(288), P(0x440dbc)])?;
    e.lib(0x400068, vec![L(file)])?;
    Ok(())
}

/// beast, 0x103ebe. D4 is required only when the non-file path reaches navigation.
pub fn beast(
    state: &SelectionControlState,
    frame: u32,
    registers: SelectionRegisters,
    limits: SelectionControlLimits,
    services: &mut (impl SelectionControlServices + ?Sized),
) -> Result<SelectionControlPlan, RuleError> {
    let mut e = execution(state, limits, services);
    let stack = frame.wrapping_sub(284);
    let body = e.w(frame.wrapping_add(8))?;
    let mode = e.w(frame.wrapping_add(10))?;
    let mut extra = e.w(frame.wrapping_add(12))?;
    let mut selection = registers.d4_word;
    e.trap(0xa874, vec![P(frame.wrapping_sub(260))])?;
    e.trap(0xaa1c, vec![L(e.l(0x43952c)?)])?;
    let mut channel = 0_i16;
    while channel <= e.w(0x439576)? {
        e.step()?;
        e.quiet(channel, stack)?;
        channel = channel.wrapping_add(1);
    }
    e.sound(201, stack)?;
    e.runtime(0x13e13c)?;
    let count = e.trap(0xa950, vec![L(e.l(0x438722)?)])?;
    e.ww(0x439cf8, count as i16)?;
    e.runtime(0x13a498)?;
    let dialog = e.trap(0xa97c, vec![W(158), L(0), L(u32::MAX)])?;
    e.lw(0x4394bc, dialog)?;
    e.lw(0x4394b8, e.l(0x4394bc)?)?;
    e.trap(0xa873, vec![L(e.l(0x4394bc)?)])?;
    e.trap(0xaa0b, vec![L(e.l(0x4358ce)?)])?;
    e.trap(0xa88a, vec![W(10)])?;
    e.trap(0xa887, vec![W(10)])?;
    e.trap(0xa862, vec![L(69)])?;
    let x = e
        .w(0x4395a4)?
        .wrapping_add(142)
        .wrapping_sub(half(e.w(0x40856c)?));
    let y = e.w(0x4395a2)?.wrapping_add(40);
    e.trap(
        0xa91b,
        vec![L(e.l(0x4394bc)?), W(x as u16), W(y as u16), B(0)],
    )?;
    for opcode in [0xa915, 0xa981, 0xa922, 0xa923] {
        e.trap(opcode, vec![L(e.l(0x4394bc)?)])?;
    }
    e.get_item(e.l(0x4394bc)?, 37, 0x4434ba)?;
    e.pict(stack, 142, 0x4434ba)?;
    if mode == 1 {
        read_creature(&mut e, stack, body, extra)?;
        selection = Some(body);
    } else {
        e.copy_longs(npc(body.wrapping_sub(10)), 0x440dbc, 72)?;
    }
    let mut redraw = true;
    for _ in 0..limits.prompt.maximum_polls {
        e.step()?;
        if redraw {
            e.lib(0x400090, vec![P(0x440e96), P(0x438aa4)])?;
            e.trap(0xa862, vec![L(69)])?;
            e.di_str(stack.wrapping_sub(8), 7, 0x438aa4)?;
            if mode == 1 {
                e.filename(stack, 0x404072)?;
                let file = e.lib(0x400070, vec![P(0x40407c), P(0x4389a4)])?.value;
                if file == 0 {
                    e.scratch(stack, 5)?;
                    if e.plan.flow != LoadFlow::Returned {
                        return Ok(e.plan);
                    }
                }
                let record = if extra != 0 {
                    extra
                } else {
                    e.w(index(
                        0x4395a6,
                        selection.ok_or_else(|| {
                            RuleError::Invalid("beast description needs reached D4.W".into())
                        })?,
                        2,
                    ))?
                    .wrapping_sub(1)
                };
                e.lib(
                    0x400078,
                    vec![W(0), L((i32::from(record) as u32).wrapping_shl(8)), L(file)],
                )?;
                e.lib(
                    0x400048,
                    vec![L(file), L(1), L(256), P(frame.wrapping_sub(256))],
                )?;
                e.lib(0x400068, vec![L(file)])?;
                e.di_str(stack.wrapping_sub(20), 28, frame.wrapping_sub(256))?;
                extra = 0;
            }
            e.di_str(stack, 6, 0x404071)?;
            if e.abs(e.w(0x440dcc)?, stack.wrapping_sub(6))? < 2 {
                e.trap(0xa8a9, vec![P(0x4434ba), L(16)])?;
            }
            if matches!(e.w(0x440dcc)?, 0 | 2) {
                e.trap(0xa8a9, vec![P(0x4434ba), L(0x100000)])?;
            }
            e.lw(0x439598, 0)?;
            let icon = e.trap(0xaa1e, vec![W(e.w(0x440e0c)? as u16)])?;
            e.lw(0x439598, icon)?;
            if e.l(0x439598)? != 0 {
                e.trap(0xaa1f, vec![P(0x4434ba), L(e.l(0x439598)?)])?;
                e.trap(0xaa25, vec![L(e.l(0x439598)?)])?;
            }
            for item in 11..23 {
                e.step()?;
                e.di_str(stack, item, 0x404071)?;
            }
            if e.w(0x440e0a)? != 0 {
                for (n, pointer) in [(8, 0x40406a), (9, 0x404062), (10, 0x40405a)] {
                    e.di_str(stack.wrapping_sub(((n - 8) * 6) as u32), n, pointer)?;
                }
            } else {
                for attack in 0..5_i16 {
                    e.step()?;
                    let record = index(0x440dbc, attack, 4);
                    if e.b(record + 0x93)? != 0 {
                        let value =
                            i16::from(e.b(record + 0x93)? as i8).wrapping_add(e.w(0x440dfc)?);
                        e.dialog_number(stack, 13 + attack, value)?;
                        let value =
                            i16::from(e.b(record + 0x94)? as i8).wrapping_add(e.w(0x440dfc)?);
                        e.dialog_number(stack.wrapping_sub(4), 18 + attack, value)?;
                        let string = i16::from(e.b(record + 0x95)? as i8).wrapping_sub(98);
                        e.lib(0x400178, vec![P(0x479ed8), W(128), W(string as u16)])?;
                        e.di_str(stack.wrapping_sub(8), 8 + attack, 0x479ed8)?;
                    } else {
                        for (n, offset) in [(13 + attack, 0), (18 + attack, 6), (8 + attack, 12)] {
                            e.di_str(stack.wrapping_sub(offset), n, 0x404071)?;
                        }
                    }
                }
            }
            e.di_str(stack, 27, 0x404046)?;
            for attack in 0..5_i16 {
                e.step()?;
                let b = e.b(index(0x440dbc, attack, 4) + 0x96)?;
                if b != 0 {
                    e.lib(
                        0x400178,
                        vec![
                            P(0x479ed8),
                            W(137),
                            W(i16::from(b as i8).wrapping_add(1) as u16),
                        ],
                    )?;
                    e.di_str(stack, 27, 0x479ed8)?;
                }
            }
            for (offset, (item, address)) in [
                (24, 0x440dc6),
                (38, 0x440dbc),
                (26, 0x440dc4),
                (23, 0x440dc2),
                (25, 0x440e12),
            ]
            .into_iter()
            .enumerate()
            {
                e.dialog_number(stack.wrapping_sub((offset * 4) as u32), item, e.w(address)?)?;
            }
            if mode != 0 {
                for n in 0..8_i16 {
                    e.step()?;
                    e.get_item(e.l(0x4394bc)?, 29 + n, 0x4434ba)?;
                    let pict = e.l(if e.b(0x440e64 + n as u32)? != 0 {
                        0x439550
                    } else {
                        0x439554
                    })?;
                    e.trap(0xa8f6, vec![L(pict), P(0x4434ba)])?;
                }
                for (base, negative) in [(45, false), (38, true)] {
                    for n in 1..8_i16 {
                        e.step()?;
                        e.get_item(e.l(0x4394bc)?, base + n, 0x4434ba)?;
                        let b = e.b(0x440e47 + n as u32)? as i8;
                        let yes = if negative { b < 0 } else { b > 100 };
                        e.trap(
                            0xa8f6,
                            vec![L(e.l(if yes { 0x439550 } else { 0x439554 })?), P(0x4434ba)],
                        )?;
                    }
                }
            }
            redraw = false;
        }
        e.flush_events()?;
        e.trap(0xa991, vec![L(0), P(0x4434b2)])?;
        e.music(stack)?;
        if e.plan.flow != LoadFlow::Returned {
            return Ok(e.plan);
        }
        e.get_item(e.l(0x4394bc)?, e.w(0x4434b2)?, 0x44292c)?;
        if matches!(e.w(0x4434b2)?, 1 | 4) {
            e.get_item(e.l(0x4394bc)?, 4, 0x44292c)?;
            e.press(stack, 1, 1, 0, 0x44292c)?;
            e.sound(205, stack.wrapping_sub(14))?;
            e.trap(0xa873, vec![L(e.l(frame.wrapping_sub(260))?)])?;
            e.trap(0xa983, vec![L(e.l(0x4394bc)?)])?;
            e.out(stack.wrapping_sub(16))?;
            e.full(0);
            return Ok(e.plan);
        }
        if matches!(e.w(0x4434b2)?, 2 | 3) {
            e.sound(202, stack)?;
            e.press(stack.wrapping_sub(2), 3, 2, 0, 0x44292c)?;
            e.delay(stack.wrapping_sub(16), 3)?;
            let mut n = selection.ok_or_else(|| {
                RuleError::Invalid("beast navigation needs inherited D4.W".into())
            })?;
            if e.w(0x4434b2)? == 3 {
                n = n.wrapping_add(2);
            }
            n = n.wrapping_sub(1).max(1);
            if n > e.w(0x439cf8)? {
                n = e.w(0x439cf8)?;
            }
            selection = Some(n);
            read_creature(&mut e, stack, n, extra)?;
            extra = 0;
            e.press(stack.wrapping_sub(20), 3, 1, 0, 0x44292c)?;
            redraw = true;
        }
    }
    Err(RuleError::Invalid(
        "beast dialog polling budget exhausted".into(),
    ))
}
fn survivor_cursor<S: SelectionControlServices + ?Sized>(
    e: &mut Execution<'_, S>,
    selected: i16,
    capacity: i16,
    allow_null: bool,
) -> Result<(), RuleError> {
    let id = 147_i16.wrapping_sub(selected).wrapping_add(capacity);
    let handle = e.trap(
        0xaa1b,
        vec![W(if (147..155).contains(&id) {
            id as u16
        } else {
            147
        })],
    )?;
    if handle != 0 || !allow_null {
        e.trap(0xaa1c, vec![L(handle)])?;
        e.trap(0xaa26, vec![L(handle)])?;
    }
    Ok(())
}
fn survivor_notice<S: SelectionControlServices + ?Sized>(
    e: &mut Execution<'_, S>,
    stack: u32,
    pointer: u32,
) -> Result<(), RuleError> {
    e.trap(0xa889, vec![W(1)])?;
    e.trap(0xa88a, vec![W(12)])?;
    e.trap(0xa862, vec![L(69)])?;
    e.di_str(stack, 41, pointer)?;
    e.sound(6000, stack.wrapping_sub(6))?;
    e.delay(stack.wrapping_sub(8), 80)?;
    e.di_str(stack.wrapping_sub(10), 41, 0x404000)?;
    Ok(())
}

/// bodycount, 0x1047f8. The survivor list is 110 native WORDs in the caller frame.
pub fn body_count(
    state: &SelectionControlState,
    frame: u32,
    limits: SelectionControlLimits,
    services: &mut (impl SelectionControlServices + ?Sized),
) -> Result<SelectionControlPlan, RuleError> {
    let mut e = execution(state, limits, services);
    let stack = frame.wrapping_sub(240);
    if e.w(0x408578)? != 0 {
        return Ok(e.plan);
    }
    let mut count = 0_i16;
    let mut capacity = 4_i16;
    let mut selected = 0_i16;
    e.ww(0x4434aa, 0)?;
    while e.w(0x4434aa)? < 110 {
        e.step()?;
        e.ww(list_address(frame, e.w(0x4434aa)?), 0)?;
        e.addw(0x4434aa, 1)?;
    }
    e.ww(0x4434aa, 0)?;
    while e.w(0x4434aa)? < e.w(0x43874a)? {
        e.step()?;
        if e.b(npc(e.w(0x4434aa)?) + 0xb0)? == 255 && e.b(npc(e.w(0x4434aa)?) + 0xb1)? == 1 {
            e.write(npc(e.w(0x4434aa)?) + 0xb0, &[1])?;
            e.ww(npc(e.w(0x4434aa)?) + 14, 0)?;
        }
        for offset in [0xd8, 0xcf, 0xb7, 0xb4, 0xb3, 0xb2, 0x105] {
            e.write(npc(e.w(0x4434aa)?) + offset, &[0])?;
        }
        if e.w(npc(e.w(0x4434aa)?) + 0x5a)? < 0 {
            e.ww(npc(e.w(0x4434aa)?) + 14, 0)?;
        }
        if e.w(npc(e.w(0x4434aa)?) + 0x110)? > 0
            && e.w(npc(e.w(0x4434aa)?) + 14)? == 0
            && e.w(npc(e.w(0x4434aa)?) + 0x5a)? != 0
        {
            e.ww(list_address(frame, count), e.w(0x4434aa)?.wrapping_add(1))?;
            count = count.wrapping_add(1);
        }
        e.addw(0x4434aa, 1)?;
    }
    if count == 0 {
        e.npc_menu()?;
        e.full(0);
        return Ok(e.plan);
    }
    e.runtime(0x13e13c)?;
    let dialog = e.trap(0xa97c, vec![W(173), L(0), L(u32::MAX)])?;
    e.lw(0x4394b8, dialog)?;
    e.trap(0xa873, vec![L(dialog)])?;
    e.trap(0xaa0b, vec![L(e.l(0x4358ce)?)])?;
    e.trap(0xa862, vec![L(69)])?;
    e.trap(0xa915, vec![L(dialog)])?;
    e.trap(0xa8a3, vec![P(dialog.wrapping_add(16))])?;
    e.trap(0xa981, vec![L(dialog)])?;
    e.trap(0xa862, vec![L(33)])?;
    e.trap(0xa863, vec![L(30)])?;
    e.trap(0xa88a, vec![W(10)])?;
    if e.w(0x439da8)? != 0 {
        e.sound(e.w(0x439da8)?, stack)?;
    }
    e.ww(0x4434aa, 0)?;
    while e.w(0x4434aa)? < count.wrapping_sub(1) {
        e.ww(0x4434ac, 0)?;
        while e.w(0x4434ac)? < count.wrapping_sub(1) {
            e.step()?;
            let n = e.w(0x4434ac)?;
            let left = e.w(list_address(frame, n))?;
            let right = e.w(list_address(frame, n.wrapping_add(1)))?;
            if e.w(npc(left.wrapping_sub(1)) + 0x110)? < e.w(npc(right.wrapping_sub(1)) + 0x110)?
                || e.w(npc(right.wrapping_sub(1)) + 0x5a)? == -1
            {
                e.ww(0x4434b6, left)?;
                e.ww(list_address(frame, e.w(0x4434ac)?), right)?;
                e.ww(
                    list_address(frame, e.w(0x4434ac)?.wrapping_add(1)),
                    e.w(0x4434b6)?,
                )?;
            }
            e.addw(0x4434ac, 1)?;
        }
        e.addw(0x4434aa, 1)?;
    }
    e.ww(0x4434aa, 0)?;
    while e.w(0x4434aa)? < 32 {
        e.step()?;
        if current_list(&e, frame)? != 0 {
            e.get_item(dialog, e.w(0x4434aa)?.wrapping_add(3), 0x44292c)?;
            e.button(stack, false, true)?;
            e.monster(
                stack.wrapping_sub(2),
                current_list(&e, frame)?.wrapping_sub(1),
                0x44292c,
            )?;
            let member = e
                .abs(current_list(&e, frame)?, stack.wrapping_sub(12))?
                .wrapping_sub(1);
            if e.w(npc(member) + 0x5a)? == -1 && selected < 18 {
                e.button(stack, true, true)?;
                let value = e.abs(current_list(&e, frame)?, stack.wrapping_sub(2))?;
                e.ww(list_address(frame, e.w(0x4434aa)?), value.wrapping_neg())?;
                selected = selected.wrapping_add(1);
            }
        }
        e.addw(0x4434aa, 1)?;
    }
    e.ww(0x4434aa, 0)?;
    while e.w(0x4434aa)? < 32 {
        e.step()?;
        if current_list(&e, frame)? < 0 {
            capacity = capacity.wrapping_add(1);
        }
        e.addw(0x4434aa, 1)?;
    }
    capacity = capacity.min(18);
    e.ww(0x4434aa, 0)?;
    while e.w(0x4434aa)? < 10 {
        e.step()?;
        if current_list(&e, frame)? > 0 {
            e.get_item(dialog, e.w(0x4434aa)?.wrapping_add(3), 0x44292c)?;
            if selected < capacity {
                e.button(stack, true, true)?;
                let value = e.abs(current_list(&e, frame)?, stack.wrapping_sub(2))?;
                e.ww(list_address(frame, e.w(0x4434aa)?), value.wrapping_neg())?;
                selected = selected.wrapping_add(1);
            }
        }
        e.addw(0x4434aa, 1)?;
    }
    let mut redraw = true;
    for _ in 0..limits.prompt.maximum_polls {
        e.step()?;
        if redraw {
            e.lw(0x4394b8, dialog)?;
            e.trap(0xa873, vec![L(dialog)])?;
            let sound = e.rand(4, stack)?.wrapping_add(29999);
            e.sound(sound, stack)?;
            e.trap(0xa862, vec![L(33)])?;
            e.trap(0xa863, vec![L(30)])?;
            e.trap(0xa922, vec![L(dialog)])?;
            e.trap(0xa923, vec![L(dialog)])?;
            survivor_cursor(&mut e, selected, capacity, true)?;
            e.first_text(stack, 0x220023)?;
            if e.plan.flow != LoadFlow::Returned {
                return Ok(e.plan);
            }
            redraw = false;
        }
        e.flush_events()?;
        e.trap(0xa991, vec![L(0), P(0x4434b2)])?;
        e.get_item(dialog, e.w(0x4434b2)?, 0x44292c)?;
        if e.w(0x4434b2)? == 2 {
            selected = selected.min(18);
            e.ww(0x44257c, selected)?;
            e.ww(0x4434aa, 0)?;
            while e.w(0x4434aa)? < 32 {
                e.step()?;
                if current_list(&e, frame)? < 0 {
                    selected = selected.wrapping_sub(1);
                    if selected >= 0 {
                        let from = npc(e.abs(current_list(&e, frame)?, stack)?.wrapping_sub(1));
                        let to = index(0x440edc, selected, 288);
                        e.copy_longs(from, to, 72)?;
                        e.ww(0x4434ac, 0)?;
                        while e.w(0x4434ac)? < 4 {
                            let a = to
                                .wrapping_add(0xb2)
                                .wrapping_add(i32::from(e.w(0x4434ac)?) as u32);
                            if e.b(a)? as i8 > 0 {
                                e.write(a, &[0])?;
                            }
                            e.addw(0x4434ac, 1)?;
                        }
                        let n = e.abs(current_list(&e, frame)?, stack)?.wrapping_sub(1);
                        e.ww(npc(n), 0)?;
                        let n = e
                            .abs(current_list(&e, frame)?, stack.wrapping_sub(2))?
                            .wrapping_sub(1);
                        e.ww(npc(n) + 0x110, 0)?;
                        e.ww(0x4434ac, 0)?;
                        while e.w(0x4434ac)? < 6 {
                            e.step()?;
                            let n = e.abs(current_list(&e, frame)?, stack)?.wrapping_sub(1);
                            e.ww(index(npc(n) + 0x26, e.w(0x4434ac)?, 2), 0)?;
                            e.addw(0x4434ac, 1)?;
                        }
                        e.ww(0x4434ac, 0)?;
                        while e.w(0x4434ac)? < 3 {
                            e.step()?;
                            let n = e.abs(current_list(&e, frame)?, stack)?.wrapping_sub(1);
                            e.ww(npc(n) + 0x3a, 0)?;
                            e.addw(0x4434ac, 1)?;
                        }
                    }
                }
                e.addw(0x4434aa, 1)?;
            }
            e.ww(0x4434aa, 0)?;
            while e.w(0x4434aa)? < 100 {
                e.step()?;
                e.ww(npc(e.w(0x4434aa)?) + 0x110, 0)?;
                e.addw(0x4434aa, 1)?;
            }
            e.trap(0xa914, vec![L(dialog)])?;
            e.npc_menu()?;
            e.full(0);
            return Ok(e.plan);
        }
        if e.w(0x4434b2)? > 2 && e.w(0x4434b2)? < 35 {
            let value = selected_list(&e, frame)?;
            if (selected > capacity.wrapping_sub(1) && value > 0) || value == 0 {
                survivor_notice(&mut e, stack, 0x404002)?;
                redraw = true;
                continue;
            }
            if value < 0 {
                let n = e.abs(value, stack)?.wrapping_sub(1);
                let important = e.w(npc(n) + 0x5a)? == -1;
                e.button(stack, false, true)?;
                e.sound(141, stack.wrapping_sub(2))?;
                let value = e.abs(selected_list(&e, frame)?, stack.wrapping_sub(4))?;
                e.ww(list_address(frame, e.w(0x4434b2)?.wrapping_sub(3)), value)?;
                selected = selected.wrapping_sub(1);
                if important {
                    survivor_notice(&mut e, stack.wrapping_sub(6), 0x403fb0)?;
                    redraw = true;
                    continue;
                }
            } else {
                if selected >= capacity {
                    survivor_notice(&mut e, stack, 0x404002)?;
                    redraw = true;
                    continue;
                }
                e.button(stack, true, true)?;
                e.sound(141, stack.wrapping_sub(2))?;
                let value = e.abs(selected_list(&e, frame)?, stack.wrapping_sub(4))?;
                e.ww(
                    list_address(frame, e.w(0x4434b2)?.wrapping_sub(3)),
                    value.wrapping_neg(),
                )?;
                selected = selected.wrapping_add(1);
            }
        }
        survivor_cursor(&mut e, selected, capacity, false)?;
    }
    Err(RuleError::Invalid(
        "bodycount dialog polling budget exhausted".into(),
    ))
}
