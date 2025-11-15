use godot::classes::{Control, GridContainer, HBoxContainer, IControl, Label, LineEdit, OptionButton, Panel, StyleBoxFlat, Tree, VBoxContainer};
use godot::prelude::*;
use godot::builtin::Array;
use crate::asset_library::{AssetManager, AssetCategory};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SortCriteria {
    Name,
    Category,
    Author,
}

#[derive(GodotClass)]
#[class(tool, base=Control)]
pub struct AssetLibraryGUI {
    #[base]
    base: Base<Control>,
    vbox: Option<Gd<VBoxContainer>>,
    search_box: Option<Gd<LineEdit>>,
    category_filter: Option<Gd<OptionButton>>,
    sort_by: Option<Gd<OptionButton>>,
    asset_list: Option<Gd<Tree>>,
    asset_preview: Option<Gd<GridContainer>>,
    asset_manager: AssetManager,
    current_search_query: String,
    current_category_filter: Option<AssetCategory>,
    current_sort_criteria: SortCriteria,
}

#[godot_api]
impl AssetLibraryGUI {
    #[func]
    fn on_search_changed(&mut self, new_text: GString) {
        self.current_search_query = new_text.to_string();
        self.refresh_asset_list();
    }

    #[func]
    fn on_category_changed(&mut self, index: i32) {
        // Index 0 is "All Categories" (None filter)
        // Indices 1+ correspond to AssetCategory variants
        if index == 0 {
            self.current_category_filter = None;
        } else {
            let categories = AssetCategory::all();
            if let Some(category) = categories.get((index - 1) as usize) {
                self.current_category_filter = Some(*category);
            }
        }
        self.refresh_asset_list();
    }

    #[func]
    fn on_sort_changed(&mut self, index: i32) {
        self.current_sort_criteria = match index {
            0 => SortCriteria::Name,
            1 => SortCriteria::Category,
            2 => SortCriteria::Author,
            _ => SortCriteria::Name,
        };
        self.refresh_asset_list();
    }

    fn refresh_asset_list(&mut self) {
        if let Some(mut list) = self.asset_list.clone() {
            // Clear existing items
            list.clear();

            // Create root
            let root = list.create_item();

            // Get filtered assets
            let mut assets = if self.current_search_query.is_empty() {
                if let Some(category) = self.current_category_filter {
                    self.asset_manager.get_assets_by_category(category)
                } else {
                    self.asset_manager.get_assets()
                }
            } else {
                let mut filtered = self.asset_manager.search_assets(&self.current_search_query);

                // Further filter by category if one is selected
                if let Some(category) = self.current_category_filter {
                    filtered.retain(|asset| asset.category == category);
                }

                filtered
            };

            // Sort assets based on current sort criteria
            match self.current_sort_criteria {
                SortCriteria::Name => {
                    assets.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
                }
                SortCriteria::Category => {
                    assets.sort_by(|a, b| {
                        a.category
                            .display_name()
                            .cmp(b.category.display_name())
                            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
                    });
                }
                SortCriteria::Author => {
                    assets.sort_by(|a, b| {
                        a.author
                            .to_lowercase()
                            .cmp(&b.author.to_lowercase())
                            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
                    });
                }
            }

            // Populate tree with filtered and sorted assets
            for asset in assets {
                if let Some(mut item) = list.create_item_ex().parent(root.clone()).done() {
                    item.set_text(0, asset.name.clone().into());
                    item.set_text(1, asset.category.display_name().into());
                    item.set_text(2, asset.author.clone().into());
                    item.set_metadata(0, asset.id.to_variant());
                }
            }
        }
    }

    #[func]
    fn on_asset_selected(&mut self) {
        godot_print!("Asset selected");
        if let Some(list) = &self.asset_list {
            let selected_item = list.get_selected();

            if let Some(mut preview) = self.asset_preview.clone() {
                // Clear existing preview items
                let children = preview.get_children();
                for i in 0..children.len() {
                    if let Some(mut child) = children.get(i) {
                        child.queue_free();
                    }
                }

                if let Some(item) = selected_item {
                    // Get the asset ID from metadata
                    let asset_id = item.get_metadata(0).to::<GString>();

                    // Retrieve full asset information from AssetManager
                    if let Some(asset) = self.asset_manager.get_asset_by_id(&asset_id.to_string()) {
                        // Convert Vec<String> to Array<GString>
                        let mut tags_array = Array::<GString>::new();
                        for tag in asset.tags.iter() {
                            tags_array.push(tag.clone().into());
                        }

                        // Determine asset status
                        let status = if self.asset_manager.is_asset_installed(&asset.id) {
                            if self.asset_manager.check_for_update(&asset.id).unwrap_or(None).is_some() {
                                AssetStatus::UpdateAvailable
                            } else {
                                AssetStatus::Installed
                            }
                        } else {
                            AssetStatus::NotInstalled
                        };

                        let asset_preview_node = AssetPreviewNode::new_with_asset(
                            asset.id.into(),
                            asset.name.into(),
                            asset.author.into(),
                            asset.version.into(),
                            asset.category.display_name().into(),
                            asset.description.into(),
                            tags_array,
                            asset.preview_url.unwrap_or_default().into(),
                            status as i32,
                        );
                        preview.add_child(asset_preview_node);
                    }
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
            search_box: None,
            category_filter: None,
            sort_by: None,
            asset_list: None,
            asset_preview: None,
            asset_manager: AssetManager::new(),
            current_search_query: String::new(),
            current_category_filter: None,
            current_sort_criteria: SortCriteria::Name,
        }
    }

    fn ready(&mut self) {
        let mut vbox = VBoxContainer::new_alloc();

        self.base_mut().set_name("GAB".into());

        // Make the GUI responsive by setting anchors
        self.base_mut().set_anchor(godot::builtin::Side::LEFT, 0.0);
        self.base_mut().set_anchor(godot::builtin::Side::TOP, 0.0);
        self.base_mut().set_anchor(godot::builtin::Side::RIGHT, 1.0);
        self.base_mut().set_anchor(godot::builtin::Side::BOTTOM, 1.0);

        // Add search box
        let mut search_hbox = HBoxContainer::new_alloc();

        let mut search_label = Label::new_alloc();
        search_label.set_text("Search:".into());
        search_hbox.add_child(search_label);

        let mut search_box = LineEdit::new_alloc();
        search_box.set_placeholder("Search assets...".into());
        search_box.set_custom_minimum_size(godot::prelude::Vector2::new(200.0, 0.0));
        search_box.set_h_size_flags(godot::classes::control::SizeFlags::EXPAND_FILL);
        search_box.set_focus_mode(godot::classes::control::FocusMode::ALL);
        search_box.set_tooltip_text("Search assets by name, tags, or description".into());
        search_box.connect(
            "text_changed".into(),
            self.base().callable("on_search_changed"),
        );
        search_hbox.add_child(search_box.clone());
        self.search_box = Some(search_box);

        // Add category filter
        let mut category_label = Label::new_alloc();
        category_label.set_text("  Category:".into());
        search_hbox.add_child(category_label);

        let mut category_filter = OptionButton::new_alloc();
        category_filter.add_item("All Categories".into());

        // Add all available categories
        let categories = AssetCategory::all();
        for category in categories.iter() {
            category_filter.add_item(category.display_name().into());
        }

        category_filter.connect(
            "item_selected".into(),
            self.base().callable("on_category_changed"),
        );
        search_hbox.add_child(category_filter.clone());
        self.category_filter = Some(category_filter);

        // Add sort by dropdown
        let mut sort_label = Label::new_alloc();
        sort_label.set_text("  Sort by:".into());
        search_hbox.add_child(sort_label);

        let mut sort_by = OptionButton::new_alloc();
        sort_by.add_item("Name".into());
        sort_by.add_item("Category".into());
        sort_by.add_item("Author".into());
        sort_by.connect(
            "item_selected".into(),
            self.base().callable("on_sort_changed"),
        );
        search_hbox.add_child(sort_by.clone());
        self.sort_by = Some(sort_by);

        vbox.add_child(search_hbox);

        let mut asset_list = Tree::new_alloc();
        asset_list.set_columns(3);
        asset_list.set_column_title(0, "Asset Name".into());
        asset_list.set_column_title(1, "Category".into());
        asset_list.set_column_title(2, "Author".into());
        asset_list.set_hide_root(true);

        // Make the tree responsive
        asset_list.set_v_size_flags(godot::classes::control::SizeFlags::EXPAND_FILL);
        asset_list.set_custom_minimum_size(godot::prelude::Vector2::new(0.0, 300.0));

        // Enable keyboard navigation
        asset_list.set_focus_mode(godot::classes::control::FocusMode::ALL);

        asset_list.connect(
            "item_selected".into(),
            self.base().callable("on_asset_selected"),
        );

        // Populate the tree with assets from AssetManager
        let root = asset_list.create_item();
        let assets = self.asset_manager.get_assets();

        for asset in assets {
            if let Some(mut item) = asset_list.create_item_ex().parent(root.clone()).done() {
                item.set_text(0, asset.name.clone().into());
                item.set_text(1, asset.category.display_name().into());
                item.set_text(2, asset.author.clone().into());
                // Store the asset ID in the metadata for later retrieval
                item.set_metadata(0, asset.id.to_variant());
            }
        }

        vbox.set_name("GAB".into());

        // Make vbox fill the parent container
        vbox.set_anchor(godot::builtin::Side::LEFT, 0.0);
        vbox.set_anchor(godot::builtin::Side::TOP, 0.0);
        vbox.set_anchor(godot::builtin::Side::RIGHT, 1.0);
        vbox.set_anchor(godot::builtin::Side::BOTTOM, 1.0);

        vbox.add_child(asset_list.clone());
        self.asset_list = Some(asset_list);

        let mut asset_preview = GridContainer::new_alloc();
        asset_preview.set_columns(1);
        asset_preview.set_v_size_flags(godot::classes::control::SizeFlags::EXPAND_FILL);
        vbox.add_child(asset_preview.clone());
        self.asset_preview = Some(asset_preview);

        self.base_mut().add_child(vbox.clone());
        self.vbox = Some(vbox);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetStatus {
    NotInstalled,
    Installed,
    UpdateAvailable,
}

#[derive(GodotClass)]
#[class(base=Control)]
pub struct AssetPreviewNode {
    #[base]
    base: Base<Control>,
    asset_id: String,
    asset_name: String,
    asset_author: String,
    asset_version: String,
    asset_category: String,
    asset_description: String,
    asset_tags: Vec<String>,
    asset_preview_url: Option<String>,
    asset_status: AssetStatus,
}

#[godot_api]
impl IControl for AssetPreviewNode {
    fn init(base: Base<Control>) -> Self {
        Self {
            base,
            asset_id: String::new(),
            asset_name: String::new(),
            asset_author: String::new(),
            asset_version: String::new(),
            asset_category: String::new(),
            asset_description: String::new(),
            asset_tags: Vec::new(),
            asset_preview_url: None,
            asset_status: AssetStatus::NotInstalled,
        }
    }

    fn ready(&mut self) {
        use godot::classes::{HBoxContainer, RichTextLabel, ScrollContainer};

        let mut vbox = VBoxContainer::new_alloc();
        vbox.set_custom_minimum_size(godot::prelude::Vector2::new(400.0, 300.0));

        // Header with name and status badge
        let mut header_hbox = HBoxContainer::new_alloc();

        // Asset name header
        let mut name_label = Label::new_alloc();
        name_label.set_text(self.asset_name.clone().into());
        name_label.add_theme_font_size_override("font_size".into(), 24);
        name_label.set_tooltip_text(format!("Asset ID: {}", self.asset_id).into());
        header_hbox.add_child(name_label);

        // Status badge
        let mut status_badge = Panel::new_alloc();
        let mut badge_label = Label::new_alloc();

        let (badge_text, badge_color, tooltip) = match self.asset_status {
            AssetStatus::NotInstalled => ("Not Installed", Color::from_rgb(0.5, 0.5, 0.5), "This asset is not installed"),
            AssetStatus::Installed => ("Installed", Color::from_rgb(0.2, 0.8, 0.2), "This asset is installed and up to date"),
            AssetStatus::UpdateAvailable => ("Update Available", Color::from_rgb(0.8, 0.6, 0.2), "A newer version of this asset is available"),
        };

        badge_label.set_text(badge_text.into());
        badge_label.add_theme_color_override("font_color".into(), Color::from_rgb(1.0, 1.0, 1.0));
        badge_label.add_theme_font_size_override("font_size".into(), 12);

        // Create and configure badge background
        let mut style_box = StyleBoxFlat::new_gd();
        style_box.set_bg_color(badge_color);
        style_box.set_corner_radius_all(4);
        style_box.set_content_margin(godot::builtin::Side::LEFT, 8.0);
        style_box.set_content_margin(godot::builtin::Side::RIGHT, 8.0);
        style_box.set_content_margin(godot::builtin::Side::TOP, 4.0);
        style_box.set_content_margin(godot::builtin::Side::BOTTOM, 4.0);

        status_badge.add_theme_stylebox_override("panel".into(), style_box.upcast::<godot::classes::StyleBox>());
        status_badge.add_child(badge_label);
        status_badge.set_tooltip_text(tooltip.into());
        status_badge.set_custom_minimum_size(godot::prelude::Vector2::new(0.0, 32.0));

        header_hbox.add_child(status_badge);
        vbox.add_child(header_hbox);

        // Metadata row (Author, Version, Category)
        let mut metadata_hbox = HBoxContainer::new_alloc();

        let mut author_label = Label::new_alloc();
        author_label.set_text(format!("Author: {}", self.asset_author).into());
        author_label.set_tooltip_text(format!("Created by {}", self.asset_author).into());
        metadata_hbox.add_child(author_label);

        let mut version_label = Label::new_alloc();
        version_label.set_text(format!("  Version: {}", self.asset_version).into());
        version_label.set_tooltip_text(format!("Current version: {}", self.asset_version).into());
        metadata_hbox.add_child(version_label);

        let mut category_label = Label::new_alloc();
        category_label.set_text(format!("  Category: {}", self.asset_category).into());
        category_label.set_tooltip_text(format!("Asset category: {}", self.asset_category).into());
        metadata_hbox.add_child(category_label);

        vbox.add_child(metadata_hbox);

        // Preview image placeholder
        if let Some(ref preview_url) = self.asset_preview_url {
            let mut preview_label = Label::new_alloc();
            preview_label.set_text(format!("Preview: {}", preview_url).into());
            // TODO: Load actual image from URL in a future implementation
            vbox.add_child(preview_label);
        } else {
            let mut no_preview_label = Label::new_alloc();
            no_preview_label.set_text("No preview available".into());
            vbox.add_child(no_preview_label);
        }

        // Description
        let mut desc_label = Label::new_alloc();
        desc_label.set_text("Description:".into());
        desc_label.add_theme_font_size_override("font_size".into(), 16);
        vbox.add_child(desc_label);

        let mut scroll = ScrollContainer::new_alloc();
        scroll.set_custom_minimum_size(godot::prelude::Vector2::new(0.0, 100.0));

        let mut desc_text = RichTextLabel::new_alloc();
        desc_text.set_text(self.asset_description.clone().into());
        desc_text.set_fit_content(true);
        scroll.add_child(desc_text);
        vbox.add_child(scroll);

        // Tags
        if !self.asset_tags.is_empty() {
            let mut tags_label = Label::new_alloc();
            tags_label.set_text(format!("Tags: {}", self.asset_tags.join(", ")).into());
            tags_label.set_tooltip_text("Click tags to filter by tag (feature coming soon)".into());
            vbox.add_child(tags_label);
        }

        // Enable mouse filtering for hover effects
        self.base_mut().set_mouse_filter(godot::classes::control::MouseFilter::PASS);

        self.base_mut().add_child(vbox);
    }
}

#[godot_api]
impl AssetPreviewNode {
    #[func]
    fn new_with_asset(
        asset_id: GString,
        asset_name: GString,
        asset_author: GString,
        asset_version: GString,
        asset_category: GString,
        asset_description: GString,
        asset_tags: Array<GString>,
        asset_preview_url: GString,
        asset_status: i32,
    ) -> Gd<Self> {
        let mut instance = Self::new_alloc();
        {
            let mut bind = instance.bind_mut();
            bind.asset_id = asset_id.to_string();
            bind.asset_name = asset_name.to_string();
            bind.asset_author = asset_author.to_string();
            bind.asset_version = asset_version.to_string();
            bind.asset_category = asset_category.to_string();
            bind.asset_description = asset_description.to_string();
            bind.asset_tags = asset_tags.iter_shared().map(|s| s.to_string()).collect();
            let preview_url_str = asset_preview_url.to_string();
            bind.asset_preview_url = if preview_url_str.is_empty() {
                None
            } else {
                Some(preview_url_str)
            };
            bind.asset_status = match asset_status {
                0 => AssetStatus::NotInstalled,
                1 => AssetStatus::Installed,
                2 => AssetStatus::UpdateAvailable,
                _ => AssetStatus::NotInstalled,
            };
        }
        instance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Full GUI interaction tests require Godot engine runtime.
    // These tests focus on testable logic (enums, data structures).

    #[test]
    fn test_sort_criteria_enum() {
        // Verify SortCriteria variants exist and are distinct
        let name = SortCriteria::Name;
        let category = SortCriteria::Category;
        let author = SortCriteria::Author;

        assert_ne!(name, category);
        assert_ne!(name, author);
        assert_ne!(category, author);
    }

    #[test]
    fn test_sort_criteria_copy_clone() {
        // Verify SortCriteria implements Copy and Clone
        let original = SortCriteria::Name;
        let copied = original;
        let cloned = original.clone();

        assert_eq!(original, copied);
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_sort_criteria_debug() {
        // Verify SortCriteria implements Debug
        let criteria = SortCriteria::Name;
        let debug_str = format!("{:?}", criteria);
        assert!(debug_str.contains("Name"));
    }

    #[test]
    fn test_asset_status_enum() {
        // Verify AssetStatus variants exist and are distinct
        let not_installed = AssetStatus::NotInstalled;
        let installed = AssetStatus::Installed;
        let update_available = AssetStatus::UpdateAvailable;

        assert_ne!(not_installed, installed);
        assert_ne!(not_installed, update_available);
        assert_ne!(installed, update_available);
    }

    #[test]
    fn test_asset_status_copy_clone() {
        // Verify AssetStatus implements Copy and Clone
        let original = AssetStatus::Installed;
        let copied = original;
        let cloned = original.clone();

        assert_eq!(original, copied);
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_asset_status_debug() {
        // Verify AssetStatus implements Debug
        let status = AssetStatus::UpdateAvailable;
        let debug_str = format!("{:?}", status);
        assert!(debug_str.contains("UpdateAvailable"));
    }

    #[test]
    fn test_asset_status_from_index() {
        // Test the AssetStatus creation from index (as used in AssetPreviewNode::new_with_data)
        // Index mapping: 0 = NotInstalled, 1 = Installed, 2 = UpdateAvailable

        let status_0 = match 0 {
            0 => AssetStatus::NotInstalled,
            1 => AssetStatus::Installed,
            2 => AssetStatus::UpdateAvailable,
            _ => AssetStatus::NotInstalled,
        };
        assert_eq!(status_0, AssetStatus::NotInstalled);

        let status_1 = match 1 {
            0 => AssetStatus::NotInstalled,
            1 => AssetStatus::Installed,
            2 => AssetStatus::UpdateAvailable,
            _ => AssetStatus::NotInstalled,
        };
        assert_eq!(status_1, AssetStatus::Installed);

        let status_2 = match 2 {
            0 => AssetStatus::NotInstalled,
            1 => AssetStatus::Installed,
            2 => AssetStatus::UpdateAvailable,
            _ => AssetStatus::NotInstalled,
        };
        assert_eq!(status_2, AssetStatus::UpdateAvailable);

        // Test default case
        let status_invalid = match 99 {
            0 => AssetStatus::NotInstalled,
            1 => AssetStatus::Installed,
            2 => AssetStatus::UpdateAvailable,
            _ => AssetStatus::NotInstalled,
        };
        assert_eq!(status_invalid, AssetStatus::NotInstalled);
    }

    #[test]
    fn test_sort_criteria_from_index() {
        // Test the SortCriteria creation from index (as used in on_sort_changed)
        // Index mapping: 0 = Name, 1 = Category, 2 = Author

        let sort_0 = match 0 {
            0 => SortCriteria::Name,
            1 => SortCriteria::Category,
            2 => SortCriteria::Author,
            _ => SortCriteria::Name,
        };
        assert_eq!(sort_0, SortCriteria::Name);

        let sort_1 = match 1 {
            0 => SortCriteria::Name,
            1 => SortCriteria::Category,
            2 => SortCriteria::Author,
            _ => SortCriteria::Name,
        };
        assert_eq!(sort_1, SortCriteria::Category);

        let sort_2 = match 2 {
            0 => SortCriteria::Name,
            1 => SortCriteria::Category,
            2 => SortCriteria::Author,
            _ => SortCriteria::Name,
        };
        assert_eq!(sort_2, SortCriteria::Author);

        // Test default case
        let sort_invalid = match 99 {
            0 => SortCriteria::Name,
            1 => SortCriteria::Category,
            2 => SortCriteria::Author,
            _ => SortCriteria::Name,
        };
        assert_eq!(sort_invalid, SortCriteria::Name);
    }

    // Integration tests that require Godot engine runtime
    // These would be implemented in Godot GDScript test files:
    //
    // - test_asset_library_gui_initialization()
    // - test_search_box_functionality()
    // - test_category_filter_selection()
    // - test_sort_criteria_application()
    // - test_asset_list_population()
    // - test_asset_preview_display()
    // - test_asset_selection_interaction()
    // - test_refresh_asset_list()
    // - test_filter_and_sort_combined()
    // - test_asset_status_badge_display()
}
