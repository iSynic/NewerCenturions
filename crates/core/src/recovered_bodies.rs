//! Body display, placement, and removal recovered from ordered instructions.

use crate::{
    RuleError,
    combat::Combat,
    recovered_geometry::{BodyRect, RectField, RectWrite},
};
use centurions_content::{span, word};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BodyEffect {
    SetPort(u32),
    ForeColor(i32),
    MovePen {
        horizontal: i16,
        vertical: i16,
    },
    DrawNumber(i16),
    GlobalRect(RectWrite),
    PlotIcon2(i16),
    IconHandle(u32),
    GetColorIcon {
        resource: i16,
        handle: u32,
    },
    InsetRect {
        horizontal: i16,
        vertical: i16,
        rect: BodyRect,
    },
    PlotColorIcon {
        handle: u32,
        rect: BodyRect,
    },
    DisposeColorIcon(u32),
    FastPlot {
        tile: i16,
        rect: BodyRect,
        mode_word: i16,
        copy_word: i16,
        trailing_word: Option<i16>,
    },
    EnemyWord {
        index: i16,
        offset: usize,
        value: i16,
    },
    EnemyByte {
        index: i16,
        offset: usize,
        value: u8,
    },
    MapWord {
        index: usize,
        value: i16,
    },
    BodySize(i16),
    KilledWord(i16),
    OrderWord {
        index: usize,
        value: i16,
    },
    Center {
        body: i16,
    },
    ShowResults {
        body: i16,
        code: i16,
        attacker: i16,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BodyState {
    pub combat: Combat,
    pub native_origin: (i32, i32),
    pub enemy_count: i16,
    pub killed_word: i16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BodyPlan {
    pub state: BodyState,
    pub effects: Vec<BodyEffect>,
    pub native_return: Option<u32>,
}

pub trait ColorIconServices {
    /// Return zero for an observed nil resource handle; unavailable lookup diagnoses.
    fn get_color_icon(&self, resource: i16) -> Result<u32, RuleError>;
}

pub trait BodyServices: ColorIconServices {
    /// This service must retain center's origin and position changes.
    fn center(&self, state: &BodyState, body: i16) -> Result<BodyPlan, RuleError>;
    /// Preserve the effects of the original three-WORD showresults call.
    fn show_results(
        &self,
        state: &BodyState,
        body: i16,
        code: i16,
        attacker: i16,
    ) -> Result<BodyPlan, RuleError>;
}

fn invalid(message: &str) -> RuleError {
    RuleError::Invalid(message.into())
}

fn record(state: &BodyState, index: i16) -> Result<&[u8], RuleError> {
    let bytes = &state
        .combat
        .enemies
        .records
        .get(usize::from(index as u16))
        .ok_or_else(|| invalid("body enemy record unavailable"))?
        .bytes;
    span(bytes, 0, 288)?;
    Ok(bytes)
}

fn enemy_position(state: &BodyState, index: i16) -> Result<(i16, i16), RuleError> {
    state
        .combat
        .enemies
        .positions
        .get(usize::try_from(index).map_err(|_| invalid("body enemy position unavailable"))?)
        .copied()
        .ok_or_else(|| invalid("body enemy position unavailable"))
}

fn write_enemy_word(
    plan: &mut BodyPlan,
    index: i16,
    offset: usize,
    value: i16,
) -> Result<(), RuleError> {
    record(&plan.state, index)?;
    let bytes = &mut plan.state.combat.enemies.records[usize::from(index as u16)].bytes;
    span(bytes, offset, 2)?;
    bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
    plan.effects.push(BodyEffect::EnemyWord {
        index,
        offset,
        value,
    });
    Ok(())
}

fn write_enemy_byte(
    plan: &mut BodyPlan,
    index: i16,
    offset: usize,
    value: u8,
) -> Result<(), RuleError> {
    record(&plan.state, index)?;
    let bytes = &mut plan.state.combat.enemies.records[usize::from(index as u16)].bytes;
    span(bytes, offset, 1)?;
    bytes[offset] = value;
    plan.effects.push(BodyEffect::EnemyByte {
        index,
        offset,
        value,
    });
    Ok(())
}

fn new_plan(state: &BodyState, native_return: Option<u32>) -> BodyPlan {
    BodyPlan {
        state: state.clone(),
        effects: Vec::new(),
        native_return,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BodyGroundInput {
    pub body: i16,
    pub copy_word: i16,
    pub drawing_enabled: i16,
    pub port: u32,
}

fn tile_rect(horizontal: i16, vertical: i16) -> BodyRect {
    let top = vertical.wrapping_mul(32);
    let left = horizontal.wrapping_mul(32);
    BodyRect {
        top,
        left,
        bottom: top.wrapping_add(32),
        right: left.wrapping_add(32),
    }
}

pub fn bodyground(state: &BodyState, input: BodyGroundInput) -> Result<Vec<BodyEffect>, RuleError> {
    if input.drawing_enabled == 0 {
        return Ok(Vec::new());
    }
    let mut effects = vec![BodyEffect::SetPort(input.port)];
    if input.body <= 9 {
        let index = usize::try_from(input.body)
            .map_err(|_| invalid("body ground warrior position unavailable"))?;
        let position = *state
            .combat
            .squad
            .positions
            .get(index)
            .ok_or_else(|| invalid("body ground warrior position unavailable"))?;
        let tile = *state
            .combat
            .squad
            .under_tiles
            .get(index)
            .ok_or_else(|| invalid("body ground warrior under-tile unavailable"))?;
        effects.push(BodyEffect::FastPlot {
            tile,
            rect: tile_rect(position.0, position.1),
            mode_word: 0,
            copy_word: input.copy_word,
            trailing_word: None,
        });
    } else {
        let index = input.body.wrapping_sub(10);
        let position = enemy_position(state, index)?;
        let horizontal = i16::from(position.0.wrapping_sub(1) as i8);
        let vertical = i16::from(position.1.wrapping_sub(1) as i8);
        let bytes = record(state, index)?;
        for horizontal_offset in 0..2 {
            for vertical_offset in 0..2 {
                let offset = 0x118 + horizontal_offset as usize * 4 + vertical_offset as usize * 2;
                let tile = word(bytes, offset)?;
                if tile != 0 {
                    effects.push(BodyEffect::FastPlot {
                        tile,
                        rect: tile_rect(
                            horizontal.wrapping_add(horizontal_offset),
                            vertical.wrapping_add(vertical_offset),
                        ),
                        mode_word: 0,
                        copy_word: input.copy_word,
                        trailing_word: Some(0),
                    });
                }
            }
        }
    }
    Ok(effects)
}

fn inset(rect: BodyRect, horizontal: i16, vertical: i16) -> BodyRect {
    BodyRect {
        top: rect.top.wrapping_add(vertical),
        left: rect.left.wrapping_add(horizontal),
        bottom: rect.bottom.wrapping_sub(vertical),
        right: rect.right.wrapping_sub(horizontal),
    }
}

fn icon_lookup(
    effects: &mut Vec<BodyEffect>,
    resource: i16,
    services: &(impl ColorIconServices + ?Sized),
) -> Result<u32, RuleError> {
    effects.push(BodyEffect::IconHandle(0));
    let handle = services.get_color_icon(resource)?;
    effects.push(BodyEffect::GetColorIcon { resource, handle });
    effects.push(BodyEffect::IconHandle(handle));
    Ok(handle)
}

pub fn monsterupdate(
    state: &BodyState,
    enemy_index: i16,
    mut rect: BodyRect,
    services: &(impl ColorIconServices + ?Sized),
) -> Result<Vec<BodyEffect>, RuleError> {
    let bytes = record(state, enemy_index)?;
    let mut effects = vec![BodyEffect::ForeColor(69)];
    for (horizontal, vertical, offset) in [
        (5, 14, 0x110),
        (40, 14, 0x112),
        (5, 29, 0x10e),
        (40, 29, 0x52),
    ] {
        effects.push(BodyEffect::MovePen {
            horizontal: rect.left.wrapping_add(horizontal),
            vertical: rect.bottom.wrapping_add(vertical),
        });
        effects.push(BodyEffect::DrawNumber(word(bytes, offset)?));
    }
    if word(bytes, 0x5a)? == -1 {
        effects.extend([
            BodyEffect::GlobalRect(RectWrite {
                field: RectField::Top,
                value: rect.top,
            }),
            BodyEffect::GlobalRect(RectWrite {
                field: RectField::Left,
                value: rect.left,
            }),
            BodyEffect::GlobalRect(RectWrite {
                field: RectField::Right,
                value: rect.left.wrapping_add(64),
            }),
            BodyEffect::GlobalRect(RectWrite {
                field: RectField::Bottom,
                value: rect.top.wrapping_add(64),
            }),
            BodyEffect::PlotIcon2(2003),
        ]);
    }
    let handle = icon_lookup(&mut effects, word(bytes, 0x50)?, services)?;
    let signed_size = word(bytes, 0x10)?;
    for (condition, horizontal, vertical) in [
        (signed_size == 0, 19, 19),
        (signed_size == 2, 19, 3),
        (signed_size == 1, 3, 19),
        (signed_size.wrapping_abs() == 3, 3, 3),
    ] {
        if condition {
            rect = inset(rect, horizontal, vertical);
            effects.push(BodyEffect::InsetRect {
                horizontal,
                vertical,
                rect,
            });
        }
    }
    if handle != 0 {
        effects.push(BodyEffect::PlotColorIcon { handle, rect });
        effects.push(BodyEffect::DisposeColorIcon(handle));
    }
    Ok(effects)
}

fn placement_index(row: i16, column: i16) -> Result<usize, RuleError> {
    let offset = i64::from(u32::from(row as u16) * 180) + i64::from(column) * 2;
    usize::try_from(offset / 2).map_err(|_| invalid("body placement map address unavailable"))
}

fn map_word(state: &BodyState, index: usize) -> Result<i16, RuleError> {
    state
        .combat
        .map
        .tiles
        .get(index)
        .copied()
        .ok_or_else(|| invalid("body map word unavailable"))
}

fn write_map_word(plan: &mut BodyPlan, index: usize, value: i16) -> Result<(), RuleError> {
    *plan
        .state
        .combat
        .map
        .tiles
        .get_mut(index)
        .ok_or_else(|| invalid("body map word unavailable"))? = value;
    plan.effects.push(BodyEffect::MapWord { index, value });
    Ok(())
}

pub fn placemonster(
    state: &BodyState,
    position: (i16, i16),
    enemy_index: i16,
) -> Result<BodyPlan, RuleError> {
    let size = word(record(state, enemy_index)?, 0x10)?.wrapping_abs();
    let row = position.0.wrapping_add(state.native_origin.0 as i16);
    let column = position.1.wrapping_add(state.native_origin.1 as i16);
    let body = enemy_index.wrapping_add(10);
    let mut plan = new_plan(state, None);
    plan.effects.push(BodyEffect::BodySize(size));
    for (row_offset, column_offset, offset, occupied) in [
        (0, 0, 0x11e, true),
        (-1, -1, 0x118, size == 3),
        (0, -1, 0x11c, size == 1 || size == 3),
        (-1, 0, 0x11a, size > 1),
    ] {
        if occupied {
            let cell = placement_index(
                row.wrapping_add(row_offset),
                column.wrapping_add(column_offset),
            )?;
            let under = map_word(&plan.state, cell)?;
            write_enemy_word(&mut plan, enemy_index, offset, under)?;
            write_map_word(
                &mut plan,
                cell,
                if offset == 0x11e {
                    body
                } else {
                    body.wrapping_neg()
                },
            )?;
        } else {
            write_enemy_word(&mut plan, enemy_index, offset, 0)?;
        }
    }
    Ok(plan)
}

fn restoration_index(
    state: &BodyState,
    position: (i16, i16),
    first_offset: i32,
    second_offset: i32,
) -> Result<usize, RuleError> {
    let row = state
        .native_origin
        .0
        .wrapping_add(i32::from(position.0))
        .wrapping_add(first_offset);
    let column = state
        .native_origin
        .1
        .wrapping_add(i32::from(position.1))
        .wrapping_add(second_offset);
    let address = row.wrapping_mul(180).wrapping_add(column.wrapping_mul(2));
    usize::try_from(address / 2).map_err(|_| invalid("body restoration map address unavailable"))
}

fn restore_enemy(plan: &mut BodyPlan, enemy_index: i16) -> Result<(), RuleError> {
    let signed_size = word(record(&plan.state, enemy_index)?, 0x10)?;
    let size = signed_size.wrapping_abs();
    let position = enemy_position(&plan.state, enemy_index)?;
    for (first, second, offset, occupied) in [
        (-1, -1, 0x118, size == 3),
        (-1, 0, 0x11a, size > 1),
        (0, -1, 0x11c, size == 3 || signed_size == 1),
        (0, 0, 0x11e, true),
    ] {
        if occupied {
            let value = word(record(&plan.state, enemy_index)?, offset)?;
            let cell = restoration_index(&plan.state, position, first, second)?;
            write_map_word(plan, cell, value)?;
        }
    }
    Ok(())
}

pub fn destroymonster(
    state: &BodyState,
    enemy_index: i16,
    display: BodyGroundInput,
    services: &(impl BodyServices + ?Sized),
) -> Result<BodyPlan, RuleError> {
    let body = enemy_index.wrapping_add(10);
    let mut plan = new_plan(state, Some(0));
    write_enemy_word(&mut plan, enemy_index, 0x110, 1)?;
    plan.effects.push(BodyEffect::Center { body });
    let centered = services.center(&plan.state, body)?;
    plan.state = centered.state;
    plan.effects.extend(centered.effects);
    write_enemy_word(&mut plan, enemy_index, 0x110, 0)?;
    let size = word(record(&plan.state, enemy_index)?, 0x10)?.wrapping_abs();
    let position = enemy_position(&plan.state, enemy_index)?;
    let mut rect = tile_rect(position.0, position.1);
    let bytes = record(&plan.state, enemy_index)?;
    if word(bytes, 0x0e)? != 0 && bytes[0xb0] != 0xff {
        plan.state.killed_word = plan.state.killed_word.wrapping_add(1);
        plan.effects
            .push(BodyEffect::KilledWord(plan.state.killed_word));
    }
    write_enemy_word(&mut plan, enemy_index, 0x0e, 0)?;
    write_enemy_byte(&mut plan, enemy_index, 0x106, 0)?;
    for offset in 0xb2..0xda {
        write_enemy_byte(&mut plan, enemy_index, offset, 0)?;
    }
    if size == 1 || size == 3 {
        rect.top = rect.top.wrapping_sub(32);
    }
    if size > 1 {
        rect.left = rect.left.wrapping_sub(32);
    }
    plan.effects.push(BodyEffect::SetPort(display.port));
    let handle = icon_lookup(&mut plan.effects, size.wrapping_add(2015), services)?;
    if handle != 0 {
        plan.effects
            .push(BodyEffect::PlotColorIcon { handle, rect });
        plan.effects.push(BodyEffect::DisposeColorIcon(handle));
    }
    for index in 1..110 {
        let entry = plan
            .state
            .combat
            .order
            .get_mut(index)
            .ok_or_else(|| invalid("body removal order word unavailable"))?;
        if *entry == body {
            *entry = -1;
            plan.effects
                .push(BodyEffect::OrderWord { index, value: -1 });
        }
    }
    let code = if record(&plan.state, enemy_index)?[0xab] == 0 {
        -2
    } else {
        -14
    };
    plan.effects.push(BodyEffect::ShowResults {
        body,
        code,
        attacker: body,
    });
    let results = services.show_results(&plan.state, body, code, body)?;
    plan.state = results.state;
    plan.effects.extend(results.effects);
    plan.effects.extend(bodyground(
        &plan.state,
        BodyGroundInput {
            body,
            copy_word: 0,
            ..display
        },
    )?);
    restore_enemy(&mut plan, enemy_index)?;
    for index in 0..plan.state.enemy_count {
        if word(record(&plan.state, index)?, 0x10c)? == enemy_index {
            write_enemy_word(&mut plan, index, 0x10c, -1)?;
        }
    }
    Ok(plan)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::{EnemySetup, SquadPlacement, TacticalMap};
    use centurions_content::combat::MonsterRecord;

    fn state() -> BodyState {
        BodyState {
            combat: Combat {
                battle: 1,
                map: TacticalMap {
                    tiles: (0..8100).map(|i| i as i16).collect(),
                    edge_word: 0,
                    shift: (0, 0),
                },
                squad: SquadPlacement {
                    positions: vec![(0, 0)],
                    under_tiles: vec![1001],
                },
                enemies: EnemySetup {
                    records: vec![MonsterRecord {
                        bytes: vec![0; 288],
                    }],
                    positions: vec![(5, 5)],
                },
                order: vec![-1; 110],
                cursor: 0,
                round: 0,
                round_scripts: Default::default(),
                death_queue: Default::default(),
                spell_fields: vec![0; 1200],
                field_used: vec![0; 128],
                attack_packet: None,
                attack_identity: None,
                weapon_work: Default::default(),
                unavailable: 0,
            },
            native_origin: (0, 0),
            enemy_count: 1,
            killed_word: 32767,
        }
    }

    fn put(state: &mut BodyState, offset: usize, value: i16) {
        state.combat.enemies.records[0].bytes[offset..offset + 2]
            .copy_from_slice(&value.to_be_bytes());
    }

    struct Services;
    impl ColorIconServices for Services {
        fn get_color_icon(&self, _: i16) -> Result<u32, RuleError> {
            Ok(123)
        }
    }
    impl BodyServices for Services {
        fn center(&self, state: &BodyState, body: i16) -> Result<BodyPlan, RuleError> {
            assert_eq!(body, 10);
            assert_eq!(word(record(state, 0)?, 0x110)?, 1);
            Ok(new_plan(state, Some(0)))
        }
        fn show_results(
            &self,
            state: &BodyState,
            body: i16,
            code: i16,
            attacker: i16,
        ) -> Result<BodyPlan, RuleError> {
            assert_eq!((body, code, attacker), (10, -14, 10));
            assert_eq!(word(record(state, 0)?, 0x110)?, 0);
            Ok(new_plan(state, Some(0)))
        }
    }

    #[test]
    fn placement_uses_origin_low_words_and_retains_large_size_layout() {
        let mut state = state();
        state.native_origin = (65536, 65536);
        put(&mut state, 0x10, 4);
        let plan = placemonster(&state, (2, 2), 0).unwrap();
        assert_eq!(word(record(&plan.state, 0).unwrap(), 0x11e).unwrap(), 182);
        assert_eq!(word(record(&plan.state, 0).unwrap(), 0x11a).unwrap(), 92);
        assert_eq!(plan.state.combat.map.tiles[182], 10);
        assert_eq!(plan.state.combat.map.tiles[92], -10);
        assert_eq!(word(record(&plan.state, 0).unwrap(), 0x118).unwrap(), 0);
        assert_eq!(
            plan.state.combat.enemies.positions,
            state.combat.enemies.positions
        );
    }

    #[test]
    fn enemy_ground_narrows_positions_to_bytes_and_preserves_plot_order() {
        let mut state = state();
        state.combat.enemies.positions[0] = (130, 129);
        for (offset, value) in [(0x118, 11), (0x11a, 12), (0x11c, 13), (0x11e, 14)] {
            put(&mut state, offset, value);
        }
        let input = BodyGroundInput {
            body: 10,
            copy_word: 257,
            drawing_enabled: -1,
            port: 99,
        };
        let effects = bodyground(&state, input).unwrap();
        assert_eq!(effects[0], BodyEffect::SetPort(99));
        assert!(matches!(
            effects[1],
            BodyEffect::FastPlot {
                tile: 11,
                rect: BodyRect {
                    top: -4096,
                    left: -4064,
                    ..
                },
                copy_word: 257,
                trailing_word: Some(0),
                ..
            }
        ));
        assert!(matches!(effects[2], BodyEffect::FastPlot { tile: 12, .. }));
        assert!(matches!(effects[3], BodyEffect::FastPlot { tile: 13, .. }));
        assert!(matches!(effects[4], BodyEffect::FastPlot { tile: 14, .. }));
        assert!(
            bodyground(
                &state,
                BodyGroundInput {
                    drawing_enabled: 0,
                    ..input
                }
            )
            .unwrap()
            .is_empty()
        );
    }

    #[test]
    fn monster_update_uses_the_full_rect_and_signed_inset_cases() {
        let mut state = state();
        put(&mut state, 0x10, -2);
        put(&mut state, 0x50, 2007);
        put(&mut state, 0x5a, -1);
        put(&mut state, 0x110, 17);
        let rect = BodyRect {
            top: 100,
            left: 200,
            bottom: 300,
            right: 400,
        };
        let effects = monsterupdate(&state, 0, rect, &Services).unwrap();
        assert_eq!(effects[0], BodyEffect::ForeColor(69));
        assert_eq!(
            effects[1],
            BodyEffect::MovePen {
                horizontal: 205,
                vertical: 314
            }
        );
        assert_eq!(effects[2], BodyEffect::DrawNumber(17));
        assert!(effects.contains(&BodyEffect::GlobalRect(RectWrite {
            field: RectField::Right,
            value: 264
        })));
        assert!(effects.contains(&BodyEffect::PlotColorIcon { handle: 123, rect }));
        assert!(
            !effects
                .iter()
                .any(|effect| matches!(effect, BodyEffect::InsetRect { .. }))
        );
        put(&mut state, 0x10, -3);
        let effects = monsterupdate(&state, 0, rect, &Services).unwrap();
        assert!(effects.contains(&BodyEffect::PlotColorIcon {
            handle: 123,
            rect: BodyRect {
                top: 103,
                left: 203,
                bottom: 297,
                right: 397
            }
        }));
    }

    #[test]
    fn destroy_wraps_killed_word_before_clearing_and_preserves_order_zero() {
        let mut state = state();
        put(&mut state, 0xe, 1);
        put(&mut state, 0x10, -2);
        put(&mut state, 0x11a, 1002);
        put(&mut state, 0x11e, 1003);
        state.combat.enemies.records[0].bytes[0xab] = 1;
        state.combat.enemies.records[0].bytes[0xb2..0xda].fill(1);
        state.combat.enemies.records[0].bytes[0x106] = 255;
        state.combat.order[0] = 10;
        state.combat.order[1] = 10;
        state.combat.order[109] = 10;
        let original = state.clone();
        let plan = destroymonster(
            &state,
            0,
            BodyGroundInput {
                body: 0,
                copy_word: 99,
                drawing_enabled: 1,
                port: 88,
            },
            &Services,
        )
        .unwrap();
        assert_eq!(state, original);
        assert_eq!(plan.state.killed_word, -32768);
        assert_eq!(word(record(&plan.state, 0).unwrap(), 0xe).unwrap(), 0);
        assert_eq!(plan.state.combat.order[0], 10);
        assert_eq!(plan.state.combat.order[1], -1);
        assert_eq!(plan.state.combat.order[109], -1);
        assert_eq!(plan.state.combat.map.tiles[4 * 90 + 5], 1002);
        assert_eq!(plan.state.combat.map.tiles[5 * 90 + 5], 1003);
        assert_eq!(word(record(&plan.state, 0).unwrap(), 0x10c).unwrap(), -1);
        let count = plan
            .effects
            .iter()
            .position(|e| *e == BodyEffect::KilledWord(-32768))
            .unwrap();
        let faction = plan
            .effects
            .iter()
            .position(|e| {
                matches!(
                    e,
                    BodyEffect::EnemyWord {
                        offset: 0xe,
                        value: 0,
                        ..
                    }
                )
            })
            .unwrap();
        assert!(count < faction);
        assert!(
            record(&plan.state, 0).unwrap()[0xb2..0xda]
                .iter()
                .all(|b| *b == 0)
        );
    }
}
