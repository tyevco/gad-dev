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
