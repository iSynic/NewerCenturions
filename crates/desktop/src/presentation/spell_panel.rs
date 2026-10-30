//! The combat panel's spell rectangles while a command plays: ploticon3's cast
//! cicn, the result rectangle (showspellinfo, showresults and killbody) and
//! showspellinfo's name and number.
//!
//! The cast rectangle sits at (527, 335 - 32) and the result rectangle at
//! (570, 328 - 32) from the panel origin; the controls sit 20 pixels lower here
//! than in the game's window.
use crate::{
    assets::Assets,
    combat_playback::{Playback, ResultSlot},
};
use centurions_core::SpellInfo;
use macroquad::prelude::*;

/// Where the cast spell's cicn is plotted in the combat panel.
const CAST: Rect = Rect::new(687., 463., 32., 32.);
/// Where a spell's result is plotted in the combat panel.
const RESULT: Rect = Rect::new(730., 456., 64., 64.);

/// Draws the cast spell's icon and the spell or target result icon in the combat
/// panel.
pub(super) fn draw(assets: &Assets, playback: &Playback) {
    if let Some(icon) = playback.cast_icon
        && let Some(texture) = assets.icons.get(&icon)
    {
        plot(texture, CAST);
    }
    match playback.result_slot {
        Some(ResultSlot::Spell { icon, area }) => spell_icon(assets, icon, area),
        Some(ResultSlot::Target { icon, size }) => {
            // showresults erases the rectangle, then plots into it inset 16 on
            // each axis the size class does not extend.
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
///
/// Original: showspellinfo
pub(crate) fn draw_info(assets: &Assets, info: &SpellInfo) {
    if info.firearm
        && let Some(texture) = info.weapon_icon.and_then(|icon| assets.icons.get(&icon))
    {
        plot(texture, CAST);
    }
    spell_icon(assets, info.icon, info.area);
    text(assets, info);
}

/// Draws the spell's result icon; an area spell adds cicn 14022 in its top left
/// corner.
fn spell_icon(assets: &Assets, icon: i16, area: bool) {
    if let Some(texture) = assets.icons.get(&icon) {
        plot(texture, RESULT);
    }
    // An area spell adds cicn 14022 in the rectangle's top-left 32 square,
    // OffsetRect(0, -5).
    if area && let Some(texture) = assets.icons.get(&14022) {
        plot(texture, Rect::new(RESULT.x, RESULT.y - 5., 32., 32.));
    }
}

/// The spell's name in yellowColor and number in redColor, TextSize 12.
///
/// Original: showspellinfo
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
    crate::assets::texture_in(texture, rect);
}
