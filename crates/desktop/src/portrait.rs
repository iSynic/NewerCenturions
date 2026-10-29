//! updatepictbox 0x161b80 and plotportrait 0x14a3e6: a warrior's portrait cicn
//! in its 32-pixel box with its condition, class and stamina marks.
use crate::assets::Assets;
use centurions_content::Warrior;
use macroquad::prelude::*;

pub struct Marks {
    /// Preference item 17 (0x439584): the class label.
    pub class_label: bool,
    /// The left edge plotportrait gives its condition cicns in the current
    /// window, or `None` where it skips them (0x14a4cc..0x14a584).
    pub conditions_left: Option<f32>,
    /// The member's BYTE at 0x442742.
    pub automatic: bool,
}

/// Plots the portrait with its 32-pixel box's upper-left corner at `at`. A
/// 40-pixel cicn grows the box by four on every side (0x14a47e).
pub fn draw(assets: &Assets, warrior: &Warrior, at: Vec2, marks: Marks) {
    let record = warrior.record();
    let texture = record
        .portrait()
        .ok()
        .and_then(|id| assets.portraits.get(&id));
    let rect = if texture.is_some_and(|texture| texture.height() == 40.) {
        Rect::new(at.x - 4., at.y - 4., 40., 40.)
    } else {
        Rect::new(at.x, at.y, 32., 32.)
    };
    if let Some(texture) = texture {
        icon(texture, rect);
    }
    // Each condition cicn is 16 pixels left of the last, at the box's top + 3.
    if let Some(mut left) = marks.conditions_left {
        for id in record
            .portrait_condition_icons(marks.automatic)
            .unwrap_or_default()
        {
            left -= 16.;
            if let Some(texture) = assets.icons.get(&id) {
                icon(texture, Rect::new(left, rect.y + 3., 16., 16.));
            }
        }
    }
    // 0x14a7fe: the STR# 131 class name in white, application font, bold,
    // size 10 at the box's left + 5 and bottom - 3, cut to seven characters and
    // then one more while wider than 38 pixels.
    if marks.class_label
        && let Some(name) = usize::try_from(record.profession().unwrap_or(0))
            .ok()
            .and_then(|class| {
                assets
                    .character_strings
                    .get(&131)
                    .and_then(|bytes| centurions_content::ui::string_list(bytes).ok())?
                    .get(class.checked_sub(1)?)
                    .cloned()
            })
    {
        let mut label: String = name.chars().take(7).collect();
        if measure_text(&label, Some(&assets.font), 10, 1.).width > 38. {
            label.pop();
        }
        assets.text(&label, rect.x + 5., rect.bottom() - 3., 10, WHITE);
    }
    // updatepictbox 0x161cbc: stamina below 1 adds cicn 2019 over the box grown
    // by four, and below -9 also cicn 2015 over the 32-pixel box.
    let stamina = record.stamina().unwrap_or(0);
    if stamina < 1 {
        for (id, rect) in [
            (2019, Rect::new(at.x - 4., at.y - 4., 40., 40.)),
            (2015, Rect::new(at.x, at.y, 32., 32.)),
        ] {
            if (id == 2019 || stamina < -9)
                && let Some(texture) = assets.icons.get(&id)
            {
                icon(texture, rect);
            }
        }
    }
}

fn icon(texture: &Texture2D, rect: Rect) {
    draw_texture_ex(
        texture,
        rect.x,
        rect.y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(rect.size()),
            ..Default::default()
        },
    );
}
