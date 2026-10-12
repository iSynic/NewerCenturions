//! Shop drawing and NPC menu bodies recovered from CODE 1 instructions.

use crate::{
    RuleError,
    recovered_drawing::DrawingEffect,
    recovered_geometry::BodyRect,
    recovered_records::RecordWrite,
    recovered_text::{NumericServices, PascalString},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShopContext {
    pub width: i16,
    pub window: u32,
    pub erase_pattern: u32,
    pub normal_pattern: u32,
    pub credits: i32,
    pub text_work: PascalString,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShopEffect {
    Drawing(DrawingEffect),
    TextMode(i16),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShopPlan {
    pub native_return: i32,
    pub effects: Vec<ShopEffect>,
}

fn half_width(value: i16) -> i16 {
    value.wrapping_add(((value as u16) >> 15) as i16) >> 1
}

fn draw(effects: &mut Vec<ShopEffect>, effect: DrawingEffect) {
    effects.push(ShopEffect::Drawing(effect));
}

fn erase(effects: &mut Vec<ShopEffect>, rect: BodyRect) {
    draw(effects, DrawingEffect::EraseRect(rect));
}

fn offset(effects: &mut Vec<ShopEffect>, rect: &mut BodyRect, horizontal: i16) {
    draw(
        effects,
        DrawingEffect::OffsetRect {
            rect: *rect,
            horizontal,
            vertical: 0,
        },
    );
    rect.left = rect.left.wrapping_add(horizontal);
    rect.right = rect.right.wrapping_add(horizontal);
}

/// eraseshopstats, 0013a068. Packed Pascal arguments move horizontally.
pub fn eraseshopstats(context: &ShopContext, selector: i16) -> ShopPlan {
    let mut effects = Vec::new();
    draw(&mut effects, DrawingEffect::TextFace(0));
    draw(
        &mut effects,
        DrawingEffect::BackPixelPattern(context.erase_pattern),
    );
    let left = half_width(context.width).wrapping_add(108);
    let mut rect = BodyRect {
        top: 415,
        left,
        bottom: 429,
        right: left.wrapping_add(41),
    };
    if selector != 0 {
        erase(&mut effects, rect);
        offset(&mut effects, &mut rect, 52);
        erase(&mut effects, rect);
        offset(&mut effects, &mut rect, 53);
        erase(&mut effects, rect);
    } else {
        offset(&mut effects, &mut rect, 105);
        erase(&mut effects, rect);
    }
    draw(
        &mut effects,
        DrawingEffect::BackPixelPattern(context.normal_pattern),
    );
    ShopPlan {
        native_return: 0,
        effects,
    }
}

/// eraseshopname, 0013a10a. Its original leaves the erase pattern installed.
pub fn eraseshopname(context: &ShopContext, selector: i16) -> ShopPlan {
    let mut effects = Vec::new();
    draw(&mut effects, DrawingEffect::TextFace(0));
    draw(
        &mut effects,
        DrawingEffect::BackPixelPattern(context.erase_pattern),
    );
    let half = half_width(context.width);
    let (left, width, horizontal) = if selector == 0 {
        (half.wrapping_add(67), 188, half.wrapping_add(71))
    } else {
        (half.wrapping_add(422), 206, half.wrapping_add(423))
    };
    erase(
        &mut effects,
        BodyRect {
            top: 437,
            left,
            bottom: 454,
            right: left.wrapping_add(width),
        },
    );
    draw(
        &mut effects,
        DrawingEffect::MoveTo {
            horizontal,
            vertical: 451,
        },
    );
    ShopPlan {
        native_return: 0,
        effects,
    }
}

/// FUN_00139faa: complete shop credit display, including the Pack7 observation.
pub fn shop_credit_display(
    context: &mut ShopContext,
    services: &mut (impl NumericServices + ?Sized),
) -> Result<ShopPlan, RuleError> {
    let mut candidate = context.clone();
    let mut effects = Vec::new();
    draw(&mut effects, DrawingEffect::SetPort(candidate.window));
    effects.push(ShopEffect::TextMode(1));
    draw(&mut effects, DrawingEffect::TextSize(12));
    draw(&mut effects, DrawingEffect::TextFace(0));
    draw(
        &mut effects,
        DrawingEffect::BackPixelPattern(candidate.erase_pattern),
    );
    draw(&mut effects, DrawingEffect::ForeColor(69));
    let left = half_width(candidate.width).wrapping_add(423);
    erase(
        &mut effects,
        BodyRect {
            top: 416,
            left,
            bottom: 428,
            right: left.wrapping_add(88),
        },
    );
    draw(
        &mut effects,
        DrawingEffect::MoveTo {
            horizontal: half_width(candidate.width).wrapping_add(425),
            vertical: 427,
        },
    );
    draw(
        &mut effects,
        DrawingEffect::NumberToString(candidate.credits),
    );
    candidate.text_work = services.number_to_string(candidate.credits)?;
    draw(
        &mut effects,
        DrawingEffect::Write(RecordWrite {
            address: 0x479ed8,
            bytes: candidate.text_work.raw().to_vec(),
        }),
    );
    draw(
        &mut effects,
        DrawingEffect::DrawString(candidate.text_work.clone()),
    );
    draw(
        &mut effects,
        DrawingEffect::BackPixelPattern(candidate.normal_pattern),
    );
    draw(&mut effects, DrawingEffect::BackColor(30));
    *context = candidate;
    Ok(ShopPlan {
        native_return: 0,
        effects,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NpcMenuContext {
    pub menu: u32,
    pub count: i16,
    /// Native data starting at 440edc, including reached adjoining bytes.
    pub record_bytes: Vec<u8>,
    /// Pascal bytes at 404044, supplied by the actual string resource.
    pub placeholder: PascalString,
    pub scratch: PascalString,
}

/// The external library call at 400090 copies/converts the native NPC name.
pub trait NpcNameServices {
    fn copy_name(
        &self,
        source_address: u32,
        source_bytes: &[u8],
    ) -> Result<PascalString, RuleError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NpcMenuEffect {
    EnableItem {
        menu: u32,
        item: i16,
    },
    DisableItem {
        menu: u32,
        item: i16,
    },
    SetItemText {
        menu: u32,
        item: i16,
        text: PascalString,
    },
    SetItemIcon {
        menu: u32,
        item: i16,
        icon: i16,
    },
    CopyName {
        source: u32,
        destination: u32,
    },
    Write(RecordWrite),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NpcMenuPlan {
    pub native_return: i32,
    pub effects: Vec<NpcMenuEffect>,
}

fn missing_npc() -> RuleError {
    RuleError::Invalid("NPC menu addressed record unavailable".into())
}

/// updatenpcmenu, 001046e0. The repeated BYTE105 clear is intentional.
pub fn updatenpcmenu(
    context: &mut NpcMenuContext,
    maximum_updates: usize,
    services: &(impl NpcNameServices + ?Sized),
) -> Result<NpcMenuPlan, RuleError> {
    let mut candidate = context.clone();
    let count = usize::from(candidate.count.max(0) as u16);
    // A count of 32767 wraps the native WORD loop into a nonterminating cycle.
    if candidate.count == i16::MAX || count > maximum_updates {
        return Err(RuleError::Invalid(
            "NPC menu update budget exhausted".into(),
        ));
    }
    let menu = candidate.menu;
    let mut effects = vec![NpcMenuEffect::EnableItem { menu, item: 0 }];
    for item in 1..=20 {
        effects.push(NpcMenuEffect::SetItemText {
            menu,
            item,
            text: candidate.placeholder.clone(),
        });
        effects.push(NpcMenuEffect::DisableItem { menu, item });
        effects.push(NpcMenuEffect::SetItemIcon {
            menu,
            item,
            icon: 0,
        });
    }
    for index in 0..count {
        let offset = index * 288;
        let item = index as i16 + 1;
        effects.push(NpcMenuEffect::EnableItem { menu, item });
        let source = 0x440edc + offset as u32 + 218;
        let bytes = candidate
            .record_bytes
            .get(offset + 218..)
            .ok_or_else(missing_npc)?;
        effects.push(NpcMenuEffect::CopyName {
            source,
            destination: 0x479ed8,
        });
        candidate.scratch = services.copy_name(source, bytes)?;
        effects.push(NpcMenuEffect::Write(RecordWrite {
            address: 0x479ed8,
            bytes: candidate.scratch.raw().to_vec(),
        }));
        effects.push(NpcMenuEffect::SetItemText {
            menu,
            item,
            text: candidate.scratch.clone(),
        });
        let bytes = candidate
            .record_bytes
            .get(offset + 80..offset + 82)
            .ok_or_else(missing_npc)?;
        let icon = i16::from_be_bytes([bytes[0], bytes[1]]);
        if icon < 512 {
            effects.push(NpcMenuEffect::SetItemIcon { menu, item, icon });
        }
        for (field, bytes) in [(268, vec![0, 0]), (261, vec![0]), (261, vec![0])] {
            let destination = candidate
                .record_bytes
                .get_mut(offset + field..offset + field + bytes.len())
                .ok_or_else(missing_npc)?;
            destination.copy_from_slice(&bytes);
            effects.push(NpcMenuEffect::Write(RecordWrite {
                address: 0x440edc + offset as u32 + field as u32,
                bytes,
            }));
        }
    }
    *context = candidate;
    Ok(NpcMenuPlan {
        native_return: 0,
        effects,
    })
}
