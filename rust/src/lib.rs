use godot::prelude::*;
use godot::classes::{EditorPlugin, IEditorPlugin};
use godot::classes::editor_plugin::DockSlot;
use crate::gui::asset_library_ui::AssetLibraryGUI;

mod asset_library;
mod gui;
mod asset_library_extension;

struct GodotAssetBrowserExtension;

#[gdextension]
unsafe impl ExtensionLibrary for GodotAssetBrowserExtension {}

#[derive(GodotClass)]
#[class(tool, base=EditorPlugin)]
struct AssetBrowserPlugin {
    base: Base<EditorPlugin>,
    ux: Option<Gd<gui::asset_library_ui::AssetLibraryGUI>>,
    asset_manager: Option<asset_library::asset_manager::AssetManager>,
    config_manager: Option<asset_library::config_manager::ConfigManager>,
}

#[godot_api]
impl IEditorPlugin for AssetBrowserPlugin {
    fn init(base: Base<EditorPlugin>) -> Self {
        Self {
            base,
            ux: None,
            asset_manager: None,
            config_manager: None,
        }
    }

    fn get_plugin_name(&self) -> GString {
        "Asset Browser".into()
    }

    fn enter_tree(&mut self) {
        godot_print!("Godot Asset Browser Plugin Activated");

        let mut ux = gui::asset_library_ui::AssetLibraryGUI::new_alloc();
        self.ux = Some(ux.clone());
        self.base_mut().add_child(ux.upcast());

        // Create the asset manager and config manager
        self.asset_manager = Some(asset_library::asset_manager::AssetManager::new());
        self.config_manager = Some(asset_library::config_manager::ConfigManager::new());
    }

    fn exit_tree(&mut self) {
        godot_print!("Godot Asset Browser Plugin Deactivated");
        // Clean up the UI and other components
        if let Some(ux) = self.ux.take() {
            ux.queue_free();
        }
        self.asset_manager = None;
        self.config_manager = None;
    }
}