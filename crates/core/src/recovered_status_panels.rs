//! Status panels and selection menus from complete CODE 1 bodies.

use crate::recovered_attack::DamageFrame;
use crate::recovered_audio::{self, AudioContext, AudioEffect, AudioObservation};
use crate::recovered_bodies::{self, BodyEffect, BodyGroundInput, BodyState};
use crate::recovered_combat_panel::{self, CombatPanelEffect, CombatPanelServices};
use crate::recovered_drawing::{self, BuildingWorld, DrawingContext, DrawingEffect};
use crate::recovered_equipment::{self, EquipmentEffect, EquipmentLimits, EquipmentServices};
use crate::recovered_equipment_ui::{self, EquipmentUiEffect};
use crate::recovered_feedback::{self, FeedbackArg, FeedbackEffect};
use crate::recovered_inventory::{self, CompactState};
use crate::recovered_inventory_rules::{self, InventoryEffect};
use crate::recovered_records::{self, ItemTables, RecordWrite};
use crate::recovered_rendering::{self, RenderingEffect, RenderingMemory, RenderingServices};
use crate::recovered_runtime::{
    self, RuntimeContext, RuntimeEffect, RuntimeMemory, RuntimeObservation,
};
use crate::recovered_text::{self, NumericServices, PascalString, TextContext, TextEffect};
use crate::{RandomState, RuleError, rules};
use centurions_content::{ITEM_BYTES, WARRIOR_BYTES, Warrior};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StatusPanelRequest {
    /// Arguments retain native Pascal push order. D0 input is used by FlushEvents.
    Toolbox {
        opcode: u16,
        arguments: Vec<FeedbackArg>,
        d0_input: Option<u32>,
    },
    /// Arguments retain the source import's right-to-left stack push order.
    Imported {
        address: u32,
        arguments: Vec<FeedbackArg>,
    },
    NumberToString {
        value: i32,
        destination: u32,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusPanelObservation {
    pub value: u32,
    pub d0: u32,
    pub memory: RenderingMemory,
}

pub trait StatusPanelServices: EquipmentServices + CombatPanelServices + RenderingServices {
    fn observe_status(
        &self,
        request: &StatusPanelRequest,
        memory: &RenderingMemory,
    ) -> Result<StatusPanelObservation, RuleError>;
    /// Original input view for the real bodyground helper; selected bytes are checked.
    fn body_ground_state(&self, memory: &RenderingMemory) -> Result<BodyState, RuleError>;
    fn quiet_observations(
        &self,
        selector: i16,
        context: &AudioContext,
        memory: &RenderingMemory,
    ) -> Result<Vec<AudioObservation>, RuleError>;
    fn heap_observations(
        &self,
        context: &RuntimeContext,
    ) -> Result<Vec<RuntimeObservation>, RuleError>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StatusPanelReturn {
    /// None means a void/captured child has not established the register half.
    pub low_word: Option<u16>,
    pub upper_word: Option<u16>,
}
impl StatusPanelReturn {
    fn full(value: u32) -> Self {
        Self {
            low_word: Some(value as u16),
            upper_word: Some((value >> 16) as u16),
        }
    }
    fn unknown() -> Self {
        Self {
            low_word: None,
            upper_word: None,
        }
    }
    pub fn known_long(self) -> Option<u32> {
        Some((u32::from(self.upper_word?) << 16) | u32::from(self.low_word?))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum StatusPanelEffect {
    Write(RecordWrite),
    Service {
        request: StatusPanelRequest,
        observation: StatusPanelObservation,
    },
    Drawing(DrawingEffect),
    Text(TextEffect),
    Feedback(FeedbackEffect),
    Audio(AudioEffect),
    Runtime(RuntimeEffect),
    Equipment(EquipmentEffect),
    EquipmentUi(EquipmentUiEffect),
    Inventory(InventoryEffect),
    Body(BodyEffect),
    Rendering(RenderingEffect),
    CombatPanel(CombatPanelEffect),
    Selection {
        input: i16,
        base: i16,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct StatusPanelPlan {
    pub memory: RenderingMemory,
    pub random: RandomState,
    pub effects: Vec<StatusPanelEffect>,
    pub native_return: StatusPanelReturn,
    /// Typed children retain their captured views and numeric/drawing contracts.
    pub qualification: StatusPanelQualification,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusPanelQualification {
    SourcePortWithObservedBoundariesAndProjectedChildren,
}

impl StatusPanelPlan {
    fn new(memory: &RenderingMemory, random: &RandomState) -> Self {
        Self {
            memory: memory.clone(),
            random: random.clone(),
            effects: vec![],
            native_return: StatusPanelReturn::unknown(),
            qualification:
                StatusPanelQualification::SourcePortWithObservedBoundariesAndProjectedChildren,
        }
    }
    fn bytes(&self, a: u32, n: usize) -> Result<Vec<u8>, RuleError> {
        equipment_read(&self.memory, a, n)
    }
    fn b(&self, a: u32) -> Result<u8, RuleError> {
        self.memory.byte(a)
    }
    fn w(&self, a: u32) -> Result<i16, RuleError> {
        Ok(self.memory.word(a)? as i16)
    }
    fn l(&self, a: u32) -> Result<u32, RuleError> {
        self.memory.long(a)
    }
    fn write(&mut self, a: u32, bytes: &[u8]) -> Result<(), RuleError> {
        self.bytes(a, bytes.len())?;
        self.memory.insert_region(a, bytes);
        self.effects.push(StatusPanelEffect::Write(RecordWrite {
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
    fn add(&mut self, a: u32, value: i16) -> Result<(), RuleError> {
        self.ww(a, self.w(a)?.wrapping_add(value))
    }
    fn observe(
        &mut self,
        request: StatusPanelRequest,
        services: &impl StatusPanelServices,
    ) -> Result<u32, RuleError> {
        let observation = services.observe_status(&request, &self.memory)?;
        let value = observation.value;
        self.memory = observation.memory.clone();
        self.native_return = StatusPanelReturn::full(observation.d0);
        self.effects.push(StatusPanelEffect::Service {
            request,
            observation,
        });
        Ok(value)
    }
    fn toolbox(
        &mut self,
        opcode: u16,
        arguments: Vec<FeedbackArg>,
        services: &impl StatusPanelServices,
    ) -> Result<u32, RuleError> {
        self.observe(
            StatusPanelRequest::Toolbox {
                opcode,
                arguments,
                d0_input: None,
            },
            services,
        )
    }
    fn imported(
        &mut self,
        address: u32,
        arguments: Vec<FeedbackArg>,
        services: &impl StatusPanelServices,
    ) -> Result<u32, RuleError> {
        self.observe(
            StatusPanelRequest::Imported { address, arguments },
            services,
        )
    }
    fn flush(&mut self, services: &impl StatusPanelServices) -> Result<(), RuleError> {
        self.observe(
            StatusPanelRequest::Toolbox {
                opcode: 0xa032,
                arguments: vec![],
                d0_input: Some(0xffff),
            },
            services,
        )?;
        Ok(())
    }
    fn sound(&mut self, id: i16, services: &impl StatusPanelServices) -> Result<(), RuleError> {
        let child = recovered_feedback::sound(&self.memory, id, services)?;
        self.memory = child.memory;
        self.effects
            .extend(child.effects.into_iter().map(StatusPanelEffect::Feedback));
        self.native_return = child
            .native_return
            .map_or_else(StatusPanelReturn::unknown, StatusPanelReturn::full);
        Ok(())
    }
    fn load_item(&mut self, id: i16) -> Result<(), RuleError> {
        let backing = recovered_equipment::item_tables(&self.memory, id)?;
        let tables = ItemTables {
            families: std::array::from_fn(|i| backing[i].as_slice()),
        };
        let current = self
            .bytes(0x45363c, ITEM_BYTES)?
            .try_into()
            .expect("item record");
        let child = recovered_records::loaditem(&tables, &current, id)?;
        for effect in child.writes {
            self.write(effect.address, &effect.bytes)?;
        }
        self.native_return = StatusPanelReturn::unknown();
        Ok(())
    }
    fn selection(
        &mut self,
        plus_one: bool,
        services: &impl StatusPanelServices,
    ) -> Result<(), RuleError> {
        let input = self.w(0x4536a2)?;
        let base = rules::selection_base(input);
        self.ww(0x4471e8, base)?;
        self.effects
            .push(StatusPanelEffect::Selection { input, base });
        let index = self
            .w(0x4536a2)?
            .wrapping_sub(self.w(0x4471e8)?)
            .wrapping_add(1);
        self.indexed(base.wrapping_add(i16::from(plus_one)), index, services)
    }
    fn indexed(
        &mut self,
        resource: i16,
        index: i16,
        services: &impl StatusPanelServices,
    ) -> Result<(), RuleError> {
        self.imported(
            0x400178,
            vec![ptr(0x479ed8), word(resource), word(index)],
            services,
        )?;
        Ok(())
    }
    fn copy22(&mut self, source: u32) -> Result<(), RuleError> {
        for offset in [0, 4, 8, 12, 16] {
            let value = self.l(source.wrapping_add(offset))?;
            self.lw(0x41fc48 + offset, value)?;
        }
        self.ww(0x41fc5c, self.w(source.wrapping_add(20))?)
    }
    fn drawing_effects(&mut self, effects: Vec<DrawingEffect>) -> Result<(), RuleError> {
        for effect in effects {
            if let DrawingEffect::Write(ref write) = effect {
                self.write(write.address, &write.bytes)?;
            } else {
                self.effects.push(StatusPanelEffect::Drawing(effect));
            }
        }
        self.native_return = StatusPanelReturn::full(0);
        Ok(())
    }
    fn item_context(&self, id: i16) -> Result<DrawingContext, RuleError> {
        let mut context = blank_drawing(self.memory.rect(0x4471f2)?);
        context.item_work = self.bytes(0x45363c, ITEM_BYTES)?.try_into().expect("item");
        context.window = self.l(0x439404)?;
        context.item_window = self.l(0x4393fc)?;
        context.half_width_input = self.w(0x40856c)?;
        context.omit_banner = self.b(0x442756)?;
        if context.omit_banner == 0 {
            context.banner_rect = self.memory.rect(0x4471ac)?;
        }
        if id == 0 && context.window == context.item_window {
            context.active_member = self.b(0x44273e)? as i8;
        }
        context.show_item_background = self.b(0x44275a)?;
        context.text_in_other_port = self.b(0x442750)?;
        context.erase_pattern = self.l(0x4358da)?;
        context.default_font = self.w(0x402f3c)?;
        if context.text_in_other_port != 0 {
            context.text_port = self.l(0x438798)?;
        }
        Ok(context)
    }
    fn plot_item(&mut self, services: &mut impl StatusPanelServices) -> Result<(), RuleError> {
        let id = self.w(0x4434b4)?;
        let backing = recovered_equipment::item_tables(&self.memory, id)?;
        let tables = ItemTables {
            families: std::array::from_fn(|i| backing[i].as_slice()),
        };
        let mut context = self.item_context(id)?;
        let loaded = recovered_records::loaditemnotype(&tables, &context.item_work, id)?;
        let item_id = i16::from_be_bytes([loaded.bytes[102], loaded.bytes[103]]);
        context.selection = rules::selection_base(item_id);
        let child = recovered_drawing::ploticon(&mut context, &tables, id, services)?;
        for effect in child.effects {
            if matches!(effect, DrawingEffect::GetIndexedString { .. }) {
                self.ww(0x4471e8, context.selection)?;
            }
            if let DrawingEffect::Write(ref write) = effect {
                self.write(write.address, &write.bytes)?;
            } else {
                self.effects.push(StatusPanelEffect::Drawing(effect));
            }
        }
        self.native_return = StatusPanelReturn::full(child.native_return as u32);
        Ok(())
    }
    fn show_charge(
        &mut self,
        frame: u32,
        charges: i16,
        services: &mut impl StatusPanelServices,
    ) -> Result<(), RuleError> {
        self.ww(frame.wrapping_add(10), 0)?;
        self.ww(frame.wrapping_add(8), charges)?;
        let child = recovered_equipment_ui::show_charge(&self.memory, frame, services)?;
        self.memory = child.memory;
        self.effects.extend(
            child
                .effects
                .into_iter()
                .map(StatusPanelEffect::EquipmentUi),
        );
        self.native_return = StatusPanelReturn::full(child.native_return);
        Ok(())
    }
    fn shop_render(
        &mut self,
        frame: u32,
        charges: i16,
        services: &mut impl StatusPanelServices,
    ) -> Result<(), RuleError> {
        self.ww(frame.wrapping_sub(22), self.w(0x4434b4)?)?;
        self.plot_item(services)?;
        if self.w(0x4434b4)? != 0 {
            self.show_charge(frame.wrapping_sub(32), charges, services)?;
        }
        self.write(0x44275a, &[0])
    }
    fn dialog_item(
        &mut self,
        item: i16,
        services: &impl StatusPanelServices,
    ) -> Result<(), RuleError> {
        self.toolbox(
            0xa98d,
            vec![
                long(self.l(0x4394b8)?),
                word(item),
                ptr(0x4434b0),
                ptr(0x4434dc),
                ptr(0x4434ba),
            ],
            services,
        )?;
        Ok(())
    }
    fn pascal(&self, address: u32) -> Result<PascalString, RuleError> {
        PascalString::from_raw(&self.bytes(address, usize::from(self.b(address)?) + 1)?)
    }
    fn dialog_string(
        &mut self,
        item: i16,
        address: u32,
        services: &impl StatusPanelServices,
    ) -> Result<(), RuleError> {
        self.dialog_item(item, services)?;
        let effects = recovered_text::dialog_string(item, &self.pascal(address)?);
        self.effects
            .extend(effects.into_iter().map(StatusPanelEffect::Text));
        self.toolbox(
            0xa98f,
            vec![long(self.l(0x4434dc)?), ptr(address)],
            services,
        )?;
        self.native_return = StatusPanelReturn::full(0);
        Ok(())
    }
    fn dialog_number(
        &mut self,
        item: i16,
        value: i16,
        services: &impl StatusPanelServices,
    ) -> Result<(), RuleError> {
        self.dialog_item(item, services)?;
        let mut numeric = StatusNumeric {
            services,
            memory: self.memory.clone(),
            observation: None,
        };
        let effects =
            recovered_text::dialog_number(&mut TextContext::default(), item, value, &mut numeric)?;
        let observation = numeric
            .observation
            .ok_or_else(|| RuleError::Invalid("missing status conversion observation".into()))?;
        self.memory = observation.memory.clone();
        self.effects.push(StatusPanelEffect::Service {
            request: StatusPanelRequest::NumberToString {
                value: i32::from(value),
                destination: 0x479ed8,
            },
            observation,
        });
        self.effects
            .extend(effects.into_iter().map(StatusPanelEffect::Text));
        self.toolbox(
            0xa98f,
            vec![long(self.l(0x4434dc)?), ptr(0x479ed8)],
            services,
        )?;
        self.native_return = StatusPanelReturn::full(0);
        Ok(())
    }
    fn compact_items(&mut self, member: u16) -> Result<(), RuleError> {
        let selected = member as u8 as i8;
        let highest = self.b(0x44273a)? as i8;
        if selected < 0 || selected > highest {
            self.native_return = StatusPanelReturn::unknown();
            return Ok(());
        }
        let base = member_address(selected as u16);
        let state = CompactState {
            warrior: Warrior {
                name: String::new(),
                original_sha256: String::new(),
                bytes: self.bytes(base, WARRIOR_BYTES)?,
            },
            highest_party_index: highest,
            scan: self.w(0x4434aa)?,
            pass: self.w(0x4434ac)?,
        };
        let child = recovered_inventory::compactitems(&state, selected)?;
        for effect in child.effects {
            match effect {
                recovered_inventory::InventoryEffect::Write(write) => {
                    self.write(write.address, &write.bytes)?
                }
                _ => {
                    return Err(RuleError::Invalid(
                        "unexpected compactitems child effect".into(),
                    ));
                }
            }
        }
        self.native_return = StatusPanelReturn::unknown();
        Ok(())
    }
    fn panel(
        &mut self,
        frame: u32,
        body: i16,
        damage: DamageFrame,
        services: &mut impl StatusPanelServices,
    ) -> Result<(), RuleError> {
        self.ww(frame.wrapping_add(8), body)?;
        let child =
            recovered_combat_panel::combat_update2(&self.memory, frame, body, damage, services)?;
        self.memory = child.memory;
        self.effects.extend(
            child
                .effects
                .into_iter()
                .map(StatusPanelEffect::CombatPanel),
        );
        self.native_return = StatusPanelReturn::full(child.native_return as u32);
        Ok(())
    }
    fn sprite(
        &mut self,
        frame: u32,
        body: i16,
        force: i16,
        services: &impl StatusPanelServices,
    ) -> Result<(), RuleError> {
        self.ww(frame.wrapping_add(12), 0)?;
        self.ww(frame.wrapping_add(10), force)?;
        self.ww(frame.wrapping_add(8), body)?;
        let child = recovered_rendering::drawbody(&self.memory, frame, body, force, 0, services)?;
        self.memory = child.memory;
        self.effects
            .extend(child.effects.into_iter().map(StatusPanelEffect::Rendering));
        self.native_return = StatusPanelReturn::unknown();
        Ok(())
    }
}

fn equipment_read(memory: &RenderingMemory, a: u32, n: usize) -> Result<Vec<u8>, RuleError> {
    (0..n)
        .map(|i| memory.byte(a.wrapping_add(i as u32)))
        .collect()
}
fn word(value: i16) -> FeedbackArg {
    FeedbackArg::Word(value as u16)
}
fn long(value: u32) -> FeedbackArg {
    FeedbackArg::Long(value)
}
fn ptr(value: u32) -> FeedbackArg {
    FeedbackArg::Pointer(value)
}
fn byte(value: u8) -> FeedbackArg {
    FeedbackArg::Byte(value)
}
fn indexed(base: u32, index: i16) -> u32 {
    base.wrapping_add((i32::from(index) * 2) as u32)
}
fn member_address(member: u16) -> u32 {
    0x448aa0_u32.wrapping_add(u32::from(member) * 3138)
}
fn node_address(member: u16, slot: u16) -> u32 {
    member_address(member)
        .wrapping_add(u32::from(slot) * 22)
        .wrapping_add(0x132)
}
fn half(value: i16) -> i16 {
    value.wrapping_add(((value as u16) >> 15) as i16) >> 1
}
fn step(remaining: &mut usize) -> Result<(), RuleError> {
    if *remaining == 0 {
        return Err(RuleError::Invalid(
            "status panel iteration limit exhausted".into(),
        ));
    }
    *remaining -= 1;
    Ok(())
}
fn blank_drawing(rect: crate::recovered_geometry::BodyRect) -> DrawingContext {
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
struct StatusNumeric<'a, S> {
    services: &'a S,
    memory: RenderingMemory,
    observation: Option<StatusPanelObservation>,
}
impl<S: StatusPanelServices> NumericServices for StatusNumeric<'_, S> {
    fn number_to_string(&mut self, value: i32) -> Result<PascalString, RuleError> {
        let observation = self.services.observe_status(
            &StatusPanelRequest::NumberToString {
                value,
                destination: 0x479ed8,
            },
            &self.memory,
        )?;
        let n = usize::from(observation.memory.byte(0x479ed8)?) + 1;
        let result = PascalString::from_raw(&equipment_read(&observation.memory, 0x479ed8, n)?)?;
        self.observation = Some(observation);
        Ok(result)
    }
    fn string_to_number(&mut self, _: &PascalString) -> Result<i32, RuleError> {
        Err(RuleError::Invalid(
            "status panel never parses numeric text".into(),
        ))
    }
}

fn stock(e: &mut StatusPanelPlan, extra: i16, charges: &mut i16) -> Result<(), RuleError> {
    e.ww(0x4434b4, 0)?;
    let index = e
        .w(0x4470b2)?
        .wrapping_add(e.w(0x4471a8)?)
        .wrapping_add(extra);
    if e.b(0x478a58_u32.wrapping_add(i32::from(index) as u32))? != 0 {
        e.ww(0x4434b4, e.w(indexed(0x478288, index))?)?;
    }
    if e.w(0x4434b4)? < 2000 {
        e.load_item(e.w(0x4434b4)?)?;
        e.ww(0x41fc48, e.w(0x4536a2)?)?;
        e.ww(0x41fc4e, e.w(0x453650)?)?;
        e.ww(0x41fc4a, e.w(0x453650)?)?;
        e.write(0x41fc51, &[0])?;
        e.ww(0x41fc4c, 0)?;
        for (source, destination) in [
            (0x4536c7, 0x41fc55),
            (0x45365f, 0x41fc50),
            (0x453657, 0x41fc52),
            (0x45368a, 0x41fc56),
            (0x45368d, 0x41fc57),
            (0x45368f, 0x41fc58),
            (0x453690, 0x41fc59),
            (0x453691, 0x41fc5a),
            (0x453692, 0x41fc5b),
            (0x45368c, 0x41fc5d),
            (0x45368b, 0x41fc5c),
        ] {
            e.write(destination, &[e.b(source)?])?;
        }
        *charges = e.w(0x453650)?;
    }
    Ok(())
}
fn inventory_row(
    e: &mut StatusPanelPlan,
    base: u32,
    owner: u32,
    row: i16,
    refresh_flag: bool,
    charges: &mut i16,
) -> Result<(), RuleError> {
    let node = base
        .wrapping_add(u32::from(row as u16) * 22)
        .wrapping_add(0x132);
    e.ww(0x4434b4, e.w(node)?)?;
    *charges = e.w(node.wrapping_add(2))?;
    if refresh_flag {
        e.write(0x44275a, &[e.b(node.wrapping_add(9))?])?;
    }
    e.write(0x41f708, &[row as u8])?;
    e.write(0x41f709, &[e.b(owner)?])?;
    let selected = e.b(0x41f708)? as i8 as i16 as u16;
    e.copy22(
        base.wrapping_add(u32::from(selected) * 22)
            .wrapping_add(0x132),
    )
}
fn up_row(e: &mut StatusPanelPlan, charges: &mut i16) -> Result<(), RuleError> {
    if e.w(0x43875c)? == 1 {
        stock(e, 0, charges)?;
    }
    if e.w(0x43875c)? == 0 && (e.b(0x44273e)? as i8) > -1 {
        inventory_row(e, 0x447e5e, 0x44273e, e.w(0x4470b2)?, true, charges)?;
    }
    if e.l(0x439404)? == e.l(0x439400)? {
        inventory_row(e, 0x44721c, 0x44273d, e.w(0x4470b2)?, true, charges)?;
    }
    Ok(())
}
fn scroll(
    e: &mut StatusPanelPlan,
    rect: u32,
    vertical: i16,
    services: &impl StatusPanelServices,
) -> Result<(), RuleError> {
    e.toolbox(
        0xa8ef,
        vec![ptr(rect), word(0), word(vertical), long(0)],
        services,
    )?;
    Ok(())
}

/// updateshop, 0x139244. Incoming D3 supplies the untouched single-step charge scratch.
pub fn update_shop(
    memory: &RenderingMemory,
    random: &RandomState,
    frame: u32,
    incoming_d3: i16,
    maximum_steps: usize,
    services: &mut impl StatusPanelServices,
) -> Result<StatusPanelPlan, RuleError> {
    let mut e = StatusPanelPlan::new(memory, random);
    e.l(frame.wrapping_add(10))?;
    for (opcode, destination) in [(0xa960, 0x4470b2), (0xa962, 0x4470b6), (0xa961, 0x4470b4)] {
        let value = e.toolbox(opcode, vec![long(e.l(0x439404)?)], services)? as i16;
        e.ww(destination, value)?;
    }
    let rect = frame.wrapping_sub(8);
    e.ww(rect.wrapping_add(2), half(e.w(0x40856c)?).wrapping_add(390))?;
    if e.l(0x439404)? == e.l(0x439400)? {
        e.ww(rect.wrapping_add(2), 10)?;
    }
    e.ww(
        rect.wrapping_add(6),
        e.w(rect.wrapping_add(2))?
            .wrapping_add(250)
            .wrapping_add(half(e.w(0x40856c)?)),
    )?;
    let operation = e.w(frame.wrapping_add(8))?;
    let mut remaining = maximum_steps;
    let mut charges = incoming_d3;
    match operation {
        20 => {
            e.ww(rect, 6)?;
            e.ww(rect.wrapping_add(4), 368)?;
            if e.w(0x4470b2)? > e.w(0x4470b4)? || e.w(0x438756)? == 1 {
                e.add(0x4470b2, -1)?;
                e.ww(0x4434aa, 0)?;
                while e.w(0x4434aa)? < 4 {
                    step(&mut remaining)?;
                    scroll(&mut e, rect, 10, services)?;
                    e.add(0x4434aa, 1)?;
                }
                e.ww(0x4471f2, 16)?;
                up_row(&mut e, &mut charges)?;
                e.shop_render(frame, charges, services)?;
            }
        }
        21 => {
            e.ww(rect, 16)?;
            e.ww(rect.wrapping_add(4), 386)?;
            if e.w(0x4470b2)? < e.w(0x4470b6)? || e.w(0x438756)? == 1 {
                e.add(0x4470b2, 1)?;
                let mut should_scroll = true;
                if e.l(0x439404)? == e.l(0x439400)? && e.w(0x447236)? < 10 {
                    should_scroll = false;
                }
                if e.l(0x439404)? == e.l(0x4393fc)? && e.w(0x447e78)? < 9 && e.b(0x44273e)? != 0xff
                {
                    should_scroll = false;
                }
                if should_scroll {
                    e.ww(0x4434aa, 0)?;
                    while e.w(0x4434aa)? < 4 {
                        step(&mut remaining)?;
                        scroll(&mut e, rect, -10, services)?;
                        e.add(0x4434aa, 1)?;
                    }
                }
                e.ww(0x4471f2, 336)?;
                if e.l(0x439404)? == e.l(0x4393fc)? {
                    if e.b(0x44273e)? == 0xff {
                        stock(&mut e, 8, &mut charges)?;
                    } else {
                        let top = e.w(0x447e78)?.wrapping_mul(40).wrapping_add(16);
                        e.ww(0x4471f2, top)?;
                        if e.w(0x4471f2)? > 336 {
                            e.ww(0x4471f2, 336)?;
                        }
                        let row = e.w(0x4470b2)?.wrapping_add(8);
                        inventory_row(&mut e, 0x447e5e, 0x44273e, row, true, &mut charges)?;
                    }
                } else {
                    let top = e.w(0x447236)?.wrapping_mul(40).wrapping_sub(24);
                    e.ww(0x4471f2, top)?;
                    if e.w(0x4471f2)? > 336 {
                        e.ww(0x4471f2, 336)?;
                    }
                    let row = e.w(0x4470b2)?.wrapping_add(8);
                    inventory_row(&mut e, 0x44721c, 0x44273d, row, true, &mut charges)?;
                    if e.w(0x438756)? == 1 {
                        let row = e.w(0x447236)?.wrapping_sub(1);
                        inventory_row(&mut e, 0x44721c, 0x44273d, row, false, &mut charges)?;
                    }
                }
                e.shop_render(frame, charges, services)?;
            }
        }
        22 | 23 => {
            let up = operation == 22;
            e.ww(rect, if up { 6 } else { 16 })?;
            e.ww(rect.wrapping_add(4), if up { 368 } else { 386 })?;
            e.ww(0x4434ac, 0)?;
            while e.w(0x4434ac)? < 9 {
                step(&mut remaining)?;
                charges = 0;
                let eligible = if up {
                    e.w(0x4470b2)? > e.w(0x4470b4)?
                } else {
                    e.w(0x4470b2)? < e.w(0x4470b6)?
                };
                if eligible {
                    e.add(0x4470b2, if up { -1 } else { 1 })?;
                    scroll(&mut e, rect, if up { 40 } else { -40 }, services)?;
                    e.ww(0x4471f2, if up { 16 } else { 336 })?;
                    if up {
                        up_row(&mut e, &mut charges)?;
                    } else {
                        if e.w(0x43875c)? == 1 {
                            stock(&mut e, 8, &mut charges)?;
                        }
                        if e.w(0x43875c)? == 0 && (e.b(0x44273e)? as i8) > -1 {
                            let row = e.w(0x4470b2)?.wrapping_add(8);
                            inventory_row(&mut e, 0x447e5e, 0x44273e, row, true, &mut charges)?;
                        }
                        if e.l(0x439404)? == e.l(0x439400)? {
                            let row = e.w(0x4470b2)?.wrapping_add(8);
                            inventory_row(&mut e, 0x44721c, 0x44273d, row, true, &mut charges)?;
                        }
                        if e.w(0x438756)? == 1 {
                            let idrow = e.w(0x447236)?.wrapping_sub(1) as u16;
                            e.ww(
                                0x4434b4,
                                e.w(0x44734e_u32.wrapping_add(u32::from(idrow) * 22))?,
                            )?;
                            let row = e.w(0x4470b2)?.wrapping_sub(1);
                            let node = 0x44734e_u32.wrapping_add(u32::from(row as u16) * 22);
                            charges = e.w(node.wrapping_add(2))?;
                            e.write(0x44275a, &[e.b(node.wrapping_add(9))?])?;
                            e.write(0x41f708, &[row as u8])?;
                            e.write(0x41f709, &[e.b(0x44273d)?])?;
                            let copyrow = e.b(0x41f708)? as i8 as i16 as u16;
                            e.copy22(0x44734e_u32.wrapping_add(u32::from(copyrow) * 22))?;
                        }
                    }
                    e.shop_render(frame, charges, services)?;
                }
                e.add(0x4434ac, 1)?;
            }
        }
        _ => (),
    }
    e.toolbox(
        0xa963,
        vec![long(e.l(0x439404)?), word(e.w(0x4470b2)?)],
        services,
    )?;
    Ok(e)
}

fn menu_text(
    e: &mut StatusPanelPlan,
    frame: u32,
    member: u16,
    ranged: bool,
    services: &impl StatusPanelServices,
) -> Result<(), RuleError> {
    e.selection(true, services)?;
    e.imported(0x400180, vec![ptr(0x479ed8)], services)?;
    let node = node_address(member, e.w(0x4434aa)? as u16);
    if ranged || e.w(node.wrapping_add(2))? > 0 {
        e.imported(
            0x400098,
            vec![ptr(if ranged { 0x403d22 } else { 0x403cfa }), ptr(0x479ed8)],
            services,
        )?;
        let node = node_address(member, e.w(0x4434aa)? as u16);
        let quantity = i32::from(e.w(node.wrapping_add(2))?);
        let local = frame.wrapping_sub(0x100);
        e.imported(0x400168, vec![long(quantity as u32), ptr(local)], services)?;
        e.imported(0x400180, vec![ptr(local)], services)?;
        e.imported(0x400098, vec![ptr(local), ptr(0x479ed8)], services)?;
        e.imported(
            0x400098,
            vec![ptr(if ranged { 0x403d1c } else { 0x403cf4 }), ptr(0x479ed8)],
            services,
        )?;
    }
    e.imported(0x400188, vec![ptr(0x479ed8)], services)?;
    Ok(())
}

/// changeweapon, 0x1174f0. Local mapping entries are actual addressed WORDs.
pub fn change_weapon(
    memory: &RenderingMemory,
    random: &RandomState,
    frame: u32,
    limits: EquipmentLimits,
    damage: DamageFrame,
    services: &mut impl StatusPanelServices,
) -> Result<StatusPanelPlan, RuleError> {
    let mut e = StatusPanelPlan::new(memory, random);
    let member = e.w(frame.wrapping_add(8))? as u16;
    let warrior = member_address(member);
    let menu = e.toolbox(0xa9bf, vec![word(1000)], services)?;
    e.toolbox(0xa935, vec![long(menu), word(-1)], services)?;
    e.toolbox(0xa947, vec![long(menu), word(1), ptr(0x403d28)], services)?;
    let mut count = 0i16;
    let mut alternate = 0i16;
    let mut primary = 0i16;
    let mut remaining = limits.maximum_steps;
    for ranged in [true, false] {
        if !ranged {
            e.toolbox(0xa933, vec![long(menu), ptr(0x403cfc)], services)?;
            count = count.wrapping_add(1);
        }
        e.ww(0x4434aa, 0)?;
        while e.w(0x4434aa)? <= e.w(warrior.wrapping_add(0x1a))? {
            step(&mut remaining)?;
            let node = node_address(member, e.w(0x4434aa)? as u16);
            let kind = e.b(node.wrapping_add(10))? as i8;
            let wanted = if ranged { 15 } else { 2 };
            if kind == wanted || kind == -wanted {
                let id = e.w(node)?;
                e.load_item(id)?;
                let mapping = indexed(frame.wrapping_sub(0x164), count);
                count = count.wrapping_add(1);
                e.ww(mapping, e.w(0x4434aa)?)?;
                menu_text(&mut e, frame, member, ranged, services)?;
                e.toolbox(0xa933, vec![long(menu), ptr(0x479ed8)], services)?;
                let node = node_address(member, e.w(0x4434aa)? as u16);
                if e.w(node)? == e.w(warrior.wrapping_add(if ranged { 0xe4 } else { 0xca }))?
                    && e.b(node.wrapping_add(9))? != 0
                {
                    if ranged {
                        alternate = e.w(0x4434aa)?.wrapping_add(1);
                    }
                    e.toolbox(
                        0xa945,
                        vec![long(menu), word(count.wrapping_add(1)), byte(0x13)],
                        services,
                    )?;
                    if !ranged {
                        primary = e.w(0x4434aa)?.wrapping_add(1);
                    }
                }
            }
            e.add(0x4434aa, 1)?;
        }
    }
    e.toolbox(0xa972, vec![ptr(0x4471ee)], services)?;
    e.toolbox(0xa870, vec![ptr(0x4471ee)], services)?;
    let selection = e.toolbox(
        0xa80b,
        vec![
            long(menu),
            word(e.w(0x4471ee)?.wrapping_sub(150)),
            word(e.w(0x4471f0)?),
            word(alternate),
        ],
        services,
    )?;
    e.ww(0x4434b2, selection as i16)?;
    e.toolbox(0xa936, vec![word(1000)], services)?;
    e.toolbox(0xa9a3, vec![long(menu)], services)?;
    if e.w(0x4434b2)? > 1 {
        e.sound(405, services)?;
        let slot = e.w(indexed(
            frame.wrapping_sub(0x164),
            e.w(0x4434b2)?.wrapping_sub(2),
        ))? as u16;
        let kind = e.b(node_address(member, slot).wrapping_add(10))?;
        let remove = if kind == 2 && e.w(warrior.wrapping_add(0xca))? != 0 {
            Some(primary.wrapping_sub(1))
        } else {
            let slot = e.w(indexed(
                frame.wrapping_sub(0x164),
                e.w(0x4434b2)?.wrapping_sub(2),
            ))? as u16;
            if e.b(node_address(member, slot).wrapping_add(10))? == 15
                && e.w(warrior.wrapping_add(0xe4))? != 0
            {
                Some(alternate.wrapping_sub(1))
            } else {
                None
            }
        };
        if let Some(slot) = remove {
            let child_frame = frame.wrapping_sub(392);
            e.lw(child_frame.wrapping_add(12), 0)?;
            e.ww(child_frame.wrapping_add(10), slot)?;
            e.ww(child_frame.wrapping_add(8), member as i16)?;
            let child = recovered_inventory_rules::removeitem(
                &e.memory,
                child_frame,
                limits.maximum_polls,
                services,
            )?;
            e.memory = child.memory;
            e.effects
                .extend(child.effects.into_iter().map(StatusPanelEffect::Inventory));
            e.native_return = StatusPanelReturn::full(child.native_return);
        }
        let emit = 1i16.wrapping_sub(e.w(0x438776)?);
        let slot = e.w(indexed(
            frame.wrapping_sub(0x164),
            e.w(0x4434b2)?.wrapping_sub(2),
        ))?;
        let child_frame = frame.wrapping_sub(390);
        e.ww(child_frame.wrapping_add(12), emit)?;
        e.ww(child_frame.wrapping_add(10), slot)?;
        e.ww(child_frame.wrapping_add(8), member as i16)?;
        let child = recovered_equipment::wear(&e.memory, &e.random, child_frame, limits, services)?;
        e.memory = child.memory;
        e.random = child.random;
        e.effects
            .extend(child.effects.into_iter().map(StatusPanelEffect::Equipment));
        e.native_return = StatusPanelReturn {
            low_word: Some(child.native_return.low_word),
            upper_word: child.native_return.upper_word,
        };
        if e.w(0x438776)? != 0 {
            e.add(warrior.wrapping_add(0x40), -1)?;
            e.compact_items(member)?;
            e.ww(0x4471ea, 1)?;
        }
    } else {
        e.ww(0x4471ea, 0)?;
    }
    if e.w(0x438776)? != 0 {
        let front = e.toolbox(0xa924, vec![], services)?;
        if front == e.l(0x43879c)? {
            e.panel(frame.wrapping_sub(386), member as i16, damage, services)?;
        }
    }
    e.native_return.low_word = Some(e.w(0x4471ea)? as u16);
    Ok(e)
}

fn fastplot(
    e: &mut StatusPanelPlan,
    tile: i16,
    destination: crate::recovered_geometry::BodyRect,
    services: &mut impl StatusPanelServices,
) -> Result<(), RuleError> {
    let rect = if tile >= 23000 {
        e.memory.rect(0x4471f2)?
    } else {
        destination
    };
    let mut context = blank_drawing(rect);
    if tile >= 0 {
        context.map_port = e.l(0x43879c)?;
    }
    if (0..23000).contains(&tile) {
        context.tile_port = e.l(0x4386c4)?;
    }
    if tile >= 23000 {
        context.backing_port = e.l(0x4386bc)?;
        let index = usize::from(tile.wrapping_sub(23000) as u16);
        let pointer = e.l(0x41c220 + index as u32 * 4)?;
        let bounds = if pointer == 0 {
            context.depth = e.b(0x43873f)? as i8;
            None
        } else {
            Some(e.memory.rect(pointer.wrapping_add(16))?)
        };
        context.buildings.resize(
            index + 1,
            BuildingWorld {
                pointer: 0,
                bounds: None,
            },
        );
        context.buildings[index] = BuildingWorld { pointer, bounds };
    }
    let child = recovered_drawing::fastplot(&mut context, tile, destination, 0, 0, services)?;
    e.drawing_effects(child.effects)
}
fn ground(
    e: &mut StatusPanelPlan,
    frame: u32,
    body: i16,
    services: &mut impl StatusPanelServices,
) -> Result<(), RuleError> {
    e.ww(frame.wrapping_add(10), 0)?;
    e.ww(frame.wrapping_add(8), body)?;
    let enabled = e.w(0x438776)?;
    if enabled == 0 {
        e.native_return = StatusPanelReturn::full(0);
        return Ok(());
    }
    let state = services.body_ground_state(&e.memory)?;
    if body <= 9 {
        let index = usize::try_from(body)
            .map_err(|_| RuleError::Invalid("negative bodyground warrior view index".into()))?;
        let position = *state
            .combat
            .squad
            .positions
            .get(index)
            .ok_or_else(|| RuleError::Invalid("missing bodyground warrior position".into()))?;
        let tile = *state
            .combat
            .squad
            .under_tiles
            .get(index)
            .ok_or_else(|| RuleError::Invalid("missing bodyground warrior tile".into()))?;
        if position
            != (
                e.w(0x44709a_u32.wrapping_add((i32::from(body) * 4) as u32))?,
                e.w(0x44709c_u32.wrapping_add((i32::from(body) * 4) as u32))?,
            )
            || tile != e.w(indexed(0x437a6e, body))?
        {
            return Err(RuleError::Invalid(
                "bodyground warrior view disagrees with addressed input".into(),
            ));
        }
    } else {
        let index = body.wrapping_sub(10);
        let selected = usize::from(index as u16);
        let position = *state
            .combat
            .enemies
            .positions
            .get(selected)
            .ok_or_else(|| RuleError::Invalid("missing bodyground NPC position".into()))?;
        let offset = (i32::from(index) * 4) as u32;
        if position
            != (
                e.w(0x446f0a_u32.wrapping_add(offset))?,
                e.w(0x446f0c_u32.wrapping_add(offset))?,
            )
        {
            return Err(RuleError::Invalid(
                "bodyground NPC position disagrees with addressed input".into(),
            ));
        }
        let record = state
            .combat
            .enemies
            .records
            .get(selected)
            .ok_or_else(|| RuleError::Invalid("missing bodyground NPC record".into()))?;
        let bytes = record
            .bytes
            .get(0x118..0x120)
            .ok_or_else(|| RuleError::Invalid("short bodyground NPC record".into()))?;
        if bytes
            != e.bytes(
                0x439d3c_u32
                    .wrapping_add(u32::from(index as u16) * 288)
                    .wrapping_add(0x118),
                8,
            )?
        {
            return Err(RuleError::Invalid(
                "bodyground NPC tiles disagree with addressed input".into(),
            ));
        }
    }
    let effects = recovered_bodies::bodyground(
        &state,
        BodyGroundInput {
            body,
            copy_word: 0,
            drawing_enabled: enabled,
            port: e.l(0x43879c)?,
        },
    )?;
    for effect in effects {
        e.effects.push(StatusPanelEffect::Body(effect.clone()));
        if let BodyEffect::FastPlot { tile, rect, .. } = effect {
            fastplot(e, tile, rect, services)?;
        }
    }
    e.native_return = StatusPanelReturn::full(0);
    Ok(())
}

/// combatinfo, 0x117a54. The three retained argument groups give distinct child frames.
pub fn combat_info(
    memory: &RenderingMemory,
    random: &RandomState,
    frame: u32,
    maximum_polls: usize,
    damage: DamageFrame,
    services: &mut impl StatusPanelServices,
) -> Result<StatusPanelPlan, RuleError> {
    let mut e = StatusPanelPlan::new(memory, random);
    let mut current = e.w(frame.wrapping_add(8))?;
    let cursor = e.toolbox(0xaa1b, vec![word(132)], services)?;
    e.ww(0x438782, 1)?;
    e.toolbox(0xaa1c, vec![long(e.l(0x43952c)?)], services)?;
    let mut remaining = maximum_polls;
    loop {
        step(&mut remaining)?;
        if e.toolbox(0xa974, vec![], services)? as u8 == 0 {
            break;
        }
        e.toolbox(0xa873, vec![long(e.l(0x43879c)?)], services)?;
        e.toolbox(0xa972, vec![ptr(0x4471ee)], services)?;
        let divide = |value: i16| value.wrapping_add(if value < 0 { 31 } else { 0 }) >> 5;
        let row = e
            .l(0x442780)?
            .wrapping_add(i32::from(divide(e.w(0x4471f0)?)) as u32);
        let column = e
            .l(0x442784)?
            .wrapping_add(i32::from(divide(e.w(0x4471ee)?)) as u32);
        let address = 0x4085b8_u32
            .wrapping_add(row.wrapping_mul(180))
            .wrapping_add(column.wrapping_mul(2));
        let target = e.w(address)?.wrapping_abs();
        if target <= 110 && target != current && target > -1 {
            ground(&mut e, frame.wrapping_sub(24), current, services)?;
            let force = e.w(0x438760)?;
            e.sprite(frame.wrapping_sub(30), current, force, services)?;
            e.sprite(frame.wrapping_sub(36), target, 1, services)?;
            let highlighted = if target < 9 {
                e.w(member_address(target as u16).wrapping_add(0x16))? != 0
            } else {
                e.b(0x439d3c_u32
                    .wrapping_add(u32::from(target.wrapping_sub(10) as u16) * 288)
                    .wrapping_add(0x104))?
                    != 0
            };
            e.toolbox(
                0xaa1c,
                vec![long(if highlighted { cursor } else { e.l(0x43952c)? })],
                services,
            )?;
            e.panel(frame.wrapping_sub(22), target, damage, services)?;
            current = target;
        }
    }
    ground(&mut e, frame.wrapping_sub(24), current, services)?;
    e.sprite(frame.wrapping_sub(30), current, 0, services)?;
    if e.w(0x438760)? != 0 {
        let actor = e.w(0x4435d2)?;
        e.panel(frame.wrapping_sub(22), actor, damage, services)?;
    }
    e.ww(0x438782, 0)?;
    e.flush(services)?;
    e.toolbox(0xaa26, vec![long(cursor)], services)?;
    Ok(e)
}

fn quiet(
    e: &mut StatusPanelPlan,
    selector: i16,
    services: &impl StatusPanelServices,
) -> Result<(), RuleError> {
    let last = e.w(0x439576)?;
    let mut context = AudioContext {
        last_channel: last,
        ..AudioContext::default()
    };
    if last >= 0 {
        let maximum = if selector > 0 {
            selector
        } else {
            last.wrapping_sub(1)
        };
        if maximum >= 0 {
            context.channels.resize(maximum as usize + 1, 0);
            if selector > 0 {
                context.channels[selector as usize] =
                    e.l(0x4436b6_u32.wrapping_add((i32::from(selector) * 4) as u32))?;
            } else {
                for index in 1..last {
                    context.channels[index as usize] = e.l(0x4436b6 + index as u32 * 4)?;
                }
            }
        }
    }
    let observations = services.quiet_observations(selector, &context, &e.memory)?;
    let child = recovered_audio::quiet(&mut context, selector, &observations)?;
    e.effects
        .extend(child.effects.into_iter().map(StatusPanelEffect::Audio));
    e.native_return = StatusPanelReturn::unknown();
    Ok(())
}
fn compact_heap(
    e: &mut StatusPanelPlan,
    services: &impl StatusPanelServices,
) -> Result<(), RuleError> {
    let mut context = RuntimeContext {
        memory: RuntimeMemory {
            bytes: e.memory.bytes.clone(),
        },
        random: e.random.clone(),
        step_limit: 16,
    };
    let observations = services.heap_observations(&context)?;
    let child = recovered_runtime::compact_heap(&mut context, &observations)?;
    e.memory.bytes = context.memory.bytes;
    e.random = context.random;
    e.effects
        .extend(child.effects.into_iter().map(StatusPanelEffect::Runtime));
    e.native_return = match child.result {
        recovered_runtime::RuntimeResult::Long(value) => StatusPanelReturn::full(value),
        _ => StatusPanelReturn::unknown(),
    };
    Ok(())
}

/// showcondition2, 0x1551a2. Mode BYTE is read after prefix services and children.
pub fn show_condition2(
    memory: &RenderingMemory,
    random: &RandomState,
    frame: u32,
    maximum_steps: usize,
    services: &impl StatusPanelServices,
) -> Result<StatusPanelPlan, RuleError> {
    let mut e = StatusPanelPlan::new(memory, random);
    let npc = e.b(frame.wrapping_add(9))?.wrapping_sub(10) as i8 as i16 as u16;
    let record = 0x439d3c_u32.wrapping_add(u32::from(npc) * 288);
    let mut remaining = maximum_steps;
    e.ww(0x4434aa, 0)?;
    while e.w(0x4434aa)? <= e.w(0x439576)? {
        step(&mut remaining)?;
        let selector = e.w(0x4434aa)?;
        quiet(&mut e, selector, services)?;
        e.add(0x4434aa, 1)?;
    }
    compact_heap(&mut e, services)?;
    e.sound(609, services)?;
    let menu = e.toolbox(0xa9bf, vec![word(131)], services)?;
    e.toolbox(0xa935, vec![long(menu), word(-1)], services)?;
    e.imported(
        0x400090,
        vec![ptr(record.wrapping_add(0xda)), ptr(0x438aa4)],
        services,
    )?;
    e.toolbox(0xa933, vec![long(menu), ptr(0x438aa4)], services)?;
    let mut count = 1i16;
    e.toolbox(
        0xa940,
        vec![
            long(menu),
            word(count),
            word(e.w(record.wrapping_add(0x50))?),
        ],
        services,
    )?;
    let mode = e.b(frame.wrapping_add(11))? as i8;
    if mode == 0 || mode == 1 {
        e.ww(0x4434aa, 0)?;
        while e.w(0x4434aa)? < if mode == 0 { 40 } else { 6 } {
            step(&mut remaining)?;
            let index = e.w(0x4434aa)?;
            let present = if mode == 0 {
                e.b(record
                    .wrapping_add(0xb2)
                    .wrapping_add(i32::from(index) as u32))?
                    != 0
            } else {
                e.w(indexed(record.wrapping_add(0x26), index))? != 0
            };
            if present {
                count = count.wrapping_add(1);
                if mode == 0 {
                    e.indexed(133, e.w(0x4434aa)?.wrapping_add(1), services)?;
                } else {
                    let id = e.w(indexed(record.wrapping_add(0x26), e.w(0x4434aa)?))?;
                    e.load_item(id)?;
                    e.selection(false, services)?;
                }
                e.toolbox(0xa933, vec![long(menu), ptr(0x479ed8)], services)?;
                if mode == 1 && e.w(indexed(record.wrapping_add(0x26), e.w(0x4434aa)?))? < 0 {
                    e.toolbox(0xa945, vec![long(menu), word(count), byte(1)], services)?;
                }
            }
            e.add(0x4434aa, 1)?;
        }
    }
    let selected = e.toolbox(
        0xa80b,
        vec![long(menu), word(400), word(40), word(0)],
        services,
    )?;
    e.ww(0x4434b2, selected as i16)?;
    e.toolbox(0xa936, vec![word(131)], services)?;
    e.sound(607, services)?;
    e.toolbox(0xa9a3, vec![long(menu)], services)?;
    e.native_return = StatusPanelReturn::full(0);
    Ok(e)
}

/// showspecial, 0x15741a. Width gates and post-label value rereads are retained.
pub fn show_special(
    memory: &RenderingMemory,
    random: &RandomState,
    frame: u32,
    maximum_steps: usize,
    services: &impl StatusPanelServices,
) -> Result<StatusPanelPlan, RuleError> {
    let mut e = StatusPanelPlan::new(memory, random);
    let dialog = e.toolbox(0xa97c, vec![word(160), long(0), ptr(u32::MAX)], services)?;
    e.toolbox(0xa873, vec![long(dialog)], services)?;
    e.toolbox(0xa887, vec![word(e.w(0x402f44)?)], services)?;
    e.lw(0x4394b8, dialog)?;
    e.toolbox(0xaa0b, vec![long(e.l(0x4358ce)?)], services)?;
    e.toolbox(0xa862, vec![long(69)], services)?;
    e.sound(647, services)?;
    let mut shown = 1i16;
    let mut remaining = maximum_steps;
    e.ww(0x4434aa, 17)?;
    while e.w(0x4434aa)? < 31 {
        step(&mut remaining)?;
        let index = e.w(0x4434aa)?.wrapping_sub(17);
        if e.w(indexed(0x447310, index))? > 0 {
            e.indexed(1201, e.w(0x4434aa)?.wrapping_sub(16), services)?;
            if e.toolbox(0xa88c, vec![ptr(0x479ed8)], services)? as u16 != 0 {
                e.dialog_string(shown.wrapping_add(16), 0x479ed8, services)?;
                let index = e.w(0x4434aa)?.wrapping_sub(17);
                let value = e
                    .w(indexed(0x447310, index))?
                    .wrapping_add(e.w(indexed(0x447d7e, index))?);
                let item = shown.wrapping_add(29);
                shown = shown.wrapping_add(1);
                e.dialog_number(item, value, services)?;
            }
        }
        e.add(0x4434aa, 1)?;
    }
    e.toolbox(
        0xa91d,
        vec![
            long(dialog),
            word(300),
            word(shown.wrapping_shl(5)),
            byte(0),
        ],
        services,
    )?;
    e.toolbox(
        0xa91b,
        vec![
            long(dialog),
            word(e.w(0x4395a4)?.wrapping_add(150)),
            word(e.w(0x4395a2)?.wrapping_add(100)),
            byte(0),
        ],
        services,
    )?;
    e.toolbox(0xa915, vec![long(dialog)], services)?;
    e.flush(services)?;
    e.toolbox(0xa991, vec![long(0), ptr(frame.wrapping_sub(2))], services)?;
    e.sound(141, services)?;
    e.toolbox(0xa983, vec![long(dialog)], services)?;
    e.native_return = StatusPanelReturn::full(0);
    Ok(e)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recovered_combat_panel::{CombatPanelObservation, CombatPanelRequest};
    use crate::recovered_combat_ui::{CombatUiObservation, CombatUiRequest, CombatUiServices};
    use crate::recovered_detail_ui::{DetailReply, DetailRequest, DetailServices};
    use crate::recovered_drawing::{DrawingServices, PictureObservation};
    use crate::recovered_equipment::test_support::{self, FRAME, LIMITS, put_long, put_word};
    use crate::recovered_equipment_ui::EquipmentUiServices;
    use crate::recovered_feedback::{
        FeedbackMemory, FeedbackObservation, FeedbackRequest, FeedbackServices,
    };
    use crate::recovered_files::FileObservation;
    use crate::recovered_geometry::BodyRect;
    use crate::recovered_install::InstallMemory;
    use crate::recovered_interface::{InterfaceServices, Point};
    use crate::recovered_inventory_rules::{InventoryServices, SaneRequest};
    use crate::recovered_item_ui::{ItemUiReply, ItemUiRequest, ItemUiServices};
    use crate::recovered_rendering::{RenderingObservation, RenderingRequest};
    use crate::recovered_runtime::{RuntimeReply, RuntimeRequest};
    use std::cell::{Cell, RefCell};

    #[derive(Default)]
    struct Services {
        equipment: test_support::Services,
        requests: RefCell<Vec<StatusPanelRequest>>,
        current: i16,
        maximum: i16,
        minimum: i16,
        popup: u32,
        button: Cell<usize>,
        widths: Cell<usize>,
        late_mode: bool,
        reread_special: bool,
    }
    fn unused<T>() -> Result<T, RuleError> {
        Err(RuleError::Invalid("unused synthetic service".into()))
    }
    impl FeedbackServices for Services {
        fn observe(
            &self,
            r: &FeedbackRequest,
            m: &FeedbackMemory,
        ) -> Result<FeedbackObservation, RuleError> {
            FeedbackServices::observe(&self.equipment, r, m)
        }
        fn sound_observations(
            &self,
            id: i16,
            c: &AudioContext,
            m: &FeedbackMemory,
        ) -> Result<Vec<AudioObservation>, RuleError> {
            FeedbackServices::sound_observations(&self.equipment, id, c, m)
        }
    }
    impl InventoryServices for Services {
        fn sane(
            &self,
            r: &SaneRequest,
            m: &RenderingMemory,
        ) -> Result<Vec<RecordWrite>, RuleError> {
            InventoryServices::sane(&self.equipment, r, m)
        }
    }
    impl NumericServices for Services {
        fn number_to_string(&mut self, v: i32) -> Result<PascalString, RuleError> {
            self.equipment.number_to_string(v)
        }
        fn string_to_number(&mut self, v: &PascalString) -> Result<i32, RuleError> {
            self.equipment.string_to_number(v)
        }
    }
    impl DrawingServices for Services {
        fn color_icon(&mut self, id: i16) -> Result<u32, RuleError> {
            self.equipment.color_icon(id)
        }
        fn indexed_string(&mut self, r: i16, i: i16) -> Result<PascalString, RuleError> {
            self.equipment.indexed_string(r, i)
        }
        fn current_world(&mut self) -> Result<(u32, u32), RuleError> {
            self.equipment.current_world()
        }
        fn picture(&mut self, id: i16) -> Result<PictureObservation, RuleError> {
            self.equipment.picture(id)
        }
        fn new_world(&mut self, d: i16, r: BodyRect) -> Result<(i16, BuildingWorld), RuleError> {
            self.equipment.new_world(d, r)
        }
    }
    impl ItemUiServices for Services {
        fn observe(
            &mut self,
            r: &ItemUiRequest,
            m: &RenderingMemory,
        ) -> Result<ItemUiReply, RuleError> {
            ItemUiServices::observe(&mut self.equipment, r, m)
        }
    }
    impl EquipmentUiServices for Services {
        fn pen_position(
            &mut self,
            d: u32,
            m: &RenderingMemory,
        ) -> Result<Vec<RecordWrite>, RuleError> {
            self.equipment.pen_position(d, m)
        }
    }
    impl EquipmentServices for Services {
        fn profile_observations(
            &mut self,
            r: i16,
            c: i16,
            m: &RenderingMemory,
        ) -> Result<Vec<FileObservation>, RuleError> {
            self.equipment.profile_observations(r, c, m)
        }
    }
    impl CombatUiServices for Services {
        fn observe(
            &self,
            _: &CombatUiRequest,
            _: &RenderingMemory,
        ) -> Result<CombatUiObservation, RuleError> {
            unused()
        }
    }
    impl DetailServices for Services {
        fn observe(
            &mut self,
            _: &DetailRequest,
            _: &RenderingMemory,
        ) -> Result<DetailReply, RuleError> {
            unused()
        }
    }
    impl InterfaceServices for Services {
        fn word(&mut self, _: u32) -> Result<i16, RuleError> {
            unused()
        }
        fn current_port(&mut self) -> Result<u32, RuleError> {
            unused()
        }
        fn pen_state(&mut self) -> Result<[u8; 18], RuleError> {
            unused()
        }
        fn local_to_global(&mut self, _: Point) -> Result<Point, RuleError> {
            unused()
        }
        fn point_in_rect(&mut self, _: Point, _: BodyRect) -> Result<u8, RuleError> {
            unused()
        }
        fn new_region(&mut self) -> Result<u32, RuleError> {
            unused()
        }
        fn main_device_rect(&mut self) -> Result<BodyRect, RuleError> {
            unused()
        }
    }
    impl CombatPanelServices for Services {
        fn observe_panel(
            &mut self,
            _: &CombatPanelRequest,
            _: &RenderingMemory,
        ) -> Result<CombatPanelObservation, RuleError> {
            unused()
        }
        fn visibility_memory(&mut self, _: &RenderingMemory) -> Result<InstallMemory, RuleError> {
            unused()
        }
    }
    impl RenderingServices for Services {
        fn observe(
            &self,
            _: &RenderingRequest,
            _: &RenderingMemory,
        ) -> Result<RenderingObservation, RuleError> {
            unused()
        }
    }
    impl StatusPanelServices for Services {
        fn observe_status(
            &self,
            request: &StatusPanelRequest,
            memory: &RenderingMemory,
        ) -> Result<StatusPanelObservation, RuleError> {
            self.requests.borrow_mut().push(request.clone());
            let mut memory = memory.clone();
            let mut value = 0;
            let mut d0 = 0x1234abcd;
            match request {
                StatusPanelRequest::Toolbox {
                    opcode, arguments, ..
                } => match opcode {
                    0xa960 => value = u32::from(self.current as u16),
                    0xa962 => value = u32::from(self.maximum as u16),
                    0xa961 => value = u32::from(self.minimum as u16),
                    0xa9bf | 0xa97c => value = 0x7777,
                    0xa80b => value = self.popup,
                    0xaa1b => value = 0xdead,
                    0xaa26 => d0 = 0xbeef1234,
                    0xa974 => {
                        value = u32::from(self.button.get() == 0);
                        self.button.set(self.button.get() + 1);
                    }
                    0xa972 => {
                        put_word(&mut memory, 0x4471ee, -32);
                        put_word(&mut memory, 0x4471f0, -31);
                    }
                    0xa940 if self.late_mode => memory.insert_region(FRAME + 11, &[1]),
                    0xa88c => {
                        value = u32::from(self.widths.get() != 0);
                        self.widths.set(self.widths.get() + 1);
                    }
                    0xa98d => {
                        put_word(&mut memory, 0x4434b0, 4);
                        put_long(&mut memory, 0x4434dc, 0x5555);
                        memory.insert_region(0x4434ba, &[0; 8]);
                    }
                    0xa98f
                        if self.reread_special && *arguments == [long(0x5555), ptr(0x479ed8)] =>
                    {
                        put_word(&mut memory, 0x447312, 9);
                    }
                    0xa991 => put_word(&mut memory, FRAME - 2, 41),
                    _ => (),
                },
                StatusPanelRequest::Imported { address, arguments } => {
                    match (*address, arguments.as_slice()) {
                        (0x400178, [FeedbackArg::Pointer(p), _, _]) => {
                            memory.insert_region(*p, &[1, b'N'])
                        }
                        (
                            0x400090,
                            [
                                FeedbackArg::Pointer(source),
                                FeedbackArg::Pointer(destination),
                            ],
                        ) => {
                            for offset in 0..256 {
                                let b = memory.byte(source + offset)?;
                                memory.insert_region(destination + offset, &[b]);
                                if b == 0 {
                                    break;
                                }
                            }
                        }
                        (0x400168, [FeedbackArg::Long(v), FeedbackArg::Pointer(p)]) => {
                            let text =
                                PascalString::from_payload((*v as i32).to_string().as_bytes())?;
                            memory.insert_region(*p, text.raw());
                        }
                        _ => (),
                    }
                }
                StatusPanelRequest::NumberToString { value, destination } => {
                    let text = PascalString::from_payload(value.to_string().as_bytes())?;
                    memory.insert_region(*destination, text.raw());
                }
            }
            Ok(StatusPanelObservation { value, d0, memory })
        }
        fn body_ground_state(&self, _: &RenderingMemory) -> Result<BodyState, RuleError> {
            unused()
        }
        fn quiet_observations(
            &self,
            _: i16,
            _: &AudioContext,
            _: &RenderingMemory,
        ) -> Result<Vec<AudioObservation>, RuleError> {
            Ok(vec![])
        }
        fn heap_observations(
            &self,
            c: &RuntimeContext,
        ) -> Result<Vec<RuntimeObservation>, RuleError> {
            let mut observations = vec![];
            for a in [0x4386d8, 0x4386e0, 0x4386e4, 0x4386e8, 0x4386dc] {
                observations.push(RuntimeObservation {
                    request: RuntimeRequest::UnlockPixels(c.memory.long(a)?),
                    reply: RuntimeReply::Ignored,
                });
            }
            observations.push(RuntimeObservation {
                request: RuntimeRequest::MaxMem,
                reply: RuntimeReply::MaxMem { d0: 111, a0: 222 },
            });
            for (i, a) in [0x4386b4, 0x4386bc, 0x4386c0, 0x4386c4, 0x4386b8]
                .into_iter()
                .enumerate()
            {
                let pixmap = 0x9000 + i as u32;
                observations.push(RuntimeObservation {
                    request: RuntimeRequest::GetGWorldPixMap(c.memory.long(a)?),
                    reply: RuntimeReply::Pointer(pixmap),
                });
                observations.push(RuntimeObservation {
                    request: RuntimeRequest::LockPixels(pixmap),
                    reply: RuntimeReply::LockPixels {
                        boolean: 1,
                        native_d0: 0xabcdffff,
                    },
                });
            }
            Ok(observations)
        }
    }
    fn fixture() -> RenderingMemory {
        let mut memory = test_support::fixture();
        put_long(&mut memory, 0x439570, 0);
        put_word(&mut memory, 0x439576, -1);
        put_long(&mut memory, 0x439404, 10);
        put_long(&mut memory, 0x439400, 10);
        put_long(&mut memory, 0x4393fc, 20);
        memory.insert_region(0x442756, &[1]);
        memory.insert_region(0x442750, &[0]);
        put_long(&mut memory, 0x4358da, 30);
        put_word(&mut memory, 0x402f3c, 4);
        put_word(&mut memory, 0x438776, 0);
        put_word(&mut memory, 0x438760, 0);
        memory
    }
    fn damage() -> DamageFrame {
        DamageFrame {
            unmatched_multiplier_single_bits: 0,
        }
    }

    #[test]
    fn shop_single_step_uses_normal_bounds_and_count_based_top() {
        let mut memory = fixture();
        put_word(&mut memory, FRAME + 8, 21);
        put_long(&mut memory, FRAME + 10, 0);
        put_word(&mut memory, 0x447236, 2);
        let mut services = Services {
            maximum: 4,
            ..Services::default()
        };
        let result =
            update_shop(&memory, &RandomState::new(7), FRAME, 99, 20, &mut services).unwrap();
        assert_eq!(result.memory.word(0x4470b2).unwrap(), 1);
        assert_eq!(result.memory.word(0x4471f2).unwrap(), 56);
        assert!(
            !services
                .requests
                .borrow()
                .iter()
                .any(|r| matches!(r, StatusPanelRequest::Toolbox { opcode: 0xa8ef, .. }))
        );
        assert_eq!(result.native_return.known_long(), Some(0x1234abcd));
        assert_eq!(result.random, RandomState::new(7));
    }
    #[test]
    fn shop_page_down_preserves_split_forced_id_and_charge_copy_rows() {
        let mut memory = fixture();
        put_word(&mut memory, FRAME + 8, 23);
        put_long(&mut memory, FRAME + 10, 0);
        put_word(&mut memory, 0x438756, 1);
        put_word(&mut memory, 0x43875c, 0);
        put_word(&mut memory, 0x447236, 2);
        memory.insert_region(0x44273e, &[0xff]);
        memory.insert_region(0x44273d, &[3]);
        put_word(&mut memory, 0x44734e, 222);
        put_word(&mut memory, 0x447350, 7);
        memory.insert_region(0x447357, &[0x77]);
        let mut services = Services {
            maximum: 1,
            ..Services::default()
        };
        let result =
            update_shop(&memory, &RandomState::new(7), FRAME, 99, 20, &mut services).unwrap();
        assert_eq!(result.memory.word(0x4434b4).unwrap(), 0);
        assert_eq!(result.memory.word(0x41fc48).unwrap(), 222);
        assert_eq!(result.memory.word(0x41fc4a).unwrap(), 7);
        assert_eq!(result.memory.byte(0x41fc51).unwrap(), 0x77);
        assert_eq!(result.memory.byte(0x41f708).unwrap(), 0);
        assert_eq!(result.memory.byte(0x41f709).unwrap(), 3);
        assert_eq!(result.memory.word(0x4434ac).unwrap(), 9);
        assert!(services.requests.borrow().iter().any(|r| matches!(r,StatusPanelRequest::Toolbox { opcode:0xa8ef,arguments,.. } if *arguments==[ptr(FRAME-8),word(0),word(-40),long(0)])));
    }
    #[test]
    fn weapon_menu_includes_count_slot_and_caches_actual_mapping() {
        let mut memory = fixture();
        put_word(&mut memory, FRAME + 8, 0);
        put_word(&mut memory, member_address(0) + 0x1a, 0);
        let node = node_address(0, 0);
        put_word(&mut memory, node, 201);
        memory.insert_region(node + 10, &[15]);
        test_support::item(&mut memory, 201, 15);
        let mut services = Services {
            popup: 1,
            ..Services::default()
        };
        let result = change_weapon(
            &memory,
            &RandomState::new(7),
            FRAME,
            LIMITS,
            damage(),
            &mut services,
        )
        .unwrap();
        assert_eq!(result.memory.word(FRAME - 0x164).unwrap(), 0);
        assert_eq!(result.memory.word(0x4471e8).unwrap(), 200);
        assert_eq!(result.memory.word(0x4471ea).unwrap(), 0);
        assert_eq!(result.native_return.known_long(), Some(0x12340000));
        let requests = services.requests.borrow();
        assert!(requests.iter().any(|r| *r
            == StatusPanelRequest::Imported {
                address: 0x400178,
                arguments: vec![ptr(0x479ed8), word(201), word(2)]
            }));
        assert!(requests.iter().any(|r| *r
            == StatusPanelRequest::Imported {
                address: 0x400168,
                arguments: vec![long(0), ptr(FRAME - 0x100)]
            }));
        assert_eq!(
            requests
                .iter()
                .filter(|r| matches!(r, StatusPanelRequest::Toolbox { opcode: 0xa933, .. }))
                .count(),
            2
        );
    }
    #[test]
    fn combat_info_rounds_negative_points_and_disposes_observed_cursor() {
        let mut memory = fixture();
        put_word(&mut memory, FRAME + 8, 2);
        put_word(&mut memory, 0x4085b6, 111);
        let mut services = Services::default();
        let result = combat_info(
            &memory,
            &RandomState::new(7),
            FRAME,
            4,
            damage(),
            &mut services,
        )
        .unwrap();
        assert_eq!(result.memory.word(0x438782).unwrap(), 0);
        assert_eq!(result.memory.word(FRAME - 22).unwrap(), 2);
        assert_eq!(result.native_return.known_long(), Some(0xbeef1234));
        assert!(services.requests.borrow().iter().any(|r| *r
            == StatusPanelRequest::Toolbox {
                opcode: 0xaa26,
                arguments: vec![long(0xdead)],
                d0_input: None
            }));
        assert!(services.requests.borrow().iter().any(|r| *r
            == StatusPanelRequest::Toolbox {
                opcode: 0xa032,
                arguments: vec![],
                d0_input: Some(0xffff)
            }));
        let original = memory.clone();
        memory.bytes.remove(&(FRAME + 8));
        assert!(
            combat_info(
                &memory,
                &RandomState::new(7),
                FRAME,
                4,
                damage(),
                &mut Services::default()
            )
            .is_err()
        );
        assert_eq!(original.word(0x438782).unwrap(), 0);
        let mut memory = fixture();
        let destination = BodyRect {
            top: 1,
            left: 2,
            bottom: 33,
            right: 34,
        };
        for a in 0x4471f2..0x4471fa {
            memory.bytes.remove(&a);
        }
        let mut plan = StatusPanelPlan::new(&memory, &RandomState::new(7));
        fastplot(&mut plan, -1, destination, &mut Services::default()).unwrap();
        assert!(plan.effects.is_empty());
        fastplot(&mut plan, 1000, destination, &mut Services::default()).unwrap();
        assert!(plan.effects.iter().any(|e| matches!(e, StatusPanelEffect::Drawing(DrawingEffect::CopyBits { destination: rect, mode: 0, .. }) if *rect == destination)));
        memory.insert_region(0x4471f2, &[0; 8]);
        put_long(&mut memory, 0x41c220, 0x9100);
        memory.insert_region(0x9110, &[0, 0, 0, 0, 0, 64, 0, 96]);
        put_long(&mut memory, 0x4386bc, 0x9876);
        let mut plan = StatusPanelPlan::new(&memory, &RandomState::new(7));
        fastplot(&mut plan, 23000, destination, &mut Services::default()).unwrap();
        assert!(plan.effects.iter().any(|e| matches!(
            e,
            StatusPanelEffect::Drawing(DrawingEffect::CopyBits {
                source_bitmap: 0x9102,
                destination_bitmap: 0x9878,
                mode: 36,
                ..
            })
        )));
    }
    #[test]
    fn condition_menu_reads_mode_late_and_uses_unincremented_selection_resource() {
        let mut memory = fixture();
        memory.insert_region(FRAME + 9, &[11]);
        memory.insert_region(FRAME + 11, &[0]);
        let npc = 0x439d3c + 288;
        put_word(&mut memory, npc + 0x26, -201);
        memory.insert_region(npc + 0xda, b"NPC\0");
        test_support::item(&mut memory, 201, 2);
        let services = Services {
            late_mode: true,
            popup: 0x12340002,
            ..Services::default()
        };
        let result = show_condition2(&memory, &RandomState::new(7), FRAME, 50, &services).unwrap();
        assert_eq!(result.memory.word(0x4434b2).unwrap(), 2);
        assert_eq!(result.native_return.known_long(), Some(0));
        assert_eq!(result.memory.long(0x4386dc).unwrap(), 0x9004);
        let requests = services.requests.borrow();
        assert!(requests.iter().any(|r| *r
            == StatusPanelRequest::Imported {
                address: 0x400178,
                arguments: vec![ptr(0x479ed8), word(200), word(2)]
            }));
        assert!(requests.iter().any(|r| *r
            == StatusPanelRequest::Toolbox {
                opcode: 0xa945,
                arguments: vec![long(0x7777), word(2), byte(1)],
                d0_input: None
            }));
    }
    #[test]
    fn specialty_dialog_skips_zero_width_and_rereads_value_after_label() {
        let mut memory = fixture();
        put_word(&mut memory, 0x447310, 4);
        put_word(&mut memory, 0x447312, 5);
        put_word(&mut memory, 0x447d80, 7);
        let services = Services {
            reread_special: true,
            ..Services::default()
        };
        let result = show_special(&memory, &RandomState::new(7), FRAME, 20, &services).unwrap();
        assert_eq!(result.memory.word(FRAME - 2).unwrap(), 41);
        assert_eq!(result.native_return.known_long(), Some(0));
        let requests = services.requests.borrow();
        assert!(requests.contains(&StatusPanelRequest::NumberToString {
            value: 16,
            destination: 0x479ed8
        }));
        assert!(requests.iter().any(|r| *r
            == StatusPanelRequest::Toolbox {
                opcode: 0xa91d,
                arguments: vec![long(0x7777), word(300), word(64), byte(0)],
                d0_input: None
            }));
        assert_eq!(
            requests
                .iter()
                .filter(|r| matches!(r, StatusPanelRequest::Toolbox { opcode: 0xa98d, .. }))
                .count(),
            2
        );
        assert_eq!(
            requests
                .iter()
                .filter(|r| matches!(r, StatusPanelRequest::Toolbox { opcode: 0xa98f, .. }))
                .count(),
            2
        );
    }
}
