use godot::prelude::*;

mod asset_library;
mod gui;
mod asset_library_extension;

struct GodotAssetBrowserExtension;

#[gdextension]
unsafe impl ExtensionLibrary for GodotAssetBrowserExtension {}