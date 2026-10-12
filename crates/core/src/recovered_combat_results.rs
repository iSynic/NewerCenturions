//! Damage resolution and combat result presentation from CODE 1.

use crate::recovered_ai::{SaveContext, SaveResult};
use crate::recovered_character_panels::{self as character, CharacterPanelEffect};
use crate::recovered_combat_center::{self as center, CombatCenterEffect};
use crate::recovered_combat_ui::{self as combat_ui, CombatUiEffect};
use crate::recovered_drawing::{
    self as drawing, BuildingWorld, DrawingContext, DrawingEffect, DrawingServices,
    PictureObservation,
};
use crate::recovered_feedback::{
    self as feedback, FeedbackArg, FeedbackEffect, FeedbackObservation, FeedbackRequest,
    FeedbackServices,
};
use crate::recovered_geometry::{
    self as geometry, BodyPosition, BodyRect, BodyRectServices, RectField,
};
use crate::recovered_load_control::LoadFlow;
use crate::recovered_prompt_control::{
    self as prompt, LibraryObservation, LibraryRequest, PromptControlEffect, PromptControlServices,
    PromptControlState, PromptLimits,
};
use crate::recovered_records::RecordWrite;
use crate::recovered_rendering::RenderingMemory;
use crate::recovered_rendering::{self as rendering, RenderingEffect};
use crate::recovered_runtime::{self as runtime, RuntimeEffect, RuntimeResult};
use crate::recovered_text::{self as text, NumericServices, PascalString, TextContext, TextEffect};
use crate::recovered_timing::{self as timing, TimingEffect, TimingServices};
use crate::{RandomState, RuleError, recovered_ai};
use FeedbackArg::{Long as L, Pointer as P, Word as W};
use centurions_content::{Warrior, combat::MonsterRecord};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CombatResultState {
    pub memory: RenderingMemory,
    pub random: RandomState,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CombatResultReturn {
    Full(u32),
    LowWord(u16),
    NoReturn,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphicsDispatchRequest {
    pub d0_input: u32,
    pub arguments: Vec<FeedbackArg>,
}
pub trait CombatResultServices: PromptControlServices {
    fn graphics_dispatch(
        &mut self,
        request: &GraphicsDispatchRequest,
        memory: &RenderingMemory,
    ) -> Result<FeedbackObservation, RuleError>;
}
#[derive(Clone, Debug, PartialEq)]
pub enum CombatResultEffect {
    Write(RecordWrite),
    Feedback(FeedbackEffect),
    Runtime(RuntimeEffect),
    Center(CombatCenterEffect),
    Save {
        selector: i16,
        body: i16,
        result: SaveResult,
    },
    Character(CharacterPanelEffect),
    Combat(CombatUiEffect),
    Rendering(RenderingEffect),
    Timing(TimingEffect),
    Prompt(PromptControlEffect),
    Library {
        request: LibraryRequest,
        observation: LibraryObservation,
    },
    GraphicsDispatch {
        request: GraphicsDispatchRequest,
        observation: FeedbackObservation,
    },
}
#[derive(Clone, Debug, PartialEq)]
pub struct CombatResultPlan {
    pub state: CombatResultState,
    pub effects: Vec<CombatResultEffect>,
    pub native_return: CombatResultReturn,
    pub flow: LoadFlow,
}
struct Execution<'a, S: ?Sized> {
    plan: CombatResultPlan,
    services: &'a mut S,
    limits: PromptLimits,
    remaining: u32,
}
impl<S: CombatResultServices + ?Sized> Execution<'_, S> {
    fn step(&mut self) -> Result<(), RuleError> {
        self.remaining = self.remaining.checked_sub(1).ok_or_else(|| {
            RuleError::Invalid("combat result processing budget exhausted".into())
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
    fn write(&mut self, a: u32, bytes: &[u8]) -> Result<(), RuleError> {
        for i in 0..bytes.len() {
            self.b(a.wrapping_add(i as u32))?;
        }
        self.plan.state.memory.insert_region(a, bytes);
        self.plan
            .effects
            .push(CombatResultEffect::Write(RecordWrite {
                address: a,
                bytes: bytes.to_vec(),
            }));
        Ok(())
    }
    fn ww(&mut self, a: u32, value: i16) -> Result<(), RuleError> {
        self.write(a, &value.to_be_bytes())
    }
    fn lw(&mut self, a: u32, value: u32) -> Result<(), RuleError> {
        self.write(a, &value.to_be_bytes())
    }
    fn addw(&mut self, a: u32, value: i16) -> Result<(), RuleError> {
        self.ww(a, self.w(a)?.wrapping_add(value))
    }
    fn rand(&mut self, bound: i16, stack: u32) -> Result<(i16, u32), RuleError> {
        self.step()?;
        self.ww(stack.wrapping_sub(2), bound)?;
        let mut context =
            runtime::RuntimeContext::new(self.plan.state.random.clone(), self.remaining as usize);
        let child = runtime::rand(&mut context, bound)?;
        self.plan.state.random = context.random;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(CombatResultEffect::Runtime));
        match child.result {
            RuntimeResult::Word { value, native_d0 } => Ok((value, native_d0)),
            _ => Err(RuleError::Invalid("unexpected Rand return".into())),
        }
    }
    fn range(&mut self, minimum: i16, maximum: i16, stack: u32) -> Result<i16, RuleError> {
        self.ww(stack.wrapping_sub(2), maximum)?;
        self.ww(stack.wrapping_sub(4), minimum)?;
        // Four argument bytes, LINK, and saved D3/D4 precede Rand's argument.
        let (value, _) = self.rand(
            maximum.wrapping_sub(minimum).wrapping_add(1),
            stack.wrapping_sub(20),
        )?;
        Ok(minimum.wrapping_add(value).wrapping_sub(1))
    }
    fn trap(&mut self, opcode: u16, arguments: Vec<FeedbackArg>) -> Result<u32, RuleError> {
        self.step()?;
        let request = FeedbackRequest::Toolbox { opcode, arguments };
        let observation =
            FeedbackServices::observe(self.services, &request, &self.plan.state.memory)?;
        let value = observation.value;
        self.plan.state.memory = observation.memory.clone();
        self.plan
            .effects
            .push(CombatResultEffect::Feedback(FeedbackEffect::Service {
                request,
                observation,
            }));
        Ok(value)
    }
    fn dispatch(&mut self, d0_input: u32, arguments: Vec<FeedbackArg>) -> Result<(), RuleError> {
        self.step()?;
        let request = GraphicsDispatchRequest {
            d0_input,
            arguments,
        };
        let observation = self
            .services
            .graphics_dispatch(&request, &self.plan.state.memory)?;
        self.plan.state.memory = observation.memory.clone();
        self.plan
            .effects
            .push(CombatResultEffect::GraphicsDispatch {
                request,
                observation,
            });
        Ok(())
    }
    fn sane(&mut self, source: u32, destination: u32, selector: u16) -> Result<(), RuleError> {
        self.trap(0xa9eb, vec![P(source), P(destination), W(selector)])?;
        Ok(())
    }
    fn sound(&mut self, id: i16, stack: u32) -> Result<(), RuleError> {
        self.ww(stack.wrapping_sub(2), id)?;
        let child = feedback::sound(&self.plan.state.memory, id, self.services)?;
        self.plan.state.memory = child.memory;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(CombatResultEffect::Feedback));
        Ok(())
    }
    fn center(&mut self, frame: u32, x: i16) -> Result<(), RuleError> {
        self.ww(frame.wrapping_add(8), x)?;
        let child = center::centerfield(
            &self.plan.state.memory,
            frame,
            self.remaining,
            self.services,
        )?;
        self.plan.state.memory = child.memory;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(CombatResultEffect::Center));
        Ok(())
    }
    fn flash(
        &mut self,
        frame: u32,
        message: u32,
        h: i16,
        v: i16,
        wait: i16,
        sound: i16,
    ) -> Result<(), RuleError> {
        self.lw(
            frame.wrapping_add(16),
            (u32::from(wait as u16) << 16) | u32::from(sound as u16),
        )?;
        self.lw(
            frame.wrapping_add(12),
            (u32::from(h as u16) << 16) | u32::from(v as u16),
        )?;
        self.lw(frame.wrapping_add(8), message)?;
        let child = feedback::flashmessage(
            &self.plan.state.memory,
            frame,
            message,
            h,
            v,
            wait,
            sound,
            self.limits.maximum_polls,
            self.services,
        )?;
        self.plan.state.memory = child.memory;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(CombatResultEffect::Feedback));
        Ok(())
    }
    fn save(&mut self, selector: i16, body: i16, stack: u32) -> Result<bool, RuleError> {
        self.step()?;
        self.ww(stack.wrapping_sub(2), body)?;
        self.ww(stack.wrapping_sub(4), selector)?;
        self.ww(stack.wrapping_sub(14), 0)?;
        let combat = self.w(0x438760)? != 0;
        let delivery_kind = if combat { self.b(0x443757)? as i8 } else { 0 };
        let disabled = combat && delivery_kind > 1;
        let context = SaveContext {
            combat,
            delivery_kind,
            modifier_factor: if disabled { 0 } else { self.b(0x443756)? as i8 },
            modifier_scale: if disabled { 0 } else { self.w(0x4435cc)? },
            modifier_base: if disabled { 0 } else { self.b(0x443754)? as i8 },
            resistance_bonus: false,
        };
        let mut warriors = vec![];
        let mut monsters = vec![];
        // Prefix records are adapter storage; save_against reads only the selected record.
        if !disabled {
            self.ww(stack.wrapping_sub(32), 100)?;
            if (0..9).contains(&body) {
                warriors.resize_with(body as usize + 1, || Warrior {
                    name: String::new(),
                    original_sha256: String::new(),
                    bytes: vec![],
                });
                warriors[body as usize].bytes = read(&self.plan.state.memory, warrior(body), 3138)?;
            } else if body > 9 {
                monsters.resize_with((body - 10) as usize + 1, || MonsterRecord { bytes: vec![] });
                monsters[(body - 10) as usize].bytes =
                    read(&self.plan.state.memory, enemy(body), 288)?;
            }
        }
        if !disabled && self.plan.state.random.draws == u64::MAX {
            return Err(RuleError::Invalid("random draw count exhausted".into()));
        }
        let child = recovered_ai::save_against(
            &warriors,
            &monsters,
            context,
            selector,
            body,
            &self.plan.state.random,
        )?;
        let saved = child.saved;
        self.plan.state.random = child.random.clone();
        self.plan.effects.push(CombatResultEffect::Save {
            selector,
            body,
            result: child,
        });
        Ok(saved)
    }
    fn observe(&mut self, request: FeedbackRequest) -> Result<u32, RuleError> {
        self.step()?;
        let observation =
            FeedbackServices::observe(self.services, &request, &self.plan.state.memory)?;
        let value = observation.value;
        self.plan.state.memory = observation.memory.clone();
        self.plan
            .effects
            .push(CombatResultEffect::Feedback(FeedbackEffect::Service {
                request,
                observation,
            }));
        Ok(value)
    }
    fn delay(&mut self, duration: i16, stack: u32) -> Result<(), RuleError> {
        self.ww(stack.wrapping_sub(2), duration)?;
        let default = if duration == 0 { self.w(0x439568)? } else { 0 };
        let remaining = self.limits.maximum_polls;
        let child = timing::delay(
            duration,
            default,
            &mut Clock {
                execution: self,
                remaining,
            },
        )?;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(CombatResultEffect::Timing));
        Ok(())
    }
    fn copy(
        &mut self,
        source: u32,
        destination: u32,
        source_rect: u32,
        destination_rect: u32,
    ) -> Result<(), RuleError> {
        self.trap(
            0xa8ec,
            vec![
                P(self.l(source)?.wrapping_add(2)),
                P(self.l(destination)?.wrapping_add(2)),
                P(source_rect),
                P(destination_rect),
                W(0),
                L(0),
            ],
        )?;
        Ok(())
    }
    fn body_rect(&mut self, body: i16, stack: u32) -> Result<(), RuleError> {
        self.ww(stack.wrapping_sub(2), body)?;
        let child = geometry::calcbodyrect(body, &MemoryPositions(&self.plan.state.memory))
            .map_err(|error| RuleError::Invalid(error.to_string()))?;
        for write in child.writes {
            let offset = match write.field {
                RectField::Top => 0,
                RectField::Left => 2,
                RectField::Bottom => 4,
                RectField::Right => 6,
            };
            self.ww(0x44371a + offset, write.value)?;
        }
        Ok(())
    }
    fn ground(&mut self, body: i16, stack: u32) -> Result<(), RuleError> {
        self.ww(stack.wrapping_sub(2), 0)?;
        self.ww(stack.wrapping_sub(4), body)?;
        let child = center::restore_body_ground(
            &self.plan.state.memory,
            body,
            0,
            self.remaining,
            self.services,
        )?;
        self.plan.state.memory = child.memory;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(CombatResultEffect::Center));
        Ok(())
    }
    fn body(&mut self, body: i16, force: i16, secondary: i16, stack: u32) -> Result<(), RuleError> {
        self.lw(
            stack.wrapping_sub(4),
            (u32::from(force as u16) << 16) | u32::from(secondary as u16),
        )?;
        self.ww(stack.wrapping_sub(6), body)?;
        let child = rendering::drawbody(
            &self.plan.state.memory,
            stack.wrapping_sub(14),
            body,
            force,
            secondary,
            self.services,
        )?;
        self.plan.state.memory = child.memory;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(CombatResultEffect::Rendering));
        Ok(())
    }
    fn character(&mut self, body: i16, mode: i16, stack: u32) -> Result<(), RuleError> {
        self.character_update(body, mode, stack, true)
    }
    fn character_update(
        &mut self,
        body: i16,
        mode: i16,
        stack: u32,
        short: bool,
    ) -> Result<(), RuleError> {
        self.ww(stack.wrapping_sub(2), mode)?;
        self.ww(stack.wrapping_sub(4), body)?;
        let update = if short {
            character::update_character_short
        } else {
            character::update_character
        };
        let child = update(
            &self.plan.state.memory,
            stack.wrapping_sub(12),
            self.remaining,
            self.services,
        )?;
        self.plan.state.memory = child.memory;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(CombatResultEffect::Character));
        Ok(())
    }
    fn icon(&mut self, id: i16, rect: u32, stack: u32) -> Result<(), RuleError> {
        self.lw(stack.wrapping_sub(4), self.l(rect.wrapping_add(4))?)?;
        self.lw(stack.wrapping_sub(8), self.l(rect)?)?;
        self.ww(stack.wrapping_sub(10), id)?;
        self.lw(0x439598, 0)?;
        let handle = self.trap(0xaa1e, vec![W(id as u16)])?;
        self.lw(0x439598, handle)?;
        let rect_pointer = stack.wrapping_sub(8);
        let bounds = self.plan.state.memory.rect(rect_pointer)?;
        let child = drawing::ploticon3(
            &mut icon_context(bounds),
            id,
            bounds,
            &mut FixedDrawing(handle),
        )?;
        if child.native_return != 0 {
            return Err(RuleError::Invalid("unexpected ploticon3 return".into()));
        }
        if self.l(0x439598)? != 0 {
            self.trap(0xaa1f, vec![P(rect_pointer), L(self.l(0x439598)?)])?;
            self.trap(0xaa25, vec![L(self.l(0x439598)?)])?;
        }
        Ok(())
    }
    fn lib(&mut self, entry: u32, arguments: Vec<FeedbackArg>) -> Result<u32, RuleError> {
        self.step()?;
        let request = LibraryRequest { entry, arguments };
        let observation = self.services.library(&request, &self.plan.state.memory)?;
        let d0 = observation.d0;
        self.plan.state.memory = observation.memory.clone();
        self.plan.effects.push(CombatResultEffect::Library {
            request,
            observation,
        });
        Ok(d0)
    }
    fn abs(&mut self, value: i16, stack: u32) -> Result<i16, RuleError> {
        self.ww(stack.wrapping_sub(2), value)?;
        Ok(self.lib(0x400038, vec![W(value as u16)])? as i16)
    }
    fn string(&mut self, value: i16, stack: u32) -> Result<(), RuleError> {
        self.ww(stack.wrapping_sub(2), value)?;
        self.lib(0x400168, vec![L(i32::from(value) as u32), P(0x479ed8)])?;
        let bytes = read(
            &self.plan.state.memory,
            0x479ed8,
            usize::from(self.b(0x479ed8)?) + 1,
        )?;
        let observed = PascalString::from_raw(&bytes)?;
        let child = text::string(
            &mut TextContext::default(),
            value,
            &mut FixedNumber(observed),
        )?;
        if !matches!(
            child.as_slice(),
            [TextEffect::NumberToString(_), TextEffect::DrawString(_)]
        ) {
            return Err(RuleError::Invalid("unexpected string child effects".into()));
        }
        self.trap(0xa884, vec![P(0x479ed8)])?;
        Ok(())
    }
    fn indexed_string(&mut self, index: i16) -> Result<(), RuleError> {
        self.lib(0x400178, vec![P(0x479ed8), W(134), W(index as u16)])?;
        self.trap(0xa884, vec![P(0x479ed8)])?;
        Ok(())
    }
    fn pict(&mut self, id: i16, rect: u32, stack: u32) -> Result<(), RuleError> {
        self.lw(stack.wrapping_sub(4), self.l(rect.wrapping_add(4))?)?;
        self.lw(stack.wrapping_sub(8), self.l(rect)?)?;
        self.ww(stack.wrapping_sub(10), id)?;
        let handle = self.trap(0xa9bc, vec![W(id as u16)])?;
        self.lw(stack.wrapping_sub(22), handle)?;
        let rect = self.plan.state.memory.rect(stack.wrapping_sub(8))?;
        let child = drawing::pict(id, rect, &mut FixedDrawing(handle))?;
        if !matches!(
            child.as_slice(),
            [
                DrawingEffect::GetPicture(_),
                DrawingEffect::DrawPicture { .. }
            ]
        ) {
            return Err(RuleError::Invalid("unexpected pict child effects".into()));
        }
        self.trap(
            0xa8f6,
            vec![L(self.l(stack.wrapping_sub(22))?), P(stack.wrapping_sub(8))],
        )?;
        Ok(())
    }
    fn prompt(&mut self, frame: u32, reset: bool) -> Result<(), RuleError> {
        let state = PromptControlState {
            memory: self.plan.state.memory.clone(),
            random: self.plan.state.random.clone(),
        };
        let limits = PromptLimits {
            maximum_steps: self.remaining,
            ..self.limits
        };
        let child = if reset {
            prompt::reset_text_area(&state, frame, limits, self.services)?
        } else {
            prompt::update_controls(&state, frame, limits, self.services)?
        };
        self.plan.state.memory = child.state.memory;
        self.plan.state.random = child.state.random;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(CombatResultEffect::Prompt));
        self.plan.flow = child.flow;
        if child.flow != LoadFlow::Returned {
            self.plan.native_return = CombatResultReturn::NoReturn;
        }
        Ok(())
    }
    fn reset(&mut self, stack: u32) -> Result<(), RuleError> {
        self.ww(stack.wrapping_sub(2), 0)?;
        self.prompt(stack.wrapping_sub(10), true)
    }
    fn show_damage(
        &mut self,
        body: i16,
        icon: i16,
        amount: i16,
        stack: u32,
    ) -> Result<(), RuleError> {
        self.ww(stack.wrapping_sub(2), amount)?;
        self.ww(stack.wrapping_sub(4), icon)?;
        self.ww(stack.wrapping_sub(6), body)?;
        let child =
            combat_ui::showdamage(&self.plan.state.memory, body, icon, amount, self.services)?;
        self.plan.state.memory = child.memory;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(CombatResultEffect::Combat));
        Ok(())
    }
}
fn execution<'a, S: ?Sized>(
    state: &CombatResultState,
    limits: PromptLimits,
    services: &'a mut S,
) -> Execution<'a, S> {
    Execution {
        plan: CombatResultPlan {
            state: state.clone(),
            effects: vec![],
            native_return: CombatResultReturn::Full(0),
            flow: LoadFlow::Returned,
        },
        services,
        limits,
        remaining: limits.maximum_steps,
    }
}
fn warrior(body: i16) -> u32 {
    0x448aa0_u32.wrapping_add(u32::from(body as u16) * 3138)
}
fn enemy(body: i16) -> u32 {
    0x439d3c_u32.wrapping_add(u32::from(body.wrapping_sub(10) as u16) * 288)
}
fn idx(a: u32, n: i16, stride: i32) -> u32 {
    a.wrapping_add((i32::from(n) * stride) as u32)
}
fn read(memory: &RenderingMemory, a: u32, length: usize) -> Result<Vec<u8>, RuleError> {
    (0..length)
        .map(|i| memory.byte(a.wrapping_add(i as u32)))
        .collect()
}
fn damage(frame: u32, channel: u32) -> u32 {
    frame.wrapping_sub(20).wrapping_add(channel * 2)
}

struct Clock<'a, 'b, S: ?Sized> {
    execution: &'a mut Execution<'b, S>,
    remaining: usize,
}
impl<S: CombatResultServices + ?Sized> TimingServices for Clock<'_, '_, S> {
    fn tick_count(&mut self) -> Result<u32, RuleError> {
        self.remaining = self.remaining.checked_sub(1).ok_or_else(|| {
            RuleError::Invalid("combat result delay observation budget exhausted".into())
        })?;
        self.execution.observe(FeedbackRequest::TickCount)
    }
    fn button(&mut self) -> Result<bool, RuleError> {
        Ok(self.execution.observe(FeedbackRequest::Button)? as u8 != 0)
    }
}
struct MemoryPositions<'a>(&'a RenderingMemory);
impl BodyRectServices for MemoryPositions<'_> {
    fn warrior_position(&self, body: i16) -> Option<BodyPosition> {
        Some(BodyPosition {
            horizontal: self.0.word(idx(0x44709a, body, 4)).ok()? as i16,
            vertical: self.0.word(idx(0x44709c, body, 4)).ok()? as i16,
        })
    }
    fn enemy_position(&self, n: i16) -> Option<BodyPosition> {
        Some(BodyPosition {
            horizontal: self.0.word(idx(0x446f0a, n, 4)).ok()? as i16,
            vertical: self.0.word(idx(0x446f0c, n, 4)).ok()? as i16,
        })
    }
    fn enemy_size(&self, n: u16) -> Option<i16> {
        self.0
            .word(0x439d4c_u32.wrapping_add(u32::from(n) * 288))
            .ok()
            .map(|v| v as i16)
    }
}
struct FixedDrawing(u32);
struct FixedNumber(PascalString);
impl NumericServices for FixedNumber {
    fn number_to_string(&mut self, _: i32) -> Result<PascalString, RuleError> {
        Ok(self.0.clone())
    }
    fn string_to_number(&mut self, _: &PascalString) -> Result<i32, RuleError> {
        Err(RuleError::Invalid(
            "unused combat result numeric parse".into(),
        ))
    }
}
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
        Err(RuleError::Invalid("unused combat result string".into()))
    }
    fn current_world(&mut self) -> Result<(u32, u32), RuleError> {
        Err(RuleError::Invalid("unused combat result world".into()))
    }
    fn new_world(&mut self, _: i16, _: BodyRect) -> Result<(i16, BuildingWorld), RuleError> {
        Err(RuleError::Invalid("unused combat result allocation".into()))
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

/// spelleffect, 0x15b2f6. Animation frame and body loops retain signed WORD bounds.
pub fn spelleffect(
    state: &CombatResultState,
    frame: u32,
    limits: PromptLimits,
    services: &mut (impl CombatResultServices + ?Sized),
) -> Result<CombatResultPlan, RuleError> {
    let mut e = execution(state, limits, services);
    let stack = frame.wrapping_sub(36);
    e.ww(frame.wrapping_sub(14), 0)?;
    e.ww(frame.wrapping_sub(16), 0)?;
    e.ww(frame.wrapping_sub(10), 64)?;
    e.ww(frame.wrapping_sub(12), 64)?;
    let animation = e.b(0x443760)?;
    let first = if animation == 0 {
        0x2ef8
    } else {
        i16::from(animation as i8)
            .wrapping_shl(3)
            .wrapping_add(0x2ed8)
    };
    let end = first.wrapping_add(8);
    if e.w(0x438776)? == 0 {
        let top = e.w(frame.wrapping_add(8))?.wrapping_mul(50).wrapping_add(9);
        e.ww(0x44371a, top)?;
        e.ww(0x44371e, e.w(0x44371a)?.wrapping_add(32))?;
        e.ww(0x44371c, e.w(0x40856c)?.wrapping_add(339))?;
        e.ww(0x443720, e.w(0x44371c)?.wrapping_add(32))?;
        let port = if e.w(0x44719a)? != 0 && e.b(0x442750)? != 0 {
            e.l(0x438798)?
        } else {
            e.l(0x43878c)?
        };
        e.trap(0xa873, vec![L(port)])?;
        e.trap(0xa862, vec![L(33)])?;
        e.trap(0xa863, vec![L(30)])?;
        e.copy(0x3fe302, 0x4386b4, 0x44371a, frame.wrapping_sub(16))?;
        let mut id = first;
        while id < end {
            e.step()?;
            e.copy(0x4386b4, 0x3fe302, frame.wrapping_sub(16), 0x44371a)?;
            e.icon(id, 0x44371a, stack)?;
            e.delay(
                (e.w(0x439568)? / 30).wrapping_add(1),
                stack.wrapping_sub(10),
            )?;
            id = id.wrapping_add(1);
        }
        e.character_update(e.w(frame.wrapping_add(8))?, 3, stack, false)?;
        return Ok(e.plan);
    }
    if e.w(frame.wrapping_add(10))? == 0 || e.b(0x4358c1)? == 0 {
        e.trap(0xa873, vec![L(e.l(0x43879c)?)])?;
        e.body_rect(e.w(frame.wrapping_add(8))?, stack)?;
        e.ww(
            frame.wrapping_sub(12),
            e.w(0x44371e)?.wrapping_sub(e.w(0x44371a)?),
        )?;
        e.ww(
            frame.wrapping_sub(10),
            e.w(0x443720)?.wrapping_sub(e.w(0x44371c)?),
        )?;
        e.copy(0x43879c, 0x4386b4, 0x44371a, frame.wrapping_sub(16))?;
        // The calcbodyrect argument is still present during both children.
        e.ground(e.w(frame.wrapping_add(8))?, stack.wrapping_sub(2))?;
        e.body(e.w(frame.wrapping_add(8))?, 0, 0, stack.wrapping_sub(6))?;
        let kind = e.b(0x44374f)?;
        if (kind == 9 || kind == 10 || (e.w(0x439588)? != 0 && kind == 3)) && e.b(0x4358c2)? == 0 {
            e.body(e.w(frame.wrapping_add(8))?, 1, 0, stack)?;
            if e.w(0x439588)? == 0 {
                e.delay(10, stack)?;
            }
            e.body(e.w(frame.wrapping_add(8))?, 0, 0, stack)?;
            return Ok(e.plan);
        }
        let mut id = first;
        while id < end {
            e.step()?;
            if e.trap(
                0xa8aa,
                vec![P(0x44371a), P(0x4471b4), P(frame.wrapping_sub(8))],
            )? as u8
                != 0
            {
                e.copy(0x4386b4, 0x43879c, frame.wrapping_sub(16), 0x44371a)?;
            }
            e.icon(id, 0x44371a, stack)?;
            e.delay(
                (e.w(0x439568)? / 30).wrapping_add(1),
                stack.wrapping_sub(10),
            )?;
            id = id.wrapping_add(1);
        }
        if e.trap(
            0xa8aa,
            vec![P(0x44371a), P(0x4471b4), P(frame.wrapping_sub(8))],
        )? as u8
            != 0
        {
            e.copy(0x4386b4, 0x43879c, frame.wrapping_sub(16), 0x44371a)?;
        }
        return Ok(e.plan);
    }
    // Native redundant gate is retained because service/child writes can alter globals.
    if e.w(frame.wrapping_add(10))? == 0 && e.b(0x4358c1)? == 0 {
        return Ok(e.plan);
    }
    e.trap(0xa873, vec![L(e.l(0x43879c)?)])?;
    e.trap(0xa862, vec![L(33)])?;
    e.trap(0xa863, vec![L(30)])?;
    e.copy(0x43879c, 0x4386bc, 0x4471b4, 0x4471b4)?;
    let mut id = first;
    while id < end {
        e.step()?;
        let handle = e.trap(0xaa1e, vec![W(id as u16)])?;
        e.lw(0x439598, handle)?;
        e.copy(0x4386bc, 0x4386c0, 0x4471b4, 0x4471b4)?;
        let mut first_draw = false;
        for body in 0..110_i16 {
            e.step()?;
            if e.w(idx(0x4434e0, body, 2))? == 0 {
                continue;
            }
            if body > 9 {
                let size = e.w(enemy(body).wrapping_add(16))?;
                let size = e.abs(size, stack)?;
                e.ww(0x4435ce, size)?;
                e.ww(
                    0x44371a,
                    e.w(idx(0x446f0c, body.wrapping_sub(10), 4))?
                        .wrapping_shl(5),
                )?;
                e.ww(
                    0x44371c,
                    e.w(idx(0x446f0a, body.wrapping_sub(10), 4))?
                        .wrapping_shl(5),
                )?;
            } else {
                e.ww(0x4435ce, 0)?;
                e.ww(0x44371a, e.w(idx(0x44709c, body, 4))?.wrapping_shl(5))?;
                e.ww(0x44371c, e.w(idx(0x44709a, body, 4))?.wrapping_shl(5))?;
            }
            e.ww(0x44371e, e.w(0x44371a)?.wrapping_add(32))?;
            e.ww(0x443720, e.w(0x44371c)?.wrapping_add(32))?;
            if matches!(e.w(0x4435ce)?, 1 | 3) {
                e.addw(0x44371a, -32)?;
            }
            if e.w(0x4435ce)? > 1 {
                e.addw(0x44371c, -32)?;
            }
            if e.trap(
                0xa8aa,
                vec![P(0x44371a), P(0x4471b4), P(frame.wrapping_sub(8))],
            )? as u8
                == 0
            {
                continue;
            }
            if !first_draw {
                e.trap(0xaa1f, vec![P(0x44371a), L(e.l(0x439598)?)])?;
                first_draw = true;
            }
            e.dispatch(0x80005, vec![P(0x4046da), P(0x4046de)])?;
            e.dispatch(0x80006, vec![L(e.l(0x4386c0)?), L(0)])?;
            e.trap(0xaa1f, vec![P(0x44371a), L(e.l(0x439598)?)])?;
            e.dispatch(0x80006, vec![L(e.l(0x4046da)?), L(e.l(0x4046de)?)])?;
        }
        e.copy(0x4386c0, 0x43879c, 0x4471b4, 0x4471b4)?;
        e.trap(0xaa25, vec![L(e.l(0x439598)?)])?;
        if e.b(0x4358c2)? == 0 {
            e.delay(4, stack)?;
        }
        id = id.wrapping_add(1);
    }
    e.copy(0x4386bc, 0x43879c, 0x4471b4, 0x4471b4)?;
    Ok(e.plan)
}

/// resolvedamage, 0x15075a. Caller flags are read again at each native branch.
pub fn resolvedamage(
    state: &CombatResultState,
    frame: u32,
    limits: PromptLimits,
    services: &mut (impl CombatResultServices + ?Sized),
) -> Result<CombatResultPlan, RuleError> {
    let mut e = execution(state, limits, services);
    let stack = frame.wrapping_sub(376);
    e.write(frame.wrapping_sub(25), &[0])?;
    if e.w(frame.wrapping_add(8))? < 0 {
        return Ok(e.plan);
    }
    if e.w(frame.wrapping_add(8))? > 9 {
        let mut source = enemy(e.w(frame.wrapping_add(8))?);
        let mut destination = frame.wrapping_sub(338);
        for _ in 0..72 {
            let value = e.l(source)?;
            e.lw(destination, value)?;
            source = source.wrapping_add(4);
            destination = destination.wrapping_add(4);
        }
    }
    for channel in 0..9 {
        e.ww(frame.wrapping_sub(46).wrapping_add(channel * 2), 0)?;
        e.ww(damage(frame, channel), 0)?;
    }
    if e.w(0x443770)? != 0 {
        let value = e.range(e.w(0x44376e)?, e.w(0x443770)?, stack)?;
        e.ww(damage(frame, 0), value)?;
    }
    let mut n = 0_i16;
    while n < e.w(0x4435cc)? {
        e.step()?;
        if e.b(0x44375a)? != 0 {
            let value = e.range(
                i16::from(e.b(0x443759)? as i8),
                i16::from(e.b(0x44375a)? as i8),
                stack,
            )?;
            e.addw(damage(frame, 1), value)?;
        }
        n = n.wrapping_add(1);
    }
    for channel in 2..9 {
        let bound = e.b(0x443765 + channel - 2)?;
        if bound != 0 {
            let (value, _) = e.rand(i16::from(bound), stack)?;
            e.ww(damage(frame, channel), value)?;
        }
    }
    if e.b(0x443778)? != 0 && e.w(frame.wrapping_add(8))? > 9 {
        e.ww(frame.wrapping_sub(24), 0)?;
        while e.w(frame.wrapping_sub(24))? < 9 {
            e.step()?;
            let size = e.abs(
                e.w(enemy(e.w(frame.wrapping_add(8))?).wrapping_add(16))?,
                stack,
            )?;
            let address = damage(frame, e.w(frame.wrapping_sub(24))? as u32);
            e.ww(address, e.w(address)?.wrapping_mul(size))?;
            e.addw(frame.wrapping_sub(24), 1)?;
        }
    }
    if e.w(frame.wrapping_add(16))? != 0 {
        let body = e.w(frame.wrapping_add(8))?;
        let threshold = if body < 9 {
            e.w(warrior(body).wrapping_add(44))?
        } else {
            e.w(enemy(body).wrapping_add(8))?
        }
        .min(95);
        for channel in 0..9 {
            e.ww(frame.wrapping_sub(24), 0)?;
            while e.w(frame.wrapping_sub(24))? < e.w(damage(frame, channel))? {
                let (roll, _) = e.rand(100, stack)?;
                if roll > threshold {
                    e.addw(frame.wrapping_sub(46).wrapping_add(channel * 2), 1)?;
                }
                e.addw(frame.wrapping_sub(24), 1)?;
            }
            e.ww(
                damage(frame, channel),
                e.w(frame.wrapping_sub(46).wrapping_add(channel * 2))?,
            )?;
        }
    }
    if e.w(frame.wrapping_add(8))? > 9
        && e.b(enemy(e.w(frame.wrapping_add(8))?).wrapping_add(176))? == 255
    {
        e.ww(frame.wrapping_add(10), 0)?;
    }
    if e.w(0x443776)? == 57 {
        e.ww(0x4434b8, 0)?;
        for channel in 0..9 {
            e.addw(0x4434b8, e.w(damage(frame, channel))?)?;
        }
        e.plan.native_return = CombatResultReturn::LowWord(e.w(0x4434b8)?.wrapping_neg() as u16);
        return Ok(e.plan);
    }
    if e.w(frame.wrapping_add(10))? != 0 {
        let body = e.w(frame.wrapping_add(8))?;
        if body < 9 {
            if e.w(warrior(body).wrapping_add(38))? != 0 {
                e.ww(
                    frame.wrapping_sub(22),
                    e.w(warrior(e.w(frame.wrapping_add(8))?).wrapping_add(36))?,
                )?;
                let mut energy = e.w(warrior(e.w(frame.wrapping_add(8))?).wrapping_add(38))?;
                let cost = e.w(warrior(e.w(frame.wrapping_add(8))?).wrapping_add(42))?;
                if e.w(frame.wrapping_sub(22))? > 95 {
                    e.ww(frame.wrapping_sub(22), 95)?;
                }
                if energy >= cost {
                    'shield: for channel in 0..9 {
                        let count = e.w(damage(frame, channel))?;
                        let mut n = 0_i16;
                        while n < count {
                            let (roll, _) = e.rand(100, stack)?;
                            if roll < e.w(frame.wrapping_sub(22))?
                                && energy >= cost
                                && e.w(damage(frame, channel))? > 1
                            {
                                e.addw(damage(frame, channel), -1)?;
                                energy = energy.wrapping_sub(cost);
                                e.write(frame.wrapping_sub(25), &[1])?;
                            } else if energy < cost {
                                let body = e.w(frame.wrapping_add(8))?;
                                // Four centering argument bytes remain below the parent's saved registers.
                                let y = e.w(idx(0x44709c, body, 4))?;
                                e.ww(stack.wrapping_sub(2), y)?;
                                let x = e.w(idx(0x44709a, e.w(frame.wrapping_add(8))?, 4))?;
                                e.center(stack.wrapping_sub(12), x)?;
                                e.flash(stack.wrapping_sub(24), 0x401898, 30, 30, 80, 133)?;
                                break 'shield;
                            }
                            n = n.wrapping_add(1);
                        }
                    }
                    e.ww(
                        warrior(e.w(frame.wrapping_add(8))?).wrapping_add(38),
                        energy,
                    )?;
                    e.ww(
                        warrior(e.w(frame.wrapping_add(8))?).wrapping_add(352),
                        energy,
                    )?;
                }
            }
        } else {
            e.ww(frame.wrapping_sub(22), e.w(enemy(body).wrapping_add(86))?)?;
            let mut energy = e.w(enemy(e.w(frame.wrapping_add(8))?).wrapping_add(88))?;
            let mut cost =
                i16::from(e.b(enemy(e.w(frame.wrapping_add(8))?).wrapping_add(266))? as i8);
            if e.w(frame.wrapping_sub(22))? > 95 {
                e.ww(frame.wrapping_sub(22), 95)?;
            }
            if e.w(frame.wrapping_sub(22))? != 0 && energy != 0 && cost == 0 {
                cost = 1;
            }
            for channel in 0..9 {
                let count = e.w(damage(frame, channel))?;
                let mut n = 0_i16;
                while n < count {
                    let (roll, _) = e.rand(100, stack)?;
                    if roll < e.w(frame.wrapping_sub(22))?
                        && energy >= cost
                        && e.w(damage(frame, channel))? > 1
                    {
                        e.addw(damage(frame, channel), -1)?;
                        energy = energy.wrapping_sub(cost);
                        e.write(frame.wrapping_sub(25), &[1])?;
                    }
                    n = n.wrapping_add(1);
                }
            }
            e.ww(enemy(e.w(frame.wrapping_add(8))?).wrapping_add(88), energy)?;
        }
    }
    if e.b(frame.wrapping_sub(25))? == 1 {
        e.sound(206, stack)?;
    }
    if e.w(frame.wrapping_add(12))? != 0 {
        let body = e.w(frame.wrapping_add(8))?;
        e.ww(
            frame.wrapping_sub(22),
            if body < 9 {
                e.w(warrior(body).wrapping_add(44))?
            } else {
                e.w(enemy(body).wrapping_add(8))?
            },
        )?;
        if e.w(frame.wrapping_sub(22))? > 95 {
            e.ww(frame.wrapping_sub(22), 95)?;
        }
        for channel in 0..9 {
            let count = if channel == 6 {
                0
            } else {
                e.w(damage(frame, channel))?
            };
            let mut n = 0_i16;
            while n < count {
                let (roll, _) = e.rand(100, stack)?;
                if roll < e.w(frame.wrapping_sub(22))? && e.w(damage(frame, channel))? > 1 {
                    e.addw(damage(frame, channel), -1)?;
                }
                n = n.wrapping_add(1);
            }
        }
    }
    if e.w(frame.wrapping_add(14))? != 0 {
        if (e.b(0x443757)? as i8) < 2 {
            for channel in 2..9 {
                if e.w(damage(frame, channel))? != 0
                    && e.save(channel as i16 - 1, e.w(frame.wrapping_add(8))?, stack)?
                {
                    e.ww(damage(frame, channel), e.w(damage(frame, channel))? / 2)?;
                }
            }
        }
        if e.w(frame.wrapping_add(8))? < 9 {
            for selector in 9..16 {
                if e.w(warrior(e.w(frame.wrapping_add(8))?).wrapping_add(94 + selector * 2))? != 0 {
                    e.ww(
                        damage(frame, selector - 7),
                        e.w(damage(frame, selector - 7))? / 2,
                    )?;
                }
            }
            if e.w(warrior(e.w(frame.wrapping_add(8))?).wrapping_add(50))? == 3 {
                e.ww(damage(frame, 8), 0)?;
                e.addw(damage(frame, 5), e.w(damage(frame, 5))?)?;
            }
        } else {
            for selector in 9..16 {
                if e.b(enemy(e.w(frame.wrapping_add(8))?).wrapping_add(178 + selector))? != 0 {
                    e.ww(
                        damage(frame, selector - 7),
                        e.w(damage(frame, selector - 7))? / 2,
                    )?;
                }
            }
            for channel in 2..9 {
                let captured = frame.wrapping_sub(338).wrapping_add(139 + channel - 1);
                if e.w(damage(frame, channel))? != 0 && (e.b(captured)? as i8) < 0 {
                    let magnitude = e.abs(i16::from(e.b(captured)? as i8), stack)?;
                    e.ww(frame.wrapping_sub(356), magnitude)?;
                    e.sane(frame.wrapping_sub(356), frame.wrapping_sub(356), 0x200e)?;
                    e.sane(frame.wrapping_sub(356), frame.wrapping_sub(356), 0x1010)?;
                    e.lw(frame.wrapping_sub(50), e.l(frame.wrapping_sub(356))?)?;
                    e.lw(frame.wrapping_sub(346), 100.0_f32.to_bits())?;
                    e.sane(frame.wrapping_sub(50), frame.wrapping_sub(356), 0x100e)?;
                    e.sane(frame.wrapping_sub(346), frame.wrapping_sub(356), 0x1006)?;
                    e.sane(frame.wrapping_sub(356), frame.wrapping_sub(50), 0x1010)?;
                    e.lw(frame.wrapping_sub(346), 1.0_f32.to_bits())?;
                    e.sane(frame.wrapping_sub(50), frame.wrapping_sub(356), 0x100e)?;
                    e.sane(frame.wrapping_sub(346), frame.wrapping_sub(356), 0x1000)?;
                    e.sane(frame.wrapping_sub(356), frame.wrapping_sub(50), 0x1010)?;
                    // The binary uses the prior global total, not this channel's damage.
                    e.lw(frame.wrapping_sub(356), i32::from(e.w(0x4434b8)?) as u32)?;
                    e.sane(frame.wrapping_sub(356), frame.wrapping_sub(356), 0x280e)?;
                    e.sane(frame.wrapping_sub(50), frame.wrapping_sub(356), 0x1004)?;
                    e.sane(frame.wrapping_sub(356), frame.wrapping_sub(346), 0x1010)?;
                    e.lw(frame.wrapping_sub(342), e.l(frame.wrapping_sub(346))?)?;
                    e.sane(frame.wrapping_sub(342), frame.wrapping_sub(356), 0x100e)?;
                    e.trap(0xa9eb, vec![P(frame.wrapping_sub(356)), W(0x16)])?;
                    e.sane(frame.wrapping_sub(356), frame.wrapping_sub(356), 0x2810)?;
                    e.ww(damage(frame, channel), e.w(frame.wrapping_sub(354))?)?;
                }
                if (e.b(captured)? as i8) > 100 {
                    e.ww(damage(frame, channel), 0)?;
                }
            }
            if e.b(frame.wrapping_sub(167))? != 0 {
                e.ww(damage(frame, 8), 0)?;
            }
        }
    }
    e.ww(0x41f710, 0)?;
    e.ww(0x4434b8, 0)?;
    let icons = [157, 157, 159, 160, 161, 162, 163, 164, 159];
    for channel in 0..9 {
        e.addw(0x4434b8, e.w(damage(frame, channel))?)?;
        if e.w(damage(frame, channel))? != 0 && e.w(0x41f710)? == 0 {
            let body = e.w(frame.wrapping_add(8))?;
            if body < 9 {
                e.ww(warrior(body).wrapping_add(20), 1)?;
            } else {
                e.write(enemy(body).wrapping_add(261), &[1])?;
            }
            e.ww(0x41f710, icons[channel as usize])?;
        }
    }
    e.plan.native_return = CombatResultReturn::LowWord(e.w(0x4434b8)? as u16);
    Ok(e.plan)
}

/// showresults, 0x158454. The CODE 1 jump table contains 81 WORD entries.
pub fn showresults(
    state: &CombatResultState,
    frame: u32,
    limits: PromptLimits,
    services: &mut (impl CombatResultServices + ?Sized),
) -> Result<CombatResultPlan, RuleError> {
    let mut e = execution(state, limits, services);
    let stack = frame.wrapping_sub(28);
    let body = e.w(frame.wrapping_add(8))?;
    // Some native callers push only two WORDs. Their inherited third WORD is real input.
    let attacker = e.w(frame.wrapping_add(12))?;
    let mut icon = -1_i16;
    e.lw(0x439598, 0)?;
    e.ww(0x44331a, 0)?;
    e.ww(0x438786, 0)?;
    e.trap(0xa873, vec![L(e.l(0x43878c)?)])?;
    if e.w(0x438776)? == 0 {
        e.reset(stack)?;
        if e.plan.flow != LoadFlow::Returned {
            return Ok(e.plan);
        }
    } else {
        if e.w(0x4434b8)? != 0 || e.w(0x442bc8)? != 0 {
            e.trap(0xa852, vec![])?;
        }
        e.pict(156, 0x44370a, stack)?;
        e.lw(frame.wrapping_sub(8), e.l(0x4436fa)?)?;
        e.lw(frame.wrapping_sub(4), e.l(0x4436fe)?)?;
        if !matches!(e.b(0x44374f)?, 9 | 10 | 6) {
            e.ww(stack.wrapping_sub(2), body)?;
            let child = center::center(
                &e.plan.state.memory,
                stack.wrapping_sub(10),
                e.remaining,
                e.services,
            )?;
            e.plan.state.memory = child.memory;
            e.plan
                .effects
                .extend(child.effects.into_iter().map(CombatResultEffect::Center));
        }
        if body <= 9 {
            e.ww(0x4435ce, 0)?;
            let handle = e.trap(0xaa1e, vec![W(e.w(warrior(body).wrapping_add(86))? as u16)])?;
            e.lw(0x439598, handle)?;
        } else {
            e.ww(0x4434b6, e.w(enemy(body).wrapping_add(80))?)?;
            if e.b(enemy(body).wrapping_add(263))? == 1 {
                e.addw(0x4434b6, 308)?;
            }
            let handle = e.trap(0xaa1e, vec![W(e.w(0x4434b6)? as u16)])?;
            e.lw(0x439598, handle)?;
            let size = e.abs(e.w(enemy(body).wrapping_add(16))?, stack)?;
            e.ww(0x4435ce, size)?;
        }
        if matches!(e.w(0x4435ce)?, 2 | 0) {
            e.addw(frame.wrapping_sub(8), 16)?;
            e.addw(frame.wrapping_sub(4), -16)?;
        }
        if matches!(e.w(0x4435ce)?, 1 | 0) {
            e.addw(frame.wrapping_sub(6), 16)?;
            e.addw(frame.wrapping_sub(2), -16)?;
        }
        e.trap(0xa8a3, vec![P(0x4436fa)])?;
        if e.l(0x439598)? != 0 {
            e.trap(0xaa1f, vec![P(frame.wrapping_sub(8)), L(e.l(0x439598)?)])?;
            e.trap(0xaa25, vec![L(e.l(0x439598)?)])?;
        }
    }
    e.trap(0xa889, vec![W(1)])?;
    e.trap(0xa88a, vec![W(20)])?;
    let vertical = if e.w(0x438776)? == 0 {
        30
    } else {
        e.w(0x40856c)?.wrapping_add(420)
    };
    let horizontal = e
        .w(0x40856e)?
        .wrapping_add(if e.w(0x438776)? == 0 { 400 } else { 443 })
        .wrapping_sub(e.w(0x439582)?.wrapping_shl(5));
    e.trap(0xa893, vec![W(vertical as u16), W(horizontal as u16)])?;
    e.trap(0xa862, vec![L(205)])?;
    let code = e.w(frame.wrapping_add(10))?;
    match code {
        0 => {
            let amount = e.abs(e.w(0x4434b8)?, stack)?;
            e.string(amount, stack)?;
            e.trap(0xa862, vec![L(69)])?;
            if e.w(0x438784)? != 0 {
                e.sound(104, stack)?;
                e.trap(0xa884, vec![P(0x400ee6)])?;
                e.ww(0x442bc8, e.w(0x4434b8)?)?;
                icon = 159;
            } else if e.w(0x438766)? != 0 {
                e.sound(624, stack)?;
                e.ww(0x438766, 0)?;
                e.trap(0xa884, vec![P(0x400ed2)])?;
            } else {
                if (e.w(0x438760)? == 0 || (e.b(0x443761)? as i8) > 7) && e.w(0x438776)? != 0 {
                    e.trap(0xa873, vec![L(e.l(0x43878c)?)])?;
                    let sound = if attacker <= 9 {
                        if e.w(0x438760)? == 0 || (e.b(0x443761)? as i8) <= 7 {
                            e.w(warrior(attacker).wrapping_add(296))?
                        } else {
                            e.w(0x443774)?
                        }
                    } else if e.w(enemy(attacker).wrapping_add(78))? != 0 {
                        e.w(0x453776)?
                    } else {
                        let offset = i32::from(e.w(0x4435d4)?).wrapping_mul(4) as u32;
                        let field = enemy(attacker).wrapping_add(147).wrapping_add(offset);
                        let field = if e.b(field)? == 0 {
                            enemy(attacker).wrapping_add(149)
                        } else {
                            enemy(attacker).wrapping_add(149).wrapping_add(offset)
                        };
                        i16::from(e.b(field)? as i8)
                    };
                    e.sound(sound, stack)?;
                }
                let amount = e.w(0x4434b8)?;
                if amount > 1 {
                    e.trap(0xa884, vec![P(0x400ec2)])?;
                } else if amount == 1 {
                    e.trap(0xa884, vec![P(0x400eb4)])?;
                } else if amount < 0 {
                    e.trap(0xa884, vec![P(0x400ea4)])?;
                    if body > 9
                        && e.w(enemy(body).wrapping_add(272))?
                            > e.w(enemy(body).wrapping_add(274))?
                    {
                        e.ww(
                            enemy(body).wrapping_add(272),
                            e.w(enemy(body).wrapping_add(274))?,
                        )?;
                    }
                }
            }
            if e.w(0x4434b8)? != 0 {
                if body < 9 {
                    e.ww(warrior(body).wrapping_add(20), 1)?;
                    e.character(body, 0, stack)?;
                    if body == e.w(idx(0x4470b8, e.w(0x447194)?, 2))? {
                        e.prompt(stack.wrapping_sub(8), false)?;
                    }
                } else {
                    e.write(enemy(body).wrapping_add(261), &[1])?;
                }
            }
        }
        -40 => {
            e.indexed_string(e.w(0x45366a)?.wrapping_sub(19))?;
            e.sound(e.w(0x45364a)?, stack)?;
            icon = 165;
        }
        -34 | -33 | -32 | -30 | -29 | -28 | -27 | -26 => {
            let (message, sound, result_icon) = match code {
                -34 => (0x400e9a, 104, 159),
                -33 => (0x400e8e, 624, 164),
                -32 => (0x400e84, 630, 159),
                -30 => (0x400e76, 651, 164),
                -29 => (0x400e6c, 659, 164),
                -28 => (0x400e62, 667, 159),
                -27 => (0x400e58, 644, 164),
                _ => (0x400e50, 664, 164),
            };
            e.trap(0xa884, vec![P(message)])?;
            e.sound(sound, stack)?;
            icon = result_icon;
        }
        -25 | -24 | 25 => {
            let (message, sound) = match code {
                -25 => (0x400e42, 664),
                -24 => (0x400e22, 664),
                _ => (0x400e34, 650),
            };
            e.trap(0xa884, vec![P(message)])?;
            e.sound(sound, stack)?;
            e.ground(body, stack.wrapping_sub(2))?;
            e.body(body, 0, 0, stack.wrapping_sub(6))?;
            e.delay(0, stack.wrapping_sub(12))?;
            e.delay(20, stack.wrapping_sub(14))?;
            if e.w(0x438776)? == 0 {
                e.reset(stack)?;
            }
        }
        -23 | -22 | -21 | -20 | -19 | -16 | -15 => {
            e.string(e.w(0x442bc8)?, stack)?;
            e.trap(0xa862, vec![L(69)])?;
            let (message, sound, result_icon) = match code {
                -23 => (0x400e0e, 649, 163),
                -22 => (0x400df8, 107, 159),
                -21 => (0x400de2, 344, 162),
                -20 => (0x400dd0, 431, 161),
                -19 => (0x400dbe, 601, 160),
                -16 => (0x400d8a, 618, 159),
                _ => (0x400d76, 360, 164),
            };
            if code == -23 || code == -22 {
                e.trap(0xa884, vec![P(message)])?;
                e.sound(sound, stack.wrapping_sub(2))?;
            } else {
                e.sound(sound, stack.wrapping_sub(2))?;
                e.trap(0xa884, vec![P(message)])?;
            }
            icon = result_icon;
        }
        -18 | -17 | -51 | -50 | -45 => {
            let (message, sound, duration) = match code {
                -18 => (0x400dac, 693, 40),
                -17 => (0x400d9a, 693, 40),
                -51 => (0x400d34, 629, 60),
                -50 => (0x400d22, 629, 60),
                _ => (0x400d0e, 629, 60),
            };
            e.trap(0xa884, vec![P(message)])?;
            e.sound(sound, stack)?;
            e.delay(duration, stack.wrapping_sub(2))?;
        }
        22 | 23 | 29 => {
            e.string(e.w(0x4434b8)?, stack)?;
            e.trap(0xa862, vec![L(205)])?;
            let (message, sound) = match code {
                22 => (0x400d46, 612),
                23 => (0x400d58, 612),
                _ => (0x400d64, 107),
            };
            e.trap(0xa884, vec![P(message)])?;
            e.sound(sound, stack.wrapping_sub(2))?;
            if body < 9 {
                e.character(body, 1, stack)?;
            }
            e.delay(0, stack)?;
        }
        -47 => {
            e.trap(0xa884, vec![P(0x400d02)])?;
            e.sound(631, stack)?;
        }
        -48 => {
            e.trap(0xa884, vec![P(0x400cf2)])?;
            e.sound(-674, stack)?;
            e.sound(-674, stack.wrapping_sub(2))?;
            e.delay(30, stack.wrapping_sub(4))?;
        }
        -12 => {
            e.trap(0xa884, vec![P(0x400ce2)])?;
        }
        -11 | -14 => {
            e.trap(0xa862, vec![L(69)])?;
            if e.b(0x442750)? == 0 {
                e.trap(
                    0xa884,
                    vec![P(if code == -11 { 0x400cd2 } else { 0x400cc2 })],
                )?;
            }
            if code == -11 && body < 9 {
                let (roll, _) = e.rand(2, stack)?;
                e.sound(roll.wrapping_add(506), stack)?;
                e.ww(warrior(body).wrapping_add(16), 0)?;
                e.character(body, 1, stack.wrapping_sub(2))?;
            } else {
                e.sound(e.w(enemy(body).wrapping_add(106))?, stack)?;
            }
        }
        -10 | -3 | -4 => {
            e.trap(
                0xa884,
                vec![P(match code {
                    -10 => 0x400cba,
                    -3 => 0x400c5a,
                    _ => 0x400c6a,
                })],
            )?;
            let offset = if code == -4 {
                e.sound(133, stack)?;
                2
            } else {
                0
            };
            let restored_body = if code == -4 { 0 } else { body };
            e.ground(restored_body, stack.wrapping_sub(offset))?;
            e.body(restored_body, 0, 0, stack.wrapping_sub(offset + 4))?;
        }
        -9..=-5 => {
            e.trap(
                0xa884,
                vec![P(match code {
                    -9 => 0x400cac,
                    -8 => 0x400c98,
                    -7 => 0x400c8c,
                    -6 => 0x400c7c,
                    _ => 0x400c72,
                })],
            )?;
            if code == -5 {
                icon = 166;
            }
        }
        -2 => {
            e.trap(0xa884, vec![P(0x400c52)])?;
            if body < 9 {
                e.character(body, 1, stack)?;
                let (roll, _) = e.rand(2, stack.wrapping_sub(4))?;
                e.sound(roll.wrapping_add(506), stack.wrapping_sub(4))?;
            } else {
                e.sound(e.w(enemy(body).wrapping_add(106))?, stack)?;
            }
        }
        -1 => {
            e.trap(0xa884, vec![P(0x400c4c)])?;
            e.sound(650, stack)?;
        }
        _ => {
            if e.b(0x442750)? == 0 {
                e.indexed_string(e.w(frame.wrapping_add(10))?)?;
                e.delay(0, stack)?;
                if e.w(0x438776)? == 0 {
                    e.reset(stack)?;
                }
            }
        }
    }
    if e.plan.flow != LoadFlow::Returned {
        return Ok(e.plan);
    }
    if (e.w(0x438760)? == 0 || (e.b(0x443761)? as i8) > 7) && e.w(0x438776)? != 0 {
        if icon == 0 {
            e.delay(0, stack)?;
        } else {
            if icon == 166 {
                e.ww(0x4434b8, 0)?;
                e.show_damage(body, icon, e.w(0x45366c)?, stack)?;
            } else if icon == 165 && e.w(0x4434b8)? > 0 {
                e.show_damage(body, icon, e.w(0x45366c)?, stack)?;
            } else if icon > 0 && e.w(0x442bc8)? > 0 {
                e.show_damage(body, icon, e.w(0x442bc8)?, stack)?;
            } else if e.w(0x4434b8)? > 0 && e.w(frame.wrapping_add(10))? == 0 {
                e.show_damage(body, -1, e.w(0x4434b8)?, stack)?;
            }
            e.delay(30_i16.wrapping_sub(e.w(0x43956a)?.wrapping_mul(2)), stack)?;
            e.ww(stack.wrapping_sub(4), e.w(0x408572)?)?;
            let x = e.w(0x408570)?;
            e.center(stack.wrapping_sub(14), x)?;
        }
    }
    e.ww(0x4434c2, 0)?;
    e.ww(0x438784, 0)?;
    e.trap(0xa853, vec![])?;
    Ok(e.plan)
}

/// resist, 0x150114. BitTst's returned Boolean, not the decompiler's flag guess,
/// controls immunity. The input body is held in the native saved D4.W.
pub fn resist(
    state: &CombatResultState,
    frame: u32,
    limits: PromptLimits,
    services: &mut (impl CombatResultServices + ?Sized),
) -> Result<CombatResultPlan, RuleError> {
    let mut e = execution(state, limits, services);
    let stack = frame.wrapping_sub(18);
    let body = e.w(frame.wrapping_add(8))?;
    if body < 0 {
        e.plan.native_return = CombatResultReturn::Full(1);
        return Ok(e.plan);
    }
    e.ww(0x4471ea, 0)?;
    if e.w(0x443780)? != 0 {
        let bit = e.w(warrior(body).wrapping_add(50))?.wrapping_sub(1);
        if e.trap(0xa85d, vec![P(0x443780), L(i32::from(bit) as u32)])? as u8 != 0 {
            e.flash(stack.wrapping_sub(20), 0x4018b4, 50, 50, 40, 651)?;
            e.plan.native_return = CombatResultReturn::Full(1);
            return Ok(e.plan);
        }
    }
    if e.w(0x45375a)? != 0 || e.w(0x4537f0)? != 0 {
        return Ok(e.plan);
    }
    if e.b(0x443761)? == 9 {
        let mut threshold = 33_i16;
        let acting = e.w(idx(0x4470b8, e.w(0x447194)?, 2))?;
        if (0..9).contains(&acting) {
            threshold = threshold.wrapping_add(
                e.w(warrior(acting).wrapping_add(4))?
                    .wrapping_add(e.w(warrior(acting).wrapping_add(2904))?),
            );
        } else if acting > 9 {
            threshold = threshold.wrapping_add(e.w(enemy(acting).wrapping_add(102))?);
            threshold = threshold.wrapping_add(e.w(enemy(acting))?.wrapping_mul(5));
        }
        if body < 9 {
            if e.w(warrior(body).wrapping_add(96))? != 0 {
                return Ok(e.plan);
            }
            threshold = threshold
                .wrapping_sub(
                    e.w(warrior(body).wrapping_add(8))?
                        .wrapping_add(e.w(warrior(body).wrapping_add(2908))?),
                )
                .wrapping_sub(5);
            let (roll, _) = e.rand(e.w(warrior(body).wrapping_add(2536))? / 3, stack)?;
            threshold = threshold.wrapping_sub(roll);
            // MULU at 0x1502ce restores the full record offset before this read.
            threshold = threshold.wrapping_sub(
                e.w(warrior(body).wrapping_add(22))?
                    .wrapping_mul(e.w(warrior(body).wrapping_add(58))?),
            );
        } else {
            if e.b(enemy(body).wrapping_add(179))? != 0 {
                return Ok(e.plan);
            }
            threshold = threshold.wrapping_sub(e.w(enemy(body).wrapping_add(98))?);
            let (roll, _) = e.rand(e.w(enemy(body).wrapping_add(4))? / 3, stack)?;
            threshold = threshold
                .wrapping_sub(roll)
                .wrapping_sub(e.w(enemy(body))?.wrapping_mul(3));
            threshold = threshold.wrapping_sub(
                i16::from(e.b(enemy(body).wrapping_add(260))? as i8)
                    .wrapping_mul(e.w(enemy(body))?),
            );
            let size = e.abs(e.w(enemy(body).wrapping_add(16))?, stack.wrapping_sub(2))?;
            threshold = threshold.wrapping_add(match size {
                0 => -5,
                1 | 2 => 5,
                3 => 10,
                _ => 0,
            });
        }
        let maximum = 90_i16.wrapping_sub(e.w(0x442576)?.wrapping_mul(5));
        e.ww(stack.wrapping_sub(2), maximum)?;
        let minimum = 15_i16.wrapping_sub(e.w(0x442576)?.wrapping_mul(5));
        e.ww(stack.wrapping_sub(4), minimum)?;
        e.ww(stack.wrapping_sub(6), threshold)?;
        threshold = crate::rules::pin(threshold, minimum, maximum);
        let (roll, _) = e.rand(100, stack.wrapping_sub(6))?;
        e.ww(0x4434b6, roll)?;
        if threshold < e.w(0x4434b6)? {
            e.ww(0x4471ea, 1)?;
        }
        e.plan.native_return = CombatResultReturn::LowWord(e.w(0x4471ea)? as u16);
        return Ok(e.plan);
    }
    let selector = e.b(0x443762)? as i8;
    let test_resistance = if selector < 0 {
        e.abs(i16::from(selector), stack)? != 9 || e.b(0x443761)? == 0
    } else {
        e.b(0x443761)? == 0
    };
    if test_resistance {
        let mut threshold;
        if e.b(0x443761)? == 0 {
            if body < 9 {
                threshold = e.w(warrior(body).wrapping_add(274))?;
                if e.w(warrior(body).wrapping_add(50))? == 3 {
                    threshold = threshold.wrapping_add(1000);
                }
            } else {
                threshold = e.w(enemy(body))?.wrapping_shl(2).wrapping_add(35);
                if e.b(enemy(body).wrapping_add(171))? != 0 {
                    threshold = threshold.wrapping_add(1000);
                }
            }
        } else {
            threshold = 35_i16.wrapping_add(
                if body < 9 {
                    e.w(warrior(body).wrapping_add(58))?
                } else {
                    e.w(enemy(body))?
                }
                .wrapping_mul(5),
            );
            if e.w(idx(0x4470b8, e.w(0x447194)?, 2))? < 9 {
                threshold = threshold.wrapping_sub(
                    e.w(warrior(e.w(0x4435d2)?).wrapping_add(58))?
                        .wrapping_mul(5),
                );
            } else {
                threshold = threshold.wrapping_sub(
                    e.w(0x439d3c_u32.wrapping_add(u32::from(e.w(0x447196)? as u16) * 288))?
                        .wrapping_mul(5),
                );
            }
        }
        if e.b(0x443756)? != 0 {
            threshold = threshold
                .wrapping_add(i16::from(e.b(0x443756)? as i8).wrapping_mul(e.w(0x4435cc)?));
        }
        let (roll, _) = e.rand(100, stack)?;
        if roll <= threshold {
            let code = if e.b(0x435662)? == 0 { 25 } else { -25 };
            e.ww(stack.wrapping_sub(2), code)?;
            e.ww(stack.wrapping_sub(4), body)?;
            let child = showresults(&e.plan.state, stack.wrapping_sub(12), e.limits, e.services)?;
            e.plan.state = child.state;
            e.plan.effects.extend(child.effects);
            e.plan.flow = child.flow;
            if child.flow != LoadFlow::Returned {
                e.plan.native_return = CombatResultReturn::NoReturn;
                return Ok(e.plan);
            }
            e.delay(0, stack)?;
            e.plan.native_return = CombatResultReturn::Full(2);
            return Ok(e.plan);
        }
    }
    if body > 9 && (e.b(0x443761)? as i8) < 7 {
        let resistance = enemy(body)
            .wrapping_add(139)
            .wrapping_add(i32::from(e.b(0x443761)? as i8) as u32);
        if (e.b(resistance)? as i8) > 100 || e.w(enemy(body).wrapping_add(10))? > 100 {
            e.plan.native_return = CombatResultReturn::Full(1);
            return Ok(e.plan);
        }
    }
    let delivery = e.b(0x443757)? as i8;
    if (delivery == 1 || delivery > 2) && e.b(0x443761)? != 9 && (e.b(0x443761)? as i8) != -9 {
        return Ok(e.plan);
    }
    let mut selector = e.w(0x4435c8)?;
    while selector < 7 {
        e.step()?;
        let selected = selector.wrapping_add(16);
        let active = if body < 9 {
            e.w(warrior(body)
                .wrapping_add(94)
                .wrapping_add((i32::from(selected) * 2) as u32))?
                != 0
        } else {
            e.b(enemy(body)
                .wrapping_add(178)
                .wrapping_add(i32::from(selected) as u32))?
                != 0
        };
        if active {
            e.ww(0x4471ea, 1)?;
        }
        selector = selector.wrapping_add(1);
    }
    if e.b(0x443761)? == 0 || e.b(0x443761)? == 5 {
        let active = if body < 9 {
            e.w(warrior(body).wrapping_add(144))? != 0
        } else {
            e.b(enemy(body).wrapping_add(203))? != 0
        };
        if active {
            e.plan.native_return = CombatResultReturn::Full(1);
            return Ok(e.plan);
        }
    }
    let modifier = i16::from(e.b(0x443758)? as i8).wrapping_mul(e.w(0x4435cc)?);
    let base = if body < 9 {
        e.w(warrior(body).wrapping_add(34))?
    } else {
        e.w(enemy(body).wrapping_add(10))?
    };
    e.ww(frame.wrapping_sub(2), modifier.wrapping_add(base))?;
    let (roll, _) = e.rand(100, stack)?;
    if roll <= e.w(frame.wrapping_sub(2))? {
        if body < 9 {
            return Ok(e.plan);
        }
        e.ww(0x4471ea, 1)?;
    }
    e.plan.native_return = CombatResultReturn::LowWord(e.w(0x4471ea)? as u16);
    Ok(e.plan)
}
