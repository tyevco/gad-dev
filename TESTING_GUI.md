# GUI Testing Guide

## Overview

GUI testing for the Godot Asset Browser is split into two categories:

1. **Rust Unit Tests** - Test logic, data structures, and non-Godot-dependent code
2. **Godot Integration Tests** - Test actual GUI interactions and visual behavior (requires Godot engine)

## Rust Unit Tests

Located in:
- `rust/src/gui/ui_components.rs` (12 tests)
- `rust/src/gui/asset_library_gui.rs` (8 tests)

### Coverage

#### Theme & Styling Tests
- Color constant validation (valid RGB ranges, semantic correctness)
- Spacing constant validation (positive values, geometric progression)
- Border radius validation and proportions
- Theme consistency checks

#### Data Structure Tests
- `SortCriteria` enum (Name, Category, Author)
- `AssetStatus` enum (NotInstalled, Installed, UpdateAvailable)
- Enum trait implementations (Copy, Clone, Debug, PartialEq, Eq)
- Index-to-enum conversion logic

### Running Rust Tests

```bash
cd rust
cargo test --lib gui::
```

Expected output: **20 tests passing**

## Godot Integration Tests

Full GUI interaction testing requires the Godot engine runtime. These tests should be implemented using Godot's built-in testing framework (GDScript or GUT - Godot Unit Test).

### Recommended Test Structure

Create a `tests/` directory in the Godot project with GDScript test files:

```
godot_project/
├── tests/
│   ├── test_asset_library_gui.gd
│   ├── test_ui_components.gd
│   └── test_asset_interactions.gd
```

### Example GDScript Tests

#### test_ui_components.gd

```gdscript
extends GutTest

var loading_spinner
var error_message
var confirmation_dialog

func before_each():
    # Set up test components
    loading_spinner = LoadingSpinner.new()
    error_message = ErrorMessage.new()
    confirmation_dialog = ConfirmationDialog.new()
    add_child_autofree(loading_spinner)
    add_child_autofree(error_message)
    add_child_autofree(confirmation_dialog)

func test_loading_spinner_displays_message():
    loading_spinner.set_message("Loading assets...")
    await get_tree().process_frame
    var label = loading_spinner.get_node("VBoxContainer/Label")
    assert_eq(label.text, "Loading assets...", "Spinner should display custom message")

func test_loading_spinner_progress_updates():
    loading_spinner.set_progress(50.0)
    await get_tree().process_frame
    var progress_bar = loading_spinner.get_node("VBoxContainer/ProgressBar")
    assert_eq(progress_bar.value, 50.0, "Progress bar should show 50%")
    assert_false(loading_spinner.indeterminate, "Should not be indeterminate when progress is set")

func test_loading_spinner_progress_clamping():
    # Test upper bound
    loading_spinner.set_progress(150.0)
    assert_eq(loading_spinner.progress, 100.0, "Progress should clamp to 100")

    # Test lower bound
    loading_spinner.set_progress(-50.0)
    assert_eq(loading_spinner.progress, 0.0, "Progress should clamp to 0")

func test_error_message_styling():
    error_message.set_error("Test Error", "This is a test error message")
    await get_tree().process_frame

    var panel_style = error_message.get_theme_stylebox("panel")
    assert_not_null(panel_style, "Error message should have styled panel")
    assert_eq(panel_style.bg_color.r, 0.8, "Error background should be red")

func test_confirmation_dialog_buttons():
    await get_tree().process_frame

    var button_container = confirmation_dialog.get_node("VBoxContainer/HBoxContainer")
    assert_eq(button_container.get_child_count(), 2, "Dialog should have 2 buttons")

    var cancel_btn = button_container.get_child(0)
    var confirm_btn = button_container.get_child(1)

    assert_eq(cancel_btn.text, "Cancel", "First button should be Cancel")
    assert_eq(confirm_btn.text, "Confirm", "Second button should be Confirm")

func test_theme_color_application():
    var panel = Panel.new()
    add_child_autofree(panel)

    # Test applying theme colors
    var style = AssetBrowserTheme.create_styled_panel(
        AssetBrowserTheme.PRIMARY_COLOR,
        AssetBrowserTheme.BORDER_RADIUS_MEDIUM
    )

    panel.add_theme_stylebox_override("panel", style)
    await get_tree().process_frame

    var applied_style = panel.get_theme_stylebox("panel")
    assert_not_null(applied_style, "Panel should have themed style")
```

#### test_asset_library_gui.gd

```gdscript
extends GutTest

var asset_library_gui

func before_each():
    asset_library_gui = AssetLibraryGUI.new()
    add_child_autofree(asset_library_gui)

func test_search_box_filters_assets():
    await get_tree().process_frame

    # Simulate search input
    asset_library_gui.on_search_changed("shader")
    await get_tree().process_frame

    # Verify filtered results
    var tree = asset_library_gui.asset_list
    var root = tree.get_root()
    var child_count = root.get_child_count()

    # Should only show assets matching "shader"
    for i in range(child_count):
        var item = root.get_child(i)
        var item_text = item.get_text(0).to_lower()
        assert_true(
            "shader" in item_text,
            "All displayed items should match search query"
        )

func test_category_filter_selection():
    await get_tree().process_frame

    # Select "2D" category (index 1, since 0 is "All")
    asset_library_gui.on_category_changed(1)
    await get_tree().process_frame

    # Verify only 2D assets are shown
    var tree = asset_library_gui.asset_list
    var root = tree.get_root()

    for i in range(root.get_child_count()):
        var item = root.get_child(i)
        var category = item.get_metadata(0).category
        assert_eq(category, AssetCategory.TwoD, "Only 2D assets should be displayed")

func test_sort_by_name():
    await get_tree().process_frame

    # Select "Name" sort (index 0)
    asset_library_gui.on_sort_changed(0)
    await get_tree().process_frame

    var tree = asset_library_gui.asset_list
    var root = tree.get_root()

    # Verify assets are sorted alphabetically
    var prev_name = ""
    for i in range(root.get_child_count()):
        var item = root.get_child(i)
        var name = item.get_text(0).to_lower()
        assert_true(
            name >= prev_name,
            "Assets should be sorted alphabetically by name"
        )
        prev_name = name

func test_sort_by_category():
    await get_tree().process_frame

    # Select "Category" sort (index 1)
    asset_library_gui.on_sort_changed(1)
    await get_tree().process_frame

    var tree = asset_library_gui.asset_list
    var root = tree.get_root()

    # Verify assets are sorted by category
    var prev_category = ""
    for i in range(root.get_child_count()):
        var item = root.get_child(i)
        var category = item.get_metadata(0).category.display_name()
        assert_true(
            category >= prev_category,
            "Assets should be sorted by category"
        )
        prev_category = category

func test_combined_filter_and_sort():
    await get_tree().process_frame

    # Apply both search and category filter
    asset_library_gui.on_search_changed("material")
    asset_library_gui.on_category_changed(2)  # 3D category
    asset_library_gui.on_sort_changed(0)  # Sort by name
    await get_tree().process_frame

    var tree = asset_library_gui.asset_list
    var root = tree.get_root()

    # Verify results match both filters and are sorted
    var prev_name = ""
    for i in range(root.get_child_count()):
        var item = root.get_child(i)
        var name = item.get_text(0).to_lower()
        var category = item.get_metadata(0).category

        assert_true("material" in name, "Should match search filter")
        assert_eq(category, AssetCategory.ThreeD, "Should match category filter")
        assert_true(name >= prev_name, "Should be sorted by name")
        prev_name = name

func test_asset_status_badges():
    await get_tree().process_frame

    var tree = asset_library_gui.asset_list
    var root = tree.get_root()

    # Check for status badges on installed assets
    for i in range(root.get_child_count()):
        var item = root.get_child(i)
        var asset = item.get_metadata(0)
        var badge_text = item.get_text(1)  # Status column

        if asset.is_installed:
            assert_true(
                "Installed" in badge_text or "Update Available" in badge_text,
                "Installed assets should have status badge"
            )
```

#### test_asset_interactions.gd

```gdscript
extends GutTest

var asset_library_gui

func before_each():
    asset_library_gui = AssetLibraryGUI.new()
    add_child_autofree(asset_library_gui)
    await get_tree().process_frame

func test_asset_preview_click():
    var preview_node = AssetPreviewNode.new_with_data(
        "Test Asset",
        "Test Category",
        "Test Author",
        "https://example.com/preview.png",
        0  # NotInstalled status
    )
    add_child_autofree(preview_node)
    await get_tree().process_frame

    # Simulate click
    var click_event = InputEventMouseButton.new()
    click_event.button_index = MOUSE_BUTTON_LEFT
    click_event.pressed = true
    preview_node._gui_input(click_event)

    # Verify asset detail panel is shown
    # (Implementation depends on signal connections)
    assert_true(preview_node.is_selected, "Preview should be selected on click")

func test_download_button_interaction():
    # Test that download button triggers download
    # This would require mocking the AssetManager
    pass

func test_install_button_state():
    # Test that install button is disabled during download
    # Test that install button is enabled when download completes
    pass
```

## Running Godot Integration Tests

### Using GUT (Godot Unit Test)

1. Install GUT addon: https://github.com/bitwes/Gut
2. Configure GUT in your project
3. Run tests via GUT panel or command line:

```bash
godot --path . -s addons/gut/gut_cmdln.gd -gtest=tests/
```

### Using Godot's Built-in Testing

Godot 4.0+ has built-in unit testing support:

```bash
godot --path . --headless --script tests/test_ui_components.gd
```

## Test Coverage Goals

### Rust Tests (Achieved)
- ✅ 20 tests covering theme constants, data structures, and enum logic
- ✅ All tests passing without requiring Godot engine

### Godot Integration Tests (Recommended)
- UI component initialization and display
- User interaction simulation (clicks, input, keyboard)
- Visual regression testing (screenshots comparison)
- Accessibility testing (keyboard navigation, focus)
- Performance testing (large asset lists, scrolling)
- State management (filter combinations, search persistence)

## Continuous Integration

For CI/CD pipelines:

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  rust-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Run Rust tests
        run: cd rust && cargo test --lib

  godot-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Setup Godot
        uses: godotengine/godot-action@v1
      - name: Run Godot tests
        run: godot --path . --headless --script tests/run_all.gd
```

## Contributing

When adding new GUI features:

1. **Add Rust tests** for any non-Godot logic (data structures, enums, utilities)
2. **Document expected behavior** in code comments if full testing requires Godot
3. **Create GDScript tests** for user interactions and visual behavior
4. **Ensure all tests pass** before submitting PR

## Resources

- [GUT Documentation](https://github.com/bitwes/Gut/wiki)
- [Godot Testing Best Practices](https://docs.godotengine.org/en/stable/contributing/development/testing/introduction.html)
- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
