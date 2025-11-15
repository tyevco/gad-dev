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
