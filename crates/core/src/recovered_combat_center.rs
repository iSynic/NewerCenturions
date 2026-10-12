//! Combat view centering and range overlays from complete CODE 1 bodies.

use crate::recovered_attack::DamageFrame;
use crate::recovered_bodies::{BodyEffect, BodyGroundInput, BodyState};
use crate::recovered_combat_panel::{CombatPanelEffect, CombatPanelServices};
use crate::recovered_combat_ui::{CombatUiEffect, CombatUiServices};
use crate::recovered_drawing::{BuildingWorld, DrawingContext, DrawingEffect};
use crate::recovered_feedback::{FeedbackArg, FeedbackEffect, FeedbackRequest, FeedbackServices};
use crate::recovered_geometry::BodyRect;
use crate::recovered_interface::{InterfaceEffect, InterfaceServices, Point};
use crate::recovered_records::RecordWrite;
use crate::recovered_rendering::{RenderingEffect, RenderingMemory, RenderingServices};
use crate::recovered_spatial::{Position, RangeWork, SpatialServices};
use crate::recovered_text::PascalString;
use crate::recovered_world_display::{WorldDisplayEffect, WorldDisplayServices, WorldDisplayState};
use crate::{
    RuleError, recovered_bodies, recovered_combat_panel, recovered_combat_ui, recovered_drawing,
    recovered_feedback, recovered_interface, recovered_navigation, recovered_rendering,
    recovered_spatial, recovered_world_display,
};
use FeedbackArg::{Long as L, Pointer as P, Word as W};
use centurions_content::ITEM_BYTES;

pub trait CombatCenterServices:
    RenderingServices + FeedbackServices + CombatUiServices + WorldDisplayServices
{
    /// Actual record/position inputs for bodyground's existing captured view.
    fn bodyground_state(&mut self, _memory: &RenderingMemory) -> Result<BodyState, RuleError> {
        Err(RuleError::Unsupported {
            action: "combat_center.bodyground",
            dependency: "actual bodyground record and position view",
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum CombatCenterEffect {
    Write(RecordWrite),
    Feedback(FeedbackEffect),
    Rendering(RenderingEffect),
    Combat(CombatUiEffect),
    Panel(CombatPanelEffect),
    World(WorldDisplayEffect),
    Drawing(DrawingEffect),
    Interface(InterfaceEffect),
    Range {
        body: i16,
        target: Position,
        work: RangeWork,
    },
    Between {
        value: i16,
        result: i32,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct CombatCenterPlan {
    pub memory: RenderingMemory,
    pub effects: Vec<CombatCenterEffect>,
    /// Void dispatchers preserve an incoming D0 when they do not call a child.
    pub native_return: Option<u32>,
}

struct Execution {
    plan: CombatCenterPlan,
    remaining: u32,
}
impl Execution {
    fn new(memory: &RenderingMemory, limit: u32, native_return: Option<u32>) -> Self {
        Self {
            plan: CombatCenterPlan {
                memory: memory.clone(),
                effects: vec![],
                native_return,
            },
            remaining: limit,
        }
    }
    fn step(&mut self) -> Result<(), RuleError> {
        self.remaining = self.remaining.checked_sub(1).ok_or_else(|| {
            RuleError::Invalid("combat centering processing budget exhausted".into())
        })?;
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
    fn write(&mut self, a: u32, bytes: &[u8]) -> Result<(), RuleError> {
        for i in 0..bytes.len() {
            self.b(a.wrapping_add(i as u32))?;
        }
        self.plan.memory.insert_region(a, bytes);
        self.plan
            .effects
            .push(CombatCenterEffect::Write(RecordWrite {
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
    fn trap(
        &mut self,
        opcode: u16,
        arguments: Vec<FeedbackArg>,
        s: &impl CombatCenterServices,
    ) -> Result<u32, RuleError> {
        self.step()?;
        let request = FeedbackRequest::Toolbox { opcode, arguments };
        let observation = FeedbackServices::observe(s, &request, &self.plan.memory)?;
        let value = observation.value;
        self.plan.memory = observation.memory.clone();
        self.plan
            .effects
            .push(CombatCenterEffect::Feedback(FeedbackEffect::Service {
                request,
                observation,
            }));
        Ok(value)
    }
    fn port(&mut self, pointer: u32, s: &impl CombatCenterServices) -> Result<(), RuleError> {
        self.trap(0xa873, vec![L(pointer)], s)?;
        Ok(())
    }
    fn sound(&mut self, id: i16, s: &impl CombatCenterServices) -> Result<(), RuleError> {
        let child = recovered_feedback::sound(&self.plan.memory, id, s)?;
        self.plan.memory = child.memory;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(CombatCenterEffect::Feedback));
        Ok(())
    }
    fn render(&mut self, child: recovered_rendering::RenderingPlan) {
        self.plan.memory = child.memory;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(CombatCenterEffect::Rendering));
    }
    fn drawbody(
        &mut self,
        frame: u32,
        body: i16,
        force: i16,
        secondary: i16,
        s: &impl CombatCenterServices,
    ) -> Result<(), RuleError> {
        self.ww(frame.wrapping_add(12), secondary)?;
        self.ww(frame.wrapping_add(10), force)?;
        self.ww(frame.wrapping_add(8), body)?;
        let child =
            recovered_rendering::drawbody(&self.plan.memory, frame, body, force, secondary, s)?;
        self.render(child);
        Ok(())
    }
    fn merge(&mut self, child: CombatCenterPlan) {
        self.plan.memory = child.memory;
        self.plan.effects.extend(child.effects);
        self.plan.native_return = child.native_return;
    }
}
fn idx(a: u32, n: i16, stride: i32) -> u32 {
    a.wrapping_add((i32::from(n) * stride) as u32)
}
fn rec(a: u32, n: i16, stride: u32) -> u32 {
    a.wrapping_add(u32::from(n as u16) * stride)
}

/// showrange, 0x1580be. The real PointInRect reply selects the overlay origin.
pub fn showrange(
    memory: &RenderingMemory,
    frame: u32,
    limit: u32,
    s: &impl CombatCenterServices,
) -> Result<CombatCenterPlan, RuleError> {
    let mut e = Execution::new(memory, limit, Some(0));
    e.port(e.l(0x43879c)?, s)?;
    e.trap(0xab1d, vec![L(0x80005), P(0x4046da), P(0x4046de)], s)?;
    if e.w(frame.wrapping_add(8))? == 0 {
        e.ww(0x4434aa, 0)?;
        while e.w(0x4434aa)? < e.w(0x43874a)? {
            e.step()?;
            if e.w(rec(0x439e4c, e.w(0x4434aa)?, 288))? > 0 {
                let body = e.w(0x4434aa)?.wrapping_add(10);
                e.drawbody(frame.wrapping_sub(34), body, 1, 0, s)?;
            }
            e.addw(0x4434aa, 1)?;
        }
        e.ww(0x4434aa, 0)?;
        while e.w(0x4434aa)? <= i16::from(e.b(0x44273a)? as i8) {
            e.step()?;
            if e.b(rec(0x44947f, e.w(0x4434aa)?, 3138))? != 0 {
                let body = e.w(0x4434aa)?;
                e.drawbody(frame.wrapping_sub(34), body, 1, 0, s)?;
            }
            e.addw(0x4434aa, 1)?;
        }
    }
    e.ww(0x4471f0, e.w(idx(0x44709a, e.w(0x4435d2)?, 4))?)?;
    e.ww(0x4471ee, e.w(idx(0x44709c, e.w(0x4435d2)?, 4))?)?;
    e.trap(0xa8a7, vec![P(0x4434ba), W(0), W(0), W(9), W(9)], s)?;
    let inside = e.trap(0xa8ad, vec![L(e.l(0x4471ee)?), P(0x4434ba)], s)? as u8 != 0;
    let (origin_x, origin_y) = if inside {
        let x = e
            .w(idx(0x44709a, e.w(0x4435d2)?, 4))?
            .wrapping_shl(5)
            .wrapping_add(16);
        let y = e
            .w(idx(0x44709c, e.w(0x4435d2)?, 4))?
            .wrapping_shl(5)
            .wrapping_add(16);
        (x, y)
    } else {
        (160, 160)
    };
    if e.w(frame.wrapping_add(8))? == 0 {
        e.port(e.l(0x43879c)?, s)?;
    } else {
        e.trap(0xab1d, vec![L(0x80006), L(e.l(0x4386b4)?), L(0)], s)?;
    }
    e.trap(0xa89c, vec![W(0)], s)?;
    e.ww(0x4434aa, 0)?;
    while e.w(0x4434aa)? < e.w(0x43874a)? {
        e.step()?;
        if e.w(rec(0x439e4c, e.w(0x4434aa)?, 288))? > 0 {
            let size = e.w(rec(0x439d4c, e.w(0x4434aa)?, 288))?.wrapping_abs();
            let mut x = e.w(idx(0x446f0a, e.w(0x4434aa)?, 4))?.wrapping_shl(5);
            let mut y = e.w(idx(0x446f0c, e.w(0x4434aa)?, 4))?.wrapping_shl(5);
            if size < 2 {
                x = x.wrapping_add(16);
            }
            if size == 2 || size == 0 {
                y = y.wrapping_add(16);
            }
            e.trap(0xa862, vec![L(205)], s)?;
            if e.w(rec(0x439d4a, e.w(0x4434aa)?, 288))? == 0 {
                e.trap(0xaa14, vec![P(0x443374)], s)?;
            } else if e.b(rec(0x439dec, e.w(0x4434aa)?, 288))? == 255
                || e.b(rec(0x439def, e.w(0x4434aa)?, 288))? != 0
            {
                e.trap(0xa862, vec![L(409)], s)?;
            }
            e.trap(0xa893, vec![W(origin_x as u16), W(origin_y as u16)], s)?;
            e.trap(0xa891, vec![W(x as u16), W(y as u16)], s)?;
        }
        e.addw(0x4434aa, 1)?;
    }
    e.ww(0x4434aa, 0)?;
    while e.w(0x4434aa)? <= i16::from(e.b(0x44273a)? as i8) {
        e.step()?;
        if e.b(rec(0x44947f, e.w(0x4434aa)?, 3138))? != 0 {
            e.trap(0xaa14, vec![P(0x443374)], s)?;
            if e.w(rec(0x448ab0, e.w(0x4434aa)?, 3138))? != 0 {
                e.trap(0xa862, vec![L(205)], s)?;
            }
            if e.w(rec(0x448b00, e.w(0x4434aa)?, 3138))? != 0 {
                e.trap(0xa862, vec![L(409)], s)?;
            }
            e.trap(0xa893, vec![W(origin_x as u16), W(origin_y as u16)], s)?;
            let x = e
                .w(idx(0x44709a, e.w(0x4434aa)?, 4))?
                .wrapping_shl(5)
                .wrapping_add(16);
            let y = e
                .w(idx(0x44709c, e.w(0x4434aa)?, 4))?
                .wrapping_shl(5)
                .wrapping_add(16);
            e.trap(0xa891, vec![W(x as u16), W(y as u16)], s)?;
        }
        e.addw(0x4434aa, 1)?;
    }
    e.trap(0xa862, vec![L(33)], s)?;
    e.trap(0xa89c, vec![W(2)], s)?;
    if e.w(frame.wrapping_add(8))? == 0 {
        e.sound(137, s)?;
        e.trap(0xa032, vec![L(65535)], s)?;
        loop {
            e.trap(0xa860, vec![W(65535), P(0x44720a), L(0), L(0)], s)?;
            if matches!(e.w(0x44720a)?, 1 | 3) {
                break;
            }
        }
    } else {
        e.trap(
            0xab1d,
            vec![L(0x80006), L(e.l(0x4046da)?), L(e.l(0x4046de)?)],
            s,
        )?;
    }
    Ok(e.plan)
}

fn fastplot(
    e: &mut Execution,
    tile: i16,
    destination: BodyRect,
    copy: i8,
    s: &mut impl CombatCenterServices,
) -> Result<(), RuleError> {
    let rect = if tile >= 23000 {
        e.plan.memory.rect(0x4471f2)?
    } else {
        BodyRect {
            top: 0,
            left: 0,
            bottom: 0,
            right: 0,
        }
    };
    let mut buildings = vec![];
    let mut depth = 0;
    if tile >= 23000 {
        let index = usize::from(tile.wrapping_sub(23000) as u16);
        let pointer = e.l(0x41c220 + index as u32 * 4)?;
        let bounds = if pointer == 0 {
            depth = e.b(0x43873f)? as i8;
            None
        } else {
            Some(e.plan.memory.rect(pointer.wrapping_add(16))?)
        };
        // Prefix entries and unused fields are adapter storage, never native writes.
        buildings.resize(
            index + 1,
            BuildingWorld {
                pointer: 0,
                bounds: None,
            },
        );
        buildings[index] = BuildingWorld { pointer, bounds };
    }
    let mut context = DrawingContext {
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
        map_port: if tile >= 0 { e.l(0x43879c)? } else { 0 },
        text_port: 0,
        secondary_port: if (0..23000).contains(&tile) && copy == 1 {
            e.l(0x4386c0)?
        } else {
            0
        },
        backing_port: if tile >= 23000 || ((0..23000).contains(&tile) && copy == 2) {
            e.l(0x4386bc)?
        } else {
            0
        },
        tile_port: if (0..23000).contains(&tile) {
            e.l(0x4386c4)?
        } else {
            0
        },
        depth,
        old_world: 0,
        old_device: 0,
        buildings,
    };
    let child = recovered_drawing::fastplot(&mut context, tile, destination, 0, copy, s)?;
    for effect in child.effects {
        if let DrawingEffect::Write(write) = &effect {
            e.write(write.address, &write.bytes)?;
        } else {
            e.plan.effects.push(CombatCenterEffect::Drawing(effect));
        }
    }
    Ok(())
}

fn bodyground(
    e: &mut Execution,
    body: i16,
    copy: i16,
    s: &mut impl CombatCenterServices,
) -> Result<(), RuleError> {
    let drawing_enabled = e.w(0x438776)?;
    if drawing_enabled == 0 {
        return Ok(());
    }
    let port = e.l(0x43879c)?;
    e.port(port, s)?;
    let state = s.bodyground_state(&e.plan.memory)?;
    if body <= 9 {
        let n = usize::try_from(body).map_err(|_| {
            RuleError::Invalid("bodyground captured party index unavailable".into())
        })?;
        let position = (e.w(idx(0x44709a, body, 4))?, e.w(idx(0x44709c, body, 4))?);
        let under = e.w(idx(0x437a6e, body, 2))?;
        if state.combat.squad.positions.get(n).copied() != Some(position)
            || state.combat.squad.under_tiles.get(n).copied() != Some(under)
        {
            return Err(RuleError::Invalid(
                "bodyground party view disagrees with native input bytes".into(),
            ));
        }
    } else {
        let n = body.wrapping_sub(10);
        let slot = usize::from(n as u16);
        let position = (e.w(idx(0x446f0a, n, 4))?, e.w(idx(0x446f0c, n, 4))?);
        let bytes = (0..288)
            .map(|i| e.b(rec(0x439d3c, n, 288).wrapping_add(i)))
            .collect::<Result<Vec<_>, _>>()?;
        if state.combat.enemies.positions.get(slot).copied() != Some(position)
            || state.combat.enemies.records.get(slot).map(|r| &r.bytes) != Some(&bytes)
        {
            return Err(RuleError::Invalid(
                "bodyground enemy view disagrees with native input bytes".into(),
            ));
        }
    }
    let child = recovered_bodies::bodyground(
        &state,
        BodyGroundInput {
            body,
            copy_word: copy,
            drawing_enabled,
            port,
        },
    )?;
    for effect in child.into_iter().skip(1) {
        if let BodyEffect::FastPlot {
            tile,
            rect,
            copy_word,
            ..
        } = effect
        {
            fastplot(e, tile, rect, copy_word as i8, s)?;
        } else {
            return Err(RuleError::Invalid(
                "unexpected bodyground projection effect".into(),
            ));
        }
    }
    Ok(())
}

/// Compose the existing bodyground and fastplot children with native input checks.
pub fn restore_body_ground(
    memory: &RenderingMemory,
    body: i16,
    copy: i16,
    limit: u32,
    services: &mut impl CombatCenterServices,
) -> Result<CombatCenterPlan, RuleError> {
    let mut e = Execution::new(memory, limit, None);
    bodyground(&mut e, body, copy, services)?;
    Ok(e.plan)
}

struct Positions<'a>(&'a RenderingMemory);
impl SpatialServices for Positions<'_> {
    fn warrior_position(&self, n: i16) -> Option<Position> {
        Some((
            self.0.word(idx(0x44709a, n, 4)).ok()? as i16,
            self.0.word(idx(0x44709c, n, 4)).ok()? as i16,
        ))
    }
    fn enemy_position(&self, n: i16) -> Option<Position> {
        Some((
            self.0.word(idx(0x446f0a, n, 4)).ok()? as i16,
            self.0.word(idx(0x446f0c, n, 4)).ok()? as i16,
        ))
    }
    fn warrior_byte(&self, n: i16, o: usize) -> Option<u8> {
        self.0
            .byte(rec(0x448aa0, n, 3138).wrapping_add(o as u32))
            .ok()
    }
    fn warrior_word(&self, n: i16, o: usize) -> Option<i16> {
        self.0
            .word(rec(0x448aa0, n, 3138).wrapping_add(o as u32))
            .ok()
            .map(|v| v as i16)
    }
    fn enemy_byte(&self, n: i16, o: usize) -> Option<u8> {
        self.0
            .byte(rec(0x439d3c, n, 288).wrapping_add(o as u32))
            .ok()
    }
    fn enemy_word(&self, n: i16, o: usize) -> Option<i16> {
        self.0
            .word(rec(0x439d3c, n, 288).wrapping_add(o as u32))
            .ok()
            .map(|v| v as i16)
    }
    fn map_word(&self, x: i32, y: i32) -> Option<i16> {
        self.0
            .word(
                0x4085b8u32
                    .wrapping_add(x.wrapping_mul(180).wrapping_add(y.wrapping_mul(2)) as u32),
            )
            .ok()
            .map(|v| v as i16)
    }
}
fn range(e: &mut Execution, body: i16, target: Position) -> Result<i16, RuleError> {
    let work = recovered_spatial::getrangetobody(body, target, &Positions(&e.plan.memory))
        .map_err(|err| RuleError::Invalid(err.to_string()))?;
    let value = work.range_word;
    e.plan
        .effects
        .push(CombatCenterEffect::Range { body, target, work });
    e.ww(0x4434b6, value)?;
    e.w(0x4434b6)
}

fn copy(
    e: &mut Execution,
    source_global: u32,
    destination_global: u32,
    sr: u32,
    dr: u32,
    s: &impl CombatCenterServices,
) -> Result<(), RuleError> {
    e.trap(
        0xa8ec,
        vec![
            P(e.l(source_global)?.wrapping_add(2)),
            P(e.l(destination_global)?.wrapping_add(2)),
            P(sr),
            P(dr),
            W(0),
            L(0),
        ],
        s,
    )?;
    Ok(())
}

fn field(
    memory: &RenderingMemory,
    frame: u32,
    large: bool,
    limit: u32,
    s: &mut impl CombatCenterServices,
) -> Result<CombatCenterPlan, RuleError> {
    let mut e = Execution::new(memory, limit, Some(0));
    if large {
        e.ww(frame.wrapping_sub(112), 1)?;
    }
    e.trap(0xa91f, vec![L(e.l(0x43879c)?)], s)?;
    e.port(e.l(0x43879c)?, s)?;
    e.ww(0x4471b6, 0)?;
    for i in 0..110 {
        e.write(frame.wrapping_sub(110).wrapping_add(i), &[0])?;
    }
    let tx = frame.wrapping_sub(if large { 113 } else { 111 });
    let ty = frame.wrapping_sub(if large { 114 } else { 112 });
    let x = e
        .l(0x442780)?
        .wrapping_add(i32::from(e.w(frame.wrapping_add(8))?) as u32)
        .wrapping_sub(i32::from(e.w(0x408570)?) as u32) as u8;
    e.write(tx, &[x])?;
    let y = e
        .l(0x442784)?
        .wrapping_add(i32::from(e.w(frame.wrapping_add(10))?) as u32)
        .wrapping_sub(i32::from(e.w(0x408572)?) as u32) as u8;
    e.write(ty, &[y])?;
    if (e.b(tx)? as i8) < 0 {
        e.addw(frame.wrapping_add(8), -i16::from(e.b(tx)? as i8))?;
    }
    if (e.b(ty)? as i8) < 0 {
        e.addw(frame.wrapping_add(10), -i16::from(e.b(ty)? as i8))?;
    }
    let bound = if large { 75 } else { 79 };
    if i16::from(e.b(tx)? as i8) > bound {
        e.addw(frame.wrapping_add(8), bound - i16::from(e.b(tx)? as i8))?;
    }
    if i16::from(e.b(ty)? as i8) > bound {
        e.addw(frame.wrapping_add(10), bound - i16::from(e.b(ty)? as i8))?;
    }
    let dx = e.w(frame.wrapping_add(8))?.wrapping_sub(e.w(0x408570)?);
    e.lw(0x442780, e.l(0x442780)?.wrapping_add(i32::from(dx) as u32))?;
    let dy = e.w(frame.wrapping_add(10))?.wrapping_sub(e.w(0x408572)?);
    e.lw(0x442784, e.l(0x442784)?.wrapping_add(i32::from(dy) as u32))?;
    let mut n = 0i16;
    while n <= i16::from(e.b(0x44273a)? as i8) {
        e.step()?;
        e.addw(
            idx(0x44709a, n, 4),
            e.w(0x408570)?.wrapping_sub(e.w(frame.wrapping_add(8))?),
        )?;
        e.addw(
            idx(0x44709c, n, 4),
            e.w(0x408572)?.wrapping_sub(e.w(frame.wrapping_add(10))?),
        )?;
        n = n.wrapping_add(1);
    }
    for n in 0..100 {
        e.step()?;
        e.addw(
            idx(0x446f0a, n, 4),
            e.w(0x408570)?.wrapping_sub(e.w(frame.wrapping_add(8))?),
        )?;
        e.addw(
            idx(0x446f0c, n, 4),
            e.w(0x408572)?.wrapping_sub(e.w(frame.wrapping_add(10))?),
        )?;
    }
    e.port(e.l(0x43879c)?, s)?;
    e.ww(0x4471f2, -32)?;
    e.ww(0x4471f6, 0)?;
    e.trap(0xa862, vec![L(33)], s)?;
    e.trap(0xa863, vec![L(30)], s)?;
    let span = if large { 16 } else { 11 };
    let mut y = e.w(0x442786)?;
    while i32::from(y) < (e.l(0x442784)? as i32).wrapping_add(span) {
        e.step()?;
        e.addw(0x4471f2, 32)?;
        e.addw(0x4471f6, 32)?;
        e.ww(0x4471f4, -32)?;
        e.ww(0x4471f8, 0)?;
        let mut x = e.w(0x442782)?;
        while i32::from(x) < (e.l(0x442780)? as i32).wrapping_add(span) {
            e.step()?;
            e.addw(0x4471f4, 32)?;
            e.addw(0x4471f8, 32)?;
            let tile = e.w(rec(0x4085b8, x, 180).wrapping_add((i32::from(y) * 2) as u32))?;
            e.ww(0x4471f0, x)?;
            e.ww(0x4471ee, y)?;
            if tile > 999 {
                let tile = tile.wrapping_sub(1000);
                let row = tile.wrapping_sub(1) / 20;
                let column = tile.wrapping_sub(row.wrapping_mul(20)).wrapping_sub(1);
                e.ww(0x4434ba, row.wrapping_shl(5))?;
                e.ww(0x4434bc, column.wrapping_shl(5))?;
                e.ww(0x4434c0, e.w(0x4434bc)?.wrapping_add(32))?;
                e.ww(0x4434be, e.w(0x4434ba)?.wrapping_add(32))?;
                copy(&mut e, 0x4386c4, 0x4386c0, 0x4434ba, 0x4471f2, s)?;
            } else if tile > -1 {
                bodyground(&mut e, tile, 1, s)?;
                e.write(frame.wrapping_sub(110).wrapping_add(tile as u32), &[1])?;
            }
            x = x.wrapping_add(1);
        }
        y = y.wrapping_add(1);
    }
    let base = frame.wrapping_sub(if large { 134 } else { 132 });
    let child = recovered_rendering::showque(&e.plan.memory, base.wrapping_sub(8), s)?;
    e.render(child);
    e.ww(0x4434bc, -32)?;
    e.ww(0x4434ba, -32)?;
    e.ww(0x4434be, 0)?;
    e.ww(0x4434c0, 0)?;
    if e.b(0x43565f)? != 0 {
        for body in 0..110i16 {
            e.step()?;
            if e.b(frame.wrapping_sub(110).wrapping_add(body as u32))? == 0 {
                continue;
            }
            if body == e.w(0x4435d2)? {
                e.drawbody(base.wrapping_sub(14), body, 1, 1, s)?;
            }
            if body <= 9 {
                continue;
            }
            let index = body.wrapping_sub(10);
            if large && e.w(rec(0x439d4c, index, 288))? != 0 {
                e.addw(frame.wrapping_sub(112), 1)?;
            }
            let target = (e.w(idx(0x446f0a, index, 4))?, e.w(idx(0x446f0c, index, 4))?);
            let actor = e.w(0x4435d2)?;
            let distance = range(&mut e, actor, target)?;
            if (large && distance > e.w(frame.wrapping_sub(112))?) || (!large && distance != 1) {
                continue;
            }
            if e.w(rec(0x439d4a, index, 288))? == 0 || e.b(rec(0x439dec, index, 288))? == 0 {
                continue;
            }
            if e.b(rec(0x439dec, index, 288))? == 255 {
                let actor = e.w(0x4435d2)?;
                let sum = e
                    .w(rec(0x449604, actor, 3138))?
                    .wrapping_add(e.w(rec(0x448b96, actor, 3138))?);
                if sum == 0 {
                    continue;
                }
            }
            e.drawbody(base.wrapping_sub(14), body, 1, 1, s)?;
        }
    } else {
        for body in 0..110i16 {
            e.step()?;
            if e.b(frame.wrapping_sub(110).wrapping_add(body as u32))? != 0 {
                let force = e
                    .w(0x438760)?
                    .wrapping_sub(i16::from(e.b(0x435661)?))
                    .wrapping_sub(i16::from(e.b(0x435882)?));
                e.drawbody(base.wrapping_sub(14), body, force, 1, s)?;
            }
        }
    }
    copy(&mut e, 0x4386c0, 0x43879c, 0x4471b4, 0x4471b4, s)?;
    let current = e.w(idx(0x4470b8, e.w(0x447194)?, 2))?;
    if current < 9 && e.w(idx(0x4470b8, e.w(0x447194)?, 2))? > -1 {
        copy(&mut e, 0x4386c0, 0x4386b4, 0x4471b4, 0x4471b4, s)?;
        let child_frame = base.wrapping_sub(10);
        e.ww(child_frame.wrapping_add(8), 1)?;
        let child = showrange(&e.plan.memory, child_frame, e.remaining, s)?;
        e.merge(child);
        e.trap(0xa8a9, vec![P(0x4471b4), W(10), W(10)], s)?;
        copy(&mut e, 0x4386c0, 0x4386b4, 0x4471b4, 0x4471b4, s)?;
        e.trap(
            0xa8a9,
            vec![P(0x4471b4), W((-10i16) as u16), W((-10i16) as u16)],
            s,
        )?;
        copy(&mut e, 0x4386b4, 0x43879c, 0x4471b4, 0x4471b4, s)?;
    } else {
        copy(&mut e, 0x4386c0, 0x43879c, 0x4471b4, 0x4471b4, s)?;
    }
    if e.w(0x438760)? != 0 {
        let child = recovered_combat_ui::showtargets(&e.plan.memory, s)?;
        e.plan.memory = child.memory;
        e.plan
            .effects
            .extend(child.effects.into_iter().map(CombatCenterEffect::Combat));
    }
    e.port(e.l(0x43878c)?, s)?;
    e.plan.native_return = Some(0);
    Ok(e.plan)
}

pub fn centerfieldlarge(
    memory: &RenderingMemory,
    frame: u32,
    limit: u32,
    s: &mut impl CombatCenterServices,
) -> Result<CombatCenterPlan, RuleError> {
    field(memory, frame, true, limit, s)
}
pub fn centerfieldsmall(
    memory: &RenderingMemory,
    frame: u32,
    limit: u32,
    s: &mut impl CombatCenterServices,
) -> Result<CombatCenterPlan, RuleError> {
    field(memory, frame, false, limit, s)
}

pub fn centerfield(
    memory: &RenderingMemory,
    frame: u32,
    limit: u32,
    s: &mut impl CombatCenterServices,
) -> Result<CombatCenterPlan, RuleError> {
    let mut e = Execution::new(memory, limit, None);
    if e.w(0x438776)? != 0 {
        let large = e.w(0x439582)? != 0;
        let y = e.w(frame.wrapping_add(10))?;
        e.ww(frame.wrapping_sub(2), y)?;
        let x = e.w(frame.wrapping_add(8))?;
        e.ww(frame.wrapping_sub(4), x)?;
        let child = field(
            &e.plan.memory,
            frame.wrapping_sub(12),
            large,
            e.remaining,
            s,
        )?;
        e.merge(child);
    }
    Ok(e.plan)
}

pub fn centerpict(
    state: &WorldDisplayState,
    frame: u32,
    limit: u32,
    s: &mut impl CombatCenterServices,
) -> Result<CombatCenterPlan, RuleError> {
    if (state.memory.byte(0x44273a)? as i8) < 0 || state.memory.word(0x438776)? != 0 {
        return Ok(Execution::new(&state.memory, limit, None).plan);
    }
    let child = if state.memory.word(0x439582)? != 0 {
        recovered_world_display::centerpictlarge(state, frame.wrapping_sub(8), limit, s)?
    } else {
        recovered_world_display::centerpictsmall(state, frame.wrapping_sub(8), limit, s)?
    };
    if child.state.random != state.random {
        return Err(RuleError::Invalid(
            "viewport centering unexpectedly consumed random state".into(),
        ));
    }
    Ok(CombatCenterPlan {
        memory: child.state.memory,
        effects: child
            .effects
            .into_iter()
            .map(CombatCenterEffect::World)
            .collect(),
        native_return: Some(child.native_return as u32),
    })
}

pub fn center(
    memory: &RenderingMemory,
    frame: u32,
    limit: u32,
    s: &mut impl CombatCenterServices,
) -> Result<CombatCenterPlan, RuleError> {
    let mut e = Execution::new(memory, limit, Some(0));
    let body = e.b(frame.wrapping_add(9))? as i8;
    let (base, n) = if body > 9 {
        (0x446f0a, body.wrapping_sub(10))
    } else {
        (0x44709a, body)
    };
    let address = idx(base, i16::from(n), 4);
    let x = e.w(address)?;
    let first = recovered_navigation::twixt(x, 1, 8);
    e.plan.effects.push(CombatCenterEffect::Between {
        value: x,
        result: first,
    });
    if first != 0 {
        let y = e.w(address.wrapping_add(2))?;
        let second = recovered_navigation::twixt(y, 1, 8);
        e.plan.effects.push(CombatCenterEffect::Between {
            value: y,
            result: second,
        });
        if second != 0 {
            return Ok(e.plan);
        }
    }
    let y = e.w(address.wrapping_add(2))?;
    e.ww(frame.wrapping_sub(6), y)?;
    let x = e.w(address)?;
    e.ww(frame.wrapping_sub(8), x)?;
    let child = centerfield(&e.plan.memory, frame.wrapping_sub(16), e.remaining, s)?;
    e.merge(child);
    e.plan.native_return = Some(0);
    Ok(e.plan)
}

struct WindowBridge<'a, S: CombatCenterServices> {
    e: &'a mut Execution,
    s: &'a S,
    frame: u32,
    position_read: usize,
}
fn unused_interface() -> RuleError {
    RuleError::Invalid("unused centering interface observation".into())
}
impl<S: CombatCenterServices> InterfaceServices for WindowBridge<'_, S> {
    fn word(&mut self, a: u32) -> Result<i16, RuleError> {
        let value = self.e.w(a)?;
        let local = match self.position_read {
            0 => self.frame.wrapping_sub(10),
            1 => self.frame.wrapping_sub(12),
            _ => return Err(unused_interface()),
        };
        self.e.ww(local, value)?;
        self.position_read += 1;
        Ok(value)
    }
    fn point_in_rect(&mut self, _point: Point, _rect: BodyRect) -> Result<u8, RuleError> {
        let value = self.e.trap(
            0xa8ad,
            vec![
                L(self.e.l(self.frame.wrapping_sub(12))?),
                P(self.frame.wrapping_sub(8)),
            ],
            self.s,
        )?;
        Ok(value as u8)
    }
    fn current_port(&mut self) -> Result<u32, RuleError> {
        Err(unused_interface())
    }
    fn pen_state(&mut self) -> Result<[u8; 18], RuleError> {
        Err(unused_interface())
    }
    fn local_to_global(&mut self, _: Point) -> Result<Point, RuleError> {
        Err(unused_interface())
    }
    fn new_region(&mut self) -> Result<u32, RuleError> {
        Err(unused_interface())
    }
    fn main_device_rect(&mut self) -> Result<BodyRect, RuleError> {
        Err(unused_interface())
    }
}

pub fn centerstage2(
    memory: &RenderingMemory,
    frame: u32,
    limit: u32,
    s: &mut impl CombatCenterServices,
) -> Result<CombatCenterPlan, RuleError> {
    let mut e = Execution::new(memory, limit, Some(0));
    let body = e.w(frame.wrapping_add(8))?;
    let f = frame.wrapping_sub(14);
    e.ww(f.wrapping_add(8), body)?;
    e.ww(f.wrapping_sub(6), 1)?;
    e.ww(f.wrapping_sub(8), 1)?;
    let height = e.w(0x439582)?;
    e.ww(f.wrapping_sub(4), height.wrapping_shl(2).wrapping_add(8))?;
    e.ww(
        f.wrapping_sub(2),
        e.w(0x439582)?.wrapping_mul(5).wrapping_add(8),
    )?;
    if e.w(0x439582)? != height {
        return Err(RuleError::Unsupported {
            action: "combat_center.inwindow",
            dependency: "height aliases the captured local rectangle view",
        });
    }
    let child = recovered_interface::inwindow(
        body as i8,
        height,
        &mut WindowBridge {
            e: &mut e,
            s,
            frame: f,
            position_read: 0,
        },
    )?;
    let inside = child.native_return != 0;
    e.plan
        .effects
        .extend(child.effects.into_iter().map(CombatCenterEffect::Interface));
    if !inside || e.w(frame.wrapping_add(10))? != 0 {
        let address = if body < 9 {
            idx(0x44709a, body, 4)
        } else {
            idx(0x446f0a, body.wrapping_sub(10), 4)
        };
        let y = e.w(address.wrapping_add(2))?;
        e.ww(frame.wrapping_sub(6), y)?;
        let x = e.w(address)?;
        e.ww(frame.wrapping_sub(8), x)?;
        let child = centerfield(&e.plan.memory, frame.wrapping_sub(16), e.remaining, s)?;
        e.merge(child);
    }
    e.plan.native_return = Some(0);
    Ok(e.plan)
}

pub fn centerstage<S: CombatCenterServices + CombatPanelServices>(
    memory: &RenderingMemory,
    frame: u32,
    odds_damage_frame: DamageFrame,
    limit: u32,
    s: &mut S,
) -> Result<CombatCenterPlan, RuleError> {
    let mut e = Execution::new(memory, limit, Some(0));
    let direction = e.w(frame.wrapping_add(8))?;
    loop {
        e.step()?;
        e.addw(0x4435c2, direction)?;
        if e.w(0x4435c2)? > 109 {
            e.ww(0x4435c2, 1)?;
        }
        if e.w(0x4435c2)? < 1 {
            e.ww(0x4435c2, 109)?;
        }
        if e.w(idx(0x4470b8, e.w(0x4435c2)?, 2))? != -1 {
            break;
        }
    }
    let body = e.w(idx(0x4470b8, e.w(0x4435c2)?, 2))?;
    let address = if body < 9 && e.w(rec(0x448af0, body, 3138))? > 0 {
        Some(idx(0x44709a, body, 4))
    } else if e.w(rec(0x439e4c, body.wrapping_sub(10), 288))? > 0 {
        Some(idx(0x446f0a, body.wrapping_sub(10), 4))
    } else {
        None
    };
    if let Some(address) = address {
        let middle = e.w(0x439582)?.wrapping_mul(2).wrapping_add(5);
        let y = e.w(address.wrapping_add(2))?.wrapping_sub(
            e.w(0x4471a0)?
                .wrapping_sub(middle)
                .wrapping_mul(e.w(0x438760)?),
        );
        e.ww(frame.wrapping_sub(10), y)?;
        let middle = e.w(0x439582)?.wrapping_mul(2).wrapping_add(5);
        let x = e.w(address)?.wrapping_sub(
            e.w(0x44719e)?
                .wrapping_sub(middle)
                .wrapping_mul(e.w(0x438760)?),
        );
        e.ww(frame.wrapping_sub(12), x)?;
        let child = centerfield(&e.plan.memory, frame.wrapping_sub(20), e.remaining, s)?;
        e.merge(child);
    }
    if direction != 0 {
        e.sound(147, s)?;
    }
    e.drawbody(frame.wrapping_sub(22), body, 1, 0, s)?;
    e.ww(0x438782, 1)?;
    e.ww(frame.wrapping_sub(16), body)?;
    let child = recovered_combat_panel::combat_update2(
        &e.plan.memory,
        frame.wrapping_sub(24),
        body,
        odds_damage_frame,
        s,
    )?;
    e.plan.memory = child.memory;
    e.plan
        .effects
        .extend(child.effects.into_iter().map(CombatCenterEffect::Panel));
    e.ww(0x438782, 0)?;
    e.plan.native_return = Some(0);
    Ok(e.plan)
}

pub fn exitcapturemode(
    memory: &RenderingMemory,
    frame: u32,
    limit: u32,
    s: &mut impl CombatCenterServices,
) -> Result<CombatCenterPlan, RuleError> {
    let mut e = Execution::new(memory, limit, Some(0));
    if e.b(0x43565f)? != 0 {
        e.write(0x43565f, &[0])?;
        e.trap(0xaa1c, vec![L(e.l(0x43952c)?)], s)?;
        let f = frame.wrapping_sub(20);
        e.lw(f.wrapping_add(16), 0xffff0000)?;
        e.lw(f.wrapping_add(12), 0x001e012c)?;
        e.lw(f.wrapping_add(8), 0x40250d)?;
        let child = recovered_feedback::flashmessage(
            &e.plan.memory,
            f,
            0x40250d,
            30,
            300,
            -1,
            0,
            e.remaining as usize,
            s,
        )?;
        e.plan.memory = child.memory;
        e.plan
            .effects
            .extend(child.effects.into_iter().map(CombatCenterEffect::Feedback));
        let y = e.w(0x408572)?;
        e.ww(frame.wrapping_sub(14), y)?;
        let x = e.w(0x408570)?;
        e.ww(frame.wrapping_sub(16), x)?;
        let child = centerfield(&e.plan.memory, frame.wrapping_sub(24), e.remaining, s)?;
        e.merge(child);
    }
    e.plan.native_return = Some(0);
    Ok(e.plan)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recovered_audio::{AudioContext, AudioObservation};
    use crate::recovered_combat_ui::{CombatUiObservation, CombatUiRequest};
    use crate::recovered_drawing::{DrawingServices, PictureObservation};
    use crate::recovered_feedback::FeedbackObservation;
    use crate::recovered_rendering::{RenderingObservation, RenderingRequest};
    use crate::recovered_text::NumericServices;
    use crate::recovered_world_display::{WorldDisplayObservation, WorldDisplayRequest};
    use std::cell::RefCell;

    struct Observed {
        inside: u8,
        move_on_world: bool,
        calls: RefCell<Vec<FeedbackRequest>>,
    }
    impl Observed {
        fn new(inside: u8) -> Self {
            Self {
                inside,
                move_on_world: false,
                calls: RefCell::new(vec![]),
            }
        }
    }
    impl FeedbackServices for Observed {
        fn observe(
            &self,
            request: &FeedbackRequest,
            memory: &RenderingMemory,
        ) -> Result<FeedbackObservation, RuleError> {
            self.calls.borrow_mut().push(request.clone());
            let mut after = memory.clone();
            let mut value = 0;
            if let FeedbackRequest::Toolbox { opcode, arguments } = request {
                match *opcode {
                    0xab1d if matches!(arguments.first(), Some(L(0x80005))) => {
                        after.insert_region(0x4046da, &0xdeadbeefu32.to_be_bytes());
                        after.insert_region(0x4046de, &0x12345678u32.to_be_bytes());
                        if self.move_on_world {
                            after.insert_region(0x44709a, &5i16.to_be_bytes());
                            after.insert_region(0x44709c, &6i16.to_be_bytes());
                        }
                    }
                    0xa8a7 => {
                        if let [P(p), W(left), W(top), W(right), W(bottom)] = arguments.as_slice() {
                            for (offset, v) in [(0, top), (2, left), (4, bottom), (6, right)] {
                                after.insert_region(p + offset, &v.to_be_bytes());
                            }
                        }
                    }
                    0xa8ad => value = u32::from(self.inside),
                    0xa860 => after.insert_region(0x44720a, &3i16.to_be_bytes()),
                    _ => {}
                }
            }
            Ok(FeedbackObservation {
                value,
                d0: 0,
                memory: after,
            })
        }
        fn sound_observations(
            &self,
            _: i16,
            _: &AudioContext,
            _: &RenderingMemory,
        ) -> Result<Vec<AudioObservation>, RuleError> {
            Err(RuleError::Invalid("unexpected test audio lookup".into()))
        }
    }
    impl RenderingServices for Observed {
        fn observe(
            &self,
            _: &RenderingRequest,
            _: &RenderingMemory,
        ) -> Result<RenderingObservation, RuleError> {
            Err(RuleError::Invalid("unexpected test render lookup".into()))
        }
    }
    impl CombatUiServices for Observed {
        fn observe(
            &self,
            _: &CombatUiRequest,
            _: &RenderingMemory,
        ) -> Result<CombatUiObservation, RuleError> {
            Err(RuleError::Invalid("unexpected test combat lookup".into()))
        }
    }
    impl DrawingServices for Observed {
        fn color_icon(&mut self, _: i16) -> Result<u32, RuleError> {
            Err(unused_interface())
        }
        fn indexed_string(&mut self, _: i16, _: i16) -> Result<PascalString, RuleError> {
            Err(unused_interface())
        }
        fn current_world(&mut self) -> Result<(u32, u32), RuleError> {
            Err(unused_interface())
        }
        fn picture(&mut self, _: i16) -> Result<PictureObservation, RuleError> {
            Err(unused_interface())
        }
        fn new_world(&mut self, _: i16, _: BodyRect) -> Result<(i16, BuildingWorld), RuleError> {
            Err(unused_interface())
        }
    }
    impl NumericServices for Observed {
        fn number_to_string(&mut self, _: i32) -> Result<PascalString, RuleError> {
            Err(unused_interface())
        }
        fn string_to_number(&mut self, _: &PascalString) -> Result<i32, RuleError> {
            Err(unused_interface())
        }
    }
    impl WorldDisplayServices for Observed {
        fn observe(
            &mut self,
            _: &WorldDisplayRequest,
            _: &RenderingMemory,
        ) -> Result<WorldDisplayObservation, RuleError> {
            Err(unused_interface())
        }
    }
    impl CombatCenterServices for Observed {}

    fn memory() -> RenderingMemory {
        let mut m = RenderingMemory::default();
        for (a, n) in [
            (0x8000, 4096),
            (0x4046da, 8),
            (0x408500, 17000),
            (0x435600, 1024),
            (0x438600, 512),
            (0x442730, 16),
            (0x442780, 8),
            (0x442bd6, 1200),
            (0x4434a0, 400),
            (0x446f00, 1024),
            (0x448aa0, 3138),
        ] {
            m.insert_region(a, &vec![0; n]);
        }
        m.insert_region(0x44273a, &[255]);
        m.insert_region(0x4470b8, &(-1i16).to_be_bytes());
        m
    }
    fn trap_calls(s: &Observed, opcode: u16) -> Vec<Vec<FeedbackArg>> {
        s.calls
            .borrow()
            .iter()
            .filter_map(|r| match r {
                FeedbackRequest::Toolbox {
                    opcode: o,
                    arguments,
                } if *o == opcode => Some(arguments.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn range_uses_observed_point_reply_and_fresh_positions() {
        let mut m = memory();
        m.insert_region(0x8808, &1i16.to_be_bytes());
        m.insert_region(0x44273a, &[0]);
        m.insert_region(0x44947f, &[1]);
        m.insert_region(0x448ab0, &1i16.to_be_bytes());
        m.insert_region(0x448b00, &1i16.to_be_bytes());
        m.insert_region(0x44709a, &3i16.to_be_bytes());
        m.insert_region(0x44709c, &4i16.to_be_bytes());
        for inside in [0, 255] {
            let mut s = Observed::new(inside);
            s.move_on_world = true;
            let result = showrange(&m, 0x8800, 100, &s).unwrap();
            let origin = if inside == 0 {
                vec![W(160), W(160)]
            } else {
                vec![W(176), W(208)]
            };
            assert_eq!(trap_calls(&s, 0xa893), vec![origin]);
            assert_eq!(trap_calls(&s, 0xa891), vec![vec![W(176), W(208)]]);
            assert_eq!(
                trap_calls(&s, 0xab1d).last().unwrap(),
                &vec![L(0x80006), L(0xdeadbeef), L(0x12345678)]
            );
            assert!(trap_calls(&s, 0xa862).contains(&vec![L(409)]));
            assert_eq!(result.native_return, Some(0));
            assert_eq!(m.word(0x44709a).unwrap(), 3);
        }
    }

    #[test]
    fn both_field_spans_shift_positions_and_draw_every_terrain_cell() {
        let mut m = memory();
        m.insert_region(0x438776, &1i16.to_be_bytes());
        for i in 0..8100u32 {
            m.insert_region(0x4085b8 + i * 2, &1001i16.to_be_bytes());
        }
        m.insert_region(0x8808, &2i16.to_be_bytes());
        m.insert_region(0x880a, &3i16.to_be_bytes());
        m.insert_region(0x446f0a, &8i16.to_be_bytes());
        m.insert_region(0x446f0c, &9i16.to_be_bytes());
        for (large, count) in [(true, 258), (false, 123)] {
            let mut s = Observed::new(0);
            let result = field(&m, 0x8800, large, 2000, &mut s).unwrap();
            assert_eq!(result.memory.long(0x442780).unwrap(), 2);
            assert_eq!(result.memory.long(0x442784).unwrap(), 3);
            assert_eq!(result.memory.word(0x446f0a).unwrap(), 6);
            assert_eq!(result.memory.word(0x446f0c).unwrap(), 6);
            assert_eq!(trap_calls(&s, 0xa8ec).len(), count);
            assert_eq!(
                result.memory.word(0x4471f2).unwrap(),
                if large { 480 } else { 320 }
            );
            assert_eq!(result.native_return, Some(0));
        }
    }

    #[test]
    fn stage_two_body_nine_uses_negative_enemy_slot_and_native_local_point() {
        let mut m = memory();
        m.insert_region(0x8808, &9i16.to_be_bytes());
        m.insert_region(0x439582, &2i16.to_be_bytes());
        m.insert_region(0x446f06, &10i16.to_be_bytes());
        m.insert_region(0x446f08, &11i16.to_be_bytes());
        let mut s = Observed::new(0);
        let result = centerstage2(&m, 0x8800, 100, &mut s).unwrap();
        assert_eq!(trap_calls(&s, 0xa8ad), vec![vec![L(0x000b000a), P(0x87ea)]]);
        assert_eq!(result.memory.word(0x87e6).unwrap(), 11);
        assert_eq!(result.memory.word(0x87e8).unwrap(), 10);
        assert_eq!(result.native_return, Some(0));
    }

    #[test]
    fn missing_reached_map_bytes_leave_input_unchanged() {
        let mut m = memory();
        m.insert_region(0x438776, &1i16.to_be_bytes());
        m.bytes.remove(&0x4085b8);
        let before = m.clone();
        let mut s = Observed::new(0);
        assert!(centerfieldsmall(&m, 0x8800, 2000, &mut s).is_err());
        assert_eq!(m, before);
    }

    #[test]
    fn window_position_write_precedes_aliased_second_read() {
        let mut m = memory();
        let frame = 0x4470b4;
        m.insert_region(frame + 8, &0i16.to_be_bytes());
        m.insert_region(0x439582, &2i16.to_be_bytes());
        m.insert_region(0x44709a, &10i16.to_be_bytes());
        m.insert_region(0x44709c, &11i16.to_be_bytes());
        let mut s = Observed::new(255);
        let result = centerstage2(&m, frame, 100, &mut s).unwrap();
        assert_eq!(
            trap_calls(&s, 0xa8ad),
            vec![vec![L(0x000a000a), P(0x44709e)]]
        );
        assert_eq!(result.memory.word(0x44709a).unwrap(), 10);
        assert_eq!(result.memory.word(0x44709c).unwrap(), 10);
        assert_eq!(m.word(0x44709c).unwrap(), 11);
    }

    #[test]
    fn capture_range_keeps_the_native_result_word_write() {
        let mut m = memory();
        m.insert_region(0x44709a, &3i16.to_be_bytes());
        m.insert_region(0x44709c, &4i16.to_be_bytes());
        m.insert_region(0x4434b6, &1234i16.to_be_bytes());
        let mut e = Execution::new(&m, 100, Some(0));
        assert_eq!(range(&mut e, 0, (6, 8)).unwrap(), 5);
        assert_eq!(e.plan.memory.word(0x4434b6).unwrap(), 5);
        assert!(
            matches!(e.plan.effects.last(), Some(CombatCenterEffect::Write(write)) if write.address == 0x4434b6 && write.bytes == [0,5])
        );
        assert_eq!(m.word(0x4434b6).unwrap(), 1234);
    }

    #[test]
    fn ground_tile_copy_uses_the_actual_secondary_port() {
        let mut m = memory();
        m.insert_region(0x43879c, &0x3000u32.to_be_bytes());
        m.insert_region(0x4386c0, &0x4000u32.to_be_bytes());
        m.insert_region(0x4386c4, &0x6000u32.to_be_bytes());
        let mut e = Execution::new(&m, 100, Some(0));
        let mut s = Observed::new(0);
        let destination = BodyRect {
            top: 1,
            left: 2,
            bottom: 33,
            right: 34,
        };
        fastplot(&mut e, 1001, destination, 1, &mut s).unwrap();
        assert!(e.plan.effects.iter().any(|effect| matches!(effect,
            CombatCenterEffect::Drawing(DrawingEffect::CopyBits { source_bitmap: 0x6002, destination_bitmap: 0x4002, destination: rect, mode: 0, .. }) if *rect == destination)));
        let mut empty = Execution::new(&RenderingMemory::default(), 100, Some(0));
        fastplot(&mut empty, -1, destination, 1, &mut s).unwrap();
        assert!(empty.plan.effects.is_empty());
        assert!(empty.plan.memory.bytes.is_empty());
    }
}
