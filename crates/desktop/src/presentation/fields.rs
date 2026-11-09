//! The lingering spell fields on the tactical map: each field's picture over the cells of its
//! area pattern that hold open ground.
use crate::{assets::Assets, layout};
use centurions_content::{Content, records::SpellField};
use centurions_core::{Session, combat::Combat};
use macroquad::prelude::*;

/// The combat sprite picture (PICT 401).
const SPRITES: i16 = 401;

/// Copies the field picture of every field with rounds left, transparently, over each cell its
/// area pattern covers whose map tile is a building tile that does not block movement. The
/// picture is the 32-pixel cell of PICT 401 in row 64 at the field's kind.
///
/// Original: showque
pub(super) fn draw(
    state: &Session,
    content: &Content,
    combat: &Combat,
    assets: &Assets,
    (vx, vy): (i32, i32),
) {
    let Some(mission) = state.land(content) else {
        return;
    };
    let Some(texture) = assets.transparent_picture(SPRITES) else {
        return;
    };
    // The field table counts cells on the whole battle map; the view counts from the map shift.
    let origin = (
        vx + i32::from(combat.field.map_shift.0) + 5,
        vy + i32::from(combat.field.map_shift.1) + 5,
    );
    for bytes in combat.spell_queue.as_chunks::<20>().0 {
        let field = SpellField(bytes);
        if field.duration() == 0 {
            continue;
        }
        let Ok(kind) = field.kind() else { continue };
        let source_left = i32::from(kind.wrapping_sub(1)) * 32;
        if source_left < 0 || source_left + 32 > texture.width() as i32 {
            continue;
        }
        let shape = field.shape();
        let Ok(pattern) = usize::try_from(shape - 1)
            .map_err(|_| ())
            .and_then(|shape| {
                centurions_content::span(&mission.shared.areas, shape * 49, 49).map_err(|_| ())
            })
        else {
            continue;
        };
        let (center_x, center_y) = field.center();
        let (x, y) = (i32::from(center_x) - 3, i32::from(center_y) - 3);
        for row in 0..7 {
            for column in 0..7 {
                if pattern[row * 7 + column] == 0 {
                    continue;
                }
                let (cell_x, cell_y) = (x + column as i32, y + row as i32);
                let (left, top) = (cell_x - origin.0, cell_y - origin.1);
                if !(0..15).contains(&left) || !(0..13).contains(&top) {
                    continue;
                }
                if !(0..90).contains(&cell_x) || !(0..90).contains(&cell_y) {
                    continue;
                }
                let tile = combat.field.tiles[cell_x as usize * 90 + cell_y as usize];
                if tile > 999
                    && mission
                        .movement_blocker(tile - 1000)
                        .is_ok_and(|solid| solid == 0)
                {
                    draw_texture_ex(
                        &texture,
                        layout::MAP.x + left as f32 * 32.,
                        layout::MAP.y + top as f32 * 32.,
                        WHITE,
                        DrawTextureParams {
                            source: Some(Rect::new(source_left as f32, 64., 32., 32.)),
                            dest_size: Some(vec2(32., 32.)),
                            ..Default::default()
                        },
                    );
                }
            }
        }
    }
}
