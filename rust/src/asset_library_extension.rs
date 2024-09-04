use godot::prelude::*;
use asset_manager::AssetManager;
use config_manager::ConfigManager;

pub struct AssetLibraryExtension {
    ui: Option<AssetLibraryUI>,
    asset_manager: Option<AssetManager>,
    config_manager: Option<ConfigManager>,
}

impl AssetLibraryExtension {
    pub fn new() -> Self {
        Self {
            ui: None,
            asset_manager: None,
            config_manager: None,
        }
    }
}

impl EditorPlugin for AssetLibraryExtension {
    fn enter_tree(&mut self) {
        // Create the UI and add it to the editor's dock
        self.ui = Some(AssetLibraryUI::new());
        add_control_to_dock(DockSlot::LeftUl, self.ui.as_ref().unwrap());

        // Create the asset manager and config manager
        self.asset_manager = Some(AssetManager::new());
        self.config_manager = Some(ConfigManager::new());
    }

    fn exit_tree(&mut self) {
        // Clean up the UI and other components
        if let Some(ui) = self.ui.take() {
            ui.queue_free();
        }
        if let Some(asset_manager) = self.asset_manager.take() {
            drop(asset_manager);
        }
        if let Some(config_manager) = self.config_manager.take() {
            drop(config_manager);
        }
    }

    fn has_main_screen(&self) -> bool {
        true
    }

    fn make_visible(&mut self) {
        // Show the UI
        if let Some(ui) = self.ui.as_mut() {
            ui.show();
        }
    }
}