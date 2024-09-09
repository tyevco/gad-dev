use godot::classes::{Control, GridContainer, IControl, Label, Tree, VBoxContainer};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(tool, base=Control)]
pub struct AssetLibraryGUI {
    #[base]
    base: Base<Control>,
    vbox: Option<Gd<VBoxContainer>>,
    asset_list: Option<Gd<Tree>>,
    asset_preview: Option<Gd<GridContainer>>,
}

#[godot_api]
impl AssetLibraryGUI {
    #[func]
    fn on_asset_selected(&mut self) {
        if let Some(list) = &self.asset_list {
            let selected_item = list.get_selected();

            if let Some(mut preview) = self.asset_preview.clone() {
                let children = preview.get_children();

                if let Some(item) = selected_item {
                    let asset_name = item.get_text(0);
                    let asset_preview_node = AssetPreviewNode::new_with_asset(asset_name.to_string());
                    preview.add_child(asset_preview_node);
                }
            }
        }
    }
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
        let mut vbox = VBoxContainer::new_alloc();

        let mut asset_list = Tree::new_alloc();
        asset_list.set_columns(2);
        asset_list.set_column_title(0, "Asset".into());
        asset_list.set_column_title(1, "Type".into());
        asset_list.connect("item_selected".into(), self.base().callable("on_asset_selected"));

        vbox.add_child(asset_list.clone());
        self.asset_list = Some(asset_list);

        let asset_preview = GridContainer::new_alloc();
        vbox.add_child(asset_preview.clone());
        self.asset_preview = Some(asset_preview);

        self.base_mut().add_child(vbox.clone());
        self.vbox = Some(vbox);
    }
}

#[derive(GodotClass)]
#[class(base=Control)]
pub struct AssetPreviewNode {
    #[base]
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
        let mut vbox = VBoxContainer::new_alloc();

        let mut label = Label::new_alloc();
        label.set_text(self.asset.clone().into());
        vbox.add_child(label);

        self.base_mut().add_child(vbox);
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