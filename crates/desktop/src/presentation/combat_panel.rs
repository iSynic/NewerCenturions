//! The combat information panel under the map: the body in view with its icon,
//! name, weapon, statistics, charge and first condition, the readouts in the
//! message area, and a monster's immunity and vulnerability pips.
//!
//! Text positions are the game's own: the vertical positions of the lines are
//! offsets from the panel origin, which sits `ORIGIN` pixels down in this window.
use super::bodies::enemy_icon;
use crate::{
    assets::Assets,
    game_strings,
    layout::{self, POOL_BELOW_MAXIMUM},
    presentation::UiState,
};
use centurions_content::{
    Content, Warrior,
    records::{EnemyRecord, ItemDefinition, WarriorRecord},
};
use centurions_core::{
    Session,
    combat::{Combat, player_psi},
};
use macroquad::prelude::*;

/// The vertical position of the panel origin: the game window's origin below the
/// menu bar, less the 32 pixels the 800 x 600 window shifts the lines up.
const ORIGIN: f32 = 128.;
/// The left edge of the message area's text: the window's origin plus the offset of
/// the readouts.
const READOUT_ORIGIN: f32 = 160.;
/// The item the panel shows when a body holds no weapon.
const NO_WEAPON_ICON: i16 = 7002;
/// The entry of the warnings list that names the attack of a body with no weapon.
const NO_WEAPON_TEXT: usize = 3;

/// Draws the information panels under the map (PICT 166 and 228) for the body in
/// view, and the message area's readout: the range to that body while the player
/// has centred or stepped to it, otherwise the monsters still to defeat. Nothing
/// is read out while a scan or item is being aimed.
///
/// Original: combatupdate2
pub(super) fn draw_combat_information(
    content: &Content,
    state: &Session,
    assets: &Assets,
    ui: &UiState,
    body: i16,
) {
    let Some(combat) = state.combat.as_ref() else {
        return;
    };
    if !casting(state, ui) {
        if ui.combat_view.information(state) {
            draw_range(content, state, combat, assets, body);
        } else {
            draw_enemies_left(combat, assets);
        }
    }
    assets.panel(166, Rect::new(0., 449., 308., 139.));
    assets.panel(crate::layout::STRIP, Rect::new(0., 436., 480., 11.));
    assets.panel(228, Rect::new(310., 449., 160., 140.));
    if !crate::acting::is_warrior_body(body) {
        let Some(monster) = combat.roster.records.get((body - 10) as usize) else {
            return;
        };
        draw_monster(content, assets, ui, monster.record());
        return;
    }
    if let Some(warrior) = state.warriors.get(body as usize) {
        draw_warrior(content, assets, ui, warrior);
    }
}

/// Whether a scan or item is being aimed or its effect is playing, when the message
/// area belongs to the spell.
fn casting(state: &Session, ui: &UiState) -> bool {
    crate::targeting::active(state, ui)
        || ui
            .playback
            .as_ref()
            .is_some_and(|p| p.cast_icon.is_some() || p.spell_info.is_some())
}

/// The text the game keeps at `address`, empty when it cannot be read.
fn game_text(assets: &Assets, address: u32) -> String {
    assets.game_text.text(address, true).unwrap_or_default()
}

/// Draws the count of monsters still to defeat in the message area, in bold
/// (QuickDraw's bold repeats each glyph one pixel to the right).
///
/// Original: number
fn draw_enemies_left(combat: &Combat, assets: &Assets) {
    let y = ORIGIN + 445.;
    let label = game_text(assets, game_strings::ENEMIES_LEFT);
    let count = combat.kill_counts.remaining().to_string();
    for smear in [0., 1.] {
        assets.text(&label, READOUT_ORIGIN + 430. + smear, y, 20, YELLOW);
        assets.text(&count, READOUT_ORIGIN + 580. + smear, y, 20, YELLOW);
    }
}

/// Draws the distance from the acting warrior to the body in view, followed by
/// "Blocked" when the warrior cannot see it.
///
/// Original: flashrange
fn draw_range(content: &Content, state: &Session, combat: &Combat, assets: &Assets, body: i16) {
    let Some(actor) = crate::acting::active_warrior(state) else {
        return;
    };
    let Some(&from) = combat.squad.pos.get(actor as usize) else {
        return;
    };
    let to = if crate::acting::is_warrior_body(body) {
        usize::try_from(body)
            .ok()
            .and_then(|slot| combat.squad.pos.get(slot))
    } else {
        combat.roster.mon_pos.get((body - 10) as usize)
    };
    let Some(&to) = to else { return };
    let (x, y) = (READOUT_ORIGIN + 425., ORIGIN + 440.);
    let label = game_text(assets, game_strings::RANGE_LABEL);
    let distance = player_psi::cell_distance(from, to).to_string();
    assets.text(&label, x, y, 18, layout::TOOLBOX_GREEN);
    let x = x + assets.text_width(&label, 18);
    assets.text(&distance, x, y, 18, layout::TOOLBOX_GREEN);
    let x = x + assets.text_width(&distance, 18);
    if !state.sight_clear(content, from, to) {
        let blocked = game_text(assets, game_strings::BLOCKED);
        assets.text(&blocked, x, y, 18, layout::TOOLBOX_GREEN);
    }
}

/// Draws `text` with its baseline `row` below the panel origin.
fn put(assets: &Assets, text: &str, x: f32, row: f32, color: Color) {
    assets.text(text, x, ORIGIN + row, 12, color);
}

/// Half of a count, rounded toward zero.
fn half(value: i16) -> i16 {
    value.wrapping_add(((value as u16) >> 15) as i16) >> 1
}

/// The figures the panel shows for a body besides its name and weapon.
struct Figures {
    stamina: i16,
    stamina_max: i16,
    psi_points: i16,
    psi_points_max: i16,
    movement: i16,
    /// Whether the movement is below the figure the game compares it with: the maximum
    /// movement against the figure the panel last left behind, which for a monster is the
    /// icon drawn for it.
    movement_below_maximum: bool,
    action_points: i16,
    shield_chance: i16,
    shield_points: i16,
    armor_rating: i16,
    /// The attacks and scans the body has left this round.
    attacks: i16,
    scans: i16,
}

impl Figures {
    /// A warrior's figures; the attacks and scans are held to half the action points, and the
    /// movement is compared with `scratch`, the figure the panel left behind before this draw.
    fn warrior(record: WarriorRecord<'_>, scratch: i16) -> Self {
        let value = |figure: Result<i16, _>| figure.unwrap_or(0);
        let action_points = value(record.action_points());
        Self {
            stamina: value(record.stamina()),
            stamina_max: value(record.stamina_max()),
            psi_points: value(record.psi_points()),
            psi_points_max: value(record.psi_points_max()),
            movement: value(record.movement()),
            movement_below_maximum: scratch < value(record.movement_max()),
            action_points,
            shield_chance: value(record.shield_chance()),
            shield_points: value(record.shield_points()),
            armor_rating: value(record.armor_rating()),
            attacks: value(record.attack_limit()).min(half(action_points)),
            scans: value(record.max_spells_attacks()).min(half(action_points)),
        }
    }

    /// A monster's figures; its attacks are half its attack count and its scans its
    /// scan count, neither held to its action points.
    fn monster(record: &EnemyRecord<'_>, icon: i16) -> Self {
        let value = |figure: Result<i16, _>| figure.unwrap_or(0);
        Self {
            stamina: value(record.stamina()),
            stamina_max: value(record.stamina_max()),
            psi_points: value(record.psi_points()),
            psi_points_max: value(record.psi_points_max()),
            movement: i16::from(record.movement()),
            movement_below_maximum: icon < value(record.movement_max()),
            action_points: i16::from(record.action_points()),
            shield_chance: value(record.shield_chance()),
            shield_points: value(record.shield_points()),
            armor_rating: value(record.armor_rating()),
            attacks: half(i16::from(record.attack_count())),
            scans: i16::from(record.magic_attack_count()),
        }
    }

    /// Draws the figures; stamina and psi points below their maximum take the
    /// colour a pool below its maximum has, as does a movement whose compared figure is
    /// below the maximum movement, and movement is never drawn below zero.
    ///
    /// Original: combatupdate
    fn draw(&self, assets: &Assets) {
        let pool = |value: i16, maximum: i16| {
            if value < maximum {
                POOL_BELOW_MAXIMUM
            } else {
                YELLOW
            }
        };
        let figure = |value: i16, x: f32, row: f32, color: Color| {
            put(assets, &value.to_string(), x, row, color);
        };
        figure(
            self.stamina,
            112.,
            344.,
            pool(self.stamina, self.stamina_max),
        );
        figure(
            self.psi_points,
            112.,
            367.,
            pool(self.psi_points, self.psi_points_max),
        );
        figure(
            self.movement.max(0),
            112.,
            390.,
            if self.movement_below_maximum {
                POOL_BELOW_MAXIMUM
            } else {
                YELLOW
            },
        );
        figure(self.action_points, 212., 389., YELLOW);
        figure(self.shield_chance, 172., 344., YELLOW);
        figure(self.shield_points, 217., 344., YELLOW);
        figure(self.armor_rating, 276., 344., YELLOW);
        figure(self.attacks, 257., 389., YELLOW);
        figure(self.scans, 291., 389., YELLOW);
    }
}

/// Draws a warrior: icon, figures, weapon, name, charge and first condition. Once the figures
/// are drawn the panel keeps the weapon number as the figure the next movement is compared with.
///
/// Original: combatupdate2
fn draw_warrior(content: &Content, assets: &Assets, ui: &UiState, warrior: &Warrior) {
    let record = warrior.record();
    Figures::warrior(warrior.record(), ui.panel_scratch.get()).draw(assets);
    if let Some(texture) = record.icon().ok().and_then(|icon| assets.icons.get(&icon)) {
        crate::assets::texture_in(texture, Rect::new(24., ORIGIN + 344., 32., 32.));
    }
    let firearm = record.toggle();
    let weapon = if firearm {
        record.equipped_weapon()
    } else {
        record.melee_weapon()
    }
    .unwrap_or(0);
    ui.panel_scratch.set(weapon);
    // The weapon's name is listed only while the warrior has a weapon shown with it.
    let named = if firearm {
        record.displayed_weapon(true).unwrap_or(0) != 0
    } else {
        record.displayed_weapon(false).unwrap_or(0) > -1
    };
    let shots = if firearm && weapon != 0 {
        weapon_attacks(content, weapon)
    } else {
        1
    };
    draw_weapon(content, assets, weapon, named, shots);
    put(assets, &warrior.name, 128., 411., YELLOW);
    let charge = warrior
        .inventory()
        .unwrap_or_default()
        .iter()
        .find(|item| item.id() == weapon)
        .and_then(|item| item.charge().ok())
        .filter(|charge| *charge >= 1);
    put(
        assets,
        &charge.map_or_else(
            || game_text(assets, game_strings::NO_CHARGE),
            |charge| charge.to_string(),
        ),
        73.,
        453.,
        YELLOW,
    );
    let condition = (0..40).find(|index| record.condition_duration(*index).unwrap_or(0) != 0);
    draw_condition(assets, condition);
}

/// Draws a monster: icon, pips, figures, weapon, name, charge and first condition. The icon
/// number is kept as the figure the movement is compared with.
///
/// Original: combatupdate2
fn draw_monster(content: &Content, assets: &Assets, ui: &UiState, record: EnemyRecord<'_>) {
    draw_pips(assets, &record);
    let size = record.size().unwrap_or(0);
    let icon = record.icon().unwrap_or(0);
    let icon = if size == -3 {
        icon
    } else {
        enemy_icon(record.facing_left(), icon)
    };
    // A size one or three monster reaches a cell north and a size above one a cell west.
    let mut rect = Rect::new(24., ORIGIN + 344., 32., 32.);
    if matches!(size.abs(), 1 | 3) {
        rect.y -= 16.;
        rect.h += 32.;
    }
    if size.abs() > 1 {
        rect.x -= 16.;
        rect.w += 32.;
    }
    if let Some(texture) = assets.icons.get(&icon) {
        crate::assets::texture_in(texture, rect);
    }
    ui.panel_scratch.set(icon);
    Figures::monster(&record, icon).draw(assets);
    let weapon = record.weapon().unwrap_or(0);
    let shots = if weapon != 0 {
        weapon_attacks(content, weapon)
    } else {
        1
    };
    draw_weapon(content, assets, weapon, true, shots);
    put(
        assets,
        &record.name().unwrap_or_default(),
        128.,
        411.,
        YELLOW,
    );
    let charge = if weapon != 0 {
        game_strings::UNKNOWN_CHARGE
    } else {
        game_strings::NO_CHARGE
    };
    put(assets, &game_text(assets, charge), 73., 453., YELLOW);
    let condition = (0..40).find(|index| record.condition_duration(*index) != 0);
    draw_condition(assets, condition);
}

/// The attacks an item makes in one use.
fn weapon_attacks(content: &Content, item: i16) -> i16 {
    content
        .items
        .get(item.unsigned_abs() as usize)
        .and_then(|definition| ItemDefinition(definition).attack_count().ok())
        .unwrap_or(0)
}

/// Draws the weapon in hand: its icon, its name beside the icon and the attacks it
/// makes. A body with no weapon shows the no-weapon icon and the physical attack.
fn draw_weapon(content: &Content, assets: &Assets, weapon: i16, named: bool, shots: i16) {
    let (icon, name) = if weapon == 0 {
        (
            Some(NO_WEAPON_ICON),
            assets
                .warnings
                .get(NO_WEAPON_TEXT)
                .cloned()
                .unwrap_or_default(),
        )
    } else {
        (
            content
                .items
                .get(weapon.unsigned_abs() as usize)
                .and_then(|definition| ItemDefinition(definition).icon().ok()),
            if named {
                assets.name(weapon)
            } else {
                String::new()
            },
        )
    };
    put(assets, &name, 128., 432., YELLOW);
    put(assets, &shots.to_string(), 64., 430., YELLOW);
    if let Some(texture) = icon.and_then(|icon| assets.icons.get(&icon)) {
        crate::assets::texture_in(texture, Rect::new(8., ORIGIN + 399., 32., 32.));
    }
}

/// Draws the name of the first condition the body is under.
fn draw_condition(assets: &Assets, condition: Option<usize>) {
    if let Some(name) = condition.and_then(|index| assets.conditions.get(index)) {
        put(assets, name, 117., 453., YELLOW);
    }
}

/// Draws a monster's seven immunity pips and seven vulnerability pips: the first
/// column marks a saving throw above 100 and the second one below zero.
fn draw_pips(assets: &Assets, record: &EnemyRecord<'_>) {
    for (left, vulnerable) in [(339., false), (426., true)] {
        for category in 1..8usize {
            let value = record.save(category);
            let marked = if vulnerable { value < 0 } else { value > 100 };
            assets.panel(
                if marked { 150 } else { 146 },
                Rect::new(left, 472. + (category - 1) as f32 * 16., 9., 9.),
            );
        }
    }
}
