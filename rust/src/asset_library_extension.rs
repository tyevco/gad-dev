use godot::classes::editor_plugin::DockSlot;
use godot::classes::{EditorPlugin, IEditorPlugin};
use godot::prelude::*;
use asset_library::asset_manager::AssetManager;
use asset_library::config_manager::ConfigManager;
use crate::asset_library;
use crate::gui::asset_library_ui::AssetLibraryGUI;

#[derive(GodotClass)]
#[class(tool, init, base=EditorPlugin)]
struct GABPlugin {
    base: Base<EditorPlugin>,
    gui: Option<Gd<AssetLibraryGUI>>,
    asset_manager: Option<Gd<AssetManager>>,
    config_manager: Option<Gd<ConfigManager>>,
}


#[godot_api]
impl IEditorPlugin for GABPlugin {
    fn has_main_screen(&self) -> bool {
        true
    }

    fn make_visible(&mut self, visible: bool) {
        // Show the UI
        if visible {
            if let Some(ui) = self.gui.as_mut() {
                ui.show();
            }
        }
    }

    fn enter_tree(&mut self) {
        godot_print!("Godot Asset Browser Plugin Activated");
        // Create the UI and add it to the editor's dock
        self.gui = Some(AssetLibraryGUI::new_alloc());
        self.base_mut().add_control_to_dock(DockSlot::LEFT_UL, self.gui.as_mut().unwrap());

        // Create the asset manager and config manager
        //self.asset_manager = Some(AssetManager::new());
        //self.config_manager = Some(ConfigManager::new());
    }

    fn exit_tree(&mut self) {
        godot_print!("Godot Asset Browser Plugin Deactivated");
        // Clean up the UI and other components
        if let Some(gui) = self.gui.take() {
            gui.queue_free();
        }
        if let Some(asset_manager) = self.asset_manager.take() {
            drop(asset_manager);
        }
        if let Some(config_manager) = self.config_manager.take() {
            drop(config_manager);
        }
    }
}