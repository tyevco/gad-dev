use godot::classes::Control;
use crate::asset_library::asset_manager::AssetManager;
use godot::prelude::*;

pub struct AssetLibraryUI {
    asset_manager: AssetManager,
}

impl AssetLibraryUI {
    pub fn new(asset_manager: AssetManager) -> Self {
        Self { asset_manager }
    }

    pub fn build_ui(&self) -> Control {
        // Create a new Control node
        let control = Control::new_alloc();

        // TO DO: implement UI building logic

        // Don't forget to free the control when we're done with it
        // control.free();

        // Or, we can hand over ownership to Godot
        // control.into_godot();
        control.into_godot();
    }
}