# Godot Asset Browser (GAB)

A powerful, native Rust-based asset browser plugin for Godot 4.x that provides enhanced asset management capabilities with advanced features like bulk operations, dependency resolution, collections, and more.

## 🌟 Features

### Core Functionality
- **Asset Library Integration**: Direct integration with Godot Asset Library API
- **Advanced Search & Filtering**: Search by name, tags, category with real-time filtering
- **Asset Management**: Install, update, uninstall assets with conflict detection
- **Download Manager**: Concurrent downloads with pause/resume, retry logic, and bandwidth throttling
- **Dependency Resolution**: Automatic installation of asset dependencies

### User Features
- **Favorites/Bookmarks**: Mark assets as favorites for quick access
- **Ratings System**: Rate assets from 1-5 stars
- **Installation History**: Complete history of installations, updates, and removals
- **Collections/Bundles**: Create and manage custom asset collections
- **Export/Import**: Share collections with other users

### Advanced Asset Management
- **Bulk Operations**: Install, update, or remove multiple assets at once
- **Conflict Detection**: Identify file conflicts between assets before installation
- **Automatic Backups**: Create backups before updates with automatic rollback on failure
- **Update Notifications**: Get notified when updates are available for installed assets

### UI/UX
- **Modern Interface**: Clean, responsive UI with keyboard navigation support
- **Status Badges**: Visual indicators for installation status and available updates
- **Tooltips**: Comprehensive tooltips for all UI elements
- **Loading States**: Progress indicators and spinners for async operations
- **Error Handling**: User-friendly error messages with detailed diagnostics

## 📋 Requirements

- **Godot**: 4.0 or higher
- **Rust**: 1.70 or higher (for building from source)
- **Platform**: Windows, macOS, or Linux

## 🚀 Installation

### From Godot Asset Library (Coming Soon)

1. Open Godot Editor
2. Navigate to AssetLib tab
3. Search for "Godot Asset Browser"
4. Click Install

### From Source

1. **Clone the repository**:
   ```bash
   git clone https://github.com/yourusername/gad-dev.git
   cd gad-dev
   ```

2. **Build the Rust library**:
   ```bash
   cd rust
   cargo build --release
   ```

3. **Copy to your Godot project**:
   ```bash
   # The compiled library will be in rust/target/release/
   # Copy libgodot_asset_browser.so (Linux)
   #   or godot_asset_browser.dll (Windows)
   #   or libgodot_asset_browser.dylib (macOS)
   # to your Godot project's addons/godot_asset_browser/ folder
   ```

4. **Enable the plugin in Godot**:
   - Open your Godot project
   - Go to Project → Project Settings → Plugins
   - Enable "Godot Asset Browser"

## 📖 Usage

### Basic Asset Management

1. **Browse Assets**:
   - Open the GAB panel in Godot Editor
   - Browse available assets from the Godot Asset Library
   - Use search and filters to find specific assets

2. **Install an Asset**:
   - Select an asset from the list
   - View details in the preview panel
   - Click the Install button
   - Wait for download and installation to complete

3. **Update Assets**:
   - Assets with available updates show an "Update Available" badge
   - Click the Update button to install the latest version
   - Automatic backup ensures safe updates

### Advanced Features

#### Bulk Operations

```rust
// Install multiple assets at once
let asset_ids = vec!["asset1".to_string(), "asset2".to_string()];
let results = asset_manager.bulk_install(asset_ids).await;

// Update multiple assets
let results = asset_manager.bulk_update(asset_ids).await;

// Remove multiple assets
let results = asset_manager.bulk_uninstall(asset_ids);
```

#### Collections

1. **Create a Collection**:
   - Click "New Collection"
   - Enter name and description
   - Add assets to the collection

2. **Share a Collection**:
   - Select a collection
   - Click "Export"
   - Share the JSON file with others

3. **Import a Collection**:
   - Click "Import Collection"
   - Select the JSON file
   - Assets in the collection will be available to install

#### Favorites & History

- **Star an Asset**: Click the star icon on any asset
- **View History**: Open the History tab to see all past operations
- **Filter History**: Filter by action type (Install/Update/Uninstall)

## 🏗️ Architecture

### Project Structure

```
gad-dev/
├── rust/                           # Rust source code
│   ├── src/
│   │   ├── asset_library/         # Core asset management
│   │   │   ├── asset.rs           # Asset data structures
│   │   │   ├── asset_manager.rs   # Asset management logic
│   │   │   ├── config_manager.rs  # Configuration management
│   │   │   ├── download_manager.rs # Download queue & management
│   │   │   ├── godot_asset_library_api.rs # API client
│   │   │   └── user_features.rs   # User features (favorites, etc.)
│   │   ├── gui/                   # UI components
│   │   │   ├── asset_library_gui.rs # Main GUI
│   │   │   └── ui_components.rs   # Reusable UI components
│   │   └── lib.rs                 # Library entry point
│   └── Cargo.toml                 # Rust dependencies
└── work_items.md                  # Development roadmap
```

### Key Components

#### AssetManager
Central hub for asset operations:
- Download and install assets
- Check for updates
- Manage installed assets
- Bulk operations
- Dependency resolution

#### DownloadManager
Handles concurrent downloads:
- Queue management
- Pause/resume functionality
- Retry logic with exponential backoff
- Bandwidth throttling
- Progress tracking

#### UserFeatures
Manages user preferences:
- Favorites/bookmarks
- Ratings (1-5 stars)
- Installation history
- Collections/bundles
- Persistent storage

#### GodotAssetLibraryClient
API client for Godot Asset Library:
- Asset listing and search
- Asset details retrieval
- Rate limiting
- Response caching

## 🔧 Configuration

Configuration is stored in `user://godot_asset_browser/config.json`:

```json
{
  "asset_sources": [
    {
      "name": "Godot Asset Library",
      "url": "https://godotengine.org/asset-library/api",
      "enabled": true
    }
  ],
  "preferences": {
    "default_sort": "Name",
    "auto_update_check": true,
    "download_threads": 3,
    "cache_enabled": true
  }
}
```

## 🧪 Development

### Building for Development

```bash
cd rust
cargo build
```

### Running Tests

```bash
cargo test
```

### Code Style

We use `rustfmt` and `clippy` for code formatting and linting:

```bash
cargo fmt
cargo clippy
```

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md) for details.

### Development Workflow

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests (`cargo test`)
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Godot Engine team for the amazing game engine
- gdext team for Rust bindings
- All contributors to this project

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/yourusername/gad-dev/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/gad-dev/discussions)
- **Documentation**: [Wiki](https://github.com/yourusername/gad-dev/wiki)

## 🗺️ Roadmap

See [work_items.md](work_items.md) for detailed development roadmap.

### Completed
- ✅ Phase 1: Core Asset Management
- ✅ Phase 2: Configuration Management
- ✅ Phase 3: GUI Implementation
- ✅ Phase 4: Network & API Integration
- ✅ Phase 5: Advanced Features

### In Progress
- 🚧 Phase 6: Testing & Documentation
- 🚧 Phase 7: Polish & Release

---

**Made with ❤️ for the Godot community**
