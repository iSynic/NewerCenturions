//! Character and item panels composed from CODE 1 bodies and recovered children.

use crate::recovered_drawing::{self as drawing, DrawingContext, DrawingEffect};
use crate::recovered_equipment::{item_tables, read};
use crate::recovered_equipment_ui::{self, EquipmentUiEffect, EquipmentUiServices};
use crate::recovered_geometry::BodyRect;
use crate::recovered_interface::{self as interface, InterfaceEffect, InterfaceServices};
use crate::recovered_records::{self as records, ItemTables, RecordWrite};
use crate::recovered_rendering::RenderingMemory;
use crate::recovered_runtime::{self as runtime, RuntimeEffect, RuntimeObservation};
use crate::recovered_shop_ui::{self as shop, ShopContext, ShopEffect};
use crate::recovered_text::{self as text, PascalString, TextContext, TextEffect};
use crate::{RandomState, RuleError};
use centurions_content::ITEM_BYTES;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PanelArgument {
    Byte(u8),
    Word(u16),
    Long(u32),
    Pointer(u32),
}
use PanelArgument::{Long as L, Pointer as P, Word as W};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CharacterPanelRequest {
    Toolbox {
        selector: u16,
        arguments: Vec<PanelArgument>,
    },
    /// Native runtime entry and its actual arguments, in stack push order.
    RuntimeString {
        entry: u32,
        arguments: Vec<PanelArgument>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CharacterPanelObservation {
    pub value: u32,
    pub memory: RenderingMemory,
}

/// Observations are pure. Captured child services must not execute their effects.
pub trait CharacterPanelServices: EquipmentUiServices + InterfaceServices {
    fn panel_observe(
        &mut self,
        request: &CharacterPanelRequest,
        memory: &RenderingMemory,
    ) -> Result<CharacterPanelObservation, RuleError>;
    /// Exact observations consumed by the existing runtime child, without I/O.
    fn runtime_observations(
        &mut self,
        entry: u32,
        memory: &RenderingMemory,
    ) -> Result<Vec<RuntimeObservation>, RuleError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CharacterPanelEffect {
    Write(RecordWrite),
    Service {
        request: CharacterPanelRequest,
        value: u32,
        /// Final byte differences; these do not claim internal Toolbox write order.
        memory_changes: Vec<(u32, Option<u8>)>,
    },
    Drawing(DrawingEffect),
    Interface(InterfaceEffect),
    Equipment(EquipmentUiEffect),
    Shop(ShopEffect),
    Runtime(RuntimeEffect),
    Text(TextEffect),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CharacterPanelPlan {
    pub memory: RenderingMemory,
    pub effects: Vec<CharacterPanelEffect>,
    pub native_return: u32,
}

struct Runner<'a, S> {
    plan: CharacterPanelPlan,
    services: &'a mut S,
    remaining: u32,
}
impl<S: CharacterPanelServices> Runner<'_, S> {
    fn step(&mut self) -> Result<(), RuleError> {
        self.remaining = self.remaining.checked_sub(1).ok_or_else(|| {
            RuleError::Invalid("character panel observation budget exhausted".into())
        })?;
        Ok(())
    }
    fn b(&self, address: u32) -> Result<u8, RuleError> {
        self.plan.memory.byte(address)
    }
    fn w(&self, address: u32) -> Result<u16, RuleError> {
        self.plan.memory.word(address)
    }
    fn sw(&self, address: u32) -> Result<i16, RuleError> {
        Ok(self.w(address)? as i16)
    }
    fn sb(&self, address: u32) -> Result<i16, RuleError> {
        Ok(i16::from(self.b(address)? as i8))
    }
    fn l(&self, address: u32) -> Result<u32, RuleError> {
        self.plan.memory.long(address)
    }
    fn write(&mut self, address: u32, bytes: &[u8]) -> Result<(), RuleError> {
        read(&self.plan.memory, address, bytes.len())?;
        self.plan.memory.insert_region(address, bytes);
        self.plan
            .effects
            .push(CharacterPanelEffect::Write(RecordWrite {
                address,
                bytes: bytes.to_vec(),
            }));
        Ok(())
    }
    fn byte(&mut self, address: u32, value: u8) -> Result<(), RuleError> {
        self.write(address, &[value])
    }
    fn word(&mut self, address: u32, value: u16) -> Result<(), RuleError> {
        self.write(address, &value.to_be_bytes())
    }
    fn long(&mut self, address: u32, value: u32) -> Result<(), RuleError> {
        self.write(address, &value.to_be_bytes())
    }
    fn add(&mut self, address: u32, value: i16) -> Result<(), RuleError> {
        self.word(address, self.w(address)?.wrapping_add(value as u16))
    }
    fn observe(&mut self, request: CharacterPanelRequest) -> Result<u32, RuleError> {
        let observed = self.services.panel_observe(&request, &self.plan.memory)?;
        let mut changes = Vec::new();
        let mut before = self.plan.memory.bytes.iter().peekable();
        let mut after = observed.memory.bytes.iter().peekable();
        while before.peek().is_some() || after.peek().is_some() {
            match (before.peek(), after.peek()) {
                (Some((a, x)), Some((b, y))) if a == b => {
                    if x != y {
                        changes.push((**b, Some(**y)));
                    }
                    before.next();
                    after.next();
                }
                (Some((a, _)), Some((b, _))) if a < b => {
                    changes.push((**a, None));
                    before.next();
                }
                (Some(_), Some((b, y))) => {
                    changes.push((**b, Some(**y)));
                    after.next();
                }
                (Some((a, _)), None) => {
                    changes.push((**a, None));
                    before.next();
                }
                (None, Some((b, y))) => {
                    changes.push((**b, Some(**y)));
                    after.next();
                }
                (None, None) => break,
            }
        }
        self.plan.memory = observed.memory;
        self.plan.effects.push(CharacterPanelEffect::Service {
            request,
            value: observed.value,
            memory_changes: changes,
        });
        Ok(observed.value)
    }
    fn trap(&mut self, selector: u16, arguments: Vec<PanelArgument>) -> Result<u32, RuleError> {
        self.observe(CharacterPanelRequest::Toolbox {
            selector,
            arguments,
        })
    }
    fn string_runtime(
        &mut self,
        entry: u32,
        arguments: Vec<PanelArgument>,
    ) -> Result<(), RuleError> {
        self.observe(CharacterPanelRequest::RuntimeString { entry, arguments })?;
        Ok(())
    }
    fn port(&mut self, port: u32) -> Result<(), RuleError> {
        self.trap(0xa873, vec![L(port)])?;
        Ok(())
    }
    fn rect(&self, address: u32) -> Result<BodyRect, RuleError> {
        self.plan.memory.rect(address)
    }
    fn draw_effects(&mut self, effects: Vec<DrawingEffect>) -> Result<(), RuleError> {
        for effect in effects {
            match effect {
                DrawingEffect::Write(w) => self.write(w.address, &w.bytes)?,
                e => self.plan.effects.push(CharacterPanelEffect::Drawing(e)),
            }
        }
        Ok(())
    }
    fn pict(&mut self, id: i16, address: u32) -> Result<(), RuleError> {
        let rect = self.rect(address)?;
        let effects = drawing::pict(id, rect, self.services)?;
        self.draw_effects(effects)
    }
    fn icon(&mut self, id: i16, address: u32) -> Result<(), RuleError> {
        let rect = self.rect(address)?;
        let mut context = icon_context(rect);
        let child = drawing::ploticon3(&mut context, id, rect, self.services)?;
        self.draw_effects(child.effects)
    }
    fn numeric(&mut self, value: i16) -> Result<(), RuleError> {
        let mut context = TextContext::default();
        let effects = text::string(&mut context, value, self.services)?;
        for effect in effects {
            if matches!(effect, TextEffect::NumberToString(_)) {
                self.plan.effects.push(CharacterPanelEffect::Text(effect));
                self.write(0x479ed8, context.scratch.raw())?;
            } else {
                self.plan.effects.push(CharacterPanelEffect::Text(effect));
            }
        }
        Ok(())
    }
    fn button(&mut self, member: i16) -> Result<(), RuleError> {
        let handle = self.l(index(0x43949c, member, 4))?;
        let data = self.l(handle)?;
        self.long(0x44292c, self.l(data.wrapping_add(8))?)?;
        self.long(0x442930, self.l(data.wrapping_add(12))?)?;
        let input = interface::ButtonInput {
            rect: self.rect(0x44292c)?,
            target_port: self.l(0x43878c)?,
            dark_pattern: self.l(0x4358de)?,
            light_pattern: self.l(0x4358e2)?,
        };
        let child = interface::downbutton(input, 1, self.services)?;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(CharacterPanelEffect::Interface),
        );
        Ok(())
    }
    fn light(&mut self, member: i16) -> Result<(), RuleError> {
        let input = interface::LightInput {
            member: member as i8,
            override_picture: 0,
            port: self.l(0x43878c)?,
            width_input: self.sw(0x40856c)?,
            threshold: self.sw(0x447194)?,
            ordinary_picture: self.l(0x439550)?,
            override_handle: 0,
            low_picture: self.l(0x439558)?,
        };
        let child = interface::updatelight(input, self.services)?;
        self.plan
            .effects
            .extend(child.into_iter().map(CharacterPanelEffect::Interface));
        Ok(())
    }
    fn runtime_child(&mut self, entry: u32) -> Result<(), RuleError> {
        let observations = self
            .services
            .runtime_observations(entry, &self.plan.memory)?;
        let mut context =
            runtime::RuntimeContext::new(RandomState::new(0), self.remaining as usize);
        context.memory.bytes = self.plan.memory.bytes.clone();
        let child = match entry {
            0x13e13c => runtime::compact_heap(&mut context, &observations)?,
            0x13a498 => runtime::copy_in(&mut context, &observations)?,
            _ => {
                return Err(RuleError::Invalid(
                    "unknown character panel runtime child".into(),
                ));
            }
        };
        self.plan.memory.bytes = context.memory.bytes;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(CharacterPanelEffect::Runtime));
        Ok(())
    }
    fn prep(&mut self, frame: u32) -> Result<(), RuleError> {
        self.step()?;
        let expanded = self.b(0x4427b5)? == 0;
        let shift = if expanded {
            self.w(0x40856c)?.wrapping_mul(self.w(0x439582)?)
        } else {
            0
        };
        self.trap(
            0xa8a7,
            vec![
                P(0x4434ba),
                W(shift.wrapping_add(320)),
                W(0),
                W(shift.wrapping_add(640)),
                W(300),
            ],
        )?;
        self.pict(218, 0x4434ba)?;
        let shift = if expanded {
            self.w(0x40856c)?.wrapping_mul(self.w(0x439582)?)
        } else {
            0
        };
        self.trap(
            0xa8a7,
            vec![
                P(0x4434ba),
                W(shift.wrapping_add(380)),
                W(0),
                W(shift.wrapping_add(640)),
                W(50),
            ],
        )?;
        self.word(0x4434aa, 0)?;
        while self.sw(0x4434aa)? <= self.sb(0x44273a)? {
            self.step()?;
            self.pict(189, 0x4434ba)?;
            let child = frame.wrapping_sub(24);
            self.long(child.wrapping_add(10), 0)?;
            self.word(child.wrapping_add(8), self.w(0x4434aa)?)?;
            self.picture_box(child)?;
            self.add(0x4434ba, 50)?;
            self.add(0x4434be, 50)?;
            let member = self.sw(0x4434aa)?;
            if self.b(index(0x442742, member, 1))? != 0 {
                self.button(member)?;
            }
            self.add(0x4434aa, 1)?;
        }
        Ok(())
    }
    fn short(&mut self, frame: u32) -> Result<(), RuleError> {
        self.step()?;
        let mode = self.w(frame.wrapping_add(8))?;
        self.byte(0x4427b3, 0)?;
        self.prep(frame.wrapping_sub(12))?;
        self.word(0x4434aa, 0)?;
        while self.sw(0x4434aa)? <= self.sb(0x44273a)? {
            self.step()?;
            let child = frame.wrapping_sub(16);
            self.word(child.wrapping_add(10), mode)?;
            self.word(child.wrapping_add(8), self.w(0x4434aa)?)?;
            self.character(child, false)?;
            self.add(0x4434aa, 1)?;
        }
        Ok(())
    }
    fn selected(&mut self, frame: u32) -> Result<(), RuleError> {
        self.step()?;
        self.word(0x4434aa, 0)?;
        while self.sw(0x4434aa)? <= self.sb(0x44273a)? {
            self.step()?;
            let address = index(0x442798, self.sw(0x4434aa)?, 2);
            if self.w(address)? != 0 {
                let mode = u16::from(self.sw(address)? <= 0);
                let child = frame.wrapping_sub(12);
                self.word(child.wrapping_add(10), mode)?;
                self.word(child.wrapping_add(8), self.w(0x4434aa)?)?;
                self.character(child, true)?;
                self.word(index(0x442798, self.sw(0x4434aa)?, 2), 0)?;
            }
            self.add(0x4434aa, 1)?;
        }
        Ok(())
    }
    fn out(&mut self, frame: u32) -> Result<(), RuleError> {
        self.step()?;
        if self.l(0x43879c)? == 0 {
            return Ok(());
        }
        if self.b(0x4386a8)? == 0 {
            self.byte(0x4386a8, 1)?;
            self.byte(0x4427b3, 1)?;
            let child = frame.wrapping_sub(18);
            self.word(child.wrapping_add(8), 3)?;
            self.short(child)?;
            self.runtime_child(0x13a498)?;
        }
        self.long(frame.wrapping_sub(8), self.l(0x4471b4)?)?;
        self.long(frame.wrapping_sub(4), self.l(0x4471b8)?)?;
        self.add(frame.wrapping_sub(2), 320)?;
        self.add(frame.wrapping_sub(6), 320)?;
        self.port(self.l(0x43878c)?)?;
        self.trap(0xa862, vec![L(33)])?;
        self.trap(0xa863, vec![L(30)])?;
        self.trap(
            0xa8ec,
            vec![
                P(self.l(0x4386bc)?.wrapping_add(2)),
                P(self.l(0x43879c)?.wrapping_add(2)),
                P(0x4471b4),
                P(frame.wrapping_sub(8)),
                W(0),
                L(0),
            ],
        )?;
        Ok(())
    }
    fn update_port(&mut self, frame: u32) -> Result<(), RuleError> {
        self.step()?;
        self.port(self.l(0x438788)?)?;
        self.trap(0xaa0b, vec![L(self.l(0x4358ce)?)])?;
        self.word(frame.wrapping_sub(8), 388)?;
        self.word(
            frame.wrapping_sub(6),
            half(self.sw(0x40856c)?).wrapping_add(262) as u16,
        )?;
        self.word(
            frame.wrapping_sub(2),
            self.w(frame.wrapping_sub(6))?.wrapping_add(22),
        )?;
        self.word(
            frame.wrapping_sub(4),
            self.w(frame.wrapping_sub(8))?.wrapping_add(22),
        )?;
        let rect = frame.wrapping_sub(8);
        let mut member = 0_i16;
        while member <= self.sb(0x44273a)? {
            self.step()?;
            if member == 3 {
                self.trap(0xa8a8, vec![P(rect), L(0xffb90018)])?;
            }
            let icon = self.trap(0xaa1e, vec![W(self.w(record(member).wrapping_add(84))?)])?;
            if icon != 0 {
                self.trap(0xa8a3, vec![P(rect)])?;
                if member != self.sb(0x44273e)? {
                    self.trap(0xaa1f, vec![P(rect), L(icon)])?;
                }
            }
            if member == self.sb(0x44273d)? {
                self.icon(145, rect)?;
            }
            self.trap(0xa8a8, vec![P(rect), L(52)])?;
            if icon != 0 {
                self.trap(0xa8a3, vec![P(rect)])?;
                if member != self.sb(0x44273d)? {
                    self.trap(0xaa1f, vec![P(rect), L(icon)])?;
                }
            }
            if member == self.sb(0x44273e)? {
                self.icon(145, rect)?;
            }
            if icon != 0 {
                self.trap(0xaa25, vec![L(icon)])?;
            }
            self.trap(0xa8a8, vec![P(rect), L(0x18ffcc)])?;
            member = member.wrapping_add(1);
        }
        self.trap(0xaa0b, vec![L(self.l(0x4358da)?)])?;
        Ok(())
    }
    fn picture_box(&mut self, frame: u32) -> Result<(), RuleError> {
        self.step()?;
        let member = self.sw(frame.wrapping_add(8))?;
        if member > self.sb(0x44273a)? {
            return Ok(());
        }
        self.word(0x4471f2, member.wrapping_mul(50) as u16)?;
        self.word(0x4471f4, 320)?;
        self.word(0x4471f8, 380)?;
        self.word(0x4471f6, self.w(0x4471f2)?.wrapping_add(50))?;
        if self.b(0x4427b5)? == 0 {
            self.word(
                0x4471f4,
                self.w(0x4471f4)?
                    .wrapping_add(self.w(0x40856c)?.wrapping_mul(self.w(0x439582)?)),
            )?;
            self.word(
                0x4471f8,
                self.w(0x4471f8)?
                    .wrapping_add(self.w(0x40856c)?.wrapping_mul(self.w(0x439582)?)),
            )?;
        }
        let background = self.w(frame.wrapping_add(10))? == 0;
        if background {
            self.pict(1, 0x4471f2)?;
        }
        self.add(0x4471f4, 10)?;
        self.trap(0xa8a9, vec![P(0x4471f2), L(0x90009)])?;
        let child = frame.wrapping_sub(28);
        self.word(child.wrapping_add(22), 0)?;
        self.word(
            child.wrapping_add(20),
            if self.w(frame.wrapping_add(12))? == 0 {
                member as u16
            } else {
                u16::MAX
            },
        )?;
        self.word(
            child.wrapping_add(18),
            self.w(record(member).wrapping_add(52))?,
        )?;
        self.long(child.wrapping_add(14), self.l(0x4471f6)?)?;
        self.long(child.wrapping_add(10), self.l(0x4471f2)?)?;
        self.word(
            child.wrapping_add(8),
            self.w(record(member).wrapping_add(84))?,
        )?;
        self.portrait(child)?;
        if self.sw(record(member).wrapping_add(80))? < 1 {
            self.trap(0xa8a9, vec![P(0x4471f2), L(0xfffcfffc)])?;
            self.icon(2019, 0x4471f2)?;
            self.trap(0xa8a9, vec![P(0x4471f2), L(0x40004)])?;
            if self.sw(record(member).wrapping_add(80))? < -9 {
                self.icon(2015, 0x4471f2)?;
            } else if self.b(record(member).wrapping_add(0x9de))? == 0 && self.w(0x438776)? != 0 {
                self.icon(177, 0x4471f2)?;
            }
        }
        Ok(())
    }
    fn portrait(&mut self, frame: u32) -> Result<(), RuleError> {
        self.step()?;
        let member = self.sw(frame.wrapping_add(20))?;
        self.trap(0xa874, vec![P(frame.wrapping_sub(294))])?;
        self.trap(0xaa19, vec![P(frame.wrapping_sub(278))])?;
        self.trap(0xaa1a, vec![P(frame.wrapping_sub(284))])?;
        if member >= 0 && member <= self.sb(0x44273a)? && self.b(index(0x442742, member, 1))? != 0 {
            self.button(member)?;
        }
        self.word(
            frame.wrapping_add(16),
            self.w(frame.wrapping_add(12))?.wrapping_add(32),
        )?;
        self.word(
            frame.wrapping_add(14),
            self.w(frame.wrapping_add(10))?.wrapping_add(32),
        )?;
        self.long(0x439598, 0)?;
        let icon = self.trap(0xaa1e, vec![W(self.w(frame.wrapping_add(8))?)])?;
        self.long(0x439598, icon)?;
        // The source deliberately dereferences this handle before checking NIL.
        let data = self.l(self.l(0x439598)?)?;
        if self.w(data.wrapping_add(74))? == 40 {
            self.trap(0xa8a9, vec![P(frame.wrapping_add(10)), L(0xfffcfffc)])?;
        }
        if self.l(0x439598)? != 0 {
            self.trap(
                0xaa1f,
                vec![P(frame.wrapping_add(10)), L(self.l(0x439598)?)],
            )?;
            self.trap(0xaa25, vec![L(self.l(0x439598)?)])?;
        }
        if member >= 0 && self.w(frame.wrapping_add(22))? == 0 {
            self.long(frame.wrapping_sub(264), self.l(frame.wrapping_add(10))?)?;
            self.long(frame.wrapping_sub(260), self.l(frame.wrapping_add(14))?)?;
            if self.l(0x3fe302)? == self.l(0x43879c)? || self.l(0x3fe302)? == self.l(0x43878c)? {
                self.word(frame.wrapping_add(12), self.w(0x40856c)?.wrapping_add(574))?;
                self.add(frame.wrapping_add(10), 2)?;
            }
            let mut suppressed = false;
            if self.l(0x3fe302)? == self.l(0x438798)? {
                self.word(frame.wrapping_add(12), self.w(0x40856c)?.wrapping_add(628))?;
                self.add(frame.wrapping_add(10), 49)?;
            } else if self.l(0x3fe302)? == self.l(0x4394d0)? {
                self.word(frame.wrapping_add(12), 573)?;
                self.add(frame.wrapping_add(10), 3)?;
            } else if self.l(0x3fe302)? == self.l(0x438788)? {
                self.word(
                    frame.wrapping_add(12),
                    self.w(frame.wrapping_sub(262))?.wrapping_add(240),
                )?;
                self.add(frame.wrapping_add(10), 40)?;
            } else if self.l(0x3fe302)? == self.l(0x4394bc)? {
                self.word(frame.wrapping_add(12), 578)?;
                self.add(frame.wrapping_add(10), 25)?;
            } else if self.l(0x3fe302)? != self.l(0x43878c)? {
                suppressed = true;
            }
            self.word(
                frame.wrapping_add(14),
                self.w(frame.wrapping_add(10))?.wrapping_add(16),
            )?;
            for offset in [270, 268, 266] {
                self.word(frame.wrapping_sub(offset), 0)?;
            }
            let base = record(member);
            let poisoned = self.w(base.wrapping_add(100))? != 0
                || self.w(base.wrapping_add(150))? != 0
                || self.w(base.wrapping_add(98))? != 0;
            let disabled = self.sw(base.wrapping_add(144))? < 0
                || self.b(index(0x442742, member, 1))? != 0
                || self.w(base.wrapping_add(16))? != 0;
            if self.w(base.wrapping_add(94))? != 0
                || self.w(base.wrapping_add(98))? != 0
                || self.w(base.wrapping_add(106))? != 0
            {
                self.word(frame.wrapping_sub(266), 1)?;
            }
            let hindered = self.w(base.wrapping_add(96))? != 0
                || self.w(base.wrapping_add(146))? != 0
                || self.w(base.wrapping_add(148))? != 0;
            if self.w(base.wrapping_add(104))? != 0
                || self.w(base.wrapping_add(152))? != 0
                || self.w(base.wrapping_add(172))? != 0
            {
                self.word(frame.wrapping_sub(268), 1)?;
            }
            if self.w(base.wrapping_add(166))? != 0 || self.w(base.wrapping_add(168))? != 0 {
                self.word(frame.wrapping_sub(270), 1)?;
            }
            if !suppressed {
                for (position, id) in [180, 183, 181, 182, 185, 184].into_iter().enumerate() {
                    let condition = match position {
                        0 => self.w(frame.wrapping_sub(266))? != 0,
                        1 => hindered,
                        2 => poisoned,
                        3 => disabled,
                        4 => self.w(frame.wrapping_sub(270))? != 0,
                        _ => self.w(frame.wrapping_sub(268))? != 0,
                    };
                    if condition {
                        self.add(frame.wrapping_add(12), -16)?;
                        self.word(
                            frame.wrapping_add(16),
                            self.w(frame.wrapping_add(12))?.wrapping_add(16),
                        )?;
                        self.icon(id, frame.wrapping_add(10))?;
                    }
                }
            }
            self.long(frame.wrapping_add(10), self.l(frame.wrapping_sub(264))?)?;
            self.long(frame.wrapping_add(14), self.l(frame.wrapping_sub(260))?)?;
        }
        if self.w(0x439584)? != 0 {
            self.word(
                frame.wrapping_sub(272),
                self.w(self.l(0x3fe302)?.wrapping_add(68))?,
            )?;
            self.byte(
                frame.wrapping_sub(285),
                self.b(self.l(0x3fe302)?.wrapping_add(70))?,
            )?;
            self.word(
                frame.wrapping_sub(290),
                self.w(self.l(0x3fe302)?.wrapping_add(74))?,
            )?;
            self.word(
                frame.wrapping_sub(288),
                self.w(self.l(0x3fe302)?.wrapping_add(72))?,
            )?;
            self.trap(
                0xa893,
                vec![
                    W(self.w(frame.wrapping_add(12))?.wrapping_add(5)),
                    W(self.w(frame.wrapping_add(14))?.wrapping_sub(3)),
                ],
            )?;
            for selector in [0xa887, 0xa888, 0xa889] {
                self.trap(selector, vec![W(1)])?;
            }
            self.trap(0xa88a, vec![W(10)])?;
            self.trap(0xa862, vec![L(30)])?;
            self.string_runtime(
                0x400178,
                vec![P(0x479ed8), W(131), W(self.w(frame.wrapping_add(18))?)],
            )?;
            self.string_runtime(0x400180, vec![P(0x479ed8)])?;
            self.string_runtime(0x400090, vec![P(0x479ed8), P(frame.wrapping_sub(256))])?;
            self.byte(frame.wrapping_sub(249), 0)?;
            self.string_runtime(0x400188, vec![P(frame.wrapping_sub(256))])?;
            let width = self.trap(0xa88c, vec![P(frame.wrapping_sub(256))])?;
            self.word(0x4434b6, width as u16)?;
            let width = self.trap(0xa88c, vec![P(frame.wrapping_sub(256))])? as u16 as i16;
            if width > 38 {
                self.byte(
                    frame.wrapping_sub(256),
                    self.b(frame.wrapping_sub(256))?.wrapping_sub(1),
                )?;
            }
            self.trap(0xa884, vec![P(frame.wrapping_sub(256))])?;
            self.port(self.l(frame.wrapping_sub(294))?)?;
            self.trap(0xaa15, vec![P(frame.wrapping_sub(284))])?;
            self.trap(0xaa14, vec![P(frame.wrapping_sub(278))])?;
            self.trap(0xa887, vec![W(self.w(frame.wrapping_sub(272))?)])?;
            self.trap(0xa888, vec![W(u16::from(self.b(frame.wrapping_sub(285))?))])?;
            self.trap(0xa88a, vec![W(self.w(frame.wrapping_sub(290))?)])?;
            self.trap(0xa889, vec![W(self.w(frame.wrapping_sub(288))?)])?;
        }
        Ok(())
    }
    fn character(&mut self, frame: u32, short: bool) -> Result<(), RuleError> {
        self.step()?;
        let member = self.sw(frame.wrapping_add(8))?;
        if short && member > self.sb(0x44273a)? {
            return Ok(());
        }
        let mode = if short {
            0
        } else {
            self.w(frame.wrapping_add(10))?
        };
        let factor = if short {
            0
        } else {
            self.w(0x439582)?.wrapping_mul(u16::from(self.b(0x4427b5)?)) as u8
        };
        self.trap(0xa874, vec![P(frame.wrapping_sub(12))])?;
        if short || mode == 0 || mode == 3 {
            self.port(self.l(0x43878c)?)?;
        }
        if short {
            self.trap(0xa887, vec![W(self.w(0x402f3c)?)])?;
            self.trap(0xa889, vec![W(1)])?;
        } else {
            self.trap(0xa889, vec![W(1)])?;
            self.trap(0xa887, vec![W(self.w(0x402f3c)?)])?;
        }
        if short {
            self.word(frame.wrapping_sub(6), self.w(0x40856c)?.wrapping_add(380))?;
            self.word(frame.wrapping_sub(2), self.w(0x40856c)?.wrapping_add(640))?;
        } else {
            self.word(frame.wrapping_sub(6), 380)?;
            self.word(frame.wrapping_sub(2), 640)?;
        }
        self.word(frame.wrapping_sub(8), member.wrapping_mul(50) as u16)?;
        self.word(
            frame.wrapping_sub(4),
            self.w(frame.wrapping_sub(8))?.wrapping_add(50),
        )?;
        if !short && self.b(0x4427b5)? == 0 {
            self.word(
                frame.wrapping_sub(6),
                self.w(frame.wrapping_sub(6))?
                    .wrapping_add(self.w(0x40856c)?),
            )?;
            self.word(
                frame.wrapping_sub(2),
                self.w(frame.wrapping_sub(2))?
                    .wrapping_add(self.w(0x40856c)?),
            )?;
        }
        if !short && mode == 2 {
            self.add(frame.wrapping_sub(6), -60)?;
            self.trap(0xa8a3, vec![P(frame.wrapping_sub(8))])?;
            return Ok(());
        }
        if short || matches!(mode, 3..=5) {
            self.pict(if mode == 4 { 184 } else { 189 }, frame.wrapping_sub(8))?;
        }
        if !short && mode == 4 {
            return Ok(());
        }
        let row = if short {
            self.word(frame.wrapping_sub(6), 339)?;
            self.word(frame.wrapping_sub(2), self.w(0x4471f4)?.wrapping_add(32))?;
            self.add(frame.wrapping_sub(8), 9)?;
            self.add(frame.wrapping_sub(4), -9)?;
            let child = frame.wrapping_sub(54);
            self.word(child.wrapping_add(22), 0)?;
            self.word(child.wrapping_add(20), member as u16)?;
            self.word(
                child.wrapping_add(18),
                self.w(record(member).wrapping_add(52))?,
            )?;
            self.long(child.wrapping_add(14), self.l(frame.wrapping_sub(4))?)?;
            self.long(child.wrapping_add(10), self.l(frame.wrapping_sub(8))?)?;
            self.word(
                child.wrapping_add(8),
                self.w(record(member).wrapping_add(84))?,
            )?;
            self.portrait(child)?;
            if self.w(frame.wrapping_add(10))? != 0 {
                let child = frame.wrapping_sub(34);
                self.long(child.wrapping_add(10), 0)?;
                self.word(child.wrapping_add(8), member as u16)?;
                self.picture_box(child)?;
            }
            if self.w(0x438776)? != 0 {
                self.light(member)?;
            }
            member.wrapping_mul(50).wrapping_add(44) as u16
        } else {
            let row = self.w(frame.wrapping_sub(8))?.wrapping_add(44);
            if mode != 0 {
                let child = frame.wrapping_sub(42);
                self.long(child.wrapping_add(10), 0x10000)?;
                self.word(child.wrapping_add(8), member as u16)?;
                self.picture_box(child)?;
            }
            row
        };
        self.trap(0xa862, vec![L(69)])?;
        self.trap(0xa88a, vec![W(12)])?;
        self.position(420, row.wrapping_sub(22), factor)?;
        let name = record(member).wrapping_add(0xa21);
        self.string_runtime(0x400188, vec![P(name)])?;
        self.trap(0xa884, vec![P(name)])?;
        self.string_runtime(0x400180, vec![P(name)])?;
        self.trap(0xa888, vec![W(0)])?;
        self.trap(0xa862, vec![L(69)])?;
        self.position(610, row.wrapping_sub(22), factor)?;
        self.numeric(self.sw(record(member).wrapping_add(18))?)?;
        self.position(466, row, factor)?;
        let base = record(member);
        if self.w(base.wrapping_add(90))? == 0 {
            self.trap(0xa884, vec![P(if short { 0x400a70 } else { 0x400a76 })])?;
        } else {
            if self.sw(base.wrapping_add(88))? < self.sw(base.wrapping_add(90))? {
                self.trap(0xaa14, vec![P(0x44337a)])?;
            }
            if self.sw(base.wrapping_add(88))? > 999 {
                self.trap(0xa88a, vec![W(10)])?;
            }
            self.numeric(self.sw(base.wrapping_add(88))?)?;
        }
        self.trap(0xa88a, vec![W(12)])?;
        self.string_runtime(
            0x400168,
            vec![
                L(self.sw(base.wrapping_add(80))? as i32 as u32),
                P(0x479ed8),
            ],
        )?;
        self.position(427, row, factor)?;
        self.trap(0xa862, vec![L(69)])?;
        if self.sw(base.wrapping_add(80))? < self.sw(base.wrapping_add(82))? {
            self.trap(0xaa14, vec![P(0x44337a)])?;
        }
        if self.sw(base.wrapping_add(80))? > 999 {
            self.trap(0xa88a, vec![W(10)])?;
        }
        self.trap(0xa884, vec![P(0x479ed8)])?;
        self.trap(0xa862, vec![L(69)])?;
        for (position, offset) in [(524, 36), (563, 38), (609, 44)] {
            self.position(position, row, factor)?;
            self.numeric(self.sw(base.wrapping_add(offset))?)?;
        }
        if !short && self.w(0x438776)? != 0 {
            self.light(member)?;
        }
        self.port(self.l(frame.wrapping_sub(12))?)?;
        Ok(())
    }
    fn position(&mut self, offset: u16, row: u16, factor: u8) -> Result<(), RuleError> {
        let horizontal = self
            .w(0x40856c)?
            .wrapping_add(offset)
            .wrapping_sub(u16::from(factor).wrapping_mul(self.w(0x40856c)?));
        self.trap(0xa893, vec![W(horizontal), W(row)])?;
        Ok(())
    }
    fn display(&mut self, frame: u32) -> Result<(), RuleError> {
        self.step()?;
        self.runtime_child(0x13e13c)?;
        self.port(self.l(0x438788)?)?;
        let mut context = ShopContext {
            width: self.sw(0x40856c)?,
            window: self.l(0x438788)?,
            erase_pattern: self.l(0x4358ce)?,
            normal_pattern: self.l(0x4358da)?,
            credits: self.l(0x442788)? as i32,
            text_work: PascalString::default(),
        };
        let child = shop::shop_credit_display(&mut context, self.services)?;
        for effect in child.effects {
            match effect {
                ShopEffect::Drawing(DrawingEffect::Write(w)) => self.write(w.address, &w.bytes)?,
                e => self.plan.effects.push(CharacterPanelEffect::Shop(e)),
            }
        }
        if self.l(0x439404)? != self.l(0x439400)? {
            self.long(0x439404, self.l(0x4393fc)?)?;
        }
        let control = self.trap(0xa960, vec![L(self.l(0x439404)?)])?;
        self.word(0x4470b2, control as u16)?;
        self.word(0x4471f4, half(self.sw(0x40856c)?).wrapping_add(390) as u16)?;
        let mut first = 0_i16;
        let mut last = 9_i16;
        if self.b(0x4427b1)? != 0 && self.w(frame.wrapping_add(8))? == 0 {
            first = self.sw(0x4471e2)?.wrapping_sub(1);
            if self.b(0x4427b1)? == 1 {
                last = self.sw(0x4471e2)?;
            }
        }
        if self.b(0x435660)? != 0 {
            first = 0;
            last = 9;
        }
        self.byte(0x4427b1, 0)?;
        let mut row = first;
        while row < last {
            self.step()?;
            let mut quantity = 0;
            if self.l(0x439404)? == self.l(0x4393fc)? && self.w(0x43875c)? != 0 {
                self.word(0x4434b4, 0)?;
                let slot = self
                    .w(0x4471a8)?
                    .wrapping_add(row as u16)
                    .wrapping_add(self.w(0x4470b2)?) as i16;
                if self.b(index(0x478288, slot, 1).wrapping_add(2000))? != 0 {
                    let slot = self
                        .w(0x4471a8)?
                        .wrapping_add(row as u16)
                        .wrapping_add(self.w(0x4470b2)?) as i16;
                    self.word(0x4434b4, self.w(index(0x478288, slot, 2))?)?;
                }
                if self.sw(0x4434b4)? < 0 {
                    return Ok(());
                }
                if self.sw(0x4434b4)? < 2000 {
                    self.load_item(self.sw(0x4434b4)?)?;
                    quantity = self.sw(0x453650)?;
                }
            } else {
                let item = self.l(0x439404)? == self.l(0x4393fc)?;
                if !item {
                    self.word(0x4471f4, 10)?;
                }
                let base = if item { 0x447f90 } else { 0x44734e };
                let node = base + u32::from(self.w(0x4470b2)?.wrapping_add(row as u16)) * 22;
                self.word(0x4434b4, self.w(node)?)?;
                if row
                    > self
                        .sw(if item { 0x447e78 } else { 0x447236 })?
                        .wrapping_sub(1)
                {
                    self.word(0x4434b4, 0)?;
                }
                let node = base + u32::from(self.w(0x4470b2)?.wrapping_add(row as u16)) * 22;
                quantity = self.sw(node.wrapping_add(2))?;
                let node = base + u32::from(self.w(0x4470b2)?.wrapping_add(row as u16)) * 22;
                self.byte(0x44275a, self.b(node.wrapping_add(9))?)?;
                self.byte(0x41f708, row.wrapping_add(self.sw(0x4470b2)?) as u8)?;
                self.byte(0x41f709, self.b(if item { 0x44273e } else { 0x44273d })?)?;
                let source = base + u32::from(self.sb(0x41f708)? as u16) * 22;
                for offset in [0, 4, 8, 12, 16] {
                    self.long(0x41fc48 + offset, self.l(source + offset)?)?;
                }
                self.word(0x41fc5c, self.w(source + 20)?)?;
            }
            self.word(0x4471f2, row.wrapping_mul(40).wrapping_add(16) as u16)?;
            if self.b(0x44273e)? == 255 && self.l(0x439404)? == self.l(0x4393fc)? {
                for (dst, src) in [
                    (0x41fc48, 0x4536a2),
                    (0x41fc4e, 0x453650),
                    (0x41fc4a, 0x453650),
                ] {
                    self.word(dst, self.w(src)?)?;
                }
                self.byte(0x41fc51, 0)?;
                self.word(0x41fc4c, 0)?;
                for (dst, src) in [
                    (0x41fc50, 0x45365f),
                    (0x41fc52, 0x453657),
                    (0x41fc56, 0x45368a),
                    (0x41fc57, 0x45368d),
                    (0x41fc58, 0x45368f),
                    (0x41fc59, 0x453690),
                    (0x41fc5a, 0x453691),
                    (0x41fc5b, 0x453692),
                    (0x41fc5d, 0x45368c),
                    (0x41fc5c, 0x45368b),
                ] {
                    self.byte(dst, self.b(src)?)?;
                }
            }
            self.plot_item(self.sw(0x4434b4)?)?;
            if self.w(0x4434b4)? != 0 {
                let child = frame.wrapping_sub(28);
                self.word(child.wrapping_add(10), 0)?;
                self.word(child.wrapping_add(8), quantity as u16)?;
                let child =
                    recovered_equipment_ui::show_charge(&self.plan.memory, child, self.services)?;
                self.plan.memory = child.memory;
                self.plan.effects.extend(
                    child
                        .effects
                        .into_iter()
                        .map(CharacterPanelEffect::Equipment),
                );
            }
            self.byte(0x44275a, 0)?;
            row = row.wrapping_add(1);
        }
        // Both final source tests converge to the same MOVEQ 0 return.
        if self.w(0x43875e)? == 0 {
            self.w(0x438756)?;
        }
        Ok(())
    }
    fn load_item(&mut self, id: i16) -> Result<(), RuleError> {
        let backing = item_tables(&self.plan.memory, id)?;
        let tables = ItemTables {
            families: std::array::from_fn(|i| backing[i].as_slice()),
        };
        let current = read(&self.plan.memory, 0x45363c, ITEM_BYTES)?
            .try_into()
            .expect("item size");
        let child = records::loaditem(&tables, &current, id)?;
        for w in child.writes {
            self.write(w.address, &w.bytes)?;
        }
        Ok(())
    }
    fn plot_item(&mut self, id: i16) -> Result<(), RuleError> {
        let backing = item_tables(&self.plan.memory, id)?;
        let tables = ItemTables {
            families: std::array::from_fn(|i| backing[i].as_slice()),
        };
        let current = read(&self.plan.memory, 0x45363c, ITEM_BYTES)?
            .try_into()
            .expect("item size");
        let loaded = records::loaditemnotype(&tables, &current, id)?;
        let selection = crate::rules::selection_base(i16::from_be_bytes([
            loaded.bytes[102],
            loaded.bytes[103],
        ]));
        let mut context = icon_context(self.rect(0x4471f2)?);
        context.item_work = current;
        context.window = self.l(0x439404)?;
        context.item_window = self.l(0x4393fc)?;
        context.half_width_input = self.sw(0x40856c)?;
        context.omit_banner = self.b(0x442756)?;
        if context.omit_banner == 0 {
            context.banner_rect = self.rect(0x4471ac)?;
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
        Ok(())
    }
}

fn index(base: u32, index: i16, stride: i32) -> u32 {
    base.wrapping_add((i32::from(index).wrapping_mul(stride)) as u32)
}
fn record(member: i16) -> u32 {
    0x448aa0_u32.wrapping_add(u32::from(member as u16) * 0xc42)
}
fn half(value: i16) -> i16 {
    value.wrapping_add(((value as u16) >> 15) as i16) >> 1
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

macro_rules! entry {
    ($name:ident, $body:ident $(, $arg:expr)?) => {
        pub fn $name(memory: &RenderingMemory, frame: u32, maximum_steps: u32, services: &mut impl CharacterPanelServices) -> Result<CharacterPanelPlan, RuleError> {
            let mut runner = Runner { plan: CharacterPanelPlan { memory: memory.clone(), effects: vec![], native_return: 0 }, services, remaining: maximum_steps };
            runner.$body(frame $(, $arg)?)?;
            Ok(runner.plan)
        }
    };
}
entry!(display, display);
entry!(update_port, update_port);
entry!(plot_portrait, portrait);
entry!(update_picture_box, picture_box);
entry!(update_character, character, false);
entry!(update_character_short, character, true);
entry!(update_preparation, prep);
entry!(short_update, short);
entry!(copy_out, out);
entry!(select_update, selected);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recovered_drawing::{BuildingWorld, DrawingServices, PictureObservation};
    use crate::recovered_interface::Point;
    use crate::recovered_item_ui::{ItemUiReply, ItemUiRequest, ItemUiServices};
    use crate::recovered_runtime::{RuntimeReply, RuntimeRequest};
    use crate::recovered_text::NumericServices;

    const FRAME: u32 = 0x8000;
    struct Services {
        widths: Vec<u16>,
        disposal_alias: bool,
        control: u16,
    }
    impl Services {
        fn new() -> Self {
            Self {
                widths: vec![39, 40],
                disposal_alias: false,
                control: 0,
            }
        }
    }
    fn fixture() -> RenderingMemory {
        let mut memory = RenderingMemory::default();
        for (address, length) in [
            (0x3fe300, 0x100),
            (0x402f30, 0x20),
            (0x408560, 0x30),
            (0x41f700, 0x560),
            (0x435600, 0x400),
            (0x438500, 0x1200),
            (0x442700, 0xe00),
            (0x447000, 0x2a00),
            (0x453600, 0x300),
            (0x479ed8, 0x100),
            (0x7c00, 0x480),
            (0x9000, 0x400),
            (0xa000, 0x100),
        ] {
            memory.insert_region(address, &vec![0; length]);
        }
        for (address, pointer) in [
            (0x3fe302, 0xa000_u32),
            (0x43878c, 0xa000),
            (0x43879c, 0xa100),
            (0x438798, 0xa200),
            (0x438788, 0xa300),
            (0x4394d0, 0xa400),
            (0x4394bc, 0xa500),
            (0x9000, 0x9200),
        ] {
            memory.insert_region(address, &pointer.to_be_bytes());
        }
        memory.insert_region(0x924a, &40_u16.to_be_bytes());
        memory.insert_region(0x40856c, &100_u16.to_be_bytes());
        memory.insert_region(0x439582, &1_u16.to_be_bytes());
        memory.insert_region(0xa044, &7_u16.to_be_bytes());
        memory.insert_region(0xa046, &[0x80]);
        memory.insert_region(0xa048, &2_u16.to_be_bytes());
        memory.insert_region(0xa04a, &14_u16.to_be_bytes());
        memory
    }
    fn args(memory: &mut RenderingMemory, member: u16, mode: u16) {
        memory.insert_region(FRAME + 8, &member.to_be_bytes());
        memory.insert_region(FRAME + 10, &mode.to_be_bytes());
    }
    fn portrait_args(memory: &mut RenderingMemory, member: u16) {
        for (offset, value) in [
            (8, 123_u16),
            (10, 10),
            (12, 20),
            (14, 100),
            (16, 200),
            (18, 12),
            (20, member),
            (22, 0),
        ] {
            memory.insert_region(FRAME + offset, &value.to_be_bytes());
        }
    }
    fn number(argument: &PanelArgument) -> u32 {
        match argument {
            PanelArgument::Byte(v) => u32::from(*v),
            W(v) => u32::from(*v),
            L(v) | P(v) => *v,
        }
    }
    fn pascal(memory: &RenderingMemory, pointer: u32) -> Result<Vec<u8>, RuleError> {
        read(memory, pointer, usize::from(memory.byte(pointer)?) + 1)
    }
    fn cstring(memory: &RenderingMemory, pointer: u32) -> Result<Vec<u8>, RuleError> {
        let mut bytes = Vec::new();
        for n in 0..1024 {
            let b = memory.byte(pointer.wrapping_add(n))?;
            bytes.push(b);
            if b == 0 {
                return Ok(bytes);
            }
        }
        Err(RuleError::Invalid("test C string limit".into()))
    }
    impl CharacterPanelServices for Services {
        fn panel_observe(
            &mut self,
            request: &CharacterPanelRequest,
            memory: &RenderingMemory,
        ) -> Result<CharacterPanelObservation, RuleError> {
            let mut memory = memory.clone();
            let mut value = 0;
            match request {
                CharacterPanelRequest::Toolbox {
                    selector,
                    arguments,
                } => {
                    let at = |i: usize| number(&arguments[i]);
                    match selector {
                        0xa874 => {
                            let port = memory.long(0x3fe302)?;
                            memory.insert_region(at(0), &port.to_be_bytes());
                        }
                        0xa873 => memory.insert_region(0x3fe302, &at(0).to_be_bytes()),
                        0xaa19 | 0xaa1a => memory.insert_region(at(0), &[1, 2, 3, 4, 5, 6]),
                        0xaa1e => value = 0x9000,
                        0xaa1f if self.disposal_alias => {
                            memory.insert_region(0x439598, &0x9004_u32.to_be_bytes())
                        }
                        0xa960 => value = u32::from(self.control),
                        0xa88c => value = u32::from(self.widths.remove(0)),
                        0xa8a7 => {
                            for (offset, v) in [(0, at(2)), (2, at(1)), (4, at(4)), (6, at(3))] {
                                memory.insert_region(at(0) + offset, &(v as u16).to_be_bytes());
                            }
                        }
                        0xa8a8 | 0xa8a9 => {
                            let pointer = at(0);
                            let horizontal = (at(1) >> 16) as u16;
                            let vertical = at(1) as u16;
                            for (offset, delta, subtract) in [
                                (0, vertical, false),
                                (2, horizontal, false),
                                (4, vertical, *selector == 0xa8a9),
                                (6, horizontal, *selector == 0xa8a9),
                            ] {
                                let before = memory.word(pointer + offset)?;
                                memory.insert_region(
                                    pointer + offset,
                                    &if subtract {
                                        before.wrapping_sub(delta)
                                    } else {
                                        before.wrapping_add(delta)
                                    }
                                    .to_be_bytes(),
                                );
                            }
                        }
                        _ => {}
                    }
                }
                CharacterPanelRequest::RuntimeString { entry, arguments } => {
                    let at = |i: usize| number(&arguments[i]);
                    match entry {
                        0x400178 => memory.insert_region(at(0), b"\x0aABCDEFGHIJ"),
                        0x400180 => {
                            let bytes = pascal(&memory, at(0))?;
                            let mut c = bytes[1..].to_vec();
                            c.push(0);
                            memory.insert_region(at(0), &c);
                        }
                        0x400090 => {
                            let bytes = cstring(&memory, at(0))?;
                            memory.insert_region(at(1), &bytes);
                        }
                        0x400188 => {
                            let c = cstring(&memory, at(0))?;
                            let mut p = vec![(c.len() - 1) as u8];
                            p.extend_from_slice(&c[..c.len() - 1]);
                            memory.insert_region(at(0), &p);
                        }
                        0x400168 => {
                            let text = (at(0) as i32).to_string();
                            let p = PascalString::from_payload(text.as_bytes())?;
                            memory.insert_region(at(1), p.raw());
                        }
                        _ => return Err(RuleError::Invalid("unexpected test string entry".into())),
                    }
                }
            }
            Ok(CharacterPanelObservation { value, memory })
        }
        fn runtime_observations(
            &mut self,
            entry: u32,
            memory: &RenderingMemory,
        ) -> Result<Vec<RuntimeObservation>, RuleError> {
            if entry != 0x13e13c {
                return Err(RuleError::Invalid("unused test runtime entry".into()));
            }
            let mut observations = Vec::new();
            for address in [0x4386d8, 0x4386e0, 0x4386e4, 0x4386e8, 0x4386dc] {
                observations.push(RuntimeObservation {
                    request: RuntimeRequest::UnlockPixels(memory.long(address)?),
                    reply: RuntimeReply::Ignored,
                });
            }
            observations.push(RuntimeObservation {
                request: RuntimeRequest::MaxMem,
                reply: RuntimeReply::MaxMem {
                    d0: 0x12345678,
                    a0: 0x11223344,
                },
            });
            for (n, address) in [0x4386b4, 0x4386bc, 0x4386c0, 0x4386c4, 0x4386b8]
                .into_iter()
                .enumerate()
            {
                let pointer = 0xb000 + n as u32;
                observations.push(RuntimeObservation {
                    request: RuntimeRequest::GetGWorldPixMap(memory.long(address)?),
                    reply: RuntimeReply::Pointer(pointer),
                });
                observations.push(RuntimeObservation {
                    request: RuntimeRequest::LockPixels(pointer),
                    reply: RuntimeReply::LockPixels {
                        boolean: 1,
                        native_d0: 0x40001,
                    },
                });
            }
            Ok(observations)
        }
    }
    impl DrawingServices for Services {
        fn color_icon(&mut self, id: i16) -> Result<u32, RuleError> {
            Ok(0x10000 + u32::from(id as u16))
        }
        fn indexed_string(&mut self, _: i16, _: i16) -> Result<PascalString, RuleError> {
            Err(RuleError::Invalid("unused test indexed string".into()))
        }
        fn current_world(&mut self) -> Result<(u32, u32), RuleError> {
            Err(RuleError::Invalid("unused test world".into()))
        }
        fn picture(&mut self, _: i16) -> Result<PictureObservation, RuleError> {
            Ok(PictureObservation {
                handle: 0,
                bounds: None,
            })
        }
        fn new_world(&mut self, _: i16, _: BodyRect) -> Result<(i16, BuildingWorld), RuleError> {
            Err(RuleError::Invalid("unused test new world".into()))
        }
    }
    impl NumericServices for Services {
        fn number_to_string(&mut self, value: i32) -> Result<PascalString, RuleError> {
            PascalString::from_payload(value.to_string().as_bytes())
        }
        fn string_to_number(&mut self, _: &PascalString) -> Result<i32, RuleError> {
            Err(RuleError::Invalid("unused test parse".into()))
        }
    }
    impl ItemUiServices for Services {
        fn observe(
            &mut self,
            _: &ItemUiRequest,
            _: &RenderingMemory,
        ) -> Result<ItemUiReply, RuleError> {
            Err(RuleError::Invalid("unused test item UI".into()))
        }
    }
    impl EquipmentUiServices for Services {
        fn pen_position(
            &mut self,
            _: u32,
            _: &RenderingMemory,
        ) -> Result<Vec<RecordWrite>, RuleError> {
            Err(RuleError::Invalid("unused test pen".into()))
        }
    }
    impl InterfaceServices for Services {
        fn word(&mut self, _: u32) -> Result<i16, RuleError> {
            Err(RuleError::Invalid("unused test light".into()))
        }
        fn current_port(&mut self) -> Result<u32, RuleError> {
            Err(RuleError::Invalid("unused test button port".into()))
        }
        fn pen_state(&mut self) -> Result<[u8; 18], RuleError> {
            Err(RuleError::Invalid("unused test pen state".into()))
        }
        fn local_to_global(&mut self, _: Point) -> Result<Point, RuleError> {
            Err(RuleError::Invalid("unused test point".into()))
        }
        fn point_in_rect(&mut self, _: Point, _: BodyRect) -> Result<u8, RuleError> {
            Err(RuleError::Invalid("unused test rectangle".into()))
        }
        fn new_region(&mut self) -> Result<u32, RuleError> {
            Err(RuleError::Invalid("unused test region".into()))
        }
        fn main_device_rect(&mut self) -> Result<BodyRect, RuleError> {
            Err(RuleError::Invalid("unused test device".into()))
        }
    }
    fn raw(plan: &CharacterPanelPlan, selector: u16) -> Vec<&[PanelArgument]> {
        plan.effects
            .iter()
            .filter_map(|e| match e {
                CharacterPanelEffect::Service {
                    request:
                        CharacterPanelRequest::Toolbox {
                            selector: s,
                            arguments,
                        },
                    ..
                } if *s == selector => Some(arguments.as_slice()),
                _ => None,
            })
            .collect()
    }
    fn icons(plan: &CharacterPanelPlan) -> Vec<(u32, BodyRect)> {
        plan.effects
            .iter()
            .filter_map(|e| match e {
                CharacterPanelEffect::Drawing(DrawingEffect::PlotColorIcon { handle, rect }) => {
                    Some((*handle, *rect))
                }
                _ => None,
            })
            .collect()
    }
    #[test]
    fn character_erase_and_picture_modes_return_without_restoring_port() {
        let mut memory = fixture();
        args(&mut memory, 2, 2);
        let before = memory.clone();
        let result = update_character(&memory, FRAME, 10, &mut Services::new()).unwrap();
        assert_eq!(
            result.memory.rect(FRAME - 8).unwrap(),
            BodyRect {
                top: 100,
                left: 420,
                bottom: 150,
                right: 740
            }
        );
        assert_eq!(raw(&result, 0xa8a3), vec![vec![P(FRAME - 8)].as_slice()]);
        assert!(raw(&result, 0xa873).is_empty());
        assert_eq!(memory, before);
        args(&mut memory, 2, 4);
        let result = update_character(&memory, FRAME, 10, &mut Services::new()).unwrap();
        assert!(result.effects.iter().any(|e| matches!(
            e,
            CharacterPanelEffect::Drawing(DrawingEffect::GetPicture(184))
        )));
        assert!(raw(&result, 0xa873).is_empty());
        assert_eq!(result.native_return, 0);
    }
    #[test]
    fn portrait_status_icons_preserve_source_order_and_restore_expanded_arguments() {
        let mut memory = fixture();
        portrait_args(&mut memory, 0);
        for offset in [94, 96, 100, 104, 166] {
            memory.insert_region(record(0) + offset, &1_u16.to_be_bytes());
        }
        memory.insert_region(record(0) + 144, &u16::MAX.to_be_bytes());
        let result = plot_portrait(&memory, FRAME, 10, &mut Services::new()).unwrap();
        let icons = icons(&result);
        assert_eq!(
            icons.iter().map(|(h, _)| *h).collect::<Vec<_>>(),
            [180, 183, 181, 182, 185, 184].map(|id| 0x10000 + id)
        );
        assert_eq!(
            icons[0].1,
            BodyRect {
                top: 8,
                left: 658,
                bottom: 24,
                right: 674
            }
        );
        assert_eq!(
            icons[5].1,
            BodyRect {
                top: 8,
                left: 578,
                bottom: 24,
                right: 594
            }
        );
        assert_eq!(
            result.memory.rect(FRAME + 10).unwrap(),
            BodyRect {
                top: 6,
                left: 16,
                bottom: 46,
                right: 56
            }
        );
        assert!(raw(&result, 0xa873).is_empty());
        let mut missing = memory.clone();
        missing.bytes.remove(&0x9000);
        let before = missing.clone();
        assert!(plot_portrait(&missing, FRAME, 10, &mut Services::new()).is_err());
        assert_eq!(missing, before);
    }
    #[test]
    fn portrait_label_clips_once_and_rereads_handle_after_observed_plot() {
        let mut memory = fixture();
        portrait_args(&mut memory, u16::MAX);
        memory.insert_region(0x439584, &1_u16.to_be_bytes());
        let mut services = Services::new();
        services.disposal_alias = true;
        let result = plot_portrait(&memory, FRAME, 10, &mut services).unwrap();
        assert_eq!(raw(&result, 0xaa25), vec![vec![L(0x9004)].as_slice()]);
        assert_eq!(result.memory.word(0x4434b6).unwrap(), 39);
        assert_eq!(pascal(&result.memory, FRAME - 256).unwrap(), b"\x06ABCDEF");
        assert_eq!(raw(&result, 0xa888).last().unwrap(), &&[W(0x80)][..]);
        assert_eq!(raw(&result, 0xa887).last().unwrap(), &&[W(7)][..]);
        assert_eq!(raw(&result, 0xa88a).last().unwrap(), &&[W(14)][..]);
        assert_eq!(raw(&result, 0xa889).last().unwrap(), &&[W(2)][..]);
        assert_eq!(raw(&result, 0xa873), vec![vec![L(0xa000)].as_slice()]);
    }
    #[test]
    fn picture_box_uses_unsigned_record_and_dead_overlay_rects() {
        let mut memory = fixture();
        args(&mut memory, 0, 1);
        memory.insert_region(FRAME + 12, &1_u16.to_be_bytes());
        memory.insert_region(record(0) + 80, &(-10_i16).to_be_bytes());
        let result = update_picture_box(&memory, FRAME, 10, &mut Services::new()).unwrap();
        let icons = icons(&result);
        assert_eq!(
            icons,
            vec![
                (
                    0x10000 + 2019,
                    BodyRect {
                        top: 5,
                        left: 435,
                        bottom: 45,
                        right: 475
                    }
                ),
                (
                    0x10000 + 2015,
                    BodyRect {
                        top: 9,
                        left: 439,
                        bottom: 41,
                        right: 471
                    }
                )
            ]
        );
        assert_eq!(
            result.memory.rect(0x4471f2).unwrap(),
            BodyRect {
                top: 9,
                left: 439,
                bottom: 41,
                right: 471
            }
        );
        assert_eq!(result.memory.word(FRAME - 28 + 20).unwrap(), u16::MAX);
        args(&mut memory, 0, 0);
        let background = update_picture_box(&memory, FRAME, 10, &mut Services::new()).unwrap();
        assert_eq!(background.memory.word(FRAME - 28 + 20).unwrap(), u16::MAX);
        assert_eq!(
            raw(&background, 0xa874),
            vec![vec![P(FRAME - 28 - 294)].as_slice()]
        );
        assert!(background.effects.iter().any(|effect| matches!(
            effect,
            CharacterPanelEffect::Drawing(DrawingEffect::GetPicture(1))
        )));
        assert!(update_picture_box(&memory, FRAME, 1, &mut Services::new()).is_err());
        args(&mut memory, u16::MAX, 1);
        assert!(update_picture_box(&memory, FRAME, 10, &mut Services::new()).is_err());
    }
    #[test]
    fn character_numeric_values_use_signed_words_and_distinct_zero_charge_literals() {
        let mut memory = fixture();
        args(&mut memory, 0, 0);
        memory.insert_region(record(0) + 0xa21, b"NAME\0");
        for (offset, value) in [(18, -2_i16), (80, -3), (36, -4), (38, -5), (44, -6)] {
            memory.insert_region(record(0) + offset, &value.to_be_bytes());
        }
        let result = update_character(&memory, FRAME, 10, &mut Services::new()).unwrap();
        assert!(raw(&result, 0xa884).contains(&&[P(0x400a76)][..]));
        let values: Vec<_> = result
            .effects
            .iter()
            .filter_map(|e| match e {
                CharacterPanelEffect::Text(TextEffect::NumberToString(n)) => Some(*n),
                _ => None,
            })
            .collect();
        assert_eq!(values, [-2, -4, -5, -6]);
        assert_eq!(raw(&result, 0xa873).last().unwrap(), &&[L(0xa000)][..]);
        let result = update_character_short(&memory, FRAME, 10, &mut Services::new()).unwrap();
        assert!(raw(&result, 0xa884).contains(&&[P(0x400a70)][..]));
        assert_eq!(result.memory.rect(FRAME - 8).unwrap().left, 339);
        assert_eq!(
            cstring(&result.memory, record(0) + 0xa21).unwrap(),
            b"NAME\0"
        );
    }
    #[test]
    fn display_uses_control_reply_single_row_and_real_runtime_child() {
        let mut memory = fixture();
        memory.insert_region(0x439404, &0x55_u32.to_be_bytes());
        memory.insert_region(0x439400, &0x55_u32.to_be_bytes());
        memory.insert_region(0x4393fc, &0x66_u32.to_be_bytes());
        memory.insert_region(0x4427b1, &[1]);
        memory.insert_region(0x4471e2, &1_u16.to_be_bytes());
        memory.insert_region(0x447236, &1_u16.to_be_bytes());
        memory.insert_region(0x442756, &[1]);
        memory.insert_region(records::ITEM_TABLE_ADDRESSES[0], &[0; ITEM_BYTES]);
        let mut services = Services::new();
        services.control = 2;
        let result = display(&memory, FRAME, 10, &mut services).unwrap();
        assert_eq!(result.memory.word(0x4470b2).unwrap(), 2);
        assert_eq!(result.memory.word(0x4471f2).unwrap(), 16);
        assert_eq!(result.memory.word(0x4471f4).unwrap(), 10);
        assert_eq!(result.memory.byte(0x41f708).unwrap(), 2);
        assert_eq!(result.memory.long(0x4386d8).unwrap(), 0xb000);
        assert_eq!(result.memory.long(0x4386dc).unwrap(), 0xb004);
        assert_eq!(result.memory.byte(0x4427b1).unwrap(), 0);
        assert_eq!(result.memory.byte(0x44275a).unwrap(), 0);
        assert_eq!(
            result
                .effects
                .iter()
                .filter(|e| matches!(e, CharacterPanelEffect::Runtime(RuntimeEffect::Service(_))))
                .count(),
            16
        );
        assert!(display(&memory, FRAME, 1, &mut Services::new()).is_err());
    }
}
