use godot::prelude::*;

pub struct AssetLibraryUI {
    vbox: Option<VBoxContainer>,
    asset_list: Option<Tree>,
    asset_preview: Option<GridContainer>,
}

impl AssetLibraryUI {
    pub fn new() -> Self {
        Self {
            vbox: None,
            asset_list: None,
            asset_preview: None,
        }
    }
}

impl Control for AssetLibraryUI {
    fn _ready(&mut self) {
        // Create the UI layout
        self.vbox = Some(VBoxContainer::new());
        self.add_child(self.vbox.as_ref().unwrap());

        // Create the asset list
        self.asset_list = Some(Tree::new());
        self.vbox.as_ref().unwrap().add_child(self.asset_list.as_ref().unwrap());

        // Create the asset preview
        self.asset_preview = Some(GridContainer::new());
        self.vbox.as_ref().unwrap().add_child(self.asset_preview.as_ref().unwrap());

        // Configure the asset list
        self.asset_list.as_ref().unwrap().set_columns(2);
        self.asset_list.as_ref().unwrap().set_column_titles(vec!["Asset".to_string(), "Type".to_string()]);

        // Connect signals
        self.asset_list.as_ref().unwrap().connect("item_selected", self, "_on_asset_selected");
    }

    fn _on_asset_selected(&mut self) {
        // Get the selected asset
        let selected_asset = self.asset_list.as_ref().unwrap().get_selected_item();

        // Update the asset preview
        self.asset_preview.as_ref().unwrap().clear();
        if let Some(asset) = selected_asset {
            // Create a new asset preview node
            let asset_preview_node = AssetPreviewNode::new(asset);
            self.asset_preview.as_ref().unwrap().add_child(asset_preview_node);
        }
    }
}

pub struct AssetPreviewNode {
    asset: String,
}

impl AssetPreviewNode {
    pub fn new(asset: String) -> Self {
        Self { asset }
    }
}

impl Control for AssetPreviewNode {
    fn _ready(&mut self) {
        // Create the asset preview UI
        let vbox = VBoxContainer::new();
        self.add_child(vbox);

        // Add asset information
        let label = Label::new();
        label.set_text(self.asset.clone());
        vbox.add_child(label);
    }
}