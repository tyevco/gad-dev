mod asset_library;
mod ui;
mod asset_library_extension;

use godot::prelude::*;
use godot::classes::{EditorPlugin, IEditorPlugin};
use godot::private::You_forgot_the_attribute__godot_api;

struct GodotAssetBrowserExtension;

#[gdextension]
unsafe impl ExtensionLibrary for GodotAssetBrowserExtension {}

#[derive(GodotClass)]
#[class(tool, init, editor_plugin, base=EditorPlugin)]
struct GABPlugin {
    base: Base<EditorPlugin>,
}


#[godot_api]
impl IEditorPlugin for GABPlugin {
    fn enter_tree(&mut self) {
        // Perform typical plugin operations here.
        godot_print!("Godot Asset Browser Plugin Activated");
    }

    fn exit_tree(&mut self) {
        // Perform typical plugin operations here.
        godot_print!("Godot Asset Browser Plugin Deactivated");
    }
}