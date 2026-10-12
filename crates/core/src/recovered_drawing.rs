//! Source-derived icon and tile drawing plans with explicit Toolbox observations.

use crate::RuleError;
use crate::recovered_geometry::BodyRect;
use crate::recovered_records::{ItemTables, RecordWrite, loaditemnotype};
use crate::recovered_text::{NumericServices, PascalString};
use centurions_content::ITEM_BYTES;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildingWorld {
    pub pointer: u32,
    /// GrafPort Rect at pointer + 0x10, when that memory is available.
    pub bounds: Option<BodyRect>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PictureObservation {
    pub handle: u32,
    /// Rect at dereferenced picture handle + 2; required only for a nonnil handle.
    pub bounds: Option<BodyRect>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DrawingContext {
    pub rect: BodyRect,
    pub banner_rect: BodyRect,
    pub source_rect: BodyRect,
    pub source_row: i16,
    pub source_column: i16,
    pub icon_handle: u32,
    pub item_work: [u8; ITEM_BYTES],
    pub text_work: PascalString,
    pub window: u32,
    pub item_window: u32,
    pub half_width_input: i16,
    pub active_member: i8,
    pub omit_banner: u8,
    pub show_item_background: u8,
    pub text_in_other_port: u8,
    pub erase_pattern: u32,
    pub default_font: i16,
    pub selection: i16,
    pub map_port: u32,
    pub text_port: u32,
    pub secondary_port: u32,
    pub backing_port: u32,
    pub tile_port: u32,
    pub depth: i8,
    pub old_world: u32,
    pub old_device: u32,
    /// Original table at 0x41c220; signed WORD indexes must be represented.
    pub buildings: Vec<BuildingWorld>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DrawingEffect {
    Write(RecordWrite),
    SetPort(u32),
    ForeColor(i32),
    BackColor(i32),
    MoveTo {
        horizontal: i16,
        vertical: i16,
    },
    LineTo {
        horizontal: i16,
        vertical: i16,
    },
    BackPixelPattern(u32),
    EraseRect(BodyRect),
    GetColorIcon(i16),
    PlotColorIcon {
        handle: u32,
        rect: BodyRect,
    },
    DisposeColorIcon(u32),
    TextFont(i16),
    TextFace(u8),
    TextSize(i16),
    DrawString(PascalString),
    NumberToString(i32),
    GetDialogItem {
        dialog: u32,
        item: i16,
    },
    OffsetRect {
        rect: BodyRect,
        horizontal: i16,
        vertical: i16,
    },
    GetIndexedString {
        resource_id: i16,
        index: i16,
    },
    GetWorld,
    GetPicture(i16),
    NewWorld {
        depth: i16,
        bounds: BodyRect,
    },
    SetWorld {
        world: u32,
        device: u32,
    },
    DrawPicture {
        handle: u32,
        bounds: BodyRect,
    },
    /// Bitmap pointers are GrafPort + 2, retaining native pointer arithmetic.
    CopyBits {
        source_bitmap: u32,
        destination_bitmap: u32,
        source: BodyRect,
        destination: BodyRect,
        mode: i16,
        mask: u32,
    },
}

/// Adapters supply observations without executing the returned presentation plan.
pub trait DrawingServices {
    fn color_icon(&mut self, id: i16) -> Result<u32, RuleError>;
    fn indexed_string(&mut self, resource_id: i16, index: i16) -> Result<PascalString, RuleError>;
    fn current_world(&mut self) -> Result<(u32, u32), RuleError>;
    fn picture(&mut self, id: i16) -> Result<PictureObservation, RuleError>;
    /// NewGWorld's OSErr is ignored by the original; returned memory is explicit.
    fn new_world(
        &mut self,
        depth: i16,
        bounds: BodyRect,
    ) -> Result<(i16, BuildingWorld), RuleError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DrawingPlan {
    pub native_return: i32,
    pub effects: Vec<DrawingEffect>,
}

fn missing(label: &str) -> RuleError {
    RuleError::Invalid(format!("drawing observation unavailable: {label}"))
}

fn write(effects: &mut Vec<DrawingEffect>, address: u32, bytes: &[u8]) {
    effects.push(DrawingEffect::Write(RecordWrite {
        address,
        bytes: bytes.to_vec(),
    }));
}

fn rect_bytes(rect: BodyRect) -> [u8; 8] {
    let mut bytes = [0; 8];
    for (out, value) in
        bytes
            .as_chunks_mut::<2>()
            .0
            .iter_mut()
            .zip([rect.top, rect.left, rect.bottom, rect.right])
    {
        *out = value.to_be_bytes();
    }
    bytes
}

fn icon(
    context: &mut DrawingContext,
    id: i16,
    rect: BodyRect,
    effects: &mut Vec<DrawingEffect>,
    services: &mut impl DrawingServices,
) -> Result<(), RuleError> {
    context.icon_handle = 0;
    write(effects, 0x439598, &0u32.to_be_bytes());
    effects.push(DrawingEffect::GetColorIcon(id));
    let handle = services.color_icon(id)?;
    context.icon_handle = handle;
    write(effects, 0x439598, &handle.to_be_bytes());
    if handle != 0 {
        effects.push(DrawingEffect::PlotColorIcon { handle, rect });
        effects.push(DrawingEffect::DisposeColorIcon(handle));
    }
    Ok(())
}

/// ploticon2, 0x149fc6: right is written before bottom, then the global Rect is used.
pub fn ploticon2(
    context: &mut DrawingContext,
    id: i16,
    services: &mut impl DrawingServices,
) -> Result<DrawingPlan, RuleError> {
    let mut candidate = context.clone();
    let mut effects = Vec::new();
    candidate.rect.right = candidate.rect.left.wrapping_add(32);
    write(&mut effects, 0x4471f8, &candidate.rect.right.to_be_bytes());
    candidate.rect.bottom = candidate.rect.top.wrapping_add(32);
    write(&mut effects, 0x4471f6, &candidate.rect.bottom.to_be_bytes());
    let rect = candidate.rect;
    icon(&mut candidate, id, rect, &mut effects, services)?;
    *context = candidate;
    Ok(DrawingPlan {
        native_return: 0,
        effects,
    })
}

/// ploticon3, 0x14a02c: the caller supplies all four Rect WORDs after the icon ID.
pub fn ploticon3(
    context: &mut DrawingContext,
    id: i16,
    rect: BodyRect,
    services: &mut impl DrawingServices,
) -> Result<DrawingPlan, RuleError> {
    let mut candidate = context.clone();
    let mut effects = Vec::new();
    icon(&mut candidate, id, rect, &mut effects, services)?;
    *context = candidate;
    Ok(DrawingPlan {
        native_return: 0,
        effects,
    })
}

fn half_word(value: i16) -> i16 {
    value.wrapping_add(((value as u16) >> 15) as i16) >> 1
}

fn item_word(context: &DrawingContext, offset: usize) -> i16 {
    i16::from_be_bytes(context.item_work[offset..offset + 2].try_into().unwrap())
}

/// ploticon, 0x14a074: item loading, erase/background/icon, indexed text, font reset.
pub fn ploticon(
    context: &mut DrawingContext,
    tables: &ItemTables<'_>,
    id: i16,
    services: &mut impl DrawingServices,
) -> Result<DrawingPlan, RuleError> {
    let mut candidate = context.clone();
    let loaded = loaditemnotype(tables, &candidate.item_work, id)?;
    candidate.item_work = loaded.bytes;
    let mut effects: Vec<_> = loaded
        .writes
        .into_iter()
        .map(DrawingEffect::Write)
        .collect();
    candidate.rect.left = if candidate.window == candidate.item_window {
        half_word(candidate.half_width_input).wrapping_add(390)
    } else {
        10
    };
    write(&mut effects, 0x4471f4, &candidate.rect.left.to_be_bytes());
    if id == 0 && (candidate.window != candidate.item_window || candidate.active_member != -1) {
        candidate.item_work[10..12].fill(0);
        write(&mut effects, 0x453646, &[0, 0]);
    }
    candidate.rect.bottom = candidate.rect.top.wrapping_add(32);
    write(&mut effects, 0x4471f6, &candidate.rect.bottom.to_be_bytes());
    candidate.rect.right = candidate
        .rect
        .left
        .wrapping_add(half_word(candidate.half_width_input))
        .wrapping_add(if candidate.omit_banner == 0 { 274 } else { 250 });
    write(&mut effects, 0x4471f8, &candidate.rect.right.to_be_bytes());
    effects.push(DrawingEffect::MoveTo {
        horizontal: candidate.rect.left.wrapping_add(42),
        vertical: candidate.rect.bottom.wrapping_sub(15),
    });
    effects.push(DrawingEffect::BackPixelPattern(candidate.erase_pattern));
    effects.push(DrawingEffect::EraseRect(candidate.rect));
    candidate.rect.right = candidate.rect.left.wrapping_add(32);
    write(&mut effects, 0x4471f8, &candidate.rect.right.to_be_bytes());
    if candidate.omit_banner == 0 {
        let rect = candidate.banner_rect;
        icon(&mut candidate, 174, rect, &mut effects, services)?;
    }
    let item_icon = item_word(&candidate, 10);
    if item_icon != 0 {
        if candidate.show_item_background != 0 {
            let item_type = item_word(&candidate, 26);
            let background = if item_type < 0 || matches!(item_type, 23..=25) {
                131
            } else {
                176
            };
            let rect = candidate.rect;
            icon(&mut candidate, background, rect, &mut effects, services)?;
        }
        let rect = candidate.rect;
        icon(&mut candidate, item_icon, rect, &mut effects, services)?;
        effects.push(DrawingEffect::ForeColor(33));
        let item_id = item_word(&candidate, 102);
        let resource_id = crate::rules::selection_base(item_id).wrapping_add(1);
        let index = item_id.wrapping_sub(candidate.selection).wrapping_add(1);
        effects.push(DrawingEffect::GetIndexedString { resource_id, index });
        candidate.text_work = services.indexed_string(resource_id, index)?;
        write(&mut effects, 0x479ed8, candidate.text_work.raw());
        if candidate.text_in_other_port != 0 {
            effects.push(DrawingEffect::SetPort(candidate.text_port));
        }
        effects.push(DrawingEffect::TextSize(12));
        effects.push(DrawingEffect::DrawString(candidate.text_work.clone()));
    }
    effects.push(DrawingEffect::TextFont(candidate.default_font));
    effects.push(DrawingEffect::TextFace(0));
    *context = candidate;
    Ok(DrawingPlan {
        native_return: 0,
        effects,
    })
}

fn building_index(context: &DrawingContext, index: i16) -> Result<usize, RuleError> {
    let index = usize::try_from(index).map_err(|_| missing("signed building index"))?;
    context
        .buildings
        .get(index)
        .ok_or_else(|| missing("building table slot"))?;
    Ok(index)
}

fn allocate(
    context: &mut DrawingContext,
    index: i16,
    effects: &mut Vec<DrawingEffect>,
    services: &mut impl DrawingServices,
) -> Result<i32, RuleError> {
    let slot = building_index(context, index)?;
    if context.buildings[slot].pointer != 0 {
        return Ok(0);
    }
    effects.push(DrawingEffect::GetWorld);
    (context.old_world, context.old_device) = services.current_world()?;
    write(effects, 0x4046da, &context.old_world.to_be_bytes());
    write(effects, 0x4046de, &context.old_device.to_be_bytes());
    let id = index.wrapping_add(23000);
    effects.push(DrawingEffect::GetPicture(id));
    let picture = services.picture(id)?;
    if picture.handle == 0 {
        return Ok(0);
    }
    let bounds = picture.bounds.ok_or_else(|| missing("picture bounds"))?;
    let depth = i16::from(context.depth);
    effects.push(DrawingEffect::NewWorld { depth, bounds });
    let (_, world) = services.new_world(depth, bounds)?;
    write(
        effects,
        0x41c220 + slot as u32 * 4,
        &world.pointer.to_be_bytes(),
    );
    context.buildings[slot] = world;
    effects.push(DrawingEffect::SetWorld {
        world: context.buildings[slot].pointer,
        device: 0,
    });
    effects.push(DrawingEffect::ForeColor(33));
    effects.push(DrawingEffect::BackColor(30));
    effects.push(DrawingEffect::DrawPicture {
        handle: picture.handle,
        bounds,
    });
    effects.push(DrawingEffect::SetWorld {
        world: context.old_world,
        device: context.old_device,
    });
    Ok(1)
}

/// allocatebuilding, 0x122b18; a missing picture returns zero without SetWorld.
pub fn allocatebuilding(
    context: &mut DrawingContext,
    index: i16,
    services: &mut impl DrawingServices,
) -> Result<DrawingPlan, RuleError> {
    let mut candidate = context.clone();
    let mut effects = Vec::new();
    let native_return = allocate(&mut candidate, index, &mut effects, services)?;
    *context = candidate;
    Ok(DrawingPlan {
        native_return,
        effects,
    })
}

fn copy_building(
    context: &mut DrawingContext,
    index: i16,
    destination: BodyRect,
    effects: &mut Vec<DrawingEffect>,
    services: &mut impl DrawingServices,
) -> Result<(), RuleError> {
    let slot = building_index(context, index)?;
    if context.buildings[slot].pointer == 0 {
        allocate(context, index, effects, services)?;
    }
    let world = &context.buildings[slot];
    if world.pointer == 0 {
        return Err(missing("building pointer for CopyBits"));
    }
    effects.push(DrawingEffect::CopyBits {
        source_bitmap: world.pointer.wrapping_add(2),
        destination_bitmap: context.backing_port.wrapping_add(2),
        source: world
            .bounds
            .ok_or_else(|| missing("building port bounds"))?,
        destination,
        mode: 36,
        mask: 0,
    });
    Ok(())
}

/// gcopy, 0x122dc4, copies the complete source port bounds with mode 36.
pub fn gcopy(
    context: &mut DrawingContext,
    index: i16,
    destination: BodyRect,
    services: &mut impl DrawingServices,
) -> Result<DrawingPlan, RuleError> {
    let mut candidate = context.clone();
    let mut effects = Vec::new();
    copy_building(&mut candidate, index, destination, &mut effects, services)?;
    *context = candidate;
    Ok(DrawingPlan {
        native_return: 0,
        effects,
    })
}

/// fastplot, 0x122bec; mode/copy are the low BYTEs of the two caller WORDs.
pub fn fastplot(
    context: &mut DrawingContext,
    mut tile: i16,
    destination: BodyRect,
    mode: i8,
    copy: i8,
    services: &mut impl DrawingServices,
) -> Result<DrawingPlan, RuleError> {
    let mut candidate = context.clone();
    let mut effects = Vec::new();
    if tile < 0 {
        return Ok(DrawingPlan {
            native_return: 0,
            effects,
        });
    }
    effects.push(DrawingEffect::SetPort(candidate.map_port));
    effects.push(DrawingEffect::ForeColor(33));
    effects.push(DrawingEffect::BackColor(30));
    if tile >= 23000 {
        let index = tile.wrapping_sub(23000);
        let slot = building_index(&candidate, index)?;
        if candidate.buildings[slot].pointer == 0 {
            allocate(&mut candidate, index, &mut effects, services)?;
        }
        candidate.source_rect = candidate.rect;
        let raw = rect_bytes(candidate.source_rect);
        write(&mut effects, 0x4434ba, &raw[..4]);
        write(&mut effects, 0x4434be, &raw[4..]);
        let world = &candidate.buildings[slot];
        if world.pointer == 0 {
            return Err(missing("building pointer for fastplot"));
        }
        let bounds = world
            .bounds
            .ok_or_else(|| missing("building port bounds"))?;
        candidate.source_rect.top = candidate.source_rect.bottom.wrapping_sub(bounds.bottom);
        write(
            &mut effects,
            0x4434ba,
            &candidate.source_rect.top.to_be_bytes(),
        );
        candidate.source_rect.left = candidate.source_rect.right.wrapping_sub(bounds.right);
        write(
            &mut effects,
            0x4434bc,
            &candidate.source_rect.left.to_be_bytes(),
        );
        // The building path passes the modified global source Rect, not the caller Rect.
        let destination = candidate.source_rect;
        copy_building(&mut candidate, index, destination, &mut effects, services)?;
    } else {
        if tile > 999 {
            tile = tile.wrapping_sub(1000);
        }
        candidate.source_row = tile.wrapping_sub(1) / 20;
        write(&mut effects, 0x4434b4, &candidate.source_row.to_be_bytes());
        candidate.source_column = tile
            .wrapping_sub(candidate.source_row.wrapping_mul(20))
            .wrapping_sub(1);
        write(
            &mut effects,
            0x4434b6,
            &candidate.source_column.to_be_bytes(),
        );
        candidate.source_rect.top = candidate.source_row.wrapping_shl(5);
        write(
            &mut effects,
            0x4434ba,
            &candidate.source_rect.top.to_be_bytes(),
        );
        candidate.source_rect.left = candidate.source_column.wrapping_shl(5);
        write(
            &mut effects,
            0x4434bc,
            &candidate.source_rect.left.to_be_bytes(),
        );
        candidate.source_rect.right = candidate.source_rect.left.wrapping_add(32);
        write(
            &mut effects,
            0x4434c0,
            &candidate.source_rect.right.to_be_bytes(),
        );
        candidate.source_rect.bottom = candidate.source_rect.top.wrapping_add(32);
        write(
            &mut effects,
            0x4434be,
            &candidate.source_rect.bottom.to_be_bytes(),
        );
        let port = match copy {
            0 => Some(candidate.map_port),
            1 => Some(candidate.secondary_port),
            2 => Some(candidate.backing_port),
            _ => None,
        };
        if let Some(port) = port {
            effects.push(DrawingEffect::CopyBits {
                source_bitmap: candidate.tile_port.wrapping_add(2),
                destination_bitmap: port.wrapping_add(2),
                source: candidate.source_rect,
                destination,
                mode: i16::from(mode),
                mask: 0,
            });
        }
    }
    *context = candidate;
    Ok(DrawingPlan {
        native_return: 0,
        effects,
    })
}

pub struct SpellDrawingInput<'a> {
    /// BYTE at 0x44374f and WORD at 0x4435bc.
    pub combat_type: u8,
    pub enabled: i16,
    pub selected_warrior: i16,
    /// Position WORD pairs at 0x44709a, selected with a signed WORD index.
    pub warrior_positions: &'a [[i16; 2]],
    pub end_position: [i16; 2],
    pub mask_port: u32,
    pub source_rect: BodyRect,
    pub destination_rect: BodyRect,
}

/// drawspell, 0x1377f4: line first, then mode-two mask CopyBits.
pub fn drawspell(
    context: &DrawingContext,
    input: SpellDrawingInput<'_>,
) -> Result<DrawingPlan, RuleError> {
    let mut effects = Vec::new();
    if input.combat_type != 5 && input.enabled != 0 {
        let index = usize::try_from(input.selected_warrior)
            .map_err(|_| missing("spell warrior position"))?;
        let position = input
            .warrior_positions
            .get(index)
            .ok_or_else(|| missing("spell warrior position"))?;
        let pixel = |coordinate: i16| coordinate.wrapping_shl(5).wrapping_add(16);
        effects.push(DrawingEffect::SetPort(context.map_port));
        effects.push(DrawingEffect::MoveTo {
            horizontal: pixel(position[0]),
            vertical: pixel(position[1]),
        });
        effects.push(DrawingEffect::LineTo {
            horizontal: pixel(input.end_position[0]),
            vertical: pixel(input.end_position[1]),
        });
        effects.push(DrawingEffect::CopyBits {
            source_bitmap: input.mask_port.wrapping_add(2),
            destination_bitmap: context.map_port.wrapping_add(2),
            source: input.source_rect,
            destination: input.destination_rect,
            mode: 2,
            mask: 0,
        });
    }
    Ok(DrawingPlan {
        native_return: 0,
        effects,
    })
}

/// pict, 0x1382a8: GetPicture then DrawPicture even for a nil handle.
/// Native D0 is not assigned a defined return value by this void routine.
pub fn pict(
    id: i16,
    rect: BodyRect,
    services: &mut impl DrawingServices,
) -> Result<Vec<DrawingEffect>, RuleError> {
    let picture = services.picture(id)?;
    Ok(vec![
        DrawingEffect::GetPicture(id),
        DrawingEffect::DrawPicture {
            handle: picture.handle,
            bounds: rect,
        },
    ])
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TierContext {
    pub dialog: u32,
    pub selected_tier: i16,
    pub item_type: i16,
    pub item_handle: u32,
    pub rect: BodyRect,
}

pub trait TierServices {
    fn dialog_item(&mut self, dialog: u32, item: i16) -> Result<(i16, u32, BodyRect), RuleError>;
}

/// updatetier, 0x10b814. GetDItem's Rect is changed to an 18-pixel icon extent.
pub fn updatetier(
    context: &mut DrawingContext,
    tier_context: &mut TierContext,
    value: i16,
    services: &mut (impl DrawingServices + TierServices),
) -> Result<DrawingPlan, RuleError> {
    let mut drawing = context.clone();
    let mut tier = tier_context.clone();
    let item = tier.selected_tier.wrapping_add(46);
    let mut effects = vec![DrawingEffect::GetDialogItem {
        dialog: tier.dialog,
        item,
    }];
    (tier.item_type, tier.item_handle, tier.rect) = services.dialog_item(tier.dialog, item)?;
    write(&mut effects, 0x4434b0, &tier.item_type.to_be_bytes());
    write(&mut effects, 0x4434dc, &tier.item_handle.to_be_bytes());
    write(&mut effects, 0x437a88, &rect_bytes(tier.rect));
    tier.rect.bottom = tier.rect.top.wrapping_add(18);
    write(&mut effects, 0x437a8c, &tier.rect.bottom.to_be_bytes());
    tier.rect.right = tier.rect.left.wrapping_add(18);
    write(&mut effects, 0x437a8e, &tier.rect.right.to_be_bytes());
    if tier.selected_tier == 1 {
        effects.push(DrawingEffect::OffsetRect {
            rect: tier.rect,
            horizontal: 87,
            vertical: 0,
        });
        tier.rect.left = tier.rect.left.wrapping_add(87);
        tier.rect.right = tier.rect.right.wrapping_add(87);
        write(&mut effects, 0x437a8a, &tier.rect.left.to_be_bytes());
        write(&mut effects, 0x437a8e, &tier.rect.right.to_be_bytes());
    }
    icon(
        &mut drawing,
        value.wrapping_add(155),
        tier.rect,
        &mut effects,
        services,
    )?;
    *context = drawing;
    *tier_context = tier;
    Ok(DrawingPlan {
        native_return: 0,
        effects,
    })
}

/// updatebootymoney, 0x1056d4. The returned Rect retains its caller-frame right overwrite.
pub fn updatebootymoney(
    context: &mut DrawingContext,
    port: u32,
    mut rect: BodyRect,
    amount: i32,
    services: &mut (impl DrawingServices + NumericServices),
) -> Result<(BodyRect, DrawingPlan), RuleError> {
    let mut candidate = context.clone();
    let mut effects = vec![DrawingEffect::SetPort(port), DrawingEffect::ForeColor(69)];
    rect.right = rect.left.wrapping_add(640);
    effects.extend(pict(163, rect, services)?);
    effects.push(DrawingEffect::MoveTo {
        horizontal: rect.left.wrapping_add(20),
        vertical: rect.top.wrapping_add(46),
    });
    effects.push(DrawingEffect::NumberToString(amount));
    candidate.text_work = services.number_to_string(amount)?;
    write(&mut effects, 0x479ed8, candidate.text_work.raw());
    effects.push(DrawingEffect::DrawString(candidate.text_work.clone()));
    *context = candidate;
    Ok((
        rect,
        DrawingPlan {
            native_return: 0,
            effects,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    struct Observations {
        icons: VecDeque<u32>,
        picture: PictureObservation,
        world: BuildingWorld,
    }

    impl DrawingServices for Observations {
        fn color_icon(&mut self, _: i16) -> Result<u32, RuleError> {
            self.icons
                .pop_front()
                .ok_or_else(|| missing("icon test input"))
        }
        fn indexed_string(
            &mut self,
            resource_id: i16,
            index: i16,
        ) -> Result<PascalString, RuleError> {
            assert_eq!((resource_id, index), (201, 4));
            PascalString::from_payload(b"Tool")
        }
        fn current_world(&mut self) -> Result<(u32, u32), RuleError> {
            Ok((700, 701))
        }
        fn picture(&mut self, _: i16) -> Result<PictureObservation, RuleError> {
            Ok(self.picture.clone())
        }
        fn new_world(
            &mut self,
            depth: i16,
            _: BodyRect,
        ) -> Result<(i16, BuildingWorld), RuleError> {
            assert_eq!(depth, -8);
            Ok((-1, self.world.clone()))
        }
    }

    impl TierServices for Observations {
        fn dialog_item(
            &mut self,
            dialog: u32,
            item: i16,
        ) -> Result<(i16, u32, BodyRect), RuleError> {
            assert_eq!((dialog, item), (13, 47));
            Ok((5, 20, rect(32760, -3, 99, 100)))
        }
    }

    impl NumericServices for Observations {
        fn number_to_string(&mut self, value: i32) -> Result<PascalString, RuleError> {
            assert_eq!(value, i32::MIN);
            PascalString::from_payload(b"-2147483648")
        }
        fn string_to_number(&mut self, _: &PascalString) -> Result<i32, RuleError> {
            Err(missing("unused numeric parse"))
        }
    }

    fn rect(top: i16, left: i16, bottom: i16, right: i16) -> BodyRect {
        BodyRect {
            top,
            left,
            bottom,
            right,
        }
    }

    fn setup() -> (DrawingContext, Observations) {
        let context = DrawingContext {
            rect: rect(32760, 32760, 1, 2),
            banner_rect: rect(1, 2, 3, 4),
            source_rect: rect(5, 6, 7, 8),
            source_row: 9,
            source_column: 10,
            icon_handle: 99,
            item_work: [0; ITEM_BYTES],
            text_work: PascalString::default(),
            window: 1,
            item_window: 1,
            half_width_input: -3,
            active_member: 0,
            omit_banner: 0,
            show_item_background: 1,
            text_in_other_port: 1,
            erase_pattern: 100,
            default_font: 4,
            selection: 200,
            map_port: 200,
            text_port: 300,
            secondary_port: 400,
            backing_port: 500,
            tile_port: 600,
            depth: -8,
            old_world: 0,
            old_device: 0,
            buildings: vec![BuildingWorld {
                pointer: 0,
                bounds: None,
            }],
        };
        let observations = Observations {
            icons: [0, 11, 12].into(),
            picture: PictureObservation {
                handle: 77,
                bounds: Some(rect(0, 0, 64, 96)),
            },
            world: BuildingWorld {
                pointer: 900,
                bounds: Some(rect(0, 0, 64, 96)),
            },
        };
        (context, observations)
    }

    #[test]
    fn icon_rectangle_wrap_nil_lifetime_and_missing_observation_are_atomic() {
        let (mut context, mut service) = setup();
        let plan = ploticon2(&mut context, 7, &mut service).unwrap();
        assert_eq!(context.rect, rect(32760, 32760, -32744, -32744));
        assert_eq!(context.icon_handle, 0);
        assert!(
            !plan
                .effects
                .iter()
                .any(|e| matches!(e, DrawingEffect::DisposeColorIcon(_)))
        );
        let global_rect = context.rect;
        let plan = ploticon3(&mut context, 8, rect(10, 20, 30, 40), &mut service).unwrap();
        assert_eq!(context.rect, global_rect);
        assert_eq!(context.icon_handle, 11);
        assert_eq!(
            &plan.effects[3..],
            &[
                DrawingEffect::PlotColorIcon {
                    handle: 11,
                    rect: rect(10, 20, 30, 40)
                },
                DrawingEffect::DisposeColorIcon(11),
            ]
        );
        service.icons.clear();
        let before = context.clone();
        assert!(ploticon2(&mut context, 9, &mut service).is_err());
        assert_eq!(context, before);
    }

    #[test]
    fn item_background_selection_and_erase_extent_follow_native_order() {
        let (mut context, mut service) = setup();
        let mut table = vec![0; 200 * ITEM_BYTES];
        let record = &mut table[203usize % 200 * ITEM_BYTES..][..ITEM_BYTES];
        record[10..12].copy_from_slice(&55i16.to_be_bytes());
        record[26..28].copy_from_slice(&(-1i16).to_be_bytes());
        record[102..104].copy_from_slice(&203i16.to_be_bytes());
        let tables = ItemTables {
            families: [&[], &table, &[], &[], &[]],
        };
        let plan = ploticon(&mut context, &tables, 203, &mut service).unwrap();
        assert_eq!(context.rect.left, 389);
        assert_eq!(context.rect.right, 421);
        assert!(
            plan.effects
                .contains(&DrawingEffect::EraseRect(rect(32760, 389, -32744, 662)))
        );
        let ids: Vec<_> = plan
            .effects
            .iter()
            .filter_map(|e| match e {
                DrawingEffect::GetColorIcon(id) => Some(*id),
                _ => None,
            })
            .collect();
        assert_eq!(ids, [174, 131, 55]);
        assert_eq!(context.text_work.payload(), b"Tool");
        assert_eq!(context.icon_handle, 12);
        assert_eq!(
            &plan.effects[plan.effects.len() - 2..],
            &[DrawingEffect::TextFont(4), DrawingEffect::TextFace(0)]
        );
    }

    #[test]
    fn tile_and_building_paths_preserve_distinct_rectangles_and_modes() {
        let (mut context, mut service) = setup();
        let before = context.clone();
        assert!(
            fastplot(&mut context, -1, rect(0, 0, 0, 0), -1, 0, &mut service)
                .unwrap()
                .effects
                .is_empty()
        );
        assert_eq!(context, before);
        let destination = rect(10, 20, 30, 40);
        let plan = fastplot(&mut context, 0, destination, -128, 1, &mut service).unwrap();
        assert_eq!(context.source_rect, rect(0, -32, 32, 0));
        assert_eq!(
            plan.effects.last(),
            Some(&DrawingEffect::CopyBits {
                source_bitmap: 602,
                destination_bitmap: 402,
                source: rect(0, -32, 32, 0),
                destination,
                mode: -128,
                mask: 0,
            })
        );
        context.rect = rect(20, 30, 84, 126);
        let plan = fastplot(&mut context, 23000, destination, 7, 0, &mut service).unwrap();
        assert_eq!(context.source_rect, rect(20, 30, 84, 126));
        assert_eq!(
            plan.effects.last(),
            Some(&DrawingEffect::CopyBits {
                source_bitmap: 902,
                destination_bitmap: 502,
                source: rect(0, 0, 64, 96),
                destination: rect(20, 30, 84, 126),
                mode: 36,
                mask: 0,
            })
        );
        assert!(plan.effects.contains(&DrawingEffect::SetWorld {
            world: 700,
            device: 701
        }));
        let (mut nil_context, mut nil_service) = setup();
        nil_service.picture.handle = 0;
        let plan = allocatebuilding(&mut nil_context, 0, &mut nil_service).unwrap();
        assert_eq!(plan.native_return, 0);
        assert!(
            !plan
                .effects
                .iter()
                .any(|e| matches!(e, DrawingEffect::SetWorld { .. }))
        );
        let before = nil_context.clone();
        assert!(gcopy(&mut nil_context, 0, destination, &mut nil_service).is_err());
        assert_eq!(nil_context, before);
    }

    #[test]
    fn spell_gate_skips_position_read_and_picture_keeps_nil_draw_request() {
        let (context, mut service) = setup();
        let disabled = SpellDrawingInput {
            combat_type: 5,
            enabled: 1,
            selected_warrior: -1,
            warrior_positions: &[],
            end_position: [3, 4],
            mask_port: 80,
            source_rect: rect(0, 0, 1, 1),
            destination_rect: rect(1, 2, 3, 4),
        };
        assert!(drawspell(&context, disabled).unwrap().effects.is_empty());
        let active = SpellDrawingInput {
            combat_type: 0,
            enabled: 1,
            selected_warrior: 0,
            warrior_positions: &[[1024, -1]],
            end_position: [3, 4],
            mask_port: 80,
            source_rect: rect(0, 0, 1, 1),
            destination_rect: rect(1, 2, 3, 4),
        };
        let plan = drawspell(&context, active).unwrap();
        assert_eq!(
            plan.effects[1],
            DrawingEffect::MoveTo {
                horizontal: -32752,
                vertical: -16
            }
        );
        assert_eq!(
            plan.effects[2],
            DrawingEffect::LineTo {
                horizontal: 112,
                vertical: 144
            }
        );
        service.picture.handle = 0;
        assert_eq!(
            pict(8, rect(1, 2, 3, 4), &mut service).unwrap(),
            [
                DrawingEffect::GetPicture(8),
                DrawingEffect::DrawPicture {
                    handle: 0,
                    bounds: rect(1, 2, 3, 4)
                },
            ]
        );
    }

    #[test]
    fn tier_and_money_helpers_compose_icon_picture_and_full_long_text() {
        let (mut context, mut services) = setup();
        services.icons = [77].into();
        let mut tier = TierContext {
            dialog: 13,
            selected_tier: 1,
            item_type: 0,
            item_handle: 0,
            rect: rect(0, 0, 0, 0),
        };
        let global_rect = context.rect;
        let plan = updatetier(&mut context, &mut tier, 2, &mut services).unwrap();
        assert_eq!(tier.rect, rect(32760, 84, -32758, 102));
        assert_eq!(context.rect, global_rect);
        assert!(plan.effects.contains(&DrawingEffect::OffsetRect {
            rect: rect(32760, -3, -32758, 15),
            horizontal: 87,
            vertical: 0
        }));
        assert!(plan.effects.contains(&DrawingEffect::GetColorIcon(157)));
        assert!(plan.effects.contains(&DrawingEffect::PlotColorIcon {
            handle: 77,
            rect: tier.rect
        }));
        let (argument, plan) = updatebootymoney(
            &mut context,
            15,
            rect(32760, 32760, 200, 0),
            i32::MIN,
            &mut services,
        )
        .unwrap();
        assert_eq!(argument, rect(32760, 32760, 200, -32136));
        assert!(plan.effects.contains(&DrawingEffect::MoveTo {
            horizontal: -32756,
            vertical: -32730
        }));
        assert_eq!(context.text_work.payload(), b"-2147483648");
        assert_eq!(
            plan.effects.last(),
            Some(&DrawingEffect::DrawString(context.text_work.clone()))
        );
    }
}
