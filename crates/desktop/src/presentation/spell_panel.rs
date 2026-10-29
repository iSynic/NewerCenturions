//! Window 128's cast rectangles during combat playback: ploticon3's cicn in rect
//! 0x4436f2, rect 0x4436fa (showspellinfo, showresults and killbody) and
//! showspellinfo's name and number.
//!
//! ToolBoxInit places rect 0x4436f2 at WORD40856c + 527, WORD40856e + 335 - 32
//! (0x1324e6..0x132526) and rect 0x4436fa at WORD40856c + 570, WORD40856e + 328 -
//! 32 (0x132400..0x132440) in window 128; the controls sit 20 pixels lower here.
use crate::{
    assets::Assets,
    combat_playback::{Playback, ResultSlot},
};
use centurions_core::SpellInfo;
use macroquad::prelude::*;

const CAST: Rect = Rect::new(687., 463., 32., 32.);
const RESULT: Rect = Rect::new(730., 456., 64., 64.);

pub(super) fn draw(assets: &Assets, playback: &Playback) {
    if let Some(icon) = playback.cast_icon
        && let Some(texture) = assets.icons.get(&icon)
    {
        plot(texture, CAST);
    }
    match playback.result_slot {
        Some(ResultSlot::Spell { icon, area }) => spell_icon(assets, icon, area),
        Some(ResultSlot::Target { icon, size }) => {
            // showresults erases the rectangle (0x1585be), then plots into it
            // inset 16 on each axis WORD4435ce does not extend (0x158582..0x1585bc).
            draw_rectangle(RESULT.x, RESULT.y, RESULT.w, RESULT.h, WHITE);
            let mut rect = RESULT;
            if size == 0 || size == 2 {
                rect.y += 16.;
                rect.h -= 32.;
            }
            if size == 0 || size == 1 {
                rect.x += 16.;
                rect.w -= 32.;
            }
            if let Some(texture) = assets.icons.get(&icon) {
                plot(texture, rect);
            }
        }
        Some(ResultSlot::Cleared) => assets.panel(207, RESULT),
        None => {}
    }
    if let Some(info) = &playback.spell_info {
        text(assets, info);
    }
}

/// showspellinfo's whole drawing, as combatchoice shows it while choosing targets.
pub(crate) fn draw_info(assets: &Assets, info: &SpellInfo) {
    if info.firearm
        && let Some(texture) = info.weapon_icon.and_then(|icon| assets.icons.get(&icon))
    {
        plot(texture, CAST);
    }
    spell_icon(assets, info.icon, info.area);
    text(assets, info);
}

fn spell_icon(assets: &Assets, icon: i16, area: bool) {
    if let Some(texture) = assets.icons.get(&icon) {
        plot(texture, RESULT);
    }
    // 0x159318..0x15936e: cicn 14022 in the rectangle's top-left 32 square,
    // OffsetRect(0, -5).
    if area && let Some(texture) = assets.icons.get(&14022) {
        plot(texture, Rect::new(RESULT.x, RESULT.y - 5., 32., 32.));
    }
}

/// showspellinfo's name in yellowColor and number in redColor, TextSize 12
/// (0x159370..0x159462).
fn text(assets: &Assets, info: &SpellInfo) {
    let yellow = Color::from_rgba(0xfc, 0xf3, 0x05, 255);
    let red = Color::from_rgba(0xdd, 0x08, 0x06, 255);
    if let Some(name) = info.name.map(|name| indexed_name(assets, name)) {
        assets.text(&name, 529., 530., 12, yellow);
    }
    if let Some(number) = info.number {
        assets.text(&number.to_string(), 708., 514., 12, red);
    }
}

/// GetIndString for the spell lists STR# 1000..4006 and the item name lists STR#
/// 1, 201, 401 and 601; a list or index outside them reads empty.
fn indexed_name(assets: &Assets, (list, index): (i16, i16)) -> String {
    let name = if (1..=4).contains(&(list / 1000)) && (0..=6).contains(&(list % 1000)) {
        (1..100)
            .contains(&index)
            .then(|| list / 1000 * 1000 + (list % 1000 + 1) * 100 + index)
            .and_then(|spell| assets.spell_names.get(&spell))
    } else if matches!(list, 1 | 201 | 401 | 601) && (1..=200).contains(&index) {
        assets.names.get(&(list + index - 2))
    } else {
        None
    };
    name.cloned().unwrap_or_default()
}

/// PlotCIcon scales the cicn to the rectangle.
fn plot(texture: &Texture2D, rect: Rect) {
    draw_texture_ex(
        texture,
        rect.x,
        rect.y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(rect.w, rect.h)),
            ..Default::default()
        },
    );
}
