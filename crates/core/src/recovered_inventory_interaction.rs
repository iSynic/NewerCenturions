//! Item details, ammunition handling and inventory interaction from CODE 1.

use crate::recovered_audio::AudioEffect;
use crate::recovered_character_panels::{
    self as character, CharacterPanelEffect, CharacterPanelServices,
};
use crate::recovered_character_upgrade::{
    self as upgrade, CharacterUpgradeEffect, CharacterUpgradeServices,
};
use crate::recovered_combat_panel::{self as panel, CombatPanelEffect};
use crate::recovered_detail_ui::{self as detail, DetailEffect};
use crate::recovered_drawing::{self as drawing, DrawingContext, DrawingEffect};
use crate::recovered_equipment::{self as equipment, EquipmentEffect, EquipmentLimits};
use crate::recovered_feedback::{
    self as feedback, FeedbackArg, FeedbackEffect, FeedbackObservation, FeedbackRequest,
    FeedbackServices,
};
use crate::recovered_files::{self as files, FileBuffer, FileContext, FileEffect};
use crate::recovered_geometry::BodyRect;
use crate::recovered_install::InstallMemory;
use crate::recovered_interface::{self as interface, ButtonInput, InterfaceEffect};
use crate::recovered_inventory::{self as inventory, CompactState};
use crate::recovered_inventory_edit::{self as edit, InventoryEditEffect, InventoryEditLimits};
use crate::recovered_inventory_rules::{self as restrictions, InventoryEffect};
use crate::recovered_load_control::LoadFlow;
use crate::recovered_navigation::{self as navigation, CoordinateEffect};
use crate::recovered_prompt_control::{
    self as prompt, LibraryObservation, LibraryRequest, PromptControlEffect, PromptControlState,
    PromptLimits, PromptRegisterRequest, PromptReturn,
};
use crate::recovered_records::{self as records, ItemTables, RecordWrite};
use crate::recovered_rendering::RenderingMemory;
use crate::recovered_runtime::{self as runtime, RuntimeEffect, RuntimeResult};
use crate::recovered_text::{self as text, NumericServices, PascalString, TextContext};
use crate::{RuleError, rules};
use FeedbackArg::{Byte as B, Long as L, Pointer as P, Word as W};
use centurions_content::{ITEM_BYTES, WARRIOR_BYTES, Warrior};

pub type InventoryInteractionState = PromptControlState;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InventoryInteractionLimits {
    pub maximum_steps: u32,
    pub prompt: PromptLimits,
    pub equipment: EquipmentLimits,
    pub maximum_random_draws: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrackControlInvocation {
    pub native_frame: u32,
    pub part: u16,
    pub control: u32,
    pub incoming_d3: i16,
}
pub trait InventoryInteractionServices: CharacterUpgradeServices {
    fn incoming_quick_info_a2(&mut self, _: &RenderingMemory) -> Result<u32, RuleError> {
        Err(RuleError::Unsupported {
            action: "inventory_interaction.quick_info",
            dependency: "actual incoming A2 register value",
        })
    }
    fn track_control_invocations(
        &mut self,
        _: &FeedbackRequest,
        _: &RenderingMemory,
    ) -> Result<Vec<TrackControlInvocation>, RuleError> {
        Err(RuleError::Unsupported {
            action: "inventory_interaction.track_control",
            dependency: "actual application callback invocation inputs",
        })
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum InventoryInteractionEffect {
    Write(RecordWrite),
    ItemsRegisters(ItemsRegisterObservation),
    Application(Box<crate::recovered_application_control::ApplicationControlEffect>),
    Selection(crate::recovered_inventory_selection::InventorySelectionEffect),
    Upgrade(CharacterUpgradeEffect),
    Detail(DetailEffect),
    Drawing(DrawingEffect),
    ItemUi(crate::recovered_item_ui::ItemUiEffect),
    Shop(crate::recovered_shop_ui::ShopEffect),
    Status(crate::recovered_status_panels::StatusPanelEffect),
    EquipmentUi(crate::recovered_equipment_ui::EquipmentUiEffect),
    Cost(crate::recovered_economy::ItemCostOutput),
    RandomItem(crate::recovered_random_item::RandomItemEffect),
    Storage(crate::recovered_storage::StorageEffect),
    Timing(crate::recovered_timing::TimingEffect),
    TrackControl {
        request: FeedbackRequest,
        invocations: Vec<TrackControlInvocation>,
    },
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
pub struct InventoryInteractionPlan {
    pub state: InventoryInteractionState,
    pub effects: Vec<InventoryInteractionEffect>,
    pub native_return: PromptReturn,
    pub flow: LoadFlow,
    pub qualification: InventoryInteractionQualification,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InventoryInteractionQualification {
    SourcePortWithObservedBoundariesAndCapturedChildren,
}
pub struct Execution<'a, S: ?Sized + 'a> {
    pub plan: InventoryInteractionPlan,
    pub services: &'a mut S,
    pub resolver: Option<&'a dyn crate::recovered_application_control::RoutineResolver>,
    pub limits: InventoryInteractionLimits,
    pub remaining: u32,
    pub initial_random_draws: u64,
    pub d0: Option<u32>,
}
impl<S: ?Sized> crate::recovered_application_control::SourceMemory for Execution<'_, S> {
    fn source_memory(&self) -> &RenderingMemory {
        &self.plan.state.memory
    }
    fn source_memory_mut(&mut self) -> &mut RenderingMemory {
        &mut self.plan.state.memory
    }
    fn source_write(&mut self, write: RecordWrite) {
        self.plan
            .effects
            .push(InventoryInteractionEffect::Write(write));
    }
}
impl<S: InventoryInteractionServices + ?Sized> Execution<'_, S> {
    pub fn random_remaining(&self) -> Result<u64, RuleError> {
        let used = self
            .plan
            .state
            .random
            .draws
            .checked_sub(self.initial_random_draws)
            .ok_or_else(|| {
                RuleError::Invalid("interaction RNG draw counter moved backward".into())
            })?;
        self.limits
            .maximum_random_draws
            .checked_sub(used)
            .ok_or_else(|| RuleError::Invalid("interaction RNG budget exhausted".into()))
    }
    pub fn check_random(&self, random: &crate::RandomState) -> Result<(), RuleError> {
        let used = random
            .draws
            .checked_sub(self.initial_random_draws)
            .ok_or_else(|| {
                RuleError::Invalid("interaction RNG draw counter moved backward".into())
            })?;
        if used > self.limits.maximum_random_draws {
            return Err(RuleError::Invalid(
                "interaction RNG budget exhausted".into(),
            ));
        }
        Ok(())
    }
    pub fn child_limits(&self) -> Result<InventoryInteractionLimits, RuleError> {
        Ok(InventoryInteractionLimits {
            maximum_steps: self.remaining,
            maximum_random_draws: self.random_remaining()?,
            ..self.limits
        })
    }
    pub fn step(&mut self) -> Result<(), RuleError> {
        self.remaining = self.remaining.checked_sub(1).ok_or_else(|| {
            RuleError::Invalid("inventory selection processing budget exhausted".into())
        })?;
        Ok(())
    }
    pub fn b(&self, a: u32) -> Result<u8, RuleError> {
        self.plan.state.memory.byte(a)
    }
    pub fn w(&self, a: u32) -> Result<u16, RuleError> {
        self.plan.state.memory.word(a)
    }
    pub fn sw(&self, a: u32) -> Result<i16, RuleError> {
        Ok(self.w(a)? as i16)
    }
    pub fn sb(&self, a: u32) -> Result<i16, RuleError> {
        Ok(i16::from(self.b(a)? as i8))
    }
    pub fn l(&self, a: u32) -> Result<u32, RuleError> {
        self.plan.state.memory.long(a)
    }
    pub fn bytes(&self, a: u32, n: usize) -> Result<Vec<u8>, RuleError> {
        equipment::read(&self.plan.state.memory, a, n)
    }
    pub fn write(&mut self, a: u32, bytes: &[u8]) -> Result<(), RuleError> {
        self.bytes(a, bytes.len())?;
        self.plan.state.memory.insert_region(a, bytes);
        self.plan
            .effects
            .push(InventoryInteractionEffect::Write(RecordWrite {
                address: a,
                bytes: bytes.to_vec(),
            }));
        Ok(())
    }
    pub fn byte(&mut self, a: u32, v: u8) -> Result<(), RuleError> {
        self.write(a, &[v])
    }
    pub fn word(&mut self, a: u32, v: u16) -> Result<(), RuleError> {
        self.write(a, &v.to_be_bytes())
    }
    pub fn long(&mut self, a: u32, v: u32) -> Result<(), RuleError> {
        self.write(a, &v.to_be_bytes())
    }
    pub fn add(&mut self, a: u32, v: i16) -> Result<(), RuleError> {
        self.word(a, self.w(a)?.wrapping_add(v as u16))
    }
    pub fn full(&mut self, d0: u32) {
        self.d0 = Some(d0);
        self.plan.native_return = PromptReturn::Full(d0);
    }
    pub fn low(&mut self, low: u16) {
        self.d0 = self.d0.map(|d| d & 0xffff0000 | u32::from(low));
        self.plan.native_return = self
            .d0
            .map_or(PromptReturn::LowWord(low), PromptReturn::Full);
    }
    pub fn unknown(&mut self) {
        self.d0 = None;
        self.plan.native_return = PromptReturn::Unchanged;
    }
    pub fn observe(&mut self, request: FeedbackRequest) -> Result<u32, RuleError> {
        self.step()?;
        let observation =
            FeedbackServices::observe(self.services, &request, &self.plan.state.memory)?;
        let value = observation.value;
        self.d0 = Some(observation.d0);
        self.plan.state.memory = observation.memory.clone();
        self.plan.effects.push(InventoryInteractionEffect::Feedback(
            FeedbackEffect::Service {
                request,
                observation,
            },
        ));
        Ok(value)
    }
    pub fn trap(&mut self, opcode: u16, arguments: Vec<FeedbackArg>) -> Result<u32, RuleError> {
        self.observe(FeedbackRequest::Toolbox { opcode, arguments })
    }
    pub fn registers(
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
            .push(InventoryInteractionEffect::RegisterToolbox {
                request,
                observation,
            });
        Ok(value)
    }
    pub fn lib(&mut self, entry: u32, arguments: Vec<FeedbackArg>) -> Result<u32, RuleError> {
        self.step()?;
        let request = LibraryRequest { entry, arguments };
        let observation = self.services.library(&request, &self.plan.state.memory)?;
        let value = observation.value;
        self.d0 = Some(observation.d0);
        self.plan.state.memory = observation.memory.clone();
        self.plan.effects.push(InventoryInteractionEffect::Library {
            request,
            observation,
        });
        Ok(value)
    }
    pub fn returned_word(&self) -> Result<u16, RuleError> {
        match self.plan.native_return {
            PromptReturn::Full(v) => Ok(v as u16),
            PromptReturn::LowWord(v) => Ok(v),
            _ => Err(RuleError::Invalid(
                "inventory child returned no WORD".into(),
            )),
        }
    }
    pub fn feedback(&mut self, child: feedback::FeedbackPlan) {
        self.plan.state.memory = child.memory;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(InventoryInteractionEffect::Feedback),
        );
        if let Some(d0) = child.native_return {
            self.full(d0);
        } else {
            self.unknown();
        }
    }
    pub fn sound(&mut self, id: i16, stack: u32) -> Result<(), RuleError> {
        self.word(stack.wrapping_sub(2), id as u16)?;
        let child = feedback::sound(&self.plan.state.memory, id, self.services)?;
        self.feedback(child);
        Ok(())
    }
    pub fn warn(&mut self, frame: u32, id: i16) -> Result<(), RuleError> {
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
    pub fn prompt(&mut self, entry: u32, frame: u32) -> Result<(), RuleError> {
        self.step()?;
        let limits = PromptLimits {
            maximum_steps: self.remaining.min(self.limits.prompt.maximum_steps),
            ..self.limits.prompt
        };
        let child = match entry {
            0x12382a => prompt::first_tutorial(&self.plan.state, frame, limits, self.services)?,
            0x12512a => prompt::get_choice(&self.plan.state, frame, limits, self.services)?,
            0x14cff6 => prompt::question(&self.plan.state, frame, limits, self.services)?,
            0x15dc34 => prompt::text_box(&self.plan.state, frame, limits, self.services)?,
            0x16043e => prompt::update_controls(&self.plan.state, frame, limits, self.services)?,
            0x133d46 => prompt::submit_music(&self.plan.state, frame, limits, self.services)?,
            0x1500d8 => prompt::reset_text_area(&self.plan.state, frame, limits, self.services)?,
            _ => return Err(RuleError::Invalid("unknown selection prompt child".into())),
        };
        self.check_random(&child.state.random)?;
        self.plan.state = child.state;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(InventoryInteractionEffect::Prompt),
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
    pub fn first(&mut self, frame: u32, first: u16, last: u16) -> Result<(), RuleError> {
        self.long(
            frame.wrapping_add(8),
            u32::from(first) << 16 | u32::from(last),
        )?;
        self.prompt(0x12382a, frame)
    }
    pub fn load_item(&mut self, id: i16) -> Result<(), RuleError> {
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
    pub fn dialog_item(&mut self, dialog: u32, item: u16, rect: u32) -> Result<(), RuleError> {
        self.trap(
            0xa98d,
            vec![L(dialog), W(item), P(0x4434b0), P(0x4434dc), P(rect)],
        )?;
        Ok(())
    }
    pub fn dialog_string(&mut self, frame: u32, item: u16, pointer: u32) -> Result<(), RuleError> {
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
    pub fn button(&mut self, frame: u32, pressed: bool, switch: u16) -> Result<(), RuleError> {
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
                .map(InventoryInteractionEffect::Interface),
        );
        self.full(child.native_return as u32);
        Ok(())
    }
    pub fn runtime(&mut self, entry: u32) -> Result<(), RuleError> {
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
        self.check_random(&context.random)?;
        self.plan.state.random = context.random;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(InventoryInteractionEffect::Runtime),
        );
        match child.result {
            RuntimeResult::Long(value) => self.full(value),
            _ => self.unknown(),
        }
        Ok(())
    }
    pub fn profile(&mut self, frame: u32, race: i16, caste: i16) -> Result<(), RuleError> {
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
                    .push(InventoryInteractionEffect::File(effect)),
            }
        }
        self.unknown();
        Ok(())
    }
    pub fn can_use(
        &mut self,
        frame: u32,
        item: i16,
        race: i16,
        caste: i16,
    ) -> Result<(), RuleError> {
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
                .map(InventoryInteractionEffect::Restriction),
        );
        self.full(child.native_return);
        Ok(())
    }
    pub fn indexed_string(&mut self, resource: u16, item: u16) -> Result<(), RuleError> {
        self.lib(0x400178, vec![P(0x479ed8), W(resource), W(item)])?;
        Ok(())
    }
    pub fn dialog_number(&mut self, frame: u32, item: u16, value: i16) -> Result<(), RuleError> {
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
    pub fn c_string(&self, pointer: u32) -> Result<Vec<u8>, RuleError> {
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
    pub fn load_ammo(&mut self, id: i16) -> Result<(), RuleError> {
        let backing = equipment::item_tables(&self.plan.state.memory, id)?;
        let tables = ItemTables {
            families: std::array::from_fn(|i| backing[i].as_slice()),
        };
        let current = self
            .bytes(0x4536d2, ITEM_BYTES)?
            .try_into()
            .expect("item bytes");
        let child = records::loadammo(&tables, &current, id)?;
        for w in child.writes {
            self.write(w.address, &w.bytes)?;
        }
        self.full(child.native_return as u32);
        Ok(())
    }
    pub fn load_spell2(&mut self, stack: u32, id: i16) -> Result<(), RuleError> {
        self.word(stack.wrapping_sub(2), id as u16)?;
        let school = id / 1000;
        let remainder = id.wrapping_sub(school.wrapping_mul(1000));
        let level = remainder / 100;
        let item = remainder.wrapping_sub(level.wrapping_mul(100));
        let offset = usize::from(school.wrapping_sub(1) as u16) * 2842
            + usize::from(level.wrapping_sub(1) as u16) * 406
            + usize::from(item.wrapping_sub(1) as u16) * records::SPELL_BYTES;
        if offset > 1024 * 1024 {
            return Err(RuleError::Invalid(
                "interaction spell span exceeds one MiB".into(),
            ));
        }
        let table = self.bytes(0x443788, offset + records::SPELL_BYTES)?;
        let child = records::loadspell2(&table, id)?;
        for w in child.record.writes {
            self.write(w.address, &w.bytes)?;
        }
        self.full(child.record.native_return as u32);
        Ok(())
    }
    pub fn icon(&mut self, stack: u32, id: i16) -> Result<(), RuleError> {
        self.word(stack.wrapping_sub(2), id as u16)?;
        let mut context = icon_context(self.plan.state.memory.rect(0x4471f2)?);
        let child = drawing::ploticon2(&mut context, id, self.services)?;
        for effect in child.effects {
            if let DrawingEffect::Write(w) = effect {
                self.write(w.address, &w.bytes)?;
            } else {
                self.plan
                    .effects
                    .push(InventoryInteractionEffect::Drawing(effect));
            }
        }
        self.full(child.native_return as u32);
        Ok(())
    }
    pub fn filename(&mut self, stack: u32, pointer: u32) -> Result<(), RuleError> {
        self.long(stack.wrapping_sub(4), pointer)?;
        let mut context = FileContext {
            scenario_directory: self.c_string(0x44260c)?,
            variant: self.b(0x44274c)?,
            filename: self.bytes(0x4389a4, self.limits.prompt.filename_capacity)?,
            ..FileContext::default()
        };
        let number = self
            .services
            .filename_number(context.variant, &self.plan.state.memory)?;
        let observation = files::FileObservation {
            request: files::FileRequest::NumberToString(i32::from(context.variant)),
            reply: files::FileReply::Pascal(number),
        };
        let name = self.c_string(pointer)?;
        let child = files::get_filename(&mut context, &name, &[observation])?;
        for effect in child.effects {
            match effect {
                FileEffect::Write {
                    destination: FileBuffer::Filename,
                    offset,
                    bytes,
                } => self.write(0x4389a4_u32.wrapping_add(offset as u32), &bytes)?,
                other => self
                    .plan
                    .effects
                    .push(InventoryInteractionEffect::File(other)),
            }
        }
        self.unknown();
        Ok(())
    }
    pub fn sane(&mut self, source: u32, destination: u32, selector: u16) -> Result<(), RuleError> {
        self.trap(0xa9eb, vec![P(source), P(destination), W(selector)])?;
        Ok(())
    }
    pub fn scaled_damage(&mut self, frame: u32) -> Result<(), RuleError> {
        let temporary = frame.wrapping_sub(0x120);
        self.sane(0x4434b6, temporary, 0x200e)?;
        self.sane(frame.wrapping_sub(0x108), temporary, 0x1004)?;
        self.trap(0xa9eb, vec![P(temporary), W(0x16)])?;
        self.sane(temporary, temporary, 0x2810)?;
        self.word(0x4434b6, self.w(frame.wrapping_sub(0x11e))?)
    }
    pub fn nz_number(&mut self, stack: u32, item: u16, value: i16) -> Result<(), RuleError> {
        self.word(stack.wrapping_sub(2), value as u16)?;
        self.word(stack.wrapping_sub(4), item)?;
        self.dialog_item(self.l(0x4394b8)?, item, 0x4434ba)?;
        if value == 0 {
            self.trap(0xa98f, vec![L(self.l(0x4434dc)?), P(0x4021a9)])?;
        } else {
            let formatted = if value > 0 {
                value.wrapping_neg()
            } else {
                value
            };
            self.lib(0x400168, vec![L(i32::from(formatted) as u32), P(0x479ed8)])?;
            if value > 0 {
                self.byte(0x479ed9, b'+')?;
            }
            self.trap(0xa98f, vec![L(self.l(0x4434dc)?), P(0x479ed8)])?;
        }
        self.full(0);
        Ok(())
    }
    pub fn selection(&mut self, entry: u32, frame: u32) -> Result<(), RuleError> {
        self.step()?;
        let limits = crate::recovered_inventory_selection::InventorySelectionLimits {
            maximum_steps: self.remaining,
            prompt: self.limits.prompt,
            equipment: self.limits.equipment,
            maximum_random_draws: self.random_remaining()?,
        };
        let child = match entry {
            0x154a46 => crate::recovered_inventory_selection::show_condition(
                &self.plan.state,
                frame,
                limits,
                self.services,
            )?,
            0x156b26 => crate::recovered_inventory_selection::show_items(
                &self.plan.state,
                frame,
                limits,
                self.services,
            )?,
            _ => {
                return Err(RuleError::Invalid(
                    "unknown interaction selection child".into(),
                ));
            }
        };
        self.check_random(&child.state.random)?;
        self.plan.state = child.state;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(InventoryInteractionEffect::Selection),
        );
        self.plan.flow = child.flow;
        match child.native_return {
            PromptReturn::Full(v) => self.full(v),
            PromptReturn::LowWord(v) => self.low(v),
            PromptReturn::Unchanged => {}
            PromptReturn::NoReturn => {
                self.unknown();
                self.plan.native_return = PromptReturn::NoReturn;
            }
        }
        Ok(())
    }
    pub fn quick_info(&mut self, frame: u32, a2: u32) -> Result<(), RuleError> {
        let child = detail::quick_info(
            &self.plan.state.memory,
            frame,
            a2,
            self.limits.prompt.filename_capacity,
            self.services,
        )?;
        self.plan.state.memory = child.memory;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(InventoryInteractionEffect::Detail),
        );
        match child.native_return {
            Some(v) => self.full(v as u32),
            None => self.unknown(),
        }
        Ok(())
    }
    pub fn append_effect_name(
        &mut self,
        stack: u32,
        destination: u32,
        prefix: u32,
        spell: bool,
    ) -> Result<(), RuleError> {
        self.lib(0x400090, vec![P(prefix), P(destination)])?;
        let (resource, index) = if spell {
            (
                self.w(0x4435c6)?
                    .wrapping_add(1)
                    .wrapping_mul(1000)
                    .wrapping_add(self.w(0x4435c8)?),
                self.w(0x4435ca)?.wrapping_add(1),
            )
        } else {
            (133, self.w(0x453668)?.wrapping_sub(19))
        };
        self.indexed_string(resource, index)?;
        self.lib(0x400180, vec![P(0x479ed8)])?;
        self.lib(0x400098, vec![P(0x479ed8), P(destination)])?;
        self.lib(0x400188, vec![P(destination)])?;
        self.dialog_string(stack.wrapping_sub(38), 48, destination)
    }
    pub fn panel(&mut self, frame: u32, body: i16) -> Result<(), RuleError> {
        self.word(frame.wrapping_add(8), body as u16)?;
        let damage = self.services.damage_frame(&self.plan.state.memory)?;
        let child =
            panel::combat_update2(&self.plan.state.memory, frame, body, damage, self.services)?;
        self.plan.state.memory = child.memory;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(InventoryInteractionEffect::Panel),
        );
        self.full(child.native_return as u32);
        Ok(())
    }
    pub fn equipment(&mut self, entry: u32, frame: u32) -> Result<(), RuleError> {
        let child = match entry {
            0x121152 => equipment::dropitem(
                &self.plan.state.memory,
                &self.plan.state.random,
                frame,
                self.limits.equipment,
                self.services,
            )?,
            0x14e5e4 => equipment::refit(
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
        self.check_random(&child.random)?;
        self.plan.state.random = child.random;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(InventoryInteractionEffect::Equipment),
        );
        if let Some(value) = child.native_return.known_long() {
            self.full(value);
        } else {
            self.d0 = None;
            self.low(child.native_return.low_word);
        }
        Ok(())
    }
    pub fn rand(&mut self, bound: i16) -> Result<i16, RuleError> {
        self.step()?;
        let capacity = usize::try_from(self.random_remaining()?).map_err(|_| {
            RuleError::Invalid("interaction random budget exceeds host usize".into())
        })?;
        if capacity == 0 {
            return Err(RuleError::Invalid(
                "interaction RNG budget exhausted".into(),
            ));
        }
        let mut context = runtime::RuntimeContext::new(self.plan.state.random.clone(), capacity);
        let child = runtime::rand(&mut context, bound)?;
        self.check_random(&context.random)?;
        self.plan.state.random = context.random;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(InventoryInteractionEffect::Runtime),
        );
        let RuntimeResult::Word { value, native_d0 } = child.result else {
            return Err(RuleError::Invalid(
                "unexpected interaction Rand result".into(),
            ));
        };
        self.full(native_d0);
        Ok(value)
    }
    pub fn selection_text(
        &mut self,
        stack: u32,
        destination: u32,
        id: i16,
    ) -> Result<(), RuleError> {
        self.long(stack.wrapping_sub(4), destination)?;
        self.word(stack.wrapping_sub(6), id as u16)?;
        let base = rules::selection_base(id);
        self.word(0x4471e8, base as u16)?;
        self.low(base as u16);
        self.lib(
            0x400178,
            vec![
                P(destination),
                W(base.wrapping_add(1) as u16),
                W((id as u16).wrapping_sub(self.w(0x4471e8)?).wrapping_add(1)),
            ],
        )?;
        self.lib(0x400180, vec![P(destination)])?;
        Ok(())
    }
    pub fn ammo_menu_text(&mut self, frame: u32, stack: u32, index: i16) -> Result<(), RuleError> {
        let text = frame.wrapping_sub(0x100);
        let quantity = frame.wrapping_sub(0x200);
        self.selection_text(stack, text, self.sw(0x453738)?)?;
        self.lib(0x400090, vec![P(text), P(0x479ed8)])?;
        self.lib(0x400098, vec![P(0x40316e), P(0x479ed8)])?;
        self.lib(
            0x400168,
            vec![
                L(self.l(crate::recovered_inventory_interaction::index(
                    0x42dbe4, index, 4,
                ))?),
                P(quantity),
            ],
        )?;
        self.lib(0x400180, vec![P(quantity)])?;
        self.lib(0x400098, vec![P(quantity), P(0x479ed8)])?;
        let suffix = if self.w(0x4536e4)? != 0 {
            0x403166
        } else {
            0x403160
        };
        self.lib(0x400098, vec![P(suffix), P(0x479ed8)])?;
        self.lib(0x400188, vec![P(0x479ed8)])?;
        Ok(())
    }
    pub fn adopt_interaction(&mut self, child: InventoryInteractionPlan) -> Result<(), RuleError> {
        self.check_random(&child.state.random)?;
        self.plan.state = child.state;
        self.plan.effects.extend(child.effects);
        self.plan.flow = child.flow;
        match child.native_return {
            PromptReturn::Full(v) => self.full(v),
            PromptReturn::LowWord(v) => self.low(v),
            PromptReturn::Unchanged => {}
            PromptReturn::NoReturn => {
                self.unknown();
                self.plan.native_return = PromptReturn::NoReturn;
            }
        }
        Ok(())
    }
    pub fn copy(&mut self, source: u32, destination: u32, length: usize) -> Result<(), RuleError> {
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
    pub fn character(&mut self, entry: u32, frame: u32) -> Result<(), RuleError> {
        self.step()?;
        let child = match entry {
            0x120872 => character::display(
                &self.plan.state.memory,
                frame,
                self.remaining,
                self.services,
            )?,
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
                .map(InventoryInteractionEffect::Character),
        );
        self.full(child.native_return);
        Ok(())
    }
    pub fn movement(&mut self, frame: u32, member: u16) -> Result<(), RuleError> {
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
        self.plan
            .effects
            .push(InventoryInteractionEffect::Movement {
                member,
                result: child,
            });
        Ok(())
    }
    pub fn compact(&mut self, frame: u32, member: i16) -> Result<(), RuleError> {
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
    pub fn change(&mut self, frame: u32, deferred: u32) -> Result<(), RuleError> {
        self.long(frame.wrapping_add(8), deferred)?;
        let limits = InventoryEditLimits {
            maximum_steps: self.remaining,
            equipment: self.limits.equipment,
            maximum_random_draws: self.random_remaining()?,
        };
        let child = edit::change(
            &self.plan.state.memory,
            &self.plan.state.random,
            frame,
            limits,
            self.services,
        )?;
        self.plan.state.memory = child.memory;
        self.check_random(&child.random)?;
        self.plan.state.random = child.random;
        self.plan.flow = child.flow;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(InventoryInteractionEffect::InventoryEdit),
        );
        if let Some(value) = child.native_return {
            self.full(value);
        } else {
            self.unknown();
        }
        Ok(())
    }
    pub fn numeric(&mut self, stack: u32, value: i16) -> Result<(), RuleError> {
        self.word(stack.wrapping_sub(2), value as u16)?;
        self.lib(0x400168, vec![L(i32::from(value) as u32), P(0x479ed8)])?;
        let string =
            PascalString::from_raw(&self.bytes(0x479ed8, usize::from(self.b(0x479ed8)?) + 1)?)?;
        let _child = text::string(&mut TextContext::default(), value, &mut FixedNumber(string))?;
        self.trap(0xa884, vec![P(0x479ed8)])?;
        Ok(())
    }
    pub fn draw_effects(&mut self, effects: Vec<DrawingEffect>) -> Result<(), RuleError> {
        for effect in effects {
            if let DrawingEffect::Write(w) = effect {
                self.write(w.address, &w.bytes)?;
            } else {
                self.plan
                    .effects
                    .push(InventoryInteractionEffect::Drawing(effect));
            }
        }
        Ok(())
    }
    pub fn icon3(&mut self, stack: u32, id: i16, rectangle: u32) -> Result<(), RuleError> {
        self.long(stack.wrapping_sub(4), self.l(rectangle.wrapping_add(4))?)?;
        self.long(stack.wrapping_sub(8), self.l(rectangle)?)?;
        self.word(stack.wrapping_sub(10), id as u16)?;
        let rect = self.plan.state.memory.rect(stack.wrapping_sub(8))?;
        let mut context = icon_context(rect);
        let child = drawing::ploticon3(&mut context, id, rect, self.services)?;
        self.draw_effects(child.effects)?;
        self.full(child.native_return as u32);
        Ok(())
    }
    pub fn plot_item(&mut self, stack: u32, id: i16) -> Result<(), RuleError> {
        self.word(stack.wrapping_sub(2), id as u16)?;
        let backing = equipment::item_tables(&self.plan.state.memory, id)?;
        let tables = ItemTables {
            families: std::array::from_fn(|i| backing[i].as_slice()),
        };
        let current = self
            .bytes(0x45363c, ITEM_BYTES)?
            .try_into()
            .expect("item bytes");
        let loaded = records::loaditemnotype(&tables, &current, id)?;
        let selection =
            rules::selection_base(i16::from_be_bytes([loaded.bytes[102], loaded.bytes[103]]));
        let mut context = icon_context(self.plan.state.memory.rect(0x4471f2)?);
        context.item_work = current;
        context.window = self.l(0x439404)?;
        context.item_window = self.l(0x4393fc)?;
        context.half_width_input = self.sw(0x40856c)?;
        context.omit_banner = self.b(0x442756)?;
        if context.omit_banner == 0 {
            context.banner_rect = self.plan.state.memory.rect(0x4471ac)?;
        }
        if id == 0 && context.window == context.item_window {
            context.active_member = self.b(0x44273e)? as i8;
        }
        context.show_item_background = self.b(0x44275a)?;
        context.text_in_other_port = self.b(0x442750)?;
        context.erase_pattern = self.l(0x4358da)?;
        context.default_font = self.sw(0x402f3c)?;
        if context.text_in_other_port != 0 {
            context.text_port = self.l(0x438798)?;
        }
        context.selection = selection;
        let child = drawing::ploticon(&mut context, &tables, id, self.services)?;
        for effect in child.effects {
            if matches!(effect, DrawingEffect::GetIndexedString { .. }) {
                self.word(0x4471e8, selection as u16)?;
            }
            self.draw_effects(vec![effect])?;
        }
        self.full(child.native_return as u32);
        Ok(())
    }
    pub fn erase_shop(&mut self, frame: u32, name: bool, selector: i16) -> Result<(), RuleError> {
        self.word(frame.wrapping_add(8), selector as u16)?;
        let context = crate::recovered_shop_ui::ShopContext {
            width: self.sw(0x40856c)?,
            window: 0,
            erase_pattern: self.l(0x4358ce)?,
            normal_pattern: if name { 0 } else { self.l(0x4358da)? },
            credits: 0,
            text_work: PascalString::default(),
        };
        let child = if name {
            crate::recovered_shop_ui::eraseshopname(&context, selector)
        } else {
            crate::recovered_shop_ui::eraseshopstats(&context, selector)
        };
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(InventoryInteractionEffect::Shop),
        );
        self.full(child.native_return as u32);
        Ok(())
    }
    pub fn shop_credit(&mut self) -> Result<(), RuleError> {
        let mut context = crate::recovered_shop_ui::ShopContext {
            width: self.sw(0x40856c)?,
            window: self.l(0x438788)?,
            erase_pattern: self.l(0x4358ce)?,
            normal_pattern: self.l(0x4358da)?,
            credits: self.l(0x442788)? as i32,
            text_work: PascalString::default(),
        };
        let child = crate::recovered_shop_ui::shop_credit_display(&mut context, self.services)?;
        for effect in child.effects {
            if let crate::recovered_shop_ui::ShopEffect::Drawing(DrawingEffect::Write(w)) = effect {
                self.write(w.address, &w.bytes)?;
            } else {
                self.plan
                    .effects
                    .push(InventoryInteractionEffect::Shop(effect));
            }
        }
        self.full(child.native_return as u32);
        Ok(())
    }
    pub fn wings(&mut self, frame: u32) -> Result<(), RuleError> {
        let child = crate::recovered_item_ui::update_shop_wings(
            &self.plan.state.memory,
            frame,
            self.services,
        )?;
        self.plan.state.memory = child.memory;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(InventoryInteractionEffect::ItemUi),
        );
        match child.native_return {
            Some(v) => self.full(v as u32),
            None => self.unknown(),
        }
        Ok(())
    }
    pub fn update_shop(&mut self, frame: u32, incoming: i16) -> Result<(), RuleError> {
        let child = crate::recovered_status_panels::update_shop(
            &self.plan.state.memory,
            &self.plan.state.random,
            frame,
            incoming,
            self.remaining as usize,
            self.services,
        )?;
        self.plan.state.memory = child.memory;
        self.check_random(&child.random)?;
        self.plan.state.random = child.random;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(InventoryInteractionEffect::Status),
        );
        match child.native_return.known_long() {
            Some(v) => self.full(v),
            None => {
                self.unknown();
                if let Some(v) = child.native_return.low_word {
                    self.low(v);
                }
            }
        }
        Ok(())
    }
    pub fn show_charge(&mut self, frame: u32) -> Result<(), RuleError> {
        let child = crate::recovered_equipment_ui::show_charge(
            &self.plan.state.memory,
            frame,
            self.services,
        )?;
        self.plan.state.memory = child.memory;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(InventoryInteractionEffect::EquipmentUi),
        );
        self.full(child.native_return);
        Ok(())
    }
    pub fn cost(&mut self, stack: u32, mode: i16) -> Result<(), RuleError> {
        self.word(stack.wrapping_sub(2), mode as u16)?;
        let child = crate::recovered_economy::itemcost(crate::recovered_economy::ItemCostInput {
            cost: self.l(0x4536ca)? as i32,
            percent: self.sw(0x478e40)?,
            mode,
        })?;
        self.long(0x4536ca, child.cost as u32)?;
        self.plan
            .effects
            .push(InventoryInteractionEffect::Cost(child));
        self.unknown();
        Ok(())
    }
    pub fn numeric_long(&mut self, stack: u32, value: i32) -> Result<(), RuleError> {
        self.long(stack.wrapping_sub(4), value as u32)?;
        self.lib(0x400168, vec![L(value as u32), P(0x479ed8)])?;
        self.trap(0xa884, vec![P(0x479ed8)])?;
        Ok(())
    }
    pub fn add_crate(&mut self, stack: u32, node: u32) -> Result<(), RuleError> {
        for offset in [18_u32, 14, 10, 6, 2] {
            self.long(
                stack.wrapping_sub(22 - offset),
                self.l(node.wrapping_add(offset))?,
            )?;
        }
        self.word(stack.wrapping_sub(22), self.w(node)?)?;
        let record: [u8; 22] = self
            .bytes(stack.wrapping_sub(22), 22)?
            .try_into()
            .expect("inventory node");
        let current = self.bytes(0x4285f4, inventory::BANK_BYTES)?;
        let child = inventory::addtocrate(&current, &record)?;
        for effect in child.effects {
            match effect {
                inventory::InventoryEffect::Write(w) => self.write(w.address, &w.bytes)?,
                inventory::InventoryEffect::Sound(v) => {
                    self.sound(v, stack.wrapping_sub(22 + 8 + 4))?
                }
                _ => return Err(RuleError::Invalid("unexpected crate child effect".into())),
            }
        }
        self.full(child.native_return as u32);
        Ok(())
    }
    pub fn install(&mut self, frame: u32) -> Result<(), RuleError> {
        let request = crate::recovered_install::InstallRequest {
            source_member: self.w(frame.wrapping_add(8))?,
            source_slot: self.w(frame.wrapping_add(12))?,
            destination_member: self.w(frame.wrapping_add(10))?,
            destination_slot: self.w(frame.wrapping_add(14))?,
            item_id: self.sw(frame.wrapping_add(16))?,
        };
        let start = 0x4046e6_u32;
        let end = 0x478288_u32;
        let current = InstallMemory {
            base_address: start,
            bytes: self.bytes(start, (end - start) as usize)?,
        };
        let backing = equipment::item_tables(&self.plan.state.memory, request.item_id)?;
        let tables = ItemTables {
            families: std::array::from_fn(|i| backing[i].as_slice()),
        };
        let child = crate::recovered_install::installitem_complete(
            &current,
            &self.plan.state.random,
            &tables,
            request,
            crate::recovered_random_item::RandomItemLimits {
                maximum_draws: self.random_remaining()?,
            },
        )?;
        self.check_random(&child.random)?;
        self.plan.state.random = child.random;
        for effect in child.effects {
            if let crate::recovered_random_item::RandomItemEffect::Write(w) = effect {
                self.write(w.address, &w.bytes)?;
            } else {
                self.plan
                    .effects
                    .push(InventoryInteractionEffect::RandomItem(effect));
            }
        }
        self.full(child.native_return as u32);
        Ok(())
    }
    pub fn refresh_wings(&mut self, stack: u32) -> Result<(), RuleError> {
        self.long(stack.wrapping_sub(4), 0xfffefffe)?;
        self.wings(stack.wrapping_sub(12))
    }
    pub fn finish_move(&mut self, stack: u32) -> Result<(), RuleError> {
        self.refresh_wings(stack)?;
        self.full(0);
        Ok(())
    }
    pub fn weight(&self) -> Result<i16, RuleError> {
        Ok(self
            .sw(0x453660)?
            .wrapping_mul(self.sw(0x453650)?)
            .wrapping_add(self.sw(0x45364c)?))
    }
    pub fn right_of(&self, n: i16) -> Result<bool, RuleError> {
        Ok(self.sw(0x4471f0)? > half(self.sw(0x40856c)?).wrapping_add(n))
    }
    pub fn limited_pack(&self, base: u32) -> Result<bool, RuleError> {
        Ok(self.w(base.wrapping_add(0x1a))? == 30
            || self
                .weight()?
                .wrapping_add(self.sw(base.wrapping_add(0x9d6))?) as u16
                > self.w(base.wrapping_add(0x9d8))?)
    }
    #[allow(clippy::too_many_arguments)]
    pub fn render_pack_row(
        &mut self,
        frame: u32,
        stack: u32,
        id: i16,
        base: u32,
        slot: u16,
        member_selector: u32,
        merged: bool,
    ) -> Result<(), RuleError> {
        let saved = frame.wrapping_sub(0x9e);
        self.copy(0x45363c, saved, ITEM_BYTES)?;
        self.plot_item(stack, id)?;
        self.copy(saved, 0x45363c, ITEM_BYTES)?;
        if merged {
            self.byte(0x44275a, 0)?;
        }
        self.byte(0x41f708, slot as u8)?;
        self.byte(0x41f709, self.b(member_selector)?)?;
        let node = temp_node(base, self.sb(0x41f708)? as u16);
        self.copy(node, 0x41fc48, 22)?;
        let child = stack.wrapping_sub(14);
        let node = temp_node(base, slot);
        self.word(child.wrapping_add(10), self.w(node.wrapping_add(4))?)?;
        self.word(child.wrapping_add(8), self.w(node.wrapping_add(2))?)?;
        self.show_charge(child)
    }
    pub fn shift_pack(&mut self, base: u32, count: u32, start: u16) -> Result<(), RuleError> {
        self.word(0x4434aa, start)?;
        while self.sw(0x4434aa)? < self.sw(count)? {
            self.step()?;
            self.copy(
                temp_node(base, self.w(0x4434aa)?.wrapping_add(1)),
                temp_node(base, self.w(0x4434aa)?),
                22,
            )?;
            self.add(0x4434aa, 1)?;
        }
        Ok(())
    }
    #[allow(clippy::too_many_arguments)]
    pub fn merge_pack(
        &mut self,
        frame: u32,
        stack: u32,
        id: i16,
        base: u32,
        count: u32,
        scroll: i16,
        member: u32,
    ) -> Result<bool, RuleError> {
        if self.w(0x453660)? == 0 {
            return Ok(false);
        }
        self.word(0x4434aa, 0)?;
        while self.sw(0x4434aa)? < self.sw(count)? {
            self.step()?;
            let node = temp_node(base, self.w(0x4434aa)?);
            if self.w(node)? == self.w(0x4536a2)? {
                self.add(node.wrapping_add(2), self.sw(0x453650)?)?;
                if self.sw(0x4434aa)?.wrapping_sub(scroll) < 9
                    && self.sw(0x4434aa)?.wrapping_sub(self.sw(0x4471e0)?) > -1
                {
                    self.word(
                        0x4471f2,
                        self.sw(0x4434aa)?
                            .wrapping_sub(scroll)
                            .wrapping_mul(40)
                            .wrapping_add(16) as u16,
                    )?;
                    self.byte(0x44275a, self.b(node.wrapping_add(9))?)?;
                    self.render_pack_row(frame, stack, id, base, self.w(0x4434aa)?, member, true)?;
                }
                return Ok(true);
            }
            self.add(0x4434aa, 1)?;
        }
        Ok(false)
    }
    pub fn insert_pack(
        &mut self,
        base: u32,
        count: u32,
        scroll: i16,
        maximum: i16,
        destination: &mut u16,
    ) -> Result<(), RuleError> {
        if scroll >= maximum {
            return Ok(());
        }
        self.word(0x4434aa, self.w(count)?)?;
        while scroll.wrapping_add(9) < self.sw(0x4434aa)? {
            self.step()?;
            self.copy(
                temp_node(base, self.w(0x4434aa)?.wrapping_sub(1)),
                temp_node(base, self.w(0x4434aa)?),
                22,
            )?;
            self.add(0x4434aa, -1)?;
        }
        *destination = scroll.wrapping_add(9) as u16;
        Ok(())
    }
    pub fn remove(&mut self, frame: u32) -> Result<(), RuleError> {
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
                .map(InventoryInteractionEffect::Restriction),
        );
        self.full(child.native_return);
        Ok(())
    }
    pub fn pict(&mut self, frame: u32, id: u16, rect: u32) -> Result<(), RuleError> {
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
    pub fn press_rect(
        &mut self,
        frame: u32,
        kind: u16,
        phase: u16,
        sound: i16,
        rectangle: u32,
    ) -> Result<(), RuleError> {
        self.long(frame.wrapping_add(18), self.l(rectangle.wrapping_add(4))?)?;
        self.long(frame.wrapping_add(14), self.l(rectangle)?)?;
        self.word(frame.wrapping_add(12), sound as u16)?;
        self.long(
            frame.wrapping_add(8),
            u32::from(kind) << 16 | u32::from(phase),
        )?;
        let child = feedback::pressbutton(
            &self.plan.state.memory,
            frame,
            kind as i16,
            phase as i16,
            sound,
            self.services,
        )?;
        self.feedback(child);
        Ok(())
    }
    pub fn press_control(
        &mut self,
        stack: u32,
        control: u32,
        kind: u16,
        phase: u16,
        sound: i16,
    ) -> Result<(), RuleError> {
        let bounds = self.l(self.l(control)?)?.wrapping_add(8);
        self.press_rect(stack.wrapping_sub(22), kind, phase, sound, bounds)
    }
    pub fn save_shop(&mut self) -> Result<(), RuleError> {
        let limits = crate::recovered_storage::StorageLimits {
            filename_capacity: self.limits.prompt.filename_capacity,
            maximum_iterations: self.remaining as usize,
            maximum_polls: self.limits.prompt.maximum_polls,
        };
        let child =
            crate::recovered_storage::save_shop(&self.plan.state.memory, limits, self.services)?;
        self.plan.state.memory = child.memory;
        self.plan.flow = child.flow;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(InventoryInteractionEffect::Storage),
        );
        match child.native_return {
            Some(v) => self.full(v),
            None => self.unknown(),
        }
        Ok(())
    }
    pub fn view_character(&mut self, frame: u32) -> Result<(), RuleError> {
        self.step()?;
        let child = upgrade::view_character(
            &self.plan.state,
            frame,
            PromptLimits {
                maximum_steps: self.remaining.min(self.limits.prompt.maximum_steps),
                ..self.limits.prompt
            },
            self.services,
        )?;
        self.check_random(&child.state.random)?;
        self.plan.state = child.state;
        self.plan.flow = child.flow;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(InventoryInteractionEffect::Upgrade),
        );
        match child.native_return {
            PromptReturn::Full(v) => self.full(v),
            PromptReturn::LowWord(v) => self.low(v),
            PromptReturn::Unchanged => {}
            PromptReturn::NoReturn => {
                self.unknown();
                self.plan.native_return = PromptReturn::NoReturn;
            }
        }
        Ok(())
    }
    pub fn track_control(
        &mut self,
        control: u32,
        point: u32,
        callback: u32,
    ) -> Result<u16, RuleError> {
        let request = FeedbackRequest::Toolbox {
            opcode: 0xa968,
            arguments: vec![L(control), L(point), P(callback)],
        };
        if callback != 0 {
            let invocations = self
                .services
                .track_control_invocations(&request, &self.plan.state.memory)?;
            self.plan
                .effects
                .push(InventoryInteractionEffect::TrackControl {
                    request: request.clone(),
                    invocations: invocations.clone(),
                });
            for call in invocations {
                self.step()?;
                self.word(call.native_frame.wrapping_add(8), call.part)?;
                self.long(call.native_frame.wrapping_add(10), call.control)?;
                self.update_shop(call.native_frame, call.incoming_d3)?;
            }
        }
        Ok(self.observe(request)? as u16)
    }
    pub fn delay(&mut self, stack: u32, duration: i16) -> Result<(), RuleError> {
        self.word(stack.wrapping_sub(2), duration as u16)?;
        let default = if duration == 0 { self.sw(0x439568)? } else { 0 };
        let polls = self.limits.prompt.maximum_polls;
        let child = crate::recovered_timing::delay(
            duration,
            default,
            &mut Clock {
                execution: self,
                remaining: polls,
            },
        )?;
        self.full(child.native_result);
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(InventoryInteractionEffect::Timing),
        );
        Ok(())
    }
    pub fn hold_warning(&mut self, frame: u32, stack: u32) -> Result<(), RuleError> {
        if self.w(0x439582)? == 0 {
            self.warn(stack.wrapping_sub(10), 60)?;
            return Ok(());
        }
        let start = self.trap(0xa975, vec![])?;
        while self.trap(0xa973, vec![])? as u8 != 0 {
            self.step()?;
            self.long(frame.wrapping_sub(0x16), self.l(0x2f0)?.wrapping_add(start))?;
            if self.trap(0xa975, vec![])? > self.l(frame.wrapping_sub(0x16))? {
                self.warn(stack.wrapping_sub(10), 60)?;
                self.delay(stack.wrapping_sub(2), 20)?;
            }
        }
        Ok(())
    }
    pub fn choose_move(&mut self, frame: u32) -> Result<(), RuleError> {
        self.step()?;
        let child = move_icon(&self.plan.state, frame, self.child_limits()?, self.services)?;
        self.adopt_interaction(child)?;
        Ok(())
    }
    pub fn pool(&mut self) -> Result<(), RuleError> {
        let mut member = 0_i16;
        while member <= self.sb(0x44273a)? {
            self.step()?;
            let address = record(member as u16);
            let mut warrior = Warrior {
                name: String::new(),
                original_sha256: String::new(),
                bytes: self.bytes(address, WARRIOR_BYTES)?,
            };
            let mut total = self.l(0x442788)? as i32;
            crate::party::pool_credits(std::slice::from_mut(&mut warrior), &mut total)?;
            self.long(0x442788, total as u32)?;
            self.word(address.wrapping_add(0x9da), 0)?;
            member = member.wrapping_add(1);
        }
        self.full(0);
        Ok(())
    }
}
struct FixedNumber(PascalString);
impl NumericServices for FixedNumber {
    fn number_to_string(&mut self, _: i32) -> Result<PascalString, RuleError> {
        Ok(self.0.clone())
    }
    fn string_to_number(&mut self, _: &PascalString) -> Result<i32, RuleError> {
        Err(RuleError::Invalid(
            "unused interaction numeric input".into(),
        ))
    }
}
pub fn index(base: u32, value: i16, stride: i32) -> u32 {
    base.wrapping_add((i32::from(value).wrapping_mul(stride)) as u32)
}
pub fn record(member: u16) -> u32 {
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
pub fn execution<'a, S: ?Sized>(
    state: &InventoryInteractionState,
    limits: InventoryInteractionLimits,
    services: &'a mut S,
) -> Execution<'a, S> {
    Execution {
        plan: InventoryInteractionPlan {
            state: state.clone(),
            effects: vec![],
            native_return: PromptReturn::Unchanged,
            flow: LoadFlow::Returned,
            qualification:
                InventoryInteractionQualification::SourcePortWithObservedBoundariesAndCapturedChildren,
        },
        services,
        resolver: None,
        limits,
        remaining: limits.maximum_steps,
        initial_random_draws: state.random.draws,
        d0: None,
    }
}

pub fn icon_context(rect: BodyRect) -> DrawingContext {
    // ploticon3 reads none of these unrelated fields; its scratch starts by clearing.
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

/// showiteminfo, 0x1555f2. Modal redirects retain the captured caller member.
pub fn show_item_info(
    state: &InventoryInteractionState,
    frame: u32,
    limits: InventoryInteractionLimits,
    services: &mut (impl InventoryInteractionServices + ?Sized),
) -> Result<InventoryInteractionPlan, RuleError> {
    let mut e = execution(state, limits, services);
    let stack = frame.wrapping_sub(0x13c);
    let at = |offset| frame.wrapping_sub(offset);
    let mut member = e.w(frame.wrapping_add(8))?;
    let mut slot = e.w(frame.wrapping_add(10))?;
    let mut item = e.w(frame.wrapping_add(12))?;
    let window = e.trap(0xa97c, vec![W(149), L(0), L(u32::MAX)])?;
    e.trap(0xa873, vec![L(window)])?;
    e.trap(
        0xa91b,
        vec![
            L(window),
            W(e.w(0x4395a4)?
                .wrapping_add(285)
                .wrapping_add(e.w(0x40856c)?)),
            W(e.w(0x4395a2)?),
            B(0),
        ],
    )?;
    e.trap(0xa915, vec![L(window)])?;
    e.trap(0xaa0b, vec![L(e.l(0x4358ce)?)])?;
    e.trap(0xa981, vec![L(window)])?;
    e.trap(0xa88a, vec![W(10)])?;
    e.trap(0xa887, vec![W(10)])?;
    e.long(0x4394b8, window)?;
    'redraw: loop {
        e.step()?;
        e.word(0x4434aa, 7)?;
        while e.sw(0x4434aa)? < 52 {
            e.step()?;
            e.dialog_string(stack.wrapping_sub(14), e.w(0x4434aa)?, 0x4010e7)?;
            e.add(0x4434aa, 1)?;
        }
        let loaded;
        if member == u16::MAX {
            e.word(stack.wrapping_sub(2), item)?;
            e.load_item(item as i16)?;
            e.word(stack.wrapping_sub(4), 0)?;
            e.load_ammo(0)?;
            e.word(at(0x112), e.w(0x453650)?)?;
            e.word(at(0x10c), e.w(0x453650)?)?;
            e.word(
                at(0x10e),
                e.w(0x453698)?.wrapping_add(e.sb(0x45368c)? as u16),
            )?;
            loaded = false;
        } else {
            let address = node(member, slot);
            e.word(stack.wrapping_sub(2), e.w(address)?)?;
            e.load_item(e.sw(address)?)?;
            loaded = e.w(address.wrapping_add(4))? != 0;
            let ammo = if loaded {
                e.sw(address.wrapping_add(4))?
            } else {
                0
            };
            e.word(stack.wrapping_sub(2), ammo as u16)?;
            e.load_ammo(ammo)?;
            e.add(0x453694, e.sb(address.wrapping_add(14))?)?;
            e.byte(
                0x45368d,
                e.b(address.wrapping_add(15))?.wrapping_add(e.b(0x453723)?),
            )?;
            e.byte(
                0x45363f,
                e.b(0x45363f)?.wrapping_add(e.b(address.wrapping_add(16))?),
            )?;
            e.byte(
                0x453640,
                e.b(0x453640)?.wrapping_add(e.b(address.wrapping_add(17))?),
            )?;
            e.byte(
                0x453691,
                e.b(address.wrapping_add(18))?.wrapping_add(e.b(0x453727)?),
            )?;
            e.add(0x45369a, e.sb(address.wrapping_add(19))?)?;
            let bonus = e
                .sb(address.wrapping_add(20))?
                .wrapping_add(e.sb(0x453721)?)
                .wrapping_add(e.sw(0x45370e)?);
            e.add(0x453678, bonus)?;
            e.word(
                at(0x10e),
                e.w(0x453698)?
                    .wrapping_add(e.sb(address.wrapping_add(21))? as u16),
            )?;
            e.word(at(0x112), e.w(address.wrapping_add(2))?)?;
            e.word(at(0x10c), e.w(address.wrapping_add(6))?)?;
        }
        e.trap(0xa873, vec![L(window)])?;
        e.long(0x4394b8, window)?;
        e.trap(0xa862, vec![L(69)])?;
        e.trap(0xa887, vec![W(10)])?;
        if e.w(0x45365c)? != 0 {
            if loaded {
                e.dialog_string(stack.wrapping_sub(14), 20, 0x4010d8)?;
                e.word(stack.wrapping_sub(8), e.w(0x453738)?)?;
                let base = rules::selection_base(e.sw(0x453738)?);
                e.word(0x4471e8, base as u16)?;
                e.low(base as u16);
                e.indexed_string(
                    base.wrapping_add(1) as u16,
                    e.w(0x453738)?.wrapping_sub(e.w(0x4471e8)?).wrapping_add(1),
                )?;
            } else {
                e.dialog_string(stack.wrapping_sub(14), 20, 0x4010c6)?;
                e.indexed_string(142, e.w(0x45365c)?)?;
            }
        } else if e.w(0x45365e)? != 0 {
            e.dialog_string(stack.wrapping_sub(14), 20, 0x4010b0)?;
            e.indexed_string(142, e.w(0x45365e)?)?;
        } else if e.b(0x453693)? != 0 {
            e.dialog_string(stack.wrapping_sub(14), 20, 0x40109a)?;
            e.indexed_string(0xd508, e.sb(0x453693)?.wrapping_add(1) as u16)?;
        } else {
            e.dialog_string(stack.wrapping_sub(14), 20, 0x401084)?;
            e.indexed_string(0xd508, e.w(0x453656)?.wrapping_add(1))?;
        }
        e.dialog_string(stack.wrapping_sub(20), 21, 0x479ed8)?;
        e.dialog_item(window, 4, 0x4471f2)?;
        e.trap(0xa8a3, vec![P(0x4471f2)])?;
        if loaded {
            e.icon(stack, e.sw(0x4536dc)?)?;
        }
        e.dialog_item(window, 54, 0x4471f2)?;
        e.trap(0xa8a3, vec![P(0x4471f2)])?;
        e.word(at(0x10a), e.w(0x453676)?)?;
        if loaded && e.w(0x45370c)? != 0 {
            e.word(at(0x10a), e.w(0x45370c)?)?;
        }
        if e.w(at(0x10a))? == 0 && e.w(0x453674)? != 0 {
            e.word(at(0x10a), 1)?;
        }
        if e.w(at(0x10a))? != 0 {
            e.icon(stack, e.sw(at(0x10a))?.wrapping_add(9999))?;
        }
        e.dialog_item(window, 3, 0x4471f2)?;
        e.trap(0xa8a3, vec![P(0x4471f2)])?;
        e.icon(stack, e.sw(0x453646)?)?;
        e.dialog_item(window, 6, 0x4471f2)?;
        e.word(at(0x110), e.w(0x4536a2)?)?;
        let file = if e.sw(0x4536a2)? > 799 {
            e.add(at(0x110), -800)?;
            e.filename(stack, 0x401078)?;
            e.lib(0x400070, vec![P(0x401074), P(0x4389a4)])?
        } else {
            e.lib(0x400070, vec![P(0x401074), P(0x40105c)])?
        };
        e.long(0x4434a6, file)?;
        e.lib(
            0x400078,
            vec![
                W(0),
                L((i32::from(e.sw(at(0x110))?) as u32).wrapping_shl(8)),
                L(e.l(0x4434a6)?),
            ],
        )?;
        e.lib(0x400048, vec![L(e.l(0x4434a6)?), L(1), L(256), P(0x479ed8)])?;
        e.lib(0x400068, vec![L(e.l(0x4434a6)?)])?;
        e.dialog_string(stack.wrapping_sub(44), 6, 0x479ed8)?;
        e.trap(0xaa14, vec![P(0x44337a)])?;
        if e.sw(0x453666)? > 19 {
            e.indexed_string(136, e.w(0x453666)?.wrapping_sub(19))?;
            e.dialog_string(stack.wrapping_sub(14), 48, 0x479ed8)?;
        }
        if e.sw(0x453666)? == -10 {
            e.append_effect_name(stack, at(0x104), 0x401046, false)?;
        }
        if e.sw(0x453668)? > 1100 && e.sw(0x453668)? < 4000 {
            e.load_spell2(stack, e.sw(0x453668)?)?;
            e.append_effect_name(stack.wrapping_sub(2), at(0x104), 0x40103c, true)?;
        }
        if e.w(0x453666)? == 122 {
            let pointer = match e.w(0x453668)? {
                1 => Some(0x40101e),
                2 => Some(0x400ffe),
                3 => Some(0x400fde),
                4 => Some(0x400fbe),
                _ => None,
            };
            if let Some(pointer) = pointer {
                e.dialog_string(stack.wrapping_sub(14), 48, pointer)?;
            }
        }
        for modifier in [0x45366a, 0x45366c] {
            if e.sw(modifier)? > 0 && e.sw(modifier)? < 16 {
                e.lib(0x400090, vec![P(0x4010e7), P(at(0x104))])?;
                if e.sw(0x45366e)? > 0 {
                    e.lib(0x400090, vec![P(0x400fbc), P(at(0x104))])?;
                }
                e.lib(
                    0x400168,
                    vec![L(i32::from(e.sw(0x45366e)?) as u32), P(0x479ed8)],
                )?;
                e.lib(0x400180, vec![P(0x479ed8)])?;
                e.lib(0x400098, vec![P(0x479ed8), P(at(0x104))])?;
                e.lib(0x400098, vec![P(0x400fb6), P(at(0x104))])?;
                e.indexed_string(1201, e.w(modifier)?)?;
                e.lib(0x400180, vec![P(0x479ed8)])?;
                e.lib(0x400098, vec![P(0x479ed8), P(at(0x104))])?;
                e.lib(0x400188, vec![P(at(0x104))])?;
                e.dialog_string(stack.wrapping_sub(50), 48, at(0x104))?;
            }
        }
        e.trap(0xa862, vec![L(69)])?;
        let modifier = if (member as i16) > -1 {
            i32::from(e.sb(node(member, slot).wrapping_add(18))?)
        } else {
            0
        };
        e.long(at(0x120), 100_u32.wrapping_add(modifier as u32))?;
        e.sane(at(0x120), at(0x120), 0x280e)?;
        e.sane(at(0x120), at(0x120), 0x1010)?;
        e.long(at(0x108), e.l(at(0x120))?)?;
        e.long(at(0x116), 0x42c80000)?;
        e.sane(at(0x108), at(0x120), 0x100e)?;
        e.sane(at(0x116), at(0x120), 0x1006)?;
        e.sane(at(0x120), at(0x108), 0x1010)?;
        e.word(0x4434b6, e.w(0x45367a)?.wrapping_add(e.w(0x453664)?))?;
        if loaded {
            e.add(0x4434b6, e.sw(0x453710)?.wrapping_add(e.sw(0x4536fa)?))?;
        }
        if e.w(0x453652)? != 0 {
            e.word(
                0x4434b6,
                e.sw(0x4434b6)?.wrapping_mul(e.sw(0x453652)?) as u16,
            )?;
        }
        e.scaled_damage(frame)?;
        let mut total = e.sw(0x4434b6)?;
        if total != 0 {
            e.dialog_number(stack.wrapping_sub(12), 27, e.sw(0x4434b6)?)?;
        }
        for (n, (ammo, weapon)) in [
            (0x45375e, 0x4536c8),
            (0x453712, 0x45367c),
            (0x453714, 0x45367e),
            (0x453716, 0x453680),
            (0x453718, 0x453682),
            (0x45371a, 0x453684),
            (0x45371c, 0x453686),
        ]
        .into_iter()
        .enumerate()
        {
            e.word(
                0x4434b6,
                e.sw(ammo)?
                    .wrapping_add(e.sw(weapon)?)
                    .wrapping_mul(e.sw(0x453652)?) as u16,
            )?;
            e.scaled_damage(frame)?;
            if e.sw(0x4434b6)? > 0 {
                total = total.wrapping_add(e.sw(0x4434b6)?);
                e.dialog_number(stack.wrapping_sub(12), 28 + n as u16, e.sw(0x4434b6)?)?;
            } else if e.sw(weapon)? < 0 {
                e.nz_number(stack, 36 + n as u16, e.sw(weapon)?.wrapping_neg())?;
            }
        }
        if total != 0 {
            e.dialog_number(stack.wrapping_sub(12), 26, total)?;
            if e.w(0x453688)? == 0 {
                e.word(0x453688, 1)?;
            }
            e.trap(0xa862, vec![L(205)])?;
            e.dialog_number(
                stack.wrapping_sub(12),
                24,
                e.sw(0x453688)?.wrapping_mul(total),
            )?;
        }
        e.trap(0xa862, vec![L(69)])?;
        if e.w(0x453688)? != 0 {
            e.dialog_number(stack.wrapping_sub(12), 25, e.sw(0x453688)?)?;
        }
        let damage_bonus = if (member as i16) > -1 {
            e.sb(node(member, slot).wrapping_add(13))?
        } else {
            e.sw(0x4536c6)?
        };
        e.dialog_number(stack.wrapping_sub(12), 17, damage_bonus)?;
        let damage_label = if e.w(0x4536c4)? != 0 {
            0x400fb2
        } else {
            0x400fae
        };
        e.dialog_string(stack.wrapping_sub(14), 49, damage_label)?;
        for (n, (source, item, byte)) in [
            (0x453691, 52, true),
            (0x45368d, 7, true),
            (0x453641, 43, true),
            (0x45363f, 45, true),
            (0x453640, 44, true),
            (0x453642, 46, true),
            (0x453643, 47, true),
            (0x453678, 8, false),
            (0x45369a, 11, false),
            (0x453694, 13, false),
        ]
        .into_iter()
        .enumerate()
        {
            let value = if byte { e.sb(source)? } else { e.sw(source)? };
            e.nz_number(stack.wrapping_sub(n as u32 * 4), item, value)?;
        }
        if e.w(0x453696)? != 0 {
            e.dialog_number(stack.wrapping_sub(12), 14, e.sw(0x453696)?)?;
        }
        if e.w(at(0x10e))? != 0 {
            e.dialog_number(stack.wrapping_sub(12), 15, e.sw(at(0x10e))?)?;
        }
        if e.w(0x453652)? != 0 {
            e.dialog_number(stack.wrapping_sub(12), 50, e.sw(0x453652)?)?;
        }
        if e.sw(at(0x10c))? > 0 {
            e.dialog_number(stack.wrapping_sub(12), 22, e.sw(at(0x112))?)?;
            e.dialog_number(stack.wrapping_sub(16), 53, e.sw(at(0x10c))?)?;
        } else if e.sw(0x453650)? > 0 || e.w(0x453654)? != 0 {
            if e.sw(0x453650)? > 0 {
                e.dialog_number(stack.wrapping_sub(12), 22, e.sw(0x453650)?)?;
                e.dialog_number(stack.wrapping_sub(16), 53, e.sw(0x453650)?)?;
            } else {
                let quantity = if e.sw(0x453650)? > 0 {
                    e.sw(0x453650)?
                } else {
                    0
                };
                e.dialog_number(stack.wrapping_sub(12), 22, quantity)?;
                e.dialog_number(stack.wrapping_sub(12), 53, e.sw(0x453654)?)?;
            }
        } else {
            e.dialog_string(stack.wrapping_sub(14), 22, 0x400faa)?;
            e.dialog_string(stack.wrapping_sub(20), 53, 0x400fa6)?;
        }
        let weight = e
            .sw(0x453660)?
            .wrapping_mul(e.sw(0x453650)?)
            .wrapping_add(e.sw(0x45364c)?);
        e.dialog_number(stack.wrapping_sub(12), 23, weight)?;
        let selected = e.sb(0x44273f)? as u16;
        let caste = e.sw(record(selected).wrapping_add(0x34))?;
        let race = e.sw(record(e.sb(0x44273f)? as u16).wrapping_add(0x32))?;
        e.can_use(stack.wrapping_sub(18), -1, race, caste)?;
        if e.returned_word()? == 0 {
            e.trap(0xaa14, vec![P(0x44337a)])?;
            if e.w(0x435504)? != 0 {
                e.dialog_string(stack.wrapping_sub(14), 18, 0x400f96)?;
            }
            if e.w(0x435506)? != 0 {
                e.dialog_string(stack.wrapping_sub(14), 19, 0x400f86)?;
            }
        }
        if e.w(0x439582)? != 0 {
            let mode = if e.b(0x442750)? != 0 {
                Some(0)
            } else if e.b(0x442756)? != 0 {
                Some(1)
            } else if e.b(0x442752)? != 0 {
                Some(2)
            } else {
                None
            };
            if let Some(mode) = mode {
                let child = stack.wrapping_sub(16);
                e.word(child.wrapping_add(14), mode)?;
                let id = if mode == 0 {
                    e.w(node(e.sb(0x44273f)? as u16, slot))?
                } else {
                    e.w(0x4536a2)?
                };
                e.word(child.wrapping_add(12), id)?;
                e.word(child.wrapping_add(10), slot)?;
                e.word(child.wrapping_add(8), e.sb(0x44273f)? as u16)?;
                e.quick_info(child, window)?;
            }
        }
        e.trap(0xa862, vec![L(69)])?;
        let mut update = true;
        loop {
            e.step()?;
            if update {
                e.trap(0xa922, vec![L(window)])?;
                e.trap(0xa923, vec![L(window)])?;
                e.first(stack.wrapping_sub(12), 56, 59)?;
                if e.plan.flow != LoadFlow::Returned {
                    e.plan.native_return = PromptReturn::NoReturn;
                    return Ok(e.plan);
                }
                e.long(stack.wrapping_sub(8), 0xffff)?;
                e.registers(0xa032, Some(0xffff), None, vec![])?;
                update = false;
            }
            e.trap(0xa860, vec![W(u16::MAX), P(0x44720a), L(0), L(0)])?;
            if e.trap(0xa97f, vec![P(0x44720a)])? as u8 != 0 {
                if e.trap(0xa980, vec![P(0x44720a), P(0x4394c0), P(0x4434b2)])? as u8 == 0 {
                    continue;
                }
                e.dialog_item(e.l(0x4394b8)?, e.w(0x4434b2)?, 0x44292c)?;
                let action = e.w(0x4434b2)?;
                if action == 1 || action == 2 {
                    e.trap(0xa873, vec![L(window)])?;
                    e.button(stack.wrapping_sub(10), true, 0)?;
                    let retained = if action == 2 {
                        let caste = e.sw(record(e.sb(0x44273f)? as u16).wrapping_add(0x34))?;
                        e.profile(stack.wrapping_sub(14), 0, caste)?;
                        6
                    } else {
                        2
                    };
                    let child = stack.wrapping_sub(retained + 18);
                    e.word(child.wrapping_add(16), e.sb(0x44273f)? as u16)?;
                    e.long(
                        child.wrapping_add(12),
                        if action == 2 { 0x30001 } else { 0x40001 },
                    )?;
                    e.word(child.wrapping_add(10), e.sb(0x44273f)? as u16)?;
                    e.word(child.wrapping_add(8), e.sb(0x44273f)? as u16)?;
                    e.selection(0x154a46, child)?;
                    if e.plan.flow != LoadFlow::Returned {
                        e.plan.native_return = PromptReturn::NoReturn;
                        return Ok(e.plan);
                    }
                    e.button(stack.wrapping_sub(retained + 20), false, 0)?;
                    update = true;
                    continue;
                }
                if action == 0 {
                    continue;
                }
                break 'redraw;
            }
            if e.w(0x44720a)? != 1 {
                continue;
            }
            let where_window = e.trap(0xa92c, vec![L(e.l(0x447214)?), P(at(4))])?;
            e.word(0x4471a6, where_window as u16)?;
            e.long(0x4471ee, e.l(0x447214)?)?;
            if e.l(at(4))? == e.l(0x438798)? {
                e.trap(0xa873, vec![L(e.l(0x438798)?)])?;
                e.trap(0xa871, vec![P(0x4471ee)])?;
                if e.sw(0x4471f0)? >= 280 {
                    break 'redraw;
                }
                slot = (e.sw(0x4471ee)?.wrapping_sub(11) / 40).wrapping_add(e.sw(0x44719c)?) as u16;
                if slot as i16 >= e.sw(record(e.sb(0x44273f)? as u16).wrapping_add(0x1a))? {
                    continue;
                }
                item = e.w(node(e.sb(0x44273f)? as u16, slot))?;
                continue 'redraw;
            }
            if e.l(at(4))? == e.l(0x438788)? {
                e.trap(0xa873, vec![L(e.l(0x438788)?)])?;
                e.trap(0xa871, vec![P(0x4471ee)])?;
                if e.sw(0x4471f0)? >= 280 {
                    continue;
                }
                let scroll = e.trap(0xa960, vec![L(e.l(0x439400)?)])? as u16;
                slot = (e.sw(0x4471ee)?.wrapping_sub(11) / 40).wrapping_add(scroll as i16) as u16;
                e.trap(0xa873, vec![L(e.l(0x438788)?)])?;
                item = e.w(0x44721c_u32
                    .wrapping_add(u32::from(slot) * 22)
                    .wrapping_add(0x132))?;
                member = e.sb(0x44273d)? as u16;
                continue 'redraw;
            }
            if e.l(at(4))? == e.l(0x438790)? {
                e.trap(0xa873, vec![L(e.l(0x438790)?)])?;
                e.trap(0xa871, vec![P(0x4471ee)])?;
                e.word(0x41f70a, (e.sw(0x4471f0)?.wrapping_sub(10) / 50) as u16)?;
                e.word(
                    0x41f70c,
                    (e.sw(0x4471ee)?.wrapping_sub(10) / 60).wrapping_add(e.sw(0x41f70e)?) as u16,
                )?;
                if e.sw(0x41f70a)? >= 6 {
                    continue;
                }
                let index = e
                    .sw(0x41f70c)?
                    .wrapping_mul(6)
                    .wrapping_add(e.sw(0x41f70a)?) as u16;
                let address = 0x41fc74_u32.wrapping_add(u32::from(index) * 22);
                if e.w(address)? == 0 {
                    continue;
                }
                item = e.w(address)?;
                continue 'redraw;
            }
        }
    }
    e.trap(0xa983, vec![L(window)])?;
    e.long(stack.wrapping_sub(4), 0xffff)?;
    e.registers(0xa032, Some(0xffff), None, vec![])?;
    e.full(0);
    Ok(e.plan)
}

/// loadammoproc, 0x1300ec. Supply rows and reload nodes use distinct local WORD tables.
pub fn load_ammo_proc(
    state: &InventoryInteractionState,
    frame: u32,
    limits: InventoryInteractionLimits,
    services: &mut (impl InventoryInteractionServices + ?Sized),
) -> Result<InventoryInteractionPlan, RuleError> {
    let mut e = execution(state, limits, services);
    let stack = frame.wrapping_sub(0x41c);
    let at = |offset| frame.wrapping_sub(offset);
    let member = e.w(frame.wrapping_add(8))?;
    let mut slot = e.w(frame.wrapping_add(10))?;
    let mut selected = 0_i16;
    e.word(at(0x3fa), 0)?;
    e.byte(at(0x3ff), 0)?;
    e.byte(at(0x403), 0)?;
    e.byte(at(0x267), 0)?;
    e.byte(0x437a6c, 0)?;
    let menu = e.trap(0xa9bf, vec![W(1000)])?;
    e.trap(0xa935, vec![L(menu), W(u16::MAX)])?;
    e.selection_text(stack, at(0x100), e.sw(0x4536a2)?)?;
    e.word(at(0x266), 0)?;
    for n in 0..200 {
        e.step()?;
        e.word(at(0x3f8).wrapping_add(n * 2), 0)?;
    }
    let inventory = node(member, slot);
    let suffix = if e.w(inventory.wrapping_add(4))? == 0 {
        e.trap(0xa93a, vec![L(menu), W(1)])?;
        Some(0x403198)
    } else {
        e.word(stack.wrapping_sub(2), e.w(inventory.wrapping_add(4))?)?;
        e.load_ammo(e.sw(stack.wrapping_sub(2))?)?;
        if e.w(0x4536e4)? != 0 || e.sw(inventory.wrapping_add(2))? < 1 {
            e.byte(at(0x267), 1)?;
            e.trap(0xa93a, vec![L(menu), W(1)])?;
            Some(0x403182)
        } else {
            None
        }
    };
    e.lib(
        0x400090,
        vec![
            P(if suffix.is_some() { 0x4031a8 } else { 0x403174 }),
            P(0x479ed8),
        ],
    )?;
    e.lib(0x400098, vec![P(at(0x100)), P(0x479ed8)])?;
    if let Some(suffix) = suffix {
        e.lib(0x400098, vec![P(suffix), P(0x479ed8)])?;
    }
    e.lib(0x400188, vec![P(0x479ed8)])?;
    e.trap(0xa947, vec![L(menu), W(1), P(0x479ed8)])?;
    e.word(stack.wrapping_sub(2), e.w(node(member, slot))?)?;
    e.load_item(e.sw(stack.wrapping_sub(2))?)?;
    for n in 0_i16..=200 {
        e.step()?;
        if (e.l(index(0x42dbe4, n, 4))? as i32) <= 0 {
            continue;
        }
        e.word(stack.wrapping_sub(2), n.wrapping_add(200) as u16)?;
        e.load_ammo(e.sw(stack.wrapping_sub(2))?)?;
        if e.w(0x4536f4)? != e.w(0x45365c)? {
            continue;
        }
        let row = e.sw(at(0x266))?;
        e.add(at(0x266), 1)?;
        e.word(index(at(0x264), row, 2), n as u16)?;
        e.ammo_menu_text(frame, stack, n)?;
        e.trap(0xa933, vec![L(menu), P(0x479ed8)])?;
        if n as u16 == e.w(0x453738)? || e.b(at(0x267))? != 0 {
            e.word(at(0x3fa), n.wrapping_add(1) as u16)?;
            e.trap(
                0xa945,
                vec![L(menu), W(e.w(at(0x266))?.wrapping_add(1)), B(19)],
            )?;
        }
    }
    if e.w(0x438776)? == 0 {
        for (pointer, disable) in [
            (0x40315e, false),
            (0x403140, true),
            (0x403124, true),
            (0x40315e, false),
            (0x40310c, true),
        ] {
            e.trap(0xa933, vec![L(menu), P(pointer)])?;
            if disable {
                let last = e.trap(0xa950, vec![L(menu)])? as u16;
                e.trap(0xa93a, vec![L(menu), W(last)])?;
            }
        }
        e.word(at(0x3fc), 0)?;
        let first = e.trap(0xa950, vec![L(menu)])? as u16;
        e.word(at(0x402), first.wrapping_add(1))?;
        for n in 0_i16..=200 {
            e.step()?;
            if (e.l(index(0x42dbe4, n, 4))? as i32) <= 0 {
                continue;
            }
            e.word(stack.wrapping_sub(2), n.wrapping_add(200) as u16)?;
            e.load_ammo(e.sw(stack.wrapping_sub(2))?)?;
            let row = e.sw(at(0x3fc))?;
            e.add(at(0x3fc), 1)?;
            e.word(index(at(0x3f8), row, 2), e.w(0x453738)?)?;
            e.ammo_menu_text(frame, stack.wrapping_sub(2), n)?;
            e.trap(0xa933, vec![L(menu), P(0x479ed8)])?;
        }
    }
    let mut direct_transfer = false;
    if e.sw(frame.wrapping_add(14))? == -1 {
        if e.w(node(member, 0).wrapping_add(4))? == 0 {
            e.full(0);
            return Ok(e.plan);
        }
        slot = 0;
        e.word(stack.wrapping_sub(2), e.w(node(member, 0))?)?;
        e.load_item(e.sw(stack.wrapping_sub(2))?)?;
        let mut n = 1_i16;
        while n <= e.sw(record(member).wrapping_add(0x1a))? {
            e.step()?;
            if e.w(node(member, n as u16))? == e.w(node(member, 0).wrapping_add(4))?
                || (e.b(at(0x267))? != 0
                    && e.b(node(member, n as u16).wrapping_add(8))?
                        == e.b(node(member, 0).wrapping_add(8))?)
            {
                selected = 3;
                e.word(at(0x262), n as u16)?;
            }
            n = n.wrapping_add(1);
        }
    } else if e.sw(frame.wrapping_add(14))? == -2 {
        slot = 0;
        e.word(stack.wrapping_sub(2), e.w(node(member, 0))?)?;
        e.load_item(e.sw(stack.wrapping_sub(2))?)?;
        let mut n = 1_i16;
        loop {
            e.step()?;
            if n > e.sw(record(member).wrapping_add(0x1a))? {
                e.full(0);
                return Ok(e.plan);
            }
            e.word(stack.wrapping_sub(2), e.w(node(member, n as u16))?)?;
            e.load_ammo(e.sw(stack.wrapping_sub(2))?)?;
            if e.w(0x4536f4)? == e.w(0x45365c)? {
                selected = 3;
                e.word(node(member, 0).wrapping_add(4), e.w(0x453738)?)?;
                e.word(at(0x262), n as u16)?;
                direct_transfer = true;
                break;
            }
            n = n.wrapping_add(1);
        }
    }
    let mut unload_only = false;
    if !direct_transfer {
        e.trap(0xa972, vec![P(0x4471ee)])?;
        e.trap(0xa870, vec![P(0x4471ee)])?;
        if e.w(frame.wrapping_add(12))? != 0 {
            selected = e.trap(
                0xa80b,
                vec![
                    L(menu),
                    W(e.w(0x4471ee)?.wrapping_sub(150)),
                    W(e.w(0x4471f0)?),
                    W(e.w(at(0x3fa))?),
                ],
            )? as i16;
        }
        e.trap(0xa936, vec![W(1000)])?;
        e.trap(0xa9a3, vec![L(menu)])?;
        if selected == 1 && e.w(node(member, slot).wrapping_add(2))? != 0 {
            e.byte(at(0x3ff), 1)?;
            unload_only = true;
        } else if selected.wrapping_sub(2) > e.sw(at(0x266))? {
            let id = e.w(index(at(0x3f8), selected.wrapping_sub(e.sw(at(0x402))?), 2))?;
            let child_frame = stack.wrapping_sub(14);
            e.word(child_frame.wrapping_add(12), id)?;
            e.long(child_frame.wrapping_add(8), u32::MAX)?;
            let child = show_item_info(&e.plan.state, child_frame, e.child_limits()?, e.services)?;
            e.adopt_interaction(child)?;
            if e.plan.flow != LoadFlow::Returned {
                e.plan.native_return = PromptReturn::NoReturn;
                return Ok(e.plan);
            }
            e.full(0);
            return Ok(e.plan);
        }
    }
    let transfer = direct_transfer || selected > 1;
    let selected_index = |e: &Execution<'_, _>| e.sw(index(at(0x264), selected.wrapping_sub(2), 2));
    let mut unload = unload_only;
    if transfer && !unload_only {
        let id = selected_index(&e)?.wrapping_add(200);
        e.word(stack.wrapping_sub(2), id as u16)?;
        e.load_ammo(e.sw(stack.wrapping_sub(2))?)?;
        let address = node(member, slot);
        unload = e.w(0x4536e4)? == 0
            && e.w(address.wrapping_add(4))? != 0
            && e.sw(address.wrapping_add(2))? >= 1
            && e.sw(address.wrapping_add(4))? != selected_index(&e)?;
    }
    if unload {
        e.byte(0x437a6c, 1)?;
        let address = node(member, slot);
        e.word(stack.wrapping_sub(2), e.w(address.wrapping_add(4))?)?;
        e.load_ammo(e.sw(stack.wrapping_sub(2))?)?;
        let quantity = i32::from(e.sw(address.wrapping_add(2))?) as u32;
        let supply = index(
            0x42dbe4,
            e.sw(address.wrapping_add(4))?.wrapping_sub(200),
            4,
        );
        e.long(supply, e.l(supply)?.wrapping_add(quantity))?;
        e.word(address.wrapping_add(2), 0)?;
        e.word(record(member).wrapping_add(0x126), 0)?;
        e.word(address.wrapping_add(4), 0)?;
    }
    if transfer && e.b(at(0x3ff))? == 0 {
        let address = node(member, slot);
        e.word(
            at(0x3fe),
            e.w(address.wrapping_add(6))?
                .wrapping_sub(e.w(address.wrapping_add(2))?),
        )?;
        let supply = index(0x42dbe4, selected_index(&e)?, 4);
        if i32::from(e.sw(at(0x3fe))?) >= e.l(supply)? as i32 {
            e.word(at(0x3fe), e.l(supply)? as u16)?;
        }
        if e.w(0x438776)? != 0 && e.sw(at(0x3fe))? > 0 {
            e.add(record(member).wrapping_add(0x40), -1)?;
        }
        e.add(address.wrapping_add(2), e.sw(at(0x3fe))?)?;
        e.long(
            supply,
            e.l(supply)?
                .wrapping_sub(i32::from(e.sw(at(0x3fe))?) as u32),
        )?;
        e.word(
            record(member).wrapping_add(0x126),
            selected_index(&e)?.wrapping_add(200) as u16,
        )?;
        e.word(
            address.wrapping_add(4),
            selected_index(&e)?.wrapping_add(200) as u16,
        )?;
        if e.w(0x453696)? != 0 && e.b(address.wrapping_add(9))? != 0 {
            e.add(record(member).wrapping_add(0x26), e.sw(at(0x3fe))?)?;
            e.byte(0x4427b3, 1)?;
        }
        if e.b(at(0x403))? == 1 {
            e.byte(0x437a6c, 1)?;
            let child = stack.wrapping_sub(16);
            e.word(child.wrapping_add(14), 0)?;
            e.word(child.wrapping_add(12), selected_index(&e)? as u16)?;
            e.word(
                child.wrapping_add(10),
                e.w(node(member, selected_index(&e)? as u16))?,
            )?;
            e.word(child.wrapping_add(8), member)?;
            e.equipment(0x121152, child)?;
        }
    }
    if e.w(0x438776)? != 0 && e.trap(0xa924, vec![])? == e.l(0x43879c)? {
        e.panel(stack.wrapping_sub(10), member as i16)?;
    }
    if selected_index(&e)? != 0 || e.b(0x437a6c)? != 0 {
        e.word(stack.wrapping_sub(2), 2)?;
        let sample = e.rand(2)?.wrapping_add(403);
        e.sound(sample, stack)?;
    }
    e.low(selected_index(&e)? as u16);
    Ok(e.plan)
}

fn temp_node(base: u32, slot: u16) -> u32 {
    base.wrapping_add(u32::from(slot) * 22).wrapping_add(0x132)
}

/// moveicon, 0x13c1ee. Drag observations select the ordered native transfer route.
pub fn move_icon(
    state: &InventoryInteractionState,
    frame: u32,
    limits: InventoryInteractionLimits,
    services: &mut (impl InventoryInteractionServices + ?Sized),
) -> Result<InventoryInteractionPlan, RuleError> {
    let mut e = execution(state, limits, services);
    let stack = frame.wrapping_sub(0x18e);
    let at = |offset| frame.wrapping_sub(offset);
    e.long(at(0x16c), 0)?;
    e.byte(at(0x167), 0)?;
    e.byte(at(0xc7), 0)?;
    e.byte(at(0xcf), 0)?;
    e.byte(at(0x16d), 0)?;
    e.word(at(0xb0), 0)?;
    e.word(at(0xb2), 0)?;
    e.word(at(0xac), 32)?;
    e.word(at(0xae), 32)?;
    e.word(at(0xa8), u16::MAX)?;
    let mut stock_changed = false;
    e.trap(0xa889, vec![W(1)])?;
    if e.l(0x439404)? != e.l(0x439400)? {
        e.long(0x439404, e.l(0x4393fc)?)?;
    }
    let value = e.trap(0xa960, vec![L(e.l(0x439404)?)])? as u16;
    e.word(0x4470b2, value)?;
    e.word(0x4471e4, e.w(0x4470b2)?.wrapping_add(e.w(0x4471e2)?))?;
    let value = e.trap(0xa960, vec![L(e.l(0x439400)?)])? as u16;
    e.word(at(0x170), value)?;
    let mut item = 0_i16;
    if !e.right_of(320)? {
        if e.w(0x4046e8)? != 0 || e.w(0x4046e6)? != 0 {
            e.warn(stack.wrapping_sub(10), 158)?;
            e.full(0);
            return Ok(e.plan);
        }
        let value = e.trap(0xa960, vec![L(e.l(0x439400)?)])? as u16;
        item = e.sw(temp_node(
            0x44721c,
            e.w(0x4471e2)?.wrapping_add(value).wrapping_sub(1),
        ))?;
        let value = e.trap(0xa960, vec![L(e.l(0x439400)?)])? as u16;
        e.word(
            at(0x17a),
            e.w(
                temp_node(0x44721c, e.w(0x4471e2)?.wrapping_add(value).wrapping_sub(1))
                    .wrapping_add(2),
            )?,
        )?;
        e.sane(at(0x17a), at(0x17a), 0x200e)?;
        e.sane(at(0x17a), at(0x17a), 0x1010)?;
        e.long(at(0xcc), e.l(at(0x17a))?)?;
    }
    if e.right_of(320)? {
        if e.sb(0x44273e)? > -1 {
            let value = e.trap(0xa960, vec![L(e.l(0x4393fc)?)])? as u16;
            item = e.sw(temp_node(
                0x447e5e,
                e.w(0x4471e2)?.wrapping_add(value).wrapping_sub(1),
            ))?;
        } else {
            item = e.sw(index(
                0x478288,
                e.sw(0x4471a8)?
                    .wrapping_add(e.sw(0x4471e4)?)
                    .wrapping_sub(1),
                2,
            ))?;
        }
    }
    e.word(stack.wrapping_sub(2), item as u16)?;
    e.load_item(e.sw(stack.wrapping_sub(2))?)?;
    if (e.l(0x4536ca)? as i32) < 0 {
        e.long(0x4536ca, (e.l(0x4536ca)? as i32).wrapping_neg() as u32)?;
    }
    e.word(at(0x17a), e.w(0x453650)?)?;
    e.sane(at(0x17a), at(0x17a), 0x200e)?;
    e.sane(at(0x17a), at(0x17a), 0x1010)?;
    e.long(at(0x16c), e.l(at(0x17a))?)?;
    if e.right_of(320)? {
        if e.sb(0x44273e)? > -1 {
            let value = e.trap(0xa960, vec![L(e.l(0x4393fc)?)])? as u16;
            e.word(
                0x453650,
                e.w(
                    temp_node(0x447e5e, e.w(0x4471e2)?.wrapping_add(value).wrapping_sub(1))
                        .wrapping_add(2),
                )?,
            )?;
        }
    } else {
        e.sane(at(0xcc), at(0x17a), 0x100e)?;
        e.trap(0xa9eb, vec![P(at(0x17a)), W(0x16)])?;
        e.sane(at(0x17a), at(0x17a), 0x2810)?;
        e.word(0x453650, e.w(at(0x178))?)?;
    }
    e.trap(0xa88a, vec![W(12)])?;
    e.trap(0xaa14, vec![P(0x44337a)])?;
    e.trap(
        0xa893,
        vec![W(half(e.sw(0x40856c)?).wrapping_add(215) as u16), W(427)],
    )?;
    e.numeric(stack, e.weight()?)?;
    if e.b(0x44273e)? != 0xff {
        e.long(0x4536ca, 0)?;
    }
    if e.right_of(320)? && e.b(0x44273e)? == 0xff {
        e.trap(0xa862, vec![L(69)])?;
        e.trap(
            0xa893,
            vec![W(half(e.sw(0x40856c)?).wrapping_add(108) as u16), W(427)],
        )?;
        e.cost(stack, 0)?;
        e.numeric_long(stack.wrapping_sub(2), e.l(0x4536ca)? as i32)?;
    } else if e.b(0x442753)? != 0 {
        if e.sw(0x453650)? > 0 {
            e.word(at(0x17a), e.w(0x453650)?)?;
            e.sane(at(0x17a), at(0x17a), 0x200e)?;
            e.sane(at(0x17a), at(0x17a), 0x1010)?;
            e.long(at(0xcc), e.l(at(0x17a))?)?;
        } else {
            e.long(at(0xcc), 0)?;
        }
        let divisor = i32::from(e.sw(0x442576)?.wrapping_mul(2).wrapping_add(10));
        let cost = (e.l(0x4536ca)? as i32)
            .checked_div(divisor)
            .ok_or_else(|| RuleError::Invalid("native item sale division traps".into()))?;
        e.long(0x4536ca, cost as u32)?;
        if e.b(0x44273e)? == 0xff {
            e.trap(
                0xa893,
                vec![W(half(e.sw(0x40856c)?).wrapping_add(161) as u16), W(427)],
            )?;
            e.trap(0xaa14, vec![P(0x443374)])?;
            e.cost(stack, 1)?;
            e.numeric_long(stack.wrapping_sub(2), e.l(0x4536ca)? as i32)?;
        } else {
            e.long(0x4536ca, 0)?;
        }
    }
    e.word(0x438756, 0)?;
    e.word(at(0xc6), 395)?;
    e.word(at(0xc4), half(e.sw(0x40856c)?).wrapping_add(67) as u16)?;
    e.word(at(0xc0), half(e.sw(0x40856c)?).wrapping_add(99) as u16)?;
    e.word(at(0xc2), 427)?;
    e.word(
        at(0xba),
        e.sw(0x4471e2)?.wrapping_mul(40).wrapping_sub(24) as u16,
    )?;
    let left = half(e.sw(0x40856c)?).wrapping_add(390).wrapping_sub(
        half(e.sw(0x40856c)?)
            .wrapping_add(380)
            .wrapping_mul(e.sw(0x438758)?),
    );
    e.word(at(0xb8), left as u16)?;
    e.word(at(0xb4), e.w(at(0xb8))?.wrapping_add(32))?;
    e.word(at(0xb6), e.w(at(0xba))?.wrapping_add(32))?;
    e.word(at(8), e.w(0x4471ee)?)?;
    e.word(at(6), e.w(0x4471f0)?)?;
    e.word(at(4), e.w(at(8))?.wrapping_add(32))?;
    e.word(at(2), e.w(at(6))?.wrapping_add(32))?;
    e.long(at(0xa6), e.l(at(0xba))?)?;
    e.long(at(0xa2), e.l(at(0xb6))?)?;
    e.trap(0xa862, vec![L(69)])?;
    e.erase_shop(stack.wrapping_sub(10), true, 0)?;
    e.word(stack.wrapping_sub(4), e.w(0x4536a2)?)?;
    let base = rules::selection_base(e.sw(0x4536a2)?);
    e.word(0x4471e8, base as u16)?;
    e.indexed_string(
        base.wrapping_add(1) as u16,
        e.w(0x4536a2)?.wrapping_sub(e.w(0x4471e8)?).wrapping_add(1),
    )?;
    e.trap(0xa884, vec![P(0x479ed8)])?;
    e.trap(0xa862, vec![L(33)])?;
    e.trap(0xa863, vec![L(30)])?;
    let control = if e.right_of(320)? {
        e.l(0x4393fc)?
    } else {
        e.l(0x439400)?
    };
    let scroll = e.trap(0xa960, vec![L(control)])? as u16;
    e.word(
        at(0xbe),
        e.w(0x4471e2)?.wrapping_add(scroll).wrapping_sub(1),
    )?;
    if e.right_of(320)? {
        e.can_use(stack.wrapping_sub(14), -1, e.sw(0x44724e)?, e.sw(0x447250)?)?;
        if e.returned_word()? == 0 {
            e.byte(at(0xcf), 1)?;
        }
    } else if e.b(0x44273e)? != 0xff {
        e.can_use(stack.wrapping_sub(14), -1, e.sw(0x447e90)?, e.sw(0x447e92)?)?;
        if e.returned_word()? == 0 {
            e.byte(at(0xcf), 1)?;
        }
    }
    if e.trap(0xa858, vec![L(u32::from(e.w(0x447218)?)), L(2048)])? != 0 {
        e.trap(0xa853, vec![])?;
        let child = stack.wrapping_sub(14);
        if e.right_of(320)? && e.b(0x44273e)? == 0xff {
            e.byte(0x44273f, e.b(0x44273d)?)?;
            e.word(child.wrapping_add(12), e.w(0x4536a2)?)?;
            e.long(child.wrapping_add(8), u32::MAX)?;
        } else {
            let selector = if e.right_of(320)? { 0x44273e } else { 0x44273d };
            e.byte(0x44273f, e.b(selector)?)?;
            let control = if e.right_of(320)? {
                e.l(0x4393fc)?
            } else {
                e.l(0x439400)?
            };
            let scroll = e.trap(0xa960, vec![L(control)])? as u16;
            e.word(child.wrapping_add(12), 0)?;
            e.word(
                child.wrapping_add(10),
                e.w(0x4471e2)?.wrapping_add(scroll).wrapping_sub(1),
            )?;
            e.word(child.wrapping_add(8), e.sb(selector)? as u16)?;
        }
        let plan = show_item_info(&e.plan.state, child, e.child_limits()?, e.services)?;
        e.adopt_interaction(plan)?;
        if e.plan.flow != LoadFlow::Returned {
            e.plan.native_return = PromptReturn::NoReturn;
            return Ok(e.plan);
        }
        e.word(stack.wrapping_sub(2), 1)?;
        e.selection(0x156b26, stack.wrapping_sub(10))?;
        if e.plan.flow != LoadFlow::Returned {
            e.plan.native_return = PromptReturn::NoReturn;
            return Ok(e.plan);
        }
        e.refresh_wings(stack.wrapping_sub(2))?;
        e.full(0);
        return Ok(e.plan);
    }
    if e.b(at(0xcf))? != 0 {
        e.trap(0xaa1c, vec![L(e.l(0x439534)?)])?;
        e.sound(674, stack)?;
    } else {
        e.trap(0xa852, vec![])?;
        e.sound(130, stack)?;
    }
    e.icon3(stack, e.sw(0x453646)?, at(0xc6))?;
    e.trap(
        0xa8a8,
        vec![P(at(0xba)), W(e.w(0x4395a4)?), W(e.w(0x4395a2)?)],
    )?;
    if e.trap(0xa858, vec![L(u32::from(e.w(0x447218)?)), L(512)])? != 0 {
        e.trap(0xa972, vec![P(0x4471ee)])?;
        e.registers(0xa032, Some(0xffff), None, vec![])?;
        e.word(0x4471f0, if e.right_of(320)? { 35 } else { 500 })?;
    } else {
        e.trap(
            0xa8ec,
            vec![
                P(0x3fe288),
                P(e.l(0x4386b4)?.wrapping_add(2)),
                P(at(0xba)),
                P(at(0xb2)),
                W(0),
                L(0),
            ],
        )?;
        loop {
            e.step()?;
            e.trap(0xa972, vec![P(0x4471ee)])?;
            if e.w(0x4471f0)? != e.w(at(0xa8))? || e.w(0x4471ee)? != e.w(at(0xaa))? {
                e.word(at(8), e.w(0x4471ee)?)?;
                e.word(at(6), e.w(0x4471f0)?)?;
                e.word(at(4), e.w(at(8))?.wrapping_add(32))?;
                e.word(at(2), e.w(at(6))?.wrapping_add(32))?;
                e.trap(0xa8a8, vec![P(at(8)), W(e.w(0x4395a4)?), W(e.w(0x4395a2)?)])?;
                e.trap(
                    0xa8a8,
                    vec![P(at(0xa6)), W(e.w(0x4395a4)?), W(e.w(0x4395a2)?)],
                )?;
                e.trap(
                    0xa8ec,
                    vec![
                        P(e.l(0x4386b4)?.wrapping_add(2)),
                        P(0x3fe288),
                        P(at(0xb2)),
                        P(at(0xa6)),
                        W(0),
                        L(0),
                    ],
                )?;
                e.trap(
                    0xa8ec,
                    vec![
                        P(0x3fe288),
                        P(e.l(0x4386b4)?.wrapping_add(2)),
                        P(at(8)),
                        P(at(0xb2)),
                        W(0),
                        L(0),
                    ],
                )?;
                e.trap(
                    0xa8ec,
                    vec![P(0x3fe288), P(0x3fe288), P(at(0xba)), P(at(8)), W(39), L(0)],
                )?;
                e.trap(
                    0xa8a8,
                    vec![
                        P(at(8)),
                        W(e.sw(0x4395a4)?.wrapping_neg() as u16),
                        W(e.sw(0x4395a2)?.wrapping_neg() as u16),
                    ],
                )?;
                e.long(at(0xa6), e.l(at(8))?)?;
                e.long(at(0xa2), e.l(at(4))?)?;
                e.long(at(0xaa), e.l(0x4471ee)?)?;
            }
            if e.trap(0xa973, vec![])? as u8 == 0 {
                break;
            }
        }
        e.trap(
            0xa8a8,
            vec![P(at(0xa6)), W(e.w(0x4395a4)?), W(e.w(0x4395a2)?)],
        )?;
        e.trap(
            0xa8ec,
            vec![
                P(e.l(0x4386b4)?.wrapping_add(2)),
                P(0x3fe288),
                P(at(0xb2)),
                P(at(0xa6)),
                W(0),
                L(0),
            ],
        )?;
    }
    e.trap(0xaa0b, vec![L(e.l(0x4358ce)?)])?;
    e.trap(0xa8a3, vec![P(at(0xc6))])?;
    e.sound(83, stack)?;
    e.trap(0xa862, vec![L(69)])?;
    e.erase_shop(stack.wrapping_sub(12), true, 0)?;
    let name = record(e.sb(0x44273d)? as u16).wrapping_add(0xa21);
    e.lib(0x400188, vec![P(name)])?;
    e.trap(
        0xa884,
        vec![P(record(e.sb(0x44273d)? as u16).wrapping_add(0xa21))],
    )?;
    e.lib(
        0x400180,
        vec![P(record(e.sb(0x44273d)? as u16).wrapping_add(0xa21))],
    )?;
    e.word(0x4434bc, half(e.sw(0x40856c)?).wrapping_add(17) as u16)?;
    e.word(0x4434c0, e.w(0x4434bc)?.wrapping_add(32))?;
    e.word(0x4434ba, 402)?;
    e.word(0x4434be, e.w(0x4434ba)?.wrapping_add(32))?;
    let portrait = stack.wrapping_sub(36);
    e.word(portrait.wrapping_add(22), 0)?;
    e.word(portrait.wrapping_add(20), e.sb(0x44273d)? as u16)?;
    e.word(
        portrait.wrapping_add(18),
        e.w(record(e.sb(0x44273d)? as u16).wrapping_add(0x34))?,
    )?;
    e.long(portrait.wrapping_add(14), e.l(0x4434be)?)?;
    e.long(portrait.wrapping_add(10), e.l(0x4434ba)?)?;
    e.word(
        portrait.wrapping_add(8),
        e.w(record(e.sb(0x44273d)? as u16).wrapping_add(0x54))?,
    )?;
    e.character(0x14a3e6, portrait)?;
    e.erase_shop(
        stack.wrapping_sub(10),
        false,
        i16::from(e.b(0x44273e)? == 0xff),
    )?;
    e.trap(0xaa1c, vec![L(e.l(0x43952c)?)])?;
    e.trap(0xa853, vec![])?;

    if (!e.right_of(280)? && e.w(0x438758)? == 0 && e.limited_pack(0x44721c)?)
        || (e.right_of(320)? && e.b(0x44273e)? != 0xff && e.limited_pack(0x447e5e)?)
    {
        e.trap(0xa853, vec![])?;
        e.warn(stack.wrapping_sub(10), 19)?;
        e.refresh_wings(stack.wrapping_sub(2))?;
        e.full(0);
        return Ok(e.plan);
    }
    let mut transfer_right = false;
    let mut transfer_left = false;
    if e.right_of(285)? && e.sw(0x4471f0)? < half(e.sw(0x40856c)?).wrapping_add(355) {
        let vertical = e.sw(0x4471ee)? / 50;
        let from_left = e.w(0x438758)? != 0;
        let from_right = e.w(0x438758)? == 0 && e.b(0x44273e)? != 0xff;
        if vertical == 6 && (from_left || from_right) {
            let selector = if from_left { 0x44273d } else { 0x44273e };
            let temporary = if from_left { 0x44721c } else { 0x447e5e };
            e.copy(temporary, record(e.sb(selector)? as u16), WARRIOR_BYTES)?;
            let address = node(e.sb(selector)? as u16, e.w(at(0xbe))?);
            e.add_crate(stack, address)?;
            if e.returned_word()? == 0 {
                e.warn(stack.wrapping_sub(10), 153)?;
                e.full(0);
                return Ok(e.plan);
            }
            let child = stack.wrapping_sub(16);
            e.word(child.wrapping_add(14), 1)?;
            e.word(child.wrapping_add(12), e.w(at(0xbe))?)?;
            e.word(
                child.wrapping_add(10),
                e.w(node(e.sb(selector)? as u16, e.w(at(0xbe))?))?,
            )?;
            e.word(child.wrapping_add(8), e.sb(selector)? as u16)?;
            e.equipment(0x121152, child)?;
            if e.sw(record(e.sb(selector)? as u16).wrapping_add(0x1a))?
                .wrapping_sub(9)
                > -1
            {
                let control = if from_left {
                    e.l(0x439400)?
                } else {
                    e.l(0x4393fc)?
                };
                e.trap(
                    0xa965,
                    vec![
                        L(control),
                        W(e.w(record(e.sb(selector)? as u16).wrapping_add(0x1a))?
                            .wrapping_sub(9)),
                    ],
                )?;
            }
            e.copy(record(e.sb(selector)? as u16), temporary, WARRIOR_BYTES)?;
            e.movement(stack.wrapping_sub(10), e.sb(selector)? as u16)?;
            e.compact(stack.wrapping_sub(12), e.sb(selector)?)?;
            e.word(stack.wrapping_sub(6), 0)?;
            e.character(0x120872, stack.wrapping_sub(14))?;
            e.full(0);
            return Ok(e.plan);
        }
        if from_left || from_right {
            let same = vertical == e.sb(if from_left { 0x44273e } else { 0x44273d })?;
            if e.limited_pack(record(vertical as u16))? {
                e.trap(0xa853, vec![])?;
                e.warn(stack.wrapping_sub(10), 19)?;
                e.refresh_wings(stack.wrapping_sub(2))?;
                e.full(0);
                return Ok(e.plan);
            }
            if !same {
                if vertical > e.sb(0x44273a)?
                    || vertical == e.sb(if from_left { 0x44273d } else { 0x44273e })?
                {
                    e.full(0);
                    return Ok(e.plan);
                }
                let selector = if from_left { 0x44273e } else { 0x44273d };
                let temporary = if from_left { 0x447e5e } else { 0x44721c };
                e.copy(temporary, record(e.sb(selector)? as u16), WARRIOR_BYTES)?;
                e.movement(stack.wrapping_sub(10), e.sb(selector)? as u16)?;
                e.compact(stack.wrapping_sub(12), e.sb(selector)?)?;
                e.byte(selector, vertical as u8)?;
                e.long(0x439404, e.l(if from_left { 0x439420 } else { 0x43941c })?)?;
                e.copy(0x45363c, at(0x166), ITEM_BYTES)?;
                e.change(stack.wrapping_sub(16), 0x10000)?;
                if e.plan.flow != LoadFlow::Returned {
                    e.plan.native_return = PromptReturn::NoReturn;
                    return Ok(e.plan);
                }
                e.copy(at(0x166), 0x45363c, ITEM_BYTES)?;
                e.trap(0xa873, vec![L(e.l(0x438788)?)])?;
                e.byte(at(0xc7), 1)?;
            }
            e.add(0x4471f0, if from_left { 100 } else { -100 })?;
            if from_left {
                transfer_right = true;
            } else {
                transfer_left = true;
            }
        }
    }
    if transfer_right || (e.right_of(320)? && e.w(0x438758)? != 0 && e.sb(0x44273e)? > -1) {
        e.long(0x439404, e.l(0x4393fc)?)?;
        let scroll = e.trap(0xa960, vec![L(e.l(0x4393fc)?)])? as i16;
        let maximum = e.trap(0xa962, vec![L(e.l(0x4393fc)?)])? as i16;
        e.word(at(0xbc), maximum as u16)?;
        let mut destination = e.w(0x447e78)?;
        e.add(0x448834, e.weight()?)?;
        e.add(0x447bf2, e.weight()?.wrapping_neg())?;
        e.movement(stack.wrapping_sub(10), 0xfffe)?;
        e.movement(stack.wrapping_sub(12), 0xffff)?;
        let merged = e.merge_pack(frame, stack, item, 0x447e5e, 0x447e78, scroll, 0x44273e)?;
        if !merged {
            e.insert_pack(0x447e5e, 0x447e78, scroll, maximum, &mut destination)?;
            let source = e.w(0x4471e2)?.wrapping_add(e.w(at(0x170))?).wrapping_sub(1);
            e.copy(
                temp_node(0x44721c, source),
                temp_node(0x447e5e, destination),
                22,
            )?;
            e.byte(temp_node(0x447e5e, destination).wrapping_add(9), 0)?;
            if e.sw(0x447e78)? < 9 {
                e.word(
                    0x4471f2,
                    e.sw(0x447e78)?.wrapping_mul(40).wrapping_add(16) as u16,
                )?;
                e.render_pack_row(frame, stack, item, 0x447e5e, destination, 0x44273e, false)?;
            }
            e.add(0x447e78, 1)?;
            if e.sw(0x447e78)? > 9 {
                e.trap(
                    0xa965,
                    vec![L(e.l(0x4393fc)?), W(e.w(0x447e78)?.wrapping_sub(9))],
                )?;
            }
            e.word(0x4471e6, 21)?;
            if scroll < e.sw(at(0xbc))? {
                e.word(0x438756, 0)?;
            }
            e.word(stack.wrapping_sub(2), e.w(0x4471e6)?)?;
            e.update_shop(stack.wrapping_sub(10), scroll)?;
        }
    }
    if transfer_left
        || (!e.right_of(250)?
            && e.w(0x438758)? == 0
            && e.sw(0x4471f0)? != half(e.sw(0x40856c)?).wrapping_add(250))
    {
        if e.w(0x4046e6)? != 0 && e.w(0x4046e8)? == 0 {
            e.warn(stack.wrapping_sub(10), 159)?;
            e.full(0);
            return Ok(e.plan);
        }
        e.long(0x439404, e.l(0x439400)?)?;
        let scroll = e.trap(0xa960, vec![L(e.l(0x439400)?)])? as i16;
        let maximum = e.trap(0xa962, vec![L(e.l(0x439400)?)])? as i16;
        e.word(at(0xbc), maximum as u16)?;
        let mut destination = e.w(0x447236)?;
        if (e.l(0x4536ca)? as i32) > (e.l(0x442788)? as i32) {
            e.trap(0xa853, vec![])?;
            e.warn(stack.wrapping_sub(10), 37)?;
            e.refresh_wings(stack.wrapping_sub(2))?;
            e.full(0);
            return Ok(e.plan);
        }
        let merged;
        if e.w(0x45365e)? != 0 {
            let supply = index(0x42dbe4, e.sw(0x4536a2)?.wrapping_sub(200), 4);
            e.long(
                supply,
                e.l(supply)?.wrapping_add(i32::from(e.sw(0x453650)?) as u32),
            )?;
            if e.w(0x453648)? != 0 {
                e.sound(e.sw(0x453648)?, stack)?;
            } else if e.w(0x45365e)? != 0 {
                e.word(stack.wrapping_sub(2), 2)?;
                let sound = e.rand(2)?.wrapping_add(403);
                e.sound(sound, stack)?;
            } else {
                e.sound(6002, stack)?;
            }
            let flash = stack.wrapping_sub(20);
            e.long(flash.wrapping_add(16), 0x1e0000)?;
            e.long(flash.wrapping_add(12), 0x320032)?;
            e.long(flash.wrapping_add(8), 0x401d9e)?;
            let child = feedback::flashmessage(
                &e.plan.state.memory,
                flash,
                0x401d9e,
                50,
                50,
                30,
                0,
                e.limits.prompt.maximum_polls,
                e.services,
            )?;
            e.feedback(child);
            merged = true;
            e.byte(at(0x16d), 1)?;
            e.word(0x453660, 0)?;
            e.word(0x45364c, 0)?;
        } else {
            merged = e.merge_pack(frame, stack, item, 0x44721c, 0x447236, scroll, 0x44273d)?;
            if !merged {
                e.insert_pack(0x44721c, 0x447236, scroll, maximum, &mut destination)?;
            }
        }
        e.long(0x442788, e.l(0x442788)?.wrapping_sub(e.l(0x4536ca)?))?;
        e.movement(stack.wrapping_sub(10), 0xffff)?;
        e.shop_credit()?;
        if !merged {
            e.byte(temp_node(0x44721c, destination).wrapping_add(9), 0)?;
        }
        if !merged {
            e.add(0x447236, 1)?;
        }
        if e.b(0x44273e)? == 0xff {
            let value = e.trap(0xa960, vec![L(e.l(0x4393fc)?)])? as u16;
            e.word(
                at(0xce),
                e.w(0x4471a8)?
                    .wrapping_add(e.w(0x4471e2)?)
                    .wrapping_add(value)
                    .wrapping_sub(1),
            )?;
            e.add(0x447bf2, e.weight()?)?;
            if e.w(0x4046e8)? != 0 {
                e.add(0x4046e8, -1)?;
            }
            if !merged {
                let child = stack.wrapping_sub(18);
                e.word(child.wrapping_add(16), e.w(0x4536a2)?)?;
                e.word(child.wrapping_add(14), destination)?;
                e.word(child.wrapping_add(12), e.w(at(0xce))?)?;
                e.long(child.wrapping_add(8), u32::MAX)?;
                e.install(child)?;
            }
            let stock = index(0x478a58, e.sw(at(0xce))?, 1);
            let before = e.b(stock)?;
            e.byte(stock, before.wrapping_sub(1))?;
            if before == 1 {
                e.word(index(0x478288, e.sw(at(0xce))?, 2), 0)?;
                e.byte(at(0x167), 1)?;
            }
        } else {
            let value = e.trap(0xa960, vec![L(e.l(0x4393fc)?)])? as u16;
            let source = e.w(0x4471e2)?.wrapping_add(value).wrapping_sub(1);
            e.copy(
                temp_node(0x447e5e, source),
                temp_node(0x44721c, destination),
                22,
            )?;
        }
        if !merged {
            if e.sw(0x447236)? > 8 {
                e.trap(
                    0xa965,
                    vec![L(e.l(0x439400)?), W(e.w(0x447236)?.wrapping_sub(9))],
                )?;
                e.word(0x4471e6, 21)?;
                e.word(0x438756, 1)?;
                if scroll < e.sw(at(0xbc))? {
                    e.word(0x438756, 0)?;
                }
                e.word(stack.wrapping_sub(2), e.w(0x4471e6)?)?;
                e.update_shop(stack.wrapping_sub(10), scroll)?;
            } else {
                e.word(
                    0x4471f2,
                    (destination as i16).wrapping_mul(40).wrapping_add(16) as u16,
                )?;
                e.render_pack_row(frame, stack, item, 0x44721c, destination, 0x44273d, false)?;
            }
        }
        e.word(0x438756, 0)?;
    }
    if e.b(at(0x167))? != 0 {
        e.word(
            0x4471f2,
            e.sw(0x4471e2)?.wrapping_mul(40).wrapping_sub(24) as u16,
        )?;
        e.word(0x4471f4, half(e.sw(0x40856c)?).wrapping_add(390) as u16)?;
        e.word(0x4471f6, e.w(0x4471f2)?.wrapping_add(32))?;
        e.word(
            0x4471f8,
            e.sw(0x4471f4)?
                .wrapping_add(250)
                .wrapping_add(half(e.sw(0x40856c)?)) as u16,
        )?;
        e.trap(0xa8a3, vec![P(0x4471f2)])?;
        e.long(0x439404, e.l(0x4393fc)?)?;
        e.icon(stack, 2000)?;
    }
    if e.sw(0x4471f0)? < half(e.sw(0x40856c)?).wrapping_add(250)
        && ((e.w(0x438758)? == 0 && e.b(0x44273e)? != 0xff) || e.b(at(0xc7))? != 0)
    {
        e.add(0x447bf2, e.weight()?)?;
        e.add(0x448834, e.weight()?.wrapping_neg())?;
        e.movement(stack.wrapping_sub(10), 0xffff)?;
        e.movement(stack.wrapping_sub(12), 0xfffe)?;
        let value = e.trap(0xa960, vec![L(e.l(0x4393fc)?)])? as u16;
        e.shift_pack(
            0x447e5e,
            0x447e78,
            e.w(0x4471e2)?.wrapping_add(value).wrapping_sub(1),
        )?;
        e.long(0x439404, e.l(0x4393fc)?)?;
        let maximum = e.trap(0xa962, vec![L(e.l(0x4393fc)?)])? as u16;
        e.word(at(0xbc), maximum)?;
        let value = e.trap(0xa960, vec![L(e.l(0x4393fc)?)])? as u16;
        e.word(0x4471e0, value)?;
        if e.w(at(0xbc))? != 0 {
            e.trap(
                0xa965,
                vec![L(e.l(0x4393fc)?), W(e.w(at(0xbc))?.wrapping_sub(1))],
            )?;
        }
        e.add(0x447e78, -1)?;
        e.word(0x43875e, 1)?;
        if e.sw(0x447e78)?.wrapping_sub(e.sw(0x4471e0)?) > 9 {
            e.byte(0x4427b1, 2)?;
        }
        e.word(stack.wrapping_sub(2), 0)?;
        e.character(0x120872, stack.wrapping_sub(10))?;
        e.byte(0x4427b1, 0)?;
        e.word(0x43875e, 0)?;
    }
    if e.right_of(320)? && (e.w(0x438758)? != 0 || e.b(at(0xc7))? != 0) {
        if e.b(0x44273e)? == 0xff {
            e.long(0x442788, e.l(0x442788)?.wrapping_add(e.l(0x4536ca)?))?;
            let value = e.trap(0xa960, vec![L(e.l(0x439400)?)])? as u16;
            item = e.sw(temp_node(
                0x44721c,
                e.w(0x4471e2)?.wrapping_add(value).wrapping_sub(1),
            ))?;
            e.add(0x447bf2, e.weight()?.wrapping_neg())?;
            e.word(stack.wrapping_sub(2), item as u16)?;
            let selection = rules::selection_base(item);
            e.word(0x4471e8, selection as u16)?;
            e.word(0x4434aa, e.w(0x4471e8)?)?;
            let mut found = false;
            while e.sw(0x4434aa)? < e.sw(0x4471e8)?.wrapping_add(200) {
                e.step()?;
                if e.sw(index(0x478288, e.sw(0x4434aa)?, 2))? == item {
                    let quantity = index(0x478a58, e.sw(0x4434aa)?, 1);
                    if e.w(0x4471e8)? == e.w(0x4471a8)? && e.b(quantity)? == 0 {
                        stock_changed = true;
                    }
                    e.byte(quantity, e.b(quantity)?.wrapping_add(1))?;
                    found = true;
                    break;
                }
                e.add(0x4434aa, 1)?;
            }
            if !found {
                e.word(0x4434aa, e.w(0x4471e8)?)?;
                while e.sw(0x4434aa)? < e.sw(0x4471e8)?.wrapping_add(200) {
                    e.step()?;
                    let quantity = index(0x478a58, e.sw(0x4434aa)?, 1);
                    if e.b(quantity)? == 0 {
                        e.byte(quantity, e.b(quantity)?.wrapping_add(1))?;
                        stock_changed = true;
                        e.word(index(0x478288, e.sw(0x4434aa)?, 2), item as u16)?;
                        break;
                    }
                    e.add(0x4434aa, 1)?;
                }
            }
        }
        e.trap(0xa853, vec![])?;
        let value = e.trap(0xa960, vec![L(e.l(0x439400)?)])? as u16;
        e.shift_pack(
            0x44721c,
            0x447236,
            e.w(0x4471e2)?.wrapping_add(value).wrapping_sub(1),
        )?;
        e.movement(stack.wrapping_sub(10), 0xffff)?;
        e.long(0x439404, e.l(0x439400)?)?;
        let maximum = e.trap(0xa962, vec![L(e.l(0x439400)?)])? as u16;
        e.word(at(0xbc), maximum)?;
        let value = e.trap(0xa960, vec![L(e.l(0x439400)?)])? as u16;
        e.word(0x4471e0, value)?;
        if e.w(at(0xbc))? != 0 {
            e.trap(
                0xa965,
                vec![L(e.l(0x439400)?), W(e.w(at(0xbc))?.wrapping_sub(1))],
            )?;
        }
        e.add(0x447236, -1)?;
        if stock_changed {
            e.word(0x438756, 1)?;
        }
        if e.sw(0x447236)?.wrapping_sub(e.sw(0x4471e0)?) > 9 {
            e.byte(0x4427b1, 2)?;
        }
        e.word(stack.wrapping_sub(2), 0)?;
        e.character(0x120872, stack.wrapping_sub(10))?;
        e.byte(0x4427b1, 0)?;
    }
    if stock_changed {
        e.long(0x439404, e.l(0x4393fc)?)?;
        e.word(0x43875c, 1)?;
        e.word(0x438756, 0)?;
        e.word(stack.wrapping_sub(2), 0)?;
        e.character(0x120872, stack.wrapping_sub(10))?;
    }
    e.finish_move(stack)?;
    Ok(e.plan)
}

pub struct FixedPicture(pub u32);
impl crate::recovered_drawing::DrawingServices for FixedPicture {
    fn color_icon(&mut self, _: i16) -> Result<u32, RuleError> {
        Err(RuleError::Invalid("unused interaction color icon".into()))
    }
    fn indexed_string(&mut self, _: i16, _: i16) -> Result<PascalString, RuleError> {
        Err(RuleError::Invalid(
            "unused interaction drawing string".into(),
        ))
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
        Err(RuleError::Invalid(
            "unused interaction graphics world".into(),
        ))
    }
    fn new_world(
        &mut self,
        _: i16,
        _: crate::recovered_geometry::BodyRect,
    ) -> Result<(i16, crate::recovered_drawing::BuildingWorld), RuleError> {
        Err(RuleError::Invalid("unused interaction new world".into()))
    }
}

pub struct Clock<'a, 'b, S: ?Sized> {
    pub execution: &'a mut Execution<'b, S>,
    pub remaining: usize,
}
impl<S: InventoryInteractionServices + ?Sized> crate::recovered_timing::TimingServices
    for Clock<'_, '_, S>
{
    fn tick_count(&mut self) -> Result<u32, RuleError> {
        self.remaining = self
            .remaining
            .checked_sub(1)
            .ok_or_else(|| RuleError::Invalid("interaction timing poll budget exhausted".into()))?;
        self.execution.observe(FeedbackRequest::TickCount)
    }
    fn button(&mut self) -> Result<bool, RuleError> {
        Ok(self.execution.observe(FeedbackRequest::Button)? as u8 != 0)
    }
}

/// choice, 0x112820. Every control comparison is repeated after reached children.
pub fn choice(
    state: &InventoryInteractionState,
    frame: u32,
    limits: InventoryInteractionLimits,
    services: &mut (impl InventoryInteractionServices + ?Sized),
) -> Result<InventoryInteractionPlan, RuleError> {
    let mut e = execution(state, limits, services);
    let stack = frame.wrapping_sub(0x26);
    let at = |offset| frame.wrapping_sub(offset);
    let key = e.w(frame.wrapping_add(8))?;
    let mut changed = false;
    e.byte(at(0x11), 0)?;
    e.word(0x43877e, 0)?;
    e.trap(0xa873, vec![L(e.l(0x438788)?)])?;
    if key == 0 {
        e.long(0x4471ee, e.l(0x447214)?)?;
        e.trap(0xa871, vec![P(0x4471ee)])?;
        let part = e.trap(
            0xa96c,
            vec![L(e.l(0x4471ee)?), L(e.l(0x438788)?), P(0x439404)],
        )? as u16;
        e.word(0x4471a6, part)?;
    }
    e.copy(0x44721c, record(e.sb(0x44273d)? as u16), WARRIOR_BYTES)?;
    if e.b(0x44273e)? != 0xff {
        e.copy(0x447e5e, record(e.sb(0x44273e)? as u16), WARRIOR_BYTES)?;
    }
    if e.l(0x439404)? != e.l(0x4393fc)? && e.l(0x439404)? != e.l(0x439400)? {
        e.word(0x4434b6, e.w(0x4471a8)?)?;
        if e.l(0x439404)? == e.l(0x439460)? || key == 0x2269 {
            e.press_control(stack, 0x439460, 5, 2, 0)?;
            if e.b(0x442753)? != 0 {
                e.save_shop()?;
                if e.plan.flow != LoadFlow::Returned {
                    e.plan.native_return = PromptReturn::NoReturn;
                    return Ok(e.plan);
                }
            }
            e.trap(0xa914, vec![L(e.l(0x438788)?)])?;
            e.long(0x438788, 0)?;
            e.byte(0x442756, 0)?;
            e.byte(0x44273f, e.b(0x44273d)?)?;
            e.full(u32::MAX);
            return Ok(e.plan);
        }
        if e.b(0x442753)? != 0 {
            if e.l(0x439404)? == e.l(0x479ed4)? && e.b(0x44273e)? != 0xff {
                e.long(0x439404, e.l(0x439408)?)?;
            }
            for (control, category, sound_first) in [
                (0x439408, 0, false),
                (0x439410, 400, true),
                (0x43940c, 200, false),
                (0x439418, 600, true),
                (0x439414, 800, true),
            ] {
                if e.l(0x439404)? != e.l(control)? {
                    continue;
                }
                if sound_first {
                    e.sound(141, stack)?;
                }
                if control != 0x439410 {
                    let bounds = e.l(e.l(control)?)?;
                    e.long(at(0x10), e.l(bounds.wrapping_add(8))?)?;
                    e.long(at(0xc), e.l(bounds.wrapping_add(12))?)?;
                    e.press_rect(
                        stack.wrapping_sub(if sound_first { 24 } else { 22 }),
                        1,
                        2,
                        if sound_first { 0 } else { 141 },
                        at(0x10),
                    )?;
                    changed = true;
                }
                e.word(0x4471a8, category)?;
            }
        }
        if e.w(0x43875c)? != 0 {
            e.trap(0xa965, vec![L(e.l(0x4393fc)?), W(191)])?;
        }
        for (control, selector, temporary, horizontal) in [
            (0x439420, 0x44273e, 0x447e5e, 314),
            (0x43941c, 0x44273d, 0x44721c, 262),
        ] {
            if e.l(0x439404)? != e.l(control)? {
                continue;
            }
            if control == 0x439420 && e.w(0x4046e8)? != 0 {
                e.warn(stack.wrapping_sub(10), 158)?;
                e.full(0);
                return Ok(e.plan);
            }
            e.trap(0xa972, vec![P(0x4471ee)])?;
            e.word(at(8), 388)?;
            e.word(at(6), half(e.sw(0x40856c)?).wrapping_add(horizontal) as u16)?;
            e.word(at(2), e.w(at(6))?.wrapping_add(22))?;
            e.word(at(4), e.w(at(8))?.wrapping_add(22))?;
            let mut n = 0_i16;
            while n <= e.sb(0x44273a)? {
                e.step()?;
                if n == 3 {
                    e.trap(0xa8a8, vec![P(at(8)), L(0xffb90018)])?;
                }
                if e.trap(0xa8ad, vec![L(e.l(0x4471ee)?), P(at(8))])? as u8 != 0 {
                    if n != e.sb(0x44273d)? && n != e.sb(0x44273e)? && n <= e.sb(0x44273a)? {
                        let mut retained = 0;
                        if selector == 0x44273d || e.b(0x44273e)? != 0xff {
                            e.copy(temporary, record(e.sb(selector)? as u16), WARRIOR_BYTES)?;
                            e.movement(stack.wrapping_sub(10), e.sb(selector)? as u16)?;
                            e.compact(stack.wrapping_sub(12), e.sb(selector)?)?;
                            if selector == 0x44273d {
                                retained = 4;
                            }
                        }
                        e.byte(selector, n as u8)?;
                        e.change(stack.wrapping_sub(retained + 12), 0x10000)?;
                        if e.plan.flow != LoadFlow::Returned {
                            e.plan.native_return = PromptReturn::NoReturn;
                            return Ok(e.plan);
                        }
                        e.trap(0xa873, vec![L(e.l(0x438788)?)])?;
                        e.full(0);
                        return Ok(e.plan);
                    }
                    e.sound(143, stack)?;
                }
                e.trap(0xa8a8, vec![P(at(8)), L(0x180000)])?;
                n = n.wrapping_add(1);
            }
        }
        for (control, selector) in [(0x439428, 0x44273d), (0x439424, 0x44273e)] {
            if e.l(0x439404)? != e.l(control)? || (control == 0x439424 && e.w(0x43875c)? != 0) {
                continue;
            }
            e.press_control(stack, control, 1, 2, 141)?;
            e.byte(0x44273f, e.b(selector)?)?;
            e.movement(stack.wrapping_sub(24), e.sb(selector)? as u16)?;
            let view = stack.wrapping_sub(28);
            e.word(view.wrapping_add(10), 0)?;
            e.word(view.wrapping_add(8), e.sb(selector)? as u16)?;
            e.view_character(view)?;
            if e.plan.flow != LoadFlow::Returned {
                e.plan.native_return = PromptReturn::NoReturn;
                return Ok(e.plan);
            }
            if control == 0x439424 {
                e.byte(0x44273f, e.b(0x44273d)?)?;
            }
            e.word(stack.wrapping_sub(22), 1)?;
            e.selection(0x156b26, stack.wrapping_sub(30))?;
            if e.plan.flow != LoadFlow::Returned {
                e.plan.native_return = PromptReturn::NoReturn;
                return Ok(e.plan);
            }
        }
        if changed && e.sb(0x44273e)? > -1 && e.b(0x442753)? != 0 {
            e.word(0x4434ba, 387)?;
            e.word(0x4434be, e.w(0x4434ba)?.wrapping_add(73))?;
            e.word(0x4434bc, half(e.sw(0x40856c)?) as u16)?;
            e.word(0x4434c0, half(e.sw(0x40856c)?).wrapping_add(640) as u16)?;
            e.pict(stack.wrapping_sub(18), 148, 0x4434ba)?;
            e.word(0x4434bc, half(e.sw(0x40856c)?).wrapping_add(17) as u16)?;
            e.word(0x4434ba, 402)?;
            e.word(0x4434be, e.w(0x4434ba)?.wrapping_add(32))?;
            e.trap(0xa88a, vec![W(14)])?;
            e.trap(0xa862, vec![L(69)])?;
            e.erase_shop(stack.wrapping_sub(20), true, 0)?;
            e.trap(0xa889, vec![W(1)])?;
            e.lib(
                0x400188,
                vec![P(record(e.sb(0x44273d)? as u16).wrapping_add(0xa21))],
            )?;
            e.trap(
                0xa884,
                vec![P(record(e.sb(0x44273d)? as u16).wrapping_add(0xa21))],
            )?;
            e.lib(
                0x400180,
                vec![P(record(e.sb(0x44273d)? as u16).wrapping_add(0xa21))],
            )?;
            let portrait = stack.wrapping_sub(44);
            e.word(portrait.wrapping_add(22), 0)?;
            e.word(portrait.wrapping_add(20), e.sb(0x44273d)? as u16)?;
            e.word(portrait.wrapping_add(18), e.w(0x447250)?)?;
            e.long(portrait.wrapping_add(14), e.l(0x4434be)?)?;
            e.long(portrait.wrapping_add(10), e.l(0x4434ba)?)?;
            e.word(portrait.wrapping_add(8), e.w(0x447270)?)?;
            e.character(0x14a3e6, portrait)?;
            e.long(0x439404, e.l(0x439420)?)?;
            e.trap(0xa963, vec![L(e.l(0x439404)?), W(0)])?;
            e.byte(0x44273e, 0xff)?;
            e.change(stack.wrapping_sub(48), 0x10000)?;
            if e.plan.flow != LoadFlow::Returned {
                e.plan.native_return = PromptReturn::NoReturn;
                return Ok(e.plan);
            }
        }
        if (e.w(0x4434b6)? != e.w(0x4471a8)? && e.b(0x44273e)? == 0xff) || changed {
            e.long(0x439404, e.l(0x4393fc)?)?;
            e.trap(0xa963, vec![L(e.l(0x439404)?), W(0)])?;
            e.word(stack.wrapping_sub(2), 0)?;
            e.character(0x120872, stack.wrapping_sub(10))?;
            if changed {
                e.press_rect(stack.wrapping_sub(22), 1, 1, 0, at(0x10))?;
            }
        }
    }
    if e.l(0x439404)? == e.l(0x439464)? || key == 0x240d || key == 0x264 {
        e.press_control(stack, 0x439464, 5, 2, 141)?;
        if e.b(0x442753)? != 0 {
            e.save_shop()?;
            if e.plan.flow != LoadFlow::Returned {
                e.plan.native_return = PromptReturn::NoReturn;
                return Ok(e.plan);
            }
        }
        e.trap(0xa914, vec![L(e.l(0x438788)?)])?;
        e.long(0x438788, 0)?;
        e.byte(0x442756, 0)?;
        e.full(0xfffffffd);
        return Ok(e.plan);
    }
    for (control, selector, next_control) in [
        (0x439468, 0x44273d, 0x43941c),
        (0x43946c, 0x44273e, 0x439420),
    ] {
        if e.l(0x439404)? != e.l(control)? || (control == 0x43946c && e.b(0x44273e)? == 0xff) {
            continue;
        }
        e.press_control(stack, control, 2, 2, 141)?;
        e.first(stack.wrapping_sub(26), 41, 41)?;
        if e.plan.flow != LoadFlow::Returned {
            e.plan.native_return = PromptReturn::NoReturn;
            return Ok(e.plan);
        }
        let refit = stack.wrapping_sub(34);
        e.word(refit.wrapping_add(14), 0)?;
        e.long(refit.wrapping_add(10), 0)?;
        e.word(refit.wrapping_add(8), e.sb(selector)? as u16)?;
        e.equipment(0x14e5e4, refit)?;
        e.long(0x439404, e.l(next_control)?)?;
        e.change(stack.wrapping_sub(38), 0)?;
        if e.plan.flow != LoadFlow::Returned {
            e.plan.native_return = PromptReturn::NoReturn;
            return Ok(e.plan);
        }
        e.trap(0xa873, vec![L(e.l(0x438788)?)])?;
        e.long(0x439404, 0)?;
        e.press_control(stack.wrapping_sub(30), control, 2, 1, 0)?;
    }
    for control in [0x4393fc, 0x439400] {
        if e.l(0x439404)? != e.l(control)? {
            continue;
        }
        let thumb = e.w(0x4471a6)? == 129;
        let part = e.track_control(
            e.l(control)?,
            e.l(0x4471ee)?,
            if thumb { 0 } else { 0x139244 },
        )?;
        e.word(0x4471a6, part)?;
        if thumb {
            e.word(0x438756, 1)?;
            e.word(stack.wrapping_sub(2), 0)?;
            e.character(0x120872, stack.wrapping_sub(10))?;
            e.word(0x438756, 0)?;
        }
        e.full(0);
        return Ok(e.plan);
    }
    let horizontal = e.sw(0x4471f0)?;
    let center = half(e.sw(0x40856c)?);
    if !((horizontal > center.wrapping_add(389) && horizontal < center.wrapping_add(423))
        || (horizontal > 9 && horizontal < 43))
    {
        e.full(0);
        return Ok(e.plan);
    }
    e.word(0x4471e2, 0)?;
    e.long(0x439404, e.l(0x4393fc)?)?;
    let value = e.trap(0xa960, vec![L(e.l(0x4393fc)?)])? as u16;
    e.word(0x4470b2, value)?;
    e.word(0x438758, 0)?;
    if e.sw(0x4471f0)? < 320 {
        e.word(0x438758, 1)?;
        e.long(0x439404, e.l(0x439400)?)?;
        let value = e.trap(0xa960, vec![L(e.l(0x439400)?)])? as u16;
        e.word(0x4470b2, value)?;
    }
    for n in 1_i16..=9 {
        let lower = 15 + (n - 1) * 40;
        if e.sw(0x4471ee)? > lower && e.sw(0x4471ee)? < lower + 33 {
            e.word(0x4471e2, n as u16)?;
        }
    }
    e.word(0x4471e4, e.w(0x4470b2)?.wrapping_add(e.w(0x4471e2)?))?;
    if e.sw(0x4471e2)? > e.sw(0x447236)? && e.w(0x438758)? != 0 {
        e.word(0x4471e2, 0)?;
    }
    if e.sw(0x4471e2)? > e.sw(0x447e78)? && e.w(0x438758)? == 0 && e.w(0x43875c)? == 0 {
        e.word(0x4471e2, 0)?;
    }
    let stock = e
        .sw(0x4471a8)?
        .wrapping_add(e.sw(0x4471e2)?)
        .wrapping_add(e.sw(0x4470b2)?)
        .wrapping_sub(1);
    if e.b(index(0x478a58, stock, 1))? == 0 && e.w(0x438758)? == 0 && e.b(0x44273e)? == 0xff {
        e.word(0x4471e2, 0)?;
    }
    if e.w(0x439582)? != 0 {
        let quick = stack.wrapping_sub(16);
        e.word(quick.wrapping_add(14), 1)?;
        if e.sw(0x4471f0)? < 300 {
            e.word(
                quick.wrapping_add(12),
                e.w(node(e.sb(0x44273d)? as u16, e.w(0x4471e4)?.wrapping_sub(1)))?,
            )?;
            e.word(quick.wrapping_add(10), e.w(0x4471e4)?.wrapping_sub(1))?;
            e.word(quick.wrapping_add(8), e.sb(0x44273d)? as u16)?;
        } else if e.sb(0x44273e)? > -1 {
            e.word(
                quick.wrapping_add(12),
                e.w(node(e.sb(0x44273e)? as u16, e.w(0x4471e4)?.wrapping_sub(1)))?,
            )?;
            e.word(quick.wrapping_add(10), e.w(0x4471e4)?.wrapping_sub(1))?;
            e.word(quick.wrapping_add(8), e.sb(0x44273e)? as u16)?;
        } else {
            let slot = e
                .sw(0x4471a8)?
                .wrapping_add(e.sw(0x4471e2)?)
                .wrapping_add(e.sw(0x4470b2)?)
                .wrapping_sub(1);
            e.word(quick.wrapping_add(12), e.w(index(0x478288, slot, 2))?)?;
            e.long(quick.wrapping_add(8), u32::MAX)?;
        }
        let a2 = e.services.incoming_quick_info_a2(&e.plan.state.memory)?;
        e.quick_info(quick, a2)?;
    }
    for left in [false, true] {
        if e.w(0x4471e2)? == 0 || (e.w(0x438758)? != 0) != left {
            continue;
        }
        let selector = if left { 0x44273d } else { 0x44273e };
        let temporary = if left { 0x44721c } else { 0x447e5e };
        let next_control = if left { 0x43941c } else { 0x439420 };
        if e.trap(0xa858, vec![L(u32::from(e.w(0x447218)?)), L(256)])? != 0 {
            if !left && e.b(0x44273e)? == 0xff {
                continue;
            }
            e.byte(0x435660, 1)?;
            let slot = e.w(0x4471e4)?.wrapping_sub(1);
            if e.b(node(e.sb(selector)? as u16, slot).wrapping_add(9))? != 0 {
                let remove = stack.wrapping_sub(16);
                e.long(remove.wrapping_add(12), 0x10000)?;
                e.word(remove.wrapping_add(10), e.w(0x4471e4)?.wrapping_sub(1))?;
                e.word(remove.wrapping_add(8), e.sb(selector)? as u16)?;
                e.remove(remove)?;
            } else {
                let wear = stack.wrapping_sub(14);
                e.word(wear.wrapping_add(12), 1)?;
                e.word(wear.wrapping_add(10), e.w(0x4471e4)?.wrapping_sub(1))?;
                e.word(wear.wrapping_add(8), e.sb(selector)? as u16)?;
                e.equipment(0x164a0e, wear)?;
            }
            e.copy(record(e.sb(selector)? as u16), temporary, WARRIOR_BYTES)?;
            e.long(0x439404, e.l(next_control)?)?;
            e.change(stack.wrapping_sub(12), 1)?;
            if e.plan.flow != LoadFlow::Returned {
                e.plan.native_return = PromptReturn::NoReturn;
                return Ok(e.plan);
            }
            e.byte(0x435660, 0)?;
            e.word(index(0x442798, e.sb(selector)?, 2), 1)?;
        } else if !left
            && e.b(0x44273e)? != 0xff
            && e.b(node(e.sb(selector)? as u16, e.w(0x4471e4)?.wrapping_sub(1)).wrapping_add(9))?
                == 0
        {
            e.choose_move(stack.wrapping_sub(8))?;
            if e.plan.flow != LoadFlow::Returned {
                e.plan.native_return = PromptReturn::NoReturn;
                return Ok(e.plan);
            }
            e.word(0x438758, 0)?;
        } else if left
            && e.b(node(e.sb(selector)? as u16, e.w(0x4471e4)?.wrapping_sub(1)).wrapping_add(9))?
                == 0
            || (!left && e.b(0x44273e)? == 0xff)
        {
            e.choose_move(stack.wrapping_sub(8))?;
            if e.plan.flow != LoadFlow::Returned {
                e.plan.native_return = PromptReturn::NoReturn;
                return Ok(e.plan);
            }
        } else if e
            .b(node(e.sb(selector)? as u16, e.w(0x4471e4)?.wrapping_sub(1)).wrapping_add(9))?
            != 0
        {
            if e.trap(0xa858, vec![L(u32::from(e.w(0x447218)?)), L(2048)])? != 0 {
                e.choose_move(stack.wrapping_sub(8))?;
                if e.plan.flow != LoadFlow::Returned {
                    e.plan.native_return = PromptReturn::NoReturn;
                    return Ok(e.plan);
                }
            } else if e.trap(0xa858, vec![L(u32::from(e.w(0x447218)?)), L(4096)])? == 0 {
                e.hold_warning(frame, stack)?;
            }
        } else {
            e.choose_move(stack.wrapping_sub(8))?;
            if e.plan.flow != LoadFlow::Returned {
                e.plan.native_return = PromptReturn::NoReturn;
                return Ok(e.plan);
            }
        }
    }
    e.full(0);
    Ok(e.plan)
}

/// seeshop, 0x152ec2. The click/key dispatcher returns the native choice WORD.
pub fn see_shop(
    state: &InventoryInteractionState,
    frame: u32,
    limits: InventoryInteractionLimits,
    services: &mut (impl InventoryInteractionServices + ?Sized),
) -> Result<InventoryInteractionPlan, RuleError> {
    let mut e = execution(state, limits, services);
    let stack = frame.wrapping_sub(8);
    e.pool()?;
    if e.w(frame.wrapping_add(8))? == 0 {
        e.word(stack.wrapping_sub(2), 0)?;
        e.selection(0x156b26, stack.wrapping_sub(10))?;
        if e.plan.flow != LoadFlow::Returned {
            e.plan.native_return = PromptReturn::NoReturn;
            return Ok(e.plan);
        }
    }
    e.trap(0xa873, vec![L(e.l(0x438788)?)])?;
    if e.b(0x442753)? != 0 {
        e.first(stack.wrapping_sub(12), 14, 15)?;
        if e.plan.flow != LoadFlow::Returned {
            e.plan.native_return = PromptReturn::NoReturn;
            return Ok(e.plan);
        }
    } else {
        e.first(stack.wrapping_sub(12), 11, 13)?;
        if e.plan.flow != LoadFlow::Returned {
            e.plan.native_return = PromptReturn::NoReturn;
            return Ok(e.plan);
        }
    }
    if e.plan.flow != LoadFlow::Returned {
        e.plan.native_return = PromptReturn::NoReturn;
        return Ok(e.plan);
    }
    e.first(stack.wrapping_sub(12), 36, 36)?;
    if e.plan.flow != LoadFlow::Returned {
        e.plan.native_return = PromptReturn::NoReturn;
        return Ok(e.plan);
    }
    loop {
        e.step()?;
        if e.trap(0xa858, vec![L(u32::from(e.w(0x447218)?)), L(1024)])? != 0 {
            e.warn(stack.wrapping_sub(10), 21)?;
        }
        e.trap(0xa9b4, vec![])?;
        e.word(stack.wrapping_sub(2), 0)?;
        e.prompt(0x133d46, stack.wrapping_sub(10))?;
        if e.plan.flow != LoadFlow::Returned {
            e.plan.native_return = PromptReturn::NoReturn;
            return Ok(e.plan);
        }
        e.trap(0xa970, vec![W(u16::MAX), P(0x44720a)])?;
        if e.trap(0xa97f, vec![P(0x44720a)])? as u8 != 0 {
            e.trap(0xa980, vec![P(0x44720a), P(0x4394c0), P(0x4434b2)])?;
            continue;
        }
        match e.w(0x44720a)? {
            6 => {
                e.trap(0xa922, vec![L(e.l(0x438788)?)])?;
                e.trap(0xa923, vec![L(e.l(0x438788)?)])?;
                if e.l(0x438794)? != 0 {
                    e.trap(0xa922, vec![L(e.l(0x438794)?)])?;
                    e.trap(0xa923, vec![L(e.l(0x438794)?)])?;
                }
            }
            1 => {
                let part =
                    e.trap(0xa92c, vec![L(e.l(0x447214)?), P(frame.wrapping_sub(4))])? as u16;
                e.word(0x4471a6, part)?;
                if e.w(0x4471a6)? == 1 {
                    e.runtime(0x13e13c)?;
                    let result = e.trap(0xa93d, vec![L(e.l(0x447214)?)])?;
                    e.long(0x438742, result)?;
                    e.trap(0xa938, vec![W(0)])?;
                } else if e.w(0x4471a6)? == 3 {
                    let child = stack.wrapping_sub(10);
                    e.word(child.wrapping_add(8), 0)?;
                    let plan = choice(&e.plan.state, child, e.child_limits()?, e.services)?;
                    e.adopt_interaction(plan)?;
                    if e.plan.flow != LoadFlow::Returned {
                        e.plan.native_return = PromptReturn::NoReturn;
                        return Ok(e.plan);
                    }
                    e.word(0x4471ea, e.returned_word()?)?;
                    if e.w(0x4471ea)? != 0 {
                        e.low(e.w(0x4471ea)?);
                        return Ok(e.plan);
                    }
                }
            }
            3 => {
                e.word(0x4434aa, e.w(0x44720e)?)?;
                e.long(0x439404, 0)?;
                let child = stack.wrapping_sub(10);
                e.word(child.wrapping_add(8), 0)?;
                let plan = choice(&e.plan.state, child, e.child_limits()?, e.services)?;
                e.adopt_interaction(plan)?;
                if e.plan.flow != LoadFlow::Returned {
                    e.plan.native_return = PromptReturn::NoReturn;
                    return Ok(e.plan);
                }
                e.word(0x4471ea, e.returned_word()?)?;
                if e.w(0x4471ea)? != 0 {
                    e.low(e.w(0x4471ea)?);
                    return Ok(e.plan);
                }
            }
            7 => {
                if (e.l(0x44720c)? >> 16) as u16 != 0 {
                    e.trap(0xa880, vec![P(0x4471ee), L(0x28001e)])?;
                    e.trap(0xa9e9, vec![L(e.l(0x4471ee)?), L(e.l(0x44720c)?), W(0)])?;
                }
            }
            15 if e.l(0x44720c)? >> 24 == 1 => {
                if e.b(0x4427b2)? == 0 {
                    e.byte(0x4427b2, 1)?;
                } else {
                    e.runtime(0x13e13c)?;
                    e.byte(0x4427b2, 0)?;
                    e.word(stack.wrapping_sub(2), 1)?;
                    e.selection(0x156b26, stack.wrapping_sub(10))?;
                    if e.plan.flow != LoadFlow::Returned {
                        e.plan.native_return = PromptReturn::NoReturn;
                        return Ok(e.plan);
                    }
                    e.registers(0xa032, Some(0xffff), None, vec![])?;
                }
            }
            _ => {}
        }
    }
}

/// Raw inputs consumed by the native Items controller's composed panel.
pub trait ItemsServices: InventoryInteractionServices {
    fn items_damage_frames(
        &mut self,
        _: &RenderingMemory,
    ) -> Result<[crate::recovered_attack::DamageFrame; 2], RuleError> {
        Err(RuleError::Unsupported {
            action: "inventory_interaction.items_panel",
            dependency: "actual scratch inputs for both calcdamage calls",
        })
    }
}

/// Known semantic CPU inputs at Items entry. Opaque ABI saves need no snapshot.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ItemsRegisters {
    pub data: [u32; 8],
    pub known_data_bits: [u32; 8],
    pub address: [Option<u32>; 8],
    pub condition_codes: u8,
    pub known_condition_codes: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemsRegisterObservation {
    pub data: [u32; 8],
    pub known_data_bits: [u32; 8],
    pub address: [Option<u32>; 8],
    pub condition_codes: u8,
    pub known_condition_codes: u8,
}

pub struct ItemsCpu {
    pub data: [u32; 8],
    pub known_data_bits: [u32; 8],
    pub address: [Option<u32>; 8],
    pub ccr: u8,
    pub known_ccr: u8,
    saved_slots: Vec<ItemsSavedSlot>,
}

struct ItemsSavedSlot {
    address: u32,
    value: u32,
    known_mask: u32,
}

impl ItemsCpu {
    pub fn new(input: &ItemsRegisters, frame: u32) -> Self {
        let mut cpu = Self {
            data: input.data,
            known_data_bits: input.known_data_bits,
            address: input.address,
            ccr: input.condition_codes & 0x1f,
            known_ccr: input.known_condition_codes & 0x1f,
            saved_slots: vec![],
        };
        cpu.address[7] = Some(frame.wrapping_add(4));
        cpu
    }
    pub fn save_register(
        &mut self,
        e: &mut dyn crate::recovered_application_control::SourceMemory,
        register: usize,
    ) -> Result<(), RuleError> {
        let (value, known_mask) = if register < 8 {
            (self.data[register], self.known_data_bits[register])
        } else {
            match self.address[register - 8] {
                Some(value) => (value, u32::MAX),
                None => (0, 0),
            }
        };
        self.offset_address(7, -4)?;
        let address = self.address(7)?;
        // Unknown saved bytes cease to be readable, even if backing memory
        // previously contained bytes here. No fabricated save value is emitted.
        for offset in 0..4 {
            let shift = (3 - offset) * 8;
            let byte_mask = 0xff_u32 << shift;
            let at = address.wrapping_add(offset);
            e.source_memory_mut().bytes.remove(&at);
            if known_mask & byte_mask == byte_mask {
                e.source_memory_mut()
                    .bytes
                    .insert(at, (value >> shift) as u8);
            }
        }
        self.saved_slots.push(ItemsSavedSlot {
            address,
            value,
            known_mask,
        });
        Ok(())
    }
    pub fn restore_register(
        &mut self,
        e: &dyn crate::recovered_application_control::SourceMemory,
        register: usize,
    ) -> Result<(), RuleError> {
        let address = self.address(7)?;
        let index = self
            .saved_slots
            .iter()
            .position(|slot| slot.address == address)
            .ok_or_else(|| {
                RuleError::Invalid("Items restore has no matching symbolic save".into())
            })?;
        let slot = self.saved_slots.remove(index);
        let mut value = slot.value;
        let mut known_mask = slot.known_mask;
        for offset in 0..4 {
            let shift = (3 - offset) * 8;
            let byte_mask = 0xff_u32 << shift;
            if let Some(byte) = e.source_memory().bytes.get(&address.wrapping_add(offset)) {
                value = (value & !byte_mask) | (u32::from(*byte) << shift);
                known_mask |= byte_mask;
            } else if slot.known_mask & byte_mask == byte_mask {
                known_mask &= !byte_mask;
            }
        }
        if register < 8 {
            self.data[register] = value;
            self.known_data_bits[register] = known_mask;
        } else {
            self.address[register - 8] = (known_mask == u32::MAX).then_some(value);
        }
        self.offset_address(7, 4)
    }
    pub fn write_memory(
        &self,
        e: &mut dyn crate::recovered_application_control::SourceMemory,
        address: u32,
        bits: u32,
        value: u32,
    ) -> Result<(), RuleError> {
        let bytes = value.to_be_bytes();
        let bytes = &bytes[(4 - bits / 8) as usize..];
        for offset in 0..bytes.len() {
            let at = address.wrapping_add(offset as u32);
            if !self
                .saved_slots
                .iter()
                .any(|slot| at.wrapping_sub(slot.address) < 4)
            {
                e.source_memory().byte(at)?;
            }
        }
        e.source_memory_mut().insert_region(address, bytes);
        e.source_write(RecordWrite {
            address,
            bytes: bytes.to_vec(),
        });
        Ok(())
    }
    pub fn opaque_error(&self, error: RuleError) -> RuleError {
        if let RuleError::Invalid(message) = &error
            && let Some(address) = message
                .strip_prefix("rendering memory unavailable at ")
                .and_then(|address| u32::from_str_radix(address, 16).ok())
            && self
                .saved_slots
                .iter()
                .any(|slot| address.wrapping_sub(slot.address) < 4)
        {
            return RuleError::Unsupported {
                action: "inventory_interaction.items",
                dependency: "reached memory access aliases an opaque native saved-register byte",
            };
        }
        error
    }
    pub fn mask(bits: u32) -> u32 {
        if bits == 32 {
            u32::MAX
        } else {
            (1_u32 << bits) - 1
        }
    }
    pub fn data(&self, register: usize, bits: u32) -> Result<u32, RuleError> {
        let mask = Self::mask(bits);
        if self.known_data_bits[register] & mask != mask {
            return Err(RuleError::Unsupported {
                action: "inventory_interaction.items",
                dependency: "the reached semantic CPU data-register bits are unavailable",
            });
        }
        Ok(self.data[register] & mask)
    }
    pub fn put_data(&mut self, register: usize, bits: u32, value: u32) {
        let mask = Self::mask(bits);
        self.data[register] = (self.data[register] & !mask) | (value & mask);
        self.known_data_bits[register] |= mask;
    }
    pub fn address(&self, register: usize) -> Result<u32, RuleError> {
        self.address[register].ok_or(RuleError::Unsupported {
            action: "inventory_interaction.items",
            dependency: "the reached semantic CPU address register is unavailable",
        })
    }
    pub fn put_address(&mut self, register: usize, value: u32) {
        self.address[register] = Some(value);
    }
    pub fn offset_address(&mut self, register: usize, offset: i32) -> Result<(), RuleError> {
        self.put_address(
            register,
            self.address(register)?.wrapping_add(offset as u32),
        );
        Ok(())
    }
    pub fn flag(&self, bit: u8) -> Result<bool, RuleError> {
        if self.known_ccr & bit == 0 {
            return Err(RuleError::Unsupported {
                action: "inventory_interaction.items",
                dependency: "the reached native condition code is unavailable",
            });
        }
        Ok(self.ccr & bit != 0)
    }
    pub fn put_flag(&mut self, bit: u8, value: bool) {
        self.known_ccr |= bit;
        if value {
            self.ccr |= bit;
        } else {
            self.ccr &= !bit;
        }
    }
    pub fn move_flags(&mut self, value: u32, bits: u32) {
        let value = value & Self::mask(bits);
        self.put_flag(8, value & (1_u32 << (bits - 1)) != 0);
        self.put_flag(4, value == 0);
        self.put_flag(2, false);
        self.put_flag(1, false);
    }
    pub fn add_flags(&mut self, destination: u32, source: u32, bits: u32, extend: bool) -> u32 {
        let mask = Self::mask(bits);
        let destination = destination & mask;
        let source = source & mask;
        let total = u64::from(destination) + u64::from(source);
        let value = total as u32 & mask;
        self.move_flags(value, bits);
        let carry = total > u64::from(mask);
        self.put_flag(1, carry);
        self.put_flag(
            2,
            (!(destination ^ source) & (destination ^ value) & (1_u32 << (bits - 1))) != 0,
        );
        if extend {
            self.put_flag(16, carry);
        }
        value
    }
    pub fn subtract_flags(
        &mut self,
        destination: u32,
        source: u32,
        bits: u32,
        extend: bool,
    ) -> u32 {
        let mask = Self::mask(bits);
        let destination = destination & mask;
        let source = source & mask;
        let value = destination.wrapping_sub(source) & mask;
        self.move_flags(value, bits);
        let carry = destination < source;
        self.put_flag(1, carry);
        self.put_flag(
            2,
            ((destination ^ source) & (destination ^ value) & (1_u32 << (bits - 1))) != 0,
        );
        if extend {
            self.put_flag(16, carry);
        }
        value
    }
    pub fn shift_right(&mut self, source: u32, count: u32, bits: u32, arithmetic: bool) -> u32 {
        let mut value = source & Self::mask(bits);
        let mut carry = false;
        for _ in 0..count {
            carry = value & 1 != 0;
            let sign = if arithmetic {
                value & (1_u32 << (bits - 1))
            } else {
                0
            };
            value = (value >> 1) | sign;
        }
        self.move_flags(value, bits);
        self.put_flag(1, carry);
        if count != 0 {
            self.put_flag(16, carry);
        }
        value
    }
    pub fn shift_left(&mut self, source: u32, count: u32, bits: u32) -> u32 {
        let mask = Self::mask(bits);
        let mut value = source & mask;
        let mut carry = false;
        for _ in 0..count {
            carry = value & (1_u32 << (bits - 1)) != 0;
            value = (value << 1) & mask;
        }
        self.move_flags(value, bits);
        self.put_flag(1, carry);
        if count != 0 {
            self.put_flag(16, carry);
        }
        value
    }
    pub fn carry_clear(&self) -> Result<bool, RuleError> {
        Ok(!self.flag(1)?)
    }
    pub fn lower_equal(&self) -> Result<bool, RuleError> {
        Ok(self.flag(4)? || self.flag(1)?)
    }
    pub fn carry(&self) -> Result<bool, RuleError> {
        self.flag(1)
    }
    pub fn divide_signed(&mut self, dividend: u32, divisor: u32) -> Result<u32, RuleError> {
        let divisor = i64::from(divisor as u16 as i16);
        if divisor == 0 {
            return Err(RuleError::Unsupported {
                action: "inventory_interaction.items",
                dependency: "the original divide-by-zero exception handler",
            });
        }
        let dividend = i64::from(dividend as i32);
        let quotient = dividend / divisor;
        if !(-32768..=32767).contains(&quotient) {
            self.known_ccr &= !(4 | 8);
            self.put_flag(2, true);
            self.put_flag(1, false);
            return Ok(dividend as u32);
        }
        let remainder = dividend % divisor;
        self.move_flags(quotient as u32, 16);
        Ok((u32::from(remainder as u16) << 16) | u32::from(quotient as u16))
    }
    pub fn equal(&self) -> Result<bool, RuleError> {
        self.flag(4)
    }
    pub fn not_equal(&self) -> Result<bool, RuleError> {
        Ok(!self.flag(4)?)
    }
    pub fn greater_equal(&self) -> Result<bool, RuleError> {
        Ok(self.flag(8)? == self.flag(2)?)
    }
    pub fn less(&self) -> Result<bool, RuleError> {
        Ok(self.flag(8)? != self.flag(2)?)
    }
    pub fn less_equal(&self) -> Result<bool, RuleError> {
        Ok(self.flag(4)? || self.less()?)
    }
    pub fn greater(&self) -> Result<bool, RuleError> {
        Ok(!self.flag(4)? && self.greater_equal()?)
    }
    pub fn higher(&self) -> Result<bool, RuleError> {
        Ok(!self.flag(1)? && !self.flag(4)?)
    }
    pub fn after_call(&mut self, d0: Option<u32>, returned: PromptReturn) {
        for register in 0..=2 {
            self.known_data_bits[register] = 0;
        }
        self.address[0] = None;
        self.address[1] = None;
        self.known_ccr = 0;
        if let Some(value) = d0 {
            self.put_data(0, 32, value);
        } else if let PromptReturn::LowWord(value) = returned {
            self.put_data(0, 16, u32::from(value));
        }
    }
    pub fn return_value(&self) -> Result<PromptReturn, RuleError> {
        if self.known_data_bits[0] == u32::MAX {
            Ok(PromptReturn::Full(self.data[0]))
        } else {
            Ok(PromptReturn::LowWord(self.data(0, 16)? as u16))
        }
    }
    pub fn snapshot(&self) -> ItemsRegisterObservation {
        ItemsRegisterObservation {
            data: self.data,
            known_data_bits: self.known_data_bits,
            address: self.address,
            condition_codes: self.ccr,
            known_condition_codes: self.known_ccr,
        }
    }
}
