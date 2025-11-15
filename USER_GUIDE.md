# Godot Asset Browser - User Guide

## Table of Contents
1. [Installation](#installation)
2. [Getting Started](#getting-started)
3. [Browsing Assets](#browsing-assets)
4. [Installing Assets](#installing-assets)
5. [Managing Assets](#managing-assets)
6. [Advanced Features](#advanced-features)
7. [Troubleshooting](#troubleshooting)

---

## Installation

### Prerequisites
- Godot Engine 4.0 or higher
- Operating System: Windows 10+, macOS 11+, or Linux (Ubuntu 20.04+)

### Installing the Asset Browser

1. **Download the Plugin**
   - Download the latest release from the [Releases page](https://github.com/your-repo/godot-asset-browser/releases)
   - Or clone from GitHub: `git clone https://github.com/your-repo/godot-asset-browser.git`

<!-- SCREENSHOT PLACEHOLDER: releases_page.png
   Description: GitHub releases page showing the latest version download button
   Should show: Version number, download assets (.zip), release notes preview
-->

2. **Install in Godot Project**
   - Extract the downloaded ZIP file
   - Copy the `addons/godot_asset_browser` folder to your project's `addons/` directory
   - If the `addons/` folder doesn't exist, create it in your project root

<!-- SCREENSHOT PLACEHOLDER: addon_folder_structure.png
   Description: File explorer showing the addon folder structure
   Should show:
   - project/
     - addons/
       - godot_asset_browser/
         - rust/
         - plugin.cfg
         - asset_library_gui.gdextension
-->

3. **Enable the Plugin**
   - Open your Godot project
   - Go to **Project → Project Settings → Plugins**
   - Find "Godot Asset Browser" in the list
   - Check the "Enable" checkbox

<!-- SCREENSHOT PLACEHOLDER: enable_plugin.png
   Description: Godot Project Settings - Plugins tab
   Should show: List of plugins with "Godot Asset Browser" and enabled checkbox highlighted
-->

4. **Verify Installation**
   - The Asset Browser tab should appear in the bottom panel (next to Output, Debugger, etc.)
   - If you don't see it, restart Godot

<!-- SCREENSHOT PLACEHOLDER: asset_browser_tab.png
   Description: Godot bottom panel showing the Asset Browser tab
   Should show: Bottom panel tabs with "Asset Browser" highlighted
-->

---

## Getting Started

### Opening the Asset Browser

The Asset Browser can be accessed from the bottom panel of the Godot editor.

**Location:** Bottom Panel → Asset Browser tab

<!-- SCREENSHOT PLACEHOLDER: asset_browser_panel_location.png
   Description: Full Godot editor with Asset Browser panel location highlighted
   Should show: Complete editor interface with red box/arrow pointing to Asset Browser tab
-->

### Interface Overview

The Asset Browser interface consists of several key components:

<!-- SCREENSHOT PLACEHOLDER: interface_overview.png
   Description: Asset Browser interface with numbered labels
   Should show:
   1. Search box
   2. Category filter dropdown
   3. Sort options dropdown
   4. Asset grid/list
   5. Asset detail panel
   6. Action buttons (Install, Update, Remove)
-->

#### 1. **Search Bar**
Located at the top, allows you to search for assets by name, description, or tags.

#### 2. **Category Filter**
Filter assets by category:
- All Categories
- 2D Assets
- 3D Assets
- Shaders
- Audio
- Scripts
- Templates
- Tools
- Addons

<!-- SCREENSHOT PLACEHOLDER: category_filter.png
   Description: Category filter dropdown expanded
   Should show: Dropdown menu with all category options listed
-->

#### 3. **Sort Options**
Sort assets by:
- Name (A-Z)
- Category
- Author

<!-- SCREENSHOT PLACEHOLDER: sort_options.png
   Description: Sort dropdown expanded
   Should show: Dropdown menu with sort options
-->

#### 4. **Asset Grid**
Displays assets as cards with:
- Preview thumbnail
- Asset name
- Category badge
- Author name
- Installation status badge

<!-- SCREENSHOT PLACEHOLDER: asset_grid.png
   Description: Asset grid showing multiple asset cards
   Should show: Grid layout with 6-8 asset cards, various categories and statuses
-->

---

## Browsing Assets

### Searching for Assets

1. Click in the **search box** at the top
2. Type your search query (e.g., "platformer", "terrain", "particle")
3. Results update automatically as you type

**Search Tips:**
- Search by asset name: "Low Poly Trees"
- Search by keywords: "medieval weapons"
- Search by tags: "pixel-art character"

<!-- SCREENSHOT PLACEHOLDER: search_results.png
   Description: Search box with query "platformer" and filtered results below
   Should show: Search box with text, results showing only platformer-related assets
-->

### Filtering by Category

1. Click the **Category** dropdown
2. Select a category (e.g., "3D Assets")
3. The asset list updates to show only assets in that category

You can combine search with category filtering for more precise results.

<!-- SCREENSHOT PLACEHOLDER: category_filtered_results.png
   Description: Category filter set to "Shaders" with relevant results
   Should show: Category dropdown showing "Shaders" selected, shader assets displayed
-->

### Sorting Assets

1. Click the **Sort by** dropdown
2. Choose your preferred sort order:
   - **Name**: Alphabetical order (A-Z)
   - **Category**: Grouped by asset type
   - **Author**: Alphabetical by creator name

<!-- SCREENSHOT PLACEHOLDER: sorted_assets.png
   Description: Assets sorted by category
   Should show: Assets grouped and labeled by category (2D, 3D, etc.)
-->

### Viewing Asset Details

Click on any asset card to view detailed information:

- **Full Description**: Detailed explanation of what the asset includes
- **Version Information**: Current version and version history
- **Author Details**: Creator name and profile
- **Tags**: All applicable tags for searching
- **Dependencies**: Required assets or plugins
- **Preview Images**: Gallery of screenshots (if available)
- **File Size**: Download size
- **License**: Usage terms and conditions

<!-- SCREENSHOT PLACEHOLDER: asset_details_panel.png
   Description: Asset detail panel showing all information
   Should show:
   - Asset name as title
   - Large preview image
   - Description text
   - Version number
   - Author name
   - Tags as chips/badges
   - File size
   - Action buttons at bottom
-->

---

## Installing Assets

### Installing a Single Asset

1. **Find the asset** you want to install (using search/filter)
2. **Click on the asset card** to view details
3. **Click the "Install" button**
4. **Wait for download** - Progress bar shows download status
5. **Installation completes** - Asset files are extracted to your project

<!-- SCREENSHOT PLACEHOLDER: install_button.png
   Description: Asset detail view with Install button highlighted
   Should show: Asset card with green "Install" button in prominent position
-->

<!-- SCREENSHOT PLACEHOLDER: download_progress.png
   Description: Download progress indicator
   Should show: Progress bar with percentage, file size downloaded/total, speed
-->

<!-- SCREENSHOT PLACEHOLDER: installation_complete.png
   Description: Successfully installed asset with confirmation
   Should show: Asset card with "Installed" badge, success notification
-->

**What Happens During Installation:**
1. Asset is downloaded to cache
2. Archive is extracted and validated
3. Files are copied to `res://addons/[asset-name]/`
4. Asset is registered as installed
5. Godot's file system rescans automatically

### Installation Location

Assets are installed to: `res://addons/[asset-id]/`

Example: An asset with ID "terrain-toolkit" installs to:
```
your-project/
  addons/
    terrain-toolkit/
      scenes/
      scripts/
      textures/
      asset.json
```

### Checking Installation Status

Asset cards display status badges:
- **No badge**: Not installed
- **"Installed"** (green): Currently installed
- **"Update Available"** (orange): New version available

<!-- SCREENSHOT PLACEHOLDER: status_badges.png
   Description: Asset grid showing different status badges
   Should show: Multiple assets with different status indicators highlighted
-->

---

## Managing Assets

### Viewing Installed Assets

To see only your installed assets:
1. Use the search box to filter
2. Or scroll through the list - installed assets show "Installed" badge

<!-- SCREENSHOT PLACEHOLDER: installed_assets_view.png
   Description: Asset grid filtered to show only installed assets
   Should show: Several asset cards, all with "Installed" badge
-->

### Updating Assets

When an update is available:
1. The asset card shows **"Update Available"** badge
2. Click on the asset to view details
3. Click the **"Update"** button
4. The new version downloads and replaces the old one

**Note:** Your existing asset files will be backed up automatically before updating.

<!-- SCREENSHOT PLACEHOLDER: update_available.png
   Description: Asset card with "Update Available" badge
   Should show: Asset card with orange "Update Available" badge and version info
-->

<!-- SCREENSHOT PLACEHOLDER: update_confirmation.png
   Description: Update confirmation dialog
   Should show: Dialog with version comparison (1.0.0 → 1.2.0), Update/Cancel buttons
-->

### Uninstalling Assets

To remove an installed asset:
1. Click on the asset card
2. Click the **"Uninstall"** button
3. **Confirm removal** in the dialog
4. Asset files are deleted from your project

**Warning:** Uninstalling an asset removes all its files. Make sure it's not being used in your scenes!

<!-- SCREENSHOT PLACEHOLDER: uninstall_confirmation.png
   Description: Uninstall confirmation dialog
   Should show: Warning message, asset name, Uninstall/Cancel buttons
-->

### Managing Dependencies

Some assets require other assets to function. The Asset Browser handles this automatically:

1. When installing an asset with dependencies, you'll see a list
2. Choose to install dependencies automatically or skip
3. Dependencies are downloaded and installed in the correct order

<!-- SCREENSHOT PLACEHOLDER: dependency_dialog.png
   Description: Dependency installation dialog
   Should show: List of required dependencies with checkboxes, Install All/Cancel buttons
-->

---

## Advanced Features

### Bulk Operations

#### Installing Multiple Assets
1. Select multiple assets (Ctrl+Click or Cmd+Click)
2. Click **"Install Selected"** button
3. All selected assets download in parallel

<!-- SCREENSHOT PLACEHOLDER: bulk_install.png
   Description: Multiple assets selected with bulk action buttons
   Should show: Asset grid with 3-4 selected cards (highlighted), "Install Selected" button
-->

#### Updating Multiple Assets
1. Filter to show assets with updates available
2. Select assets to update
3. Click **"Update Selected"**

<!-- SCREENSHOT PLACEHOLDER: bulk_update.png
   Description: Update all button with multiple updates available
   Should show: List of updatable assets, "Update All" button highlighted
-->

### Favorites and Collections

#### Adding to Favorites
1. Click on an asset
2. Click the **star icon** to favorite
3. Favorited assets appear in your Favorites list

<!-- SCREENSHOT PLACEHOLDER: favorite_button.png
   Description: Asset detail with favorite star button
   Should show: Asset card with star icon (unfilled and filled states)
-->

#### Creating Collections
1. Click **"Collections"** in the menu
2. Click **"New Collection"**
3. Name your collection (e.g., "Level Design Tools")
4. Drag and drop assets into the collection

<!-- SCREENSHOT PLACEHOLDER: collections_view.png
   Description: Collections panel showing custom collections
   Should show: Sidebar with collection names, main panel with assets in selected collection
-->

### Asset Sources

#### Adding Custom Asset Sources
You can add custom asset repositories:

1. Go to **Settings → Asset Sources**
2. Click **"Add Source"**
3. Enter the source URL
4. Optional: Add API key for private repositories

<!-- SCREENSHOT PLACEHOLDER: add_asset_source.png
   Description: Add asset source dialog
   Should show: URL input field, API key field, Add/Cancel buttons
-->

**Supported Sources:**
- Official Godot Asset Library (default)
- Custom JSON repositories
- Private company asset servers
- Local file directories

#### Managing Asset Sources
- **Enable/Disable** sources without removing them
- **Set priority** for which source to check first
- **View statistics** (total assets, last sync, health status)

<!-- SCREENSHOT PLACEHOLDER: asset_sources_management.png
   Description: Asset sources management panel
   Should show: Table with sources, toggle switches, priority arrows, status indicators
-->

### History and Activity

View your asset activity:
- Installation history
- Download history
- Update history

<!-- SCREENSHOT PLACEHOLDER: activity_history.png
   Description: Activity/history panel
   Should show: Timeline of actions with dates, asset names, action types (install/update/remove)
-->

### Offline Mode

Assets can be cached for offline use:
1. **Settings → Cache → Enable Offline Mode**
2. Downloaded assets remain available without internet
3. Install pre-downloaded assets from cache

<!-- SCREENSHOT PLACEHOLDER: offline_mode.png
   Description: Offline mode indicator and settings
   Should show: Settings panel with offline mode toggle, cache size display
-->

---

## Troubleshooting

### Common Issues

#### Asset Browser Tab Not Visible
**Solution:**
1. Go to **Project → Project Settings → Plugins**
2. Verify "Godot Asset Browser" is enabled
3. Restart Godot editor

#### Download Fails
**Possible causes:**
- No internet connection
- Server temporarily unavailable
- Firewall blocking connection

**Solution:**
1. Check your internet connection
2. Try again in a few minutes
3. Check firewall settings

#### Installation Fails
**Possible causes:**
- Insufficient disk space
- Corrupted download
- Permission issues

**Solution:**
1. Check available disk space
2. Clear cache: **Settings → Cache → Clear Cache**
3. Try downloading again

<!-- SCREENSHOT PLACEHOLDER: error_message.png
   Description: Error message display
   Should show: Red error panel with clear message and "Retry" button
-->

#### Asset Not Working After Installation
**Solution:**
1. Verify the asset is compatible with your Godot version
2. Check dependencies are installed
3. Restart Godot to refresh the file system
4. Check asset documentation for setup instructions

### Cache Management

Clear the download cache to free up space:
1. **Settings → Cache**
2. Click **"Clear Cache"**
3. Confirm the action

**Note:** This only removes downloaded archives, not installed assets.

<!-- SCREENSHOT PLACEHOLDER: cache_management.png
   Description: Cache management panel
   Should show: Cache size, number of cached files, Clear Cache button
-->

### Reporting Issues

If you encounter a bug:
1. **Help → Report Issue** in the Asset Browser
2. Or visit: https://github.com/your-repo/godot-asset-browser/issues
3. Include:
   - Godot version
   - Operating system
   - Error message (if any)
   - Steps to reproduce

---

## Keyboard Shortcuts

| Action | Windows/Linux | macOS |
|--------|--------------|-------|
| Open Asset Browser | `Ctrl+Shift+A` | `Cmd+Shift+A` |
| Search | `Ctrl+F` | `Cmd+F` |
| Refresh Asset List | `F5` | `F5` |
| View Asset Details | `Enter` | `Enter` |
| Install Selected | `Ctrl+Enter` | `Cmd+Enter` |
| Navigate Assets | `Arrow Keys` | `Arrow Keys` |
| Select Multiple | `Ctrl+Click` | `Cmd+Click` |
| Select Range | `Shift+Click` | `Shift+Click` |

<!-- SCREENSHOT PLACEHOLDER: keyboard_shortcuts.png
   Description: Keyboard shortcuts overlay/help
   Should show: Visual guide with keyboard keys and their functions
-->

---

## Tips and Best Practices

### 🎯 Discovery Tips
- **Browse by category** before searching to discover new assets
- **Check update dates** to find actively maintained assets
- **Read reviews and ratings** to find quality assets
- **Preview screenshots** carefully before installing

### 📦 Installation Tips
- **Install dependencies first** for assets that require them
- **Test in a separate project** before using in production
- **Keep backups** before updating critical assets
- **Read documentation** provided with the asset

### 🧹 Maintenance Tips
- **Regularly check for updates** to get bug fixes and features
- **Remove unused assets** to keep your project clean
- **Clear cache periodically** to free up disk space
- **Organize with collections** for large asset libraries

### ⚡ Performance Tips
- **Enable caching** for frequently used assets
- **Use offline mode** when working without internet
- **Limit concurrent downloads** to maintain editor responsiveness
- **Close asset browser** when not in use to save memory

---

## FAQ

### Q: Can I use multiple asset sources simultaneously?
**A:** Yes! Add multiple sources in Settings → Asset Sources. The browser will show assets from all enabled sources.

### Q: Are my installed assets tracked in version control?
**A:** Yes, assets are installed to your project's `addons/` folder, which should be committed to version control (Git, SVN, etc.).

### Q: Can I install different versions of the same asset?
**A:** No, only one version can be installed at a time. However, backups are created during updates.

### Q: What happens to my scenes if I uninstall an asset they use?
**A:** Godot will show missing resource warnings. Always ensure an asset isn't in use before uninstalling.

### Q: Can I share my collections with team members?
**A:** Yes! Collections can be exported as JSON files and shared. Go to Collections → Export.

### Q: Does the Asset Browser support private/commercial asset libraries?
**A:** Yes! Add your private repository URL and authentication credentials in Asset Sources settings.

### Q: How do I update the Asset Browser plugin itself?
**A:** Download the new version and replace the existing `addons/godot_asset_browser/` folder. Restart Godot.

### Q: Can I rate and review assets?
**A:** Yes, if the asset source supports it. The official Godot Asset Library supports ratings and comments.

---

## Additional Resources

- **Official Documentation**: https://docs.your-project.com
- **Tutorial Videos**: https://youtube.com/your-channel
- **Community Forum**: https://forum.your-project.com
- **Discord Server**: https://discord.gg/your-server
- **Bug Reports**: https://github.com/your-repo/issues
- **Feature Requests**: https://github.com/your-repo/discussions

---

## Credits

**Developed by:** Your Name/Team
**Godot Integration:** godot-rust/gdext
**License:** MIT License

---

**Version:** 1.0.0
**Last Updated:** 2024

---

## Screenshot Capture Checklist

When the GUI is fully implemented, capture these screenshots:

### Installation & Setup (8 screenshots)
- [ ] `releases_page.png` - GitHub releases download
- [ ] `addon_folder_structure.png` - File structure in explorer
- [ ] `enable_plugin.png` - Plugin settings panel
- [ ] `asset_browser_tab.png` - Bottom panel with tab

### Interface Overview (12 screenshots)
- [ ] `asset_browser_panel_location.png` - Full editor with location
- [ ] `interface_overview.png` - Labeled interface components
- [ ] `category_filter.png` - Category dropdown expanded
- [ ] `sort_options.png` - Sort dropdown expanded
- [ ] `asset_grid.png` - Grid with multiple assets
- [ ] `search_results.png` - Search in action
- [ ] `category_filtered_results.png` - Category filter applied
- [ ] `sorted_assets.png` - Sorted asset display
- [ ] `asset_details_panel.png` - Full detail view

### Asset Management (15 screenshots)
- [ ] `status_badges.png` - Different status indicators
- [ ] `install_button.png` - Install button highlighted
- [ ] `download_progress.png` - Progress indicator
- [ ] `installation_complete.png` - Success state
- [ ] `installed_assets_view.png` - Filtered installed view
- [ ] `update_available.png` - Update badge
- [ ] `update_confirmation.png` - Update dialog
- [ ] `uninstall_confirmation.png` - Uninstall dialog
- [ ] `dependency_dialog.png` - Dependency list
- [ ] `bulk_install.png` - Multiple selection
- [ ] `bulk_update.png` - Update all button
- [ ] `favorite_button.png` - Star icon states
- [ ] `collections_view.png` - Collections panel
- [ ] `add_asset_source.png` - Add source dialog
- [ ] `asset_sources_management.png` - Sources table

### Advanced Features (5 screenshots)
- [ ] `activity_history.png` - Activity timeline
- [ ] `offline_mode.png` - Offline settings
- [ ] `error_message.png` - Error display
- [ ] `cache_management.png` - Cache settings
- [ ] `keyboard_shortcuts.png` - Shortcuts overlay

**Total Screenshots Needed: 40**
