//! World viewport drawing and combat-map generation from complete CODE 1 bodies.

use crate::recovered_drawing::{BuildingWorld, DrawingContext, DrawingEffect, DrawingServices};
use crate::recovered_geometry::{BodyRect, VisibilityInput, VisibilityServices};
use crate::recovered_install::InstallMemory;
use crate::recovered_navigation::CoordinateEffect;
use crate::recovered_records::RecordWrite;
use crate::recovered_rendering::RenderingMemory;
use crate::recovered_text::{NumericServices, PascalString};
use crate::{
    RandomState, RuleError, recovered_ai, recovered_drawing, recovered_geometry,
    recovered_navigation,
};
use centurions_content::ITEM_BYTES;
use std::cell::RefCell;
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldDisplayState {
    pub memory: RenderingMemory,
    pub random: RandomState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorldDisplayRequest {
    SetRect {
        pointer: u32,
        left: i16,
        top: i16,
        right: i16,
        bottom: i16,
    },
    FrontWindow,
    SelectWindow(u32),
    Drawing(DrawingEffect),
    GetWorld {
        world: u32,
        device: u32,
    },
    PlotColorIcon {
        rectangle: u32,
        handle: u32,
    },
    CopyBits {
        source_bitmap: u32,
        destination_bitmap: u32,
        source_rectangle: u32,
        destination_rectangle: u32,
        mode: i16,
        mask: u32,
    },
    OffsetRect {
        pointer: u32,
        horizontal: i16,
        vertical: i16,
    },
    InsetRect {
        pointer: u32,
        horizontal: i16,
        vertical: i16,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldDisplayObservation {
    pub value: u32,
    pub memory: RenderingMemory,
}

/// Typed children retain their existing pure numeric/drawing observation contracts.
pub trait WorldDisplayServices: DrawingServices + NumericServices {
    fn observe(
        &mut self,
        request: &WorldDisplayRequest,
        memory: &RenderingMemory,
    ) -> Result<WorldDisplayObservation, RuleError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorldDisplayEffect {
    Write(RecordWrite),
    Service {
        request: WorldDisplayRequest,
        value: u32,
        /// Address-ordered boundary delta; no internal Toolbox write order is claimed.
        memory_changes: Vec<(u32, Option<u8>)>,
    },
    Drawing(DrawingEffect),
    Coordinate(CoordinateEffect),
    RandomBound {
        bound: i16,
        value: i16,
        before: RandomState,
        after: RandomState,
    },
    RandomRange {
        minimum: i16,
        maximum: i16,
        value: i16,
        before: RandomState,
        after: RandomState,
    },
    Between {
        value: i16,
        minimum: i16,
        maximum: i16,
        result: i32,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldDisplayPlan {
    pub state: WorldDisplayState,
    pub effects: Vec<WorldDisplayEffect>,
    pub native_return: i32,
}

struct Execution {
    plan: WorldDisplayPlan,
    remaining: u32,
}

impl Execution {
    fn new(state: &WorldDisplayState, maximum_steps: u32) -> Self {
        Self {
            plan: WorldDisplayPlan {
                state: state.clone(),
                effects: vec![],
                native_return: 0,
            },
            remaining: maximum_steps,
        }
    }
    fn step(&mut self) -> Result<(), RuleError> {
        if self.remaining == 0 {
            return Err(RuleError::Invalid(
                "world display processing budget exhausted".into(),
            ));
        }
        self.remaining -= 1;
        Ok(())
    }
    fn bytes(&self, address: u32, length: usize) -> Result<Vec<u8>, RuleError> {
        read(&self.plan.state.memory, address, length)
    }
    fn byte(&self, address: u32) -> Result<u8, RuleError> {
        self.plan.state.memory.byte(address)
    }
    fn signed_byte(&self, address: u32) -> Result<i16, RuleError> {
        Ok(i16::from(self.byte(address)? as i8))
    }
    fn word(&self, address: u32) -> Result<i16, RuleError> {
        Ok(self.plan.state.memory.word(address)? as i16)
    }
    fn long(&self, address: u32) -> Result<u32, RuleError> {
        self.plan.state.memory.long(address)
    }
    fn write(&mut self, address: u32, bytes: &[u8]) -> Result<(), RuleError> {
        self.bytes(address, bytes.len())?;
        self.plan.state.memory.insert_region(address, bytes);
        self.plan
            .effects
            .push(WorldDisplayEffect::Write(RecordWrite {
                address,
                bytes: bytes.to_vec(),
            }));
        Ok(())
    }
    fn ww(&mut self, address: u32, value: i16) -> Result<(), RuleError> {
        self.write(address, &value.to_be_bytes())
    }
    fn lw(&mut self, address: u32, value: u32) -> Result<(), RuleError> {
        self.write(address, &value.to_be_bytes())
    }
    fn add_word(&mut self, address: u32, amount: i16) -> Result<(), RuleError> {
        self.ww(address, self.word(address)?.wrapping_add(amount))
    }
    fn service(
        &mut self,
        request: WorldDisplayRequest,
        services: &mut (impl WorldDisplayServices + ?Sized),
    ) -> Result<u32, RuleError> {
        self.step()?;
        let observation = services.observe(&request, &self.plan.state.memory)?;
        let value = observation.value;
        let mut before = self.plan.state.memory.bytes.iter().peekable();
        let mut after = observation.memory.bytes.iter().peekable();
        let mut memory_changes = vec![];
        loop {
            match (before.peek(), after.peek()) {
                (Some((old_address, _)), Some((new_address, _))) if old_address < new_address => {
                    memory_changes.push((**old_address, None));
                    before.next();
                }
                (Some((old_address, _)), Some((new_address, new_value)))
                    if old_address > new_address =>
                {
                    memory_changes.push((**new_address, Some(**new_value)));
                    after.next();
                }
                (Some((_, old_value)), Some((new_address, new_value))) => {
                    if old_value != new_value {
                        memory_changes.push((**new_address, Some(**new_value)));
                    }
                    before.next();
                    after.next();
                }
                (Some(_), None) => {
                    memory_changes.extend(before.map(|(address, _)| (*address, None)));
                    break;
                }
                (None, Some(_)) => {
                    memory_changes.extend(after.map(|(address, value)| (*address, Some(*value))));
                    break;
                }
                (None, None) => break,
            }
        }
        self.plan.state.memory = observation.memory;
        self.plan.effects.push(WorldDisplayEffect::Service {
            request,
            value,
            memory_changes,
        });
        Ok(value)
    }
    fn drawing(
        &mut self,
        effect: DrawingEffect,
        services: &mut (impl WorldDisplayServices + ?Sized),
    ) -> Result<u32, RuleError> {
        self.service(WorldDisplayRequest::Drawing(effect), services)
    }
    fn bound(&mut self, bound: i16) -> Result<i16, RuleError> {
        self.step()?;
        let before = self.plan.state.random.clone();
        let value = self.plan.state.random.bound(bound);
        self.plan.effects.push(WorldDisplayEffect::RandomBound {
            bound,
            value,
            before,
            after: self.plan.state.random.clone(),
        });
        Ok(value)
    }
    fn range(&mut self, minimum: i16, maximum: i16) -> Result<i16, RuleError> {
        self.step()?;
        let before = self.plan.state.random.clone();
        let value = recovered_ai::random_range(minimum, maximum, &mut self.plan.state.random);
        self.plan.effects.push(WorldDisplayEffect::RandomRange {
            minimum,
            maximum,
            value,
            before,
            after: self.plan.state.random.clone(),
        });
        Ok(value)
    }
}

fn read(memory: &RenderingMemory, address: u32, length: usize) -> Result<Vec<u8>, RuleError> {
    (0..length)
        .map(|i| memory.byte(address.wrapping_add(i as u32)))
        .collect()
}
fn signed_index(address: u32, index: i16, stride: i32) -> u32 {
    address.wrapping_add((i32::from(index) * stride) as u32)
}
fn map_address(base: u32, x: i16, y: i16, stride: u32, cell: i32) -> u32 {
    base.wrapping_add(u32::from(x as u16) * stride)
        .wrapping_add((i32::from(y) * cell) as u32)
}

struct VisibilityMemory<'a> {
    memory: &'a RenderingMemory,
    reads: RefCell<BTreeSet<u32>>,
}
impl VisibilityServices for VisibilityMemory<'_> {
    fn map_word(&self, row: i32, column: i32) -> Option<i16> {
        let address = 0x4085b8_u32
            .wrapping_add(row.wrapping_mul(180).wrapping_add(column.wrapping_mul(2)) as u32);
        self.reads
            .borrow_mut()
            .extend([address, address.wrapping_add(1)]);
        self.memory.word(address).ok().map(|value| value as i16)
    }
    fn tile_visibility(&self, tile: i16) -> Option<i16> {
        let address = 0x43167c_u32
            .wrapping_add(u32::from(tile as u16) * 40)
            .wrapping_add(12);
        self.reads
            .borrow_mut()
            .extend([address, address.wrapping_add(1)]);
        self.memory.word(address).ok().map(|value| value as i16)
    }
}

fn visibility(e: &mut Execution, large: bool) -> Result<(), RuleError> {
    let input = VisibilityInput {
        center: (e.long(0x442764)? as i32, e.long(0x442768)? as i32),
        map_origin: (e.long(0x442778)? as i32, e.long(0x44277c)? as i32),
        ignore_walls: e.word(0x44256a)?,
    };
    let bridge = VisibilityMemory {
        memory: &e.plan.state.memory,
        reads: RefCell::new(BTreeSet::new()),
    };
    let child = if large {
        recovered_geometry::cansee3(input, &bridge)
    } else {
        recovered_geometry::cansee2(input, &bridge)
    }
    .map_err(|error| RuleError::Invalid(error.to_string()))?;
    if child.writes.iter().any(|effect| {
        bridge
            .reads
            .borrow()
            .contains(&(0x4358e6 + effect.index() as u32))
    }) {
        return Err(RuleError::Unsupported {
            action: "world_display.visibility",
            dependency: "reached native visibility writes alias child reads beyond the staged visibility adapter",
        });
    }
    for effect in child.writes {
        e.step()?;
        e.write(0x4358e6 + effect.index() as u32, &[1])?;
    }
    Ok(())
}

fn coordinate(
    e: &mut Execution,
    services: &mut (impl WorldDisplayServices + ?Sized),
) -> Result<(), RuleError> {
    const START: u32 = 0x40226a;
    const END: u32 = 0x479fd8;
    let memory = InstallMemory {
        base_address: START,
        bytes: e.bytes(START, (END - START) as usize)?,
    };
    let child = recovered_navigation::xy(&memory, 0, services)?;
    for effect in child.effects {
        if let CoordinateEffect::Write(write) = &effect {
            e.write(write.address, &write.bytes)?;
        } else {
            e.plan.effects.push(WorldDisplayEffect::Coordinate(effect));
        }
    }
    Ok(())
}

fn fastplot(
    e: &mut Execution,
    tile: i16,
    destination: BodyRect,
    services: &mut (impl WorldDisplayServices + ?Sized),
) -> Result<(), RuleError> {
    let rect = e.plan.state.memory.rect(0x4471f2)?;
    let mut buildings = vec![];
    let mut depth = 0;
    if tile >= 23000 {
        let index = usize::from(tile.wrapping_sub(23000) as u16);
        let pointer = e.long(0x41c220 + index as u32 * 4)?;
        let bounds = if pointer == 0 {
            depth = e.byte(0x43873f)? as i8;
            None
        } else {
            Some(e.plan.state.memory.rect(pointer.wrapping_add(0x10))?)
        };
        // The child accesses only the selected slot; earlier positions are unread.
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
        map_port: if tile >= 0 { e.long(0x43879c)? } else { 0 },
        text_port: 0,
        secondary_port: 0,
        backing_port: if tile >= 0 { e.long(0x4386bc)? } else { 0 },
        tile_port: if (0..23000).contains(&tile) {
            e.long(0x4386c4)?
        } else {
            0
        },
        depth,
        old_world: 0,
        old_device: 0,
        buildings,
    };
    let child = recovered_drawing::fastplot(&mut context, tile, destination, 0, 2, services)?;
    for effect in child.effects {
        if let DrawingEffect::Write(write) = &effect {
            e.write(write.address, &write.bytes)?;
        } else {
            e.plan.effects.push(WorldDisplayEffect::Drawing(effect));
        }
    }
    Ok(())
}

fn current_rect(e: &Execution) -> Result<BodyRect, RuleError> {
    e.plan.state.memory.rect(0x4471f2)
}
fn background(e: &Execution) -> Result<i16, RuleError> {
    e.word(signed_index(0x435508, e.signed_byte(0x402f43)?, 2))
}
fn party_style(e: &Execution) -> Result<i16, RuleError> {
    e.word(signed_index(0x435530, e.signed_byte(0x402f43)?, 2))
}
fn hidden(e: &Execution, x: i16, y: i16) -> Result<bool, RuleError> {
    Ok(e.byte(map_address(0x4358e6, x, y, 90, 1))? == 0 && e.byte(0x443291)? != 0)
}

fn icon(
    e: &mut Execution,
    id: i16,
    clear: bool,
    services: &mut (impl WorldDisplayServices + ?Sized),
) -> Result<(), RuleError> {
    if clear {
        e.lw(0x439598, 0)?;
    }
    let handle = e.drawing(DrawingEffect::GetColorIcon(id), services)?;
    e.lw(0x439598, handle)
}
fn get_world(
    e: &mut Execution,
    services: &mut (impl WorldDisplayServices + ?Sized),
) -> Result<(), RuleError> {
    e.service(
        WorldDisplayRequest::GetWorld {
            world: 0x4046da,
            device: 0x4046de,
        },
        services,
    )?;
    Ok(())
}
fn set_backing(
    e: &mut Execution,
    services: &mut (impl WorldDisplayServices + ?Sized),
) -> Result<(), RuleError> {
    e.drawing(
        DrawingEffect::SetWorld {
            world: e.long(0x4386bc)?,
            device: 0,
        },
        services,
    )?;
    Ok(())
}
fn restore_world(
    e: &mut Execution,
    services: &mut (impl WorldDisplayServices + ?Sized),
) -> Result<(), RuleError> {
    e.drawing(
        DrawingEffect::SetWorld {
            world: e.long(0x4046da)?,
            device: e.long(0x4046de)?,
        },
        services,
    )?;
    Ok(())
}
fn plot_icon(
    e: &mut Execution,
    services: &mut (impl WorldDisplayServices + ?Sized),
) -> Result<(), RuleError> {
    e.service(
        WorldDisplayRequest::PlotColorIcon {
            rectangle: 0x4471f2,
            handle: e.long(0x439598)?,
        },
        services,
    )?;
    Ok(())
}
fn dispose_icon(
    e: &mut Execution,
    services: &mut (impl WorldDisplayServices + ?Sized),
) -> Result<(), RuleError> {
    e.drawing(DrawingEffect::DisposeColorIcon(e.long(0x439598)?), services)?;
    Ok(())
}

fn large_icon_axis(e: &Execution, offset: u32) -> Result<bool, RuleError> {
    let handle = e.long(0x439598)?;
    let pointer = e.long(handle)?;
    Ok(e.word(pointer.wrapping_add(offset))? == 64)
}

fn plot_overlay_icon(
    e: &mut Execution,
    services: &mut (impl WorldDisplayServices + ?Sized),
) -> Result<(), RuleError> {
    if e.long(0x439598)? == 0 {
        return Ok(());
    }
    if large_icon_axis(e, 0x4a)? {
        e.add_word(0x4471f2, -32)?;
    }
    if large_icon_axis(e, 0x4c)? {
        e.add_word(0x4471f4, -32)?;
    }
    get_world(e, services)?;
    set_backing(e, services)?;
    plot_icon(e, services)?;
    if large_icon_axis(e, 0x4a)? {
        e.add_word(0x4471f2, 32)?;
    }
    if large_icon_axis(e, 0x4c)? {
        e.add_word(0x4471f4, 32)?;
    }
    dispose_icon(e, services)?;
    restore_world(e, services)
}

fn tile_pass(
    e: &mut Execution,
    frame: u32,
    pass: u8,
    span: i32,
    large: bool,
    services: &mut (impl WorldDisplayServices + ?Sized),
) -> Result<(), RuleError> {
    e.ww(0x4471f2, -32)?;
    e.ww(0x4471f6, 0)?;
    let mut y = e.word(0x44277e)?;
    while i32::from(y) < (e.long(0x44277c)?.wrapping_add(span as u32) as i32) {
        e.step()?;
        e.add_word(0x4471f2, 32)?;
        e.add_word(0x4471f6, 32)?;
        e.ww(0x4471f4, -32)?;
        e.ww(0x4471f8, 0)?;
        let mut x = e.word(0x44277a)?;
        while i32::from(x) < (e.long(0x442778)?.wrapping_add(span as u32) as i32) {
            e.step()?;
            e.add_word(0x4471f4, 32)?;
            e.add_word(0x4471f8, 32)?;
            if pass == 0 {
                if hidden(e, x, y)? {
                    fastplot(e, 300, current_rect(e)?, services)?;
                } else {
                    let raw = e.word(map_address(0x4085b8, x, y, 180, 2))?;
                    let tile = raw.wrapping_sub(1);
                    e.ww(frame.wrapping_sub(8), (tile / 20).wrapping_shl(5))?;
                    e.ww(frame.wrapping_sub(6), (tile % 20).wrapping_shl(5))?;
                    e.ww(
                        frame.wrapping_sub(2),
                        e.word(frame.wrapping_sub(6))?.wrapping_add(32),
                    )?;
                    e.ww(
                        frame.wrapping_sub(4),
                        e.word(frame.wrapping_sub(8))?.wrapping_add(32),
                    )?;
                    if tile < 0 {
                        fastplot(e, background(e)?, current_rect(e)?, services)?;
                        icon(e, tile.wrapping_add(1), true, services)?;
                        if e.long(0x439598)? != 0 {
                            get_world(e, services)?;
                            set_backing(e, services)?;
                            plot_icon(e, services)?;
                            dispose_icon(e, services)?;
                            restore_world(e, services)?;
                        }
                    } else {
                        e.service(
                            WorldDisplayRequest::CopyBits {
                                source_bitmap: e.long(0x4386c4)?.wrapping_add(2),
                                destination_bitmap: e.long(0x4386bc)?.wrapping_add(2),
                                source_rectangle: frame.wrapping_sub(8),
                                destination_rectangle: 0x4471f2,
                                mode: 0,
                                mask: 0,
                            },
                            services,
                        )?;
                    }
                }
            } else if x <= 89 && y <= 89 {
                let tile = e.word(map_address(0x40c500, x, y, 180, 2))?;
                if pass == 1 && tile > 0 {
                    let id = if hidden(e, x, y)? {
                        300
                    } else {
                        tile.wrapping_add(23000)
                    };
                    fastplot(e, id, current_rect(e)?, services)?;
                } else if pass == 2 && tile < 0 {
                    if large && hidden(e, x, y)? {
                        fastplot(e, 300, current_rect(e)?, services)?;
                    } else {
                        if tile > -500 {
                            fastplot(e, background(e)?, current_rect(e)?, services)?;
                        }
                        icon(e, tile.wrapping_abs(), true, services)?;
                        plot_overlay_icon(e, services)?;
                    }
                }
            }
            x = x.wrapping_add(1);
        }
        y = y.wrapping_add(1);
    }
    Ok(())
}

fn centered_rect(e: &mut Execution, right_first: bool) -> Result<(), RuleError> {
    e.ww(0x4471f2, e.long(0x442768)?.wrapping_shl(5) as i16)?;
    e.ww(0x4471f4, e.long(0x442764)?.wrapping_shl(5) as i16)?;
    let fields = if right_first {
        [0x4471f8, 0x4471f6]
    } else {
        [0x4471f6, 0x4471f8]
    };
    for address in fields {
        e.ww(address, e.word(address - 4)?.wrapping_add(32))?;
    }
    Ok(())
}

fn actor_offset(last: i16, member: i16) -> Option<(i16, i16)> {
    let row: &[(i16, i16)] = match last {
        1 => &[(-10, -10), (10, 10)],
        2 => &[(0, -10), (-10, 10), (10, 10)],
        3 => &[(-8, -14), (8, -14), (-8, 14), (8, 14)],
        4 => &[(-15, -15), (15, -15), (15, 15), (-15, 15), (0, 0)],
        5 => &[
            (-14, -12),
            (0, -12),
            (14, -12),
            (-14, 12),
            (0, 12),
            (14, 12),
        ],
        _ => return None,
    };
    usize::try_from(member)
        .ok()
        .and_then(|index| row.get(index))
        .copied()
}

fn party_icons(
    e: &mut Execution,
    large: bool,
    services: &mut (impl WorldDisplayServices + ?Sized),
) -> Result<(), RuleError> {
    centered_rect(e, true)?;
    e.lw(0x439598, 0)?;
    if e.signed_byte(0x44273a)? > 5 {
        e.write(0x44273a, &[5])?;
    }
    if e.signed_byte(0x44273a)? < 0 {
        e.write(0x44273a, &[0])?;
    }
    if !large || e.word(0x40857a)? == 0 {
        let id = if e.byte(0x442748)? != 0 {
            13006_i16.wrapping_sub(e.signed_byte(0x442740)?)
        } else if e.byte(0x44274f)? != 0 {
            party_style(e)?.wrapping_add(178)
        } else {
            i16::from(e.byte(0x435883)?).wrapping_add(192)
        };
        icon(e, id, false, services)?;
    }
    centered_rect(e, false)?;
    if e.long(0x439598)? != 0 {
        get_world(e, services)?;
        set_backing(e, services)?;
        if party_style(e)? == 0 || e.byte(0x44274f)? != 0 {
            plot_icon(e, services)?;
            dispose_icon(e, services)?;
        } else {
            let mut member = 0_i16;
            while member <= e.signed_byte(0x44273a)? {
                e.step()?;
                let warrior = 0x448aa0_u32.wrapping_add(u32::from(member as u16) * 3138);
                let id = e
                    .word(warrior.wrapping_add(0x56))?
                    .wrapping_add(10000)
                    .wrapping_sub(i16::from(e.byte(0x435883)?).wrapping_mul(10000));
                icon(e, id, false, services)?;
                centered_rect(e, false)?;
                e.service(
                    WorldDisplayRequest::InsetRect {
                        pointer: 0x4471f2,
                        horizontal: e.signed_byte(0x44273a)?,
                        vertical: e.signed_byte(0x44273a)?,
                    },
                    services,
                )?;
                if let Some((horizontal, vertical)) = actor_offset(e.signed_byte(0x44273a)?, member)
                {
                    e.service(
                        WorldDisplayRequest::OffsetRect {
                            pointer: 0x4471f2,
                            horizontal,
                            vertical,
                        },
                        services,
                    )?;
                }
                if e.word(warrior.wrapping_add(0x50))? > 0 {
                    plot_icon(e, services)?;
                }
                dispose_icon(e, services)?;
                member = member.wrapping_add(1);
            }
        }
        restore_world(e, services)?;
    }
    e.service(
        WorldDisplayRequest::CopyBits {
            source_bitmap: e.long(0x4386bc)?.wrapping_add(2),
            destination_bitmap: e.long(0x43879c)?.wrapping_add(2),
            source_rectangle: 0x4471b4,
            destination_rectangle: 0x4471b4,
            mode: 0,
            mask: 0,
        },
        services,
    )?;
    e.drawing(DrawingEffect::SetPort(e.long(0x43878c)?), services)?;
    Ok(())
}

fn center(
    state: &WorldDisplayState,
    frame: u32,
    maximum_steps: u32,
    large: bool,
    services: &mut (impl WorldDisplayServices + ?Sized),
) -> Result<WorldDisplayPlan, RuleError> {
    let mut e = Execution::new(state, maximum_steps);
    e.service(
        WorldDisplayRequest::SetRect {
            pointer: frame.wrapping_sub(16),
            left: 0,
            top: 0,
            right: 32,
            bottom: 32,
        },
        services,
    )?;
    e.write(0x4386a8, &[0])?;
    e.ww(0x4471b6, 0)?;
    let front = e.service(WorldDisplayRequest::FrontWindow, services)?;
    if front != e.long(0x43879c)? {
        let front = e.service(WorldDisplayRequest::FrontWindow, services)?;
        if front != e.long(0x4394d8)? {
            e.service(
                WorldDisplayRequest::SelectWindow(e.long(0x43879c)?),
                services,
            )?;
        }
    }
    let pivot = if large { 8_u32 } else { 5 };
    e.lw(
        0x442778,
        e.long(0x442778)?
            .wrapping_add(e.long(0x442764)?.wrapping_sub(pivot)),
    )?;
    e.lw(
        0x44277c,
        e.long(0x44277c)?
            .wrapping_add(e.long(0x442768)?.wrapping_sub(pivot)),
    )?;
    e.lw(0x442764, pivot)?;
    e.lw(0x442768, pivot)?;
    for (origin, center, limit) in [
        (0x442778, 0x442764, if large { 75_u32 } else { 80 }),
        (0x44277c, 0x442768, if large { 76 } else { 80 }),
    ] {
        let value = e.long(origin)?;
        if (value as i32) < 0 {
            e.lw(center, e.long(center)?.wrapping_add(value))?;
            e.lw(origin, 0)?;
        } else if value > limit {
            e.lw(
                center,
                e.long(center)?.wrapping_add(value.wrapping_sub(limit)),
            )?;
            e.lw(origin, limit)?;
        }
    }
    let maximum = if large { 14_u32 } else { 9 };
    for address in [0x442768, 0x442764] {
        if (e.long(address)? as i32) > maximum as i32 {
            e.lw(address, maximum)?;
        } else if (e.long(address)? as i32) < 0 {
            e.lw(address, 0)?;
        }
    }
    visibility(&mut e, large)?;
    coordinate(&mut e, services)?;
    e.drawing(DrawingEffect::SetPort(e.long(0x43879c)?), services)?;
    e.drawing(DrawingEffect::ForeColor(33), services)?;
    e.drawing(DrawingEffect::BackColor(30), services)?;
    for (pass, span) in if large {
        [(0, 15), (1, 22), (2, 18)]
    } else {
        [(0, 10), (1, 17), (2, 13)]
    } {
        tile_pass(&mut e, frame, pass, span, large, services)?;
    }
    party_icons(&mut e, large, services)?;
    Ok(e.plan)
}

/// centerpictlarge, 0x10f1c4: complete 15/22/18-cell passes and party suffix.
pub fn centerpictlarge(
    state: &WorldDisplayState,
    frame: u32,
    maximum_steps: u32,
    services: &mut (impl WorldDisplayServices + ?Sized),
) -> Result<WorldDisplayPlan, RuleError> {
    center(state, frame, maximum_steps, true, services)
}

/// centerpictsmall, 0x10fc26: complete 10/17/13-cell passes and party suffix.
pub fn centerpictsmall(
    state: &WorldDisplayState,
    frame: u32,
    maximum_steps: u32,
    services: &mut (impl WorldDisplayServices + ?Sized),
) -> Result<WorldDisplayPlan, RuleError> {
    center(state, frame, maximum_steps, false, services)
}

fn copy_tile(e: &mut Execution, tile: u16, destination: u32) -> Result<(), RuleError> {
    let source = 0x43167c_u32.wrapping_add(u32::from(tile) * 40);
    for index in 0..10 {
        let bytes = e.bytes(source.wrapping_add(index * 4), 4)?;
        e.write(destination.wrapping_add(index * 4), &bytes)?;
    }
    Ok(())
}

fn sum_local(e: &Execution, frame: u32, offsets: &[u32]) -> Result<i16, RuleError> {
    let mut value = e.word(frame.wrapping_sub(offsets[0]))?;
    for offset in &offsets[1..] {
        value = value.wrapping_add(e.word(frame.wrapping_sub(*offset))?);
    }
    Ok(value)
}

fn generated_tile(e: &mut Execution, frame: u32, x: i16, y: i16) -> Result<(), RuleError> {
    let address = map_address(0x410448, x, y, 180, 2);
    let previous = map_address(0x410448, x, y.wrapping_sub(1), 180, 2);
    copy_tile(e, e.word(address)? as u16, frame.wrapping_sub(0x2a))?;
    copy_tile(e, e.word(previous)? as u16, frame.wrapping_sub(0x52))?;
    e.ww(
        frame.wrapping_sub(0x56),
        sum_local(e, frame, &[0x26, 0x24, 0x22, 0x1e, 0x20])?,
    )?;
    e.ww(
        frame.wrapping_sub(0x54),
        sum_local(e, frame, &[0x4e, 0x4c, 0x4a, 0x46, 0x20])?,
    )?;
    if e.word(frame.wrapping_sub(0x56))? == 0 && e.word(frame.wrapping_sub(0x54))? == 0 {
        if e.word(frame.wrapping_sub(0x1a))? == 0 && e.bound(100)? >= 5 {
            return Ok(());
        }
        e.ww(
            frame.wrapping_sub(0x1a),
            if e.word(0x4336b8)? != 0 { 1 } else { 2 },
        )?;
        match e.word(frame.wrapping_sub(0x1a))? {
            1 => {
                if e.bound(100)? < 20 {
                    let value = e.range(207, 211)?;
                    e.ww(address, value)?;
                    let old = e.word(previous)?;
                    let result = recovered_navigation::twixt(old, 201, 211);
                    e.plan.effects.push(WorldDisplayEffect::Between {
                        value: old,
                        minimum: 201,
                        maximum: 211,
                        result,
                    });
                    let value = if result == 0 { e.range(201, 205)? } else { 206 };
                    e.ww(previous, value)?;
                } else if e.bound(100)? < 10 {
                    let value = e.range(212, 222)?;
                    e.ww(address, value)?;
                } else {
                    e.ww(address, background(e)?)?;
                }
            }
            2 => {
                if e.bound(100)? < 20 && e.word(previous)? > 113 {
                    let value = e.range(207, 211)?;
                    e.ww(address, value)?;
                    e.ww(previous, e.word(address)?.wrapping_sub(6))?;
                } else if e.bound(100)? < 10 {
                    let value = e.range(212, 222)?;
                    e.ww(address, value)?;
                } else {
                    e.ww(address, background(e)?)?;
                }
            }
            _ => {
                if background(e)? == e.word(address)? && e.bound(100)? < 20 {
                    let value = e.range(212, 222)?;
                    e.ww(address, value)?;
                }
            }
        }
    } else if e.word(frame.wrapping_sub(0x1a))? != 0 {
        e.ww(address, background(e)?)?;
    }
    Ok(())
}

/// combatmap, 0x11ce8e: complete expansion, source-ordered decoration and copy back.
pub fn combatmap(
    state: &WorldDisplayState,
    frame: u32,
    maximum_steps: u32,
) -> Result<WorldDisplayPlan, RuleError> {
    let mut e = Execution::new(state, maximum_steps);
    e.lw(frame.wrapping_sub(0x60), 0)?;
    for offset in [0x57, 0x58, 0x59] {
        e.write(frame.wrapping_sub(offset), &[0])?;
    }
    e.ww(frame.wrapping_sub(0x5c), 0)?;
    e.ww(0x4427aa, 0)?;
    e.ww(0x4427a8, 0)?;
    let mut origin_x = e
        .long(0x442778)?
        .wrapping_add(e.long(0x442764)?)
        .wrapping_sub(15) as i16;
    let origin_y = e
        .long(0x44277c)?
        .wrapping_add(e.long(0x442768)?)
        .wrapping_sub(15) as i16;
    e.ww(frame.wrapping_sub(2), origin_y)?;
    if origin_x < 0 {
        e.ww(0x4427a8, origin_x.wrapping_mul(3))?;
        origin_x = 0;
    }
    if e.word(frame.wrapping_sub(2))? < 0 {
        e.ww(0x4427aa, e.word(frame.wrapping_sub(2))?.wrapping_mul(3))?;
        e.ww(frame.wrapping_sub(2), 0)?;
    }
    if origin_x > 60 {
        e.ww(0x4427a8, origin_x.wrapping_sub(60).wrapping_mul(3))?;
        origin_x = 60;
    }
    if e.word(frame.wrapping_sub(2))? > 60 {
        e.ww(
            0x4427aa,
            e.word(frame.wrapping_sub(2))?
                .wrapping_sub(60)
                .wrapping_mul(3),
        )?;
        e.ww(frame.wrapping_sub(2), 60)?;
    }
    for y in 0_i16..30 {
        for x in 0_i16..30 {
            for sub_y in 0_i16..3 {
                for sub_x in 0_i16..3 {
                    e.step()?;
                    let world_x = x.wrapping_add(origin_x);
                    let world_y = y.wrapping_add(e.word(frame.wrapping_sub(2))?);
                    e.ww(
                        0x4434b6,
                        e.word(map_address(0x4085b8, world_x, world_y, 180, 2))?,
                    )?;
                    if e.word(0x4434b6)? < 0
                        || e.long(map_address(0x414390, world_x, world_y, 360, 4))? != 0
                    {
                        e.ww(0x4434b6, background(&e)?)?;
                    }
                    let tile = if e.word(0x4434b6)? > 0 && e.word(0x4434b6)? < 600 {
                        e.word(0x4434b6)?
                    } else {
                        background(&e)?
                    };
                    let source = 0x43167c_u32
                        .wrapping_add(u32::from(tile as u16) * 40)
                        .wrapping_add(u32::from(sub_y as u16) * 6)
                        .wrapping_add(20);
                    let value = e.word(signed_index(source, sub_x, 2))?;
                    e.ww(
                        map_address(
                            0x410448,
                            x.wrapping_mul(3).wrapping_add(sub_x),
                            y.wrapping_mul(3).wrapping_add(sub_y),
                            180,
                            2,
                        ),
                        value,
                    )?;
                }
            }
        }
    }
    if party_style(&e)? == 0 {
        for y in 0_i16..90 {
            for x in 0_i16..90 {
                e.step()?;
                generated_tile(&mut e, frame, x, y)?;
            }
        }
    }
    for y in 0_i16..90 {
        for x in 0_i16..90 {
            e.step()?;
            let value = e
                .word(map_address(0x410448, x, y, 180, 2))?
                .wrapping_add(1000);
            e.ww(map_address(0x4085b8, x, y, 180, 2), value)?;
        }
    }
    Ok(e.plan)
}
