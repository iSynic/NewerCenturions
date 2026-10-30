use centurions_core::Command;
use macroquad::prelude::*;
use std::collections::VecDeque;

/// The lists and pickers that open over the game: items, a crate, firearm and
/// scan targets, weapons, conditions, modifiers, specials, scans and bandaging.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Inspection {
    Items,
    Crate,
    FireTargets,
    Weapons,
    Conditions,
    Modifiers,
    Specials,
    Psi,
    PsiTargets,
    Bandage,
}
/// What the screen keeps between frames: the open inspection, the selected
/// warrior, pending messages, movement and targeting, and the combat playback.
#[derive(Default)]
pub struct UiState {
    pub inventory: crate::inventory::Inventory,
    pub collection: crate::collection::Selection,
    pub combat_view: crate::combat_controls::View,
    pub preferences: crate::preferences::Preferences,
    pub feedback: crate::feedback::Feedback,
    pub help_request: Option<(usize, usize)>,
    pub technical_error: Option<String>,
    pub diagnostic_path: Option<std::path::PathBuf>,
    pub movement: crate::input::MovementInput,
    pub field_repeat: Option<(centurions_core::field_controls::Action, f64)>,
    pub field_repeating: bool,
    pub selected: usize,
    pub inspection: Option<Inspection>,
    pub return_information: Option<usize>,
    pub page: usize,
    pub typed_text: String,
    pub typed_entry: bool,
    pub effects: VecDeque<OutdoorAnimation>,
    /// The NPC steps and result pauses of the last combat command still being shown.
    pub playback: Option<crate::combat_playback::Playback>,
    /// The PICT newland27 left over the map window.
    pub picture: Option<i16>,
    pub delay_until: f64,
    pub psi: crate::scan_picker::PsiSelection,
    pub targeting: crate::targeting::Targeting,
    /// The logical pointer while the left button is held, for downbutton's look.
    pub held_pointer: Option<Vec2>,
}
/// An outdoor spell effect playing at a map cell: its cicn base, place, size and
/// frame timing.
pub struct OutdoorAnimation {
    pub effect: i16,
    pub x: i16,
    pub y: i16,
    pub size: i16,
    pub started: f64,
    pub frame_seconds: f64,
}
/// What a click or key can ask for: a rule command, a choice in a screen of the
/// game, opening or closing a list, saving or loading.
#[derive(Clone)]
pub enum Action {
    Inventory(crate::inventory::InventoryAction),
    Collection(crate::collection::CollectionAction),
    CombatView(crate::combat_controls::ViewAction),
    PanCombat(i16, i16),
    Application(crate::application::ApplicationAction),
    Command(Command),
    Save,
    FastSave,
    Load,
    Select(usize),
    Open(Inspection),
    InformationInspection(usize, Inspection),
    Close,
    Page(i32),
    ChoosePsi(i16),
    PsiPower(i16),
    RotatePsi,
    TargetQuantity(bool),
    MedicalCancel,
    TextEntry(bool),
    PreviewPsi(i16),
    KeepMedicalTarget,
    Drop(usize, usize),
}

/// A clickable rectangle on the screen, with the action it carries out and
/// whether it is available.
pub struct Control {
    pub rect: Rect,
    pub action: Action,
    pub enabled: bool,
}

/// Which end of the pushed control list a pointer lookup starts from.
#[derive(Clone, Copy)]
pub enum HitOrder {
    /// The earliest pushed control wins.
    First,
    /// The latest pushed (topmost) control wins.
    Topmost,
}

/// The action of the control under `point` that `accepts` admits, searched in `order`.
pub fn clicked(
    controls: &[Control],
    point: Vec2,
    order: HitOrder,
    accepts: impl Fn(&Control) -> bool,
) -> Option<Action> {
    let hit = |control: &&Control| accepts(control) && control.rect.contains(point);
    match order {
        HitOrder::First => controls.iter().find(hit),
        HitOrder::Topmost => controls.iter().rev().find(hit),
    }
    .map(|control| control.action.clone())
}

/// The topmost control under `point`, enabled or not.
pub fn topmost(controls: &[Control], point: Vec2) -> Option<&Control> {
    controls
        .iter()
        .rev()
        .find(|control| control.rect.contains(point))
}

/// Draws picture `id` in `rect` and pushes the control that answers clicks there.
pub fn push_panel_control(
    assets: &crate::assets::Assets,
    controls: &mut Vec<Control>,
    id: i16,
    rect: Rect,
    enabled: bool,
    action: Action,
) {
    assets.panel(id, rect);
    controls.push(Control {
        rect,
        action,
        enabled,
    });
}
mod controls;
pub(crate) use controls::authored_text;
use controls::{button, field};

mod panels;
pub use panels::draw_specials;
use panels::{draw_clinic, draw_combat_information, draw_survivors, equipped_ammunition_reserve};

mod state;

mod bevel;
mod bodies;
mod flights;
mod map;
mod range_lines;
use map::{draw_outdoor_effect, draw_story_picture, map};

mod scene;
mod spell_panel;
pub use scene::draw;
pub(crate) use spell_panel::draw_info;

mod hud;
