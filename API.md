# Godot Asset Browser - API Documentation

This document provides detailed API documentation for the Godot Asset Browser (GAB) plugin.

## Table of Contents

1. [Core Types](#core-types)
2. [AssetManager](#assetmanager)
3. [DownloadManager](#downloadmanager)
4. [UserFeatures](#userfeatures)
5. [GodotAssetLibraryClient](#godotassetlibraryclient)
6. [ConfigManager](#configmanager)

---

## Core Types

### Asset

Represents a Godot asset with complete metadata.

```rust
pub struct Asset {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub category: AssetCategory,
    pub path: String,
    pub tags: Vec<String>,
    pub preview_url: Option<String>,
    pub download_url: String,
    pub dependencies: Vec<AssetDependency>,
    pub version_history: Vec<AssetVersionInfo>,
}
```

### AssetCategory

Enum representing asset categories:

```rust
pub enum AssetCategory {
    TwoD,        // 2D assets (sprites, tiles, etc.)
    ThreeD,      // 3D assets (models, scenes, etc.)
    Shaders,     // Shader materials
    Materials,   // Material resources
    Tools,       // Editor tools and plugins
    Scripts,     // GDScript/C# scripts
    Templates,   // Project templates
    Demos,       // Demo projects
    Audio,       // Audio files and music
    Misc,        // Miscellaneous
}
```

### AssetDependency

Represents a dependency of an asset:

```rust
pub struct AssetDependency {
    pub asset_id: String,
    pub asset_name: String,
    pub version_requirement: String,
    pub optional: bool,
}
```

---

## AssetManager

Central asset management system.

### Initialization

```rust
let asset_manager = AssetManager::new();
```

### Basic Operations

#### Get All Assets

```rust
pub fn get_assets(&self) -> Vec<Asset>
```

Returns all assets in the local catalog.

**Example:**
```rust
let assets = asset_manager.get_assets();
for asset in assets {
    println!("Asset: {} by {}", asset.name, asset.author);
}
```

#### Search Assets

```rust
pub fn search_assets(&self, query: &str) -> Vec<Asset>
```

Search assets by name, tags, or description.

**Parameters:**
- `query`: Search string

**Returns:** Vector of matching assets

**Example:**
```rust
let results = asset_manager.search_assets("platformer");
```

#### Get Asset by ID

```rust
pub fn get_asset_by_id(&self, id: &str) -> Option<Asset>
```

Retrieve a specific asset by its ID.

**Example:**
```rust
if let Some(asset) = asset_manager.get_asset_by_id("asset_123") {
    println!("Found: {}", asset.name);
}
```

### Installation & Updates

#### Download Asset

```rust
pub async fn download_asset(&self, asset_id: String) -> Result<PathBuf, String>
```

Downloads an asset to the cache directory.

**Parameters:**
- `asset_id`: ID of the asset to download

**Returns:**
- `Ok(PathBuf)`: Path to downloaded file
- `Err(String)`: Error message

**Example:**
```rust
match asset_manager.download_asset("asset_123".to_string()).await {
    Ok(path) => println!("Downloaded to: {:?}", path),
    Err(e) => eprintln!("Download failed: {}", e),
}
```

#### Import Asset

```rust
pub async fn import_asset(&self, asset_id: String) -> Result<PathBuf, String>
```

Downloads and installs an asset, extracting it to the asset directory.

**Example:**
```rust
let result = asset_manager.import_asset("asset_123".to_string()).await;
```

#### Update Asset

```rust
pub async fn update_asset(&self, asset_id: String) -> Result<PathBuf, String>
```

Updates an installed asset to the latest version.

#### Check for Updates

```rust
pub fn check_for_update(&self, asset_id: &str) -> Result<Option<Asset>, String>
```

Checks if an update is available for an asset.

**Returns:**
- `Ok(Some(Asset))`: Update available, returns new asset version
- `Ok(None)`: No update available
- `Err(String)`: Error checking for updates

**Example:**
```rust
match asset_manager.check_for_update("asset_123") {
    Ok(Some(updated)) => println!("Update to {} available!", updated.version),
    Ok(None) => println!("Asset is up to date"),
    Err(e) => eprintln!("Error: {}", e),
}
```

### Bulk Operations

#### Bulk Install

```rust
pub async fn bulk_install(&self, asset_ids: Vec<String>)
    -> HashMap<String, Result<PathBuf, String>>
```

Install multiple assets concurrently.

**Returns:** HashMap mapping asset_id to installation result

**Example:**
```rust
let assets = vec!["asset1".to_string(), "asset2".to_string()];
let results = asset_manager.bulk_install(assets).await;

for (id, result) in results {
    match result {
        Ok(path) => println!("{}: Installed to {:?}", id, path),
        Err(e) => println!("{}: Failed - {}", id, e),
    }
}
```

#### Bulk Update

```rust
pub async fn bulk_update(&self, asset_ids: Vec<String>)
    -> HashMap<String, Result<PathBuf, String>>
```

Update multiple assets. Only updates assets with available updates.

#### Bulk Uninstall

```rust
pub fn bulk_uninstall(&self, asset_ids: Vec<String>)
    -> HashMap<String, Result<(), String>>
```

Remove multiple assets.

### Conflict Detection & Resolution

#### Detect Conflicts

```rust
pub fn detect_conflicts(&self, asset_id: &str) -> Result<Vec<String>, String>
```

Check for file conflicts between assets.

**Returns:** List of conflict descriptions

**Example:**
```rust
let conflicts = asset_manager.detect_conflicts("new_asset")?;
if !conflicts.is_empty() {
    println!("Conflicts found:");
    for conflict in conflicts {
        println!("  - {}", conflict);
    }
}
```

### Backup & Recovery

#### Create Backup

```rust
pub fn backup_asset(&self, asset_id: &str) -> Result<PathBuf, String>
```

Create a timestamped backup of an asset.

**Returns:** Path to backup directory

#### Restore from Backup

```rust
pub fn restore_from_backup(&self, backup_path: &Path, asset_id: &str)
    -> Result<(), String>
```

Restore an asset from a backup.

#### Update with Automatic Backup

```rust
pub async fn update_asset_with_backup(&self, asset_id: String)
    -> Result<PathBuf, String>
```

Update an asset with automatic backup and rollback on failure.

### Dependency Management

#### Resolve Dependencies

```rust
pub fn resolve_dependencies(&self, asset_id: &str) -> Result<Vec<String>, String>
```

Check for missing dependencies.

**Returns:** List of missing dependency descriptions

#### Install with Dependencies

```rust
pub async fn install_with_dependencies(&self, asset_id: String)
    -> HashMap<String, Result<PathBuf, String>>
```

Install an asset and all its dependencies.

---

## DownloadManager

Manages concurrent downloads with queue system.

### Configuration

```rust
pub struct DownloadManagerConfig {
    pub max_concurrent: usize,        // Default: 3
    pub max_retries: u32,             // Default: 3
    pub retry_delay_seconds: u64,     // Default: 5
    pub timeout_seconds: u64,         // Default: 300
    pub chunk_size: usize,            // Default: 8192
    pub bandwidth_limit_bps: Option<u64>,  // Default: None (unlimited)
}
```

### Initialization

```rust
// With default config
let manager = DownloadManager::new();

// With custom config
let config = DownloadManagerConfig {
    max_concurrent: 5,
    bandwidth_limit_bps: Some(1024 * 1024), // 1 MB/s
    ..Default::default()
};
let manager = DownloadManager::with_config(config);
```

### Operations

#### Queue Download

```rust
pub fn queue_download(&self, id: String, url: String, destination: PathBuf)
    -> Result<(), String>
```

Add a download to the queue.

#### Start Download

```rust
pub async fn start_download(&self, id: String) -> Result<(), String>
```

Start a queued download.

#### Pause/Resume/Cancel

```rust
pub fn pause_download(&self, id: &str) -> Result<(), String>
pub fn cancel_download(&self, id: &str) -> Result<(), String>
```

#### Get Download Info

```rust
pub fn get_download_info(&self, id: &str) -> Option<DownloadInfo>
pub fn get_all_downloads(&self) -> Vec<DownloadInfo>
pub fn get_active_downloads(&self) -> Vec<DownloadInfo>
```

### DownloadInfo

```rust
pub struct DownloadInfo {
    pub id: String,
    pub url: String,
    pub destination: PathBuf,
    pub status: DownloadStatus,
    pub total_size: Option<u64>,
    pub downloaded_bytes: u64,
    pub speed_bps: f64,
    pub eta_seconds: Option<u64>,
    pub retry_count: u32,
    pub error: Option<String>,
}
```

**Methods:**
- `progress_percent() -> f32`: Returns 0-100 progress percentage

---

## UserFeatures

Manages user preferences and data.

### Initialization

```rust
let data_file = PathBuf::from("user://features.json");
let user_features = UserFeatures::load(data_file)?;
```

### Favorites

```rust
pub fn add_favorite(&mut self, asset_id: String) -> bool
pub fn remove_favorite(&mut self, asset_id: &str) -> bool
pub fn toggle_favorite(&mut self, asset_id: String) -> bool
pub fn is_favorite(&self, asset_id: &str) -> bool
pub fn get_favorites(&self) -> Vec<String>
```

**Example:**
```rust
user_features.add_favorite("asset_123".to_string());
if user_features.is_favorite("asset_123") {
    println!("Asset is favorited!");
}
```

### Ratings

```rust
pub fn set_rating(&mut self, asset_id: String, rating: u8) -> Result<(), String>
pub fn get_rating(&self, asset_id: &str) -> Option<u8>
pub fn remove_rating(&mut self, asset_id: &str) -> bool
```

**Example:**
```rust
user_features.set_rating("asset_123".to_string(), 5)?;  // 5 stars
```

### History

```rust
pub fn add_history(&mut self, asset_id: String, action: HistoryAction, version: String)
pub fn get_history(&self) -> &[HistoryEntry]
pub fn get_asset_history(&self, asset_id: &str) -> Vec<&HistoryEntry>
pub fn get_recent_history(&self, count: usize) -> Vec<&HistoryEntry>
```

**HistoryAction variants:**
- `Install`
- `Update`
- `Uninstall`
- `Download`

### Collections

```rust
pub fn create_collection(&mut self, id: String, name: String, description: String)
    -> Result<(), String>
pub fn add_to_collection(&mut self, collection_id: &str, asset_id: String)
    -> Result<(), String>
pub fn get_collection(&self, collection_id: &str) -> Option<&AssetCollection>
pub fn export_collection(&self, collection_id: &str) -> Result<String, String>
pub fn import_collection(&mut self, id: String, json_data: &str) -> Result<(), String>
```

---

## GodotAssetLibraryClient

API client for Godot Asset Library.

### Initialization

```rust
// Default (Godot Asset Library)
let client = GodotAssetLibraryClient::new();

// Custom URL
let client = GodotAssetLibraryClient::with_url("https://custom.repo.com/api".to_string());

// With authentication
let client = GodotAssetLibraryClient::new()
    .with_auth("your_token".to_string());

// With custom settings
let client = GodotAssetLibraryClient::new()
    .with_cache(true)
    .with_rate_limit(120); // 120 requests per minute
```

### API Methods

```rust
// List assets with filters
pub async fn list_assets(&self, params: AssetListParams)
    -> Result<AssetListResponse, String>

// Get asset details
pub async fn get_asset_detail(&self, asset_id: &str)
    -> Result<GodotAssetDetail, String>

// Search
pub async fn search(&self, query: &str, page: u32)
    -> Result<AssetListResponse, String>

// Get by category
pub async fn get_by_category(&self, category: &str, page: u32)
    -> Result<AssetListResponse, String>

// Health check
pub async fn health_check(&self) -> Result<bool, String>
```

### AssetListParams

Builder for filtering asset queries:

```rust
let params = AssetListParams::new()
    .with_query("platformer".to_string())
    .with_category("2d".to_string())
    .with_page(0)
    .with_max_results(20);
```

---

## ConfigManager

Manages plugin configuration.

### Methods

```rust
pub fn load_config() -> Result<Config, String>
pub fn save_config(config: &Config) -> Result<(), String>
pub fn add_asset_source(&mut self, source: AssetSource) -> Result<(), String>
pub fn remove_asset_source(&mut self, source_id: &str) -> Result<(), String>
```

### Config Structure

```rust
pub struct Config {
    pub asset_sources: Vec<AssetSource>,
    pub preferences: UserPreferences,
}

pub struct AssetSource {
    pub id: String,
    pub name: String,
    pub url: String,
    pub auth_method: Option<AuthMethod>,
    pub enabled: bool,
}

pub struct UserPreferences {
    pub default_sort: String,
    pub auto_update_check: bool,
    pub download_threads: usize,
    pub cache_enabled: bool,
}
```

---

## Error Handling

All async operations return `Result<T, String>` where the error is a descriptive message.

Common error patterns:

```rust
match operation().await {
    Ok(result) => {
        // Handle success
    }
    Err(e) => {
        eprintln!("Operation failed: {}", e);
        // Handle error
    }
}
```

---

## Best Practices

1. **Always handle errors**: Never unwrap Results without checking
2. **Use bulk operations**: For multiple assets, use bulk methods for better performance
3. **Check dependencies**: Use `resolve_dependencies()` before installation
4. **Create backups**: Use `update_asset_with_backup()` for safe updates
5. **Monitor downloads**: Poll `get_download_info()` for progress updates
6. **Save user data**: Call `user_features.save()` after modifications

---

## Examples

### Complete Workflow Example

```rust
use godot_asset_browser::*;

async fn install_asset_workflow() -> Result<(), String> {
    // Initialize
    let asset_manager = AssetManager::new();
    let mut user_features = UserFeatures::load("user://features.json".into())?;

    // Search for assets
    let results = asset_manager.search_assets("platformer");

    if let Some(asset) = results.first() {
        // Check dependencies
        let missing_deps = asset_manager.resolve_dependencies(&asset.id)?;
        if !missing_deps.is_empty() {
            println!("Missing dependencies: {:?}", missing_deps);
        }

        // Check conflicts
        let conflicts = asset_manager.detect_conflicts(&asset.id)?;
        if !conflicts.is_empty() {
            println!("Warning: conflicts detected");
            return Err("Conflicts found".to_string());
        }

        // Install with dependencies
        let results = asset_manager
            .install_with_dependencies(asset.id.clone())
            .await;

        // Add to history
        user_features.add_history(
            asset.id.clone(),
            HistoryAction::Install,
            asset.version.clone()
        );

        // Add to favorites
        user_features.add_favorite(asset.id.clone());

        // Save user data
        user_features.save()?;

        println!("Installation complete!");
    }

    Ok(())
}
```

---

For more examples, see the [examples](examples/) directory in the repository.
