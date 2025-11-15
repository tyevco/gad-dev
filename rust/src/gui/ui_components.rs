use godot::classes::{Control, IControl, IPanel, Label, Panel, ProgressBar, StyleBoxFlat, VBoxContainer};
use godot::prelude::*;
use godot::global::HorizontalAlignment;

/// A loading spinner component that displays a progress indicator
#[derive(GodotClass)]
#[class(base=Control)]
pub struct LoadingSpinner {
    #[base]
    base: Base<Control>,
    message: String,
    progress: f32,
    indeterminate: bool,
}

#[godot_api]
impl IControl for LoadingSpinner {
    fn init(base: Base<Control>) -> Self {
        Self {
            base,
            message: "Loading...".to_string(),
            progress: 0.0,
            indeterminate: true,
        }
    }

    fn ready(&mut self) {
        let mut vbox = VBoxContainer::new_alloc();
        vbox.set_alignment(godot::classes::box_container::AlignmentMode::CENTER);

        // Message label
        let mut label = Label::new_alloc();
        label.set_text(self.message.clone().into());
        label.set_horizontal_alignment(HorizontalAlignment::CENTER);
        vbox.add_child(label);

        // Progress bar
        let mut progress_bar = ProgressBar::new_alloc();
        progress_bar.set_custom_minimum_size(godot::prelude::Vector2::new(200.0, 20.0));

        if self.indeterminate {
            progress_bar.set_value(-1.0);
        } else {
            progress_bar.set_value(self.progress as f64);
        }

        vbox.add_child(progress_bar);

        // Center the content
        self.base_mut().set_anchor(godot::builtin::Side::LEFT, 0.5);
        self.base_mut().set_anchor(godot::builtin::Side::TOP, 0.5);
        self.base_mut().set_anchor(godot::builtin::Side::RIGHT, 0.5);
        self.base_mut().set_anchor(godot::builtin::Side::BOTTOM, 0.5);

        self.base_mut().add_child(vbox);
    }
}

#[godot_api]
impl LoadingSpinner {
    #[func]
    pub fn set_message(&mut self, message: GString) {
        self.message = message.to_string();
    }

    #[func]
    pub fn set_progress(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 100.0);
        self.indeterminate = false;
    }

    #[func]
    pub fn set_indeterminate(&mut self, indeterminate: bool) {
        self.indeterminate = indeterminate;
    }
}

/// An error message display component
#[derive(GodotClass)]
#[class(base=Panel)]
pub struct ErrorMessage {
    #[base]
    base: Base<Panel>,
    error_title: String,
    error_message: String,
}

#[godot_api]
impl IPanel for ErrorMessage {
    fn init(base: Base<Panel>) -> Self {
        Self {
            base,
            error_title: "Error".to_string(),
            error_message: String::new(),
        }
    }

    fn ready(&mut self) {
        // Create red background for error panel
        let mut style_box = StyleBoxFlat::new_gd();
        style_box.set_bg_color(Color::from_rgb(0.8, 0.2, 0.2));
        style_box.set_corner_radius_all(8);
        style_box.set_content_margin(godot::builtin::Side::LEFT, 12.0);
        style_box.set_content_margin(godot::builtin::Side::RIGHT, 12.0);
        style_box.set_content_margin(godot::builtin::Side::TOP, 12.0);
        style_box.set_content_margin(godot::builtin::Side::BOTTOM, 12.0);

        self.base_mut().add_theme_stylebox_override("panel".into(), style_box.upcast::<godot::classes::StyleBox>());

        let mut vbox = VBoxContainer::new_alloc();

        // Error title
        let mut title_label = Label::new_alloc();
        title_label.set_text(self.error_title.clone().into());
        title_label.add_theme_font_size_override("font_size".into(), 18);
        title_label.add_theme_color_override("font_color".into(), Color::from_rgb(1.0, 1.0, 1.0));
        vbox.add_child(title_label);

        // Error message
        let mut message_label = Label::new_alloc();
        message_label.set_text(self.error_message.clone().into());
        message_label.set_autowrap_mode(godot::classes::text_server::AutowrapMode::WORD_SMART);
        message_label.add_theme_color_override("font_color".into(), Color::from_rgb(1.0, 1.0, 1.0));
        vbox.add_child(message_label);

        self.base_mut().add_child(vbox);
    }
}

#[godot_api]
impl ErrorMessage {
    #[func]
    pub fn new_with_message(title: GString, message: GString) -> Gd<Self> {
        let mut instance = Self::new_alloc();
        {
            let mut bind = instance.bind_mut();
            bind.error_title = title.to_string();
            bind.error_message = message.to_string();
        }
        instance
    }

    #[func]
    pub fn set_error(&mut self, title: GString, message: GString) {
        self.error_title = title.to_string();
        self.error_message = message.to_string();
    }
}

/// A confirmation dialog component for destructive actions
#[derive(GodotClass)]
#[class(base=Panel)]
pub struct ConfirmationDialog {
    #[base]
    base: Base<Panel>,
    title: String,
    message: String,
    confirm_text: String,
    cancel_text: String,
}

#[godot_api]
impl IPanel for ConfirmationDialog {
    fn init(base: Base<Panel>) -> Self {
        Self {
            base,
            title: "Confirm Action".to_string(),
            message: "Are you sure you want to proceed?".to_string(),
            confirm_text: "Confirm".to_string(),
            cancel_text: "Cancel".to_string(),
        }
    }

    fn ready(&mut self) {
        use godot::classes::{Button, HBoxContainer};

        // Create styled background
        let mut style_box = StyleBoxFlat::new_gd();
        style_box.set_bg_color(Color::from_rgb(0.2, 0.2, 0.2));
        style_box.set_corner_radius_all(8);
        style_box.set_content_margin(godot::builtin::Side::LEFT, 16.0);
        style_box.set_content_margin(godot::builtin::Side::RIGHT, 16.0);
        style_box.set_content_margin(godot::builtin::Side::TOP, 16.0);
        style_box.set_content_margin(godot::builtin::Side::BOTTOM, 16.0);

        self.base_mut().add_theme_stylebox_override("panel".into(), style_box.upcast::<godot::classes::StyleBox>());

        let mut vbox = VBoxContainer::new_alloc();
        vbox.set_custom_minimum_size(godot::prelude::Vector2::new(300.0, 150.0));

        // Title
        let mut title_label = Label::new_alloc();
        title_label.set_text(self.title.clone().into());
        title_label.add_theme_font_size_override("font_size".into(), 20);
        title_label.add_theme_color_override("font_color".into(), Color::from_rgb(1.0, 1.0, 1.0));
        vbox.add_child(title_label);

        // Message
        let mut message_label = Label::new_alloc();
        message_label.set_text(self.message.clone().into());
        message_label.set_autowrap_mode(godot::classes::text_server::AutowrapMode::WORD_SMART);
        message_label.add_theme_color_override("font_color".into(), Color::from_rgb(0.9, 0.9, 0.9));
        vbox.add_child(message_label);

        // Button row
        let mut button_hbox = HBoxContainer::new_alloc();
        button_hbox.set_alignment(godot::classes::box_container::AlignmentMode::END);

        // Cancel button
        let mut cancel_button = Button::new_alloc();
        cancel_button.set_text(self.cancel_text.clone().into());
        cancel_button.set_custom_minimum_size(godot::prelude::Vector2::new(80.0, 30.0));
        button_hbox.add_child(cancel_button);

        // Confirm button
        let mut confirm_button = Button::new_alloc();
        confirm_button.set_text(self.confirm_text.clone().into());
        confirm_button.set_custom_minimum_size(godot::prelude::Vector2::new(80.0, 30.0));
        button_hbox.add_child(confirm_button);

        vbox.add_child(button_hbox);

        // Center the dialog
        self.base_mut().set_anchor(godot::builtin::Side::LEFT, 0.5);
        self.base_mut().set_anchor(godot::builtin::Side::TOP, 0.5);
        self.base_mut().set_anchor(godot::builtin::Side::RIGHT, 0.5);
        self.base_mut().set_anchor(godot::builtin::Side::BOTTOM, 0.5);

        self.base_mut().add_child(vbox);
    }
}

#[godot_api]
impl ConfirmationDialog {
    #[func]
    pub fn new_with_config(
        title: GString,
        message: GString,
        confirm_text: GString,
        cancel_text: GString,
    ) -> Gd<Self> {
        let mut instance = Self::new_alloc();
        {
            let mut bind = instance.bind_mut();
            bind.title = title.to_string();
            bind.message = message.to_string();
            bind.confirm_text = confirm_text.to_string();
            bind.cancel_text = cancel_text.to_string();
        }
        instance
    }

    #[func]
    pub fn set_config(&mut self, title: GString, message: GString) {
        self.title = title.to_string();
        self.message = message.to_string();
    }
}

/// Theme utilities and constants
pub struct AssetBrowserTheme;

impl AssetBrowserTheme {
    // Color palette
    pub const PRIMARY_COLOR: Color = Color::from_rgb(0.3, 0.5, 0.8);
    pub const SECONDARY_COLOR: Color = Color::from_rgb(0.5, 0.7, 0.9);
    pub const BACKGROUND_COLOR: Color = Color::from_rgb(0.15, 0.15, 0.15);
    pub const SURFACE_COLOR: Color = Color::from_rgb(0.2, 0.2, 0.2);
    pub const ERROR_COLOR: Color = Color::from_rgb(0.8, 0.2, 0.2);
    pub const SUCCESS_COLOR: Color = Color::from_rgb(0.2, 0.8, 0.2);
    pub const WARNING_COLOR: Color = Color::from_rgb(0.8, 0.6, 0.2);
    pub const TEXT_COLOR: Color = Color::from_rgb(0.9, 0.9, 0.9);
    pub const TEXT_SECONDARY_COLOR: Color = Color::from_rgb(0.7, 0.7, 0.7);

    // Spacing
    pub const PADDING_SMALL: f32 = 4.0;
    pub const PADDING_MEDIUM: f32 = 8.0;
    pub const PADDING_LARGE: f32 = 16.0;

    // Border radius
    pub const BORDER_RADIUS_SMALL: i32 = 4;
    pub const BORDER_RADIUS_MEDIUM: i32 = 8;
    pub const BORDER_RADIUS_LARGE: i32 = 12;

    /// Creates a styled panel with the given background color
    pub fn create_styled_panel(bg_color: Color, border_radius: i32) -> Gd<StyleBoxFlat> {
        let mut style_box = StyleBoxFlat::new_gd();
        style_box.set_bg_color(bg_color);
        style_box.set_corner_radius_all(border_radius);
        style_box.set_content_margin(godot::builtin::Side::LEFT, Self::PADDING_MEDIUM);
        style_box.set_content_margin(godot::builtin::Side::RIGHT, Self::PADDING_MEDIUM);
        style_box.set_content_margin(godot::builtin::Side::TOP, Self::PADDING_MEDIUM);
        style_box.set_content_margin(godot::builtin::Side::BOTTOM, Self::PADDING_MEDIUM);
        style_box
    }

    /// Creates a styled button
    pub fn create_styled_button(bg_color: Color) -> Gd<StyleBoxFlat> {
        let mut style_box = StyleBoxFlat::new_gd();
        style_box.set_bg_color(bg_color);
        style_box.set_corner_radius_all(Self::BORDER_RADIUS_SMALL);
        style_box.set_content_margin(godot::builtin::Side::LEFT, Self::PADDING_MEDIUM);
        style_box.set_content_margin(godot::builtin::Side::RIGHT, Self::PADDING_MEDIUM);
        style_box.set_content_margin(godot::builtin::Side::TOP, Self::PADDING_SMALL);
        style_box.set_content_margin(godot::builtin::Side::BOTTOM, Self::PADDING_SMALL);
        style_box
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Most GUI components require Godot engine runtime for full testing.
    // These tests focus on testable logic and constants.

    #[test]
    fn test_asset_browser_theme_color_constants() {
        // Verify color constants are within valid ranges (0.0-1.0)
        assert!(AssetBrowserTheme::PRIMARY_COLOR.r >= 0.0 && AssetBrowserTheme::PRIMARY_COLOR.r <= 1.0);
        assert!(AssetBrowserTheme::PRIMARY_COLOR.g >= 0.0 && AssetBrowserTheme::PRIMARY_COLOR.g <= 1.0);
        assert!(AssetBrowserTheme::PRIMARY_COLOR.b >= 0.0 && AssetBrowserTheme::PRIMARY_COLOR.b <= 1.0);

        assert!(AssetBrowserTheme::SECONDARY_COLOR.r >= 0.0 && AssetBrowserTheme::SECONDARY_COLOR.r <= 1.0);
        assert!(AssetBrowserTheme::BACKGROUND_COLOR.r >= 0.0 && AssetBrowserTheme::BACKGROUND_COLOR.r <= 1.0);
        assert!(AssetBrowserTheme::SURFACE_COLOR.r >= 0.0 && AssetBrowserTheme::SURFACE_COLOR.r <= 1.0);
        assert!(AssetBrowserTheme::ERROR_COLOR.r >= 0.0 && AssetBrowserTheme::ERROR_COLOR.r <= 1.0);
        assert!(AssetBrowserTheme::SUCCESS_COLOR.r >= 0.0 && AssetBrowserTheme::SUCCESS_COLOR.r <= 1.0);
        assert!(AssetBrowserTheme::WARNING_COLOR.r >= 0.0 && AssetBrowserTheme::WARNING_COLOR.r <= 1.0);
        assert!(AssetBrowserTheme::TEXT_COLOR.r >= 0.0 && AssetBrowserTheme::TEXT_COLOR.r <= 1.0);
        assert!(AssetBrowserTheme::TEXT_SECONDARY_COLOR.r >= 0.0 && AssetBrowserTheme::TEXT_SECONDARY_COLOR.r <= 1.0);
    }

    #[test]
    fn test_asset_browser_theme_color_semantics() {
        // Error color should be predominantly red
        assert!(AssetBrowserTheme::ERROR_COLOR.r > 0.5);
        assert!(AssetBrowserTheme::ERROR_COLOR.g < 0.5);
        assert!(AssetBrowserTheme::ERROR_COLOR.b < 0.5);

        // Success color should be predominantly green
        assert!(AssetBrowserTheme::SUCCESS_COLOR.g > 0.5);
        assert!(AssetBrowserTheme::SUCCESS_COLOR.r < 0.5);
        assert!(AssetBrowserTheme::SUCCESS_COLOR.b < 0.5);

        // Warning color should have high red and moderate green (yellow-orange)
        assert!(AssetBrowserTheme::WARNING_COLOR.r > 0.5);
        assert!(AssetBrowserTheme::WARNING_COLOR.g > 0.3);
        assert!(AssetBrowserTheme::WARNING_COLOR.b < 0.5);

        // Background should be dark
        assert!(AssetBrowserTheme::BACKGROUND_COLOR.r < 0.3);
        assert!(AssetBrowserTheme::BACKGROUND_COLOR.g < 0.3);
        assert!(AssetBrowserTheme::BACKGROUND_COLOR.b < 0.3);

        // Text should be light
        assert!(AssetBrowserTheme::TEXT_COLOR.r > 0.8);
        assert!(AssetBrowserTheme::TEXT_COLOR.g > 0.8);
        assert!(AssetBrowserTheme::TEXT_COLOR.b > 0.8);

        // Secondary text should be lighter than background but darker than primary text
        assert!(AssetBrowserTheme::TEXT_SECONDARY_COLOR.r > AssetBrowserTheme::BACKGROUND_COLOR.r);
        assert!(AssetBrowserTheme::TEXT_SECONDARY_COLOR.r < AssetBrowserTheme::TEXT_COLOR.r);
    }

    #[test]
    fn test_asset_browser_theme_spacing_constants() {
        // Verify spacing constants are positive and in ascending order
        assert!(AssetBrowserTheme::PADDING_SMALL > 0.0);
        assert!(AssetBrowserTheme::PADDING_MEDIUM > 0.0);
        assert!(AssetBrowserTheme::PADDING_LARGE > 0.0);

        assert!(AssetBrowserTheme::PADDING_SMALL < AssetBrowserTheme::PADDING_MEDIUM);
        assert!(AssetBrowserTheme::PADDING_MEDIUM < AssetBrowserTheme::PADDING_LARGE);

        // Verify spacing is within reasonable UI bounds
        assert!(AssetBrowserTheme::PADDING_SMALL <= 10.0);
        assert!(AssetBrowserTheme::PADDING_LARGE <= 50.0);
    }

    #[test]
    fn test_asset_browser_theme_border_radius_constants() {
        // Verify border radius constants are positive and in ascending order
        assert!(AssetBrowserTheme::BORDER_RADIUS_SMALL > 0);
        assert!(AssetBrowserTheme::BORDER_RADIUS_MEDIUM > 0);
        assert!(AssetBrowserTheme::BORDER_RADIUS_LARGE > 0);

        assert!(AssetBrowserTheme::BORDER_RADIUS_SMALL < AssetBrowserTheme::BORDER_RADIUS_MEDIUM);
        assert!(AssetBrowserTheme::BORDER_RADIUS_MEDIUM < AssetBrowserTheme::BORDER_RADIUS_LARGE);

        // Verify border radius is within reasonable UI bounds
        assert!(AssetBrowserTheme::BORDER_RADIUS_SMALL <= 10);
        assert!(AssetBrowserTheme::BORDER_RADIUS_LARGE <= 50);
    }

    #[test]
    fn test_loading_spinner_state_initialization() {
        // Test that LoadingSpinner initializes with correct defaults
        // Note: Cannot test Godot node creation without engine, but can document expected behavior

        // Expected defaults:
        // - message: "Loading..."
        // - progress: 0.0
        // - indeterminate: true

        // This is a documentation test - actual testing requires Godot engine
    }

    #[test]
    fn test_loading_spinner_progress_clamping() {
        // Document expected behavior for progress clamping
        // set_progress should clamp values to 0.0-100.0 range

        // Expected behavior when set_progress is called:
        // - Values < 0.0 should become 0.0
        // - Values > 100.0 should become 100.0
        // - Values in range stay unchanged
        // - indeterminate should be set to false

        // Actual testing requires Godot engine runtime
    }

    #[test]
    fn test_error_message_initialization() {
        // Test that ErrorMessage initializes with correct defaults
        // Note: Cannot test Godot node creation without engine

        // Expected defaults:
        // - error_title: "Error"
        // - error_message: empty string

        // This is a documentation test - actual testing requires Godot engine
    }

    #[test]
    fn test_confirmation_dialog_initialization() {
        // Test that ConfirmationDialog initializes with correct defaults
        // Note: Cannot test Godot node creation without engine

        // Expected defaults:
        // - title: "Confirm Action"
        // - message: "Are you sure you want to proceed?"
        // - confirm_text: "Confirm"
        // - cancel_text: "Cancel"

        // This is a documentation test - actual testing requires Godot engine
    }

    #[test]
    fn test_color_rgb_constructor_compatibility() {
        // Verify that Color::from_rgb creates valid colors
        let color = Color::from_rgb(0.5, 0.7, 0.9);
        assert_eq!(color.r, 0.5);
        assert_eq!(color.g, 0.7);
        assert_eq!(color.b, 0.9);
        assert_eq!(color.a, 1.0); // Alpha should default to 1.0
    }

    #[test]
    fn test_theme_constants_consistency() {
        // Verify that theme constants have consistent patterns

        // Primary and secondary colors should be related (blue tones)
        assert!(AssetBrowserTheme::PRIMARY_COLOR.b > AssetBrowserTheme::PRIMARY_COLOR.r);
        assert!(AssetBrowserTheme::SECONDARY_COLOR.b > AssetBrowserTheme::SECONDARY_COLOR.r);

        // Secondary should be lighter than primary
        assert!(AssetBrowserTheme::SECONDARY_COLOR.r >= AssetBrowserTheme::PRIMARY_COLOR.r);
        assert!(AssetBrowserTheme::SECONDARY_COLOR.g >= AssetBrowserTheme::PRIMARY_COLOR.g);
        assert!(AssetBrowserTheme::SECONDARY_COLOR.b >= AssetBrowserTheme::PRIMARY_COLOR.b);
    }

    #[test]
    fn test_spacing_proportions() {
        // Verify spacing follows reasonable proportions
        let ratio_medium_to_small = AssetBrowserTheme::PADDING_MEDIUM / AssetBrowserTheme::PADDING_SMALL;
        let ratio_large_to_medium = AssetBrowserTheme::PADDING_LARGE / AssetBrowserTheme::PADDING_MEDIUM;

        // Ratios should be similar (geometric progression)
        assert!(ratio_medium_to_small >= 1.5);
        assert!(ratio_medium_to_small <= 3.0);
        assert!(ratio_large_to_medium >= 1.5);
        assert!(ratio_large_to_medium <= 3.0);
    }

    #[test]
    fn test_border_radius_proportions() {
        // Verify border radius follows reasonable proportions
        let ratio_medium_to_small = AssetBrowserTheme::BORDER_RADIUS_MEDIUM as f32 / AssetBrowserTheme::BORDER_RADIUS_SMALL as f32;
        let ratio_large_to_medium = AssetBrowserTheme::BORDER_RADIUS_LARGE as f32 / AssetBrowserTheme::BORDER_RADIUS_MEDIUM as f32;

        // Ratios should be similar (geometric progression)
        assert!(ratio_medium_to_small >= 1.5);
        assert!(ratio_medium_to_small <= 3.0);
        assert!(ratio_large_to_medium >= 1.2);
        assert!(ratio_large_to_medium <= 2.0);
    }

    // Integration tests that require Godot engine runtime
    // These would be implemented in Godot GDScript test files:
    //
    // - test_create_styled_panel_visual()
    // - test_create_styled_button_visual()
    // - test_loading_spinner_display()
    // - test_error_message_display()
    // - test_confirmation_dialog_interaction()
    // - test_loading_spinner_progress_updates()
    // - test_theme_application_to_nodes()
}
