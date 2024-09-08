use godot::prelude::*;
use godot::classes::{Control, VBoxContainer, Tree, GridContainer, Label, IControl};

#[derive(GodotClass)]
#[class(tool, init, base=Control)]
pub struct AssetLibraryGUI {
    base: Base<Control>,
    vbox: Option<Gd<VBoxContainer>>,
    asset_list: Option<Gd<Tree>>,
    asset_preview: Option<Gd<GridContainer>>,
}

#[godot_api]
impl IControl for AssetLibraryGUI {
    fn init(base: Base<Control>) -> Self {
        Self {
            base,
            vbox: None,
            asset_list: None,
            asset_preview: None,
        }
    }

    fn ready(&mut self) {
        // Create the UI layout
        let vbox = VBoxContainer::new_alloc();
        self.base_mut().add_child(vbox.clone().upcast());
        self.vbox = Some(vbox);

        // Create the asset list
        let asset_list = Tree::new_alloc();
        self.vbox.as_ref().unwrap().add_child(asset_list.clone().upcast());
        self.asset_list = Some(asset_list);

        // Create the asset preview
        let asset_preview = GridContainer::new_alloc();
        self.vbox.as_ref().unwrap().add_child(asset_preview.clone().upcast());
        self.asset_preview = Some(asset_preview);

        // Configure the asset list
        if let Some(list) = &self.asset_list {
            list.set_columns(2);
            list.set_column_title(0, "Asset".into());
            list.set_column_title(1, "Type".into());
        }

        // Connect signals
        if let Some(list) = &self.asset_list {
            list.connect("item_selected".into(), self.base.callable("on_asset_selected"));
        }
    }
}

#[godot_api]
impl AssetLibraryGUI {
    #[func]
    fn on_asset_selected(&mut self) {
        // Get the selected asset
        if let Some(list) = &self.asset_list {
            let selected_item = list.get_selected();

            // Update the asset preview
            if let Some(preview) = &self.asset_preview {
                preview.clear();
                if let Some(item) = selected_item {
                    // Create a new asset preview node
                    let asset_name = item.get_text(0);
                    let asset_preview_node = AssetPreviewNode::new_alloc_with_asset(asset_name.to_string());
                    preview.add_child(asset_preview_node.upcast());
                }
            }
        }
    }
}

#[derive(GodotClass)]
#[class(base=Control)]
pub struct AssetPreviewNode {
    base: Base<Control>,
    asset: String,
}

#[godot_api]
impl IControl for AssetPreviewNode {
    fn init(base: Base<Control>) -> Self {
        Self {
            base,
            asset: String::new(),
        }
    }

    fn ready(&mut self) {
        // Create the asset preview UI
        let vbox = VBoxContainer::new_alloc();
        self.base_mut().add_child(vbox.clone().upcast());

        // Add asset information
        let label = Label::new_alloc();
        label.set_text(self.asset.clone().into());
        vbox.add_child(label.upcast());
    }
}

#[godot_api]
impl AssetPreviewNode {
    #[func]
    fn new_with_asset(asset: String) -> Gd<Self> {
        let mut instance = Self::new_alloc();
        instance.bind_mut().asset = asset;
        instance
    }
}