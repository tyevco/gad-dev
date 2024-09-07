mod asset_library;
mod gui;
mod asset_library_extension;

use godot::prelude::*;
use godot::classes::Control;
use godot::classes::{EditorPlugin, IEditorPlugin};
use crate::asset_library::asset_manager::AssetManager;
use crate::asset_library::config_manager::ConfigManager;
use gui::asset_library_ui::AssetLibraryGUI;

struct GodotAssetBrowserExtension
{
    ux: Option<AssetLibraryGUI>,
    asset_manager: Option<AssetManager>,
    config_manager: Option<ConfigManager>
}

#[gdextension]
unsafe impl ExtensionLibrary for GodotAssetBrowserExtension {}
