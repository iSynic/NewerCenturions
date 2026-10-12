//! Loot collection and reward distribution from CODE 1.

use crate::RuleError;
use crate::recovered_audio::{self as audio, AudioContext, AudioEffect};
use crate::recovered_character_creation::{self as creation, CharacterCreationEffect};
use crate::recovered_character_panels::{
    self as character, CharacterPanelEffect, CharacterPanelServices,
};
use crate::recovered_detail_ui::{self as detail, DetailEffect};
use crate::recovered_drawing::{
    self as drawing, BuildingWorld, DrawingServices, PictureObservation,
};
use crate::recovered_economy::{
    self as economy, ExperienceCounters, ExperienceField, ExperienceInput, ExperienceOutput,
    ExperienceState,
};
use crate::recovered_equipment::{self as equipment, EquipmentEffect, EquipmentLimits};
use crate::recovered_feedback::{
    self as feedback, FeedbackArg, FeedbackEffect, FeedbackObservation, FeedbackRequest,
    FeedbackServices,
};
use crate::recovered_files::{self as files, FileBuffer, FileContext, FileEffect};
use crate::recovered_geometry::BodyRect;
use crate::recovered_install::{self as install, InstallMemory, InstallRequest};
use crate::recovered_interface::{self as interface, ButtonInput, InterfaceEffect};
use crate::recovered_inventory::{self as inventory, InventoryEffect};
use crate::recovered_inventory_interaction::{
    self as interaction, InventoryInteractionEffect, InventoryInteractionLimits,
    InventoryInteractionServices,
};
use crate::recovered_inventory_rules::{self as restriction, InventoryEffect as RestrictionEffect};
use crate::recovered_inventory_selection::{
    self as selection, InventorySelectionEffect, InventorySelectionLimits,
};
use crate::recovered_item_ui::{self as item_ui, ItemUiEffect};
use crate::recovered_load_control::{LoadEffect, LoadFlow};
use crate::recovered_navigation::{self as navigation, MovementResult};
use crate::recovered_prompt_control::{
    self as prompt, LibraryObservation, LibraryRequest, PromptControlEffect, PromptControlState,
    PromptLimits, PromptRegisterRequest, PromptReturn,
};
use crate::recovered_random_item::{self as random_item, RandomItemEffect, RandomItemLimits};
use crate::recovered_records::RecordWrite;
use crate::recovered_records::{self as records, ItemTables};
use crate::recovered_rendering::RenderingMemory;
use crate::recovered_runtime::{self as runtime, RuntimeEffect, RuntimeResult};
use crate::recovered_selection_control::{
    self as survivor, SelectionControlEffect, SelectionControlLimits, SelectionControlServices,
};
use crate::recovered_text::{self as text, NumericServices, PascalString, TextContext};
use FeedbackArg::{Long as L, Pointer as P, Word as W};
use centurions_content::ITEM_BYTES;
use centurions_content::{INVENTORY_BYTES, WARRIOR_BYTES, Warrior};

pub type LootControlState = PromptControlState;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LootControlLimits {
    pub maximum_steps: u32,
    pub prompt: PromptLimits,
    pub equipment: EquipmentLimits,
    pub maximum_random_draws: u64,
}
pub trait LootControlServices: InventoryInteractionServices + SelectionControlServices {}
#[derive(Clone, Debug, PartialEq)]
pub enum LootControlEffect {
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
    Prompt(PromptControlEffect),
    Load(LoadEffect),
    Selection(InventorySelectionEffect),
    Survivor(SelectionControlEffect),
    Interaction(InventoryInteractionEffect),
    Character(CharacterPanelEffect),
    Equipment(EquipmentEffect),
    Audio(AudioEffect),
    File(FileEffect),
    Interface(InterfaceEffect),
    RandomItem(RandomItemEffect),
    Inventory(InventoryEffect),
    Restriction(RestrictionEffect),
    Detail(DetailEffect),
    ItemUi(ItemUiEffect),
    Creation(CharacterCreationEffect),
    Movement(MovementResult),
    Experience(ExperienceOutput),
}
#[derive(Clone, Debug, PartialEq)]
pub struct LootControlPlan {
    pub state: LootControlState,
    pub effects: Vec<LootControlEffect>,
    pub native_return: PromptReturn,
    pub flow: LoadFlow,
    pub qualification: LootControlQualification,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LootControlQualification {
    SourcePortWithObservedBoundariesAndCapturedChildren,
}
struct Execution<'a, S> {
    plan: LootControlPlan,
    services: &'a mut S,
    limits: LootControlLimits,
    remaining: u32,
    draws: u64,
    d0: Option<u32>,
    stack: u32,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LootRegisters {
    pub a2: Option<u32>,
    pub a4: Option<u32>,
}
struct Modal {
    registers: LootRegisters,
    d4: i16,
}
impl<S: LootControlServices> Execution<'_, S> {
    fn bytes(&self, a: u32, n: usize) -> Result<Vec<u8>, RuleError> {
        read(&self.plan.state.memory, a, n)
    }
    fn full(&mut self, d: u32) {
        self.returned(PromptReturn::Full(d));
    }
    fn unknown(&mut self) {
        self.d0 = None;
        self.plan.native_return = PromptReturn::Unchanged;
    }
    fn word(&mut self, a: u32, v: u16) -> Result<(), RuleError> {
        self.ww(a, v as i16)
    }
    fn long(&mut self, a: u32, v: u32) -> Result<(), RuleError> {
        self.lw(a, v)
    }
    fn profile(&mut self, race: i16, caste: i16, stack: u32) -> Result<(), RuleError> {
        self.profile_at(stack.wrapping_sub(12), race, caste)
    }
    fn range(&mut self, min: i16, max: i16, stack: u32) -> Result<i16, RuleError> {
        self.ww(stack.wrapping_sub(2), max)?;
        self.ww(stack.wrapping_sub(4), min)?;
        let span = max.wrapping_sub(min).wrapping_add(1);
        let value = min
            .wrapping_add(self.rand(span, stack.wrapping_sub(20))?)
            .wrapping_sub(1);
        self.d0 = self.d0.map(|d| (d & 0xffff0000) | u32::from(value as u16));
        Ok(value)
    }
    fn prompt(&mut self, entry: u32, frame: u32) -> Result<(), RuleError> {
        self.step()?;
        let before = self.plan.state.random.draws;
        let limits = PromptLimits {
            maximum_steps: self.remaining.min(self.limits.prompt.maximum_steps),
            ..self.limits.prompt
        };
        let child = match entry {
            0x12382a => prompt::first_tutorial(&self.plan.state, frame, limits, self.services)?,
            0x133d46 => prompt::submit_music(&self.plan.state, frame, limits, self.services)?,
            0x14cff6 => prompt::question(&self.plan.state, frame, limits, self.services)?,
            0x15dc34 => prompt::text_box(&self.plan.state, frame, limits, self.services)?,
            _ => return Err(RuleError::Invalid("unknown loot prompt child".into())),
        };
        self.draws = self
            .draws
            .checked_sub(child.state.random.draws.saturating_sub(before))
            .ok_or_else(|| RuleError::Invalid("loot random budget exhausted".into()))?;
        self.returned(child.native_return);
        self.plan.state = child.state;
        self.plan.flow = child.flow;
        if !self.active() {
            self.plan.native_return = PromptReturn::NoReturn;
        }
        self.plan
            .effects
            .extend(child.effects.into_iter().map(LootControlEffect::Prompt));
        Ok(())
    }
    fn flush_events(&mut self) -> Result<(), RuleError> {
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
        self.plan.effects.push(LootControlEffect::RegisterToolbox {
            request,
            observation,
        });
        Ok(())
    }
    fn c_string(&self, pointer: u32) -> Result<Vec<u8>, RuleError> {
        let mut bytes = vec![];
        for offset in 0..=self.plan.state.memory.bytes.len() {
            let b = self.b(pointer.wrapping_add(offset as u32))?;
            bytes.push(b);
            if b == 0 {
                return Ok(bytes);
            }
        }
        Err(RuleError::Invalid("unterminated loot text".into()))
    }
    fn random_item(&mut self, slot: i16, enhancement: i16, stack: u32) -> Result<(), RuleError> {
        self.ww(stack.wrapping_sub(2), enhancement)?;
        self.ww(stack.wrapping_sub(4), slot)?;
        let start = 0x4046e6_u32;
        let end = 0x478288_u32.max(pack(slot).wrapping_add(22));
        if end.wrapping_sub(start) > 1024 * 1024 {
            return Err(RuleError::Invalid("loot item arena exceeds one MiB".into()));
        }
        let memory = InstallMemory {
            base_address: start,
            bytes: self.bytes(start, (end - start) as usize)?,
        };
        let before = self.plan.state.random.draws;
        let child = random_item::randomitem(
            &memory,
            &self.plan.state.random,
            slot as u16,
            enhancement,
            RandomItemLimits {
                maximum_draws: self.draws,
            },
        )?;
        self.draws = self
            .draws
            .checked_sub(child.random.draws.saturating_sub(before))
            .ok_or_else(|| RuleError::Invalid("loot random budget exhausted".into()))?;
        self.plan.state.random = child.random;
        for effect in child.effects {
            if let RandomItemEffect::Write(w) = effect {
                self.write(w.address, &w.bytes)?;
            } else {
                self.plan
                    .effects
                    .push(LootControlEffect::RandomItem(effect));
            }
        }
        self.d0 = Some(child.native_return as u32);
        Ok(())
    }
    fn step(&mut self) -> Result<(), RuleError> {
        self.remaining = self
            .remaining
            .checked_sub(1)
            .ok_or_else(|| RuleError::Invalid("loot control processing budget exhausted".into()))?;
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
        read(&self.plan.state.memory, a, bytes.len())?;
        self.plan.state.memory.insert_region(a, bytes);
        self.plan
            .effects
            .push(LootControlEffect::Write(RecordWrite {
                address: a,
                bytes: bytes.to_vec(),
            }));
        Ok(())
    }
    fn bw(&mut self, a: u32, v: u8) -> Result<(), RuleError> {
        self.write(a, &[v])
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
    fn addl(&mut self, a: u32, v: u32) -> Result<(), RuleError> {
        self.lw(a, self.l(a)?.wrapping_add(v))
    }
    fn copy(
        &mut self,
        mut from: u32,
        mut to: u32,
        longs: usize,
        tail: bool,
    ) -> Result<(), RuleError> {
        for _ in 0..longs {
            self.step()?;
            self.lw(to, self.l(from)?)?;
            from = from.wrapping_add(4);
            to = to.wrapping_add(4);
        }
        if tail {
            self.ww(to, self.w(from)?)?;
        }
        Ok(())
    }
    fn returned(&mut self, r: PromptReturn) {
        match r {
            PromptReturn::Full(v) => self.d0 = Some(v),
            PromptReturn::LowWord(v) => self.d0 = self.d0.map(|d| (d & 0xffff0000) | u32::from(v)),
            PromptReturn::Unchanged => {}
            PromptReturn::NoReturn => self.d0 = None,
        }
        self.plan.native_return = self.d0.map_or(r, PromptReturn::Full);
    }
    fn active(&self) -> bool {
        self.plan.flow == LoadFlow::Returned
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
            .push(LootControlEffect::Feedback(FeedbackEffect::Service {
                request,
                observation,
            }));
        Ok(value)
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
        self.plan.effects.push(LootControlEffect::Library {
            request,
            observation: observation.clone(),
        });
        Ok(observation)
    }
    fn sane(
        &mut self,
        selector: u16,
        source: Option<u32>,
        destination: u32,
    ) -> Result<(), RuleError> {
        let mut args = vec![];
        if let Some(a) = source {
            args.push(P(a));
        }
        args.push(P(destination));
        args.push(W(selector));
        self.trap(0xa9eb, args)?;
        Ok(())
    }
    fn heap(&mut self) -> Result<(), RuleError> {
        let observations = CharacterPanelServices::runtime_observations(
            self.services,
            0x13e13c,
            &self.plan.state.memory,
        )?;
        let mut context =
            runtime::RuntimeContext::new(self.plan.state.random.clone(), self.remaining as usize);
        context.memory.bytes = self.plan.state.memory.bytes.clone();
        let child = runtime::compact_heap(&mut context, &observations)?;
        self.plan.state.memory.bytes = context.memory.bytes;
        self.plan.state.random = context.random;
        self.d0 = if let RuntimeResult::Long(v) = child.result {
            Some(v)
        } else {
            None
        };
        self.plan
            .effects
            .extend(child.effects.into_iter().map(LootControlEffect::Runtime));
        Ok(())
    }
    fn rand(&mut self, bound: i16, stack: u32) -> Result<i16, RuleError> {
        self.step()?;
        self.draws = self
            .draws
            .checked_sub(1)
            .ok_or_else(|| RuleError::Invalid("loot random budget exhausted".into()))?;
        self.ww(stack.wrapping_sub(2), bound)?;
        let mut context =
            runtime::RuntimeContext::new(self.plan.state.random.clone(), self.remaining as usize);
        let child = runtime::rand(&mut context, bound)?;
        self.plan.state.random = context.random;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(LootControlEffect::Runtime));
        match child.result {
            RuntimeResult::Word { value, native_d0 } => {
                self.d0 = Some(native_d0);
                Ok(value)
            }
            _ => Err(RuleError::Invalid("unexpected loot Rand result".into())),
        }
    }
    fn sound(&mut self, id: i16, stack: u32) -> Result<(), RuleError> {
        self.ww(stack.wrapping_sub(2), id)?;
        let child = feedback::sound(&self.plan.state.memory, id, self.services)?;
        self.plan.state.memory = child.memory;
        self.d0 = child.native_return;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(LootControlEffect::Feedback));
        Ok(())
    }
    fn selection_limits(&self) -> InventorySelectionLimits {
        InventorySelectionLimits {
            maximum_steps: self.remaining,
            prompt: self.limits.prompt,
            equipment: self.limits.equipment,
            maximum_random_draws: self.draws,
        }
    }
    fn selection(&mut self, entry: u32, frame: u32) -> Result<(), RuleError> {
        self.step()?;
        let before = self.plan.state.random.draws;
        let limits = self.selection_limits();
        let child = match entry {
            0x15f4b8 => selection::update_booty(&self.plan.state, frame, limits, self.services)?,
            0x154a46 => selection::show_condition(&self.plan.state, frame, limits, self.services)?,
            0x15a9cc => selection::scan_select(&self.plan.state, frame, limits, self.services)?,
            _ => return Err(RuleError::Invalid("unknown loot selection child".into())),
        };
        self.draws = self
            .draws
            .checked_sub(child.state.random.draws.saturating_sub(before))
            .ok_or_else(|| RuleError::Invalid("loot random budget exhausted".into()))?;
        self.returned(child.native_return);
        self.plan.state = child.state;
        self.plan.flow = child.flow;
        if !self.active() {
            self.plan.native_return = PromptReturn::NoReturn;
        }
        self.plan
            .effects
            .extend(child.effects.into_iter().map(LootControlEffect::Selection));
        Ok(())
    }
    fn body_count(&mut self, stack: u32) -> Result<(), RuleError> {
        self.step()?;
        let before = self.plan.state.random.draws;
        let child = survivor::body_count(
            &self.plan.state,
            stack.wrapping_sub(8),
            SelectionControlLimits {
                prompt: PromptLimits {
                    maximum_steps: self.remaining,
                    ..self.limits.prompt
                },
                maximum_random_draws: self.draws,
            },
            self.services,
        )?;
        self.draws = self
            .draws
            .checked_sub(child.state.random.draws.saturating_sub(before))
            .ok_or_else(|| RuleError::Invalid("loot random budget exhausted".into()))?;
        self.returned(child.native_return);
        self.plan.state = child.state;
        self.plan.flow = child.flow;
        if !self.active() {
            self.plan.native_return = PromptReturn::NoReturn;
        }
        self.plan
            .effects
            .extend(child.effects.into_iter().map(LootControlEffect::Survivor));
        Ok(())
    }
    fn refit(&mut self, member: i16, stack: u32) -> Result<(), RuleError> {
        self.ww(stack.wrapping_sub(2), 0)?;
        self.lw(stack.wrapping_sub(6), 0x10001)?;
        self.ww(stack.wrapping_sub(8), member)?;
        let before = self.plan.state.random.draws;
        let child = equipment::refit(
            &self.plan.state.memory,
            &self.plan.state.random,
            stack.wrapping_sub(16),
            self.limits.equipment,
            self.services,
        )?;
        self.draws = self
            .draws
            .checked_sub(child.random.draws.saturating_sub(before))
            .ok_or_else(|| RuleError::Invalid("loot random budget exhausted".into()))?;
        self.plan.state.memory = child.memory;
        self.plan.state.random = child.random;
        self.d0 = child.native_return.known_long();
        self.plan
            .effects
            .extend(child.effects.into_iter().map(LootControlEffect::Equipment));
        Ok(())
    }
}
fn execution<'a, S>(
    state: &LootControlState,
    frame: u32,
    limits: LootControlLimits,
    services: &'a mut S,
) -> Execution<'a, S> {
    Execution {
        plan: LootControlPlan {
            state: state.clone(),
            effects: vec![],
            native_return: PromptReturn::Unchanged,
            flow: LoadFlow::Returned,
            qualification:
                LootControlQualification::SourcePortWithObservedBoundariesAndCapturedChildren,
        },
        services,
        limits,
        remaining: limits.maximum_steps,
        draws: limits.maximum_random_draws,
        d0: None,
        stack: frame.wrapping_sub(0xe6),
    }
}
fn read(memory: &RenderingMemory, a: u32, n: usize) -> Result<Vec<u8>, RuleError> {
    (0..n)
        .map(|i| memory.byte(a.wrapping_add(i as u32)))
        .collect()
}
fn warrior(n: i16) -> u32 {
    0x448aa0_u32.wrapping_add(u32::from(n as u16) * 3138)
}
fn npc(n: i16) -> u32 {
    0x439d3c_u32.wrapping_add(u32::from(n as u16) * 288)
}
fn pack(n: i16) -> u32 {
    0x41fc74_u32.wrapping_add(u32::from(n as u16) * 22)
}

struct LootPreparation {
    money: u32,
    count: i16,
}
fn prepare<S: LootControlServices>(
    e: &mut Execution<'_, S>,
    f: u32,
) -> Result<Option<LootPreparation>, RuleError> {
    let s = e.stack;
    e.lw(f.wrapping_sub(0xaa), 0)?;
    e.bw(f.wrapping_sub(0x57), 1)?;
    e.bw(f.wrapping_sub(0xab), 0)?;
    e.bw(f.wrapping_sub(0xa5), 0)?;
    let mut money = 0_u32;
    e.trap(
        0xa8a7,
        vec![P(f.wrapping_sub(0x56)), L(0xfff6fff6), L(0xfff6fff6)],
    )?;
    e.lw(f.wrapping_sub(0x6a), e.l(f.wrapping_sub(0x56))?)?;
    e.lw(f.wrapping_sub(0x66), e.l(f.wrapping_sub(0x52))?)?;
    e.ww(f.wrapping_sub(0x7e), -1)?;
    e.ww(f.wrapping_sub(0x60), 0)?;
    e.ww(f.wrapping_sub(0x5e), e.w(0x40856c)?.wrapping_add(330))?;
    e.ww(f.wrapping_sub(0x5a), e.w(0x40856c)?.wrapping_add(380))?;
    e.ww(f.wrapping_sub(0x5c), 50)?;
    e.ww(f.wrapping_sub(0x92), 5)?;
    e.ww(f.wrapping_sub(0x90), 5)?;
    e.ww(f.wrapping_sub(0x8c), e.w(0x40856c)?.wrapping_add(316))?;
    e.ww(f.wrapping_sub(0x8e), e.w(0x40856c)?.wrapping_add(425))?;
    e.bw(f.wrapping_sub(0x61), 0)?;
    e.ww(0x438776, 0)?;
    e.bw(0x442739, 0)?;
    e.ww(0x4427a6, 0)?;
    e.heap()?;
    for n in 0..1000_i16 {
        e.copy(0x41fc32, pack(n), 5, true)?;
        e.bw(0x42df04 + n as u32, 0)?;
    }
    if e.w(0x43957e)? != 0 {
        let mut n = 0_i16;
        while n <= i16::from(e.b(0x44273a)? as i8) {
            e.step()?;
            e.refit(n, s)?;
            n = n.wrapping_add(1);
        }
    }
    if e.w(f.wrapping_add(8))? == 10 {
        let value = e.l(0x44278c)?;
        e.addl(0x442788, value)?;
        money = e.l(0x44278c)?;
        e.lw(0x44278c, 0)?;
        for n in 0..1000_i16 {
            e.ww(pack(n), 0)?;
        }
        for n in 0..180_i16 {
            e.ww(pack(n), e.w(0x425264 + n as u32 * 22)?)?;
        }
        e.lw(f.wrapping_sub(0x96), 0)?;
        e.lw(0x443392, 0)?;
        e.lw(f.wrapping_sub(0xc6), 0)?;
        e.sane(0x280e, Some(f.wrapping_sub(0xc6)), f.wrapping_sub(0xc6))?;
        e.sane(0x1010, Some(f.wrapping_sub(0xc6)), f.wrapping_sub(0xc6))?;
        e.lw(f.wrapping_sub(0x82), e.l(f.wrapping_sub(0xc6))?)?;
    } else {
        if e.w(0x4427a6)? > 8 {
            e.ww(f.wrapping_add(8), 2)?;
        }
        let mut n = 0_i16;
        while n <= i16::from(e.b(0x44273a)? as i8) {
            e.step()?;
            let body = warrior(n);
            e.ww(body + 0x10, 0)?;
            e.bw(body + 0x9de, 0)?;
            e.ww(body + 0x60, 0)?;
            for k in 0..4 {
                let a = body + 0x5e + k * 2;
                if e.w(a)? > 0 {
                    e.ww(a, 0)?;
                }
            }
            if e.w(body + 0x50)? < 1 || e.w(body + 0x90)? < 0 {
                e.bw(0x442739, e.b(0x442739)?.wrapping_add(1))?;
            }
            if e.w(body + 0x90)? > 0 {
                e.ww(body + 0x90, 0)?;
            }
            n = n.wrapping_add(1);
        }
        e.trap(0xa939, vec![L(e.l(0x43870a)?), W(3)])?;
        e.trap(0xa939, vec![L(e.l(0x43870a)?), W(4)])?;
        for n in 0..60 {
            e.bw(0x442bd6 + n * 20 + 18, 0)?;
            e.bw(0x442bd6 + n * 20 + 15, 0)?;
        }
        for n in 0..1000_i16 {
            e.ww(pack(n), 0)?;
        }
        e.trap(0xa939, vec![L(e.l(0x43872a)?), W(0)])?;
        e.trap(0xa939, vec![L(e.l(0x43871e)?), W(0)])?;
        e.bw(0x442752, 1)?;
        e.trap(0xaa1c, vec![L(e.l(0x43952c)?)])?;
        let abort = e.w(f.wrapping_add(8))? == 2 || e.b(0x4386fc)? != 0;
        if abort {
            e.bw(0x442752, 0)?;
            mark_survivors(e)?;
            for n in 0..100_i16 {
                let a = npc(n);
                if e.w(a + 0x5a)? != -1 {
                    e.ww(a + 0x110, 0)?;
                }
            }
        }
        e.body_count(s)?;
        if !e.active() {
            return Ok(None);
        }
        let mut n = 0_i16;
        while n < e.w(0x44257c)? {
            e.step()?;
            let a = 0x440edc + u32::from(n as u16) * 288;
            if e.w(a + 0x26)? < 200 {
                e.ww(a + 0x4e, e.w(a + 0x26)?)?;
            }
            n = n.wrapping_add(1);
        }
        if abort {
            e.trap(0xa89b, vec![L(0x10001)])?;
            e.returned(PromptReturn::Full(0));
            return Ok(None);
        }
        if e.b(f.wrapping_sub(0x57))? != 0 && e.w(f.wrapping_add(10))? == 0 {
            e.sound(20, s)?;
        }
        if e.b(f.wrapping_sub(0x57))? != 0 && e.w(f.wrapping_add(10))? != 0 {
            e.sound(134, s)?;
        }
        e.trap(0xa93c, vec![L(e.l(0x4427be)?)])?;
        e.trap(0xa937, vec![])?;
        e.lw(f.wrapping_sub(0x9a), 0)?;
        e.lw(f.wrapping_sub(0x96), 0)?;
        e.lw(f.wrapping_sub(0xc6), 0)?;
        e.sane(0x280e, Some(f.wrapping_sub(0xc6)), f.wrapping_sub(0xc6))?;
        e.sane(0x1010, Some(f.wrapping_sub(0xc6)), f.wrapping_sub(0xc6))?;
        e.lw(f.wrapping_sub(0x82), e.l(f.wrapping_sub(0xc6))?)?;
        e.sane(0x100e, Some(f.wrapping_sub(0xc6)), f.wrapping_sub(0xbc))?;
        e.sane(0x16, None, f.wrapping_sub(0xbc))?;
        e.sane(0x2810, Some(f.wrapping_sub(0xbc)), f.wrapping_sub(0xbc))?;
        let mut count = e.w(f.wrapping_sub(0xba))?;
        e.ww(f.wrapping_sub(0x4c), count)?;
        e.ww(0x41f70e, count)?;
        e.ww(0x41f70c, count)?;
        e.ww(0x41f70a, count)?;
        if e.w(f.wrapping_add(8))? != 0 && e.w(f.wrapping_add(8))? != 5 {
            for n in 0..20_u32 {
                let a = 0x44331c + n * 2;
                if e.w(a)? > 0 {
                    count = e.w(f.wrapping_sub(0x4c))?;
                    e.addw(f.wrapping_sub(0x4c), 1)?;
                    e.ww(pack(count), e.w(a)?)?;
                    if e.w(a + 40)? != 0 {
                        e.addw(a + 40, e.w(0x442576)?)?;
                        if e.w(a + 40)? < 1 {
                            e.ww(a + 40, 1)?;
                        }
                    }
                    e.random_item(count, e.w(a + 40)?, s)?;
                    e.ww(a, 0)?;
                    e.ww(a + 40, 0)?;
                }
            }
            let value = e.l(0x44336c)?;
            let delta = if (value as i32) < 0 {
                e.lw(s.wrapping_sub(4), value)?;
                let abs = e.lib(0x400038, vec![L(value)])?.d0 as i16;
                i32::from(e.rand(abs, s)?) as u32
            } else {
                value
            };
            e.addl(0x443392, delta)?;
            let value = e.l(0x443370)?;
            let delta = if (value as i32) < 0 {
                e.lw(s.wrapping_sub(4), value)?;
                let abs = e.lib(0x400038, vec![L(value)])?.d0 as i16;
                i32::from(e.rand(abs, s)?) as u32
            } else {
                value
            };
            money = money.wrapping_add(delta);
            e.bw(f.wrapping_sub(0xa5), 1)?;
        }
        if e.w(f.wrapping_add(10))? != 0 {
            e.ww(0x43874a, 0)?;
        }
        if e.w(0x43874a)? != 0 {
            let mut n = 0_i16;
            while n < e.w(0x43874a)? {
                e.step()?;
                let a = npc(n);
                e.ww(a + 14, 1)?;
                if e.w(a + 14)? != 0 || e.w(a + 0x110)? < 1 {
                    money = money.wrapping_add(i32::from(e.range(0, e.w(a + 0x3a)?, s)?) as u32);
                    let level = e.w(a)? as u16;
                    let (exp, scale) = REWARDS
                        .get(usize::from(level))
                        .copied()
                        .unwrap_or((6200, 80));
                    e.ww(f.wrapping_sub(0x2a), exp)?;
                    if e.w(a + 0x10e)? != 0 {
                        e.lw(f.wrapping_sub(0xbc), 0x3fffc000)?;
                        e.lw(f.wrapping_sub(0xb8), 0)?;
                        e.ww(f.wrapping_sub(0xb4), 0)?;
                        e.sane(0x200e, Some(f.wrapping_sub(0x2a)), f.wrapping_sub(0xc6))?;
                        e.sane(4, Some(f.wrapping_sub(0xbc)), f.wrapping_sub(0xc6))?;
                        e.sane(0x16, None, f.wrapping_sub(0xc6))?;
                        e.sane(0x2810, Some(f.wrapping_sub(0xc6)), f.wrapping_sub(0xc6))?;
                        e.ww(f.wrapping_sub(0x2a), e.w(f.wrapping_sub(0xc4))?)?;
                    }
                    if e.b(a + 0x7f)? != 0 {
                        e.addw(f.wrapping_sub(0x2a), e.w(a)?.wrapping_mul(30))?;
                    }
                    e.addl(0x443392, i32::from(e.w(f.wrapping_sub(0x2a))?) as u32)?;
                    e.addl(0x443392, i32::from(e.w(a + 0x54)?) as u32)?;
                    e.addl(
                        0x443392,
                        i32::from(scale.wrapping_mul(e.w(a + 0x112)?)) as u32,
                    )?;
                    if e.w(f.wrapping_add(8))? == 5 {
                        for k in 0..1000_i16 {
                            e.bw(0x42df04 + k as u32, 0)?;
                            e.ww(pack(k), 0)?;
                        }
                        n = 1000;
                        money = 0;
                    } else {
                        for k in 0..6_u32 {
                            let item = a + 0x26 + k * 2;
                            if e.w(item)? != 0 {
                                e.load_item(e.w(item)?, s)?;
                                e.ww(
                                    f.wrapping_sub(0xb2),
                                    70_i16.wrapping_sub(e.w(0x442576)?.wrapping_mul(20)),
                                )?;
                                let die = e.rand(150, s.wrapping_sub(2))?;
                                if die < e.w(f.wrapping_sub(0xb2))?
                                    || [25, 23, 24].contains(&e.w(0x453656)?)
                                {
                                    let at = e.w(f.wrapping_sub(0x4c))?;
                                    e.addw(f.wrapping_sub(0x4c), 1)?;
                                    e.ww(pack(at), e.w(item)?)?;
                                }
                            }
                        }
                    }
                }
                n = n.wrapping_add(1);
            }
            let a = npc(n);
            for k in 0..40 {
                e.bw(a + 0xb2 + k, 0)?;
            }
            e.ww(a + 14, 0)?;
            e.ww(a + 0x110, 0)?;
        }
        e.addl(0x442788, money)?;
        e.ww(0x43874a, 0)?;
        e.ww(0x43874c, 0)?;
        let mut n = 0_i16;
        while n <= i16::from(e.b(0x44273a)? as i8) {
            e.step()?;
            let a = warrior(n);
            if e.w(a + 0x50)? > 0 && e.w(a + 0x90)? > -1 && e.b(a + 0x9df)? != 0 {
                e.addl(f.wrapping_sub(0x9a), 1)?;
            }
            n = n.wrapping_add(1);
        }
        e.addl(f.wrapping_sub(0x9a), i32::from(e.w(0x44257e)?) as u32)?;
        e.ww(0x44257e, 0)?;
        if e.l(f.wrapping_sub(0x9a))? != 0 {
            e.lw(f.wrapping_sub(0xbc), e.l(0x443392)?)?;
            e.sane(0x280e, Some(f.wrapping_sub(0xbc)), f.wrapping_sub(0xbc))?;
            e.sane(0x1010, Some(f.wrapping_sub(0xbc)), f.wrapping_sub(0xbc))?;
            e.lw(f.wrapping_sub(0x82), e.l(f.wrapping_sub(0xbc))?)?;
            e.sane(0x100e, Some(f.wrapping_sub(0x82)), f.wrapping_sub(0xbc))?;
            e.sane(0x16, None, f.wrapping_sub(0xbc))?;
            e.sane(0x2810, Some(f.wrapping_sub(0xbc)), f.wrapping_sub(0xbc))?;
            let quotient = (e.l(f.wrapping_sub(0xbc))? as i32)
                .checked_div(e.l(f.wrapping_sub(0x9a))? as i32)
                .ok_or_else(|| {
                    RuleError::Invalid("native loot reward division exception".into())
                })?;
            e.lw(f.wrapping_sub(0x96), quotient as u32)?;
            if (e.l(f.wrapping_sub(0x96))? as i32) < 0 {
                e.lw(f.wrapping_sub(0x96), (quotient.wrapping_neg()) as u32)?;
            }
        }
        e.ww(f.wrapping_add(8), 0)?;
        count = 0;
        if e.b(f.wrapping_sub(0xa5))? == 0 {
            for n in 0..1000_i16 {
                if e.w(pack(n))? != 0 {
                    e.random_item(n, 0, s)?;
                    count = count.wrapping_add(1);
                }
            }
        }
        if e.w(f.wrapping_add(10))? == 1 {
            count = 0;
            for n in 0..1000_i16 {
                let a = 0x4285f4 + n as u32 * 22;
                if e.w(a)? != 0 {
                    e.copy(a, pack(n), 5, true)?;
                    count = count.wrapping_add(1);
                }
            }
        }
        if e.w(f.wrapping_add(10))? == 0 {
            for _ in 0..count.max(0) {
                for n in 0..count.max(0) {
                    e.step()?;
                    let next = n.wrapping_add(1);
                    if (e.b(pack(next) + 10)? as i8) > (e.b(pack(n) + 10)? as i8) {
                        e.copy(pack(next), 0x41fc48, 5, true)?;
                        e.ww(
                            f.wrapping_sub(0x4e),
                            i16::from(e.b(0x42df04 + next as u32)? as i8),
                        )?;
                        e.copy(pack(n), pack(next), 5, true)?;
                        e.bw(0x42df04 + next as u32, e.b(0x42df04 + n as u32)?)?;
                        e.copy(0x41fc48, pack(n), 5, true)?;
                        e.bw(0x42df04 + n as u32, e.b(f.wrapping_sub(0x4d))?)?;
                    }
                }
            }
        }
    }
    let mut count = 0_i16;
    for n in 0..1000_i16 {
        if e.w(pack(n))? != 0 {
            count = count.wrapping_add(1);
        }
    }
    e.ww(f.wrapping_sub(0x4c), 0)?;
    let mut n = 0_i16;
    while n <= i16::from(e.b(0x44273a)? as i8) {
        e.step()?;
        let a = warrior(n);
        if e.w(a + 0x50)? > 0 && e.w(a + 0x90)? > -1 && e.b(a + 0x9df)? != 0 {
            e.lw(f.wrapping_sub(0xbc), e.l(f.wrapping_sub(0x96))?)?;
            e.sane(0x280e, Some(f.wrapping_sub(0xbc)), f.wrapping_sub(0xbc))?;
            e.sane(0x1010, Some(f.wrapping_sub(0xbc)), f.wrapping_sub(0xbc))?;
            e.lw(f.wrapping_sub(0x82), e.l(f.wrapping_sub(0xbc))?)?;
            e.profile(e.w(a + 0x32)?, 0, s)?;
            e.sane(0x280e, Some(a + 0x9d2), f.wrapping_sub(0xbc))?;
            e.sane(0x1000, Some(f.wrapping_sub(0x82)), f.wrapping_sub(0xbc))?;
            e.sane(0x16, None, f.wrapping_sub(0xbc))?;
            e.sane(0x2810, Some(f.wrapping_sub(0xbc)), f.wrapping_sub(0xbc))?;
            e.lw(a + 0x9d2, e.l(f.wrapping_sub(0xbc))?)?;
            if (e.l(a + 0x9d2)? as i32) < 0 {
                let value = e.l(a + 0x9d2)?;
                e.lw(s.wrapping_sub(4), value)?;
                let abs = e.lib(0x400038, vec![L(value)])?.d0 as i16;
                e.lw(a + 0x9d2, i32::from(abs) as u32)?;
            }
            e.sane(0x280e, Some(a + 0x9ce), f.wrapping_sub(0xbc))?;
            e.sane(0x1002, Some(f.wrapping_sub(0x82)), f.wrapping_sub(0xbc))?;
            e.sane(0x16, None, f.wrapping_sub(0xbc))?;
            e.sane(0x2810, Some(f.wrapping_sub(0xbc)), f.wrapping_sub(0xbc))?;
            e.lw(a + 0x9ce, e.l(f.wrapping_sub(0xbc))?)?;
        }
        n = n.wrapping_add(1);
    }
    Ok(Some(LootPreparation { money, count }))
}
fn mark_survivors<S: LootControlServices>(e: &mut Execution<'_, S>) -> Result<(), RuleError> {
    let mut n = 0_i16;
    while n <= i16::from(e.b(0x44273a)? as i8) {
        e.step()?;
        e.bw(warrior(n) + 0x9df, 1)?;
        n = n.wrapping_add(1);
    }
    Ok(())
}
const REWARDS: [(i16, i16); 21] = [
    (15, 3),
    (30, 6),
    (45, 9),
    (65, 12),
    (80, 15),
    (100, 18),
    (140, 21),
    (200, 24),
    (300, 27),
    (450, 30),
    (700, 33),
    (1100, 36),
    (1800, 39),
    (2300, 42),
    (2800, 45),
    (3200, 50),
    (3700, 55),
    (4200, 60),
    (4700, 65),
    (5200, 70),
    (5700, 75),
];

impl<S: LootControlServices> Execution<'_, S> {
    fn character(&mut self, entry: u32, frame: u32) -> Result<(), RuleError> {
        self.step()?;
        let child = match entry {
            0x15f66c => character::update_character(
                &self.plan.state.memory,
                frame,
                self.remaining,
                self.services,
            )?,
            0x161b80 => character::update_picture_box(
                &self.plan.state.memory,
                frame,
                self.remaining,
                self.services,
            )?,
            _ => return Err(RuleError::Invalid("unknown loot character child".into())),
        };
        self.plan.state.memory = child.memory;
        self.d0 = Some(child.native_return);
        self.plan
            .effects
            .extend(child.effects.into_iter().map(LootControlEffect::Character));
        Ok(())
    }
    fn row(&mut self, member: i16, mode: i16, stack: u32) -> Result<(), RuleError> {
        self.lw(
            stack.wrapping_sub(4),
            (u32::from(member as u16) << 16) | u32::from(mode as u16),
        )?;
        self.character(0x15f66c, stack.wrapping_sub(12))
    }
    fn picture_box(&mut self, member: i16, mode: u32, stack: u32) -> Result<(), RuleError> {
        self.lw(stack.wrapping_sub(4), mode)?;
        self.ww(stack.wrapping_sub(6), member)?;
        self.character(0x161b80, stack.wrapping_sub(14))
    }
    fn quick_info(
        &mut self,
        member: i16,
        slot: i16,
        item: i16,
        mode: i16,
        a2: u32,
        stack: u32,
    ) -> Result<(), RuleError> {
        self.ww(stack.wrapping_sub(2), mode)?;
        self.ww(stack.wrapping_sub(4), item)?;
        self.ww(stack.wrapping_sub(6), slot)?;
        self.ww(stack.wrapping_sub(8), member)?;
        let child = detail::quick_info(
            &self.plan.state.memory,
            stack.wrapping_sub(16),
            a2,
            self.limits.prompt.filename_capacity,
            self.services,
        )?;
        self.plan.state.memory = child.memory;
        self.d0 = child.native_return.map(|v| v as u32);
        self.plan
            .effects
            .extend(child.effects.into_iter().map(LootControlEffect::Detail));
        Ok(())
    }
    fn show_pack(
        &mut self,
        slot: i16,
        markers: i16,
        rect: u32,
        stack: u32,
    ) -> Result<(), RuleError> {
        self.ww(stack.wrapping_sub(2), markers)?;
        self.lw(stack.wrapping_sub(6), self.l(rect.wrapping_add(4))?)?;
        self.lw(stack.wrapping_sub(10), self.l(rect)?)?;
        self.ww(stack.wrapping_sub(12), slot)?;
        let child = item_ui::show_pack(
            &self.plan.state.memory,
            stack.wrapping_sub(20),
            self.remaining as usize,
            self.services,
        )?;
        self.plan.state.memory = child.memory;
        self.d0 = child.native_return.map(|v| v as u32);
        self.plan
            .effects
            .extend(child.effects.into_iter().map(LootControlEffect::ItemUi));
        Ok(())
    }
    fn can_use(&mut self, member: i16, stack: u32) -> Result<bool, RuleError> {
        let a = warrior(member);
        let caste = self.w(a + 0x34)?;
        self.ww(stack.wrapping_sub(2), caste)?;
        let race = self.w(a + 0x32)?;
        self.ww(stack.wrapping_sub(4), race)?;
        self.ww(stack.wrapping_sub(6), -1)?;
        let observations = if race == 0 && caste == 0 {
            vec![]
        } else {
            equipment::EquipmentServices::profile_observations(
                self.services,
                race,
                caste,
                &self.plan.state.memory,
            )?
        };
        let child = restriction::canuse(
            &self.plan.state.memory,
            stack.wrapping_sub(14),
            self.limits.prompt.filename_capacity,
            &observations,
        )?;
        self.plan.state.memory = child.memory;
        self.d0 = Some(child.native_return);
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(LootControlEffect::Restriction),
        );
        Ok(child.native_return as u16 != 0)
    }
    fn text_box(&mut self, rect: u32, group: i16, id: i16, stack: u32) -> Result<(), RuleError> {
        self.lw(stack.wrapping_sub(4), self.l(rect.wrapping_add(4))?)?;
        self.lw(stack.wrapping_sub(8), self.l(rect)?)?;
        self.lw(stack.wrapping_sub(12), 1)?;
        self.ww(stack.wrapping_sub(14), id)?;
        self.ww(stack.wrapping_sub(16), group)?;
        self.prompt(0x15dc34, stack.wrapping_sub(24))
    }
    fn format(&mut self, entry: u32, value: i16, stack: u32) -> Result<(), RuleError> {
        self.ww(stack.wrapping_sub(2), value)?;
        self.lib(0x400168, vec![L(i32::from(value) as u32), P(0x479ed8)])?;
        let current = self.w(stack.wrapping_sub(2))?;
        let mut context = TextContext::default();
        let mut fixed = FixedNumber(pascal(&self.plan.state.memory, 0x479ed8)?);
        let _child = match entry {
            0x1382d0 => text::string(&mut context, value, &mut fixed)?,
            0x13832a => text::string_plus(&mut context, current, &mut fixed)?,
            0x138368 => text::string_no_zero_no_plus(&mut context, current, &mut fixed)?,
            0x1383a6 => text::string_no_zero(&mut context, current, &mut fixed)?,
            _ => return Err(RuleError::Invalid("unknown loot number child".into())),
        };
        if [0x13832a, 0x1383a6].contains(&entry) && current > 0 {
            self.trap(0xa884, vec![P(0x40227a)])?;
        }
        if [0x1382d0, 0x13832a].contains(&entry) || current > 0 {
            self.trap(0xa884, vec![P(0x479ed8)])?;
        }
        Ok(())
    }
    fn long_number(&mut self, value: u32, stack: u32) -> Result<(), RuleError> {
        self.lw(stack.wrapping_sub(4), value)?;
        self.lib(0x400168, vec![L(value), P(0x479ed8)])?;
        let _child = text::string_long(
            &mut TextContext::default(),
            value as i32,
            &mut FixedNumber(pascal(&self.plan.state.memory, 0x479ed8)?),
        )?;
        self.trap(0xa884, vec![P(0x479ed8)])?;
        Ok(())
    }
    fn move_pen(&mut self, h: i16, v: i16) -> Result<(), RuleError> {
        self.trap(0xa893, vec![W(h as u16), W(v as u16)])?;
        Ok(())
    }
    fn create_control(&mut self, rect: u32) -> Result<u32, RuleError> {
        self.trap(
            0xa954,
            vec![
                L(self.l(0x438790)?),
                P(rect),
                P(0x403fae),
                FeedbackArg::Byte(1),
                L(0),
                L(0),
                L(0),
            ],
        )
    }
    fn convert_long(&mut self, value: u32, retained: u32) -> Result<(), RuleError> {
        let base = self.stack.wrapping_sub(retained);
        self.lw(base.wrapping_sub(4), value)?;
        self.lw(base.wrapping_sub(8), 0x479ed8)?;
        self.lib(0x400168, vec![L(value), P(0x479ed8)])?;
        self.stack = self.stack.wrapping_sub(8);
        Ok(())
    }
    fn money(&mut self, rect: u32, amount: u32, stack: u32) -> Result<(), RuleError> {
        self.lw(stack.wrapping_sub(4), amount)?;
        self.lw(stack.wrapping_sub(8), self.l(rect.wrapping_add(4))?)?;
        self.lw(stack.wrapping_sub(12), self.l(rect)?)?;
        let frame = stack.wrapping_sub(20);
        let argument = frame.wrapping_add(8);
        self.trap(0xa873, vec![L(self.l(0x438790)?)])?;
        self.trap(0xa862, vec![L(69)])?;
        self.ww(
            argument.wrapping_add(6),
            self.w(argument.wrapping_add(2))?.wrapping_add(640),
        )?;
        self.pict(frame, 163, argument)?;
        self.move_pen(
            self.w(argument.wrapping_add(2))?.wrapping_add(20),
            self.w(argument)?.wrapping_add(46),
        )?;
        self.lib(
            0x400168,
            vec![L(self.l(frame.wrapping_add(16))?), P(0x479ed8)],
        )?;
        self.trap(0xa884, vec![P(0x479ed8)])?;
        self.d0 = Some(0);
        Ok(())
    }
}

fn flat<S: LootControlServices>(
    e: &Execution<'_, S>,
    row: i16,
    column: i16,
) -> Result<i16, RuleError> {
    let a = e
        .w(0x439582)?
        .wrapping_mul(2)
        .wrapping_add(5)
        .wrapping_mul(row);
    let b = e
        .w(0x439582)?
        .wrapping_mul(2)
        .wrapping_add(5)
        .wrapping_mul(e.w(0x41f70e)?);
    Ok(a.wrapping_add(b).wrapping_add(column))
}
fn control(value: Option<u32>, register: &str) -> Result<u32, RuleError> {
    value.ok_or_else(|| {
        RuleError::Invalid(format!(
            "loot reached inherited {register} control register without an input"
        ))
    })
}
fn display_cell<S: LootControlServices>(
    e: &mut Execution<'_, S>,
    m: &mut Modal,
    f: u32,
    row: i16,
    column: i16,
    scroll: Option<bool>,
) -> Result<(), RuleError> {
    let s = e.stack;
    let slot = if scroll == Some(false) {
        flat(e, 0, column)?
            .wrapping_add(15)
            .wrapping_add(e.w(0x439582)?.wrapping_mul(6))
    } else {
        flat(e, row, column)?
    };
    e.ww(0x4434b6, e.w(pack(slot))?)?;
    if e.w(0x4434b6)? == 0 {
        return Ok(());
    }
    e.load_item(e.w(0x4434b6)?, s)?;
    if scroll != Some(true) {
        e.lw(0x439598, 0)?;
        let handle = e.trap(0xaa1e, vec![W(e.w(0x453646)? as u16)])?;
        e.lw(0x439598, handle)?;
    }
    let top = match scroll {
        Some(true) => 15,
        Some(false) => 321,
        None => row.wrapping_mul(102).wrapping_add(15),
    };
    e.ww(f.wrapping_sub(8), top)?;
    if scroll.is_some() {
        e.ww(f.wrapping_sub(4), e.w(f.wrapping_sub(8))?.wrapping_add(32))?;
    }
    let left = e
        .w(0x439582)?
        .wrapping_mul(5)
        .wrapping_add(63)
        .wrapping_mul(column)
        .wrapping_add(8);
    e.ww(f.wrapping_sub(6), left)?;
    e.ww(f.wrapping_sub(2), e.w(f.wrapping_sub(6))?.wrapping_add(32))?;
    if scroll.is_none() {
        e.ww(f.wrapping_sub(4), e.w(f.wrapping_sub(8))?.wrapping_add(32))?;
    }
    if scroll == Some(true) {
        e.lw(0x439598, 0)?;
        let handle = e.trap(0xaa1e, vec![W(e.w(0x453646)? as u16)])?;
        e.lw(0x439598, handle)?;
    }
    if e.l(0x439598)? == 0 {
        let handle = e.trap(0xaa1e, vec![W(6520)])?;
        e.lw(0x439598, handle)?;
    }
    if e.l(0x439598)? == 0 {
        return Ok(());
    }
    let top = match scroll {
        Some(true) => 4,
        Some(false) => 310,
        None => row.wrapping_mul(102).wrapping_add(4),
    };
    e.ww(0x4434ba, top)?;
    let left = e
        .w(0x439582)?
        .wrapping_mul(5)
        .wrapping_add(63)
        .wrapping_mul(column)
        .wrapping_add(4);
    e.ww(0x4434bc, left)?;
    e.ww(
        0x4434c0,
        e.w(0x439582)?
            .wrapping_mul(4)
            .wrapping_add(60)
            .wrapping_add(e.w(0x4434bc)?),
    )?;
    e.ww(0x4434be, e.w(0x4434ba)?.wrapping_add(100))?;
    e.trap(0xa89c, vec![W(0)])?;
    e.trap(0xa862, vec![L(30)])?;
    e.trap(0xa8a2, vec![P(0x4434ba)])?;
    e.trap(0xaa1f, vec![P(f.wrapping_sub(8)), L(e.l(0x439598)?)])?;
    e.trap(0xaa25, vec![L(e.l(0x439598)?)])?;
    let marker_index = if scroll.is_some() {
        flat(e, 0, column)?.wrapping_add(m.d4)
    } else {
        flat(e, row, column)?
    };
    let markers = i16::from(e.b(index(0x42df04, marker_index, 1))? as i8);
    let slot = if scroll == Some(false) {
        flat(e, 0, column)?
            .wrapping_add(15)
            .wrapping_add(e.w(0x439582)?.wrapping_mul(6))
    } else {
        flat(e, row, column)?
    };
    e.show_pack(slot, markers, f.wrapping_sub(8), s)?;
    Ok(())
}
fn grid<S: LootControlServices>(
    e: &mut Execution<'_, S>,
    m: &mut Modal,
    f: u32,
) -> Result<(), RuleError> {
    let s = e.stack;
    for row in 0..4_i16 {
        m.d4 = 0;
        while m.d4 < e.w(0x439582)?.wrapping_mul(2).wrapping_add(5) {
            e.step()?;
            display_cell(e, m, f, row, m.d4, None)?;
            m.d4 = m.d4.wrapping_add(1);
        }
    }
    e.ww(f.wrapping_sub(0x4c), 0)?;
    e.ww(f.wrapping_sub(6), e.w(0x40856c)?.wrapping_add(320))?;
    e.ww(f.wrapping_sub(2), e.w(0x40856c)?.wrapping_add(640))?;
    e.ww(f.wrapping_sub(8), 0)?;
    e.ww(
        f.wrapping_sub(4),
        i16::from(e.b(0x44273a)? as i8)
            .wrapping_add(1)
            .wrapping_mul(50),
    )?;
    if e.b(f.wrapping_sub(0x57))? != 0 {
        e.trap(0xa873, vec![L(e.l(0x438790)?)])?;
        e.trap(0xaa0b, vec![L(e.l(0x4358da)?)])?;
        let handle = e.create_control(f.wrapping_sub(8))?;
        e.lw(f.wrapping_sub(0x9e), handle)?;
        let mut n = 0_i16;
        while n <= i16::from(e.b(0x44273a)? as i8) {
            e.step()?;
            e.row(n, 4, s)?;
            n = n.wrapping_add(1);
        }
        e.ww(f.wrapping_sub(8), 0)?;
        e.ww(f.wrapping_sub(4), 50)?;
        e.ww(f.wrapping_sub(6), e.w(0x40856c)?.wrapping_add(380))?;
        e.ww(f.wrapping_sub(2), e.w(0x40856c)?.wrapping_add(640))?;
        e.lw(f.wrapping_sub(0x20), e.l(f.wrapping_sub(8))?)?;
        e.lw(f.wrapping_sub(0x1c), e.l(f.wrapping_sub(4))?)?;
        e.trap(0xa873, vec![L(e.l(0x438790)?)])?;
        e.trap(0xaa0b, vec![L(e.l(0x4358da)?)])?;
        e.trap(0xa862, vec![L(33)])?;
        e.trap(0xa863, vec![L(30)])?;
        e.lw(f.wrapping_sub(0x20), e.l(f.wrapping_sub(8))?)?;
        e.lw(f.wrapping_sub(0x1c), e.l(f.wrapping_sub(4))?)?;
        e.trap(0xa862, vec![L(69)])?;
        portrait_rows(e, f)?;
    }
    e.trap(0xa862, vec![L(69)])?;
    let mut n = 0_i16;
    while n <= i16::from(e.b(0x44273a)? as i8) {
        e.step()?;
        e.ww(s.wrapping_sub(2), n)?;
        e.selection(0x15f4b8, s.wrapping_sub(10))?;
        if !e.active() {
            return Ok(());
        }
        n = n.wrapping_add(1);
    }
    e.ww(f.wrapping_sub(6), e.w(0x40856c)?.wrapping_add(320))?;
    e.ww(
        f.wrapping_sub(8),
        i16::from(e.b(0x44273a)? as i8)
            .wrapping_add(1)
            .wrapping_mul(50),
    )?;
    e.ww(f.wrapping_sub(2), e.w(0x40856c)?.wrapping_add(640))?;
    e.ww(f.wrapping_sub(4), 300)?;
    e.trap(0xaa0b, vec![L(e.l(0x4358ce)?)])?;
    e.trap(0xa8a3, vec![P(f.wrapping_sub(8))])?;
    e.trap(0xaa0b, vec![L(e.l(0x4358da)?)])?;
    e.ww(f.wrapping_sub(8), e.w(f.wrapping_sub(4))?)?;
    e.ww(f.wrapping_sub(4), e.w(f.wrapping_sub(8))?.wrapping_add(110))?;
    e.pict(s, 198, f.wrapping_sub(8))?;
    e.lw(f.wrapping_sub(0x74), e.l(f.wrapping_sub(8))?)?;
    e.lw(f.wrapping_sub(0x70), e.l(f.wrapping_sub(4))?)?;
    e.trap(0xa862, vec![L(69)])?;
    e.addw(f.wrapping_sub(8), 35)?;
    e.addw(f.wrapping_sub(6), 12)?;
    e.addw(f.wrapping_sub(2), -10)?;
    e.addw(f.wrapping_sub(4), -10)?;
    e.lw(f.wrapping_sub(0x28), e.l(f.wrapping_sub(8))?)?;
    e.lw(f.wrapping_sub(0x24), e.l(f.wrapping_sub(4))?)?;
    Ok(())
}
fn portrait_rows<S: LootControlServices>(
    e: &mut Execution<'_, S>,
    f: u32,
) -> Result<(), RuleError> {
    let s = e.stack;
    let mut n = 0_i16;
    while n <= i16::from(e.b(0x44273a)? as i8) {
        e.step()?;
        e.picture_box(n, 1, s)?;
        e.addw(f.wrapping_sub(0x20), 50)?;
        e.addw(f.wrapping_sub(0x1c), 50)?;
        e.move_pen(
            e.w(0x40856c)?.wrapping_add(393),
            e.w(f.wrapping_sub(0x20))?.wrapping_sub(30),
        )?;
        e.trap(0xa862, vec![L(69)])?;
        let pointer = warrior(n) + 0xa21;
        e.lib(0x400188, vec![P(pointer)])?;
        e.trap(0xa884, vec![P(warrior(n) + 0xa21)])?;
        e.lib(0x400180, vec![P(warrior(n) + 0xa21)])?;
        n = n.wrapping_add(1);
    }
    Ok(())
}
fn summary_text<S: LootControlServices>(
    e: &mut Execution<'_, S>,
    f: u32,
    count: i16,
    money: u32,
    clear: bool,
) -> Result<(), RuleError> {
    if e.w(0x443314)? == 0 {
        e.ww(0x4434b6, 71)?;
        if count != 0 || money != 0 {
            e.ww(0x4434b6, 35)?;
        }
        if e.l(0x443392)? != 0 && e.w(0x4434b6)? == 35 {
            e.ww(0x4434b6, 36)?;
        } else if e.l(0x443392)? != 0 {
            e.ww(0x4434b6, 34)?;
        }
        if clear && e.w(f.wrapping_add(10))? != 0 {
            e.ww(0x4434b6, 152)?;
        }
        e.text_box(f.wrapping_sub(0x28), 3, e.w(0x4434b6)?, e.stack)?;
    } else {
        e.text_box(
            f.wrapping_sub(0x28),
            -1,
            e.w(0x443314)?.wrapping_neg(),
            e.stack,
        )?;
    }
    Ok(())
}

fn open_window<S: LootControlServices>(e: &mut Execution<'_, S>, f: u32) -> Result<(), RuleError> {
    let handle = e.trap(0xaa46, vec![W(130), L(0), L(u32::MAX)])?;
    e.lw(0x438790, handle)?;
    e.trap(0xa873, vec![L(e.l(0x438790)?)])?;
    e.trap(
        0xa91b,
        vec![
            L(e.l(0x438790)?),
            W(e.w(0x4395a4)? as u16),
            W(e.w(0x4395a2)? as u16),
            FeedbackArg::Byte(0),
        ],
    )?;
    if e.w(0x439582)? != 0 {
        let width = e.w(0x40856c)?.wrapping_add(640);
        let height = e
            .w(0x40856e)?
            .wrapping_add(460)
            .wrapping_sub(e.w(0x439582)?.wrapping_mul(20));
        e.trap(
            0xa91d,
            vec![
                L(e.l(0x438790)?),
                W(width as u16),
                W(height as u16),
                FeedbackArg::Byte(0),
            ],
        )?;
    }
    e.trap(
        0xa91b,
        vec![
            L(e.l(0x438790)?),
            W(e.w(0x4395a4)? as u16),
            W(e.w(0x4395a2)? as u16),
            FeedbackArg::Byte(0),
        ],
    )?;
    e.trap(0xa88a, vec![W(12)])?;
    e.trap(0xaa0b, vec![L(e.l(0x4358da)?)])?;
    e.trap(0xa887, vec![W(e.w(0x402f44)? as u16)])?;
    e.trap(0xa915, vec![L(e.l(0x438790)?)])?;
    let mut n = 0_i16;
    while n <= i16::from(e.b(0x44273a)? as i8) {
        e.step()?;
        let handle = e.trap(
            0xa9be,
            vec![W(n.wrapping_add(178) as u16), L(e.l(0x438790)?)],
        )?;
        let slot = index(f.wrapping_sub(0x4a), n, 4);
        e.lw(slot, handle)?;
        let handle = e.l(slot)?;
        let h = e
            .w(0x40856c)?
            .wrapping_add(e.w(e.l(handle)?.wrapping_add(10))?);
        let v = e.w(e.l(e.l(slot)?)?.wrapping_add(8))?;
        e.trap(0xa959, vec![L(handle), W(h as u16), W(v as u16)])?;
        n = n.wrapping_add(1);
    }
    Ok(())
}
fn side_numbers<S: LootControlServices>(
    e: &mut Execution<'_, S>,
    f: u32,
    count: i16,
    id: u16,
) -> Result<(), RuleError> {
    e.ww(0x4434bc, 0)?;
    e.ww(0x4434c0, e.w(0x40856c)?.wrapping_add(320))?;
    e.ww(0x4434ba, if id == 0 { 440 } else { 608 })?;
    e.ww(0x4434be, if id == 0 { 460 } else { 640 })?;
    e.pict(e.stack, (230_i16).wrapping_sub(e.w(0x439582)?), 0x4434ba)?;
    e.trap(0xa88a, vec![W(12)])?;
    e.trap(0xaa0b, vec![L(e.l(0x4358da)?)])?;
    e.trap(0xa887, vec![W(e.w(0x402f44)? as u16)])?;
    e.trap(0xa888, vec![W(0)])?;
    e.trap(0xa862, vec![L(69)])?;
    let from = e
        .w(0x439582)?
        .wrapping_mul(2)
        .wrapping_add(5)
        .wrapping_mul(e.w(0x41f70e)?)
        .wrapping_add(1);
    e.convert_long(i32::from(from) as u32, 10)?;
    let width = e.trap(0xa88c, vec![P(0x479ed8)])? as i16;
    e.move_pen(90_i16.wrapping_sub(width), 456)?;
    e.trap(0xa884, vec![P(0x479ed8)])?;
    e.move_pen(125, 456)?;
    let a = e
        .w(0x439582)?
        .wrapping_mul(2)
        .wrapping_add(5)
        .wrapping_mul(e.w(0x41f70e)?);
    let to = a.wrapping_add(e.w(0x439582)?.wrapping_mul(8).wrapping_add(20));
    e.convert_long(i32::from(to) as u32, 10)?;
    e.trap(0xa884, vec![P(0x479ed8)])?;
    e.move_pen(200, 456)?;
    e.convert_long(i32::from(count) as u32, 10)?;
    e.trap(0xa884, vec![P(0x479ed8)])?;
    let _ = f;
    Ok(())
}
fn footer<S: LootControlServices>(
    e: &mut Execution<'_, S>,
    m: &mut Modal,
    f: u32,
    count: i16,
    money: u32,
) -> Result<(), RuleError> {
    summary_text(e, f, count, money, false)?;
    if !e.active() {
        return Ok(());
    }
    e.ww(f.wrapping_sub(4), 410)?;
    if e.l(f.wrapping_sub(0x96))? != 0 {
        e.ww(0x4434ba, 439)?;
        e.ww(0x4434be, 460)?;
        e.ww(0x4434bc, 0)?;
        e.ww(0x4434c0, e.w(0x40856c)?.wrapping_add(320))?;
        e.pict(e.stack, 232_i16.wrapping_sub(e.w(0x439582)?), 0x4434ba)?;
        e.move_pen(125, e.w(0x4434ba)?.wrapping_add(16))?;
        e.convert_long(e.l(f.wrapping_sub(0x96))?, 10)?;
        e.trap(0xa884, vec![P(0x479ed8)])?;
    }
    e.ww(f.wrapping_sub(14), 4)?;
    e.ww(f.wrapping_sub(10), e.w(0x40856c)?.wrapping_add(316))?;
    e.ww(f.wrapping_sub(8), e.w(f.wrapping_sub(4))?)?;
    e.ww(f.wrapping_sub(4), e.w(f.wrapping_sub(8))?.wrapping_add(50))?;
    e.ww(f.wrapping_sub(6), e.w(0x40856c)?.wrapping_add(320))?;
    e.ww(f.wrapping_sub(2), e.w(f.wrapping_sub(6))?.wrapping_add(50))?;
    if e.b(f.wrapping_sub(0x57))? != 0 {
        m.registers.a2 = Some(e.create_control(f.wrapping_sub(8))?);
    }
    e.pict(e.stack, 168, f.wrapping_sub(8))?;
    if count > e.w(0x439582)?.wrapping_mul(8).wrapping_add(20) {
        e.addw(f.wrapping_sub(2), 50)?;
        e.addw(f.wrapping_sub(6), 50)?;
        e.ww(f.wrapping_sub(4), e.w(f.wrapping_sub(8))?.wrapping_add(25))?;
        if e.b(f.wrapping_sub(0x57))? != 0 {
            m.registers.a4 = Some(e.create_control(f.wrapping_sub(8))?);
        }
        e.addw(f.wrapping_sub(8), 25)?;
        e.addw(f.wrapping_sub(4), 25)?;
        if e.b(f.wrapping_sub(0x57))? != 0 {
            let h = e.create_control(f.wrapping_sub(8))?;
            e.lw(f.wrapping_sub(0x86), h)?;
        }
        e.addw(f.wrapping_sub(8), -25)?;
        e.pict(e.stack, 132, f.wrapping_sub(8))?;
    }
    e.addw(f.wrapping_sub(6), 50)?;
    e.addw(f.wrapping_sub(2), 70)?;
    if e.w(f.wrapping_add(10))? != 0 {
        side_numbers(e, f, count, 0)?;
    }
    e.money(f.wrapping_sub(8), money, e.stack)?;
    e.ww(
        f.wrapping_sub(0x7c),
        i16::from(e.b(0x44273f)? as i8).wrapping_mul(50),
    )?;
    e.ww(f.wrapping_sub(0x7a), e.w(0x40856c)?.wrapping_add(320))?;
    e.ww(
        f.wrapping_sub(0x76),
        e.w(f.wrapping_sub(0x7a))?.wrapping_add(10),
    )?;
    e.ww(
        f.wrapping_sub(0x78),
        e.w(f.wrapping_sub(0x7c))?.wrapping_add(50),
    )?;
    e.trap(0xa8f6, vec![L(e.l(0x43955c)?), P(f.wrapping_sub(0x7c))])?;
    if e.w(0x439582)? != 0 {
        e.quick_info(
            -1,
            0,
            e.w(0x41fc74)?,
            2,
            control(m.registers.a2, "A2")?,
            e.stack,
        )?;
    }
    Ok(())
}

fn signed_div_word(n: i32, d: i16) -> Result<i16, RuleError> {
    let q = n
        .checked_div(i32::from(d))
        .ok_or_else(|| RuleError::Invalid("native loot WORD division exception".into()))?;
    i16::try_from(q).map_err(|_| RuleError::Invalid("native loot WORD division overflow".into()))
}
fn value_word<S: LootControlServices>(
    e: &mut Execution<'_, S>,
    sp: &mut u32,
    entry: u32,
    value: i16,
) -> Result<(), RuleError> {
    e.format(entry, value, *sp)?;
    *sp = sp.wrapping_sub(2);
    Ok(())
}
fn hover<S: LootControlServices>(
    e: &mut Execution<'_, S>,
    m: &mut Modal,
    f: u32,
    count: i16,
    money: u32,
) -> Result<(), RuleError> {
    let inside = e.trap(0xa8ad, vec![L(e.l(0x4471ee)?), P(f.wrapping_sub(0x92))])? as u8 != 0;
    if !inside {
        if e.w(f.wrapping_sub(0x7e))? > -1 {
            e.trap(0xa89c, vec![W(2)])?;
            if count != 0 {
                e.trap(0xa8a1, vec![P(f.wrapping_sub(0x56))])?;
            }
            e.ww(f.wrapping_sub(0x7e), -1)?;
            for at in [0x66, 0x64, 0x68, 0x6a] {
                e.ww(f.wrapping_sub(at), -1)?;
            }
            e.pict(e.stack, 198, f.wrapping_sub(0x74))?;
            summary_text(e, f, count, money, true)?;
        }
        return Ok(());
    }
    e.trap(0xa862, vec![L(69)])?;
    let row = signed_div_word(i32::from(e.w(0x4471ee)?.wrapping_sub(10)), 100)?;
    e.ww(0x4434b6, row)?;
    if e.w(0x4434b6)? > 3 {
        e.ww(0x4434b6, 3)?;
    }
    e.trap(0xa887, vec![W(e.w(0x402f44)? as u16)])?;
    e.ww(
        f.wrapping_sub(0x56),
        e.w(0x4434b6)?.wrapping_mul(102).wrapping_add(4),
    )?;
    e.ww(
        f.wrapping_sub(0x52),
        e.w(f.wrapping_sub(0x56))?.wrapping_add(100),
    )?;
    m.d4 = signed_div_word(
        i32::from(e.w(0x4471f0)?),
        e.w(0x439582)?.wrapping_mul(5).wrapping_add(63),
    )?;
    let maximum = e.w(0x439582)?.wrapping_mul(2).wrapping_add(4);
    if m.d4 > maximum {
        m.d4 = maximum;
    }
    let slot = e
        .w(0x439582)?
        .wrapping_mul(2)
        .wrapping_add(5)
        .wrapping_mul(e.w(0x4434b6)?.wrapping_add(e.w(0x41f70e)?))
        .wrapping_add(m.d4);
    if slot == e.w(f.wrapping_sub(0x7e))? {
        return Ok(());
    }
    e.ww(
        f.wrapping_sub(0x54),
        e.w(0x439582)?
            .wrapping_mul(5)
            .wrapping_add(63)
            .wrapping_mul(m.d4)
            .wrapping_add(4),
    )?;
    e.ww(
        f.wrapping_sub(0x50),
        e.w(0x439582)?
            .wrapping_mul(4)
            .wrapping_add(60)
            .wrapping_add(e.w(f.wrapping_sub(0x54))?),
    )?;
    e.trap(0xa89c, vec![W(2)])?;
    if count != 0 {
        e.trap(0xa8a1, vec![P(f.wrapping_sub(0x6a))])?;
    }
    e.lw(f.wrapping_sub(0x6a), e.l(f.wrapping_sub(0x56))?)?;
    e.lw(f.wrapping_sub(0x66), e.l(f.wrapping_sub(0x52))?)?;
    e.ww(f.wrapping_sub(0x7e), slot)?;
    if count != 0 {
        e.trap(0xa8a1, vec![P(f.wrapping_sub(0x56))])?;
    }
    if e.w(pack(slot))? == 0 {
        e.trap(0xaa1c, vec![L(e.l(0x43952c)?)])?;
        return Ok(());
    }
    if e.w(0x439582)? != 0 {
        e.quick_info(
            -1,
            slot,
            e.w(pack(slot))?,
            2,
            control(m.registers.a2, "A2")?,
            e.stack,
        )?;
    }
    e.load_item(e.w(pack(slot))?, e.stack)?;
    let weapon = [2, 15, 26].contains(&e.w(0x453656)?);
    e.pict(
        e.stack,
        if weapon { 1213 } else { 1212 },
        f.wrapping_sub(0x74),
    )?;
    e.ww(0x4434b6, if weapon { 1 } else { 2 })?;
    e.move_pen(
        e.w(0x40856c)?.wrapping_add(330),
        e.w(f.wrapping_sub(0x28))?.wrapping_sub(14),
    )?;
    let s = e.stack;
    e.lw(s.wrapping_sub(4), 0x479ed8)?;
    let id = e.w(0x4536a2)?;
    e.ww(s.wrapping_sub(6), id)?;
    let base = crate::rules::selection_base(id);
    e.ww(0x4471e8, base)?;
    e.d0 = Some(i32::from(base) as u32);
    e.ww(s.wrapping_sub(6), base.wrapping_add(1))?;
    e.ww(
        s.wrapping_sub(8),
        e.w(0x4536a2)?.wrapping_sub(e.w(0x4471e8)?).wrapping_add(1),
    )?;
    e.lib(
        0x400178,
        vec![
            P(0x479ed8),
            W(base.wrapping_add(1) as u16),
            W(e.w(s.wrapping_sub(8))? as u16),
        ],
    )?;
    e.stack = e.stack.wrapping_sub(8);
    e.trap(0xa884, vec![P(0x479ed8)])?;
    let mut sp = e.stack;
    if e.w(0x4434b6)? == 1 {
        e.move_pen(
            e.w(0x40856c)?.wrapping_add(396),
            e.w(f.wrapping_sub(0x28))?.wrapping_add(65),
        )?;
        let mut total = 0_i16;
        for a in [
            0x453664, 0x45367a, 0x4536c8, 0x45367c, 0x45367e, 0x453680, 0x453686, 0x453684,
            0x453682,
        ] {
            total = total.wrapping_add(e.w(a)?);
        }
        e.ww(0x4434b6, total)?;
        e.addw(0x4434b6, e.w(0x453710)?.wrapping_add(e.w(0x4536fa)?))?;
        if e.w(0x453652)? != 0 {
            e.ww(0x4434b6, e.w(0x4434b6)?.wrapping_mul(e.w(0x453652)?))?;
        }
        if e.b(pack(slot) + 18)? != 0 {
            e.lw(0x4427b6, i32::from(e.b(pack(slot) + 18)? as i8) as u32)?;
            e.lw(f.wrapping_sub(0xbc), 100_u32.wrapping_add(e.l(0x4427b6)?))?;
            e.sane(0x280e, Some(f.wrapping_sub(0xbc)), f.wrapping_sub(0xbc))?;
            e.sane(0x1010, Some(f.wrapping_sub(0xbc)), f.wrapping_sub(0xbc))?;
            e.lw(f.wrapping_sub(0x8a), e.l(f.wrapping_sub(0xbc))?)?;
            e.lw(f.wrapping_sub(0xb0), 0x42c80000)?;
            e.sane(0x100e, Some(f.wrapping_sub(0x8a)), f.wrapping_sub(0xbc))?;
            e.sane(0x1006, Some(f.wrapping_sub(0xb0)), f.wrapping_sub(0xbc))?;
            e.sane(0x1010, Some(f.wrapping_sub(0xbc)), f.wrapping_sub(0x8a))?;
            e.sane(0x200e, Some(0x4434b6), f.wrapping_sub(0xbc))?;
            e.sane(0x1004, Some(f.wrapping_sub(0x8a)), f.wrapping_sub(0xbc))?;
            e.sane(0x16, None, f.wrapping_sub(0xbc))?;
            e.sane(0x2810, Some(f.wrapping_sub(0xbc)), f.wrapping_sub(0xbc))?;
            e.ww(0x4434b6, e.w(f.wrapping_sub(0xba))?)?;
        }
        value_word(e, &mut sp, 0x1382d0, e.w(0x4434b6)?)?;
        e.move_pen(
            e.w(0x40856c)?.wrapping_add(474),
            e.w(f.wrapping_sub(0x28))?.wrapping_add(64),
        )?;
        value_word(
            e,
            &mut sp,
            0x13832a,
            i16::from(e.b(pack(slot) + 11)? as i8)
                .wrapping_add(i16::from(e.b(pack(slot) + 15)? as i8)),
        )?;
        e.move_pen(
            e.w(0x40856c)?.wrapping_add(522),
            e.w(f.wrapping_sub(0x28))?.wrapping_add(30),
        )?;
        value_word(e, &mut sp, 0x138368, e.w(0x453652)?)?;
        e.move_pen(
            e.w(0x40856c)?.wrapping_add(522),
            e.w(f.wrapping_sub(0x28))?.wrapping_add(64),
        )?;
        let sum = e.w(0x453654)?.wrapping_add(e.w(0x453650)?);
        sp = sp.wrapping_add(6);
        value_word(
            e,
            &mut sp,
            0x1382d0,
            if sum > 0 {
                e.w(0x453654)?.wrapping_add(e.w(0x453650)?)
            } else {
                0
            },
        )?;
        e.move_pen(
            e.w(0x40856c)?.wrapping_add(396),
            e.w(f.wrapping_sub(0x28))?.wrapping_add(47),
        )?;
        value_word(
            e,
            &mut sp,
            0x138368,
            i16::from(e.b(pack(slot) + 20)? as i8)
                .wrapping_add(e.w(0x453678)?)
                .wrapping_add(i16::from(e.b(0x453721)? as i8))
                .wrapping_add(e.w(0x45370e)?),
        )?;
        e.move_pen(
            e.w(0x40856c)?.wrapping_add(474),
            e.w(f.wrapping_sub(0x28))?.wrapping_add(47),
        )?;
        value_word(e, &mut sp, 0x138368, e.w(0x453688)?)?;
        e.move_pen(
            e.w(0x40856c)?.wrapping_add(474),
            e.w(f.wrapping_sub(0x28))?.wrapping_add(13),
        )?;
        value_word(e, &mut sp, 0x138368, i16::from(e.b(pack(slot) + 13)? as i8))?;
        e.move_pen(
            e.w(0x40856c)?.wrapping_add(474),
            e.w(f.wrapping_sub(0x28))?.wrapping_add(30),
        )?;
        let special = e.w(0x4536c4)?;
        sp = sp.wrapping_add(6);
        e.trap(
            0xa884,
            vec![P(if special != 0 { 0x403f4e } else { 0x403f4a })],
        )?;
        detail_rect(e, f, 10, 54, 568, 64)?;
        if e.w(0x453676)? != 0 {
            e.lw(0x439598, 0)?;
            let handle = e.trap(0xaa1e, vec![W(e.w(0x453676)?.wrapping_add(9999) as u16)])?;
            e.lw(0x439598, handle)?;
            draw_current_icon(e)?;
        }
    } else {
        e.move_pen(
            e.w(0x40856c)?.wrapping_add(432),
            e.w(f.wrapping_sub(0x28))?.wrapping_add(10),
        )?;
        value_word(e, &mut sp, 0x138368, i16::from(e.b(pack(slot) + 13)? as i8))?;
        e.move_pen(
            e.w(0x40856c)?.wrapping_add(432),
            e.w(f.wrapping_sub(0x28))?.wrapping_add(29),
        )?;
        value_word(
            e,
            &mut sp,
            0x138368,
            i16::from(e.b(pack(slot) + 19)? as i8)
                .wrapping_add(i16::from(e.b(0x453692)? as i8))
                .wrapping_add(e.w(0x45369a)?),
        )?;
        e.move_pen(
            e.w(0x40856c)?.wrapping_add(526),
            e.w(f.wrapping_sub(0x28))?.wrapping_add(10),
        )?;
        value_word(e, &mut sp, 0x1383a6, i16::from(e.b(0x453643)? as i8))?;
        e.move_pen(
            e.w(0x40856c)?.wrapping_add(603),
            e.w(f.wrapping_sub(0x28))?.wrapping_add(10),
        )?;
        value_word(
            e,
            &mut sp,
            0x138368,
            i16::from(e.b(pack(slot) + 14)? as i8).wrapping_add(e.w(0x453694)?),
        )?;
        e.move_pen(
            e.w(0x40856c)?.wrapping_add(526),
            e.w(f.wrapping_sub(0x28))?.wrapping_add(29),
        )?;
        value_word(e, &mut sp, 0x1383a6, i16::from(e.b(0x453642)? as i8))?;
        e.move_pen(
            e.w(0x40856c)?.wrapping_add(603),
            e.w(f.wrapping_sub(0x28))?.wrapping_add(29),
        )?;
        value_word(e, &mut sp, 0x138368, e.w(0x453696)?)?;
        e.move_pen(
            e.w(0x40856c)?.wrapping_add(500),
            e.w(f.wrapping_sub(0x28))?.wrapping_add(53),
        )?;
        let kind = e.w(0x453656)?;
        sp = sp.wrapping_add(12);
        if kind == 2 {
            value_word(e, &mut sp, 0x1382d0, i16::from(e.b(0x45368d)? as i8))?;
            sp = sp.wrapping_add(2);
        }
        e.move_pen(
            e.w(0x40856c)?.wrapping_add(461),
            e.w(f.wrapping_sub(0x28))?.wrapping_add(49),
        )?;
        value_word(e, &mut sp, 0x1383a6, i16::from(e.b(0x45368d)? as i8))?;
        e.move_pen(
            e.w(0x40856c)?.wrapping_add(603),
            e.w(f.wrapping_sub(0x28))?.wrapping_add(48),
        )?;
        value_word(e, &mut sp, 0x138368, e.w(0x453698)?)?;
        e.move_pen(
            e.w(0x40856c)?.wrapping_add(461),
            e.w(f.wrapping_sub(0x28))?.wrapping_add(68),
        )?;
        value_word(
            e,
            &mut sp,
            0x1383a6,
            i16::from(e.b(0x45368f)? as i8).wrapping_add(i16::from(e.b(0x45363f)? as i8)),
        )?;
        e.move_pen(
            e.w(0x40856c)?.wrapping_add(508),
            e.w(f.wrapping_sub(0x28))?.wrapping_add(68),
        )?;
        value_word(
            e,
            &mut sp,
            0x1383a6,
            i16::from(e.b(0x453690)? as i8).wrapping_add(i16::from(e.b(0x453640)? as i8)),
        )?;
        sp = sp.wrapping_add(8);
    }
    e.stack = sp;
    let usable = e.can_use(i16::from(e.b(0x44273f)? as i8), e.stack)?;
    e.trap(
        0xaa1c,
        vec![L(e.l(if usable { 0x43952c } else { 0x439534 })?)],
    )?;
    e.trap(0xa862, vec![L(69)])?;
    detail_rect(e, f, -2, 32, 333, 32)?;
    e.lw(0x439598, 0)?;
    let handle = e.trap(0xaa1e, vec![W(e.w(0x453646)? as u16)])?;
    e.lw(0x439598, handle)?;
    draw_current_icon(e)?;
    if e.w(0x45365e)? != 0 || e.w(0x45365c)? != 0 {
        detail_rect(e, f, 37, 16, 332, 16)?;
        let kind = e.w(0x45365e)?.wrapping_add(e.w(0x45365c)?);
        let id = match kind {
            8 | 9 => 2109,
            11 | 12 => 2110,
            14 => 2113,
            15 => 2101,
            16 | 18 => 2103,
            17 | 21 => 2102,
            19 => 2100,
            23 => 2108,
            25 => 2105,
            27 => 2106,
            29 => 2107,
            31 => 2112,
            32 => 2111,
            _ => 2104,
        };
        e.ww(0x4434b6, id)?;
        e.lw(0x439598, 0)?;
        let handle = e.trap(0xaa1e, vec![W(e.w(0x4434b6)? as u16)])?;
        e.lw(0x439598, handle)?;
        draw_current_icon(e)?;
    }
    Ok(())
}
fn detail_rect<S: LootControlServices>(
    e: &mut Execution<'_, S>,
    f: u32,
    top: i16,
    height: i16,
    left: i16,
    width: i16,
) -> Result<(), RuleError> {
    e.ww(0x4434ba, e.w(f.wrapping_sub(0x28))?.wrapping_add(top))?;
    e.ww(0x4434be, e.w(0x4434ba)?.wrapping_add(height))?;
    e.ww(0x4434bc, e.w(0x40856c)?.wrapping_add(left))?;
    e.ww(0x4434c0, e.w(0x4434bc)?.wrapping_add(width))?;
    Ok(())
}
fn draw_current_icon<S: LootControlServices>(e: &mut Execution<'_, S>) -> Result<(), RuleError> {
    if e.l(0x439598)? != 0 {
        e.trap(0xaa1f, vec![P(0x4434ba), L(e.l(0x439598)?)])?;
        e.trap(0xaa25, vec![L(e.l(0x439598)?)])?;
    }
    Ok(())
}

impl<S: LootControlServices> Execution<'_, S> {
    fn move_calc(&mut self, member: u16, stack: u32) -> Result<(), RuleError> {
        self.ww(stack.wrapping_sub(2), member as i16)?;
        let a = if member == u16::MAX {
            0x44721c
        } else if member == 0xfffe {
            0x447e5e
        } else {
            warrior(member as i16)
        };
        let race = self.w(a + 0x32)?.wrapping_sub(1);
        let race_address = index(0x435664, race, 1);
        let start = a.min(race_address);
        let end =
            a.checked_add(WARRIOR_BYTES as u32)
                .ok_or_else(|| RuleError::Invalid("loot movement record wraps".into()))?
                .max(race_address.checked_add(1).ok_or_else(|| {
                    RuleError::Invalid("loot movement race address wraps".into())
                })?);
        if end - start > 1024 * 1024 {
            return Err(RuleError::Invalid(
                "loot movement arena exceeds one MiB".into(),
            ));
        }
        let memory = InstallMemory {
            base_address: start,
            bytes: self.bytes(start, (end - start) as usize)?,
        };
        let child = navigation::movecalc(&memory, member)?;
        for write in &child.writes {
            self.write(write.address, &write.bytes)?;
        }
        self.d0 = Some(child.native_return as u32);
        self.plan.effects.push(LootControlEffect::Movement(child));
        Ok(())
    }
    fn weight(&mut self, member: u16, stack: u32) -> Result<(), RuleError> {
        self.ww(stack.wrapping_sub(2), member as i16)?;
        let base = warrior(member as i16);
        let bytes = self.bytes(base, WARRIOR_BYTES)?;
        if base < 0x4536d2 && base.wrapping_add(WARRIOR_BYTES as u32) > 0x45363c {
            return Err(RuleError::Unsupported {
                action: "loot.weight",
                dependency: "warrior aliases the captured item-work projection",
            });
        }
        let count = i16::from_be_bytes([bytes[26], bytes[27]]);
        if count > 100 {
            return Err(RuleError::Invalid(
                "loot captured weight count exceeds 100 records".into(),
            ));
        }
        let mut backing: [Vec<u8>; 5] = std::array::from_fn(|_| vec![]);
        for n in 0..count.max(0) as usize {
            let at = 0x132 + n * 22;
            let id = i16::from_be_bytes([bytes[at], bytes[at + 1]]);
            let abs = id.wrapping_abs();
            if let Ok(family) = usize::try_from(abs / 200)
                && family < 5
            {
                let source = records::ITEM_TABLE_ADDRESSES[family]
                    .wrapping_add((abs % 200) as u32 * ITEM_BYTES as u32);
                if source < base.wrapping_add(0x9d8)
                    && source.wrapping_add(ITEM_BYTES as u32) > base.wrapping_add(0x9d6)
                {
                    return Err(RuleError::Unsupported {
                        action: "loot.weight",
                        dependency: "weight aliases a captured item-definition record",
                    });
                }
            }
            let selected = equipment::item_tables(&self.plan.state.memory, id)?;
            for (target, source) in backing.iter_mut().zip(selected) {
                if source.len() > target.len() {
                    target.resize(source.len(), 0);
                }
                if !source.is_empty() {
                    let start = source.len() - ITEM_BYTES;
                    target[start..source.len()].copy_from_slice(&source[start..]);
                }
            }
        }
        let tables = ItemTables {
            families: std::array::from_fn(|i| backing[i].as_slice()),
        };
        let child = inventory::calcw(
            &inventory::WeightState {
                warrior: Warrior {
                    name: String::new(),
                    original_sha256: String::new(),
                    bytes,
                },
                item_work: self
                    .bytes(0x45363c, ITEM_BYTES)?
                    .try_into()
                    .expect("item bytes"),
            },
            &tables,
            member,
        )?;
        for effect in child.effects {
            match effect {
                InventoryEffect::Write(write) => self.write(write.address, &write.bytes)?,
                other => self.plan.effects.push(LootControlEffect::Inventory(other)),
            }
        }
        self.d0 = Some(child.native_return as u32);
        Ok(())
    }
    fn get_exp(&mut self, member: i16, level: i16, stack: u32) -> Result<(), RuleError> {
        self.ww(stack.wrapping_sub(2), 0)?;
        self.ww(stack.wrapping_sub(4), member)?;
        self.ww(stack.wrapping_sub(6), level)?;
        let mut warriors = vec![];
        if level > 0 {
            if !(0..6).contains(&member) {
                return Err(RuleError::Invalid(
                    "loot experience recipient outside captured party domain".into(),
                ));
            }
            for n in 0..=member {
                warriors.push(Warrior {
                    name: String::new(),
                    original_sha256: String::new(),
                    bytes: self.bytes(warrior(n), WARRIOR_BYTES)?,
                });
            }
        }
        let state = ExperienceState {
            warriors,
            global: ExperienceCounters {
                remaining: 0,
                total: 0,
            },
        };
        let child = economy::getexp(
            &state,
            ExperienceInput {
                level_count: level,
                recipient: member,
                update_total: 0,
            },
        )?;
        for write in &child.writes {
            let offset = match write.field {
                ExperienceField::Remaining => 0x9ce,
                ExperienceField::Total => 0x9d2,
            };
            self.lw(warrior(member) + offset, write.after as u32)?;
        }
        self.plan.effects.push(LootControlEffect::Experience(child));
        self.d0 = Some(0);
        Ok(())
    }
    fn level_up(&mut self, member: i16, stack: u32) -> Result<bool, RuleError> {
        self.ww(stack.wrapping_sub(2), 0)?;
        self.ww(stack.wrapping_sub(4), member)?;
        let before = self.plan.state.random.draws;
        let child = creation::level_up(
            &self.plan.state,
            stack.wrapping_sub(12),
            PromptLimits {
                maximum_steps: self.remaining,
                ..self.limits.prompt
            },
            self.services,
        )?;
        let returned = match child.native_return {
            PromptReturn::Full(v) => v as u16 != 0,
            PromptReturn::LowWord(v) => v != 0,
            _ => false,
        };
        self.draws = self
            .draws
            .checked_sub(child.state.random.draws.saturating_sub(before))
            .ok_or_else(|| RuleError::Invalid("loot random budget exhausted".into()))?;
        self.returned(child.native_return);
        self.plan.state = child.state;
        self.plan.flow = child.flow;
        if !self.active() {
            self.plan.native_return = PromptReturn::NoReturn;
        }
        self.plan
            .effects
            .extend(child.effects.into_iter().map(LootControlEffect::Creation));
        Ok(returned)
    }
    fn push_record(&mut self, from: u32, stack: u32) -> Result<u32, RuleError> {
        let mut source = from.wrapping_add(22);
        let mut target = stack;
        for _ in 0..5 {
            source = source.wrapping_sub(4);
            target = target.wrapping_sub(4);
            self.lw(target, self.l(source)?)?;
        }
        source = source.wrapping_sub(2);
        target = target.wrapping_sub(2);
        self.ww(target, self.w(source)?)?;
        Ok(target)
    }
    fn bank(&mut self, from: u32, crate_bank: bool, stack: u32) -> Result<bool, RuleError> {
        let target = self.push_record(from, stack)?;
        let record: [u8; INVENTORY_BYTES] = self
            .bytes(target, INVENTORY_BYTES)?
            .try_into()
            .expect("record bytes");
        let current = self.bytes(
            if crate_bank { 0x4285f4 } else { 0x41fc74 },
            inventory::BANK_BYTES,
        )?;
        let child = if crate_bank {
            inventory::addtocrate(&current, &record)?
        } else {
            inventory::addtobooty(&current, &record)?
        };
        for effect in child.effects {
            match effect {
                InventoryEffect::Write(write) => self.write(write.address, &write.bytes)?,
                InventoryEffect::Sound(id) => self.sound(id, stack.wrapping_sub(34))?,
                InventoryEffect::Warning(id) => self.warn(stack.wrapping_sub(34), id)?,
            }
        }
        self.d0 = Some(child.native_return as u32);
        Ok(child.native_return != 0)
    }
    fn install(
        &mut self,
        member: i16,
        source_slot: i16,
        destination_slot: i16,
        id: i16,
        stack: u32,
    ) -> Result<(), RuleError> {
        self.ww(stack.wrapping_sub(2), id)?;
        self.ww(stack.wrapping_sub(4), destination_slot)?;
        self.ww(stack.wrapping_sub(6), source_slot)?;
        self.ww(stack.wrapping_sub(8), member)?;
        self.ww(stack.wrapping_sub(10), -2)?;
        let start = 0x4046e6_u32;
        let end = 0x478288_u32.max(warrior(member).wrapping_add(WARRIOR_BYTES as u32));
        if end.wrapping_sub(start) > 1024 * 1024 {
            return Err(RuleError::Invalid(
                "loot installation arena exceeds one MiB".into(),
            ));
        }
        let memory = InstallMemory {
            base_address: start,
            bytes: self.bytes(start, (end - start) as usize)?,
        };
        let backing = equipment::item_tables(&self.plan.state.memory, id)?;
        let tables = ItemTables {
            families: std::array::from_fn(|i| backing[i].as_slice()),
        };
        let before = self.plan.state.random.draws;
        let child = install::installitem_complete(
            &memory,
            &self.plan.state.random,
            &tables,
            InstallRequest {
                source_member: 0xfffe,
                destination_member: member as u16,
                source_slot: source_slot as u16,
                destination_slot: destination_slot as u16,
                item_id: id,
            },
            RandomItemLimits {
                maximum_draws: self.draws,
            },
        )?;
        self.draws = self
            .draws
            .checked_sub(child.random.draws.saturating_sub(before))
            .ok_or_else(|| RuleError::Invalid("loot random budget exhausted".into()))?;
        self.plan.state.random = child.random;
        for effect in child.effects {
            if let RandomItemEffect::Write(write) = effect {
                self.write(write.address, &write.bytes)?;
            } else {
                self.plan
                    .effects
                    .push(LootControlEffect::RandomItem(effect));
            }
        }
        self.d0 = Some(child.native_return as u32);
        Ok(())
    }
}
fn finish<S: LootControlServices>(e: &mut Execution<'_, S>, f: u32) -> Result<(), RuleError> {
    e.ww(0x443314, 0)?;
    let mut n = 0_i16;
    while n <= i16::from(e.b(0x44273a)? as i8) {
        e.step()?;
        let marker = index(f.wrapping_sub(0x30), n, 1);
        e.bw(marker, 0)?;
        let a = warrior(n);
        if (e.l(a + 0x9ce)? as i32) < 0 && e.w(a + 0x50)? > 0 {
            let s = e.stack;
            e.get_exp(n, e.w(a + 0x3a)?, s)?;
            let promoted = e.level_up(n, s.wrapping_sub(6))?;
            if !e.active() {
                return Ok(());
            }
            if promoted && e.w(a + 0x5a)? != 0 {
                e.bw(marker, 1)?;
            }
        }
        n = n.wrapping_add(1);
    }
    n = 0;
    while n <= i16::from(e.b(0x44273a)? as i8) {
        e.step()?;
        let a = warrior(n);
        if e.b(index(f.wrapping_sub(0x30), n, 1))? != 0 && e.w(a + 0x36)? != 0 {
            e.copy(a, 0x44721c, 784, true)?;
            e.selection(0x15a9cc, e.stack.wrapping_sub(8))?;
            if !e.active() {
                return Ok(());
            }
            e.copy(0x44721c, a, 784, true)?;
            for k in 0..10_u32 {
                for offset in [0, 2, 4, 6] {
                    e.ww(
                        0x43788a + u32::from(n as u16) * 80 + k * 8 + offset,
                        e.w(a + 0xa76 + k * 8 + offset)?,
                    )?;
                }
            }
        }
        n = n.wrapping_add(1);
    }
    if e.w(f.wrapping_add(10))? != 0 {
        for n in 0..1000_u32 {
            e.copy(0x41fc32, 0x4285f4 + n * 22, 5, true)?;
        }
        e.ww(0x4434b6, 0)?;
        for n in 0..1000_i16 {
            if e.w(pack(n))? != 0 {
                let destination = e.w(0x4434b6)?;
                e.addw(0x4434b6, 1)?;
                e.copy(
                    pack(n),
                    0x4285f4 + u32::from(destination as u16) * 22,
                    5,
                    true,
                )?;
            }
        }
    }
    e.trap(0xa914, vec![L(e.l(0x438790)?)])?;
    e.lw(0x438790, 0)?;
    e.bw(0x442752, 0)?;
    e.lw(0x443392, 0)?;
    e.bw(0x4427b3, 1)?;
    mark_survivors(e)?;
    e.trap(0xa89b, vec![L(0x10001)])?;
    if e.w(0x43957e)? != 0 {
        let mut n = 0_i16;
        while n <= i16::from(e.b(0x44273a)? as i8) {
            e.step()?;
            e.refit(n, e.stack)?;
            n = n.wrapping_add(1);
        }
    }
    e.returned(PromptReturn::Full(0));
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Continuation {
    Poll,
    Grid,
    Refresh,
    Finished,
    RetryItem,
}
fn copy_bits<S: LootControlServices>(
    e: &mut Execution<'_, S>,
    source: u32,
    destination: u32,
    source_rect: u32,
    destination_rect: u32,
) -> Result<(), RuleError> {
    e.trap(
        0xa8ec,
        vec![
            P(source.wrapping_add(2)),
            P(destination.wrapping_add(2)),
            P(source_rect),
            P(destination_rect),
            W(0),
            L(0),
        ],
    )?;
    Ok(())
}
fn inspect_backdrop<S: LootControlServices>(
    e: &mut Execution<'_, S>,
    f: u32,
    offset: bool,
) -> Result<(), RuleError> {
    e.ww(f.wrapping_sub(0x18), 0)?;
    e.ww(
        f.wrapping_sub(0x16),
        if offset {
            e.w(0x40856c)?.wrapping_add(320)
        } else {
            320
        },
    )?;
    e.ww(
        f.wrapping_sub(0x12),
        if offset {
            e.w(0x40856c)?.wrapping_add(640)
        } else {
            640
        },
    )?;
    e.ww(f.wrapping_sub(0x14), 320)?;
    e.trap(0xa873, vec![L(e.l(0x438790)?)])?;
    e.trap(0xaa0b, vec![L(e.l(0x4358da)?)])?;
    e.trap(0xa862, vec![L(33)])?;
    e.trap(0xa863, vec![L(30)])?;
    copy_bits(
        e,
        e.l(0x438790)?,
        e.l(0x4386b4)?,
        f.wrapping_sub(0x18),
        0x4471b4,
    )
}
fn restore_backdrop<S: LootControlServices>(
    e: &mut Execution<'_, S>,
    f: u32,
) -> Result<(), RuleError> {
    e.trap(0xa873, vec![L(e.l(0x438790)?)])?;
    e.trap(0xaa0b, vec![L(e.l(0x4358da)?)])?;
    e.trap(0xa862, vec![L(33)])?;
    e.trap(0xa863, vec![L(30)])?;
    copy_bits(
        e,
        e.l(0x4386b4)?,
        e.l(0x438790)?,
        0x4471b4,
        f.wrapping_sub(0x18),
    )
}
fn pickup_sound<S: LootControlServices>(
    e: &mut Execution<'_, S>,
    stack: u32,
) -> Result<(), RuleError> {
    if e.w(0x453648)? != 0 {
        e.sound(e.w(0x453648)?, stack)?;
    } else if e.w(0x45365e)? != 0 {
        let die = e.rand(2, stack)?;
        e.sound(die.wrapping_add(403), stack)?;
    } else {
        e.sound(6002, stack)?;
    }
    Ok(())
}
fn collection<S: LootControlServices>(
    e: &mut Execution<'_, S>,
    _m: &mut Modal,
    f: u32,
    count: i16,
) -> Result<Continuation, RuleError> {
    e.step()?;
    if (e.b(0x44273f)? as i8) <= -1 {
        return Ok(Continuation::Poll);
    }
    if e.trap(0xa8ad, vec![L(e.l(0x4471ee)?), P(f.wrapping_sub(0x92))])? as u8 == 0 {
        return Ok(Continuation::Poll);
    }
    let column = signed_div_word(
        i32::from(e.w(0x4471f0)?),
        e.w(0x439582)?.wrapping_mul(5).wrapping_add(63),
    )?;
    e.ww(0x41f70a, column)?;
    let row = signed_div_word(i32::from(e.w(0x4471ee)?.wrapping_sub(10)), 100)?
        .wrapping_add(e.w(0x41f70e)?);
    e.ww(0x41f70c, row)?;
    if e.w(0x439582)?.wrapping_mul(2).wrapping_add(5) <= e.w(0x41f70a)? {
        return Ok(Continuation::Poll);
    }
    let slot = e
        .w(0x439582)?
        .wrapping_mul(2)
        .wrapping_add(5)
        .wrapping_mul(e.w(0x41f70c)?)
        .wrapping_add(e.w(0x41f70a)?);
    e.ww(f.wrapping_sub(0x32), slot)?;
    if e.w(pack(e.w(f.wrapping_sub(0x32))?))? == 0 {
        return Ok(Continuation::Poll);
    }
    e.trap(0xa873, vec![L(e.l(0x438790)?)])?;
    e.trap(0xaa0b, vec![L(e.l(0x4358da)?)])?;
    let s = e.stack;
    let id = e.w(pack(e.w(f.wrapping_sub(0x32))?))?;
    e.ww(s.wrapping_sub(2), id)?;
    let abs = e.lib(0x400038, vec![W(id as u16)])?.d0 as i16;
    e.ww(f.wrapping_sub(0x6c), abs)?;
    if e.w(f.wrapping_sub(0x6c))? == 0 {
        e.warn(e.stack, 43)?;
        return Ok(Continuation::Poll);
    }
    e.load_item(e.w(f.wrapping_sub(0x6c))?, e.stack)?;
    e.lw(e.stack.wrapping_sub(6), 0xc80190)?;
    e.ww(e.stack.wrapping_sub(8), e.w(f.wrapping_sub(0x6c))?)?;
    let in_range = crate::rules::inclusive_range(e.w(f.wrapping_sub(0x6c))?, 200, 400);
    e.d0 = Some(u32::from(in_range));
    if in_range && e.w(0x45365e)? != 0 {
        let quantity = e.w(pack(e.w(f.wrapping_sub(0x32))?) + 2)?;
        let destination = index(0x42dbe4, e.w(f.wrapping_sub(0x6c))?.wrapping_sub(200), 4);
        e.addl(destination, i32::from(quantity) as u32)?;
        pickup_sound(e, e.stack)?;
        return collection_clear(e, f, count);
    }
    if modifier(e, 0x200)? {
        e.copy(0x41fc32, pack(e.w(f.wrapping_sub(0x32))?), 5, true)?;
        e.sound(10129, e.stack)?;
        return collection_clear(e, f, count);
    }
    if modifier(e, 0x100)? {
        if e.w(f.wrapping_add(10))? != 0 {
            e.warn(e.stack, 151)?;
            return Ok(Continuation::Refresh);
        }
        if !e.bank(pack(e.w(f.wrapping_sub(0x32))?), true, e.stack)? {
            e.warn(e.stack, 153)?;
            return Ok(Continuation::Refresh);
        }
        return collection_clear(e, f, count);
    }
    if modifier(e, 0x800)? {
        inspect_backdrop(e, f, true)?;
        e.sound(30005, e.stack)?;
        e.trap(0xa89c, vec![W(2)])?;
        if count != 0 {
            e.trap(0xa8a1, vec![P(f.wrapping_sub(0x56))])?;
        }
        let id = e.w(f.wrapping_sub(0x6c))?;
        e.ww(e.stack.wrapping_sub(2), id)?;
        e.lw(e.stack.wrapping_sub(6), u32::MAX)?;
        let before = e.plan.state.random.draws;
        let child = interaction::show_item_info(
            &e.plan.state,
            e.stack.wrapping_sub(14),
            InventoryInteractionLimits {
                maximum_steps: e.remaining,
                prompt: e.limits.prompt,
                equipment: e.limits.equipment,
                maximum_random_draws: e.draws,
            },
            e.services,
        )?;
        let returned = match child.native_return {
            PromptReturn::Full(v) => v as u16 != 0,
            PromptReturn::LowWord(v) => v != 0,
            _ => false,
        };
        e.draws = e
            .draws
            .checked_sub(child.state.random.draws.saturating_sub(before))
            .ok_or_else(|| RuleError::Invalid("loot random budget exhausted".into()))?;
        e.returned(child.native_return);
        e.plan.state = child.state;
        e.plan.flow = child.flow;
        e.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(LootControlEffect::Interaction),
        );
        if !e.active() {
            e.plan.native_return = PromptReturn::NoReturn;
            return Ok(Continuation::Finished);
        }
        if returned {
            return Ok(Continuation::RetryItem);
        }
        e.trap(0xa8a7, vec![P(f.wrapping_sub(0x6a)), L(0), L(0)])?;
        e.trap(0xa8a7, vec![P(f.wrapping_sub(0x56)), L(0), L(0)])?;
        restore_backdrop(e, f)?;
        e.bw(f.wrapping_sub(0x57), 0)?;
        return Ok(Continuation::Grid);
    }
    e.load_item(e.w(f.wrapping_sub(0x6c))?, e.stack)?;
    if e.w(0x453660)? != 0 {
        let mut n = 0_i16;
        while n < e.w(warrior(i16::from(e.b(0x44273f)? as i8)) + 26)? {
            e.step()?;
            let a = warrior(i16::from(e.b(0x44273f)? as i8));
            if e.w(0x4536a2)? == e.w(a + 0x132 + u32::from(n as u16) * 22)? {
                let quantity = e.w(pack(e.w(f.wrapping_sub(0x32))?) + 2)?;
                let a = warrior(i16::from(e.b(0x44273f)? as i8));
                e.addw(a + 0x134 + u32::from(n as u16) * 22, quantity)?;
                let quantity = e.w(pack(e.w(f.wrapping_sub(0x32))?) + 2)?;
                let a = warrior(i16::from(e.b(0x44273f)? as i8));
                e.addw(a + 0x9d6, quantity)?;
                e.bw(f.wrapping_sub(0x61), 1)?;
                break;
            }
            n = n.wrapping_add(1);
        }
    }
    e.quiet(-1, e.stack)?;
    pickup_sound(e, e.stack)?;
    let member = i16::from(e.b(0x44273f)? as i8);
    let body = warrior(member);
    let fits = e.b(f.wrapping_sub(0x61))? == 0
        && e.w(body + 26)? < 99
        && encumbrance(e, body)? <= e.w(body + 0x9d8)? as u16;
    if fits {
        let id = e.w(0x4536a2)?;
        let member = i16::from(e.b(0x44273f)? as i8);
        let body = warrior(member);
        let destination = e.w(body + 26)?;
        e.addw(body + 26, 1)?;
        e.install(member, e.w(f.wrapping_sub(0x32))?, destination, id, e.stack)?;
        e.weight(e.b(0x44273f)? as i8 as i16 as u16, e.stack.wrapping_sub(10))?;
    } else {
        let body = warrior(i16::from(e.b(0x44273f)? as i8));
        if e.w(body + 26)? > 98 {
            e.warn(e.stack, 147)?;
            return Ok(Continuation::Refresh);
        }
        let body = warrior(i16::from(e.b(0x44273f)? as i8));
        if encumbrance(e, body)? > e.w(body + 0x9d8)? as u16 {
            e.warn(e.stack, 19)?;
            return Ok(Continuation::Refresh);
        }
    }
    e.bw(f.wrapping_sub(0x61), 0)?;
    e.move_calc(e.b(0x44273f)? as i8 as i16 as u16, e.stack)?;
    e.ww(e.stack.wrapping_sub(4), i16::from(e.b(0x44273f)? as i8))?;
    e.selection(0x15f4b8, e.stack.wrapping_sub(12))?;
    if !e.active() {
        return Ok(Continuation::Finished);
    }
    collection_clear(e, f, count)
}
fn encumbrance<S: LootControlServices>(e: &Execution<'_, S>, body: u32) -> Result<u16, RuleError> {
    Ok(e.w(0x45364c)?
        .wrapping_add(e.w(body + 0x9d6)?)
        .wrapping_add(e.w(0x453650)?.wrapping_mul(e.w(0x453660)?)) as u16)
}
fn modifier<S: LootControlServices>(
    e: &mut Execution<'_, S>,
    mask: u32,
) -> Result<bool, RuleError> {
    Ok(e.trap(0xa858, vec![L(u32::from(e.w(0x447218)? as u16)), L(mask)])? != 0)
}
fn collection_clear<S: LootControlServices>(
    e: &mut Execution<'_, S>,
    f: u32,
    count: i16,
) -> Result<Continuation, RuleError> {
    e.ww(pack(e.w(f.wrapping_sub(0x32))?), 0)?;
    e.pict(e.stack, 207, f.wrapping_sub(0x6a))?;
    e.trap(0xa862, vec![L(69)])?;
    e.trap(0xa89c, vec![W(2)])?;
    if count != 0 {
        e.trap(0xa8a1, vec![P(f.wrapping_sub(0x6a))])?;
    }
    if e.w(0x4046e6)? != 0 && e.w(0x4046e8)? != 0 {
        e.addw(0x4046e8, -1)?;
        if e.w(0x4046e8)? == 0 {
            finish(e, f)?;
            return Ok(Continuation::Finished);
        }
    }
    Ok(Continuation::Poll)
}

fn press_control<S: LootControlServices>(
    e: &mut Execution<'_, S>,
    handle: u32,
    kind: i16,
    phase: i16,
    sound: i16,
) -> Result<(), RuleError> {
    let rectangle = e.l(handle)?.wrapping_add(8);
    e.press(e.stack, kind, phase, sound, rectangle)
}
fn take_all<S: LootControlServices>(
    e: &mut Execution<'_, S>,
    m: &mut Modal,
    f: u32,
) -> Result<Continuation, RuleError> {
    let handle = control(m.registers.a2, "A2")?;
    press_control(e, handle, 1, 2, 141)?;
    let mut accepted = true;
    for n in 0..1000_i16 {
        e.step()?;
        if e.w(pack(n))? == 0 {
            continue;
        }
        e.load_item(e.w(pack(n))?, e.stack)?;
        if ![23, 25, 24].contains(&e.w(0x453656)?) {
            continue;
        }
        if e.w(0x4536a2)? <= 800 {
            continue;
        }
        inspect_backdrop(e, f, false)?;
        e.lw(e.stack.wrapping_sub(4), 0x100001)?;
        e.prompt(0x14cff6, e.stack.wrapping_sub(12))?;
        if !e.active() {
            return Ok(Continuation::Finished);
        }
        let result = match e.plan.native_return {
            PromptReturn::Full(v) => v as u16,
            PromptReturn::LowWord(v) => v,
            _ => {
                return Err(RuleError::Invalid(
                    "loot confirmation returned no WORD".into(),
                ));
            }
        };
        accepted = result != 0;
        break;
    }
    if !accepted {
        restore_backdrop(e, f)?;
        e.bw(f.wrapping_sub(0x57), 0)?;
        press_control(e, control(m.registers.a2, "A2")?, 1, 2, 0)?;
        return Ok(Continuation::Grid);
    }
    finish(e, f)?;
    Ok(Continuation::Finished)
}
fn drop_member_item<S: LootControlServices>(
    e: &mut Execution<'_, S>,
    member: i16,
    item: i16,
    slot: i16,
    stack: u32,
) -> Result<(), RuleError> {
    e.ww(stack.wrapping_sub(2), 1)?;
    e.ww(stack.wrapping_sub(4), slot)?;
    e.ww(stack.wrapping_sub(6), item)?;
    e.ww(stack.wrapping_sub(8), member)?;
    let before = e.plan.state.random.draws;
    let child = equipment::dropitem(
        &e.plan.state.memory,
        &e.plan.state.random,
        stack.wrapping_sub(16),
        e.limits.equipment,
        e.services,
    )?;
    e.draws = e
        .draws
        .checked_sub(child.random.draws.saturating_sub(before))
        .ok_or_else(|| RuleError::Invalid("loot random budget exhausted".into()))?;
    e.plan.state.memory = child.memory;
    e.plan.state.random = child.random;
    e.d0 = child.native_return.known_long();
    e.plan
        .effects
        .extend(child.effects.into_iter().map(LootControlEffect::Equipment));
    Ok(())
}
fn member_control<S: LootControlServices>(
    e: &mut Execution<'_, S>,
    m: &mut Modal,
    f: u32,
) -> Result<Continuation, RuleError> {
    let mut n = 0_i16;
    while n <= i16::from(e.b(0x44273a)? as i8) {
        e.step()?;
        let handle = e.l(index(f.wrapping_sub(0x4a), n, 4))?;
        if e.l(0x439404)? != handle {
            n = n.wrapping_add(1);
            continue;
        }
        let data = e.l(handle)?;
        e.lw(0x44292c, e.l(data.wrapping_add(8))?)?;
        e.lw(0x442930, e.l(data.wrapping_add(12))?)?;
        e.trap(0xa89c, vec![W(0)])?;
        let s = e.stack;
        e.button(s, true, true)?;
        e.ww(s.wrapping_sub(4), n)?;
        e.lw(s.wrapping_sub(8), 0x10001)?;
        e.ww(s.wrapping_sub(10), n)?;
        e.ww(s.wrapping_sub(12), n)?;
        e.selection(0x154a46, s.wrapping_sub(20))?;
        if !e.active() {
            return Ok(Continuation::Finished);
        }
        let selected = match e.plan.native_return {
            PromptReturn::Full(v) => v as u16,
            PromptReturn::LowWord(v) => v,
            _ => {
                return Err(RuleError::Invalid(
                    "loot condition selector returned no WORD".into(),
                ));
            }
        };
        e.ww(0x4434b6, selected as i16)?;
        e.button(s.wrapping_sub(12), false, true)?;
        let source = warrior(n) + 0x132 + u32::from(e.w(0x4434b6)? as u16) * 22;
        let mut a = source.wrapping_add(22);
        let mut p = s.wrapping_sub(14);
        for _ in 0..5 {
            a = a.wrapping_sub(4);
            p = p.wrapping_sub(4);
            e.lw(p, e.l(a)?)?;
        }
        a = a.wrapping_sub(2);
        p = p.wrapping_sub(2);
        e.ww(p, e.w(a)?)?;
        e.load_item(e.w(p)?, p.wrapping_add(2))?;
        if [23, 25, 24].contains(&e.w(0x453656)?) {
            e.warn(e.stack, 72)?;
            return Ok(Continuation::Refresh);
        }
        if e.w(0x4434b6)? > -1 {
            let source = warrior(n) + 0x132 + u32::from(e.w(0x4434b6)? as u16) * 22;
            let success = if e.w(f.wrapping_add(10))? != 0 {
                e.bank(source, false, e.stack)?
            } else {
                e.push_record(source, e.stack)?;
                e.load_item(e.w(source)?, e.stack.wrapping_sub(20))?;
                let source = warrior(n) + 0x132 + u32::from(e.w(0x4434b6)? as u16) * 22;
                e.bank(source, true, e.stack.wrapping_sub(22))?
            };
            if !success {
                e.warn(
                    e.stack,
                    if e.w(f.wrapping_add(10))? != 0 {
                        154
                    } else {
                        153
                    },
                )?;
                return Ok(Continuation::Refresh);
            }
            let slot = e.w(0x4434b6)?;
            let id = e.w(warrior(n) + 0x132 + u32::from(slot as u16) * 22)?;
            drop_member_item(e, n, id, slot, e.stack)?;
            e.ww(e.stack.wrapping_sub(10), n)?;
            e.selection(0x15f4b8, e.stack.wrapping_sub(18))?;
            if !e.active() {
                return Ok(Continuation::Finished);
            }
        }
        let mut row = 0_i16;
        while row < 4 {
            m.d4 = 0;
            while m.d4 < e.w(0x439582)?.wrapping_mul(2).wrapping_add(5) {
                e.step()?;
                display_cell(e, m, f, row, m.d4, None)?;
                m.d4 = m.d4.wrapping_add(1);
            }
            row = row.wrapping_add(1);
        }
        n = n.wrapping_add(1);
    }
    e.trap(0xa88a, vec![W(12)])?;
    e.trap(0xaa0b, vec![L(e.l(0x4358da)?)])?;
    e.trap(0xa887, vec![W(e.w(0x402f44)? as u16)])?;
    Ok(Continuation::Poll)
}
fn scroll_control<S: LootControlServices>(
    e: &mut Execution<'_, S>,
    m: &mut Modal,
    f: u32,
    count: i16,
    up: bool,
) -> Result<(), RuleError> {
    let handle = if up {
        control(m.registers.a4, "A4")?
    } else {
        e.l(f.wrapping_sub(0x86))?
    };
    press_control(e, handle, 2, 2, 0)?;
    e.ww(f.wrapping_sub(0x10), if up { 0 } else { 4 })?;
    e.ww(f.wrapping_sub(0xc), if up { 410 } else { 435 })?;
    e.trap(0xa863, vec![L(30)])?;
    loop {
        e.step()?;
        let permitted = if up {
            e.w(0x41f70e)? > 0
        } else {
            let limit = count.wrapping_sub(e.w(0x439582)?.wrapping_mul(8).wrapping_add(20));
            e.w(0x439582)?
                .wrapping_mul(2)
                .wrapping_add(5)
                .wrapping_mul(e.w(0x41f70e)?)
                < limit
        };
        if !permitted {
            break;
        }
        if e.trap(0xa973, vec![])? as u8 == 0 {
            break;
        }
        e.quiet(-1, e.stack)?;
        e.sound(215, e.stack.wrapping_sub(2))?;
        e.addw(0x41f70e, if up { -1 } else { 1 })?;
        for _ in 0..102 {
            e.step()?;
            e.trap(
                0xa8ef,
                vec![
                    P(f.wrapping_sub(0x10)),
                    L(if up { 0x10000 } else { 0xffff0000 }),
                    L(0),
                ],
            )?;
        }
        side_numbers(e, f, count, 0)?;
        let mut column = 0_i16;
        while column < e.w(0x439582)?.wrapping_mul(2).wrapping_add(5) {
            e.step()?;
            display_cell(e, m, f, 0, column, Some(up))?;
            column = column.wrapping_add(1);
        }
    }
    let handle = if up {
        control(m.registers.a4, "A4")?
    } else {
        e.l(f.wrapping_sub(0x86))?
    };
    press_control(e, handle, 2, 1, 0)?;
    Ok(())
}
fn select_portrait<S: LootControlServices>(
    e: &mut Execution<'_, S>,
    f: u32,
) -> Result<(), RuleError> {
    let member = signed_div_word(i32::from(e.w(0x4471ee)?), 50)?;
    e.bw(0x44273f, member as u8)?;
    if (e.b(0x44273f)? as i8) > (e.b(0x44273a)? as i8) {
        return Ok(());
    }
    e.press(e.stack, 1, 1, 141, f.wrapping_sub(0x60))?;
    e.ww(
        f.wrapping_sub(0x60),
        i16::from(e.b(0x44273f)? as i8).wrapping_mul(50),
    )?;
    e.ww(f.wrapping_sub(0x5e), e.w(0x40856c)?.wrapping_add(330))?;
    e.ww(
        f.wrapping_sub(0x5a),
        e.w(f.wrapping_sub(0x5e))?.wrapping_add(50),
    )?;
    e.ww(
        f.wrapping_sub(0x5c),
        e.w(f.wrapping_sub(0x60))?.wrapping_add(50),
    )?;
    e.trap(0xa8f6, vec![L(e.l(0x439544)?), P(f.wrapping_sub(0x7c))])?;
    e.ww(f.wrapping_sub(0x7c), e.w(f.wrapping_sub(0x60))?)?;
    e.ww(f.wrapping_sub(0x78), e.w(f.wrapping_sub(0x5c))?)?;
    e.trap(0xa8f6, vec![L(e.l(0x43955c)?), P(f.wrapping_sub(0x7c))])?;
    e.press(e.stack.wrapping_sub(14), 1, 2, 0, f.wrapping_sub(0x60))?;
    Ok(())
}

fn mouse_control<S: LootControlServices>(
    e: &mut Execution<'_, S>,
    m: &mut Modal,
    f: u32,
    count: i16,
) -> Result<Continuation, RuleError> {
    e.lw(0x439404, 0)?;
    let part = e.trap(0xa92c, vec![L(e.l(0x447214)?), P(f.wrapping_sub(0xa2))])? as i16;
    e.ww(0x4471a6, part)?;
    match e.w(0x4471a6)? {
        1 => {
            e.heap()?;
            let selected = e.trap(0xa93d, vec![L(e.l(0x447214)?)])?;
            e.lw(0x438742, selected)?;
            e.trap(0xa938, vec![W(0)])?;
            Ok(Continuation::Poll)
        }
        3 => {
            let part = e.trap(
                0xa96c,
                vec![L(e.l(0x4471ee)?), L(e.l(0x438790)?), P(0x439404)],
            )? as i16;
            e.ww(0x4471a6, part)?;
            if e.l(0x439404)? == 0 {
                return collection(e, m, f, count);
            }
            if e.l(0x439404)? == control(m.registers.a2, "A2")? {
                return take_all(e, m, f);
            }
            let result = member_control(e, m, f)?;
            if result != Continuation::Poll || !e.active() {
                return Ok(result);
            }
            if e.l(0x439404)? == control(m.registers.a4, "A4")? {
                scroll_control(e, m, f, count, true)?;
            }
            if e.l(0x439404)? == e.l(f.wrapping_sub(0x86))?
                && count > e.w(0x439582)?.wrapping_mul(8).wrapping_add(20)
            {
                scroll_control(e, m, f, count, false)?;
            }
            if e.l(0x439404)? == e.l(f.wrapping_sub(0x9e))? {
                select_portrait(e, f)?;
            }
            Ok(Continuation::Poll)
        }
        _ => Ok(Continuation::Poll),
    }
}
fn refresh<S: LootControlServices>(
    e: &mut Execution<'_, S>,
    f: u32,
    count: i16,
    money: u32,
) -> Result<(), RuleError> {
    if e.w(f.wrapping_add(10))? != 0 {
        side_numbers(e, f, count, 1)?;
    }
    e.trap(0xa89b, vec![L(0x20002)])?;
    if count < 1 && e.w(f.wrapping_add(10))? == 0 {
        e.trap(0xa8a7, vec![P(0x4434ba), L(0xc8005a), L(0x11c019a)])?;
        e.pict(e.stack, 182, 0x4434ba)?;
        if e.l(0x443392)? != 0 {
            e.move_pen(100, 220)?;
            e.trap(0xa884, vec![P(0x403f9a)])?;
            e.long_number(e.l(0x443392)?, e.stack)?;
            e.trap(0xa884, vec![P(0x403f88)])?;
            e.move_pen(100, 232)?;
            e.trap(0xa884, vec![P(0x403f74)])?;
            e.long_number(e.l(f.wrapping_sub(0x96))?, e.stack.wrapping_sub(4))?;
            e.trap(0xa884, vec![P(0x403f6e)])?;
        }
        if money != 0 {
            e.move_pen(100, 270)?;
            e.trap(0xa884, vec![P(0x403f5c)])?;
            e.long_number(money, e.stack)?;
            e.trap(0xa884, vec![P(0x403f52)])?;
        }
    }
    e.lw(0x443392, 0)?;
    e.lw(
        e.stack.wrapping_sub(4),
        if e.w(f.wrapping_add(10))? != 0 {
            0x150018
        } else {
            0x1d0021
        },
    )?;
    e.prompt(0x12382a, e.stack.wrapping_sub(12))?;
    Ok(())
}
fn update_event<S: LootControlServices>(
    e: &mut Execution<'_, S>,
    f: u32,
) -> Result<Continuation, RuleError> {
    if (e.l(0x44720c)? >> 24) != 1 {
        return Ok(Continuation::Poll);
    }
    if e.b(0x4427b2)? == 0 {
        e.bw(0x4427b2, 1)?;
        return Ok(Continuation::Poll);
    }
    e.bw(f.wrapping_sub(0x57), 0)?;
    e.bw(0x4427b2, 0)?;
    e.flush_events()?;
    e.row(0, 4, e.stack)?;
    e.ww(f.wrapping_sub(8), 0)?;
    e.ww(f.wrapping_sub(4), 50)?;
    e.ww(f.wrapping_sub(6), e.w(0x40856c)?.wrapping_add(380))?;
    e.ww(f.wrapping_sub(2), e.w(0x40856c)?.wrapping_add(640))?;
    e.lw(f.wrapping_sub(0x20), e.l(f.wrapping_sub(8))?)?;
    e.lw(f.wrapping_sub(0x1c), e.l(f.wrapping_sub(4))?)?;
    e.trap(0xa873, vec![L(e.l(0x438790)?)])?;
    e.trap(0xaa0b, vec![L(e.l(0x4358da)?)])?;
    e.trap(0xa862, vec![L(33)])?;
    e.trap(0xa863, vec![L(30)])?;
    let mut n = 1_i16;
    while n <= i16::from(e.b(0x44273a)? as i8) {
        e.step()?;
        e.addw(f.wrapping_sub(0x20), 50)?;
        e.addw(f.wrapping_sub(0x1c), 50)?;
        let source = e.l(0x438790)?;
        let dest = e.l(0x438790)?;
        copy_bits(e, source, dest, f.wrapping_sub(8), f.wrapping_sub(0x20))?;
        n = n.wrapping_add(1);
    }
    e.lw(f.wrapping_sub(0x20), e.l(f.wrapping_sub(8))?)?;
    e.lw(f.wrapping_sub(0x1c), e.l(f.wrapping_sub(4))?)?;
    e.trap(0xa862, vec![L(69)])?;
    portrait_rows(e, f)?;
    Ok(Continuation::Grid)
}
/// booty, 0x105748. Caller arguments remain addressed WORDs at A6+8 and A6+10.
pub fn booty(
    state: &LootControlState,
    frame: u32,
    registers: LootRegisters,
    limits: LootControlLimits,
    services: &mut impl LootControlServices,
) -> Result<LootControlPlan, RuleError> {
    let mut e = execution(state, frame, limits, services);
    let Some(prepared) = prepare(&mut e, frame)? else {
        return Ok(e.plan);
    };
    let mut modal = Modal { registers, d4: 0 };
    open_window(&mut e, frame)?;
    let mut next = Continuation::Grid;
    let mut polls = limits.prompt.maximum_polls;
    loop {
        e.step()?;
        match next {
            Continuation::Finished => return Ok(e.plan),
            Continuation::Grid => {
                e.trap(0xa873, vec![L(e.l(0x438790)?)])?;
                e.trap(0xaa0b, vec![L(e.l(0x4358da)?)])?;
                e.ww(0x4434bc, 0)?;
                e.ww(0x4434ba, 0)?;
                e.ww(0x4434c0, e.w(0x40856c)?.wrapping_add(320))?;
                e.ww(0x4434be, 460)?;
                e.pict(e.stack, 207, 0x4434ba)?;
                grid(&mut e, &mut modal, frame)?;
                if !e.active() {
                    return Ok(e.plan);
                }
                footer(&mut e, &mut modal, frame, prepared.count, prepared.money)?;
                if !e.active() {
                    return Ok(e.plan);
                }
                refresh(&mut e, frame, prepared.count, prepared.money)?;
                if !e.active() {
                    return Ok(e.plan);
                }
            }
            Continuation::Refresh => {
                refresh(&mut e, frame, prepared.count, prepared.money)?;
                if !e.active() {
                    return Ok(e.plan);
                }
            }
            Continuation::RetryItem => {
                next = collection(&mut e, &mut modal, frame, prepared.count)?;
                continue;
            }
            Continuation::Poll => {}
        }
        polls = polls
            .checked_sub(1)
            .ok_or_else(|| RuleError::Invalid("loot modal poll budget exhausted".into()))?;
        if modifier(&mut e, 0x400)? {
            e.warn(e.stack, 21)?;
        }
        e.trap(0xa9b4, vec![])?;
        e.ww(e.stack.wrapping_sub(2), 0)?;
        e.prompt(0x133d46, e.stack.wrapping_sub(10))?;
        if !e.active() {
            return Ok(e.plan);
        }
        let occurred = e.trap(0xa970, vec![W(u16::MAX), P(0x44720a)])? as u8;
        e.ww(frame.wrapping_sub(0xa4), i16::from(occurred))?;
        e.trap(0xa972, vec![P(0x4471ee)])?;
        hover(&mut e, &mut modal, frame, prepared.count, prepared.money)?;
        if !e.active() {
            return Ok(e.plan);
        }
        next = match e.w(0x44720a)? {
            1 => mouse_control(&mut e, &mut modal, frame, prepared.count)?,
            6 => {
                e.trap(0xa922, vec![L(e.l(0x438790)?)])?;
                e.trap(0xa923, vec![L(e.l(0x438790)?)])?;
                if e.l(0x438794)? != 0 {
                    e.trap(0xa922, vec![L(e.l(0x438794)?)])?;
                    e.trap(0xa923, vec![L(e.l(0x438794)?)])?;
                }
                Continuation::Poll
            }
            15 => update_event(&mut e, frame)?,
            _ => Continuation::Poll,
        };
    }
}

impl<S: LootControlServices> Execution<'_, S> {
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
            .extend(child.effects.into_iter().map(LootControlEffect::Audio));
        self.d0 = Some(0);
        Ok(())
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
        self.plan
            .effects
            .extend(child.effects.into_iter().map(LootControlEffect::Feedback));
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
        self.plan
            .effects
            .extend(child.effects.into_iter().map(LootControlEffect::Feedback));
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
        self.plan
            .effects
            .extend(child.effects.into_iter().map(LootControlEffect::Interface));
        self.d0 = Some(0);
        Ok(())
    }
    fn load_item(&mut self, id: i16, stack: u32) -> Result<(), RuleError> {
        self.ww(stack.wrapping_sub(2), id)?;
        let backing = equipment::item_tables(&self.plan.state.memory, id)?;
        let tables = ItemTables {
            families: std::array::from_fn(|i| backing[i].as_slice()),
        };
        let current = self
            .bytes(0x45363c, ITEM_BYTES)?
            .try_into()
            .expect("item bytes");
        let child = records::loaditem(&tables, &current, id)?;
        for write in child.writes {
            self.write(write.address, &write.bytes)?;
        }
        self.full(child.native_return as u32);
        Ok(())
    }
    fn profile_at(&mut self, frame: u32, race: i16, caste: i16) -> Result<(), RuleError> {
        self.word(frame.wrapping_add(10), caste as u16)?;
        self.word(frame.wrapping_add(8), race as u16)?;
        if race == 0 && caste == 0 {
            self.unknown();
            return Ok(());
        }
        let observations = if race == 0 && caste == 0 {
            vec![]
        } else {
            equipment::EquipmentServices::profile_observations(
                self.services,
                race,
                caste,
                &self.plan.state.memory,
            )?
        };
        let mut context = FileContext {
            scenario_directory: self.c_string(0x44260c)?,
            variant: self.b(0x44274c)?,
            filename: self.bytes(0x4389a4, self.limits.prompt.filename_capacity)?,
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
                    let base: u32 = match destination {
                        FileBuffer::Filename => 0x4389a4,
                        FileBuffer::RaceProfile => 0x41f85a,
                        FileBuffer::CasteProfile => 0x41f9f2,
                        _ => {
                            return Err(RuleError::Invalid(
                                "unexpected selection profile buffer".into(),
                            ));
                        }
                    };
                    self.write(base.wrapping_add(*offset as u32), bytes)?;
                }
                FileEffect::FileHandle(handle) => self.long(0x4434a6, *handle)?,
                _ => self.plan.effects.push(LootControlEffect::File(effect)),
            }
        }
        self.unknown();
        Ok(())
    }
}
fn index(a: u32, n: i16, stride: i32) -> u32 {
    a.wrapping_add((i32::from(n) * stride) as u32)
}
fn pascal(memory: &RenderingMemory, a: u32) -> Result<PascalString, RuleError> {
    PascalString::from_raw(&read(memory, a, usize::from(memory.byte(a)?) + 1)?)
}
struct FixedNumber(PascalString);
impl NumericServices for FixedNumber {
    fn number_to_string(&mut self, _: i32) -> Result<PascalString, RuleError> {
        Ok(self.0.clone())
    }
    fn string_to_number(&mut self, _: &PascalString) -> Result<i32, RuleError> {
        Err(RuleError::Invalid("unused loot numeric parse".into()))
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
        Err(RuleError::Invalid("unused loot indexed string".into()))
    }
    fn current_world(&mut self) -> Result<(u32, u32), RuleError> {
        Err(RuleError::Invalid("unused loot world".into()))
    }
    fn new_world(&mut self, _: i16, _: BodyRect) -> Result<(i16, BuildingWorld), RuleError> {
        Err(RuleError::Invalid("unused loot allocation".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recovered_shop_ui::NpcNameServices;
    use crate::recovered_spell_ui::{DialogItemObservation, SpellUiServices};
    use crate::recovered_timing::TimingServices;
    use interaction::tests::Observed;

    const FRAME: u32 = 0x10000;
    fn unused() -> RuleError {
        RuleError::Invalid("unexpected loot fixture observation".into())
    }
    impl TimingServices for Observed {
        fn tick_count(&mut self) -> Result<u32, RuleError> {
            Err(unused())
        }
        fn button(&mut self) -> Result<bool, RuleError> {
            Err(unused())
        }
    }
    impl NpcNameServices for Observed {
        fn copy_name(&self, _: u32, _: &[u8]) -> Result<PascalString, RuleError> {
            Err(unused())
        }
    }
    impl SpellUiServices for Observed {
        fn dialog_item(&mut self, _: u32, _: i16) -> Result<DialogItemObservation, RuleError> {
            Err(unused())
        }
        fn string_width(&mut self, _: &PascalString) -> Result<i16, RuleError> {
            Err(unused())
        }
        fn color_cursor(&mut self, _: i16) -> Result<u32, RuleError> {
            Err(unused())
        }
        fn square_root_extended(&mut self, _: [u8; 10]) -> Result<[u8; 10], RuleError> {
            Err(unused())
        }
        fn truncate_extended(&mut self, _: [u8; 10]) -> Result<i32, RuleError> {
            Err(unused())
        }
    }
    impl SelectionControlServices for Observed {}
    impl LootControlServices for Observed {}
    impl crate::recovered_inventory_interaction::ItemsServices for Observed {}
    impl crate::recovered_native_services::NativeServices for Observed {
        fn observe_native(
            &mut self,
            request: &crate::recovered_native_services::NativeRequest,
            memory: &RenderingMemory,
        ) -> Result<crate::recovered_native_services::NativeObservation, RuleError> {
            use crate::recovered_native_services::{NativeCall, NativeObservation};
            let cleanup = match request.call {
                NativeCall::Toolbox(0xa93c) => 4,
                NativeCall::Toolbox(0xa935) => 6,
                NativeCall::Toolbox(0xa937 | 0xa032) => 0,
                _ => return Err(unused()),
            };
            Ok(NativeObservation {
                memory: memory.clone(),
                data: [None; 8],
                address: [None; 8],
                ccr: None,
                stack_after: request.stack.wrapping_add(cleanup),
                flow: LoadFlow::Returned,
            })
        }
    }
    #[test]
    fn application_event_loop_enforces_budget_through_native_menu_setup() {
        use crate::recovered_application_control::{
            ApplicationControlLimits, ApplicationRegisters,
        };
        let mut state = PromptControlState {
            memory: RenderingMemory::default(),
            random: crate::RandomState::new(12),
        };
        state.memory.insert_region(FRAME - 64, &[0xaa; 80]);
        for (at, value) in [(0x4427ba, 11_u32), (0x43871a, 22), (0x438716, 33)] {
            state.memory.insert_region(at, &value.to_be_bytes());
        }
        let before = state.clone();
        let base = limits();
        let limit = ApplicationControlLimits {
            maximum_steps: 15,
            prompt: base.prompt,
            equipment: base.equipment,
            maximum_random_draws: 0,
            maximum_call_depth: 8,
        };
        let error = crate::recovered_system_control::main_loop(
            &state,
            FRAME,
            ApplicationRegisters::default(),
            limit,
            &mut Observed::new(),
        )
        .unwrap_err();
        assert!(
            matches!(error, RuleError::Invalid(message) if message.contains("processing budget"))
        );
        assert_eq!(state, before);
    }

    impl crate::recovered_startup::StartupServices for Observed {
        fn observe_startup(
            &self,
            _: &crate::recovered_startup::StartupRequest,
            _: &RenderingMemory,
        ) -> Result<crate::recovered_startup::StartupObservation, RuleError> {
            Err(unused())
        }
        fn preference_inputs(
            &self,
            _: &RenderingMemory,
        ) -> Result<crate::recovered_startup::PreferenceInputs, RuleError> {
            Err(unused())
        }
    }
    impl crate::recovered_combat_results::CombatResultServices for Observed {
        fn graphics_dispatch(
            &mut self,
            _: &crate::recovered_combat_results::GraphicsDispatchRequest,
            _: &RenderingMemory,
        ) -> Result<FeedbackObservation, RuleError> {
            Err(unused())
        }
    }
    #[test]
    fn application_items_transaction_keeps_symbolic_entry_and_failures_atomic() {
        use crate::recovered_application_control::ApplicationControlLimits;
        use crate::recovered_inventory_interaction::{self as inventory, ItemsRegisters};
        let mut state = PromptControlState {
            memory: RenderingMemory::default(),
            random: crate::RandomState::new(12),
        };
        state.memory.insert_region(FRAME - 256, &[0xaa; 272]);
        let before = state.clone();
        let base = limits();
        let mut limits = ApplicationControlLimits {
            maximum_steps: 4,
            prompt: base.prompt,
            equipment: base.equipment,
            maximum_random_draws: 0,
            maximum_call_depth: 8,
        };
        let error = inventory::items(
            &state,
            FRAME,
            &ItemsRegisters::default(),
            limits,
            &mut Observed::new(),
        )
        .unwrap_err();
        assert!(
            matches!(error, RuleError::Invalid(message) if message.contains("processing budget"))
        );
        assert_eq!(state, before);
        limits.maximum_steps = 100;
        assert!(
            inventory::items(
                &state,
                FRAME,
                &ItemsRegisters::default(),
                limits,
                &mut Observed::new()
            )
            .is_err()
        );
        assert_eq!(state, before);
    }
    #[test]
    fn application_queue_transaction_commits_only_after_complete_bounded_traversal() {
        use crate::recovered_application_control::{
            self as application, ApplicationControlLimits, ApplicationRegisters,
        };
        let mut state = PromptControlState {
            memory: RenderingMemory::default(),
            random: crate::RandomState::new(12),
        };
        state.memory.insert_region(FRAME - 256, &[0; 272]);
        state.memory.insert_region(0x437a94, &[0x55; 28]);
        state.memory.insert_region(0x438584, &[0]);
        word(&mut state, 0x43800c, 2);
        word(&mut state, 0x437a90, 9);
        for n in 0..2 {
            word(&mut state, 0x437a98 + n * 14, 0);
        }
        let base = limits();
        let limits = ApplicationControlLimits {
            maximum_steps: 100,
            prompt: base.prompt,
            equipment: base.equipment,
            maximum_random_draws: 0,
            maximum_call_depth: 8,
        };
        let before = state.clone();
        let result = application::execute(
            &state,
            0x121524,
            FRAME,
            ApplicationRegisters::default(),
            limits,
            &mut Observed::new(),
        )
        .unwrap();
        assert_eq!(state, before);
        assert_eq!(result.state.memory.word(0x43800c).unwrap(), 0);
        assert_eq!(result.state.memory.word(0x437a90).unwrap(), 0);
        assert_eq!(result.state.memory.byte(0x438584).unwrap(), 0);
        assert_eq!(result.state.random, before.random);
        assert_eq!(result.native_return, PromptReturn::Full(0));
        let mut exhausted = limits;
        exhausted.maximum_steps = 1;
        assert!(
            application::execute(
                &state,
                0x121524,
                FRAME,
                ApplicationRegisters::default(),
                exhausted,
                &mut Observed::new()
            )
            .is_err()
        );
        assert_eq!(state, before);
        // The second active row reaches a child after the first row was processed.
        word(&mut state, 0x437aa6, 1);
        let before = state.clone();
        assert!(
            application::execute(
                &state,
                0x121524,
                FRAME,
                ApplicationRegisters::default(),
                limits,
                &mut Observed::new()
            )
            .is_err()
        );
        assert_eq!(state, before);
    }
    #[test]
    fn native_pascal_children_return_status_and_fail_atomically_after_a_child_write() {
        use crate::recovered_application_control::{
            self as application, ApplicationControlLimits, ApplicationRegisters,
        };
        let mut state = PromptControlState {
            memory: RenderingMemory::default(),
            random: crate::RandomState::new(12),
        };
        state.memory.insert_region(FRAME - 256, &[0; 512]);
        state.memory.insert_region(FRAME + 8, &[1]);
        // A null monitor list avoids any imported device or disposal service.
        state.memory.insert_region(FRAME + 10, &[0; 4]);
        let base = limits();
        let limits = ApplicationControlLimits {
            maximum_steps: 100,
            prompt: base.prompt,
            equipment: base.equipment,
            maximum_random_draws: 0,
            maximum_call_depth: 8,
        };
        let before = state.clone();
        // 124242 calls actual 124576 and 1245cc bodies. The latter writes -109
        // into its Pascal return slot and pops four argument bytes with RTD.
        let result = application::execute(
            &state,
            0x124242,
            FRAME,
            ApplicationRegisters::default(),
            limits,
            &mut Observed::new(),
        )
        .unwrap();
        assert_eq!(result.native_return, PromptReturn::Full(0xff93));
        assert_eq!(result.state.memory.word(FRAME - 2).unwrap(), 0xff93);
        assert_eq!(result.native_registers.address[7].value, FRAME + 14);
        assert_eq!(result.state.random, before.random);
        assert_eq!(state, before);
        let mut exhausted = limits;
        exhausted.maximum_steps = 20;
        assert!(
            application::execute(
                &state,
                0x124242,
                FRAME,
                ApplicationRegisters::default(),
                exhausted,
                &mut Observed::new(),
            )
            .is_err()
        );
        assert_eq!(state, before);
    }
    fn limits() -> LootControlLimits {
        let base = interaction::tests::limits();
        LootControlLimits {
            maximum_steps: 10000,
            prompt: base.prompt,
            equipment: base.equipment,
            maximum_random_draws: 10,
        }
    }
    fn word(state: &mut LootControlState, a: u32, value: u16) {
        state.memory.insert_region(a, &value.to_be_bytes());
    }
    #[test]
    fn abort_prepares_all_loot_slots_and_returns_without_opening_modal() {
        let mut state = interaction::tests::fixture();
        state
            .memory
            .insert_region(0x41fc32, &[0x55; INVENTORY_BYTES]);
        state.memory.insert_region(0x41fc74, &[0x99; 22000]);
        state.memory.insert_region(0x42df04, &[0xff; 1000]);
        state.memory.insert_region(0x442bd6, &[0x77; 1200]);
        state.memory.insert_region(0x44273a, &[0xff]);
        word(&mut state, FRAME + 8, 2);
        word(&mut state, 0x408578, 1);
        word(&mut state, 0x44257c, 0);
        for n in 0..100_i16 {
            word(&mut state, npc(n) + 0x5a, 0xffff);
        }
        let before = state.clone();
        let result = booty(
            &state,
            FRAME,
            LootRegisters::default(),
            limits(),
            &mut Observed::new(),
        )
        .unwrap();
        assert_eq!(state, before);
        assert_eq!(result.native_return, PromptReturn::Full(0));
        assert_eq!(result.flow, LoadFlow::Returned);
        assert_eq!(result.state.random, state.random);
        for n in 0..1000_i16 {
            assert_eq!(result.state.memory.word(pack(n)).unwrap(), 0);
            assert_eq!(result.state.memory.byte(pack(n) + 2).unwrap(), 0x55);
            assert_eq!(result.state.memory.byte(0x42df04 + n as u32).unwrap(), 0);
        }
        assert!(!result.effects.iter().any(|effect| matches!(
            effect,
            LootControlEffect::Feedback(FeedbackEffect::Service {
                request: FeedbackRequest::Toolbox { opcode: 0xaa46, .. },
                ..
            })
        )));
        state.memory.bytes.remove(&0x41fc32);
        let missing = state.clone();
        assert!(
            booty(
                &state,
                FRAME,
                LootRegisters::default(),
                limits(),
                &mut Observed::new()
            )
            .is_err()
        );
        assert_eq!(state, missing);
    }
}
