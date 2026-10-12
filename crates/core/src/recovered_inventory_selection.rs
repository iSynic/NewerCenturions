//! Inventory selection, squad ordering and status control from CODE 1.

use crate::recovered_attack::DamageFrame;
use crate::recovered_audio::{self as audio, AudioContext, AudioEffect};
use crate::recovered_character_panels::{
    self as character, CharacterPanelEffect, CharacterPanelServices,
};
use crate::recovered_combat_panel::{self as panel, CombatPanelEffect, CombatPanelServices};
use crate::recovered_equipment::{self as equipment, EquipmentEffect, EquipmentLimits};
use crate::recovered_feedback::{
    self as feedback, FeedbackArg, FeedbackEffect, FeedbackObservation, FeedbackRequest,
    FeedbackServices,
};
use crate::recovered_files::{self as files, FileBuffer, FileContext, FileEffect};
use crate::recovered_install::InstallMemory;
use crate::recovered_interface::{self as interface, ButtonInput, InterfaceEffect};
use crate::recovered_inventory::{self as inventory, CompactState};
use crate::recovered_inventory_edit::{
    self as edit, InventoryEditEffect, InventoryEditLimits, InventoryEditServices,
};
use crate::recovered_inventory_rules::{self as restrictions, InventoryEffect};
use crate::recovered_load_control::LoadFlow;
use crate::recovered_navigation::{self as navigation, CoordinateEffect};
use crate::recovered_prompt_control::{
    self as prompt, LibraryObservation, LibraryRequest, PromptControlEffect, PromptControlServices,
    PromptControlState, PromptLimits, PromptRegisterRequest, PromptReturn,
};
use crate::recovered_records::{self as records, ItemTables, RecordWrite};
use crate::recovered_rendering::RenderingMemory;
use crate::recovered_runtime::{self as runtime, RuntimeEffect, RuntimeResult};
use crate::recovered_text::{self as text, NumericServices, PascalString, TextContext};
use crate::{RuleError, rules};
use FeedbackArg::{Byte as B, Long as L, Pointer as P, Word as W};
use centurions_content::{ITEM_BYTES, WARRIOR_BYTES, Warrior};

pub type InventorySelectionState = PromptControlState;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InventorySelectionLimits {
    pub maximum_steps: u32,
    pub prompt: PromptLimits,
    pub equipment: EquipmentLimits,
    pub maximum_random_draws: u64,
}
pub trait InventorySelectionServices:
    PromptControlServices + InventoryEditServices + CombatPanelServices
{
    /// Actual arithmetic inputs consumed by combatupdate2's existing odds projection.
    fn damage_frame(&mut self, _: &RenderingMemory) -> Result<DamageFrame, RuleError> {
        Err(RuleError::Unsupported {
            action: "inventory_selection.combat_panel",
            dependency: "actual damage frame inputs",
        })
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum InventorySelectionEffect {
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
    Prompt(PromptControlEffect),
    Character(CharacterPanelEffect),
    Equipment(EquipmentEffect),
    InventoryEdit(InventoryEditEffect),
    Restriction(InventoryEffect),
    Runtime(RuntimeEffect),
    Audio(AudioEffect),
    File(FileEffect),
    Interface(InterfaceEffect),
    Coordinate(CoordinateEffect),
    Movement {
        member: u16,
        result: navigation::MovementResult,
    },
    Panel(CombatPanelEffect),
}
#[derive(Clone, Debug, PartialEq)]
pub struct InventorySelectionPlan {
    pub state: InventorySelectionState,
    pub effects: Vec<InventorySelectionEffect>,
    pub native_return: PromptReturn,
    pub flow: LoadFlow,
    pub qualification: InventorySelectionQualification,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InventorySelectionQualification {
    SourcePortWithObservedBoundariesAndCapturedChildren,
}
struct Execution<'a, S: ?Sized> {
    plan: InventorySelectionPlan,
    services: &'a mut S,
    limits: InventorySelectionLimits,
    remaining: u32,
    d0: Option<u32>,
}
impl<S: InventorySelectionServices + ?Sized> Execution<'_, S> {
    fn step(&mut self) -> Result<(), RuleError> {
        self.remaining = self.remaining.checked_sub(1).ok_or_else(|| {
            RuleError::Invalid("inventory selection processing budget exhausted".into())
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
    fn bytes(&self, a: u32, n: usize) -> Result<Vec<u8>, RuleError> {
        equipment::read(&self.plan.state.memory, a, n)
    }
    fn write(&mut self, a: u32, bytes: &[u8]) -> Result<(), RuleError> {
        self.bytes(a, bytes.len())?;
        self.plan.state.memory.insert_region(a, bytes);
        self.plan
            .effects
            .push(InventorySelectionEffect::Write(RecordWrite {
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
    fn full(&mut self, d0: u32) {
        self.d0 = Some(d0);
        self.plan.native_return = PromptReturn::Full(d0);
    }
    fn low(&mut self, low: u16) {
        self.d0 = self.d0.map(|d| d & 0xffff0000 | u32::from(low));
        self.plan.native_return = self
            .d0
            .map_or(PromptReturn::LowWord(low), PromptReturn::Full);
    }
    fn unknown(&mut self) {
        self.d0 = None;
        self.plan.native_return = PromptReturn::Unchanged;
    }
    fn copy(&mut self, source: u32, destination: u32, length: usize) -> Result<(), RuleError> {
        for offset in (0..length / 4 * 4).step_by(4) {
            self.long(
                destination.wrapping_add(offset as u32),
                self.l(source.wrapping_add(offset as u32))?,
            )?;
        }
        let offset = length / 4 * 4;
        if length != offset {
            let bytes = self.bytes(source.wrapping_add(offset as u32), length - offset)?;
            self.write(destination.wrapping_add(offset as u32), &bytes)?;
        }
        Ok(())
    }
    fn observe(&mut self, request: FeedbackRequest) -> Result<u32, RuleError> {
        self.step()?;
        let observation =
            FeedbackServices::observe(self.services, &request, &self.plan.state.memory)?;
        let value = observation.value;
        self.d0 = Some(observation.d0);
        self.plan.state.memory = observation.memory.clone();
        self.plan.effects.push(InventorySelectionEffect::Feedback(
            FeedbackEffect::Service {
                request,
                observation,
            },
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
            .services
            .observe_registers(&request, &self.plan.state.memory)?;
        let value = observation.value;
        self.d0 = Some(observation.d0);
        self.plan.state.memory = observation.memory.clone();
        self.plan
            .effects
            .push(InventorySelectionEffect::RegisterToolbox {
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
        self.plan.effects.push(InventorySelectionEffect::Library {
            request,
            observation,
        });
        Ok(value)
    }
    fn returned_word(&self) -> Result<u16, RuleError> {
        match self.plan.native_return {
            PromptReturn::Full(v) => Ok(v as u16),
            PromptReturn::LowWord(v) => Ok(v),
            _ => Err(RuleError::Invalid(
                "inventory child returned no WORD".into(),
            )),
        }
    }
    fn feedback(&mut self, child: feedback::FeedbackPlan) {
        self.plan.state.memory = child.memory;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(InventorySelectionEffect::Feedback),
        );
        if let Some(d0) = child.native_return {
            self.full(d0);
        } else {
            self.unknown();
        }
    }
    fn sound(&mut self, id: i16, stack: u32) -> Result<(), RuleError> {
        self.word(stack.wrapping_sub(2), id as u16)?;
        let child = feedback::sound(&self.plan.state.memory, id, self.services)?;
        self.feedback(child);
        Ok(())
    }
    fn warn(&mut self, frame: u32, id: i16) -> Result<(), RuleError> {
        self.word(frame.wrapping_add(8), id as u16)?;
        let child = feedback::warn(
            &self.plan.state.memory,
            frame,
            id,
            self.limits.prompt.maximum_polls,
            self.services,
        )?;
        self.feedback(child);
        Ok(())
    }
    fn press(&mut self, frame: u32, kind: u16, phase: u16) -> Result<(), RuleError> {
        self.long(frame.wrapping_add(18), self.l(0x442930)?)?;
        self.long(frame.wrapping_add(14), self.l(0x44292c)?)?;
        self.word(frame.wrapping_add(12), 0)?;
        self.long(
            frame.wrapping_add(8),
            u32::from(kind) << 16 | u32::from(phase),
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
    fn prompt(&mut self, entry: u32, frame: u32) -> Result<(), RuleError> {
        self.step()?;
        let limits = PromptLimits {
            maximum_steps: self.remaining,
            ..self.limits.prompt
        };
        let child = match entry {
            0x12382a => prompt::first_tutorial(&self.plan.state, frame, limits, self.services)?,
            0x12512a => prompt::get_choice(&self.plan.state, frame, limits, self.services)?,
            0x14cff6 => prompt::question(&self.plan.state, frame, limits, self.services)?,
            0x15dc34 => prompt::text_box(&self.plan.state, frame, limits, self.services)?,
            0x16043e => prompt::update_controls(&self.plan.state, frame, limits, self.services)?,
            0x1500d8 => prompt::reset_text_area(&self.plan.state, frame, limits, self.services)?,
            _ => return Err(RuleError::Invalid("unknown selection prompt child".into())),
        };
        self.plan.state = child.state;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(InventorySelectionEffect::Prompt),
        );
        self.plan.flow = child.flow;
        match child.native_return {
            PromptReturn::Full(d) => self.full(d),
            PromptReturn::LowWord(w) => self.low(w),
            PromptReturn::Unchanged => {}
            PromptReturn::NoReturn => {
                self.d0 = None;
                self.plan.native_return = PromptReturn::NoReturn;
            }
        }
        Ok(())
    }
    fn first(&mut self, frame: u32, first: u16, last: u16) -> Result<(), RuleError> {
        self.long(
            frame.wrapping_add(8),
            u32::from(first) << 16 | u32::from(last),
        )?;
        self.prompt(0x12382a, frame)
    }
    fn character(&mut self, entry: u32, frame: u32) -> Result<(), RuleError> {
        self.step()?;
        let child = match entry {
            0x13a510 => character::copy_out(
                &self.plan.state.memory,
                frame,
                self.remaining,
                self.services,
            )?,
            0x13a1dc => character::short_update(
                &self.plan.state.memory,
                frame,
                self.remaining,
                self.services,
            )?,
            0x14a3e6 => character::plot_portrait(
                &self.plan.state.memory,
                frame,
                self.remaining,
                self.services,
            )?,
            0x15f66c => character::update_character(
                &self.plan.state.memory,
                frame,
                self.remaining,
                self.services,
            )?,
            _ => {
                return Err(RuleError::Invalid(
                    "unknown selection character child".into(),
                ));
            }
        };
        self.plan.state.memory = child.memory;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(InventorySelectionEffect::Character),
        );
        self.full(child.native_return);
        Ok(())
    }
    fn character_row(&mut self, frame: u32, member: u16, mode: u16) -> Result<(), RuleError> {
        self.long(
            frame.wrapping_add(8),
            u32::from(member) << 16 | u32::from(mode),
        )?;
        self.character(0x15f66c, frame)
    }
    fn load_item(&mut self, id: i16) -> Result<(), RuleError> {
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
    fn pict(&mut self, frame: u32, id: u16, rect: u32) -> Result<(), RuleError> {
        self.long(frame.wrapping_add(14), self.l(rect.wrapping_add(4))?)?;
        self.long(frame.wrapping_add(10), self.l(rect)?)?;
        self.word(frame.wrapping_add(8), id)?;
        let handle = self.trap(0xa9bc, vec![W(id)])?;
        self.long(frame.wrapping_sub(4), handle)?;
        let bounds = self.plan.state.memory.rect(frame.wrapping_add(10))?;
        let child = crate::recovered_drawing::pict(id as i16, bounds, &mut FixedPicture(handle))?;
        if !matches!(
            child.as_slice(),
            [
                crate::recovered_drawing::DrawingEffect::GetPicture(_),
                crate::recovered_drawing::DrawingEffect::DrawPicture { .. }
            ]
        ) {
            return Err(RuleError::Invalid(
                "unexpected selection picture plan".into(),
            ));
        }
        self.trap(
            0xa8f6,
            vec![L(self.l(frame.wrapping_sub(4))?), P(frame.wrapping_add(10))],
        )?;
        Ok(())
    }
    fn dialog_item(&mut self, dialog: u32, item: u16, rect: u32) -> Result<(), RuleError> {
        self.trap(
            0xa98d,
            vec![L(dialog), W(item), P(0x4434b0), P(0x4434dc), P(rect)],
        )?;
        Ok(())
    }
    fn dialog_string(&mut self, frame: u32, item: u16, pointer: u32) -> Result<(), RuleError> {
        self.long(frame.wrapping_add(10), pointer)?;
        self.word(frame.wrapping_add(8), item)?;
        self.dialog_item(self.l(0x4394b8)?, self.w(frame.wrapping_add(8))?, 0x4434ba)?;
        let pointer = self.l(frame.wrapping_add(10))?;
        let length = usize::from(self.b(pointer)?) + 1;
        let value = PascalString::from_raw(&self.bytes(pointer, length)?)?;
        let _child = text::dialog_string(item as i16, &value);
        self.trap(0xa98f, vec![L(self.l(0x4434dc)?), P(pointer)])?;
        self.full(0);
        Ok(())
    }
    fn numeric(&mut self, stack: u32, value: i16) -> Result<(), RuleError> {
        self.word(stack.wrapping_sub(2), value as u16)?;
        self.lib(0x400168, vec![L(i32::from(value) as u32), P(0x479ed8)])?;
        let string =
            PascalString::from_raw(&self.bytes(0x479ed8, usize::from(self.b(0x479ed8)?) + 1)?)?;
        let _child = text::string(&mut TextContext::default(), value, &mut FixedNumber(string))?;
        self.trap(0xa884, vec![P(0x479ed8)])?;
        Ok(())
    }
    fn button(&mut self, frame: u32, pressed: bool, switch: u16) -> Result<(), RuleError> {
        self.word(frame.wrapping_add(8), switch)?;
        let input = ButtonInput {
            rect: self.plan.state.memory.rect(0x44292c)?,
            target_port: self.l(0x43878c)?,
            dark_pattern: self.l(0x4358de)?,
            light_pattern: self.l(0x4358e2)?,
        };
        let child = if pressed {
            interface::downbutton(input, switch as i8, self.services)?
        } else {
            interface::upbutton(input, switch as i8, self.services)?
        };
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(InventorySelectionEffect::Interface),
        );
        self.full(child.native_return as u32);
        Ok(())
    }
    fn movement(&mut self, frame: u32, member: u16) -> Result<(), RuleError> {
        self.word(frame.wrapping_add(8), member)?;
        let warrior = match member {
            0xffff => 0x44721c,
            0xfffe => 0x447e5e,
            _ => record(member),
        };
        let race = self.sw(warrior.wrapping_add(0x32))?.wrapping_sub(1);
        let race_address = 0x435664_u32.wrapping_add(i32::from(race) as u32);
        let start = warrior.min(race_address);
        let end = warrior
            .checked_add(WARRIOR_BYTES as u32)
            .ok_or_else(|| RuleError::Invalid("selection movement address wraps".into()))?
            .max(
                race_address
                    .checked_add(1)
                    .ok_or_else(|| RuleError::Invalid("selection race address wraps".into()))?,
            );
        if end - start > 1024 * 1024 {
            return Err(RuleError::Invalid(
                "selection movement span exceeds one MiB".into(),
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
        self.full(child.native_return as u32);
        self.plan.effects.push(InventorySelectionEffect::Movement {
            member,
            result: child,
        });
        Ok(())
    }
    fn runtime(&mut self, entry: u32) -> Result<(), RuleError> {
        let observations = CharacterPanelServices::runtime_observations(
            self.services,
            entry,
            &self.plan.state.memory,
        )?;
        let mut context =
            runtime::RuntimeContext::new(self.plan.state.random.clone(), self.remaining as usize);
        context.memory.bytes = self.plan.state.memory.bytes.clone();
        let child = match entry {
            0x13a498 => runtime::copy_in(&mut context, &observations)?,
            0x13e13c => runtime::compact_heap(&mut context, &observations)?,
            _ => return Err(RuleError::Invalid("unknown selection runtime child".into())),
        };
        self.plan.state.memory.bytes = context.memory.bytes;
        self.plan.state.random = context.random;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(InventorySelectionEffect::Runtime),
        );
        match child.result {
            RuntimeResult::Long(value) => self.full(value),
            _ => self.unknown(),
        }
        Ok(())
    }
    fn equipment(&mut self, entry: u32, frame: u32) -> Result<(), RuleError> {
        let child = match entry {
            0x121152 => equipment::dropitem(
                &self.plan.state.memory,
                &self.plan.state.random,
                frame,
                self.limits.equipment,
                self.services,
            )?,
            0x164a0e => equipment::wear(
                &self.plan.state.memory,
                &self.plan.state.random,
                frame,
                self.limits.equipment,
                self.services,
            )?,
            _ => {
                return Err(RuleError::Invalid(
                    "unknown selection equipment child".into(),
                ));
            }
        };
        self.plan.state.memory = child.memory;
        self.plan.state.random = child.random;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(InventorySelectionEffect::Equipment),
        );
        if let Some(value) = child.native_return.known_long() {
            self.full(value);
        } else {
            self.d0 = None;
            self.low(child.native_return.low_word);
        }
        Ok(())
    }
    fn coordinates(&mut self, frame: u32) -> Result<(), RuleError> {
        self.word(frame.wrapping_add(8), 0)?;
        let memory = InstallMemory {
            base_address: 0x40226a,
            bytes: self.bytes(0x40226a, (0x479fd8 - 0x40226a) as usize)?,
        };
        let child = navigation::xy(&memory, 0, self.services)?;
        for effect in child.effects {
            if let CoordinateEffect::Write(write) = &effect {
                self.write(write.address, &write.bytes)?;
            } else {
                self.plan
                    .effects
                    .push(InventorySelectionEffect::Coordinate(effect));
            }
        }
        self.full(child.native_return as u32);
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
            self.services
                .quiet_observations(selector, &context, &self.plan.state.memory)?;
        let child = audio::quiet(&mut context, selector, &observations)?;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(InventorySelectionEffect::Audio),
        );
        self.full(0);
        Ok(())
    }
    fn compact(&mut self, frame: u32, member: i16) -> Result<(), RuleError> {
        self.word(frame.wrapping_add(8), member as u16)?;
        let signed = member as i8;
        let last = self.b(0x44273a)? as i8;
        if signed < 0 || signed > last {
            self.full(0);
            return Ok(());
        }
        let state = CompactState {
            warrior: Warrior {
                name: String::new(),
                original_sha256: String::new(),
                bytes: self.bytes(record(signed as u16), WARRIOR_BYTES)?,
            },
            highest_party_index: last,
            scan: self.sw(0x4434aa)?,
            pass: self.sw(0x4434ac)?,
        };
        let child = inventory::compactitems(&state, signed)?;
        for effect in child.effects {
            match effect {
                inventory::InventoryEffect::Write(write) => {
                    self.write(write.address, &write.bytes)?
                }
                other => {
                    return Err(RuleError::Invalid(format!(
                        "unexpected compact item effect: {other:?}"
                    )));
                }
            }
        }
        self.full(0);
        Ok(())
    }
    fn change(&mut self, frame: u32, deferred: u32) -> Result<(), RuleError> {
        self.long(frame.wrapping_add(8), deferred)?;
        let limits = InventoryEditLimits {
            maximum_steps: self.remaining,
            equipment: self.limits.equipment,
            maximum_random_draws: self.limits.maximum_random_draws,
        };
        let child = edit::change(
            &self.plan.state.memory,
            &self.plan.state.random,
            frame,
            limits,
            self.services,
        )?;
        self.plan.state.memory = child.memory;
        self.plan.state.random = child.random;
        self.plan.flow = child.flow;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(InventorySelectionEffect::InventoryEdit),
        );
        if let Some(value) = child.native_return {
            self.full(value);
        } else {
            self.unknown();
        }
        Ok(())
    }
    fn c_string(&self, pointer: u32) -> Result<Vec<u8>, RuleError> {
        let mut bytes = vec![];
        for offset in 0..=self.plan.state.memory.bytes.len() {
            let value = self.b(pointer.wrapping_add(offset as u32))?;
            bytes.push(value);
            if value == 0 {
                return Ok(bytes);
            }
        }
        Err(RuleError::Invalid("unterminated selection text".into()))
    }
    fn profile(&mut self, frame: u32, race: i16, caste: i16) -> Result<(), RuleError> {
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
                _ => self
                    .plan
                    .effects
                    .push(InventorySelectionEffect::File(effect)),
            }
        }
        self.unknown();
        Ok(())
    }
    fn can_use(&mut self, frame: u32, item: i16, race: i16, caste: i16) -> Result<(), RuleError> {
        self.word(frame.wrapping_add(12), caste as u16)?;
        self.word(frame.wrapping_add(10), race as u16)?;
        self.word(frame.wrapping_add(8), item as u16)?;
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
        let child = restrictions::canuse(
            &self.plan.state.memory,
            frame,
            self.limits.prompt.filename_capacity,
            &observations,
        )?;
        self.plan.state.memory = child.memory;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(InventorySelectionEffect::Restriction),
        );
        self.full(child.native_return);
        Ok(())
    }
    fn remove(&mut self, frame: u32) -> Result<(), RuleError> {
        let child = restrictions::removeitem(
            &self.plan.state.memory,
            frame,
            self.limits.prompt.maximum_polls,
            self.services,
        )?;
        self.plan.state.memory = child.memory;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(InventorySelectionEffect::Restriction),
        );
        self.full(child.native_return);
        Ok(())
    }
    fn panel(&mut self, frame: u32, body: i16) -> Result<(), RuleError> {
        self.word(frame.wrapping_add(8), body as u16)?;
        let damage = self.services.damage_frame(&self.plan.state.memory)?;
        let child =
            panel::combat_update2(&self.plan.state.memory, frame, body, damage, self.services)?;
        self.plan.state.memory = child.memory;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(InventorySelectionEffect::Panel),
        );
        self.full(child.native_return as u32);
        Ok(())
    }
    fn indexed_string(&mut self, resource: u16, item: u16) -> Result<(), RuleError> {
        self.lib(0x400178, vec![P(0x479ed8), W(resource), W(item)])?;
        Ok(())
    }
    fn dialog_number(&mut self, frame: u32, item: u16, value: i16) -> Result<(), RuleError> {
        self.word(frame.wrapping_add(10), value as u16)?;
        self.word(frame.wrapping_add(8), item)?;
        self.dialog_item(self.l(0x4394b8)?, self.w(frame.wrapping_add(8))?, 0x4434ba)?;
        let value = self.sw(frame.wrapping_add(10))?;
        self.lib(0x400168, vec![L(i32::from(value) as u32), P(0x479ed8)])?;
        let string =
            PascalString::from_raw(&self.bytes(0x479ed8, usize::from(self.b(0x479ed8)?) + 1)?)?;
        let _child = text::dialog_number(
            &mut TextContext::default(),
            item as i16,
            value,
            &mut FixedNumber(string),
        )?;
        self.trap(0xa98f, vec![L(self.l(0x4434dc)?), P(0x479ed8)])?;
        self.full(0);
        Ok(())
    }
    fn load_spell(
        &mut self,
        frame: u32,
        school: i16,
        level: i16,
        item: i16,
    ) -> Result<(), RuleError> {
        self.word(frame.wrapping_add(12), item as u16)?;
        self.word(frame.wrapping_add(10), level as u16)?;
        self.word(frame.wrapping_add(8), school as u16)?;
        let coordinates = [school as i8, level as i8, item as i8];
        let offset = usize::from(i16::from(coordinates[0]) as u16) * 2842
            + usize::from(i16::from(coordinates[1]) as u16) * 406
            + usize::from(i16::from(coordinates[2]) as u16) * records::SPELL_BYTES;
        if offset > 1024 * 1024 {
            return Err(RuleError::Invalid(
                "selection spell projection exceeds one MiB".into(),
            ));
        }
        let table = self.bytes(0x443788, offset + records::SPELL_BYTES)?;
        let child = records::loadspell(&table, coordinates[0], coordinates[1], coordinates[2])?;
        for write in child.writes {
            self.write(write.address, &write.bytes)?;
        }
        self.full(child.native_return as u32);
        Ok(())
    }
}
struct FixedNumber(PascalString);
impl NumericServices for FixedNumber {
    fn number_to_string(&mut self, _: i32) -> Result<PascalString, RuleError> {
        Ok(self.0.clone())
    }
    fn string_to_number(&mut self, _: &PascalString) -> Result<i32, RuleError> {
        Err(RuleError::Invalid("unused selection numeric input".into()))
    }
}
struct FixedPicture(u32);
impl crate::recovered_drawing::DrawingServices for FixedPicture {
    fn color_icon(&mut self, _: i16) -> Result<u32, RuleError> {
        Err(RuleError::Invalid("unused selection color icon".into()))
    }
    fn indexed_string(&mut self, _: i16, _: i16) -> Result<PascalString, RuleError> {
        Err(RuleError::Invalid("unused selection drawing string".into()))
    }
    fn picture(
        &mut self,
        _: i16,
    ) -> Result<crate::recovered_drawing::PictureObservation, RuleError> {
        Ok(crate::recovered_drawing::PictureObservation {
            handle: self.0,
            bounds: None,
        })
    }
    fn current_world(&mut self) -> Result<(u32, u32), RuleError> {
        Err(RuleError::Invalid("unused selection graphics world".into()))
    }
    fn new_world(
        &mut self,
        _: i16,
        _: crate::recovered_geometry::BodyRect,
    ) -> Result<(i16, crate::recovered_drawing::BuildingWorld), RuleError> {
        Err(RuleError::Invalid("unused selection new world".into()))
    }
}
fn index(base: u32, value: i16, stride: i32) -> u32 {
    base.wrapping_add((i32::from(value).wrapping_mul(stride)) as u32)
}
fn record(member: u16) -> u32 {
    0x448aa0_u32.wrapping_add(u32::from(member) * 3138)
}
fn node(member: u16, slot: u16) -> u32 {
    record(member)
        .wrapping_add(u32::from(slot) * 22)
        .wrapping_add(0x132)
}
fn half(value: i16) -> i16 {
    value.wrapping_add(((value as u16) >> 15) as i16) >> 1
}
fn execution<'a, S: ?Sized>(
    state: &InventorySelectionState,
    limits: InventorySelectionLimits,
    services: &'a mut S,
) -> Execution<'a, S> {
    Execution {
        plan: InventorySelectionPlan {
            state: state.clone(),
            effects: vec![],
            native_return: PromptReturn::Unchanged,
            flow: LoadFlow::Returned,
            qualification:
                InventorySelectionQualification::SourcePortWithObservedBoundariesAndCapturedChildren,
        },
        services,
        limits,
        remaining: limits.maximum_steps,
        d0: None,
    }
}

/// checkforitem, 0x112650: first matching node, with optional quantity consumption.
pub fn check_for_item(
    state: &InventorySelectionState,
    frame: u32,
    limits: InventorySelectionLimits,
    services: &mut (impl InventorySelectionServices + ?Sized),
) -> Result<InventorySelectionPlan, RuleError> {
    let mut e = execution(state, limits, services);
    let item = e.sw(frame.wrapping_add(8))?;
    let consume = e.w(frame.wrapping_add(10))?;
    e.word(frame.wrapping_sub(4), 0)?;
    e.word(frame.wrapping_sub(2), e.sb(0x44273a)? as u16)?;
    e.word(0x438748, 0)?;
    if e.sw(frame.wrapping_add(12))? != -1 {
        e.word(frame.wrapping_sub(4), e.w(frame.wrapping_add(12))?)?;
        e.word(frame.wrapping_sub(2), e.w(frame.wrapping_add(12))?)?;
    }
    let mut member = e.sw(frame.wrapping_sub(4))?;
    while member <= e.sw(frame.wrapping_sub(2))? {
        e.step()?;
        let mut slot = 0_i16;
        while slot < e.sw(record(member as u16).wrapping_add(0x1a))? {
            e.step()?;
            let address = node(member as u16, slot as u16);
            if e.sw(address)? == item {
                let answer = member.wrapping_add(1) as u16;
                e.word(0x438748, slot as u16)?;
                if consume == 0 {
                    e.d0 = None;
                    e.low(answer);
                    return Ok(e.plan);
                }
                if e.sw(address.wrapping_add(2))? <= 0 {
                    e.d0 = None;
                    e.low(0);
                    return Ok(e.plan);
                }
                e.word(frame.wrapping_sub(26), item as u16)?;
                e.load_item(item)?;
                e.add(address.wrapping_add(2), -1)?;
                if e.w(0x453660)? != 0 {
                    e.add(
                        record(member as u16).wrapping_add(0x9d6),
                        e.sw(0x453660)?.wrapping_neg(),
                    )?;
                }
                if e.w(address.wrapping_add(2))? == 0 && e.w(0x453662)? != 0 {
                    let child = frame.wrapping_sub(40);
                    e.word(child.wrapping_add(14), 1)?;
                    e.word(child.wrapping_add(12), slot as u16)?;
                    e.word(child.wrapping_add(10), item as u16)?;
                    e.word(child.wrapping_add(8), member as u16)?;
                    e.equipment(0x121152, child)?;
                }
                e.d0 = None;
                e.low(answer);
                return Ok(e.plan);
            }
            slot = slot.wrapping_add(1);
        }
        member = member.wrapping_add(1);
    }
    e.d0 = None;
    e.low(0);
    Ok(e.plan)
}

/// checkfordoauto, 0x13a5a6: toggle the selected warrior's automatic control.
pub fn check_for_do_auto(
    state: &InventorySelectionState,
    frame: u32,
    limits: InventorySelectionLimits,
    services: &mut (impl InventorySelectionServices + ?Sized),
) -> Result<InventorySelectionPlan, RuleError> {
    let mut e = execution(state, limits, services);
    let mut selected = 0_i16;
    for member in 0..6_i16 {
        e.step()?;
        if e.l(0x439404)? != e.l(index(0x43949c, member, 4))? {
            continue;
        }
        e.first(frame.wrapping_sub(24), 37, 37)?;
        if e.plan.flow != LoadFlow::Returned {
            return Ok(e.plan);
        }
        let handle = e.l(index(0x43949c, member, 4))?;
        let data = e.l(handle)?;
        e.long(0x44292c, e.l(data.wrapping_add(8))?)?;
        e.long(0x442930, e.l(data.wrapping_add(12))?)?;
        if e.b(index(0x442742, member, 1))? != 0 {
            e.sound(139, frame.wrapping_sub(12))?;
            e.byte(index(0x442742, member, 1), 0)?;
            e.character_row(frame.wrapping_sub(26), member as u16, 3)?;
        } else {
            e.sound(147, frame.wrapping_sub(12))?;
            e.byte(index(0x442742, member, 1), 1)?;
            e.character_row(frame.wrapping_sub(26), member as u16, 3)?;
            e.button(frame.wrapping_sub(28), true, 1)?;
            if e.w(0x438776)? != 0 && member == e.sw(index(0x4470b8, e.sw(0x447194)?, 2))? {
                e.long(0x439404, e.l(0x439458)?)?;
            }
            let mut count = 0_i16;
            while count <= e.sb(0x44273a)? {
                e.step()?;
                if e.b(index(0x442742, count, 1))? != 0 {
                    selected = selected.wrapping_add(1);
                }
                count = count.wrapping_add(1);
            }
        }
    }
    if selected > e.sb(0x44273a)? {
        e.warn(frame.wrapping_sub(22), 107)?;
    }
    e.full(0);
    Ok(e.plan)
}

/// order, 0x152188: copy all records before applying the raw choice permutation.
pub fn order(
    state: &InventorySelectionState,
    frame: u32,
    limits: InventorySelectionLimits,
    services: &mut (impl InventorySelectionServices + ?Sized),
) -> Result<InventorySelectionPlan, RuleError> {
    let mut e = execution(state, limits, services);
    let textbox = frame.wrapping_sub(0x49a4);
    e.long(textbox.wrapping_add(12), 0)?;
    e.long(textbox.wrapping_add(8), 0x30021)?;
    e.prompt(0x15dc34, textbox)?;
    if e.plan.flow != LoadFlow::Returned {
        return Ok(e.plan);
    }
    let choice = frame.wrapping_sub(0x49aa);
    e.long(choice.wrapping_add(10), 0x10001)?;
    e.word(
        choice.wrapping_add(8),
        e.sb(0x44273a)?.wrapping_add(1) as u16,
    )?;
    e.prompt(0x12512a, choice)?;
    if e.plan.flow != LoadFlow::Returned {
        return Ok(e.plan);
    }
    let backup = frame.wrapping_sub(0x498c);
    let mut member = 0_i16;
    while member <= e.sb(0x44273a)? {
        e.step()?;
        e.copy(
            record(member as u16),
            backup.wrapping_add(u32::from(member as u16) * 3138),
            WARRIOR_BYTES,
        )?;
        for slot in 0..10_u32 {
            let address = 0x43788a_u32
                .wrapping_add(u32::from(member as u16) * 80)
                .wrapping_add(slot * 8);
            for offset in [6, 4, 2, 0] {
                e.word(address.wrapping_add(offset), 0)?;
            }
        }
        member = member.wrapping_add(1);
    }
    member = 0;
    while member <= e.sb(0x44273a)? {
        e.step()?;
        let destination = e.w(index(0x4434e0, member, 2))?.wrapping_sub(1);
        e.copy(
            backup.wrapping_add(u32::from(member as u16) * 3138),
            record(destination),
            WARRIOR_BYTES,
        )?;
        member = member.wrapping_add(1);
    }
    member = 0;
    while member <= e.sb(0x44273a)? {
        e.step()?;
        for slot in 0..10_u32 {
            for offset in [0, 2, 4, 6] {
                let source = record(member as u16).wrapping_add(0xa76 + slot * 8 + offset);
                let destination =
                    0x43788a_u32.wrapping_add(u32::from(member as u16) * 80 + slot * 8 + offset);
                e.word(destination, e.w(source)?)?;
            }
        }
        member = member.wrapping_add(1);
    }
    e.trap(0xa873, vec![L(e.l(0x43878c)?)])?;
    let reset = frame.wrapping_sub(0x499e);
    e.word(reset.wrapping_add(8), 0)?;
    e.prompt(0x1500d8, reset)?;
    if e.plan.flow != LoadFlow::Returned {
        return Ok(e.plan);
    }
    let short = frame.wrapping_sub(0x49a0);
    e.word(short.wrapping_add(8), 3)?;
    e.character(0x13a1dc, short)?;
    e.prompt(0x16043e, short)?;
    Ok(e.plan)
}

/// updatebooty, 0x15f4b8: movement, encumbrance and inventory count readout.
pub fn update_booty(
    state: &InventorySelectionState,
    frame: u32,
    limits: InventorySelectionLimits,
    services: &mut (impl InventorySelectionServices + ?Sized),
) -> Result<InventorySelectionPlan, RuleError> {
    let mut e = execution(state, limits, services);
    let member = e.w(frame.wrapping_add(8))?;
    e.trap(0xa873, vec![L(e.l(0x438790)?)])?;
    e.trap(0xaa0b, vec![L(e.l(0x4358ce)?)])?;
    e.trap(0xa88a, vec![W(12)])?;
    e.trap(0xa889, vec![W(1)])?;
    e.trap(0xa887, vec![W(e.w(0x402f3c)?)])?;
    e.trap(0xa862, vec![L(69)])?;
    let rectangle = frame.wrapping_sub(8);
    e.word(rectangle.wrapping_add(2), e.w(0x40856c)?.wrapping_add(470))?;
    e.word(
        rectangle.wrapping_add(6),
        e.w(rectangle.wrapping_add(2))?.wrapping_add(20),
    )?;
    e.word(rectangle, member.wrapping_mul(50).wrapping_add(30))?;
    e.word(rectangle.wrapping_add(4), e.w(rectangle)?.wrapping_add(15))?;
    e.movement(frame.wrapping_sub(22), member)?;
    e.trap(
        0xa893,
        vec![
            W(e.w(rectangle.wrapping_add(2))?),
            W(e.w(rectangle)?.wrapping_add(13)),
        ],
    )?;
    e.trap(0xa8a3, vec![P(rectangle)])?;
    e.numeric(
        frame.wrapping_sub(14),
        e.sw(record(member).wrapping_add(0x3e))?
            .wrapping_add(e.sw(record(member).wrapping_add(0x30))?),
    )?;
    e.add(rectangle.wrapping_add(2), 72)?;
    e.add(rectangle.wrapping_add(6), 93)?;
    e.lib(
        0x400168,
        vec![
            L(u32::from(e.w(record(member).wrapping_add(0x9d6))?)),
            P(0x479ed8),
        ],
    )?;
    e.trap(
        0xa893,
        vec![
            W(e.w(rectangle.wrapping_add(2))?),
            W(e.w(rectangle)?.wrapping_add(13)),
        ],
    )?;
    e.trap(0xa8a3, vec![P(rectangle)])?;
    e.trap(0xa884, vec![P(0x479ed8)])?;
    e.lib(
        0x400168,
        vec![
            L(u32::from(e.w(record(member).wrapping_add(0x9d8))?)),
            P(0x479ed8),
        ],
    )?;
    let shift = if e.w(record(member).wrapping_add(0x9d8))? <= 9999 {
        52
    } else {
        47
    };
    e.trap(
        0xa893,
        vec![
            W(e.w(rectangle.wrapping_add(2))?.wrapping_add(shift)),
            W(e.w(rectangle)?.wrapping_add(13)),
        ],
    )?;
    e.trap(0xa884, vec![P(0x479ed8)])?;
    e.add(rectangle.wrapping_add(2), 71)?;
    e.word(
        rectangle.wrapping_add(6),
        e.w(rectangle.wrapping_add(2))?.wrapping_add(19),
    )?;
    e.add(rectangle, -22)?;
    e.add(rectangle.wrapping_add(4), -22)?;
    e.trap(
        0xa893,
        vec![
            W(e.w(rectangle.wrapping_add(2))?),
            W(e.w(rectangle)?.wrapping_add(13)),
        ],
    )?;
    e.trap(0xa8a3, vec![P(rectangle)])?;
    e.numeric(
        frame.wrapping_sub(12),
        e.sw(record(member).wrapping_add(0x1a))?,
    )?;
    Ok(e.plan)
}

/// updatefat, 0x160a8e: accepted-window gating and bounded fatigue indicator.
pub fn update_fat(
    state: &InventorySelectionState,
    frame: u32,
    limits: InventorySelectionLimits,
    services: &mut (impl InventorySelectionServices + ?Sized),
) -> Result<InventorySelectionPlan, RuleError> {
    let mut e = execution(state, limits, services);
    let delta = e.sw(frame.wrapping_add(10))?;
    let first = e.trap(0xa924, vec![])?;
    if first != e.l(0x43879c)? {
        let second = e.trap(0xa924, vec![])?;
        if second != e.l(0x4386ae)? && e.w(frame.wrapping_add(14))? == 0 {
            return Ok(e.plan);
        }
    }
    e.trap(0xa873, vec![L(e.l(0x43878c)?)])?;
    e.trap(0xaa0b, vec![L(e.l(0x4358ce)?)])?;
    if e.w(frame.wrapping_add(12))? != 0 {
        fat_rect(&mut e, 301, 320, 321, 640)?;
        e.pict(frame.wrapping_sub(22), 219, 0x44292c)?;
    }
    if e.w(0x438776)? != 0 {
        return Ok(e.plan);
    }
    if delta < 0 {
        let old_right = e.w(0x442578)?.wrapping_add(501);
        fat_rect(&mut e, 304, 318, 500, old_right)?;
        e.trap(0xa8a3, vec![P(0x44292c)])?;
    }
    e.add(0x442578, delta)?;
    if e.sw(0x442578)? > 135 {
        e.word(0x442578, 135)?;
    }
    if e.sw(0x442578)? < 4 {
        e.word(0x442578, 4)?;
    }
    if e.w(frame.wrapping_add(8))? != 0 {
        fat_rect(&mut e, 300, 321, 321, 640)?;
        e.pict(frame.wrapping_sub(22), 199, 0x44292c)?;
    }
    let right = e.w(0x442578)?.wrapping_add(500);
    fat_rect(&mut e, 304, 317, 500, right)?;
    if e.sw(0x442578)? > 70 {
        e.trap(0xaa0b, vec![L(e.l(0x4358ca)?)])?;
    }
    if e.sw(0x442578)? > 105 {
        e.trap(0xaa0b, vec![L(e.l(0x4358c6)?)])?;
    }
    e.trap(0xa8a3, vec![P(0x44292c)])?;
    e.button(frame.wrapping_sub(14), true, 1)?;
    e.trap(0xaa0b, vec![L(e.l(0x4358ce)?)])?;
    if delta == 0 {
        e.coordinates(frame.wrapping_sub(14))?;
    }
    Ok(e.plan)
}

fn fat_rect<S: InventorySelectionServices + ?Sized>(
    e: &mut Execution<'_, S>,
    top: u16,
    bottom: u16,
    left: u16,
    right: u16,
) -> Result<(), RuleError> {
    e.word(
        0x44292c,
        e.w(0x40856e)?
            .wrapping_add(top)
            .wrapping_sub(e.w(0x439582)?.wrapping_mul(32)),
    )?;
    e.word(
        0x442930,
        e.w(0x40856e)?
            .wrapping_add(bottom)
            .wrapping_sub(e.w(0x439582)?.wrapping_mul(32)),
    )?;
    e.word(0x44292e, e.w(0x40856c)?.wrapping_add(left))?;
    e.word(0x442932, e.w(0x40856c)?.wrapping_add(right))?;
    Ok(())
}

/// actionpicker, 0x100642: eight independent action switches and modal acceptance.
pub fn action_picker(
    state: &InventorySelectionState,
    frame: u32,
    limits: InventorySelectionLimits,
    services: &mut (impl InventorySelectionServices + ?Sized),
) -> Result<InventorySelectionPlan, RuleError> {
    let mut e = execution(state, limits, services);
    e.long(frame.wrapping_sub(12), 0)?;
    e.runtime(0x13a498)?;
    let mut count = 0_u8;
    e.word(0x4434aa, 0)?;
    while e.sw(0x4434aa)? < 8 {
        e.step()?;
        if e.b(index(0x4433f8, e.sw(0x4434aa)?, 1))? != 0 {
            count = count.wrapping_add(1);
        }
        e.add(0x4434aa, 1)?;
    }
    cursor(&mut e, u16::from(count).wrapping_add(147))?;
    let dialog = e.trap(0xa97c, vec![W(148), L(0), L(u32::MAX)])?;
    e.long(0x4394bc, dialog)?;
    e.long(0x4394b8, e.l(0x4394bc)?)?;
    e.trap(0xa873, vec![L(e.l(0x4394bc)?)])?;
    e.trap(0xaa0b, vec![L(e.l(0x4358ce)?)])?;
    e.trap(0xa887, vec![W(e.w(0x402f44)?)])?;
    e.trap(0xa862, vec![L(33)])?;
    e.trap(0xa863, vec![L(30)])?;
    e.trap(
        0xa91b,
        vec![
            L(e.l(0x4394bc)?),
            W(e.w(0x4395a4)?
                .wrapping_add(407)
                .wrapping_add(e.w(0x40856c)?)),
            W(e.w(0x4395a2)?.wrapping_add(4).wrapping_add(e.w(0x40856e)?)),
            B(0),
        ],
    )?;
    e.trap(0xa915, vec![L(e.l(0x4394bc)?)])?;
    let local = frame.wrapping_sub(8);
    e.dialog_item(e.l(0x4394bc)?, 9, local)?;
    e.pict(frame.wrapping_sub(42), 156, local)?;
    e.dialog_item(e.l(0x4394bc)?, 1, 0x4434ba)?;
    e.pict(frame.wrapping_sub(52), 179, 0x4434ba)?;
    e.copy(local, 0x4434ba, 8)?;
    e.word(0x4434aa, 1)?;
    while e.sw(0x4434aa)? < 8 {
        e.step()?;
        e.add(0x4434ba, 50)?;
        e.add(0x4434be, 50)?;
        e.trap(
            0xa8ec,
            vec![
                L(e.l(0x4394bc)?.wrapping_add(2)),
                L(e.l(0x4394bc)?.wrapping_add(2)),
                P(local),
                P(0x4434ba),
                W(0),
                L(0),
            ],
        )?;
        e.add(0x4434aa, 1)?;
    }
    e.copy(local, 0x4434ba, 8)?;
    e.trap(0xa862, vec![L(69)])?;
    e.word(0x4434aa, 0)?;
    while e.sw(0x4434aa)? < 8 {
        e.step()?;
        e.word(index(0x438768, e.sw(0x4434aa)?, 2), 0)?;
        e.trap(
            0xa893,
            vec![
                W(e.w(0x4434bc)?.wrapping_add(30)),
                W(e.w(0x4434ba)?.wrapping_add(30)),
            ],
        )?;
        e.add(0x4434ba, 50)?;
        e.dialog_string(
            frame.wrapping_sub(38),
            e.w(0x4434aa)?.wrapping_add(19),
            index(0x438af4, e.sw(0x4434aa)?, 256),
        )?;
        e.add(0x4434aa, 1)?;
    }
    loop {
        e.step()?;
        cursor(
            &mut e,
            if (count as i8) < 0 {
                147
            } else {
                u16::from(count).wrapping_add(147)
            },
        )?;
        e.registers(0xa032, Some(0xffff), None, vec![])?;
        e.trap(0xa991, vec![L(0), P(0x4434b2)])?;
        e.dialog_item(e.l(0x4394bc)?, e.w(0x4434b2)?, 0x44292c)?;
        match e.w(0x4434b2)? {
            17 if count != 0 => {
                e.trap(0xa873, vec![L(e.l(0x43879c)?)])?;
                e.warn(frame.wrapping_sub(34), 58)?;
                e.trap(0xa873, vec![L(e.l(0x4394bc)?)])?;
            }
            17 | 18 => {
                let accepted = e.w(0x4434b2)? == 17;
                e.press(frame.wrapping_sub(46), 1, 1)?;
                e.word(0x4471ea, u16::from(accepted))?;
                break;
            }
            _ => {
                e.dialog_item(e.l(0x4394bc)?, e.w(0x4434b2)?.wrapping_sub(8), 0x4434ba)?;
                e.add(0x4434ba, 2)?;
                e.add(0x4434bc, 2)?;
                e.add(0x4434c0, -2)?;
                e.add(0x4434be, -2)?;
                let selected = e.w(index(0x438768, e.sw(0x4434b2)?.wrapping_sub(9), 2))? != 0;
                e.trap(
                    0xa8f6,
                    vec![
                        L(e.l(if selected { 0x439554 } else { 0x439550 })?),
                        P(0x4434ba),
                    ],
                )?;
                e.sound(if selected { 144 } else { 145 }, frame.wrapping_sub(24))?;
                e.word(
                    index(0x438768, e.sw(0x4434b2)?.wrapping_sub(9), 2),
                    u16::from(!selected),
                )?;
                count = if selected {
                    count.wrapping_add(1)
                } else {
                    count.wrapping_sub(1)
                };
            }
        }
    }
    e.trap(0xa983, vec![L(e.l(0x4394bc)?)])?;
    e.trap(0xaa1c, vec![L(e.l(0x43952c)?)])?;
    e.character(0x13a510, frame.wrapping_sub(32))?;
    e.low(e.w(0x4471ea)?);
    Ok(e.plan)
}

fn cursor<S: InventorySelectionServices + ?Sized>(
    e: &mut Execution<'_, S>,
    id: u16,
) -> Result<(), RuleError> {
    let handle = e.trap(0xaa1b, vec![W(id)])?;
    e.trap(0xaa1c, vec![L(handle)])?;
    e.trap(0xaa26, vec![L(handle)])?;
    Ok(())
}

/// Showitems, 0x156b26. Recovered directly from the complete native instruction body.
pub fn show_items(
    state: &InventorySelectionState,
    frame: u32,
    limits: InventorySelectionLimits,
    services: &mut (impl InventorySelectionServices + ?Sized),
) -> Result<InventorySelectionPlan, RuleError> {
    let mut e = execution(state, limits, services);
    let mode = e.w(frame.wrapping_add(8))?;
    e.byte(0x442756, 1)?;
    e.trap(0xa93c, vec![L(e.l(0x4427be)?)])?;
    e.trap(0xa937, vec![])?;
    e.trap(0xaa1c, vec![L(e.l(0x43952c)?)])?;
    e.word(0x4434aa, 0)?;
    while e.sw(0x4434aa)? <= e.sw(0x439576)? {
        e.step()?;
        e.word(frame.wrapping_sub(6), e.w(0x4434aa)?)?;
        e.quiet(e.sw(0x4434aa)?)?;
        e.add(0x4434aa, 1)?;
    }
    e.runtime(0x13e13c)?;
    if mode == 0 {
        e.compact(frame.wrapping_sub(14), e.sb(0x44273d)?)?;
        e.compact(frame.wrapping_sub(16), e.sb(0x44273e)?)?;
        let window = e.trap(0xaa46, vec![W(129), L(0), L(u32::MAX)])?;
        e.long(0x438788, window)?;
        e.trap(
            0xa91d,
            vec![
                L(e.l(0x438788)?),
                W(e.w(0x40856c)?.wrapping_add(640)),
                W(e.w(0x40856e)?
                    .wrapping_add(460)
                    .wrapping_sub(e.w(0x439582)?.wrapping_mul(20))),
                B(0),
            ],
        )?;
        e.trap(0xa873, vec![L(e.l(0x438788)?)])?;
        e.trap(0xa888, vec![W(0)])?;
        for (resource, address) in [
            (144, 0x479ed4),
            (153, 0x439424),
            (137, 0x439428),
            (138, 0x439468),
            (139, 0x43946c),
            (200, 0x43941c),
            (201, 0x439420),
            (128, 0x439400),
            (129, 0x4393fc),
            (141, 0x439464),
            (133, 0x439460),
        ] {
            new_control(&mut e, resource, address)?;
        }
        if e.b(0x442753)? != 0 {
            for (resource, address) in [
                (131, 0x439408),
                (130, 0x43940c),
                (132, 0x439410),
                (134, 0x439418),
                (135, 0x439414),
            ] {
                new_control(&mut e, resource, address)?;
            }
        }
    }
    e.word(0x438762, 1)?;
    e.trap(0xa873, vec![L(e.l(0x438788)?)])?;
    e.trap(0xa88a, vec![W(14)])?;
    e.trap(0xa887, vec![W(e.w(0x402f3c)?)])?;
    e.trap(0xa889, vec![W(0)])?;
    e.trap(
        0xa91b,
        vec![
            L(e.l(0x438788)?),
            W(e.w(0x4395a4)?),
            W(e.w(0x4395a2)?),
            B(0),
        ],
    )?;
    e.trap(0xa915, vec![L(e.l(0x438788)?)])?;
    let stack = frame.wrapping_sub(4);
    if e.w(0x439582)? != 0 {
        e.word(0x4434ba, 460)?;
        e.word(0x4434bc, 0)?;
        e.word(0x4434c0, 800)?;
        e.word(0x4434be, 580)?;
        e.pict(stack.wrapping_sub(18), 226, 0x4434ba)?;
        e.word(0x4434ba, 387)?;
        e.word(0x4434be, e.w(0x4434ba)?.wrapping_add(73))?;
        e.word(0x4434bc, 0)?;
        e.word(0x4434c0, 80)?;
        e.pict(stack.wrapping_sub(28), 227, 0x4434ba)?;
        e.word(0x4434bc, 720)?;
        e.word(0x4434c0, 800)?;
        e.pict(stack.wrapping_sub(38), 227, 0x4434ba)?;
    }
    e.word(0x4434ba, 387)?;
    e.word(0x4434be, e.w(0x4434ba)?.wrapping_add(73))?;
    e.word(0x4434bc, half(e.sw(0x40856c)?) as u16)?;
    e.word(0x4434c0, (half(e.sw(0x40856c)?) as u16).wrapping_add(640))?;
    if mode != 2 {
        e.pict(stack.wrapping_sub(18), 148, 0x4434ba)?;
    }
    e.sound(30005, stack)?;
    e.word(0x4434ba, 0)?;
    e.word(0x4434be, 387)?;
    e.word(0x4434bc, (half(e.sw(0x40856c)?) as u16).wrapping_add(263))?;
    e.word(0x4434c0, e.w(0x4434bc)?.wrapping_add(114))?;
    e.trap(0xa958, vec![L(e.l(0x439400)?)])?;
    e.trap(0xa958, vec![L(e.l(0x4393fc)?)])?;
    e.pict(stack.wrapping_sub(20), 170, 0x4434ba)?;
    e.trap(0xa957, vec![L(e.l(0x439400)?)])?;
    e.trap(0xa957, vec![L(e.l(0x4393fc)?)])?;
    if e.b(0x442753)? == 0 {
        e.add(0x4434be, -50)?;
        e.add(0x4434bc, 32)?;
        e.add(0x4434c0, -32)?;
        e.trap(0xaa0b, vec![L(e.l(0x4358ce)?)])?;
        e.trap(0xa8a3, vec![P(0x4434ba)])?;
        e.word(0x4434aa, 0)?;
        while e.sw(0x4434aa)? < 6 {
            e.step()?;
            e.word(0x4434be, e.w(0x4434ba)?.wrapping_add(50))?;
            e.pict(stack.wrapping_sub(18), 154, 0x4434ba)?;
            e.add(0x4434ba, 8)?;
            e.add(0x4434bc, 8)?;
            let member = e.w(0x4434aa)?;
            if e.w(record(member).wrapping_add(0x54))? != 0 {
                let child = stack.wrapping_sub(24);
                e.word(child.wrapping_add(22), 1)?;
                e.word(child.wrapping_add(20), e.w(0x4434aa)?)?;
                e.word(
                    child.wrapping_add(18),
                    e.w(record(e.w(0x4434aa)?).wrapping_add(0x34))?,
                )?;
                e.long(child.wrapping_add(14), e.l(0x4434be)?)?;
                e.long(child.wrapping_add(10), e.l(0x4434ba)?)?;
                e.word(
                    child.wrapping_add(8),
                    e.w(record(e.w(0x4434aa)?).wrapping_add(0x54))?,
                )?;
                e.character(0x14a3e6, child)?;
            }
            e.add(0x4434ba, -8)?;
            e.add(0x4434bc, -8)?;
            e.add(0x4434ba, 50)?;
            e.add(0x4434be, 50)?;
            e.add(0x4434aa, 1)?;
        }
        e.word(0x4434be, e.w(0x4434ba)?.wrapping_add(37))?;
        e.pict(stack.wrapping_sub(18), 175, 0x4434ba)?;
        e.trap(0xaa0b, vec![L(e.l(0x4358da)?)])?;
        e.trap(0xa863, vec![L(30)])?;
    }
    e.trap(
        0xa965,
        vec![
            L(e.l(0x439400)?),
            W(e.sw(0x447236)?.wrapping_sub(9).max(0) as u16),
        ],
    )?;
    if mode == 0 {
        e.word(0x4471a8, 0)?;
    }
    if mode == 2 {
        e.long(0x439404, e.l(0x439420)?)?;
        e.change(stack.wrapping_sub(12), 2)?;
    } else {
        e.long(0x439404, e.l(0x43941c)?)?;
        e.change(stack.wrapping_sub(12), 0)?;
        if e.plan.flow != LoadFlow::Returned {
            return Ok(e.plan);
        }
        e.long(0x439404, e.l(0x439420)?)?;
        e.change(stack.wrapping_sub(16), 0)?;
    }
    Ok(e.plan)
}
/// showcondition, 0x154a46: five menu modes and the selected equipment action.
pub fn show_condition(
    state: &InventorySelectionState,
    frame: u32,
    limits: InventorySelectionLimits,
    services: &mut (impl InventorySelectionServices + ?Sized),
) -> Result<InventorySelectionPlan, RuleError> {
    let mut e = execution(state, limits, services);
    let selected = e.w(frame.wrapping_add(16))?;
    let mut entries = 0_u16;
    let mut initial = 0_u16;
    e.byte(frame.wrapping_sub(0x135), 0)?;
    e.byte(frame.wrapping_sub(0x136), 1)?;
    e.sound(609, frame.wrapping_sub(0x150))?;
    if e.trap(0xa858, vec![L(u32::from(e.w(0x447218)?)), L(256)])? != 0 {
        e.word(frame.wrapping_add(8), 0)?;
        e.word(frame.wrapping_add(10), e.sb(0x44273a)? as u16)?;
        e.byte(frame.wrapping_sub(0x136), 0)?;
    }
    let menu = e.trap(0xa9bf, vec![W(131)])?;
    e.trap(0xa935, vec![L(menu), W(u16::MAX)])?;
    let mut member = e.sw(frame.wrapping_add(8))?;
    while member <= e.sw(frame.wrapping_add(10))? {
        e.step()?;
        if e.w(frame.wrapping_add(12))? != 4 && e.w(frame.wrapping_add(12))? != 3 {
            let name = record(member as u16).wrapping_add(0xa21);
            e.lib(0x400188, vec![P(name)])?;
            e.trap(0xa933, vec![L(menu), P(name)])?;
            e.lib(0x400180, vec![P(name)])?;
            entries = entries.wrapping_add(1);
            e.trap(
                0xa940,
                vec![
                    L(menu),
                    W(entries),
                    W(e.w(record(member as u16).wrapping_add(0x54))?),
                ],
            )?;
        }
        match e.w(frame.wrapping_add(12))? {
            0 => {
                e.first(frame.wrapping_sub(0x15c), 10, 10)?;
                if e.plan.flow != LoadFlow::Returned {
                    return Ok(e.plan);
                }
                for condition in 0..40_i16 {
                    e.step()?;
                    let address = index(record(member as u16).wrapping_add(0x5e), condition, 2);
                    if e.w(address)? == 0 {
                        continue;
                    }
                    entries = entries.wrapping_add(1);
                    e.indexed_string(133, condition as u16 + 1)?;
                    if e.sw(address)? > 0 {
                        let scratch = frame.wrapping_sub(0x132);
                        e.lib(
                            0x400168,
                            vec![L(i32::from(e.sw(address)?) as u32), P(scratch)],
                        )?;
                        e.lib(0x400180, vec![P(scratch)])?;
                        e.lib(0x400180, vec![P(0x479ed8)])?;
                        e.lib(0x400098, vec![P(0x40113c), P(0x479ed8)])?;
                        e.lib(0x400098, vec![P(scratch), P(0x479ed8)])?;
                        e.lib(0x400188, vec![P(0x479ed8)])?;
                    }
                    e.trap(0xa933, vec![L(menu), P(0x479ed8)])?;
                    let style = if e.sw(address)? > 0 {
                        (-41_i16) as u16
                    } else {
                        19
                    };
                    e.trap(0xa944, vec![L(menu), W(entries), W(style)])?;
                }
            }
            1 => {
                e.first(frame.wrapping_sub(0x15c), 9, 9)?;
                if e.plan.flow != LoadFlow::Returned {
                    return Ok(e.plan);
                }
                for slot in 0..100_u16 {
                    e.step()?;
                    let address = node(member as u16, slot);
                    if e.w(address)? == 0 {
                        continue;
                    }
                    entries = entries.wrapping_add(1);
                    e.word(frame.wrapping_sub(0x152), e.w(address)?)?;
                    e.load_item(e.sw(address)?)?;
                    e.word(frame.wrapping_sub(0x154), e.w(0x4536a2)?)?;
                    let base = rules::selection_base(e.sw(0x4536a2)?);
                    e.word(0x4471e8, base as u16)?;
                    e.full(i32::from(base) as u32);
                    e.indexed_string(
                        e.w(0x4471e8)?.wrapping_add(1),
                        e.w(0x4536a2)?.wrapping_sub(e.w(0x4471e8)?).wrapping_add(1),
                    )?;
                    e.trap(0xa933, vec![L(menu), P(0x479ed8)])?;
                    if e.b(address.wrapping_add(9))? != 0 {
                        e.trap(0xa944, vec![L(menu), W(entries), W(19)])?;
                    }
                }
            }
            2 => {
                e.first(frame.wrapping_sub(0x15c), 43, 43)?;
                if e.plan.flow != LoadFlow::Returned {
                    return Ok(e.plan);
                }
                let scratch = frame.wrapping_sub(0x32);
                for stat in 0..12_i16 {
                    e.step()?;
                    let address = index(record(member as u16).wrapping_add(0xae), stat, 2);
                    if e.w(address)? == 0 {
                        continue;
                    }
                    e.lib(0x400090, vec![P(0x40113b), P(scratch)])?;
                    entries = entries.wrapping_add(1);
                    let prefix = if e.sw(address)? > 0 {
                        0x401138
                    } else {
                        0x401136
                    };
                    e.lib(0x400090, vec![P(prefix), P(scratch)])?;
                    e.lib(
                        0x400168,
                        vec![L(i32::from(e.sw(address)?) as u32), P(0x479ed8)],
                    )?;
                    e.lib(0x400180, vec![P(0x479ed8)])?;
                    e.lib(0x400098, vec![P(0x479ed8), P(scratch)])?;
                    e.indexed_string(132, stat as u16 + 1)?;
                    e.lib(0x400180, vec![P(0x479ed8)])?;
                    e.lib(0x400098, vec![P(0x479ed8), P(scratch)])?;
                    e.lib(0x400188, vec![P(scratch)])?;
                    e.trap(0xa933, vec![L(menu), P(scratch)])?;
                }
            }
            3 => {
                e.first(frame.wrapping_sub(0x15c), 44, 44)?;
                if e.plan.flow != LoadFlow::Returned {
                    return Ok(e.plan);
                }
                e.word(0x447214, e.w(0x4395a2)?)?;
                e.word(0x447216, e.w(0x4395a4)?.wrapping_add(335))?;
                e.profile(
                    frame.wrapping_sub(0x160),
                    e.sw(record(selected).wrapping_add(0x32))?,
                    e.sw(record(selected).wrapping_add(0x34))?,
                )?;
                e.trap(0xa933, vec![L(menu), P(0x401112)])?;
                e.trap(
                    0xa940,
                    vec![L(menu), W(1), W(e.w(record(selected).wrapping_add(0x54))?)],
                )?;
                initial = 1;
                for bit in 0..64_i16 {
                    e.step()?;
                    e.indexed_string(6, bit as u16 + 1)?;
                    if e.trap(0xa88c, vec![P(0x479ed8)])? as u16 == 0 {
                        continue;
                    }
                    if e.trap(0xa85d, vec![P(0x41fbac), L(i32::from(bit) as u32)])? as u8 != 0
                        && e.trap(0xa85d, vec![P(0x41f9aa), L(i32::from(bit) as u32)])? as u8 != 0
                    {
                        e.trap(0xa933, vec![L(menu), P(0x479ed8)])?;
                    }
                    if e.trap(0xa85d, vec![P(0x4536bc), L(i32::from(bit) as u32)])? as u8 != 0 {
                        initial = (bit as u16).wrapping_add(2);
                        e.trap(0xa940, vec![L(menu), W(initial), W(385)])?;
                    }
                }
            }
            4 => {
                e.first(frame.wrapping_sub(0x15c), 45, 45)?;
                if e.plan.flow != LoadFlow::Returned {
                    return Ok(e.plan);
                }
                initial = 1;
                e.word(0x447214, e.w(0x4395a2)?)?;
                e.word(0x447216, e.w(0x4395a4)?.wrapping_add(335))?;
                e.trap(0xa933, vec![L(menu), P(0x4010e8)])?;
                e.trap(0xa940, vec![L(menu), L(0x17b0001)])?;
                member = 0;
                while member <= e.sb(0x44273a)? {
                    e.step()?;
                    let name = record(member as u16).wrapping_add(0xa21);
                    e.lib(0x400188, vec![P(name)])?;
                    e.trap(0xa933, vec![L(menu), P(name)])?;
                    e.trap(
                        0xa940,
                        vec![
                            L(menu),
                            W((member as u16).wrapping_add(2)),
                            W(e.w(record(member as u16).wrapping_add(0x54))?),
                        ],
                    )?;
                    e.lib(0x400180, vec![P(name)])?;
                    e.can_use(
                        frame.wrapping_sub(0x166),
                        e.sw(0x4536a2)?,
                        e.sw(record(member as u16).wrapping_add(0x32))?,
                        e.sw(record(member as u16).wrapping_add(0x34))?,
                    )?;
                    if e.returned_word()? == 0 {
                        e.trap(0xa93a, vec![L(menu), W((member as u16).wrapping_add(2))])?;
                    }
                    member = member.wrapping_add(1);
                }
                e.trap(0xa944, vec![L(menu), W(selected.wrapping_add(2)), W(19)])?;
            }
            _ => {}
        }
        member = member.wrapping_add(1);
    }
    let (vertical, horizontal) = if matches!(e.w(frame.wrapping_add(12))?, 3 | 4) {
        (e.w(0x447214)?, e.w(0x447216)?.wrapping_sub(50))
    } else {
        (400, 50)
    };
    let choice = e.trap(
        0xa80b,
        vec![L(menu), W(vertical), W(horizontal), W(initial)],
    )? as u16;
    e.word(frame.wrapping_sub(0x134), choice)?;
    e.trap(
        0xa943,
        vec![
            L(menu),
            W(e.w(frame.wrapping_sub(0x134))?),
            P(frame.wrapping_sub(0x138)),
        ],
    )?;
    e.trap(0xa936, vec![W(131)])?;
    e.trap(0xa9a3, vec![L(menu)])?;
    e.word(
        0x4434b6,
        e.w(record(e.w(frame.wrapping_add(8))?).wrapping_add(0xca))?,
    )?;
    e.full(u32::from(e.w(frame.wrapping_add(8))?) * 3138);
    if e.w(frame.wrapping_add(12))? == 0
        || e.sw(frame.wrapping_sub(0x134))? <= 1
        || e.b(frame.wrapping_sub(0x136))? == 0
        || e.w(frame.wrapping_add(14))? != 0
    {
        e.sound(607, frame.wrapping_sub(0x150))?;
    } else if e.w(frame.wrapping_sub(0x138))? == 0 {
        let child = frame.wrapping_sub(0x15e);
        e.word(child.wrapping_add(12), 1)?;
        e.word(
            child.wrapping_add(10),
            e.w(frame.wrapping_sub(0x134))?.wrapping_sub(2),
        )?;
        e.word(child.wrapping_add(8), e.w(frame.wrapping_add(8))?)?;
        e.equipment(0x164a0e, child)?;
        if e.returned_word()? != 0 {
            e.byte(
                node(
                    e.w(frame.wrapping_add(8))?,
                    e.w(frame.wrapping_sub(0x134))?.wrapping_sub(2),
                )
                .wrapping_add(9),
                1,
            )?;
            e.character_row(frame.wrapping_sub(0x15c), e.w(frame.wrapping_add(8))?, 3)?;
            e.byte(frame.wrapping_sub(0x135), 1)?;
        }
    } else if e.w(frame.wrapping_sub(0x138))? == 19 {
        let child = frame.wrapping_sub(0x160);
        e.long(child.wrapping_add(12), 0x10000)?;
        e.word(
            child.wrapping_add(10),
            e.w(frame.wrapping_sub(0x134))?.wrapping_sub(2),
        )?;
        e.word(child.wrapping_add(8), e.w(frame.wrapping_add(8))?)?;
        e.remove(child)?;
        if e.returned_word()? == 0 {
            e.full(u32::MAX);
            return Ok(e.plan);
        }
        e.character_row(frame.wrapping_sub(0x15c), e.w(frame.wrapping_add(8))?, 3)?;
        e.byte(frame.wrapping_sub(0x135), 1)?;
    } else {
        e.low(e.w(frame.wrapping_sub(0x138))?.wrapping_sub(19));
    }
    if e.w(0x438776)? != 0 && e.b(frame.wrapping_sub(0x135))? != 0 {
        e.panel(frame.wrapping_sub(0x15a), e.sw(frame.wrapping_add(8))?)?;
        e.prompt(0x16043e, frame.wrapping_sub(0x15a))?;
        if e.plan.flow != LoadFlow::Returned {
            return Ok(e.plan);
        }
    }
    e.low(e.w(frame.wrapping_sub(0x134))?.wrapping_sub(2));
    Ok(e.plan)
}

/// scanselect, 0x15a9cc: seven-by-seven learning selection and memorized-slot cleanup.
pub fn scan_select(
    state: &InventorySelectionState,
    frame: u32,
    limits: InventorySelectionLimits,
    services: &mut (impl InventorySelectionServices + ?Sized),
) -> Result<InventorySelectionPlan, RuleError> {
    let mut e = execution(state, limits, services);
    let cost_table = frame.wrapping_sub(0x16);
    let rectangle = frame.wrapping_sub(8);
    let backup = frame.wrapping_sub(0xc5e);
    let base = frame.wrapping_sub(0xc7e);
    e.copy(0x400aea, cost_table, 14)?;
    e.sound(3000, base)?;
    e.word(base.wrapping_sub(4), e.w(0x447256)?)?;
    let mut remaining = rules::spell_count(e.sw(0x447256)?) as i16;
    e.word(frame.wrapping_sub(0xc60), 0)?;
    e.copy(0x44721c, backup, WARRIOR_BYTES)?;
    e.profile(frame.wrapping_sub(0xc8e), e.sw(0x44724e)?, e.sw(0x447250)?)?;
    e.word(
        frame.wrapping_sub(0x1c),
        e.w(0x41fa4c)?
            .wrapping_add(e.w(0x41fa52)?)
            .wrapping_add(e.w(0x41fa58)?),
    )?;
    let dialog = e.trap(0xa97c, vec![W(135), L(0), L(u32::MAX)])?;
    e.trap(0xa915, vec![L(dialog)])?;
    e.trap(0xa873, vec![L(dialog)])?;
    e.trap(0xaa0b, vec![L(e.l(0x4358ce)?)])?;
    e.trap(0xa88a, vec![W(12)])?;
    e.long(0x4394b8, dialog)?;
    e.trap(0xa862, vec![L(69)])?;
    e.trap(0xa981, vec![L(dialog)])?;
    e.trap(0xa922, vec![L(dialog)])?;
    e.trap(0xa923, vec![L(dialog)])?;
    e.dialog_item(dialog, 34, 0x4434ba)?;
    let portrait = frame.wrapping_sub(0xc9a);
    e.long(portrait.wrapping_add(20), 0xffff0000)?;
    e.word(portrait.wrapping_add(18), e.w(0x447250)?)?;
    e.long(portrait.wrapping_add(14), e.l(0x4434be)?)?;
    e.long(portrait.wrapping_add(10), e.l(0x4434ba)?)?;
    e.word(portrait.wrapping_add(8), e.w(0x447270)?)?;
    e.character(0x14a3e6, portrait)?;
    e.trap(0xa862, vec![L(69)])?;
    e.trap(0xa887, vec![W(e.w(0x402f3c)?)])?;
    e.lib(0x400188, vec![P(0x447c3d)])?;
    e.dialog_string(frame.wrapping_sub(0xca4), 33, 0x447c3d)?;
    e.lib(0x400180, vec![P(0x447c3d)])?;
    e.trap(0xa887, vec![W(e.w(0x402f44)?)])?;
    for level in 0..7_i16 {
        e.dialog_number(
            base.wrapping_sub(12),
            24 + level as u16,
            e.sw(index(cost_table, level, 2))?,
        )?;
    }
    let mut level = 0_u16;
    e.word(frame.wrapping_sub(0xc62), 10)?;
    e.dialog_item(dialog, 23, rectangle)?;
    e.pict(base.wrapping_sub(18), 131, rectangle)?;
    e.dialog_item(dialog, 9, 0x4434ba)?;
    e.pict(base.wrapping_sub(28), 130, 0x4434ba)?;
    e.dialog_item(dialog, 2, 0x44292c)?;
    e.button(base.wrapping_sub(30), true, 0)?;
    e.add(rectangle, 3)?;
    e.word(rectangle.wrapping_add(2), 152)?;
    e.word(rectangle.wrapping_add(6), 161)?;
    e.word(rectangle.wrapping_add(4), e.w(rectangle)?.wrapping_add(9))?;
    e.dialog_string(base.wrapping_sub(36), 36, 0x400ae8)?;
    scan_rows(&mut e, dialog, frame, level, false)?;
    for row in 0..7_u16 {
        for column in 0..7_i16 {
            e.step()?;
            let address = learned(0x447c0c, row, column);
            if e.b(address)? != 0
                && remaining.wrapping_sub(e.sw(index(cost_table, row as i16, 2))?) > -1
            {
                remaining = remaining.wrapping_sub(e.sw(index(cost_table, row as i16, 2))?);
            } else if e.b(address)? != 0 {
                e.byte(address, 0)?;
            }
        }
    }
    e.dialog_number(base.wrapping_sub(12), 40, remaining)?;
    e.trap(0xa922, vec![L(dialog)])?;
    e.trap(0xa923, vec![L(dialog)])?;
    e.first(base.wrapping_sub(16), 135, 136)?;
    if e.plan.flow != LoadFlow::Returned {
        return Ok(e.plan);
    }
    loop {
        e.step()?;
        e.trap(0xa873, vec![L(dialog)])?;
        e.registers(0xa032, Some(0xffff), None, vec![])?;
        e.trap(0xa991, vec![L(0), P(0x4434b2)])?;
        e.dialog_item(dialog, e.w(0x4434b2)?, 0x44292c)?;
        if matches!(e.w(0x4434b2)?, 1 | 32) {
            if remaining != 0 {
                let question = base.wrapping_sub(12);
                e.long(question.wrapping_add(8), 0x110001)?;
                e.prompt(0x14cff6, question)?;
                if e.plan.flow != LoadFlow::Returned {
                    return Ok(e.plan);
                }
                if e.returned_word()? == 0 {
                    continue;
                }
            }
            e.press(base.wrapping_sub(22), 1, 1)?;
            break;
        }
        if e.sw(0x4434b2)? < 9 && e.sw(0x4434b2)? > 1 {
            if e.sw(0x4434b2)?.wrapping_sub(1) > e.sw(frame.wrapping_sub(0x1c))? {
                e.warn(base.wrapping_sub(10), 74)?;
                continue;
            }
            e.dialog_item(dialog, 9, 0x4434ba)?;
            e.pict(base.wrapping_sub(18), 130, 0x4434ba)?;
            e.button(base.wrapping_sub(20), true, 0)?;
            e.sound(137, base.wrapping_sub(12))?;
            level = e.w(0x4434b2)?.wrapping_sub(2);
            e.dialog_item(dialog, 23, rectangle)?;
            e.pict(base.wrapping_sub(32), 131, rectangle)?;
            e.add(rectangle, 2)?;
            e.word(rectangle.wrapping_add(4), e.w(rectangle)?.wrapping_add(9))?;
            e.word(rectangle.wrapping_add(2), 152)?;
            e.word(rectangle.wrapping_add(6), 161)?;
            e.trap(0xa862, vec![L(69)])?;
            scan_rows(&mut e, dialog, frame, level, true)?;
        }
        if e.sw(0x4434b2)? <= 9 || e.sw(0x4434b2)? >= 22 {
            continue;
        }
        e.trap(0xa862, vec![L(69)])?;
        e.word(
            0x4434b6,
            (level as i16)
                .wrapping_mul(7)
                .wrapping_add(e.sw(0x447252)?.wrapping_sub(1).wrapping_mul(49))
                .wrapping_add(e.sw(0x4434b2)?)
                .wrapping_sub(10) as u16,
        )?;
        let handle = e.lib(0x400070, vec![P(0x400aca), P(0x400ace)])?;
        e.lib(
            0x400078,
            vec![
                W(0),
                L((i32::from(e.sw(0x4434b6)?) as u32).wrapping_shl(8)),
                L(handle),
            ],
        )?;
        e.lib(0x400048, vec![L(handle), L(1), L(256), P(0x479ed8)])?;
        e.lib(0x400068, vec![L(handle)])?;
        e.dialog_string(base.wrapping_sub(52), 39, 0x479ed8)?;
        e.trap(0xa990, vec![L(e.l(0x4434dc)?), P(0x479ed8)])?;
        let mut changed = false;
        let original = e.b(learned(
            backup.wrapping_add(0x9f0),
            level,
            e.sw(0x4434b2)?.wrapping_sub(10),
        ))?;
        let should_remove = if original == 0
            && e.b(learned(0x447c0c, level, e.sw(0x4434b2)?.wrapping_sub(10)))? != 0
        {
            true
        } else {
            e.b(learned(0x447c0c, level, e.sw(0x4434b2)?.wrapping_sub(10)))? != 0
        };
        if should_remove {
            e.button(base.wrapping_sub(10), false, 0)?;
            e.trap(0xa862, vec![L(69)])?;
            e.indexed_string(
                e.w(0x447252)?.wrapping_mul(1000).wrapping_add(level),
                e.w(0x4434b2)?.wrapping_sub(9),
            )?;
            e.dialog_string(base.wrapping_sub(16), e.w(0x4434b2)?, 0x479ed8)?;
            e.sound(144, base.wrapping_sub(8))?;
            scan_mark(&mut e, rectangle, false, false)?;
            e.byte(
                learned(0x447c0c, level, e.sw(0x4434b2)?.wrapping_sub(10)),
                0,
            )?;
            remaining = remaining.wrapping_add(e.sw(index(cost_table, level as i16, 2))?);
            e.dialog_number(base.wrapping_sub(22), 40, remaining)?;
            e.add(index(0x44725e, level as i16, 2), 1)?;
            changed = true;
        }
        if e.b(learned(0x447c0c, level, e.sw(0x4434b2)?.wrapping_sub(10)))? == 0
            && remaining >= e.sw(index(cost_table, level as i16, 2))?
            && !changed
        {
            e.button(base.wrapping_sub(10), true, 0)?;
            e.indexed_string(
                e.w(0x447252)?.wrapping_mul(1000).wrapping_add(level),
                e.w(0x4434b2)?.wrapping_sub(9),
            )?;
            e.dialog_string(base.wrapping_sub(16), e.w(0x4434b2)?, 0x479ed8)?;
            e.trap(0xa862, vec![L(69)])?;
            e.sound(145, base.wrapping_sub(8))?;
            scan_mark(&mut e, rectangle, true, true)?;
            remaining = remaining.wrapping_sub(e.sw(index(cost_table, level as i16, 2))?);
            e.dialog_number(base.wrapping_sub(22), 40, remaining)?;
            e.byte(
                learned(0x447c0c, level, e.sw(0x4434b2)?.wrapping_sub(10)),
                1,
            )?;
            e.add(index(0x44725e, level as i16, 2), -1)?;
        }
    }
    e.byte(0x447bf8, 0)?;
    for row in 0..7_u16 {
        for column in 0..7_i16 {
            e.step()?;
            if e.b(learned(0x447c0c, row, column))? != 0 {
                e.load_spell(
                    base.wrapping_sub(14),
                    e.sw(0x447252)?.wrapping_sub(1),
                    row as i16,
                    column,
                )?;
                if e.w(0x443776)? == 57 {
                    e.byte(0x447bf8, 1)?;
                }
            }
        }
    }
    for slot in 0..10_i16 {
        e.step()?;
        let memorized = index(0x447c92, slot, 8);
        e.word(frame.wrapping_sub(0x18), e.w(memorized.wrapping_add(2))?)?;
        e.word(frame.wrapping_sub(0x1a), e.w(memorized.wrapping_add(4))?)?;
        if e.b(learned(
            0x447c0c,
            e.w(frame.wrapping_sub(0x18))?,
            e.sw(frame.wrapping_sub(0x1a))?,
        ))? == 0
        {
            for offset in [0, 2, 4, 6] {
                e.word(memorized.wrapping_add(offset), 0)?;
            }
        }
    }
    e.word(0x45379e, e.sb(0x44375f)? as u16)?;
    e.word(0x45379c, e.sb(0x443760)? as u16)?;
    e.trap(0xa983, vec![L(dialog)])?;
    e.full(0);
    Ok(e.plan)
}
fn learned(base: u32, level: u16, column: i16) -> u32 {
    base.wrapping_add(u32::from(level) * 7)
        .wrapping_add(i32::from(column) as u32)
}
fn scan_rows<S: InventorySelectionServices + ?Sized>(
    e: &mut Execution<'_, S>,
    dialog: u32,
    frame: u32,
    level: u16,
    switched: bool,
) -> Result<(), RuleError> {
    let rectangle = frame.wrapping_sub(8);
    let base = frame.wrapping_sub(0xc7e);
    for column in 0..7_i16 {
        e.step()?;
        e.dialog_item(dialog, column as u16 + 10, 0x44292c)?;
        if e.b(learned(0x447c0c, level, column))? != 0 {
            if !switched {
                e.trap(0xa8f6, vec![L(e.l(0x439550)?), P(rectangle)])?;
            }
            e.button(base.wrapping_sub(10), true, 0)?;
        } else {
            e.button(base.wrapping_sub(10), false, 0)?;
            e.trap(0xa862, vec![L(69)])?;
        }
        if switched {
            if e.b(learned(0x447c0c, level, column))? != 0 {
                e.trap(0xa8f6, vec![L(e.l(0x439550)?), P(rectangle)])?;
            }
            e.add(rectangle, 24)?;
            e.add(rectangle.wrapping_add(4), 24)?;
        }
        e.indexed_string(
            e.w(0x447252)?.wrapping_mul(1000).wrapping_add(level),
            column as u16 + 1,
        )?;
        e.dialog_string(base.wrapping_sub(14), column as u16 + 42, 0x479ed8)?;
        if !switched {
            e.add(rectangle, 24)?;
            e.add(rectangle.wrapping_add(4), 24)?;
        }
    }
    Ok(())
}
fn scan_mark<S: InventorySelectionServices + ?Sized>(
    e: &mut Execution<'_, S>,
    rectangle: u32,
    selected: bool,
    bottom_first: bool,
) -> Result<(), RuleError> {
    e.word(
        rectangle,
        e.sw(0x4434b2)?
            .wrapping_sub(10)
            .wrapping_mul(24)
            .wrapping_add(210) as u16,
    )?;
    if bottom_first {
        e.word(rectangle.wrapping_add(4), e.w(rectangle)?.wrapping_add(9))?;
    }
    e.word(rectangle.wrapping_add(2), 152)?;
    e.word(rectangle.wrapping_add(6), 161)?;
    if !bottom_first {
        e.word(rectangle.wrapping_add(4), e.w(rectangle)?.wrapping_add(9))?;
    }
    e.trap(
        0xa8f6,
        vec![
            L(e.l(if selected { 0x439550 } else { 0x439554 })?),
            P(rectangle),
        ],
    )?;
    Ok(())
}

fn new_control<S: InventorySelectionServices + ?Sized>(
    e: &mut Execution<'_, S>,
    resource: u16,
    address: u32,
) -> Result<(), RuleError> {
    let handle = e.trap(0xa9be, vec![W(resource), L(e.l(0x438788)?)])?;
    e.long(address, handle)?;
    let data = e.l(e.l(address)?)?;
    e.trap(
        0xa959,
        vec![
            L(e.l(address)?),
            W((half(e.sw(0x40856c)?) as u16).wrapping_add(e.w(data.wrapping_add(10))?)),
            W(e.w(data.wrapping_add(8))?),
        ],
    )?;
    Ok(())
}
