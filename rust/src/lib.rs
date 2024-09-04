use godot::classes::{EditorPlugin, IEditorPlugin};
use godot::prelude::*;

struct GodotAssetBrowserExtension;

#[gdextension]
unsafe impl ExtensionLibrary for GodotAssetBrowserExtension {}

#[derive(GodotClass)]
#[class(base=EditorPlugin, tool)]
struct GABPlugin {
    base: Base<EditorPlugin>
}

#[godot_api]
impl GABPlugin {
    #[func]
    fn enter_tree(&mut self) {
        godot_print!("Godot Asset Browser Plugin Activated");
    }

    #[func]
    fn exit_tree(&mut self) {
        godot_print!("Godot Asset Browser Plugin Deactivated");
    }
}

#[godot_api]
impl IEditorPlugin for GABPlugin {
    fn init(base: Base<EditorPlugin>) -> Self {
        Self { base }
    }
}