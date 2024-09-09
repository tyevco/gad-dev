use crate::asset_library::asset_manager::AssetManager;
use crate::asset_library::config_manager::ConfigManager;
use crate::gui::asset_library_ui::AssetLibraryGUI;
use godot::classes::editor_plugin::DockSlot;
use godot::classes::{EditorPlugin, IEditorPlugin};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(tool, init, base=EditorPlugin)]
struct GABPlugin {
    #[base]
    base: Base<EditorPlugin>,
    gui: Option<Gd<AssetLibraryGUI>>,
    asset_manager: Option<AssetManager>,
    config_manager: Option<ConfigManager>,
}

#[godot_api]
impl IEditorPlugin for GABPlugin {
    fn enter_tree(&mut self) {
        godot_print!("Godot Asset Browser Plugin Activated");
        // Create the UI and add it to the editor's dock
        let gui = AssetLibraryGUI::new_alloc();
        self.base_mut().add_control_to_dock(DockSlot::LEFT_UL, gui.clone());
        self.gui = Some(gui);
        // Create the asset manager and config manager
        self.asset_manager = Some(AssetManager::new());
        self.config_manager = Some(ConfigManager::new());
    }

    fn exit_tree(&mut self) {
        godot_print!("Godot Asset Browser Plugin Deactivated");
        // Clean up the UI and other components
        if let Some(mut gui) = self.gui.take() {
            gui.queue_free();
        }
        self.asset_manager = None;
        self.config_manager = None;
    }

    fn has_main_screen(&self) -> bool {
        true
    }

    fn make_visible(&mut self, visible: bool) {
        // Show the UI
        if let Some(gui) = self.gui.as_mut() {
        if visible {
                gui.show();
        } else {
                gui.hide();
            }
        }
    }
}
