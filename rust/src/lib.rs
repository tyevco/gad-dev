use godot::{classes::{EditorPlugin, IEditorPlugin}, prelude::*};

struct GodotAssetBrowserExtension;

#[gdextension]
unsafe impl ExtensionLibrary for GodotAssetBrowserExtension {}

#[derive(GodotClass)]
#[class(tool, init, editor_plugin, base=EditorPlugin)]
struct MyEditorPlugin {
    base: Base<EditorPlugin>,
}

#[godot_api]
impl IEditorPlugin for MyEditorPlugin {
    fn enter_tree(&mut self) {
        // Perform typical plugin operations here.
    }

    fn exit_tree(&mut self) {
        // Perform typical plugin operations here.
    }
}