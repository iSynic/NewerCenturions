use centurions_core::RuleError;
use centurions_core::recovered_attack::DamageFrame;
use centurions_core::recovered_audio::{AudioContext, AudioObservation};
use centurions_core::recovered_bodies::BodyState;
use centurions_core::recovered_character_panels::{
    CharacterPanelObservation, CharacterPanelRequest,
};
use centurions_core::recovered_character_ui::{CharacterUiObservation, CharacterUiRequest};
use centurions_core::recovered_combat_panel::{CombatPanelObservation, CombatPanelRequest};
use centurions_core::recovered_combat_results::GraphicsDispatchRequest;
use centurions_core::recovered_combat_ui::{CombatUiMemory, CombatUiObservation, CombatUiRequest};
use centurions_core::recovered_detail_ui::{DetailReply, DetailRequest};
use centurions_core::recovered_drawing::{BuildingWorld, PictureObservation};
use centurions_core::recovered_feedback::{FeedbackMemory, FeedbackObservation, FeedbackRequest};
use centurions_core::recovered_files::{FileContext, FileObservation};
use centurions_core::recovered_gamma::{GammaError, GammaMemory, GammaObservation, GammaRequest};
use centurions_core::recovered_gamma_control::GammaControlRequest;
use centurions_core::recovered_geometry::BodyRect;
use centurions_core::recovered_install::InstallMemory;
use centurions_core::recovered_interface::Point;
use centurions_core::recovered_inventory_edit::{InventoryEditObservation, InventoryEditRequest};
use centurions_core::recovered_inventory_rules::SaneRequest;
use centurions_core::recovered_item_ui::{ItemUiReply, ItemUiRequest};
use centurions_core::recovered_load_control::{LoadObservation, LoadRequest, VersionInputs};
use centurions_core::recovered_prompt_control::{LibraryObservation, LibraryRequest};
use centurions_core::recovered_records::RecordWrite;
use centurions_core::recovered_rendering::{
    RenderingMemory, RenderingObservation, RenderingRequest,
};
use centurions_core::recovered_runtime::{RuntimeContext, RuntimeObservation};
use centurions_core::recovered_spell_ui::DialogItemObservation;
use centurions_core::recovered_startup::{PreferenceInputs, StartupObservation, StartupRequest};
use centurions_core::recovered_status_panels::{StatusPanelObservation, StatusPanelRequest};
use centurions_core::recovered_storage::{StorageObservation, StorageRequest};
use centurions_core::recovered_text::PascalString;
use centurions_core::recovered_world_display::{WorldDisplayObservation, WorldDisplayRequest};
pub struct NoServices;

macro_rules! unavailable {
    ($trait:path { $(fn $name:ident($($args:tt)*) -> $result:ty;)* }) => {
        impl $trait for NoServices {
            $(fn $name($($args)*) -> $result {
                panic!("unexpected service call: {}", stringify!($name))
            })*
        }
    };
}
unavailable! {
    centurions_core::recovered_rendering::RenderingServices {
        fn observe(
            &self,
            _: &RenderingRequest,
            _: &RenderingMemory,
        ) -> Result<RenderingObservation, RuleError>;
    }
}

unavailable! {
    centurions_core::recovered_feedback::FeedbackServices {
        fn observe(
            &self,
            _: &FeedbackRequest,
            _: &FeedbackMemory,
        ) -> Result<FeedbackObservation, RuleError>;
        fn sound_observations(
            &self,
            _: i16,
            _: &AudioContext,
            _: &FeedbackMemory,
        ) -> Result<Vec<AudioObservation>, RuleError>;
    }
}

unavailable! {
    centurions_core::recovered_combat_ui::CombatUiServices {
        fn observe(
            &self,
            _: &CombatUiRequest,
            _: &CombatUiMemory,
        ) -> Result<CombatUiObservation, RuleError>;
    }
}

unavailable! {
    centurions_core::recovered_drawing::DrawingServices {
        fn color_icon(&mut self, _: i16) -> Result<u32, RuleError>;
        fn indexed_string(&mut self, _: i16, _: i16) -> Result<PascalString, RuleError>;
        fn current_world(&mut self) -> Result<(u32, u32), RuleError>;
        fn picture(&mut self, _: i16) -> Result<PictureObservation, RuleError>;
        fn new_world(&mut self, _: i16, _: BodyRect) -> Result<(i16, BuildingWorld), RuleError>;
    }
}

unavailable! {
    centurions_core::recovered_text::NumericServices {
        fn number_to_string(&mut self, _: i32) -> Result<PascalString, RuleError>;
        fn string_to_number(&mut self, _: &PascalString) -> Result<i32, RuleError>;
    }
}

unavailable! {
    centurions_core::recovered_world_display::WorldDisplayServices {
        fn observe(
            &mut self,
            _: &WorldDisplayRequest,
            _: &RenderingMemory,
        ) -> Result<WorldDisplayObservation, RuleError>;
    }
}

unavailable! {
    centurions_core::recovered_combat_center::CombatCenterServices {
    }
}

unavailable! {
    centurions_core::recovered_item_ui::ItemUiServices {
        fn observe(&mut self, _: &ItemUiRequest, _: &RenderingMemory)
        -> Result<ItemUiReply, RuleError>;
    }
}

unavailable! {
    centurions_core::recovered_equipment_ui::EquipmentUiServices {
        fn pen_position(&mut self, _: u32, _: &RenderingMemory) -> Result<Vec<RecordWrite>, RuleError>;
    }
}

unavailable! {
    centurions_core::recovered_interface::InterfaceServices {
        fn word(&mut self, _: u32) -> Result<i16, RuleError>;
        fn current_port(&mut self) -> Result<u32, RuleError>;
        fn pen_state(&mut self) -> Result<[u8; 18], RuleError>;
        fn local_to_global(&mut self, _: Point) -> Result<Point, RuleError>;
        fn point_in_rect(&mut self, _: Point, _: BodyRect) -> Result<u8, RuleError>;
        fn new_region(&mut self) -> Result<u32, RuleError>;
        fn main_device_rect(&mut self) -> Result<BodyRect, RuleError>;
    }
}

unavailable! {
    centurions_core::recovered_character_panels::CharacterPanelServices {
        fn panel_observe(
            &mut self,
            _: &CharacterPanelRequest,
            _: &RenderingMemory,
        ) -> Result<CharacterPanelObservation, RuleError>;
        fn runtime_observations(
            &mut self,
            _: u32,
            _: &RenderingMemory,
        ) -> Result<Vec<RuntimeObservation>, RuleError>;
    }
}

unavailable! {
    centurions_core::recovered_inventory_rules::InventoryServices {
        fn sane(&self, _: &SaneRequest, _: &RenderingMemory) -> Result<Vec<RecordWrite>, RuleError>;
    }
}

unavailable! {
    centurions_core::recovered_gamma::GammaServices {
        fn observe(&self, _: &GammaRequest, _: &GammaMemory) -> Result<GammaObservation, GammaError>;
    }
}

unavailable! {
    centurions_core::recovered_gamma_control::GammaControlServices {
        fn observe_control(
            &self,
            _: &GammaControlRequest,
            _: &GammaMemory,
        ) -> Result<GammaObservation, GammaError>;
    }
}

unavailable! {
    centurions_core::recovered_load_control::LoadServices {
        fn observe_load(
            &self,
            _: &LoadRequest,
            _: &RenderingMemory,
        ) -> Result<LoadObservation, RuleError>;
        fn version_inputs(
            &self,
            _: i16,
            _: i16,
            _: &RenderingMemory,
        ) -> Result<VersionInputs, RuleError>;
        fn filename_number(&self, _: u8, _: &RenderingMemory) -> Result<PascalString, RuleError>;
    }
}

unavailable! {
    centurions_core::recovered_prompt_control::PromptControlServices {
        fn library(
            &mut self,
            _: &LibraryRequest,
            _: &RenderingMemory,
        ) -> Result<LibraryObservation, RuleError>;
        fn quiet_observations(
            &mut self,
            _: i16,
            _: &AudioContext,
            _: &RenderingMemory,
        ) -> Result<Vec<AudioObservation>, RuleError>;
        fn incoming_mechanical_mode(&mut self, _: &RenderingMemory) -> Result<i16, RuleError>;
    }
}

unavailable! {
    centurions_core::recovered_character_ui::CharacterUiServices {
        fn observe(
            &mut self,
            _: &CharacterUiRequest,
            _: &RenderingMemory,
        ) -> Result<CharacterUiObservation, RuleError>;
        fn profile_observations(
            &mut self,
            _: i16,
            _: i16,
            _: &FileContext,
        ) -> Result<Vec<FileObservation>, RuleError>;
    }
}

unavailable! {
    centurions_core::recovered_equipment::EquipmentServices {
        fn profile_observations(
            &mut self,
            _: i16,
            _: i16,
            _: &RenderingMemory,
        ) -> Result<Vec<FileObservation>, RuleError>;
    }
}

unavailable! {
    centurions_core::recovered_inventory_edit::InventoryEditServices {
        fn observe_edit(
            &mut self,
            _: &InventoryEditRequest,
            _: &RenderingMemory,
        ) -> Result<InventoryEditObservation, RuleError>;
    }
}

unavailable! {
    centurions_core::recovered_storage::StorageServices {
        fn observe_storage(
            &self,
            _: &StorageRequest,
            _: &RenderingMemory,
        ) -> Result<StorageObservation, RuleError>;
    }
}

unavailable! {
    centurions_core::recovered_detail_ui::DetailServices {
        fn observe(&mut self, _: &DetailRequest, _: &RenderingMemory)
        -> Result<DetailReply, RuleError>;
    }
}

unavailable! {
    centurions_core::recovered_combat_panel::CombatPanelServices {
        fn observe_panel(
            &mut self,
            _: &CombatPanelRequest,
            _: &RenderingMemory,
        ) -> Result<CombatPanelObservation, RuleError>;
        fn visibility_memory(&mut self, _: &RenderingMemory) -> Result<InstallMemory, RuleError>;
    }
}

unavailable! {
    centurions_core::recovered_status_panels::StatusPanelServices {
        fn observe_status(
            &self,
            _: &StatusPanelRequest,
            _: &RenderingMemory,
        ) -> Result<StatusPanelObservation, RuleError>;
        fn body_ground_state(&self, _: &RenderingMemory) -> Result<BodyState, RuleError>;
        fn quiet_observations(
            &self,
            _: i16,
            _: &AudioContext,
            _: &RenderingMemory,
        ) -> Result<Vec<AudioObservation>, RuleError>;
        fn heap_observations(&self, _: &RuntimeContext) -> Result<Vec<RuntimeObservation>, RuleError>;
    }
}

unavailable! {
    centurions_core::recovered_inventory_selection::InventorySelectionServices {
    }
}

unavailable! {
    centurions_core::recovered_field_control::FilePrepServices {
        fn preference_inputs(&mut self, _: &RenderingMemory) -> Result<PreferenceInputs, RuleError>;
        fn metadata_observations(
            &mut self,
            _: &[u8],
            _: &[u8],
            _: &RenderingMemory,
        ) -> Result<Vec<FileObservation>, RuleError>;
    }
}

unavailable! {
    centurions_core::recovered_scenario_menus::ScenarioMenuServices {
    }
}

unavailable! {
    centurions_core::recovered_field_control::FieldControlServices {
        fn incoming_attack_slot(&mut self, _: &RenderingMemory) -> Result<u16, RuleError>;
    }
}

unavailable! {
    centurions_core::recovered_character_upgrade::CharacterUpgradeServices {
    }
}

unavailable! {
    centurions_core::recovered_inventory_interaction::InventoryInteractionServices {
    }
}

unavailable! {
    centurions_core::recovered_timing::TimingServices {
        fn tick_count(&mut self) -> Result<u32, RuleError>;
        fn button(&mut self) -> Result<bool, RuleError>;
    }
}

unavailable! {
    centurions_core::recovered_spell_ui::SpellUiServices {
        fn dialog_item(&mut self, _: u32, _: i16) -> Result<DialogItemObservation, RuleError>;
        fn string_width(&mut self, _: &PascalString) -> Result<i16, RuleError>;
        fn color_cursor(&mut self, _: i16) -> Result<u32, RuleError>;
        fn square_root_extended(&mut self, _: [u8; 10]) -> Result<[u8; 10], RuleError>;
        fn truncate_extended(&mut self, _: [u8; 10]) -> Result<i32, RuleError>;
    }
}

unavailable! {
    centurions_core::recovered_shop_ui::NpcNameServices {
        fn copy_name(&self, _: u32, _: &[u8]) -> Result<PascalString, RuleError>;
    }
}

unavailable! {
    centurions_core::recovered_selection_control::SelectionControlServices {
    }
}

unavailable! {
    centurions_core::recovered_loot_control::LootControlServices {
    }
}

unavailable! {
    centurions_core::recovered_startup::StartupServices {
        fn observe_startup(
            &self,
            _: &StartupRequest,
            _: &RenderingMemory,
        ) -> Result<StartupObservation, RuleError>;
        fn preference_inputs(&self, _: &RenderingMemory) -> Result<PreferenceInputs, RuleError>;
    }
}

unavailable! {
    centurions_core::recovered_combat_results::CombatResultServices {
        fn graphics_dispatch(
            &mut self,
            _: &GraphicsDispatchRequest,
            _: &RenderingMemory,
        ) -> Result<FeedbackObservation, RuleError>;
    }
}

unavailable! {
    centurions_core::recovered_inventory_interaction::ItemsServices {
        fn items_damage_frames(&mut self, _: &RenderingMemory) -> Result<[DamageFrame; 2], RuleError>;
    }
}

unavailable! {
    centurions_core::recovered_native_services::NativeServices {
    }
}
