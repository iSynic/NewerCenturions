//! Warrior promotion, character viewing and party selection from CODE 1.

use crate::RuleError;
use crate::recovered_character_creation::{self as creation, CharacterCreationServices};
use crate::recovered_character_panels::{self as character, CharacterPanelServices};
use crate::recovered_character_ui::{self as character_ui, CharacterUiEffect, CharacterUiState};
use crate::recovered_drawing::{
    self as drawing, BuildingWorld, DrawingEffect, DrawingServices, PictureObservation,
};
use crate::recovered_equipment::{self as equipment, read};
use crate::recovered_feedback::{
    self as feedback, FeedbackArg, FeedbackEffect, FeedbackRequest, FeedbackServices,
};
use crate::recovered_field_control::{self as field, FieldControlServices};
use crate::recovered_inventory_edit::{
    self as inventory_edit, InventoryEditEffect, InventoryEditLimits,
};
use crate::recovered_inventory_selection::{
    self as selection, InventorySelectionEffect, InventorySelectionLimits,
};
use crate::recovered_load_control::{self as load, LoadFlow};
use crate::recovered_prompt_control::{
    self as prompt, LibraryRequest, PromptControlEffect, PromptControlState, PromptLimits,
    PromptRegisterRequest, PromptReturn,
};
use crate::recovered_records::{self as records, ItemTables, RecordWrite};
use crate::recovered_rendering::RenderingMemory;
use crate::recovered_runtime::{self as runtime, RuntimeResult};
use crate::recovered_storage::{self as storage, StorageEffect, StorageLimits};
use crate::recovered_text::PascalString;
use crate::recovered_timing::{self as timing, TimingServices};
use FeedbackArg::{Byte as B, Long as L, Pointer as P, Word as W};
use centurions_content::{ITEM_BYTES, WARRIOR_BYTES, Warrior};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharacterUpgradeQualification {
    SourcePortWithObservedBoundariesAndProjectedChildren,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpgradeSaneRequest {
    pub opcode: u16,
    pub arguments: Vec<FeedbackArg>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpgradeSaneObservation {
    pub memory: RenderingMemory,
    pub d0: u32,
    pub ccr: u8,
}
pub trait CharacterUpgradeServices: CharacterCreationServices + FieldControlServices {
    fn observe_sane(
        &mut self,
        _: &UpgradeSaneRequest,
        _: &RenderingMemory,
    ) -> Result<UpgradeSaneObservation, RuleError> {
        Err(RuleError::Unsupported {
            action: "character_upgrade.sane",
            dependency: "the reached SANE memory and condition-code observation is missing",
        })
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum CharacterUpgradeEffect {
    Prompt(PromptControlEffect),
    CharacterUi(CharacterUiEffect),
    InventoryEdit(InventoryEditEffect),
    Storage(StorageEffect),
    Selection(InventorySelectionEffect),
    Creation(creation::CharacterCreationEffect),
    Field(field::FieldControlEffect),
    Status(crate::recovered_status_panels::StatusPanelEffect),
    Sane {
        request: UpgradeSaneRequest,
        observation: UpgradeSaneObservation,
    },
    Location {
        argument: u16,
        native_d0: i32,
    },
}
#[derive(Clone, Debug, PartialEq)]
pub struct CharacterUpgradePlan {
    pub state: PromptControlState,
    pub effects: Vec<CharacterUpgradeEffect>,
    pub native_return: PromptReturn,
    pub flow: LoadFlow,
    pub qualification: CharacterUpgradeQualification,
}
struct Execution<'a, S: ?Sized> {
    plan: CharacterUpgradePlan,
    s: &'a mut S,
    limits: PromptLimits,
    remaining: u32,
    d0: Option<u32>,
}
impl<S: CharacterUpgradeServices + ?Sized> Execution<'_, S> {
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
            .push(CharacterUpgradeEffect::Prompt(PromptControlEffect::Write(
                RecordWrite {
                    address: a,
                    bytes: bytes.to_vec(),
                },
            )));
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
            PromptReturn::LowWord(w) => {
                self.d0 = None;
                self.plan.native_return = PromptReturn::LowWord(w);
            }
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
        self.plan.effects.push(CharacterUpgradeEffect::Prompt(
            PromptControlEffect::Feedback(FeedbackEffect::Service {
                request,
                observation,
            }),
        ));
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
        self.plan.effects.push(CharacterUpgradeEffect::Prompt(
            PromptControlEffect::RegisterToolbox {
                request,
                observation,
            },
        ));
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
        self.plan.effects.push(CharacterUpgradeEffect::Prompt(
            PromptControlEffect::Library {
                request,
                observation,
            },
        ));
        Ok(value)
    }

    fn port(&mut self, pointer: u32) -> Result<(), RuleError> {
        self.trap(0xa873, vec![L(pointer)])?;
        Ok(())
    }

    fn feedback(&mut self, child: feedback::FeedbackPlan) {
        self.plan.state.memory = child.memory;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(|e| CharacterUpgradeEffect::Prompt(PromptControlEffect::Feedback(e))),
        );
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
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(|e| CharacterUpgradeEffect::Prompt(PromptControlEffect::Runtime(e))),
        );
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

    fn rand(&mut self, bound: i16) -> Result<i16, RuleError> {
        let mut context =
            runtime::RuntimeContext::new(self.plan.state.random.clone(), self.remaining as usize);
        let child = runtime::rand(&mut context, bound)?;
        self.plan.state.random = context.random;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(|e| CharacterUpgradeEffect::Prompt(PromptControlEffect::Runtime(e))),
        );
        let RuntimeResult::Word { value, native_d0 } = child.result else {
            return Err(RuleError::Invalid("unexpected native Rand result".into()));
        };
        self.full(native_d0);
        Ok(value)
    }

    fn active(&self) -> bool {
        self.plan.flow == LoadFlow::Returned
    }

    fn merge_prompt(&mut self, child: prompt::PromptControlPlan) {
        self.plan.state = child.state;
        self.plan.flow = child.flow;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(CharacterUpgradeEffect::Prompt),
        );
        self.returned(child.native_return);
    }

    fn question(&mut self, frame: u32, id: u16, flag: u16) -> Result<u16, RuleError> {
        self.long(
            frame.wrapping_add(8),
            (u32::from(id) << 16) | u32::from(flag),
        )?;
        let child = prompt::question(&self.plan.state, frame, self.limits, self.s)?;
        self.merge_prompt(child);
        match self.plan.native_return {
            PromptReturn::Full(d) => Ok(d as u16),
            PromptReturn::LowWord(w) => Ok(w),
            _ => Err(RuleError::Invalid("question has no native result".into())),
        }
    }

    fn gitem(&mut self, dialog: u32, item: u16, rect: u32) -> Result<(), RuleError> {
        self.trap(
            0xa98d,
            vec![L(dialog), W(item), P(0x4434b0), P(0x4434dc), P(rect)],
        )?;
        Ok(())
    }

    fn di_str(&mut self, frame: u32, item: u16, address: u32) -> Result<(), RuleError> {
        self.long(frame.wrapping_add(10), address)?;
        self.word(frame.wrapping_add(8), item)?;
        self.gitem(self.l(0x4394b8)?, self.w(frame.wrapping_add(8))?, 0x4434ba)?;
        self.trap(
            0xa98f,
            vec![L(self.l(0x4434dc)?), P(self.l(frame.wrapping_add(10))?)],
        )?;
        self.full(0);
        Ok(())
    }

    fn di_num(&mut self, frame: u32, item: u16, value: i16, nz: bool) -> Result<(), RuleError> {
        self.word(frame.wrapping_add(10), value as u16)?;
        self.word(frame.wrapping_add(8), item)?;
        let captured = value;
        self.gitem(self.l(0x4394b8)?, item, 0x4434ba)?;
        if nz && captured == 0 {
            self.trap(0xa98f, vec![L(self.l(0x4434dc)?), P(0x4021a9)])?;
        } else {
            let number = if nz && captured > 0 {
                captured.wrapping_neg()
            } else if nz {
                captured
            } else {
                self.sw(frame.wrapping_add(10))?
            };
            self.lib(0x400168, vec![L(number as i32 as u32), P(0x479ed8)])?;
            if nz && captured > 0 {
                self.byte(0x479ed9, b'+')?;
            }
            self.trap(0xa98f, vec![L(self.l(0x4434dc)?), P(0x479ed8)])?;
        }
        self.full(0);
        Ok(())
    }

    fn copy(&mut self, source: u32, target: u32, count: u32) -> Result<(), RuleError> {
        for i in 0..count / 4 {
            self.step()?;
            self.long(
                target.wrapping_add(i * 4),
                self.l(source.wrapping_add(i * 4))?,
            )?;
        }
        if count % 4 == 2 {
            self.word(
                target.wrapping_add(count - 2),
                self.w(source.wrapping_add(count - 2))?,
            )?;
        }
        Ok(())
    }

    fn icon_plot(&mut self, frame: u32, id: u16, rect: u32) -> Result<(), RuleError> {
        self.long(frame.wrapping_add(14), self.l(rect.wrapping_add(4))?)?;
        self.long(frame.wrapping_add(10), self.l(rect)?)?;
        self.word(frame.wrapping_add(8), id)?;
        self.long(0x439598, 0)?;
        let handle = self.trap(0xaa1e, vec![W(self.w(frame.wrapping_add(8))?)])?;
        self.long(0x439598, handle)?;
        if self.l(0x439598)? != 0 {
            self.trap(
                0xaa1f,
                vec![P(frame.wrapping_add(10)), L(self.l(0x439598)?)],
            )?;
            self.trap(0xaa25, vec![L(self.l(0x439598)?)])?;
        }
        self.full(0);
        Ok(())
    }

    fn edge_button(&mut self, frame: u32, up: bool) -> Result<(), RuleError> {
        self.word(frame.wrapping_add(8), 0)?;
        self.trap(0xa874, vec![P(frame.wrapping_sub(22))])?;
        self.trap(0xa898, vec![P(frame.wrapping_sub(18))])?;
        if self.b(frame.wrapping_add(9))? != 0 {
            self.port(self.l(0x43878c)?)?;
        }
        self.trap(0xa89c, vec![W(0)])?;
        self.trap(0xa89b, vec![W(1), W(1)])?;
        self.trap(
            0xaa0a,
            vec![L(self.l(if up { 0x4358e2 } else { 0x4358de })?)],
        )?;
        self.trap(
            0xa893,
            vec![W(self.w(0x44292e)?), W(self.w(0x442930)?.wrapping_sub(1))],
        )?;
        self.trap(0xa891, vec![W(self.w(0x44292e)?), W(self.w(0x44292c)?)])?;
        self.trap(
            0xa891,
            vec![W(self.w(0x442932)?.wrapping_sub(1)), W(self.w(0x44292c)?)],
        )?;
        self.trap(
            0xaa0a,
            vec![L(self.l(if up { 0x4358de } else { 0x4358e2 })?)],
        )?;
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

    fn profile_open(&mut self, race: bool) -> Result<(), RuleError> {
        let limits = StorageLimits {
            filename_capacity: self.limits.filename_capacity,
            maximum_iterations: self.remaining as usize,
            maximum_polls: self.limits.maximum_polls,
        };
        let mut child = storage::Execution::new(&self.plan.state.memory, limits, self.d0);
        child.filename(if race { 0x401e02 } else { 0x401de2 }, false, self.s)?;
        self.plan.state.memory = child.plan.memory;
        self.plan.effects.extend(
            child
                .plan
                .effects
                .into_iter()
                .map(CharacterUpgradeEffect::Storage),
        );
        let h = self.lib(0x400070, vec![P(0x40250e), P(0x4389a4)])?;
        self.long(0x4434a6, h)?;
        if h == 0 {
            let h = self.lib(
                0x400070,
                vec![P(0x40250e), P(if race { 0x401dec } else { 0x401dcc })],
            )?;
            self.long(0x4434a6, h)?;
        }
        self.full(0);
        Ok(())
    }

    fn profile(&mut self, frame: u32) -> Result<(), RuleError> {
        for (argument, is_race, size, target) in [
            (frame.wrapping_add(8), true, 408u32, 0x41f85a),
            (frame.wrapping_add(10), false, 576, 0x41f9f2),
        ] {
            if self.w(argument)? == 0 {
                continue;
            }
            self.profile_open(is_race)?;
            self.lib(
                0x400078,
                vec![
                    W(0),
                    L(u32::from(self.w(argument)?.wrapping_sub(1)) * size),
                    L(self.l(0x4434a6)?),
                ],
            )?;
            self.lib(
                0x400048,
                vec![L(self.l(0x4434a6)?), L(1), L(size), P(target)],
            )?;
            self.lib(0x400068, vec![L(self.l(0x4434a6)?)])?;
        }
        self.full(0);
        Ok(())
    }
}

impl<S: CharacterUpgradeServices + ?Sized> Execution<'_, S> {
    fn pw(&mut self, sp: &mut u32, v: u16) -> Result<(), RuleError> {
        *sp = sp.wrapping_sub(2);
        self.word(*sp, v)
    }
    fn pl(&mut self, sp: &mut u32, v: u32) -> Result<(), RuleError> {
        *sp = sp.wrapping_sub(4);
        self.long(*sp, v)
    }
    fn sane(&mut self, pointers: &[u32], selector: u16) -> Result<u8, RuleError> {
        self.step()?;
        let mut arguments: Vec<FeedbackArg> = pointers.iter().copied().map(P).collect();
        arguments.push(W(selector));
        let request = UpgradeSaneRequest {
            opcode: 0xa9eb,
            arguments,
        };
        let observation = self.s.observe_sane(&request, &self.plan.state.memory)?;
        let ccr = observation.ccr;
        self.plan.state.memory = observation.memory.clone();
        self.d0 = Some(observation.d0);
        self.plan.effects.push(CharacterUpgradeEffect::Sane {
            request,
            observation,
        });
        Ok(ccr)
    }
    fn first_call(&mut self, sp: &mut u32, pair: u32) -> Result<(), RuleError> {
        self.pl(sp, pair)?;
        let child =
            prompt::first_tutorial(&self.plan.state, sp.wrapping_sub(8), self.limits, self.s)?;
        self.merge_prompt(child);
        Ok(())
    }
    fn sound_call(&mut self, sp: &mut u32, id: u16) -> Result<(), RuleError> {
        self.pw(sp, id)?;
        self.sound(id as i16)
    }
    fn rand_call(&mut self, sp: &mut u32, bound: u16) -> Result<u16, RuleError> {
        self.pw(sp, bound)?;
        Ok(self.rand(bound as i16)? as u16)
    }
    fn location(&mut self, sp: &mut u32, argument: u16) -> Result<(), RuleError> {
        self.pw(sp, argument)?;
        let native_d0 = crate::recovered_interface::locationlog();
        self.plan.effects.push(CharacterUpgradeEffect::Location {
            argument,
            native_d0,
        });
        self.full(native_d0 as u32);
        Ok(())
    }
    fn pict_call(&mut self, sp: &mut u32, id: u16, rect: u32) -> Result<(), RuleError> {
        self.pict(sp.wrapping_sub(18), id, rect)?;
        *sp = sp.wrapping_sub(10);
        Ok(())
    }
    fn edge_call(&mut self, sp: &mut u32, up: bool) -> Result<(), RuleError> {
        self.edge_button(sp.wrapping_sub(10), up)?;
        *sp = sp.wrapping_sub(2);
        Ok(())
    }
    fn press_call(
        &mut self,
        sp: &mut u32,
        kind: u16,
        phase: u16,
        sound: u16,
        rect: u32,
    ) -> Result<(), RuleError> {
        self.press(sp.wrapping_sub(22), kind, phase, sound, rect)?;
        *sp = sp.wrapping_sub(14);
        Ok(())
    }
    fn string_call(&mut self, sp: &mut u32, item: u16, pointer: u32) -> Result<(), RuleError> {
        self.di_str(sp.wrapping_sub(14), item, pointer)?;
        *sp = sp.wrapping_sub(6);
        Ok(())
    }
    fn number_call(&mut self, sp: &mut u32, item: u16, v: u16) -> Result<(), RuleError> {
        self.di_num(sp.wrapping_sub(12), item, v as i16, false)?;
        *sp = sp.wrapping_sub(4);
        Ok(())
    }
    fn long_number_call(&mut self, sp: &mut u32, item: u16, v: u32) -> Result<(), RuleError> {
        self.pl(sp, v)?;
        self.pw(sp, item)?;
        let frame = sp.wrapping_sub(8);
        self.gitem(self.l(0x4394b8)?, self.w(frame + 8)?, 0x4434ba)?;
        self.trap(0xa9ee, vec![L(self.l(frame + 10)?), P(0x438aa4)])?;
        self.trap(0xa98f, vec![L(self.l(0x4434dc)?), P(0x438aa4)])?;
        self.full(0);
        Ok(())
    }
    fn portrait_call(
        &mut self,
        sp: &mut u32,
        member: u16,
        rect: u32,
        with_body: bool,
    ) -> Result<(), RuleError> {
        if with_body {
            self.pw(sp, 0)?;
            self.pw(sp, member)?;
        } else {
            self.pl(sp, 0xffff0000)?;
        }
        self.pw(sp, self.w(warrior(member) + 0x34)?)?;
        self.pl(sp, self.l(rect + 4)?)?;
        self.pl(sp, self.l(rect)?)?;
        self.pw(sp, self.w(warrior(member) + 0x54)?)?;
        let child = character::plot_portrait(
            &self.plan.state.memory,
            sp.wrapping_sub(8),
            self.remaining,
            self.s,
        )?;
        self.plan.state.memory = child.memory;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(|e| CharacterUpgradeEffect::Prompt(PromptControlEffect::Character(e))),
        );
        self.full(child.native_return);
        Ok(())
    }
    fn icon_call(&mut self, sp: &mut u32, member: u16, rect: u32) -> Result<(), RuleError> {
        self.pl(sp, self.l(rect + 4)?)?;
        self.pl(sp, self.l(rect)?)?;
        self.pw(sp, self.w(warrior(member) + 0x56)?)?;
        let frame = sp.wrapping_sub(8);
        self.long(0x439598, 0)?;
        let handle = self.trap(0xaa1e, vec![W(self.w(frame + 8)?)])?;
        self.long(0x439598, handle)?;
        if self.l(0x439598)? != 0 {
            self.trap(0xaa1f, vec![P(frame + 10), L(self.l(0x439598)?)])?;
            self.trap(0xaa25, vec![L(self.l(0x439598)?)])?;
        }
        self.full(0);
        Ok(())
    }
    fn load_item(&mut self, id: u16) -> Result<(), RuleError> {
        let backing = equipment::item_tables(&self.plan.state.memory, id as i16)?;
        let tables = ItemTables {
            families: std::array::from_fn(|n| backing[n].as_slice()),
        };
        let work = read(&self.plan.state.memory, 0x45363c, ITEM_BYTES)?
            .try_into()
            .unwrap();
        let child = records::loaditem(&tables, &work, id as i16)?;
        for write in child.writes {
            self.write(write.address, &write.bytes)?;
        }
        self.full(child.native_return as u32);
        Ok(())
    }
    fn weight_call(&mut self, sp: &mut u32, member: u16) -> Result<(), RuleError> {
        self.pw(sp, member)?;
        let frame = sp.wrapping_sub(8);
        self.word(frame.wrapping_sub(2), self.w(warrior(member) + 0x9d6)?)?;
        self.word(warrior(member) + 0x9d6, 0)?;
        let mut slot = 0i16;
        while slot < self.sw(warrior(member) + 0x1a)? {
            self.step()?;
            let node = warrior(member).wrapping_add(u32::from(slot as u16) * 22 + 0x132);
            self.word(frame.wrapping_sub(12), self.w(node)?)?;
            self.load_item(self.w(node)?)?;
            let v = self
                .sw(0x453660)?
                .wrapping_mul(self.sw(node + 2)?)
                .wrapping_add(self.sw(0x45364c)?);
            self.add(warrior(member) + 0x9d6, v)?;
            slot = slot.wrapping_add(1);
        }
        self.full(0);
        Ok(())
    }
    fn stats_call(&mut self, sp: &mut u32, member: u16) -> Result<(), RuleError> {
        self.pw(sp, member)?;
        let frame = sp.wrapping_sub(8);
        let base = if member == u16::MAX {
            0x44721c
        } else {
            warrior(member)
        };
        let local = frame.wrapping_sub(3138);
        self.copy(base, local, 3138)?;
        let mut w = Warrior {
            name: String::new(),
            original_sha256: String::new(),
            bytes: read(&self.plan.state.memory, local, 3138)?,
        };
        crate::character::calculate_statistics(&mut w)?;
        let mut fields = vec![
            0xb48, 0xb4a, 0xb4c, 0xb4e, 0xb50, 0xb52, 0xb54, 0xb56, 0xb58, 0xb5a, 0xb5c, 0xb5e,
            0xb7a, 0xb46,
        ];
        if self.w(local + 0x36)? != 0 {
            fields.push(0xb44);
        } else {
            fields.extend([0xb40, 0xb44]);
        }
        fields.push(0xb60);
        for (skill, dest) in [
            (0xf4, 0xb62),
            (0xf6, 0xb64),
            (0xf8, 0xb66),
            (0xfa, 0xb68),
            (0xfc, 0xb6a),
            (0xfe, 0xb6c),
            (0x100, 0xb6e),
            (0x102, 0xb70),
            (0x104, 0xb72),
        ] {
            if self.w(local + skill)? != 0 {
                fields.push(dest);
            }
        }
        for at in fields {
            self.word(
                local + at,
                u16::from_be_bytes([w.bytes[at as usize], w.bytes[at as usize + 1]]),
            )?;
        }
        self.copy(local, base, 3138)?;
        self.full(0);
        Ok(())
    }
    fn apply_upgrade_call(&mut self, sp: &mut u32, member: u16) -> Result<(), RuleError> {
        self.pw(sp, 1)?;
        self.pw(sp, self.w(0x4042da)?)?;
        self.pw(sp, member)?;
        let frame = sp.wrapping_sub(8);
        let local = frame.wrapping_sub(3138);
        let base = if member == u16::MAX {
            0x44721c
        } else {
            warrior(member)
        };
        self.copy(base, local, 3138)?;
        let selector = self.w(frame + 10)?;
        let mut w = Warrior {
            name: String::new(),
            original_sha256: String::new(),
            bytes: read(&self.plan.state.memory, local, 3138)?,
        };
        crate::character::apply_upgrade(&mut w, selector, self.sw(frame + 12)?)?;
        let target = match selector {
            0..=6 => Some(0x9e2 + u32::from(selector) * 2),
            7 => Some(0x9d8),
            8..=15 => Some(0x112 + u32::from(selector - 8) * 2),
            16..=19 => Some(4 + u32::from(selector - 16) * 2),
            20 => Some(0xb42),
            21 => Some(0xb40),
            22 => Some(0x22),
            23..=31 => Some(0xf4 + u32::from(selector - 23) * 2),
            32 => Some(0xc),
            _ => None,
        };
        if let Some(at) = target {
            self.word(
                local + at,
                u16::from_be_bytes([w.bytes[at as usize], w.bytes[at as usize + 1]]),
            )?;
            if selector == 0 {
                self.word(
                    local + 0x9d8,
                    u16::from_be_bytes([w.bytes[0x9d8], w.bytes[0x9d9]]),
                )?;
            }
        }
        for at in [0x52, 0x5a] {
            self.word(
                local + at,
                u16::from_be_bytes([w.bytes[at as usize], w.bytes[at as usize + 1]]),
            )?;
        }
        self.copy(local, base, 3138)?;
        self.full(0);
        Ok(())
    }
    fn ui_state(&self) -> CharacterUiState {
        CharacterUiState {
            memory: self.plan.state.memory.clone(),
            files: crate::recovered_files::FileContext {
                filename: vec![0; self.limits.filename_capacity],
                race_profile: vec![0; 408],
                caste_profile: vec![0; 576],
                application_name: vec![0; 256],
                ..Default::default()
            },
        }
    }
    fn ui_call(&mut self, sp: &mut u32, v: u16, upgrade: bool) -> Result<(), RuleError> {
        self.pw(sp, v)?;
        let state = self.ui_state();
        let child = if upgrade {
            character_ui::show_upgrade(&state, v, self.s)?
        } else {
            character_ui::show_stats(&state, v as i16, sp.wrapping_sub(8), self.s)?
        };
        self.merge_ui(child);
        Ok(())
    }
    fn merge_ui(&mut self, child: character_ui::CharacterUiResult) {
        self.plan.state.memory = child.state.memory;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(CharacterUpgradeEffect::CharacterUi),
        );
        match child.native_return {
            Some(d) => self.full(d as u32),
            None => {
                self.d0 = None;
                self.plan.native_return = PromptReturn::Unchanged;
            }
        }
    }
    fn merge_creation(&mut self, child: creation::CharacterCreationPlan) {
        self.plan.state = child.state;
        self.plan.flow = child.flow;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(CharacterUpgradeEffect::Creation),
        );
        self.returned(child.native_return);
    }
    fn selection_limits(&self) -> InventorySelectionLimits {
        InventorySelectionLimits {
            maximum_steps: self.remaining,
            prompt: self.limits,
            equipment: equipment::EquipmentLimits {
                maximum_steps: self.remaining as usize,
                maximum_polls: self.limits.maximum_polls,
                filename_capacity: self.limits.filename_capacity,
            },
            maximum_random_draws: u64::from(self.remaining),
        }
    }
    fn merge_selection(&mut self, child: selection::InventorySelectionPlan) {
        self.plan.state = child.state;
        self.plan.flow = child.flow;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(CharacterUpgradeEffect::Selection),
        );
        self.returned(child.native_return);
    }
    fn special_call(&mut self, frame: u32) -> Result<(), RuleError> {
        let child = crate::recovered_status_panels::show_special(
            &self.plan.state.memory,
            &self.plan.state.random,
            frame,
            self.remaining as usize,
            self.s,
        )?;
        self.plan.state.memory = child.memory;
        self.plan.state.random = child.random;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(CharacterUpgradeEffect::Status),
        );
        self.d0 = child.native_return.known_long();
        self.plan.native_return = match (self.d0, child.native_return.low_word) {
            (Some(d), _) => PromptReturn::Full(d),
            (None, Some(w)) => PromptReturn::LowWord(w),
            _ => PromptReturn::Unchanged,
        };
        Ok(())
    }
    fn condition_call(&mut self, sp: &mut u32, member: u16, mode: u32) -> Result<u16, RuleError> {
        self.pw(sp, member)?;
        self.pl(sp, mode)?;
        self.pw(sp, member)?;
        self.pw(sp, member)?;
        let child = selection::show_condition(
            &self.plan.state,
            sp.wrapping_sub(8),
            self.selection_limits(),
            self.s,
        )?;
        self.merge_selection(child);
        if !self.active() {
            return Ok(0);
        }
        match self.plan.native_return {
            PromptReturn::Full(d) => Ok(d as u16),
            PromptReturn::LowWord(w) => Ok(w),
            _ => Err(RuleError::Invalid(
                "condition child has no native result".into(),
            )),
        }
    }
    fn edit_call(
        &mut self,
        sp: &mut u32,
        pointer: u32,
        value: u16,
        plus: bool,
    ) -> Result<(), RuleError> {
        self.pw(sp, value)?;
        self.pl(sp, pointer)?;
        let limits = InventoryEditLimits {
            maximum_steps: self.remaining,
            equipment: self.selection_limits().equipment,
            maximum_random_draws: u64::from(self.remaining),
        };
        let child = if plus {
            inventory_edit::plus(
                &self.plan.state.memory,
                &self.plan.state.random,
                sp.wrapping_sub(8),
                limits,
                self.s,
            )?
        } else {
            inventory_edit::minus(
                &self.plan.state.memory,
                &self.plan.state.random,
                sp.wrapping_sub(8),
                limits,
                self.s,
            )?
        };
        self.plan.state.memory = child.memory;
        self.plan.state.random = child.random;
        self.plan.flow = child.flow;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(CharacterUpgradeEffect::InventoryEdit),
        );
        self.d0 = child.native_return;
        self.plan.native_return = if self.active() {
            child
                .native_return
                .map_or(PromptReturn::Unchanged, PromptReturn::Full)
        } else {
            PromptReturn::NoReturn
        };
        Ok(())
    }
    fn copied_scans(&mut self, member: u16, slot: i16) -> Result<(), RuleError> {
        for field in [0, 2, 4, 6] {
            let source = warrior(member)
                .wrapping_add(0xa76)
                .wrapping_add((i32::from(slot) * 8) as u32)
                .wrapping_add(field);
            let target = 0x43788a_u32
                .wrapping_add(u32::from(member) * 80)
                .wrapping_add((i32::from(slot) * 8) as u32)
                .wrapping_add(field);
            self.word(target, self.w(source)?)?;
        }
        Ok(())
    }
}

impl<S: CharacterUpgradeServices + ?Sized> Execution<'_, S> {
    fn upgrade_hidden_rows(&mut self, sp: &mut u32, member: u16) -> Result<(), RuleError> {
        self.word(0x4434aa, 71)?;
        while self.sw(0x4434aa)? < 80 {
            self.step()?;
            if self.sw(index(
                warrior(member) + 0xf4,
                self.sw(0x4434aa)?.wrapping_sub(71),
                2,
            ))? < 1
            {
                self.gitem(
                    self.l(0x4394c4)?,
                    self.w(0x4434aa)?.wrapping_add(19),
                    0x44292c,
                )?;
                self.pict_call(sp, 181, 0x44292c)?;
                *sp = sp.wrapping_add(10);
            }
            self.add(0x4434aa, 1)?;
        }
        Ok(())
    }
    fn upgrade_spell_icon(
        &mut self,
        sp: &mut u32,
        member: u16,
        get_item: bool,
    ) -> Result<(), RuleError> {
        if self.w(warrior(member) + 0x36)? == 0 {
            if get_item {
                self.gitem(self.l(0x4394c4)?, 99, 0x44292c)?;
            }
            self.pict_call(sp, 181, 0x44292c)?;
            self.string_call(sp, 26, 0x4005f6)?;
            *sp = sp.wrapping_add(16);
        } else {
            self.pict_call(sp, 151, 0x44292c)?;
            *sp = sp.wrapping_add(10);
        }
        Ok(())
    }
    fn upgrade_cost(&mut self, frame: u32, sp: &mut u32, member: u16) -> Result<(), RuleError> {
        let ext = frame.wrapping_sub(24);
        let single = frame.wrapping_sub(14);
        let work = frame.wrapping_sub(10);
        self.word(ext, self.w(index(0x402f4a, self.sw(0x4042da)?, 2))?)?;
        self.sane(&[ext, ext], 0x200e)?;
        self.sane(&[ext, ext], 0x1010)?;
        self.long(0x4042ce, self.l(ext)?)?;
        self.word(ext, self.w(index(0x402fae, self.sw(0x4042da)?, 2))?)?;
        self.sane(&[ext, ext], 0x200e)?;
        self.sane(&[ext, ext], 0x1010)?;
        self.long(0x4042d6, self.l(ext)?)?;
        self.long(ext, 0xbfff8000)?;
        self.long(ext + 4, 0)?;
        self.word(ext + 8, 0)?;
        let ccr = self.sane(&[0x4042d6, ext], 0x1008)?;
        if ccr & 4 != 0 {
            let address = match self.w(0x4042da)? {
                7 => Some(0x41fad6),
                16 => Some(0x41fae2),
                17 => Some(0x41fade),
                18 => Some(0x41faea),
                19 => Some(0x41faee),
                20 => Some(0x41fad2),
                21 => Some(0x41fafa),
                22 => Some(0x41faf8),
                23..=31 => Some(index(0x41fa0e, self.sw(0x4042da)?.wrapping_sub(23), 2)),
                32 => Some(0x41fae6),
                _ => None,
            };
            if let Some(a) = address {
                self.word(ext, self.w(a)?)?;
                self.sane(&[ext, ext], 0x200e)?;
                self.sane(&[ext, ext], 0x1010)?;
                self.long(0x4042d6, self.l(ext)?)?;
            }
        }
        self.long(0x4042ca, 0)?;
        self.word(0x4434aa, 0)?;
        while self.sw(0x4434aa)? < self.sw(index(warrior(member) + 0xace, self.sw(0x4042da)?, 2))? {
            self.step()?;
            self.sane(&[0x4042ca, ext], 0x100e)?;
            self.sane(&[0x4042ce, ext], 0x1000)?;
            self.sane(&[ext, 0x4042ca], 0x1010)?;
            self.long(ext, 0x4005c800)?;
            self.long(ext + 4, 0)?;
            self.word(ext + 8, 0)?;
            self.sane(&[0x4042d6, ext], 0x1000)?;
            self.sane(&[ext, single], 0x1010)?;
            self.long(0x4042d2, self.l(single)?)?;
            self.long(ext, 0x4005c800)?;
            self.long(ext + 4, 0)?;
            self.word(ext + 8, 0)?;
            self.sane(&[0x4042d2, work], 0x100e)?;
            self.sane(&[ext, work], 6)?;
            self.sane(&[work, single], 0x1010)?;
            self.long(0x4042d2, self.l(single)?)?;
            self.sane(&[0x4042ce, work], 0x100e)?;
            self.sane(&[0x4042d2, work], 0x1004)?;
            self.sane(&[work, single], 0x1010)?;
            self.long(0x4042ce, self.l(single)?)?;
            self.add(0x4434aa, 1)?;
        }
        self.long(single, 0x4c3ebc20)?;
        self.sane(&[0x4042ce, work], 0x100e)?;
        let ccr = self.sane(&[single, work], 0x1008)?;
        if ccr & 4 == 0 && ((ccr >> 3) & 1) == ((ccr >> 1) & 1) {
            self.long(0x4042ce, 0x4c3ebc20)?;
        }
        self.sane(&[0x4042ce, work], 0x100e)?;
        self.sane(&[work], 0x16)?;
        self.sane(&[work, work], 0x2810)?;
        self.long(0x4427b6, self.l(work)?)?;
        self.long(work, self.l(0x4427b6)?)?;
        self.sane(&[work, work], 0x280e)?;
        self.sane(&[work, work], 0x1010)?;
        self.long(0x4042ce, self.l(work)?)?;
        self.long_number_call(sp, 44, self.l(0x4427b6)?)?;
        self.sane(&[0x4042d6, work], 0x100e)?;
        self.sane(&[work], 0x16)?;
        self.sane(&[work, work], 0x2810)?;
        self.long(0x4427b6, self.l(work)?)?;
        self.long_number_call(sp, 46, self.l(0x4427b6)?)?;
        *sp = sp.wrapping_add(12);
        Ok(())
    }
    fn upgrade_warrior(&mut self, frame: u32) -> Result<(), RuleError> {
        let mut sp = frame.wrapping_sub(28);
        let mut member = self.w(frame + 8)?;
        self.location(&mut sp, 400)?;
        sp = sp.wrapping_add(2);
        if self.w(0x4046ea)? == 0 && self.w(0x4471ec)? == 1 {
            self.word(0x4434aa, 0)?;
            while self.sw(0x4434aa)? < self.sb(0x44273a)? {
                self.step()?;
                let n = self.w(0x4434aa)?;
                self.copy(warrior(n), 0x44ecb0 + u32::from(n) * 3138, 3138)?;
                self.add(0x4434aa, 1)?;
            }
            self.flash(sp.wrapping_sub(20), 0x4005f8, 75, 75, 0, 5000)?;
        }
        self.sound_call(&mut sp, 647)?;
        self.location(&mut sp, 401)?;
        let dialog = self.trap(0xa97c, vec![W(155), L(0), L(u32::MAX)])?;
        sp = sp.wrapping_add(4);
        self.long(0x4394c4, dialog)?;
        self.byte(0x43565a, 1)?;
        self.gitem(self.l(0x4394c4)?, 48, 0x4042dc)?;
        self.port(self.l(0x4394c4)?)?;
        self.trap(0xaa0b, vec![L(self.l(0x4358ce)?)])?;
        self.trap(0xa915, vec![L(self.l(0x4394c4)?)])?;
        self.location(&mut sp, 402)?;
        for op in [0xa981, 0xa922, 0xa923] {
            self.trap(op, vec![L(self.l(0x4394c4)?)])?;
        }
        sp = sp.wrapping_add(2);
        'refresh: loop {
            self.step()?;
            self.long(0x4394b8, self.l(0x4394c4)?)?;
            self.port(self.l(0x4394c4)?)?;
            self.trap(0xaa1c, vec![L(self.l(0x43952c)?)])?;
            self.location(&mut sp, 403)?;
            self.weight_call(&mut sp, member)?;
            self.stats_call(&mut sp, member)?;
            self.byte(0x44273f, member as u8)?;
            self.location(&mut sp, 404)?;
            self.copy(warrior(member), 0x44721c, 3138)?;
            self.copy(0x44721c, 0x44e06e, 3138)?;
            self.copy(0x44e06e, 0x44d42c, 3138)?;
            self.pw(&mut sp, self.w(0x447250)?)?;
            self.pw(&mut sp, self.w(0x44724e)?)?;
            self.profile(sp.wrapping_sub(8))?;
            sp = sp.wrapping_add(12);
            self.upgrade_hidden_rows(&mut sp, member)?;
            self.trap(0xa862, vec![L(69)])?;
            self.upgrade_spell_icon(&mut sp, member, true)?;
            self.location(&mut sp, 405)?;
            self.long_number_call(&mut sp, 43, self.l(warrior(member) + 0x9d2)?)?;
            self.gitem(self.l(0x4394c4)?, 89, 0x4434ba)?;
            self.pict_call(&mut sp, 195, 0x4434ba)?;
            self.runtime(0x13e13c)?;
            self.ui_call(&mut sp, member, true)?;
            self.gitem(self.l(0x4394c4)?, 3, 0x4434ba)?;
            self.portrait_call(&mut sp, member, 0x4434ba, true)?;
            sp = sp.wrapping_add(36);
            self.upgrade_hidden_rows(&mut sp, member)?;
            self.gitem(self.l(0x4394c4)?, 99, 0x44292c)?;
            self.upgrade_spell_icon(&mut sp, member, false)?;
            if self.w(0x4471ec)? != 0 {
                self.gitem(self.l(0x4394c4)?, 100, 0x44292c)?;
                self.pict_call(&mut sp, 233, 0x44292c)?;
                self.number_call(&mut sp, 101, self.w(warrior(member) + 0x3a)?)?;
                sp = sp.wrapping_add(14);
            }
            'updates: loop {
                self.step()?;
                for op in [0xa922, 0xa923] {
                    self.trap(op, vec![L(self.l(0x4394c4)?)])?;
                }
                let mut modal = true;
                if self.b(0x479fe0)? == 0 {
                    self.byte(0x479fe0, 1)?;
                    self.word(0x4434b2, 48)?;
                    self.gitem(self.l(0x4394c4)?, 48, 0x44292c)?;
                    modal = false;
                }
                loop {
                    self.step()?;
                    let mut cost = false;
                    if modal {
                        self.first_call(&mut sp, 0x00310037)?;
                        sp = sp.wrapping_add(4);
                        if !self.active() {
                            return Ok(());
                        }
                        if self.w(0x4471ec)? != 0 {
                            self.first_call(&mut sp, 0x008b008b)?;
                            sp = sp.wrapping_add(4);
                            if !self.active() {
                                return Ok(());
                            }
                        }
                        self.flush()?;
                        self.trap(0xa991, vec![L(0), P(0x4434b2)])?;
                        self.gitem(self.l(0x4394c4)?, self.w(0x4434b2)?, 0x44292c)?;
                        if self.w(0x4434b2)? == 84 || self.w(0x4434b2)? == 85 {
                            self.byte(0x479fe0, 0)?;
                            member = member.wrapping_add(1);
                            if self.w(0x4434b2)? == 84 {
                                member = member.wrapping_sub(2);
                            }
                            if member as i16 > self.sb(0x44273a)? {
                                member = 0;
                            } else if (member as i16) < 0 {
                                member = self.sb(0x44273a)? as u16;
                            }
                            continue 'refresh;
                        }
                        if self.w(0x4434b2)? == 82 || self.w(0x4434b2)? == 83 {
                            let reset = self.w(0x4434b2)? == 83;
                            self.sound_call(&mut sp, if reset { 133 } else { 138 })?;
                            self.copy(
                                if reset { 0x44e06e } else { 0x44d42c },
                                warrior(member),
                                3138,
                            )?;
                            self.stats_call(&mut sp, member)?;
                            self.ui_call(&mut sp, member, true)?;
                            sp = sp.wrapping_add(6);
                            cost = true;
                        } else if self.w(0x4434b2)? == 39 {
                            if self.w(warrior(member) + 0xac6)? == 0 {
                                self.flash(sp.wrapping_sub(20), 0x4005d4, 50, 50, 0, 6000)?;
                                continue 'updates;
                            }
                            self.copy(warrior(member), 0x44721c, 3138)?;
                            let child = selection::scan_select(
                                &self.plan.state,
                                sp.wrapping_sub(8),
                                self.selection_limits(),
                                self.s,
                            )?;
                            self.merge_selection(child);
                            if !self.active() {
                                return Ok(());
                            }
                            self.copy(0x44721c, warrior(member), 3138)?;
                            self.word(0x4434ac, 0)?;
                            while self.sw(0x4434ac)? < 10 {
                                self.step()?;
                                self.copied_scans(member, self.sw(0x4434ac)?)?;
                                self.add(0x4434ac, 1)?;
                            }
                            self.long(0x4394b8, self.l(0x4394c4)?)?;
                            self.number_call(&mut sp, 101, self.w(warrior(member) + 0x3a)?)?;
                            self.trap(0xa981, vec![L(self.l(0x4394c4)?)])?;
                            sp = sp.wrapping_add(4);
                            continue 'refresh;
                        }
                    }
                    modal = true;
                    if !cost && self.sw(0x4434b2)? >= 48 && self.sw(0x4434b2)? <= 80 {
                        if self.sw(0x4434b2)? >= 71
                            && self.sw(0x4434b2)? <= 79
                            && self.sw(index(
                                warrior(member) + 0xf4,
                                self.sw(0x4434b2)?.wrapping_sub(71),
                                2,
                            ))? < 1
                        {
                            self.warn(sp.wrapping_sub(10), 150)?;
                            continue 'updates;
                        }
                        if self.w(0x4434b2)? == 69 && self.w(warrior(member) + 0x36)? == 0 {
                            self.warn(sp.wrapping_sub(10), 150)?;
                            continue 'updates;
                        }
                        self.word(0x4042da, self.w(0x4434b2)?.wrapping_sub(48))?;
                        self.long(0x4042e4, self.l(0x44292c)?)?;
                        self.long(0x4042e8, self.l(0x442930)?)?;
                        self.long(0x44292c, self.l(0x4042dc)?)?;
                        self.long(0x442930, self.l(0x4042e0)?)?;
                        self.edge_call(&mut sp, true)?;
                        self.long(0x44292c, self.l(0x4042e4)?)?;
                        self.long(0x442930, self.l(0x4042e8)?)?;
                        self.edge_call(&mut sp, false)?;
                        self.sound_call(&mut sp, (-141i16) as u16)?;
                        self.long(0x4042dc, self.l(0x44292c)?)?;
                        self.long(0x4042e0, self.l(0x442930)?)?;
                        self.gitem(self.l(0x4394c4)?, 42, 0x44292c)?;
                        self.lib(
                            0x400178,
                            vec![P(0x479ed8), W(143), W(self.w(0x4042da)?.wrapping_add(1))],
                        )?;
                        self.string_call(&mut sp, 42, 0x479ed8)?;
                        self.gitem(self.l(0x4394c4)?, 42, 0x44292c)?;
                        self.lib(
                            0x400178,
                            vec![P(0x479ed8), W(144), W(self.w(0x4042da)?.wrapping_add(1))],
                        )?;
                        self.string_call(&mut sp, 47, 0x479ed8)?;
                        sp = sp.wrapping_add(18);
                        cost = true;
                    }
                    loop {
                        if cost {
                            self.upgrade_cost(frame, &mut sp, member)?;
                        }
                        if self.w(0x4434b2)? != 45 {
                            break;
                        }
                        let work = frame.wrapping_sub(10);
                        let ext = frame.wrapping_sub(24);
                        self.long(work, self.l(warrior(member) + 0x9d2)?)?;
                        self.sane(&[work, work], 0x280e)?;
                        self.sane(&[work, work], 0x1010)?;
                        self.sane(&[work, ext], 0x100e)?;
                        let ccr = self.sane(&[0x4042ce, ext], 0x1008)?;
                        if ((ccr >> 3) & 1) != ((ccr >> 1) & 1) && self.w(0x4471ec)? != 1 {
                            break;
                        }
                        self.sound_call(&mut sp, 10129)?;
                        self.copy(warrior(member), 0x44d42c, 3138)?;
                        sp = sp.wrapping_add(2);
                        if self.sw(index(warrior(member) + 0xace, self.sw(0x4042da)?, 2))? < 0 {
                            self.warn(sp.wrapping_sub(10), 149)?;
                            continue 'updates;
                        }
                        if self.w(0x4471ec)? != 1 {
                            self.sane(&[warrior(member) + 0x9d2, work], 0x280e)?;
                            self.sane(&[0x4042ce, work], 0x1002)?;
                            self.sane(&[work], 0x16)?;
                            self.sane(&[work, work], 0x2810)?;
                            self.long(warrior(member) + 0x9d2, self.l(work)?)?;
                        }
                        self.add(index(warrior(member) + 0xace, self.sw(0x4042da)?, 2), 1)?;
                        self.long_number_call(&mut sp, 43, self.l(warrior(member) + 0x9d2)?)?;
                        self.word(0x4434b2, 0)?;
                        self.apply_upgrade_call(&mut sp, member)?;
                        self.stats_call(&mut sp, member)?;
                        self.ui_call(&mut sp, member, true)?;
                        sp = sp.wrapping_add(16);
                        cost = true;
                    }
                    if self.w(0x4434b2)? == 81 && self.w(0x4471ec)? == 1 {
                        self.pw(&mut sp, 2)?;
                        self.pw(&mut sp, member)?;
                        let child = creation::level_up(
                            &self.plan.state,
                            sp.wrapping_sub(8),
                            self.limits,
                            self.s,
                        )?;
                        self.merge_creation(child);
                        if !self.active() {
                            return Ok(());
                        }
                        self.long(0x4394b8, self.l(0x4394c4)?)?;
                        self.number_call(&mut sp, 101, self.w(warrior(member) + 0x3a)?)?;
                        self.trap(0xa981, vec![L(self.l(0x4394c4)?)])?;
                        sp = sp.wrapping_add(8);
                        continue 'refresh;
                    }
                    if self.w(0x4434b2)? == 1 || self.w(0x4434b2)? == 4 {
                        self.gitem(self.l(0x4394c4)?, self.w(0x4434b2)?, 0x4434ba)?;
                        self.press_call(&mut sp, 1, 2, 0, 0x4434ba)?;
                        self.sound_call(&mut sp, 141)?;
                        self.trap(0xa983, vec![L(self.l(0x4394c4)?)])?;
                        sp = sp.wrapping_add(16);
                        if self.w(0x4434b2)? == 1 {
                            self.sound_call(&mut sp, 133)?;
                            self.copy(0x44e06e, warrior(member), 3138)?;
                            self.stats_call(&mut sp, member)?;
                            sp = sp.wrapping_add(4);
                        }
                        self.byte(0x43565a, 0)?;
                        if self.w(0x4046ea)? == 0 && self.w(0x4471ec)? == 1 {
                            self.word(0x4434aa, 0)?;
                            while self.sw(0x4434aa)? < self.sb(0x44273a)? {
                                self.step()?;
                                let n = self.w(0x4434aa)?;
                                self.copy(0x44ecb0 + u32::from(n) * 3138, warrior(n), 3138)?;
                                self.add(0x4434aa, 1)?;
                            }
                            self.flash(sp.wrapping_sub(20), 0x4005b0, 75, 75, 0, 5000)?;
                        }
                        self.full(0);
                        return Ok(());
                    }
                }
            }
        }
    }
}

impl<S: CharacterUpgradeServices + ?Sized> Execution<'_, S> {
    fn member_word(member: u8) -> u16 {
        i16::from(member as i8) as u16
    }
    fn statistics_total(&self, member: u16) -> Result<u32, RuleError> {
        let base = warrior(member);
        let mut total = self
            .l(base + 0xa4a)?
            .wrapping_sub(self.l(base + 0xa4e)?.wrapping_mul(2));
        total = total.wrapping_add(
            self.l(base + 0xa56)?
                .wrapping_sub(self.l(base + 0xa52)?.wrapping_mul(2)),
        );
        total = total.wrapping_add(
            self.l(base + 0xa6e)?
                .wrapping_mul(6)
                .wrapping_add(self.l(base + 0xa66)?.wrapping_mul((-3i32) as u32))
                .wrapping_add(self.l(base + 0xa6a)?.wrapping_mul(4)),
        );
        total = total.wrapping_add(
            self.l(base + 0xa5e)?
                .wrapping_mul((-75i32) as u32)
                .wrapping_add(self.l(base + 0xa5a)?.wrapping_mul(3))
                .wrapping_add(self.l(base + 0xa62)?.wrapping_mul((-35i32) as u32)),
        );
        total = total.wrapping_add(
            ((self.l(base + 0xa46)?.wrapping_sub(self.l(base + 0xa42)?) as i32) / 20) as u32,
        );
        Ok(total.wrapping_sub(self.l(base + 0xa72)?))
    }
    fn statistics_dialog(&mut self, sp: &mut u32, member: &mut u8) -> Result<(), RuleError> {
        self.edge_call(sp, false)?;
        let sound = self.rand_call(sp, 2)?.wrapping_add(200);
        *sp = sp.wrapping_add(2);
        self.sound_call(sp, sound)?;
        let dialog = self.trap(0xa97c, vec![W(177), L(0), L(u32::MAX)])?;
        *sp = sp.wrapping_add(4);
        self.port(dialog)?;
        self.long(0x4394b8, dialog)?;
        self.trap(0xaa0b, vec![L(self.l(0x4358ce)?)])?;
        self.trap(0xa862, vec![L(30)])?;
        for op in [0xa915, 0xa981, 0xa922, 0xa923] {
            self.trap(op, vec![L(dialog)])?;
        }
        self.word(0x4434aa, 0)?;
        while self.sw(0x4434aa)? <= self.sb(0x44273a)? {
            self.step()?;
            self.gitem(dialog, self.w(0x4434aa)?.wrapping_add(78), 0x4434ba)?;
            self.portrait_call(sp, self.w(0x4434aa)?, 0x4434ba, false)?;
            *sp = sp.wrapping_add(16);
            if Self::member_word(*member) == self.w(0x4434aa)? {
                self.gitem(dialog, self.w(0x4434aa)?.wrapping_add(72), 0x4434ba)?;
                self.press_call(sp, 1, 2, 0, 0x4434ba)?;
                *sp = sp.wrapping_add(14);
            }
            self.add(0x4434aa, 1)?;
        }
        'redraw: loop {
            self.step()?;
            self.gitem(dialog, 84, 0x4434ba)?;
            self.pict_call(sp, 154, 0x4434ba)?;
            self.gitem(dialog, 86, 0x4434ba)?;
            self.portrait_call(sp, Self::member_word(*member), 0x4434ba, true)?;
            self.gitem(dialog, 85, 0x4434ba)?;
            self.pict_call(sp, 154, 0x4434ba)?;
            self.gitem(dialog, 87, 0x4434ba)?;
            self.icon_call(sp, Self::member_word(*member), 0x4434ba)?;
            self.trap(0xa88a, vec![W(12)])?;
            self.trap(0xa887, vec![W(self.w(0x402f3c)?)])?;
            self.trap(0xa862, vec![L(69)])?;
            self.pl(sp, warrior(Self::member_word(*member)) + 0xa21)?;
            self.lib(
                0x400188,
                vec![P(warrior(Self::member_word(*member)) + 0xa21)],
            )?;
            self.string_call(sp, 37, warrior(Self::member_word(*member)) + 0xa21)?;
            self.pl(sp, warrior(Self::member_word(*member)) + 0xa21)?;
            self.lib(
                0x400180,
                vec![P(warrior(Self::member_word(*member)) + 0xa21)],
            )?;
            self.trap(0xa887, vec![W(self.w(0x402f44)?)])?;
            self.trap(0xa888, vec![W(0)])?;
            self.long_number_call(sp, 6, self.statistics_total(Self::member_word(*member))?)?;
            for (offset, item, negative) in [
                (0xa46, 26, false),
                (0xa4a, 27, false),
                (0xa56, 42, false),
                (0xa5a, 28, false),
                (0xa6a, 30, false),
                (0xa6e, 35, false),
                (0xa72, 71, true),
                (0xa66, 33, false),
                (0xa42, 31, false),
                (0xa4e, 32, false),
                (0xa52, 43, false),
                (0xa62, 29, false),
                (0xa5e, 34, false),
            ] {
                let v = self.l(warrior(Self::member_word(*member)) + offset)?;
                self.long_number_call(sp, item, if negative { v.wrapping_neg() } else { v })?;
            }
            self.flush()?;
            self.first_call(sp, 0x002a002a)?;
            *sp = sp.wrapping_add(148);
            if !self.active() {
                return Ok(());
            }
            loop {
                self.step()?;
                self.trap(0xa991, vec![L(0), P(0x4434b2)])?;
                if self.sw(0x4434b2)? > 71
                    && self.sw(0x4434b2)? < 78
                    && self.sw(0x4434b2)?.wrapping_sub(72) <= self.sb(0x44273a)?
                {
                    self.gitem(
                        dialog,
                        Self::member_word(*member).wrapping_add(72),
                        0x4434ba,
                    )?;
                    self.press_call(sp, 1, 1, 0, 0x4434ba)?;
                    self.gitem(dialog, self.w(0x4434b2)?, 0x44292c)?;
                    self.press_call(sp, 1, 2, 141, 0x44292c)?;
                    *member = self.w(0x4434b2)?.wrapping_sub(72) as u8;
                    *sp = sp.wrapping_add(28);
                    continue 'redraw;
                }
                if self.w(0x4434b2)? == 58 {
                    self.sound_call(sp, 214)?;
                    self.trap(0xa983, vec![L(dialog)])?;
                    *sp = sp.wrapping_add(2);
                    return Ok(());
                }
            }
        }
    }
    fn view_character(&mut self, frame: u32) -> Result<(), RuleError> {
        let mut sp = frame.wrapping_sub(34);
        let mut member = self.b(frame + 9)?;
        let mode = self.b(frame + 11)?;
        self.copy(0x4005a6, frame.wrapping_sub(10), 10)?;
        let dialog = self.trap(0xa97c, vec![W(138), L(0), L(u32::MAX)])?;
        self.long(0x4394bc, dialog)?;
        'refresh: loop {
            self.step()?;
            self.port(self.l(0x4394bc)?)?;
            self.trap(0xaa0b, vec![L(self.l(0x4358ce)?)])?;
            for op in [0xa915, 0xa981, 0xa922, 0xa923] {
                self.trap(op, vec![L(self.l(0x4394bc)?)])?;
            }
            self.long(0x4394b8, self.l(0x4394bc)?)?;
            self.port(self.l(0x4394bc)?)?;
            self.trap(0xaa1c, vec![L(self.l(0x43952c)?)])?;
            let first = self.rand_call(&mut sp, 5)?;
            let second = self.rand_call(&mut sp, 5)?;
            self.sound_call(
                &mut sp,
                self.w(index(
                    frame.wrapping_sub(10),
                    first.wrapping_sub(1) as i16,
                    2,
                ))?,
            )?;
            self.weight_call(&mut sp, Self::member_word(member))?;
            self.copy(warrior(Self::member_word(member)), 0x44721c, 3138)?;
            self.runtime(0x13e13c)?;
            self.byte(0x44273f, member)?;
            self.ui_call(&mut sp, 1, false)?;
            self.gitem(self.l(0x4394bc)?, 68, 0x4434ba)?;
            self.portrait_call(&mut sp, Self::member_word(member), 0x4434ba, true)?;
            self.gitem(self.l(0x4394bc)?, 80, 0x4434ba)?;
            self.icon_call(&mut sp, Self::member_word(member), 0x4434ba)?;
            sp = sp.wrapping_add(36);
            self.word(0x4434aa, 0)?;
            while self.sw(0x4434aa)? <= self.sb(0x44273a)? {
                self.step()?;
                self.gitem(
                    self.l(0x4394bc)?,
                    self.w(0x4434aa)?.wrapping_add(73),
                    0x4434ba,
                )?;
                self.trap(0xa8a9, vec![P(0x4434ba), L(0x00090009)])?;
                self.portrait_call(&mut sp, self.w(0x4434aa)?, 0x4434ba, false)?;
                self.gitem(
                    self.l(0x4394bc)?,
                    self.w(0x4434aa)?.wrapping_add(84),
                    0x4434ba,
                )?;
                self.icon_call(&mut sp, self.w(0x4434aa)?, 0x4434ba)?;
                self.add(0x4434aa, 1)?;
                sp = sp.wrapping_add(26);
            }
            if first != second {
                self.sound_call(
                    &mut sp,
                    self.w(index(
                        frame.wrapping_sub(10),
                        second.wrapping_sub(1) as i16,
                        2,
                    ))?,
                )?;
                sp = sp.wrapping_add(2);
            }
            self.first_call(&mut sp, 0x00260028)?;
            sp = sp.wrapping_add(4);
            if !self.active() {
                return Ok(());
            }
            if self.w(0x4471ec)? != 0 {
                self.first_call(&mut sp, 0x008a008a)?;
                sp = sp.wrapping_add(4);
                if !self.active() {
                    return Ok(());
                }
            }
            loop {
                self.step()?;
                self.flush()?;
                self.trap(0xa991, vec![L(0), P(0x4434b2)])?;
                self.gitem(self.l(0x4394bc)?, self.w(0x4434b2)?, 0x44292c)?;
                if self.w(0x4434b2)? == 3 && self.l(0x43879c)? != 0 {
                    self.edge_call(&mut sp, false)?;
                    let result =
                        self.condition_call(&mut sp, Self::member_word(member), 0x00010000)?;
                    if !self.active() {
                        return Ok(());
                    }
                    sp = sp.wrapping_add(12);
                    if (result as i16) > -1 {
                        self.copy(warrior(Self::member_word(member)), 0x44721c, 3138)?;
                        self.ui_call(&mut sp, 1, false)?;
                        sp = sp.wrapping_add(2);
                    }
                    self.edge_call(&mut sp, true)?;
                    sp = sp.wrapping_add(2);
                }
                if self.w(0x4434b2)? == 2 && self.l(0x43879c)? != 0 {
                    self.edge_call(&mut sp, false)?;
                    self.condition_call(&mut sp, Self::member_word(member), 0)?;
                    if !self.active() {
                        return Ok(());
                    }
                    self.edge_call(&mut sp, true)?;
                    sp = sp.wrapping_add(14);
                }
                if self.w(0x4434b2)? == 4 && self.l(0x43879c)? != 0 {
                    self.first_call(&mut sp, 0x00300030)?;
                    if !self.active() {
                        return Ok(());
                    }
                    self.edge_call(&mut sp, false)?;
                    self.copy(warrior(Self::member_word(member)), 0x44721c, 3138)?;
                    self.special_call(sp.wrapping_sub(8))?;
                    self.long(0x4394b8, self.l(0x4394bc)?)?;
                    self.ui_call(&mut sp, 1, false)?;
                    self.edge_call(&mut sp, true)?;
                    sp = sp.wrapping_add(10);
                }
                if self.w(0x4434b2)? == 5 && self.l(0x43879c)? != 0 {
                    self.edge_call(&mut sp, false)?;
                    self.condition_call(&mut sp, Self::member_word(member), 0x00020000)?;
                    if !self.active() {
                        return Ok(());
                    }
                    self.edge_call(&mut sp, true)?;
                    sp = sp.wrapping_add(14);
                }
                if self.sw(0x4434b2)? > 83
                    && self.sw(0x4434b2)? < 90
                    && mode != 1
                    && self.sw(0x4434b2)?.wrapping_sub(84) <= self.sb(0x44273a)?
                {
                    self.add(0x4434b2, -11)?;
                }
                if self.w(0x4434b2)? == 105 || self.w(0x4434b2)? == 65 {
                    self.pw(&mut sp, Self::member_word(member))?;
                    self.upgrade_warrior(sp.wrapping_sub(8))?;
                    if !self.active() {
                        return Ok(());
                    }
                    sp = sp.wrapping_add(2);
                    continue 'refresh;
                }
                if self.w(0x4434b2)? == 97 || self.w(0x4434b2)? == 96 {
                    let race = self.w(0x4434b2)? == 97;
                    self.first_call(&mut sp, if race { 0x002e002e } else { 0x002f002f })?;
                    if !self.active() {
                        return Ok(());
                    }
                    self.copy(warrior(Self::member_word(member)), 0x44721c, 3138)?;
                    self.pw(&mut sp, 1)?;
                    let child = if race {
                        creation::race(&self.plan.state, sp.wrapping_sub(8), self.limits, self.s)?
                    } else {
                        creation::caste(&self.plan.state, sp.wrapping_sub(8), self.limits, self.s)?
                    };
                    self.merge_creation(child);
                    if !self.active() {
                        return Ok(());
                    }
                    sp = sp.wrapping_add(6);
                    continue 'refresh;
                }
                if self.w(0x4434b2)? == 100 {
                    self.statistics_dialog(&mut sp, &mut member)?;
                    if !self.active() {
                        return Ok(());
                    }
                    continue 'refresh;
                }
                if self.sw(0x4434b2)? > 72
                    && self.sw(0x4434b2)? < 79
                    && mode != 1
                    && self.sw(0x4434b2)?.wrapping_sub(73) <= self.sb(0x44273a)?
                {
                    self.sound_call(&mut sp, 208)?;
                    member = self.w(0x4434b2)?.wrapping_sub(73) as u8;
                    self.gitem(self.l(0x4394bc)?, 15, 0x4434ba)?;
                    self.pict_call(&mut sp, 154, 0x4434ba)?;
                    self.weight_call(&mut sp, Self::member_word(member))?;
                    self.copy(warrior(Self::member_word(member)), 0x44721c, 3138)?;
                    self.ui_call(&mut sp, 1, false)?;
                    self.gitem(self.l(0x4394bc)?, 68, 0x4434ba)?;
                    self.portrait_call(&mut sp, Self::member_word(member), 0x4434ba, true)?;
                    self.gitem(self.l(0x4394bc)?, 79, 0x4434ba)?;
                    self.pict_call(&mut sp, 154, 0x4434ba)?;
                    self.gitem(self.l(0x4394bc)?, 80, 0x4434ba)?;
                    self.icon_call(&mut sp, Self::member_word(member), 0x4434ba)?;
                    sp = sp.wrapping_add(52);
                }
                if self.w(0x4434b2)? == 72 || self.w(0x4434b2)? == 1 {
                    self.gitem(self.l(0x4394bc)?, 72, 0x4434ba)?;
                    self.press_call(&mut sp, 1, 2, 0, 0x4434ba)?;
                    self.sound_call(&mut sp, 141)?;
                    self.trap(0xa983, vec![L(self.l(0x4394bc)?)])?;
                    self.byte(0x44273f, member)?;
                    self.full(0);
                    return Ok(());
                }
            }
        }
    }
}

impl<S: CharacterUpgradeServices + ?Sized> Execution<'_, S> {
    fn lib_call(
        &mut self,
        sp: &mut u32,
        entry: u32,
        arguments: Vec<FeedbackArg>,
    ) -> Result<u32, RuleError> {
        for argument in &arguments {
            match *argument {
                W(v) => self.pw(sp, v)?,
                B(v) => self.pw(sp, u16::from(v) << 8)?,
                L(v) | P(v) => self.pl(sp, v)?,
            }
        }
        self.lib(entry, arguments)
    }
    fn filename_call(&mut self, sp: &mut u32, pointer: u32) -> Result<(), RuleError> {
        self.pl(sp, pointer)?;
        let limits = StorageLimits {
            filename_capacity: self.limits.filename_capacity,
            maximum_iterations: self.remaining as usize,
            maximum_polls: self.limits.maximum_polls,
        };
        let mut child = storage::Execution::new(&self.plan.state.memory, limits, self.d0);
        child.filename(pointer, false, self.s)?;
        self.plan.state.memory = child.plan.memory;
        self.plan.effects.extend(
            child
                .plan
                .effects
                .into_iter()
                .map(CharacterUpgradeEffect::Storage),
        );
        self.d0 = child.plan.native_return;
        Ok(())
    }
    fn set_file_info(&mut self, frame: u32, name: u32, kind: u32) -> Result<(), RuleError> {
        self.long(frame + 12, name)?;
        self.long(frame + 8, kind)?;
        self.lib(0x400188, vec![P(name)])?;
        self.lib(0x400150, vec![P(name), W(0), P(0x439cfc)])?;
        self.lib(0x400090, vec![P(self.l(frame + 8)?), P(0x439cfc)])?;
        self.lib(0x400090, vec![P(0x402436), P(0x439d00)])?;
        self.lib(0x400158, vec![P(name), W(0), P(0x439cfc)])?;
        self.lib(0x400180, vec![P(name)])?;
        self.full(0);
        Ok(())
    }
    fn scratch_call(&mut self, sp: &mut u32, code: u16, fatal: bool) -> Result<(), RuleError> {
        self.pw(sp, code)?;
        let child = if fatal {
            load::scratch2(&self.plan.state.memory, sp.wrapping_sub(8), self.s)?
        } else {
            load::scratch(&self.plan.state.memory, sp.wrapping_sub(8), self.s)?
        };
        self.plan.state.memory = child.memory;
        self.plan.flow = child.flow;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(|e| CharacterUpgradeEffect::Prompt(PromptControlEffect::Load(e))),
        );
        self.d0 = child.native_return;
        self.plan.native_return = if self.active() {
            child
                .native_return
                .map_or(PromptReturn::Unchanged, PromptReturn::Full)
        } else {
            PromptReturn::NoReturn
        };
        Ok(())
    }
    fn panel_short(&mut self, sp: &mut u32) -> Result<(), RuleError> {
        self.pw(sp, 1)?;
        let child = character::short_update(
            &self.plan.state.memory,
            sp.wrapping_sub(8),
            self.remaining,
            self.s,
        )?;
        self.plan.state.memory = child.memory;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(|e| CharacterUpgradeEffect::Prompt(PromptControlEffect::Character(e))),
        );
        self.full(child.native_return);
        Ok(())
    }
    fn panel_character(&mut self, sp: &mut u32, member: u16) -> Result<(), RuleError> {
        self.pw(sp, 5)?;
        self.pw(sp, member)?;
        let child = character::update_character(
            &self.plan.state.memory,
            sp.wrapping_sub(8),
            self.remaining,
            self.s,
        )?;
        self.plan.state.memory = child.memory;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(|e| CharacterUpgradeEffect::Prompt(PromptControlEffect::Character(e))),
        );
        self.full(child.native_return);
        Ok(())
    }
    fn restrictions(&mut self) -> Result<(), RuleError> {
        let state = self.ui_state();
        let child = character_ui::view_restrictions(&state, self.s)?;
        self.merge_ui(child);
        Ok(())
    }
    fn field_call(&mut self, frame: u32, save: bool) -> Result<(), RuleError> {
        let limits = field::FieldControlLimits {
            maximum_steps: self.remaining,
            maximum_polls: self.limits.maximum_polls,
            filename_capacity: self.limits.filename_capacity,
            maximum_random_draws: u64::from(self.remaining),
            equipment: self.selection_limits().equipment,
        };
        let child = if save {
            field::save_character(&self.plan.state, frame, limits, self.s)?
        } else {
            field::band_aid(&self.plan.state, frame, limits, self.s)?
        };
        self.plan.state = child.state;
        self.plan.flow = child.flow;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(CharacterUpgradeEffect::Field));
        self.returned(child.native_return);
        Ok(())
    }
    fn dialog_number(&mut self, sp: &mut u32, item: u16) -> Result<u16, RuleError> {
        self.pw(sp, item)?;
        let frame = sp.wrapping_sub(8);
        self.gitem(self.l(0x4394b8)?, self.w(frame + 8)?, 0x4434ba)?;
        self.trap(0xa990, vec![L(self.l(0x4434dc)?), P(0x479ed8)])?;
        self.lib(0x400160, vec![P(0x479ed8), P(0x4471dc)])?;
        let v = self.w(0x4471de)?;
        self.low(v);
        Ok(v)
    }
    fn get_text(&mut self, dialog: u32, item: u16) -> Result<(), RuleError> {
        self.gitem(dialog, item, 0x4434ba)?;
        self.trap(0xa990, vec![L(self.l(0x4434dc)?), P(0x479ed8)])?;
        Ok(())
    }
    fn warning_colors(&mut self, sp: &mut u32, id: u16, set_port: bool) -> Result<(), RuleError> {
        if set_port {
            self.port(self.l(0x4394d0)?)?;
        }
        self.trap(0xa862, vec![L(33)])?;
        self.trap(0xa863, vec![L(30)])?;
        self.pw(sp, id)?;
        self.warn(sp.wrapping_sub(8), id as i16)?;
        Ok(())
    }
    fn party_costs(&mut self, frame: u32, sp: &mut u32, selected: u16) -> Result<(), RuleError> {
        let ext = frame.wrapping_sub(182);
        let divisor = frame.wrapping_sub(172);
        let total = frame.wrapping_sub(162);
        self.word(ext, self.w(frame.wrapping_sub(120))?)?;
        self.sane(&[ext, ext], 0x200e)?;
        self.sane(&[ext, ext], 0x1010)?;
        self.long(frame.wrapping_sub(148), self.l(ext)?)?;
        self.long(ext, self.l(0x44276c)?)?;
        self.sane(&[ext, ext], 0x280e)?;
        self.sane(&[ext, ext], 0x1010)?;
        self.long(frame.wrapping_sub(144), self.l(ext)?)?;
        self.word(ext, self.w(0x442576)?)?;
        self.sane(&[ext, ext], 0x200e)?;
        self.long(divisor, 0x3ffda8f5)?;
        self.long(divisor + 4, 0xc28f5c28)?;
        self.word(divisor + 8, 0xf5c3)?;
        self.sane(&[ext, divisor], 4)?;
        self.long(total, 0x3fff8000)?;
        self.long(total + 4, 0)?;
        self.word(total + 8, 0)?;
        self.sane(&[divisor, total], 0)?;
        self.sane(&[total, frame.wrapping_sub(152)], 0x1010)?;
        self.long(0x442794, self.l(frame.wrapping_sub(152))?)?;
        let handle = self.lib_call(sp, 0x400070, vec![P(0x401afe), P(0x401b02)])?;
        *sp = sp.wrapping_add(18);
        if handle == 0 {
            self.scratch_call(sp, 131, false)?;
            if !self.active() {
                return Ok(());
            }
            *sp = sp.wrapping_add(2);
        }
        let offset = i32::from(self.sw(frame.wrapping_sub(106))?.wrapping_shl(5)) as u32;
        self.lib_call(sp, 0x400078, vec![W(0), L(offset), L(handle)])?;
        *sp = sp.wrapping_add(10);
        let mut item = 1u16;
        while (item as i16) < 14 {
            self.step()?;
            self.trap(0xa862, vec![L(if item == selected { 69 } else { 273 })])?;
            self.lib_call(
                sp,
                0x400048,
                vec![L(handle), L(1), L(30), P(frame.wrapping_sub(104))],
            )?;
            self.lib_call(
                sp,
                0x400048,
                vec![L(handle), L(1), L(2), P(frame.wrapping_sub(118))],
            )?;
            self.lib_call(sp, 0x400188, vec![P(frame.wrapping_sub(104))])?;
            *sp = sp.wrapping_add(36);
            if self.b(handle + 8)? == 0 {
                self.string_call(sp, item, frame.wrapping_sub(104))?;
                let width = self.trap(0xa88c, vec![P(frame.wrapping_sub(104))])? as u16;
                *sp = sp.wrapping_add(6);
                if width != 0 {
                    self.number_call(sp, item.wrapping_add(40), self.w(frame.wrapping_sub(118))?)?;
                    *sp = sp.wrapping_add(4);
                } else {
                    self.string_call(sp, item.wrapping_add(40), 0x401aa2)?;
                    *sp = sp.wrapping_add(6);
                }
            } else {
                self.string_call(sp, item, 0x401aa2)?;
                self.string_call(sp, item.wrapping_add(40), 0x401aa2)?;
                *sp = sp.wrapping_add(12);
            }
            item = item.wrapping_add(1);
        }
        self.lib_call(sp, 0x400068, vec![L(handle)])?;
        self.trap(0xa862, vec![L(69)])?;
        self.filename_call(sp, 0x401a9a)?;
        let handle = self.lib_call(sp, 0x400070, vec![P(0x401afe), P(0x4389a4)])?;
        *sp = sp.wrapping_add(16);
        if handle != 0 {
            self.lib_call(sp, 0x400048, vec![L(handle), L(1), L(320), P(0x41f71a)])?;
            self.lib_call(sp, 0x400068, vec![L(handle)])?;
            self.byte(frame.wrapping_sub(115), 1)?;
            self.gitem(self.l(0x4394d0)?, 33, 0x4434ba)?;
            self.pict_call(sp, 216, 0x4434ba)?;
            *sp = sp.wrapping_add(30);
        }
        self.first_call(sp, 0x0079007c)?;
        *sp = sp.wrapping_add(4);
        Ok(())
    }
}

impl<S: CharacterUpgradeServices + ?Sized> Execution<'_, S> {
    fn party_select(&mut self, frame: u32) -> Result<(), RuleError> {
        let mut sp = frame.wrapping_sub(206);
        self.byte(frame.wrapping_sub(115), 0)?;
        self.byte(0x4427b5, 1)?;
        self.word(frame.wrapping_sub(114), 0)?;
        self.word(frame.wrapping_sub(110), 50)?;
        self.word(frame.wrapping_sub(112), 330)?;
        self.word(frame.wrapping_sub(108), 380)?;
        self.lib_call(
            &mut sp,
            0x400090,
            vec![P(0x401b14), P(frame.wrapping_sub(136))],
        )?;
        self.long(frame.wrapping_sub(144), 0)?;
        self.long(frame.wrapping_sub(182), 0)?;
        self.long(frame.wrapping_sub(178), 0)?;
        self.word(frame.wrapping_sub(174), 0)?;
        self.sane(&[frame.wrapping_sub(182)], 0x16)?;
        self.sane(&[frame.wrapping_sub(182), frame.wrapping_sub(182)], 0x2810)?;
        let mut selected = self.w(frame.wrapping_sub(180))?;
        self.word(frame.wrapping_sub(120), selected)?;
        self.word(frame.wrapping_sub(106), selected)?;
        let mut member = selected;
        let mut previous = selected;
        let dialog = self.trap(0xa97c, vec![W(145), L(0), L(u32::MAX)])?;
        sp = sp.wrapping_add(4);
        self.long(0x4394d0, dialog)?;
        self.sound_call(&mut sp, 10004)?;
        sp = sp.wrapping_add(6);
        'setup: loop {
            self.step()?;
            self.trap(0xaa1c, vec![L(self.l(0x43952c)?)])?;
            self.port(self.l(0x4394d0)?)?;
            self.trap(0xa887, vec![W(self.w(0x402f3c)?)])?;
            self.trap(0xa88a, vec![W(12)])?;
            self.long(0x4394b8, self.l(0x4394d0)?)?;
            self.trap(0xaa0b, vec![L(self.l(0x4358ce)?)])?;
            self.trap(0xa862, vec![L(69)])?;
            for op in [0xa915, 0xa981, 0xa922, 0xa923] {
                self.trap(op, vec![L(self.l(0x4394d0)?)])?;
            }
            if self.sb(0x44273a)? > -1 {
                self.press_call(&mut sp, 1, 2, 0, frame.wrapping_sub(114))?;
                sp = sp.wrapping_add(14);
            }
            let handle = self.lib_call(&mut sp, 0x400070, vec![P(0x401afe), P(0x401b02)])?;
            sp = sp.wrapping_add(8);
            if handle == 0 {
                let handle = self.lib_call(&mut sp, 0x400070, vec![P(0x401afa), P(0x401b02)])?;
                sp = sp.wrapping_add(8);
                if handle == 0 {
                    self.scratch_call(&mut sp, 10, true)?;
                    if !self.active() {
                        return Ok(());
                    }
                    sp = sp.wrapping_add(2);
                }
                self.lib_call(&mut sp, 0x400068, vec![L(handle)])?;
                self.set_file_info(sp.wrapping_sub(16), 0x401b02, 0x401af4)?;
                sp = sp.wrapping_sub(8);
                self.flash(sp.wrapping_sub(20), 0x401aa4, 15, 60, 0, 6000)?;
                sp = sp.wrapping_add(12);
            }
            self.trap(
                0xa946,
                vec![L(self.l(0x43870e)?), W(self.w(0x442560)?), P(0x479ed8)],
            )?;
            self.lib_call(&mut sp, 0x400180, vec![P(0x479ed8)])?;
            self.filename_call(&mut sp, 0x479ed8)?;
            let handle = self.lib_call(&mut sp, 0x400070, vec![P(0x401afe), P(0x4389a4)])?;
            sp = sp.wrapping_add(16);
            if handle == 0 {
                self.scratch_call(&mut sp, 130, false)?;
                if !self.active() {
                    return Ok(());
                }
                sp = sp.wrapping_add(2);
            }
            self.lib_call(&mut sp, 0x400048, vec![L(handle), L(2), L(4), P(0x44276c)])?;
            self.lib_call(&mut sp, 0x400068, vec![L(handle)])?;
            let registered = crate::recovered_verification::doreg();
            self.full(registered as u32);
            sp = sp.wrapping_add(20);
            if registered != 0 {
                self.long(0x442770, 999)?;
            }
            self.byte(0x44273f, 0)?;
            let mut n = 0i16;
            while n <= self.sb(0x44273a)? {
                self.step()?;
                self.byte(warrior(n as u16) + 0x9df, 1)?;
                n = n.wrapping_add(1);
            }
            for item in 27..32 {
                self.gitem(self.l(0x4394d0)?, item, 0x4434ba)?;
                self.trap(0xa98f, vec![L(self.l(0x4434dc)?), P(0x401aa2)])?;
                self.pict_call(&mut sp, 179, 0x4434ba)?;
                sp = sp.wrapping_add(10);
            }
            'short: loop {
                self.step()?;
                self.port(self.l(0x4394d0)?)?;
                if self.w(frame + 8)? != 0 || self.sb(0x44273a)? > 1 {
                    self.gitem(self.l(0x4394d0)?, 71, 0x4434ba)?;
                    self.pict_call(&mut sp, 154, 0x4434ba)?;
                    sp = sp.wrapping_add(10);
                }
                self.panel_short(&mut sp)?;
                sp = sp.wrapping_add(2);
                'list: loop {
                    self.step()?;
                    self.long(0x4394b8, self.l(0x4394d0)?)?;
                    self.port(self.l(0x4394d0)?)?;
                    self.trap(0xa889, vec![W(1)])?;
                    self.trap(0xa887, vec![W(self.w(0x402f3c)?)])?;
                    self.trap(0xa862, vec![L(69)])?;
                    self.gitem(self.l(0x4394d0)?, 71, 0x4434ba)?;
                    self.pict_call(&mut sp, 160, 0x4434ba)?;
                    sp = sp.wrapping_add(10);
                    if self.sb(0x44273a)? > 4 {
                        self.icon_plot(sp.wrapping_sub(18), 2019, 0x4434ba)?;
                        self.byte(0x44273a, 5)?;
                    }
                    self.gitem(
                        self.l(0x4394d0)?,
                        self.w(0x442576)?.wrapping_add(29),
                        0x4434ba,
                    )?;
                    self.trap(0xa8a9, vec![P(0x4434ba), L(0x00020002)])?;
                    self.trap(0xa8f6, vec![L(self.l(0x439550)?), P(0x4434ba)])?;
                    self.gitem(self.l(0x4394d0)?, member.wrapping_add(34), 0x4434ba)?;
                    self.trap(0xa8f6, vec![L(self.l(0x43955c)?), P(0x4434ba)])?;
                    self.word(frame.wrapping_sub(120), 0)?;
                    let mut n = 0i16;
                    while n <= self.sb(0x44273a)? {
                        self.step()?;
                        self.number_call(
                            &mut sp,
                            (n as u16).wrapping_add(54),
                            self.w(warrior(n as u16) + 0x3a)?,
                        )?;
                        self.add(frame.wrapping_sub(120), self.sw(warrior(n as u16) + 0x3a)?)?;
                        self.edit_call(&mut sp, warrior(n as u16) + 0xa21, 1, false)?;
                        if !self.active() {
                            return Ok(());
                        }
                        sp = sp.wrapping_add(10);
                        n = n.wrapping_add(1);
                    }
                    self.number_call(&mut sp, 66, self.w(frame.wrapping_sub(120))?)?;
                    self.long_number_call(&mut sp, 65, self.l(0x44276c)?)?;
                    self.party_costs(frame, &mut sp, selected)?;
                    if !self.active() {
                        return Ok(());
                    }
                    loop {
                        self.step()?;
                        self.long(0x4394b8, self.l(0x4394d0)?)?;
                        self.flush()?;
                        self.trap(0xa991, vec![L(0), P(0x4434b2)])?;
                        self.gitem(self.l(0x4394d0)?, self.w(0x4434b2)?, 0x44292c)?;
                        if self.w(0x4434b2)? == 33 && self.b(frame.wrapping_sub(115))? != 0 {
                            self.restrictions()?;
                            continue 'setup;
                        }
                        if self.sw(0x4434b2)? > 26 && self.sw(0x4434b2)? < 32 {
                            if self.sw(0x4434b2)? > 29 {
                                self.full(crate::recovered_verification::doreg() as u32);
                            }
                            self.sound_call(&mut sp, 144)?;
                            self.gitem(
                                self.l(0x4394d0)?,
                                self.w(0x442576)?.wrapping_add(29),
                                0x4434ba,
                            )?;
                            self.trap(0xa8a9, vec![P(0x4434ba), L(0x00020002)])?;
                            self.trap(0xa8f6, vec![L(self.l(0x439554)?), P(0x4434ba)])?;
                            self.word(0x442576, self.w(0x4434b2)?.wrapping_sub(29))?;
                            sp = sp.wrapping_add(2);
                            continue 'list;
                        }
                        if self.w(0x4434b2)? == 17 || self.w(0x4434b2)? == 18 {
                            self.gitem(self.l(0x4394d0)?, self.w(0x4434b2)?, 0x44292c)?;
                            self.press_call(&mut sp, 2, 2, 141, 0x44292c)?;
                            self.pw(&mut sp, 10)?;
                            self.delay(10)?;
                            self.add(frame.wrapping_sub(106), -5)?;
                            let next = self.w(0x4434b2)? == 18;
                            sp = sp.wrapping_add(16);
                            if next {
                                self.add(frame.wrapping_sub(106), 10)?;
                            }
                            if self.sw(frame.wrapping_sub(106))? < 0 {
                                self.word(frame.wrapping_sub(106), 0)?;
                            }
                            self.press_call(&mut sp, 2, 1, 0, 0x44292c)?;
                            sp = sp.wrapping_add(14);
                            continue 'short;
                        }
                        if self.w(0x4434b2)? == 16 && selected != 0 {
                            self.press_call(&mut sp, 1, 2, 0, 0x44292c)?;
                            self.get_text(self.l(0x4394d0)?, selected)?;
                            self.pw(&mut sp, 15)?;
                            self.delay(15)?;
                            self.press_call(&mut sp, 1, 1, 0, 0x44292c)?;
                            let width = self.trap(0xa88c, vec![P(0x479ed8)])? as u16;
                            sp = sp.wrapping_add(30);
                            if width == 0 {
                                continue 'short;
                            }
                            let answer = self.question(sp.wrapping_sub(12), 3, 1)?;
                            if !self.active() {
                                return Ok(());
                            }
                            if answer != 0 {
                                self.get_text(self.l(0x4394d0)?, selected)?;
                                self.lib_call(&mut sp, 0x400180, vec![P(0x479ed8)])?;
                                self.edit_call(&mut sp, 0x479ed8, 0, false)?;
                                if !self.active() {
                                    return Ok(());
                                }
                                sp = sp.wrapping_add(10);
                            }
                            self.port(self.l(0x4394d0)?)?;
                            selected = 0;
                            continue 'setup;
                        }
                        if self.w(0x4434b2)? == 20 {
                            self.gitem(self.l(0x4394d0)?, self.w(0x4434b2)?, 0x44292c)?;
                            self.press_call(&mut sp, 1, 2, 0, 0x44292c)?;
                            self.trap(
                                0xa9ea,
                                vec![
                                    L(self.l(frame.wrapping_sub(140))?),
                                    P(0x479ed8),
                                    L(0),
                                    W(u16::MAX),
                                    P(frame.wrapping_sub(136)),
                                    L(0),
                                    P(frame.wrapping_sub(74)),
                                    W(2),
                                ],
                            )?;
                            sp = sp.wrapping_add(14);
                            if self.b(frame.wrapping_sub(74))? == 0 {
                                continue 'setup;
                            }
                            self.lib(
                                0x400150,
                                vec![
                                    P(frame.wrapping_sub(64)),
                                    W(self.w(frame.wrapping_sub(68))?),
                                    P(0x439cfc),
                                ],
                            )?;
                            let kind = self.l(0x439cfc)?;
                            if [
                                0x57617272, 0x496e6669, 0x48776570, 0x5073696f, 0x4d656469,
                                0x536f7073, 0x4d697363,
                            ]
                            .contains(&kind)
                            {
                                self.lib_call(&mut sp, 0x400180, vec![P(0x438aa4)])?;
                                self.lib_call(&mut sp, 0x400180, vec![P(frame.wrapping_sub(64))])?;
                                self.lib_call(
                                    &mut sp,
                                    0x400090,
                                    vec![P(frame.wrapping_sub(64)), P(0x438aa4)],
                                )?;
                                self.lib_call(&mut sp, 0x400090, vec![P(0x401a8e), P(0x4389a4)])?;
                                self.lib_call(
                                    &mut sp,
                                    0x4000a0,
                                    vec![W(30), P(0x438aa4), P(0x4389a4)],
                                )?;
                                self.edit_call(&mut sp, frame.wrapping_sub(64), 1, false)?;
                                if !self.active() {
                                    return Ok(());
                                }
                                let h = self.lib_call(
                                    &mut sp,
                                    0x400070,
                                    vec![P(0x401afe), P(0x4389a4)],
                                )?;
                                sp = sp.wrapping_add(48);
                                if h != 0 {
                                    self.lib_call(
                                        &mut sp,
                                        0x400048,
                                        vec![L(h), L(1), L(3138), P(0x44721c)],
                                    )?;
                                    self.lib_call(&mut sp, 0x400068, vec![L(h)])?;
                                    self.edit_call(&mut sp, 0x447c3d, self.w(0x447256)?, true)?;
                                    if !self.active() {
                                        return Ok(());
                                    }
                                    sp = sp.wrapping_add(26);
                                    continue 'setup;
                                }
                            }
                            self.warning_colors(&mut sp, 38, true)?;
                            sp = sp.wrapping_add(2);
                            continue 'setup;
                        }
                        if self.w(0x4434b2)? == 19 {
                            self.press_call(&mut sp, 1, 1, 0, 0x44292c)?;
                            sp = sp.wrapping_add(14);
                            if self.sb(0x44273a)? > -1 {
                                self.sound_call(&mut sp, 26260)?;
                                sp = sp.wrapping_add(2);
                            }
                            let mut n = 0i16;
                            while n <= self.sb(0x44273a)? {
                                self.step()?;
                                self.edit_call(
                                    &mut sp,
                                    warrior(n as u16) + 0xa21,
                                    self.w(warrior(n as u16) + 0x3a)?,
                                    true,
                                )?;
                                if !self.active() {
                                    return Ok(());
                                }
                                sp = sp.wrapping_add(6);
                                n = n.wrapping_add(1);
                            }
                            self.trap(0xa983, vec![L(self.l(0x4394d0)?)])?;
                            self.word(0x4434ac, 0)?;
                            while self.sw(0x4434ac)? <= self.sb(0x44273a)? {
                                self.step()?;
                                for slot in 0..10 {
                                    self.copied_scans(self.w(0x4434ac)?, slot)?;
                                }
                                self.add(0x4434ac, 1)?;
                            }
                            self.field_call(sp.wrapping_sub(8), false)?;
                            if !self.active() {
                                return Ok(());
                            }
                            self.byte(0x4427b5, 0)?;
                            self.full(0);
                            return Ok(());
                        }
                        if self.sw(0x4434b2)? < 14 {
                            self.sound_call(&mut sp, 145)?;
                            self.get_text(self.l(0x4394d0)?, self.w(0x4434b2)?)?;
                            self.lib_call(&mut sp, 0x400180, vec![P(0x479ed8)])?;
                            self.lib_call(&mut sp, 0x400090, vec![P(0x401a8e), P(0x4389a4)])?;
                            self.lib_call(
                                &mut sp,
                                0x4000a0,
                                vec![W(30), P(0x479ed8), P(0x4389a4)],
                            )?;
                            let h =
                                self.lib_call(&mut sp, 0x400070, vec![P(0x401afe), P(0x4389a4)])?;
                            sp = sp.wrapping_add(32);
                            if h == 0 {
                                self.edit_call(&mut sp, 0x479ed8, 1, false)?;
                                if !self.active() {
                                    return Ok(());
                                }
                                self.warning_colors(&mut sp, 38, true)?;
                                self.get_text(self.l(0x4394d0)?, selected)?;
                                self.lib_call(&mut sp, 0x400180, vec![P(0x479ed8)])?;
                                sp = sp.wrapping_add(12);
                                continue 'setup;
                            }
                            self.lib_call(
                                &mut sp,
                                0x400048,
                                vec![L(h), L(1), L(3138), P(0x44721c)],
                            )?;
                            self.lib_call(&mut sp, 0x400068, vec![L(h)])?;
                            sp = sp.wrapping_add(20);
                            if selected == self.w(0x4434b2)? {
                                let h = self.lib_call(
                                    &mut sp,
                                    0x400070,
                                    vec![P(0x401afe), P(0x4389a4)],
                                )?;
                                self.copy(0x448aa0, 0x447e5e, 3138)?;
                                self.lib_call(
                                    &mut sp,
                                    0x400048,
                                    vec![L(h), L(1), L(3138), P(0x448aa0)],
                                )?;
                                self.lib_call(&mut sp, 0x400068, vec![L(h)])?;
                                self.pl(&mut sp, 1)?;
                                self.view_character(sp.wrapping_sub(8))?;
                                if !self.active() {
                                    return Ok(());
                                }
                                selected = 0;
                                self.copy(0x447e5e, 0x448aa0, 3138)?;
                                sp = sp.wrapping_add(32);
                                continue 'setup;
                            }
                            if selected == 0 {
                                selected = self.w(0x4434b2)?;
                            }
                            self.trap(0xa862, vec![L(273)])?;
                            self.get_text(self.l(0x4394d0)?, selected)?;
                            self.string_call(&mut sp, selected, 0x479ed8)?;
                            self.get_text(self.l(0x4394d0)?, selected.wrapping_add(40))?;
                            self.string_call(&mut sp, selected.wrapping_add(40), 0x479ed8)?;
                            self.trap(0xa862, vec![L(69)])?;
                            self.get_text(self.l(0x4394d0)?, self.w(0x4434b2)?.wrapping_add(40))?;
                            self.string_call(
                                &mut sp,
                                self.w(0x4434b2)?.wrapping_add(40),
                                0x479ed8,
                            )?;
                            self.get_text(self.l(0x4394d0)?, self.w(0x4434b2)?)?;
                            self.string_call(&mut sp, self.w(0x4434b2)?, 0x479ed8)?;
                            self.trap(0xa862, vec![L(33)])?;
                            self.lib_call(&mut sp, 0x400180, vec![P(0x479ed8)])?;
                            self.lib_call(
                                &mut sp,
                                0x400090,
                                vec![P(0x479ed8), P(frame.wrapping_sub(104))],
                            )?;
                            selected = self.w(0x4434b2)?;
                            self.word(0x4434b6, self.w(0x447256)?)?;
                            self.trap(0xaa1c, vec![L(self.l(0x43952c)?)])?;
                            sp = sp.wrapping_add(36);
                            if self.sw(0x4434b6)? > self.sw(0x41f81c)? && self.w(0x41f81c)? != 0 {
                                self.trap(0xaa1c, vec![L(self.l(0x439534)?)])?;
                            }
                            if self.sb(0x44273a)?.wrapping_add(1) >= self.sw(0x41f81a)?
                                && self.w(0x41f81a)? != 0
                            {
                                self.trap(0xaa1c, vec![L(self.l(0x439534)?)])?;
                            }
                            if self.b(index(0x41f81e, self.sw(0x44724e)?.wrapping_sub(1), 1))? != 0
                            {
                                self.trap(0xaa1c, vec![L(self.l(0x439534)?)])?;
                            }
                            if self.b(index(0x41f83c, self.sw(0x447250)?.wrapping_sub(1), 1))? != 0
                            {
                                self.trap(0xaa1c, vec![L(self.l(0x439534)?)])?;
                            }
                        }
                        if self.sw(0x4434b2)? >= 21
                            && self.sw(0x4434b2)? <= self.sb(0x44273a)?.wrapping_add(21)
                        {
                            self.press_call(&mut sp, 1, 1, 0, frame.wrapping_sub(114))?;
                            self.gitem(
                                self.l(0x4394d0)?,
                                self.w(0x4434b2)?,
                                frame.wrapping_sub(114),
                            )?;
                            self.press_call(&mut sp, 1, 2, 141, frame.wrapping_sub(114))?;
                            member = self.w(0x4434b2)?.wrapping_sub(21);
                            self.byte(0x44273f, member as u8)?;
                            member = i16::from(member as u8 as i8) as u16;
                            self.gitem(self.l(0x4394d0)?, previous.wrapping_add(34), 0x4434ba)?;
                            self.trap(0xa8f6, vec![L(self.l(0x439544)?), P(0x4434ba)])?;
                            self.gitem(self.l(0x4394d0)?, member.wrapping_add(34), 0x4434ba)?;
                            self.trap(0xa8f6, vec![L(self.l(0x43955c)?), P(0x4434ba)])?;
                            sp = sp.wrapping_add(28);
                            if member == previous {
                                self.trap(0xa862, vec![L(69)])?;
                                self.pw(&mut sp, 0)?;
                                self.pw(&mut sp, member)?;
                                self.view_character(sp.wrapping_sub(8))?;
                                if !self.active() {
                                    return Ok(());
                                }
                                selected = 0;
                                sp = sp.wrapping_add(4);
                                continue 'setup;
                            }
                            previous = member;
                        }
                        if self.w(0x4434b2)? == 15 && self.sb(0x44273a)? > -1 {
                            self.byte(0x44273f, 0)?;
                            if self.w(frame + 8)? != 0 && self.b(0x44273a)? == 0 {
                                self.warning_colors(&mut sp, 67, true)?;
                                self.port(self.l(0x4394d0)?)?;
                                self.trap(0xa862, vec![L(69)])?;
                                sp = sp.wrapping_add(2);
                            } else {
                                let mut blocked = false;
                                let mut n = 0i16;
                                while n < self.sw(warrior(member) + 0x1a)? {
                                    self.step()?;
                                    self.pw(
                                        &mut sp,
                                        self.w(warrior(member) + u32::from(n as u16) * 22 + 0x132)?,
                                    )?;
                                    self.load_item(self.w(sp)?)?;
                                    sp = sp.wrapping_add(2);
                                    let kind = self.sw(0x453656)?;
                                    if kind == 25 || kind == 23 || kind == 24 || kind < 0 {
                                        blocked = true;
                                    }
                                    n = n.wrapping_add(1);
                                }
                                if blocked {
                                    self.warning_colors(&mut sp, 100, true)?;
                                    self.port(self.l(0x4394d0)?)?;
                                    self.trap(0xa862, vec![L(69)])?;
                                    self.long(0x4394b8, self.l(0x4394d0)?)?;
                                    let answer = self.question(sp.wrapping_sub(12), 13, 1)?;
                                    if !self.active() {
                                        return Ok(());
                                    }
                                    sp = sp.wrapping_add(2);
                                    if answer == 0 {
                                        continue 'setup;
                                    }
                                    self.gitem(self.l(0x4394d0)?, 13, 0x44292c)?;
                                }
                                self.gitem(self.l(0x4394d0)?, previous.wrapping_add(34), 0x4434ba)?;
                                self.trap(0xa8f6, vec![L(self.l(0x439544)?), P(0x4434ba)])?;
                                self.press_call(&mut sp, 1, 2, 0, 0x44292c)?;
                                self.sound_call(&mut sp, 663)?;
                                self.edit_call(
                                    &mut sp,
                                    warrior(member) + 0xa21,
                                    self.w(warrior(member) + 0x3a)?,
                                    true,
                                )?;
                                if !self.active() {
                                    return Ok(());
                                }
                                self.trap(0xa862, vec![L(69)])?;
                                self.pw(&mut sp, 0)?;
                                self.pw(&mut sp, member)?;
                                self.field_call(sp.wrapping_sub(8), true)?;
                                if !self.active() {
                                    return Ok(());
                                }
                                self.string_call(
                                    &mut sp,
                                    self.sb(0x44273a)?.wrapping_add(54) as u16,
                                    0x401aa2,
                                )?;
                                sp = sp.wrapping_add(32);
                                if self.sw(warrior(member) + 0x50)? < 1
                                    || self.w(warrior(member) + 0x90)? != 0
                                {
                                    self.byte(0x442739, self.b(0x442739)?.wrapping_sub(1))?;
                                }
                                let mut n = member as i16;
                                while n < self.sb(0x44273a)? {
                                    self.step()?;
                                    self.copy(
                                        warrior((n as u16).wrapping_add(1)),
                                        warrior(n as u16),
                                        3138,
                                    )?;
                                    n = n.wrapping_add(1);
                                }
                                self.gitem(self.l(0x4394d0)?, member.wrapping_add(22), 0x4434ba)?;
                                self.word(0x4434ba, (member as i16).wrapping_mul(50) as u16)?;
                                self.word(0x4434be, 300)?;
                                self.add(0x4434bc, -10)?;
                                self.word(0x4434c0, 640)?;
                                self.port(self.l(0x4394d0)?)?;
                                self.trap(0xa8ef, vec![P(0x4434ba), L(0xffce0000), L(0)])?;
                                self.word(0x4434bc, 319)?;
                                self.word(0x4434c0, 640)?;
                                self.word(0x4434ba, 250)?;
                                self.word(0x4434be, 300)?;
                                self.trap(0xa8a3, vec![P(0x4434ba)])?;
                                self.byte(0x44273a, self.b(0x44273a)?.wrapping_sub(1))?;
                                selected = 0;
                                previous = selected;
                                if (member as i16) > self.sb(0x44273a)? {
                                    member = member.wrapping_sub(1);
                                    if (member as i16) < 0 {
                                        member = member.wrapping_add(1);
                                    }
                                }
                                self.press_call(&mut sp, 1, 1, 0, 0x44292c)?;
                                sp = sp.wrapping_add(14);
                                continue 'setup;
                            }
                        }
                        if self.w(0x4434b2)? != 14 || self.sb(0x44273a)? >= 5 || selected == 0 {
                            continue;
                        }
                        let level = self.dialog_number(&mut sp, selected.wrapping_add(40))?;
                        self.word(0x4434b6, level)?;
                        sp = sp.wrapping_add(2);
                        if self.b(frame.wrapping_sub(115))? != 0 {
                            self.get_text(self.l(0x4394d0)?, selected)?;
                            if self.trap(0xa88c, vec![P(0x479ed8)])? as u16 != 0 {
                                self.lib_call(&mut sp, 0x400180, vec![P(0x479ed8)])?;
                                self.lib_call(&mut sp, 0x400090, vec![P(0x401a8e), P(0x4389a4)])?;
                                self.lib_call(&mut sp, 0x400098, vec![P(0x479ed8), P(0x4389a4)])?;
                                let h = self.lib_call(
                                    &mut sp,
                                    0x400070,
                                    vec![P(0x401afe), P(0x4389a4)],
                                )?;
                                sp = sp.wrapping_add(28);
                                if h == 0 {
                                    self.warning_colors(&mut sp, 38, true)?;
                                    sp = sp.wrapping_add(2);
                                    continue 'setup;
                                }
                                self.lib_call(
                                    &mut sp,
                                    0x400048,
                                    vec![L(h), L(1), L(3138), P(0x44721c)],
                                )?;
                                self.lib_call(&mut sp, 0x400068, vec![L(h)])?;
                                sp = sp.wrapping_add(20);
                            }
                            let restriction = if self.sw(0x4434b6)? > self.sw(0x41f81c)?
                                && self.w(0x41f81c)? != 0
                            {
                                Some(134)
                            } else if self.sb(0x44273a)?.wrapping_add(1) >= self.sw(0x41f81a)?
                                && self.w(0x41f81a)? != 0
                            {
                                Some(135)
                            } else if self.b(index(
                                0x41f81e,
                                self.sw(0x44724e)?.wrapping_sub(1),
                                1,
                            ))? != 0
                            {
                                Some(136)
                            } else if self.b(index(
                                0x41f83c,
                                self.sw(0x447250)?.wrapping_sub(1),
                                1,
                            ))? != 0
                            {
                                Some(137)
                            } else {
                                None
                            };
                            if let Some(id) = restriction {
                                self.warning_colors(&mut sp, id, false)?;
                                self.restrictions()?;
                                sp = sp.wrapping_add(2);
                                continue 'setup;
                            }
                        }
                        self.gitem(self.l(0x4394d0)?, self.w(0x4434b2)?, 0x44292c)?;
                        self.press_call(&mut sp, 1, 2, 0, 0x44292c)?;
                        self.get_text(self.l(0x4394d0)?, selected)?;
                        let width = self.trap(0xa88c, vec![P(0x479ed8)])? as u16;
                        if width == 0 {
                            sp = sp.wrapping_add(14);
                            continue;
                        }
                        self.lib_call(&mut sp, 0x400180, vec![P(0x479ed8)])?;
                        self.sound_call(&mut sp, 662)?;
                        self.pw(&mut sp, 10)?;
                        self.delay(10)?;
                        self.press_call(&mut sp, 1, 1, 0, 0x44292c)?;
                        self.lib_call(&mut sp, 0x400090, vec![P(0x401a8e), P(0x4389a4)])?;
                        self.lib_call(&mut sp, 0x400098, vec![P(0x479ed8), P(0x4389a4)])?;
                        self.byte(0x44273a, self.b(0x44273a)?.wrapping_add(1))?;
                        let h = self.lib_call(&mut sp, 0x400070, vec![P(0x401afe), P(0x4389a4)])?;
                        sp = sp.wrapping_add(46);
                        if h == 0 {
                            self.byte(0x44273a, self.b(0x44273a)?.wrapping_sub(1))?;
                            self.warning_colors(&mut sp, 38, false)?;
                            self.get_text(self.l(0x4394d0)?, selected)?;
                            self.lib_call(&mut sp, 0x400180, vec![P(0x479ed8)])?;
                            self.edit_call(&mut sp, 0x479ed8, 1, false)?;
                            if !self.active() {
                                return Ok(());
                            }
                            sp = sp.wrapping_add(12);
                            continue 'short;
                        }
                        self.lib_call(&mut sp, 0x400048, vec![L(h), L(1), L(3138), P(0x44721c)])?;
                        self.lib_call(&mut sp, 0x400068, vec![L(h)])?;
                        sp = sp.wrapping_add(20);
                        if self.sw(0x44721c)? < 1 {
                            self.byte(0x44273a, self.b(0x44273a)?.wrapping_sub(1))?;
                            self.warning_colors(&mut sp, 73, false)?;
                            sp = sp.wrapping_add(2);
                            continue 'short;
                        }
                        if self.sw(0x447256)? > 3 {
                            self.full(crate::recovered_verification::doreg() as u32);
                        }
                        self.copy(0x44721c, warrior(self.sb(0x44273a)? as u16), 3138)?;
                        self.edit_call(
                            &mut sp,
                            warrior(self.sb(0x44273a)? as u16) + 0xa21,
                            1,
                            false,
                        )?;
                        if !self.active() {
                            return Ok(());
                        }
                        selected = 0;
                        self.gitem(
                            self.l(0x4394d0)?,
                            self.sb(0x44273a)?.wrapping_add(21) as u16,
                            0x4434ba,
                        )?;
                        self.add(0x4434bc, -10)?;
                        self.pict_call(&mut sp, 1, 0x4434ba)?;
                        self.panel_character(&mut sp, self.sb(0x44273a)? as u16)?;
                        self.press_call(&mut sp, 1, 2, 0, 0x44292c)?;
                        sp = sp.wrapping_add(34);
                        if self.sw(warrior(self.sb(0x44273a)? as u16) + 0x50)? < 1
                            || self.w(warrior(self.sb(0x44273a)? as u16) + 0x90)? != 0
                        {
                            self.byte(0x442739, self.b(0x442739)?.wrapping_add(1))?;
                        }
                        continue 'list;
                    }
                }
            }
        }
    }
}
fn index(base: u32, value: i16, stride: i32) -> u32 {
    base.wrapping_add(i32::from(value).wrapping_mul(stride) as u32)
}
fn warrior(member: u16) -> u32 {
    0x448aa0_u32.wrapping_add(u32::from(member) * WARRIOR_BYTES as u32)
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
        Err(RuleError::Invalid("unused upgrade drawing string".into()))
    }
    fn current_world(&mut self) -> Result<(u32, u32), RuleError> {
        Err(RuleError::Invalid("unused upgrade drawing world".into()))
    }
    fn new_world(
        &mut self,
        _: i16,
        _: crate::recovered_geometry::BodyRect,
    ) -> Result<(i16, BuildingWorld), RuleError> {
        Err(RuleError::Invalid(
            "unused upgrade drawing allocation".into(),
        ))
    }
}
struct Clock<'a, 'b, S: ?Sized> {
    execution: &'a mut Execution<'b, S>,
    remaining: usize,
}
impl<S: CharacterUpgradeServices + ?Sized> TimingServices for Clock<'_, '_, S> {
    fn tick_count(&mut self) -> Result<u32, RuleError> {
        self.remaining = self
            .remaining
            .checked_sub(1)
            .ok_or_else(|| RuleError::Invalid("upgrade tick budget exhausted".into()))?;
        self.execution.observe(FeedbackRequest::TickCount)
    }
    fn button(&mut self) -> Result<bool, RuleError> {
        Ok(self.execution.observe(FeedbackRequest::Button)? != 0)
    }
}
fn execution<'a, S: CharacterUpgradeServices + ?Sized>(
    state: &PromptControlState,
    limits: PromptLimits,
    services: &'a mut S,
) -> Execution<'a, S> {
    Execution {
        plan: CharacterUpgradePlan {
            state: state.clone(),
            effects: vec![],
            native_return: PromptReturn::Unchanged,
            flow: LoadFlow::Returned,
            qualification:
                CharacterUpgradeQualification::SourcePortWithObservedBoundariesAndProjectedChildren,
        },
        s: services,
        limits,
        remaining: limits.maximum_steps,
        d0: None,
    }
}
macro_rules! entry {
    ($name:ident) => {
        pub fn $name(
            state: &PromptControlState,
            frame: u32,
            limits: PromptLimits,
            services: &mut (impl CharacterUpgradeServices + ?Sized),
        ) -> Result<CharacterUpgradePlan, RuleError> {
            let mut e = execution(state, limits, services);
            e.$name(frame)?;
            Ok(e.plan)
        }
    };
}
entry!(upgrade_warrior);
entry!(view_character);
entry!(party_select);
