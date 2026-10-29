use centurions_core::Command;
use macroquad::prelude::*;
use std::collections::VecDeque;

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
#[derive(Default)]
pub struct UiState {
    pub inventory: crate::inventory::Inventory,
    pub collection_slot: Option<usize>,
    pub collection_drag: bool,
    pub collection_drop: Option<usize>,
    pub collection_drop_scroll: usize,
    pub combat_view: crate::combat_controls::View,
    pub preferences: crate::preferences::Preferences,
    pub feedback: VecDeque<centurions_core::feedback::NativeFeedback>,
    pub persistent_feedback: Option<centurions_core::feedback::NativeFeedback>,
    pub feedback_started: Option<f64>,
    pub feedback_until: f64,
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
    pub psi_power: i16,
    pub psi_choice: Option<(i16, i16)>,
    pub psi_pending: Option<(i16, i16)>,
    pub psi_rotation: u8,
    pub psi_preview: Option<(i16, i16)>,
    pub firearm_map_target: bool,
    pub estimate_target: Option<i16>,
    pub estimate: std::cell::RefCell<Option<crate::targeting::EstimateCache>>,
    pub medical_cancel: bool,
    /// The logical pointer while the left button is held, for downbutton's look.
    pub held_pointer: Option<Vec2>,
}
pub struct OutdoorAnimation {
    pub effect: i16,
    pub x: i16,
    pub y: i16,
    pub size: i16,
    pub started: f64,
    pub frame_seconds: f64,
}
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

pub struct Control {
    pub rect: Rect,
    pub action: Action,
    pub enabled: bool,
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
