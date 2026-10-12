//! Inventory editing and warrior-index maintenance from complete CODE 1 bodies.

use crate::recovered_character_panels::{self, CharacterPanelEffect, CharacterPanelServices};
use crate::recovered_drawing::{self as drawing, DrawingContext, DrawingEffect};
use crate::recovered_equipment::{
    self as equipment, EquipmentEffect, EquipmentLimits, EquipmentServices,
};
use crate::recovered_feedback::{self as feedback, FeedbackArg, FeedbackEffect};
use crate::recovered_install::{self, InstallMemory, InstallRequest};
use crate::recovered_inventory::{self as inventory, WeightState};
use crate::recovered_inventory_rules::{self, InventoryEffect};
use crate::recovered_item_ui::{self, ItemUiEffect};
use crate::recovered_load_control::{self, LoadEffect, LoadFlow, LoadServices};
use crate::recovered_random_item::{RandomItemEffect, RandomItemLimits};
use crate::recovered_records::{self as records, ItemTables, RecordWrite};
use crate::recovered_rendering::RenderingMemory;
use crate::recovered_runtime::{self, RuntimeContext, RuntimeEffect, RuntimeMemory};
use crate::recovered_shop_ui::{self, ShopContext, ShopEffect};
use crate::recovered_text::{self as text, PascalString, TextEffect};
use crate::{RandomState, RuleError, rules};
use centurions_content::{ITEM_BYTES, WARRIOR_BYTES, Warrior};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InventoryEditRequest {
    Toolbox {
        opcode: u16,
        arguments: Vec<FeedbackArg>,
        d0_input: Option<u32>,
    },
    /// Actual import address and argument push order, including pointer buffers.
    Imported {
        address: u32,
        arguments: Vec<FeedbackArg>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InventoryEditObservation {
    pub value: u32,
    pub d0: u32,
    pub memory: RenderingMemory,
}
pub trait InventoryEditServices: EquipmentServices + CharacterPanelServices + LoadServices {
    fn observe_edit(
        &mut self,
        request: &InventoryEditRequest,
        memory: &RenderingMemory,
    ) -> Result<InventoryEditObservation, RuleError>;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InventoryEditLimits {
    pub maximum_steps: u32,
    pub equipment: EquipmentLimits,
    pub maximum_random_draws: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub enum InventoryEditEffect {
    Write(RecordWrite),
    Service {
        request: InventoryEditRequest,
        observation: InventoryEditObservation,
    },
    Character(CharacterPanelEffect),
    ItemUi(ItemUiEffect),
    Equipment(EquipmentEffect),
    Inventory(InventoryEffect),
    Feedback(FeedbackEffect),
    Load(LoadEffect),
    Runtime(RuntimeEffect),
    Drawing(DrawingEffect),
    Shop(ShopEffect),
    Text(TextEffect),
    Install(RandomItemEffect),
    Pin {
        value: i16,
        minimum: i16,
        maximum: i16,
        result: i16,
    },
}
#[derive(Clone, Debug, PartialEq)]
pub struct InventoryEditPlan {
    pub memory: RenderingMemory,
    pub random: RandomState,
    pub effects: Vec<InventoryEditEffect>,
    pub native_return: Option<u32>,
    pub flow: LoadFlow,
    pub qualification: InventoryEditQualification,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InventoryEditQualification {
    SourcePortWithObservedBoundariesAndCapturedChildren,
}
struct Runner<'a, S: ?Sized> {
    plan: InventoryEditPlan,
    services: &'a mut S,
    limits: InventoryEditLimits,
    remaining: u32,
}
impl<S: InventoryEditServices + ?Sized> Runner<'_, S> {
    fn step(&mut self) -> Result<(), RuleError> {
        self.remaining = self
            .remaining
            .checked_sub(1)
            .ok_or_else(|| RuleError::Invalid("inventory edit iteration limit exhausted".into()))?;
        Ok(())
    }
    fn b(&self, a: u32) -> Result<u8, RuleError> {
        self.plan.memory.byte(a)
    }
    fn w(&self, a: u32) -> Result<i16, RuleError> {
        Ok(self.plan.memory.word(a)? as i16)
    }
    fn l(&self, a: u32) -> Result<u32, RuleError> {
        self.plan.memory.long(a)
    }
    fn bytes(&self, a: u32, n: usize) -> Result<Vec<u8>, RuleError> {
        equipment::read(&self.plan.memory, a, n)
    }
    fn write(&mut self, a: u32, b: &[u8]) -> Result<(), RuleError> {
        self.bytes(a, b.len())?;
        self.plan.memory.insert_region(a, b);
        self.plan
            .effects
            .push(InventoryEditEffect::Write(RecordWrite {
                address: a,
                bytes: b.to_vec(),
            }));
        Ok(())
    }
    fn ww(&mut self, a: u32, v: i16) -> Result<(), RuleError> {
        self.write(a, &v.to_be_bytes())
    }
    fn lw(&mut self, a: u32, v: u32) -> Result<(), RuleError> {
        self.write(a, &v.to_be_bytes())
    }
    fn add(&mut self, a: u32, v: i16) -> Result<(), RuleError> {
        self.ww(a, self.w(a)?.wrapping_add(v))
    }
    fn observe(&mut self, r: InventoryEditRequest) -> Result<u32, RuleError> {
        let o = self.services.observe_edit(&r, &self.plan.memory)?;
        let v = o.value;
        self.plan.memory = o.memory.clone();
        self.plan.native_return = Some(o.d0);
        self.plan.effects.push(InventoryEditEffect::Service {
            request: r,
            observation: o,
        });
        Ok(v)
    }
    fn trap(&mut self, opcode: u16, arguments: Vec<FeedbackArg>) -> Result<u32, RuleError> {
        self.observe(InventoryEditRequest::Toolbox {
            opcode,
            arguments,
            d0_input: None,
        })
    }
    fn import(&mut self, address: u32, arguments: Vec<FeedbackArg>) -> Result<u32, RuleError> {
        self.observe(InventoryEditRequest::Imported { address, arguments })
    }
    fn flush(&mut self) -> Result<(), RuleError> {
        self.observe(InventoryEditRequest::Toolbox {
            opcode: 0xa032,
            arguments: vec![],
            d0_input: Some(0xffff),
        })?;
        Ok(())
    }
    fn copy(&mut self, source: u32, destination: u32, n: usize) -> Result<(), RuleError> {
        for offset in (0..n / 4 * 4).step_by(4) {
            let b = self.bytes(source.wrapping_add(offset as u32), 4)?;
            self.write(destination.wrapping_add(offset as u32), &b)?;
        }
        let offset = n / 4 * 4;
        if offset < n {
            let b = self.bytes(source.wrapping_add(offset as u32), n - offset)?;
            self.write(destination.wrapping_add(offset as u32), &b)?;
        }
        Ok(())
    }
    fn sound(&mut self, id: i16) -> Result<(), RuleError> {
        let c = feedback::sound(&self.plan.memory, id, self.services)?;
        self.plan.memory = c.memory;
        self.plan.native_return = c.native_return;
        self.plan
            .effects
            .extend(c.effects.into_iter().map(InventoryEditEffect::Feedback));
        Ok(())
    }
    fn runtime(&mut self) -> Result<(), RuleError> {
        let obs = self
            .services
            .runtime_observations(0x13e13c, &self.plan.memory)?;
        let mut context = RuntimeContext {
            memory: RuntimeMemory {
                bytes: self.plan.memory.bytes.clone(),
            },
            random: self.plan.random.clone(),
            step_limit: self.remaining as usize,
        };
        let child = recovered_runtime::compact_heap(&mut context, &obs)?;
        self.plan.memory.bytes = context.memory.bytes;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(InventoryEditEffect::Runtime));
        Ok(())
    }
    fn load_item(&mut self, id: i16) -> Result<(), RuleError> {
        let backing = equipment::item_tables(&self.plan.memory, id)?;
        let tables = ItemTables {
            families: std::array::from_fn(|i| backing[i].as_slice()),
        };
        let current = self.bytes(0x45363c, ITEM_BYTES)?.try_into().expect("item");
        let child = records::loaditem(&tables, &current, id)?;
        for w in child.writes {
            self.write(w.address, &w.bytes)?;
        }
        Ok(())
    }
    fn weight(&mut self, member: u16) -> Result<(), RuleError> {
        let base = record(member);
        let bytes = self.bytes(base, WARRIOR_BYTES)?;
        if base < 0x4536d2 && base.wrapping_add(WARRIOR_BYTES as u32) > 0x45363c {
            return Err(RuleError::Unsupported {
                action: "inventory_edit.weight",
                dependency: "selected warrior overlaps the captured item-work view",
            });
        }
        let count = i16::from_be_bytes([bytes[0x1a], bytes[0x1b]]);
        if count > 100 {
            return Err(RuleError::Invalid(
                "captured weight count exceeds 100 records".into(),
            ));
        }
        let mut backing: [Vec<u8>; 5] = std::array::from_fn(|_| vec![]);
        for slot in 0..count.max(0) as usize {
            let o = 0x132 + slot * 22;
            let id = i16::from_be_bytes([bytes[o], bytes[o + 1]]);
            let absolute = id.wrapping_abs();
            if let Ok(family) = usize::try_from(absolute / 200)
                && family < 5
            {
                let source = records::ITEM_TABLE_ADDRESSES[family]
                    .wrapping_add((absolute % 200) as u32 * ITEM_BYTES as u32);
                let weight = base.wrapping_add(0x9d6);
                if source < weight.wrapping_add(2)
                    && source.wrapping_add(ITEM_BYTES as u32) > weight
                {
                    return Err(RuleError::Unsupported {
                        action: "inventory_edit.weight",
                        dependency: "the weight write aliases a captured item-definition record",
                    });
                }
            }
            let selected = equipment::item_tables(&self.plan.memory, id)?;
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
        let state = WeightState {
            warrior: Warrior {
                name: String::new(),
                original_sha256: String::new(),
                bytes,
            },
            item_work: self.bytes(0x45363c, ITEM_BYTES)?.try_into().expect("item"),
        };
        let child = inventory::calcw(&state, &tables, member)?;
        for effect in child.effects {
            match effect {
                inventory::InventoryEffect::Write(w) => self.write(w.address, &w.bytes)?,
                _ => return Err(RuleError::Invalid("unexpected weight child effect".into())),
            }
        }
        Ok(())
    }
    fn character(&mut self, entry: u32, frame: u32) -> Result<(), RuleError> {
        self.step()?;
        let child = match entry {
            0x14a29e => recovered_character_panels::update_port(
                &self.plan.memory,
                frame,
                self.remaining,
                self.services,
            )?,
            0x14a3e6 => recovered_character_panels::plot_portrait(
                &self.plan.memory,
                frame,
                self.remaining,
                self.services,
            )?,
            0x120872 => recovered_character_panels::display(
                &self.plan.memory,
                frame,
                self.remaining,
                self.services,
            )?,
            _ => {
                return Err(RuleError::Invalid(
                    "unknown inventory character child".into(),
                ));
            }
        };
        self.plan.memory = child.memory;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(InventoryEditEffect::Character),
        );
        self.plan.native_return = Some(child.native_return);
        Ok(())
    }
    fn portrait(
        &mut self,
        frame: u32,
        id: i16,
        rect: crate::recovered_geometry::BodyRect,
        label: i16,
        member: i16,
    ) -> Result<(), RuleError> {
        self.ww(frame.wrapping_add(22), 0)?;
        self.ww(frame.wrapping_add(20), member)?;
        self.ww(frame.wrapping_add(18), label)?;
        self.lw(frame.wrapping_add(14), pack(rect.bottom, rect.right))?;
        self.lw(frame.wrapping_add(10), pack(rect.top, rect.left))?;
        self.ww(frame.wrapping_add(8), id)?;
        self.character(0x14a3e6, frame)
    }
    fn drawing(&mut self, effects: Vec<DrawingEffect>) -> Result<(), RuleError> {
        for effect in effects {
            if let DrawingEffect::Write(w) = effect {
                self.write(w.address, &w.bytes)?;
            } else {
                self.plan.effects.push(InventoryEditEffect::Drawing(effect));
            }
        }
        Ok(())
    }
    fn pict(
        &mut self,
        id: i16,
        rect: crate::recovered_geometry::BodyRect,
    ) -> Result<(), RuleError> {
        let c = drawing::pict(id, rect, self.services)?;
        self.drawing(c)
    }
    fn icon(
        &mut self,
        id: i16,
        rect: crate::recovered_geometry::BodyRect,
    ) -> Result<(), RuleError> {
        let child = drawing::ploticon3(&mut icon_context(rect), id, rect, self.services)?;
        self.drawing(child.effects)
    }
    fn erase_name(&mut self, selector: i16) -> Result<(), RuleError> {
        let context = ShopContext {
            width: self.w(0x40856c)?,
            window: 0,
            erase_pattern: self.l(0x4358ce)?,
            normal_pattern: 0,
            credits: 0,
            text_work: PascalString::default(),
        };
        let child = recovered_shop_ui::eraseshopname(&context, selector);
        self.plan
            .effects
            .extend(child.effects.into_iter().map(InventoryEditEffect::Shop));
        Ok(())
    }
    fn handle_rect(
        &mut self,
        handle_address: u32,
        frame: u32,
    ) -> Result<crate::recovered_geometry::BodyRect, RuleError> {
        let p = self.l(self.l(handle_address)?)?;
        self.lw(frame.wrapping_sub(8), self.l(p.wrapping_add(8))?)?;
        self.lw(frame.wrapping_sub(4), self.l(p.wrapping_add(12))?)?;
        self.plan.memory.rect(frame.wrapping_sub(8))
    }
    fn update_item(&mut self, frame: u32, member: i16, slot: i16) -> Result<(), RuleError> {
        self.ww(frame.wrapping_add(10), slot)?;
        self.ww(frame.wrapping_add(8), member)?;
        let child = recovered_item_ui::update_edit_item(
            &self.plan.memory,
            frame,
            self.limits.equipment.filename_capacity,
            self.services,
        )?;
        self.plan.memory = child.memory;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(InventoryEditEffect::ItemUi));
        self.plan.native_return = child.native_return.map(|value| value as u32);
        Ok(())
    }
    fn dialog_item(&mut self, dialog: u32, item: i16, rect: u32) -> Result<(), RuleError> {
        self.trap(
            0xa98d,
            vec![l(dialog), w(item), p(0x4434b0), p(0x4434dc), p(rect)],
        )?;
        Ok(())
    }
    fn dialog_name(&mut self, member: i16) -> Result<(), RuleError> {
        let name = record(member as u16).wrapping_add(0xa21);
        self.import(0x400188, vec![p(name)])?;
        self.dialog_item(self.l(0x4394b8)?, 72, 0x4434ba)?;
        let length = usize::from(self.b(name)?) + 1;
        let text = PascalString::from_raw(&self.bytes(name, length)?)?;
        self.plan.effects.extend(
            text::dialog_string(72, &text)
                .into_iter()
                .map(InventoryEditEffect::Text),
        );
        self.trap(0xa98f, vec![l(self.l(0x4434dc)?), p(name)])?;
        self.import(0x400180, vec![p(name)])?;
        Ok(())
    }
    fn press(&mut self, frame: u32, index: i16, mode: i16, sound: i16) -> Result<(), RuleError> {
        self.lw(frame.wrapping_add(18), self.l(0x442930)?)?;
        self.lw(frame.wrapping_add(14), self.l(0x44292c)?)?;
        self.ww(frame.wrapping_add(12), sound)?;
        self.lw(frame.wrapping_add(8), pack(index, mode))?;
        let child =
            feedback::pressbutton(&self.plan.memory, frame, index, mode, sound, self.services)?;
        self.plan.memory = child.memory;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(InventoryEditEffect::Feedback));
        Ok(())
    }
    fn flash(&mut self, frame: u32, message: u32) -> Result<(), RuleError> {
        self.lw(frame.wrapping_add(16), 5000)?;
        self.lw(frame.wrapping_add(12), pack(75, 75))?;
        self.lw(frame.wrapping_add(8), message)?;
        let child = feedback::flashmessage(
            &self.plan.memory,
            frame,
            message,
            75,
            75,
            0,
            5000,
            self.limits.equipment.maximum_polls,
            self.services,
        )?;
        self.plan.memory = child.memory;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(InventoryEditEffect::Feedback));
        Ok(())
    }
    fn wear(&mut self, frame: u32, member: i16, slot: i16) -> Result<(), RuleError> {
        self.ww(frame.wrapping_add(12), 0)?;
        self.ww(frame.wrapping_add(10), slot)?;
        self.ww(frame.wrapping_add(8), member)?;
        let child = equipment::wear(
            &self.plan.memory,
            &self.plan.random,
            frame,
            self.limits.equipment,
            self.services,
        )?;
        self.plan.memory = child.memory;
        self.plan.random = child.random;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(InventoryEditEffect::Equipment),
        );
        Ok(())
    }
    fn remove(&mut self, frame: u32, member: i16, slot: i16) -> Result<(), RuleError> {
        self.lw(frame.wrapping_add(12), 1)?;
        self.ww(frame.wrapping_add(10), slot)?;
        self.ww(frame.wrapping_add(8), member)?;
        let child = recovered_inventory_rules::removeitem(
            &self.plan.memory,
            frame,
            self.limits.equipment.maximum_polls,
            self.services,
        )?;
        self.plan.memory = child.memory;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(InventoryEditEffect::Inventory),
        );
        Ok(())
    }
    fn scratch(&mut self, frame: u32) -> Result<(), RuleError> {
        self.ww(frame.wrapping_add(8), 10)?;
        let c = recovered_load_control::scratch2(&self.plan.memory, frame, self.services)?;
        self.plan.memory = c.memory;
        self.plan.flow = c.flow;
        self.plan.native_return = c.native_return;
        self.plan
            .effects
            .extend(c.effects.into_iter().map(InventoryEditEffect::Load));
        Ok(())
    }
}
fn w(v: i16) -> FeedbackArg {
    FeedbackArg::Word(v as u16)
}
fn l(v: u32) -> FeedbackArg {
    FeedbackArg::Long(v)
}
fn p(v: u32) -> FeedbackArg {
    FeedbackArg::Pointer(v)
}
fn pack(a: i16, b: i16) -> u32 {
    (u32::from(a as u16) << 16) | u32::from(b as u16)
}
fn record(member: u16) -> u32 {
    0x448aa0_u32.wrapping_add(u32::from(member) * 3138)
}
fn node(member: i16, slot: i16) -> u32 {
    record(member as u16)
        .wrapping_add(u32::from(slot as u16) * 22)
        .wrapping_add(0x132)
}
fn half(v: i16) -> i16 {
    v.wrapping_add(((v as u16) >> 15) as i16) >> 1
}
fn icon_context(rect: crate::recovered_geometry::BodyRect) -> DrawingContext {
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
fn runner<'a, S: InventoryEditServices + ?Sized>(
    memory: &RenderingMemory,
    random: &RandomState,
    limits: InventoryEditLimits,
    services: &'a mut S,
) -> Runner<'a, S> {
    Runner {
        plan: InventoryEditPlan {
            memory: memory.clone(),
            random: random.clone(),
            effects: vec![],
            native_return: None,
            flow: LoadFlow::Returned,
            qualification:
                InventoryEditQualification::SourcePortWithObservedBoundariesAndCapturedChildren,
        },
        services,
        limits,
        remaining: limits.maximum_steps,
    }
}

/// change, 0x11084a. Preserve deferred display argument D3 and fresh buyer/member bytes.
pub fn change(
    memory: &RenderingMemory,
    random: &RandomState,
    frame: u32,
    limits: InventoryEditLimits,
    services: &mut (impl InventoryEditServices + ?Sized),
) -> Result<InventoryEditPlan, RuleError> {
    let mut e = runner(memory, random, limits, services);
    let deferred = e.w(frame.wrapping_add(10))?;
    e.ww(0x4471fc, 17)?;
    e.ww(0x4471fa, 402)?;
    e.ww(0x4471fe, e.w(0x4471fa)?.wrapping_add(32))?;
    e.trap(0xa873, vec![l(e.l(0x438788)?)])?;
    let wings = frame.wrapping_sub(24);
    e.ww(wings.wrapping_add(10), e.b(0x44273e)? as i8 as i16)?;
    e.ww(wings.wrapping_add(8), e.b(0x44273d)? as i8 as i16)?;
    let child = recovered_item_ui::update_shop_wings(&e.plan.memory, wings, e.services)?;
    e.plan.memory = child.memory;
    e.plan
        .effects
        .extend(child.effects.into_iter().map(InventoryEditEffect::ItemUi));
    e.ww(frame.wrapping_sub(18), e.b(0x44273e)? as i8 as i16)?;
    e.ww(frame.wrapping_sub(20), e.b(0x44273d)? as i8 as i16)?;
    e.character(0x14a29e, frame.wrapping_sub(28))?;
    e.weight(e.b(0x44273d)? as i8 as i16 as u16)?;
    e.copy(
        record(e.b(0x44273d)? as i8 as i16 as u16),
        0x44721c,
        WARRIOR_BYTES,
    )?;
    if (e.b(0x44273e)? as i8) > -1 {
        e.ww(
            record(e.b(0x44273e)? as i8 as i16 as u16).wrapping_add(0x9d6),
            0,
        )?;
        e.weight(e.b(0x44273e)? as i8 as i16 as u16)?;
        e.copy(
            record(e.b(0x44273e)? as i8 as i16 as u16),
            0x447e5e,
            WARRIOR_BYTES,
        )?;
    }
    if deferred != 0 {
        e.write(0x4427b1, &[1])?;
    }
    if e.w(frame.wrapping_add(8))? != 0 {
        e.sound(141)?;
    }
    if e.l(0x439404)? == e.l(0x439420)? {
        e.ww(0x4471fc, half(e.w(0x40856c)?).wrapping_add(375))?;
        e.ww(0x43875c, 1)?;
        e.ww(0x447200, e.w(0x4471fc)?.wrapping_add(32))?;
        e.lw(0x439404, e.l(0x4393fc)?)?;
        if deferred == 0 {
            e.trap(0xa965, vec![l(e.l(0x4393fc)?), w(0)])?;
            e.trap(0xa963, vec![l(e.l(0x4393fc)?), w(0)])?;
            e.trap(0xa88a, vec![w(12)])?;
            e.trap(0xa889, vec![w(1)])?;
            e.erase_name(1)?;
            e.trap(0xa862, vec![l(69)])?;
            if e.b(0x44273e)? == 255 {
                e.trap(0xa965, vec![l(e.l(0x4393fc)?), w(191)])?;
                let rect = e.handle_rect(0x439424, frame)?;
                e.pict(0, rect)?;
                e.icon(2005, e.plan.memory.rect(0x4471fa)?)?;
                e.trap(0xa884, vec![p(0x403ef8)])?;
            } else {
                let name = record(e.b(0x44273e)? as i8 as i16 as u16).wrapping_add(0xa21);
                e.import(0x400188, vec![p(name)])?;
                let name = record(e.b(0x44273e)? as i8 as i16 as u16).wrapping_add(0xa21);
                e.trap(0xa884, vec![p(name)])?;
                let name = record(e.b(0x44273e)? as i8 as i16 as u16).wrapping_add(0xa21);
                e.import(0x400180, vec![p(name)])?;
                let data = e.l(e.l(0x479ed4)?)?;
                let rect = rect_longs(e.l(data.wrapping_add(8))?, e.l(data.wrapping_add(12))?);
                e.pict(202, rect)?;
            }
        }
        if (e.b(0x44273e)? as i8) > -1 {
            e.ww(0x43875c, 0)?;
            let rect = e.handle_rect(0x439424, frame)?;
            e.pict(0, rect)?;
            let member = e.b(0x44273e)? as i8 as i16;
            e.portrait(
                frame.wrapping_sub(46),
                e.w(0x447eb2)?,
                e.plan.memory.rect(0x4471fa)?,
                e.w(0x447e92)?,
                member,
            )?;
            if e.w(0x447eae)? < 1 {
                e.icon(2019, e.plan.memory.rect(frame.wrapping_sub(8))?)?;
            }
            if e.w(0x447eae)? < -9 {
                e.icon(2015, e.plan.memory.rect(0x4471fa)?)?;
            }
            if e.w(0x447e78)? > 9 {
                e.trap(
                    0xa965,
                    vec![l(e.l(0x4393fc)?), w(e.w(0x447e78)?.wrapping_sub(9))],
                )?;
            }
        }
        let child = frame.wrapping_sub(22);
        e.ww(child.wrapping_add(8), deferred)?;
        e.character(0x120872, child)?;
    } else {
        if e.w(0x439582)? != 0 {
            e.add(0x4471fc, half(e.w(0x40856c)?))?;
        }
        e.ww(0x447200, e.w(0x4471fc)?.wrapping_add(32))?;
        let rect = e.handle_rect(0x439428, frame)?;
        e.pict(0, rect)?;
        e.trap(0xa88a, vec![w(12)])?;
        e.trap(0xa889, vec![w(1)])?;
        e.erase_name(0)?;
        e.trap(0xa862, vec![l(69)])?;
        e.import(0x400188, vec![p(0x447c3d)])?;
        e.trap(0xa884, vec![p(0x447c3d)])?;
        e.import(0x400180, vec![p(0x447c3d)])?;
        let member = e.b(0x44273d)? as i8 as i16;
        e.portrait(
            frame.wrapping_sub(56),
            e.w(0x447270)?,
            e.plan.memory.rect(0x4471fa)?,
            e.w(0x447250)?,
            member,
        )?;
        if e.w(0x44726c)? < 1 {
            e.icon(2019, e.plan.memory.rect(frame.wrapping_sub(8))?)?;
        }
        if e.w(0x44726c)? < -9 {
            e.icon(2015, e.plan.memory.rect(0x4471fa)?)?;
        }
        if deferred == 0 {
            e.trap(0xa963, vec![l(e.l(0x439400)?), w(0)])?;
            e.trap(0xa965, vec![l(e.l(0x439400)?), w(0)])?;
        }
    }
    if e.w(0x43875a)? == 0 || e.l(0x439404)? == e.l(0x43941c)? {
        if e.w(0x447236)? > 9 {
            e.trap(
                0xa965,
                vec![l(e.l(0x439400)?), w(e.w(0x447236)?.wrapping_sub(9))],
            )?;
        }
        e.lw(0x439404, e.l(0x439400)?)?;
        e.ww(0x43875a, 1)?;
        e.ww(0x438756, 1)?;
        let child = frame.wrapping_sub(22);
        e.ww(child.wrapping_add(8), 0)?;
        e.character(0x120872, child)?;
        e.ww(0x438756, 0)?;
        e.lw(0x439404, 0)?;
    }
    e.plan.native_return = Some(0);
    Ok(e.plan)
}
fn rect_longs(a: u32, b: u32) -> crate::recovered_geometry::BodyRect {
    crate::recovered_geometry::BodyRect {
        top: (a >> 16) as u16 as i16,
        left: a as u16 as i16,
        bottom: (b >> 16) as u16 as i16,
        right: b as u16 as i16,
    }
}

/// edititems, 0x134034. Demo backup and WORD-selected modifier editing are retained.
pub fn edit_items(
    memory: &RenderingMemory,
    random: &RandomState,
    frame: u32,
    limits: InventoryEditLimits,
    services: &mut (impl InventoryEditServices + ?Sized),
) -> Result<InventoryEditPlan, RuleError> {
    let mut e = runner(memory, random, limits, services);
    let mut member = e.w(frame.wrapping_add(8))?;
    let mut slot = e.w(frame.wrapping_add(10))?;
    e.trap(0xa874, vec![p(frame.wrapping_sub(0x499c))])?;
    e.trap(0xaa19, vec![p(frame.wrapping_sub(0x4992))])?;
    e.trap(0xaa1a, vec![p(frame.wrapping_sub(0x4998))])?;
    let dialog = e.trap(0xa97c, vec![w(141), l(0), p(u32::MAX)])?;
    e.trap(0xa873, vec![l(dialog)])?;
    e.trap(0xa862, vec![l(69)])?;
    e.trap(0xaa0b, vec![l(e.l(0x4358ce)?)])?;
    e.trap(0xa915, vec![l(dialog)])?;
    e.lw(0x4394b8, dialog)?;
    for opcode in [0xa981, 0xa922, 0xa923] {
        e.trap(opcode, vec![l(dialog)])?;
    }
    e.lw(0x4394b8, dialog)?;
    e.trap(0xa873, vec![l(dialog)])?;
    e.trap(0xaa1c, vec![l(e.l(0x43952c)?)])?;
    e.runtime()?;
    e.write(0x44273f, &[member as u8])?;
    e.update_item(frame.wrapping_sub(0x49bc), member, slot)?;
    e.dialog_item(dialog, 71, 0x4434ba)?;
    e.portrait(
        frame.wrapping_sub(0x49cc),
        e.w(record(member as u16).wrapping_add(0x54))?,
        e.plan.memory.rect(0x4434ba)?,
        e.w(record(member as u16).wrapping_add(0x34))?,
        member,
    )?;
    e.trap(0xa88a, vec![w(12)])?;
    e.trap(0xa887, vec![w(e.w(0x402f3c)?)])?;
    e.trap(0xa862, vec![l(69)])?;
    e.dialog_name(member)?;
    if e.w(0x4046ea)? == 0 && e.w(0x4471ec)? == 1 {
        let mut i = 0i16;
        while i < i16::from(e.b(0x44273a)? as i8) {
            e.step()?;
            e.copy(
                record(i as u16),
                frame
                    .wrapping_sub(0x498c)
                    .wrapping_add(u32::from(i as u16) * 3138),
                WARRIOR_BYTES,
            )?;
            i = i.wrapping_add(1);
        }
        e.flash(frame.wrapping_sub(0x49c4), 0x402880)?;
    }
    let mut rewear = false;
    loop {
        e.step()?;
        e.flush()?;
        e.trap(0xa991, vec![l(0), p(0x4434b2)])?;
        e.dialog_item(dialog, e.w(0x4434b2)?, 0x44292c)?;
        if e.w(0x4434b2)? < 3 {
            break;
        }
        if matches!(e.w(0x4434b2)?, 69 | 70) {
            e.press(frame.wrapping_sub(0x49c6), 2, 2, 141)?;
            slot = slot.wrapping_sub(1);
            if e.w(0x4434b2)? == 70 {
                slot = slot.wrapping_add(2);
            }
            if slot < 0 {
                slot = 0;
            }
            if slot > e.w(record(member as u16).wrapping_add(0x1a))? {
                slot = e.w(record(member as u16).wrapping_add(0x1a))?;
            }
            e.update_item(frame.wrapping_sub(0x49bc), member, slot)?;
            e.press(frame.wrapping_sub(0x49ca), 2, 1, 0)?;
        }
        if matches!(e.w(0x4434b2)?, 67 | 68) {
            e.press(frame.wrapping_sub(0x49c6), 3, 2, 141)?;
            slot = 0;
            member = member.wrapping_sub(1);
            if e.w(0x4434b2)? == 68 {
                member = member.wrapping_add(2);
            }
            if member < 0 {
                member = i16::from(e.b(0x44273a)? as i8);
            }
            if member > i16::from(e.b(0x44273a)? as i8) {
                member = 0;
            }
            e.update_item(frame.wrapping_sub(0x49bc), member, slot)?;
            e.dialog_item(dialog, 71, 0x4434ba)?;
            e.portrait(
                frame.wrapping_sub(0x49cc),
                e.w(record(member as u16).wrapping_add(0x54))?,
                e.plan.memory.rect(0x4434ba)?,
                e.w(record(member as u16).wrapping_add(0x34))?,
                member,
            )?;
            e.dialog_name(member)?;
            e.press(frame.wrapping_sub(0x49e8), 3, 1, 0)?;
        }
        let chosen = e.w(0x4434b2)?;
        if !(40..=63).contains(&chosen) {
            continue;
        }
        e.sound(141)?;
        if e.b(node(member, slot).wrapping_add(9))? != 0 {
            rewear = true;
            e.copy(node(member, slot), 0x41fc48, 22)?;
            e.remove(frame.wrapping_sub(0x49c0), member, slot)?;
        }
        let chosen = e.w(0x4434b2)?;
        let address = node(member, slot);
        const OFFSETS: [u32; 10] = [13, 15, 16, 11, 17, 18, 19, 14, 21, 20];
        match chosen {
            40..=49 => {
                let a = address.wrapping_add(OFFSETS[(chosen - 40) as usize]);
                e.write(a, &[e.b(a)?.wrapping_sub(1)])?;
            }
            52..=61 => {
                let a = address.wrapping_add(OFFSETS[(chosen - 52) as usize]);
                e.write(a, &[e.b(a)?.wrapping_add(1)])?;
            }
            50 => e.ww(address.wrapping_add(2), 0)?,
            51 => e.add(address.wrapping_add(6), -1)?,
            62 => e.ww(address.wrapping_add(2), e.w(address.wrapping_add(6))?)?,
            63 => e.add(address.wrapping_add(6), 1)?,
            _ => (),
        }
        let mut sp = frame.wrapping_sub(0x49b0);
        for (offset, minimum, maximum) in [
            (13, 1, 200),
            (15, 0, 200),
            (16, 0, 100),
            (11, 0, 100),
            (17, 0, 100),
            (18, 0, 200),
            (19, 0, 50),
            (14, 0, 50),
            (21, -10, 10),
            (20, 0, 50),
        ] {
            sp = sp.wrapping_sub(6);
            let child = sp.wrapping_sub(8);
            e.lw(child.wrapping_add(10), pack(minimum, maximum))?;
            let value = e.b(address.wrapping_add(offset))? as i8 as i16;
            e.ww(child.wrapping_add(8), value)?;
            let result = rules::pin(value, minimum, maximum);
            e.plan.effects.push(InventoryEditEffect::Pin {
                value,
                minimum,
                maximum,
                result,
            });
            e.write(address.wrapping_add(offset), &[result as u8])?;
        }
        sp = sp.wrapping_sub(6);
        let child = sp.wrapping_sub(8);
        e.ww(child.wrapping_add(12), e.w(address.wrapping_add(6))?)?;
        e.ww(child.wrapping_add(10), 0)?;
        let value = e.w(address.wrapping_add(2))?;
        e.ww(child.wrapping_add(8), value)?;
        let maximum = e.w(child.wrapping_add(12))?;
        let result = rules::pin(value, 0, maximum);
        e.plan.effects.push(InventoryEditEffect::Pin {
            value,
            minimum: 0,
            maximum,
            result,
        });
        e.ww(address.wrapping_add(2), result)?;
        if rewear {
            e.copy(node(member, slot), 0x41fc48, 22)?;
            e.wear(frame.wrapping_sub(0x49be), member, slot)?;
            rewear = false;
        }
        e.update_item(frame.wrapping_sub(0x49bc), member, slot)?;
    }
    e.sound(141)?;
    e.trap(0xa983, vec![l(dialog)])?;
    e.write(0x44273f, &[member as u8])?;
    if e.w(0x4046ea)? == 0 && e.w(0x4471ec)? == 1 {
        let mut i = 0i16;
        while i < i16::from(e.b(0x44273a)? as i8) {
            e.step()?;
            e.copy(
                frame
                    .wrapping_sub(0x498c)
                    .wrapping_add(u32::from(i as u16) * 3138),
                record(i as u16),
                WARRIOR_BYTES,
            )?;
            i = i.wrapping_add(1);
        }
        e.flash(frame.wrapping_sub(0x49c4), 0x40285c)?;
    }
    e.trap(0xa873, vec![l(e.l(frame.wrapping_sub(0x499c))?)])?;
    e.trap(0xaa15, vec![p(frame.wrapping_sub(0x4998))])?;
    e.trap(0xaa14, vec![p(frame.wrapping_sub(0x4992))])?;
    e.plan.native_return = Some(0);
    Ok(e.plan)
}

/// plus, 0x14a918. Writes the first empty warrior-index slot using observed stdio.
pub fn plus(
    memory: &RenderingMemory,
    random: &RandomState,
    frame: u32,
    limits: InventoryEditLimits,
    services: &mut (impl InventoryEditServices + ?Sized),
) -> Result<InventoryEditPlan, RuleError> {
    let mut e = runner(memory, random, limits, services);
    let source = e.l(frame.wrapping_add(8))?;
    e.ww(frame.wrapping_sub(32), 0)?;
    let mut handle = e.import(0x400070, vec![p(0x401a76), p(0x401a7a)])?;
    if handle == 0 {
        handle = e.import(0x400070, vec![p(0x401a72), p(0x401a7a)])?;
        if handle == 0 {
            e.scratch(frame.wrapping_sub(54))?;
            if e.plan.flow != LoadFlow::Returned {
                return Ok(e.plan);
            }
        }
    }
    let mut index = 0i16;
    while index < 500 {
        e.step()?;
        let local = frame.wrapping_sub(30);
        e.import(0x400090, vec![p(0x401a71), p(local)])?;
        e.import(0x400048, vec![l(handle), l(1), l(30), p(local)])?;
        e.import(
            0x400048,
            vec![l(handle), l(1), l(2), p(frame.wrapping_sub(32))],
        )?;
        if e.import(0x400088, vec![p(local)])? as u16 == 0 {
            e.import(
                0x400078,
                vec![w(0), l(i32::from(index.wrapping_shl(5)) as u32), l(handle)],
            )?;
            e.import(0x400050, vec![l(handle), l(1), l(30), p(source)])?;
            e.import(
                0x400050,
                vec![l(handle), l(1), l(2), p(frame.wrapping_add(12))],
            )?;
            e.import(0x400068, vec![l(handle)])?;
            return Ok(e.plan);
        }
        index = index.wrapping_add(1);
    }
    e.import(0x400068, vec![l(handle)])?;
    Ok(e.plan)
}
/// minus, 0x14aa22. Name matching clears only the name and retains its WORD payload.
pub fn minus(
    memory: &RenderingMemory,
    random: &RandomState,
    frame: u32,
    limits: InventoryEditLimits,
    services: &mut (impl InventoryEditServices + ?Sized),
) -> Result<InventoryEditPlan, RuleError> {
    let mut e = runner(memory, random, limits, services);
    let source = e.l(frame.wrapping_add(8))?;
    e.ww(frame.wrapping_sub(32), 0)?;
    e.import(0x400090, vec![p(0x401a66), p(0x4389a4)])?;
    e.import(0x400098, vec![p(source), p(0x4389a4)])?;
    if e.b(frame.wrapping_add(13))? == 0 {
        e.import(0x400058, vec![p(0x4389a4)])?;
    }
    let mut handle = e.import(0x400070, vec![p(0x401a76), p(0x401a7a)])?;
    if handle == 0 {
        handle = e.import(0x400070, vec![p(0x401a72), p(0x401a7a)])?;
        if handle == 0 {
            e.scratch(frame.wrapping_sub(54))?;
            if e.plan.flow != LoadFlow::Returned {
                return Ok(e.plan);
            }
        }
    }
    let mut index = 0i16;
    while index < 500 {
        e.step()?;
        let local = frame.wrapping_sub(30);
        e.import(0x400090, vec![p(0x401a71), p(local)])?;
        e.import(
            0x400078,
            vec![w(0), l(i32::from(index.wrapping_shl(5)) as u32), l(handle)],
        )?;
        if e.import(0x400048, vec![l(handle), l(1), l(30), p(local)])? == 0 {
            e.import(0x400068, vec![l(handle)])?;
            return Ok(e.plan);
        }
        e.import(
            0x400048,
            vec![l(handle), l(1), l(2), p(frame.wrapping_sub(32))],
        )?;
        if e.import(0x4000a8, vec![p(source), p(local)])? as u16 == 0 {
            e.import(0x400090, vec![p(0x401a71), p(local)])?;
            e.import(
                0x400078,
                vec![w(0), l(i32::from(index.wrapping_shl(5)) as u32), l(handle)],
            )?;
            e.import(0x400050, vec![l(handle), l(1), l(30), p(local)])?;
            e.import(
                0x400050,
                vec![l(handle), l(1), l(2), p(frame.wrapping_sub(32))],
            )?;
            e.import(0x400068, vec![l(handle)])?;
            return Ok(e.plan);
        }
        index = index.wrapping_add(1);
    }
    e.import(0x400068, vec![l(handle)])?;
    Ok(e.plan)
}

/// addinitialitems, 0x13f52c. Execute the complete install/randomitem child atomically.
pub fn add_initial_items(
    memory: &RenderingMemory,
    random: &RandomState,
    frame: u32,
    limits: InventoryEditLimits,
    services: &mut (impl InventoryEditServices + ?Sized),
) -> Result<InventoryEditPlan, RuleError> {
    let mut e = runner(memory, random, limits, services);
    let mut index = 0i16;
    let mut destination = 0u16;
    while index < 20 {
        e.step()?;
        let address = 0x41fb7a_u32.wrapping_add((i32::from(index) * 2) as u32);
        let id = e.w(address)?;
        if id != 0 {
            e.load_item(id)?;
            let weight = e
                .w(0x453650)?
                .wrapping_mul(e.w(0x453660)?)
                .wrapping_add(e.w(0x447bf2)?)
                .wrapping_add(e.w(0x45364c)?);
            if (weight as u16) <= e.w(0x447bf4)? as u16 {
                let child_frame = frame.wrapping_sub(22);
                e.ww(child_frame.wrapping_add(12), e.w(0x447250)?)?;
                e.ww(child_frame.wrapping_add(10), e.w(0x44724e)?)?;
                e.ww(child_frame.wrapping_add(8), e.w(0x4536a2)?)?;
                let race = e.w(child_frame.wrapping_add(10))?;
                let caste = e.w(child_frame.wrapping_add(12))?;
                let observations = e
                    .services
                    .profile_observations(race, caste, &e.plan.memory)?;
                let child = recovered_inventory_rules::canuse(
                    &e.plan.memory,
                    child_frame,
                    e.limits.equipment.filename_capacity,
                    &observations,
                )?;
                let eligible = child.native_return as u16 != 0;
                e.plan.memory = child.memory;
                e.plan.effects.extend(
                    child
                        .effects
                        .into_iter()
                        .map(InventoryEditEffect::Inventory),
                );
                if eligible {
                    e.add(0x447236, 1)?;
                    let id = e.w(address)?;
                    let request = InstallRequest {
                        source_member: 0xffff,
                        destination_member: 0xffff,
                        source_slot: 0,
                        destination_slot: destination,
                        item_id: id,
                    };
                    destination = destination.wrapping_add(1);
                    const START: u32 = 0x4046e6;
                    const END: u32 = 0x478288;
                    let current = InstallMemory {
                        base_address: START,
                        bytes: e.bytes(START, (END - START) as usize)?,
                    };
                    let backing = equipment::item_tables(&e.plan.memory, id)?;
                    let tables = ItemTables {
                        families: std::array::from_fn(|i| backing[i].as_slice()),
                    };
                    let child = recovered_install::installitem_complete(
                        &current,
                        &e.plan.random,
                        &tables,
                        request,
                        RandomItemLimits {
                            maximum_draws: e.limits.maximum_random_draws,
                        },
                    )?;
                    for effect in child.effects {
                        if let RandomItemEffect::Write(ref w) = effect {
                            e.write(w.address, &w.bytes)?;
                        } else {
                            e.plan.effects.push(InventoryEditEffect::Install(effect));
                        }
                    }
                    e.plan.random = child.random;
                }
            }
        }
        index = index.wrapping_add(1);
    }
    e.copy(0x44721c, 0x448aa0, WARRIOR_BYTES)?;
    e.weight(0)?;
    let mut slot = 0i16;
    while slot < e.w(0x448aba)? {
        e.step()?;
        e.wear(frame.wrapping_sub(22), 0, slot)?;
        slot = slot.wrapping_add(1);
    }
    e.copy(0x448aa0, 0x44721c, WARRIOR_BYTES)?;
    e.ww(0x447bf6, e.w(0x41fb78)?)?;
    e.plan.native_return = Some(0);
    Ok(e.plan)
}
