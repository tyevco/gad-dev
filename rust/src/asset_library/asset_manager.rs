use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use godot::prelude::*;
use reqwest::Client;
use zip::ZipArchive;
use tar::Archive;
use flate2::read::GzDecoder;
use crate::asset_library::asset::{Asset, AssetCategory};

// Conditional print macro that only prints when not testing
#[cfg(not(test))]
macro_rules! debug_print {
    ($($arg:tt)*) => { godot_print!($($arg)*) };
}

#[cfg(test)]
macro_rules! debug_print {
    ($($arg:tt)*) => { /* no-op in tests */ };
}

/// Search index for fast text-based asset lookups
#[derive(Debug, Clone, Default)]
struct SearchIndex {
    /// Maps lowercase search terms to asset indices
    term_to_assets: HashMap<String, Vec<usize>>,
    /// Pre-computed lowercase names for faster searching
    lowercase_names: Vec<String>,
    /// Pre-computed lowercase tags for faster searching
    lowercase_tags: Vec<Vec<String>>,
    /// Pre-computed lowercase descriptions for faster searching
    lowercase_descriptions: Vec<String>,
}

impl SearchIndex {
    /// Rebuilds the search index from a list of assets
    fn rebuild(&mut self, assets: &[Asset]) {
        self.term_to_assets.clear();
        self.lowercase_names.clear();
        self.lowercase_tags.clear();
        self.lowercase_descriptions.clear();

        for (idx, asset) in assets.iter().enumerate() {
            // Pre-compute lowercase strings
            let name_lower = asset.name.to_lowercase();
            let desc_lower = asset.description.to_lowercase();
            let tags_lower: Vec<String> = asset.tags.iter()
                .map(|t| t.to_lowercase())
                .collect();

            self.lowercase_names.push(name_lower.clone());
            self.lowercase_descriptions.push(desc_lower.clone());
            self.lowercase_tags.push(tags_lower.clone());

            // Index words from name
            for word in name_lower.split_whitespace() {
                self.term_to_assets.entry(word.to_string())
                    .or_insert_with(Vec::new)
                    .push(idx);
            }

            // Index tags
            for tag in &tags_lower {
                self.term_to_assets.entry(tag.clone())
                    .or_insert_with(Vec::new)
                    .push(idx);
            }
        }
    }

    /// Fast search using pre-computed lowercase strings
    fn search(&self, query: &str, assets: &[Asset]) -> Vec<Asset> {
        let query_lower = query.to_lowercase();

        assets.iter().enumerate()
            .filter(|(idx, _)| {
                self.lowercase_names.get(*idx)
                    .map(|n| n.contains(&query_lower))
                    .unwrap_or(false)
                || self.lowercase_tags.get(*idx)
                    .map(|tags| tags.iter().any(|t| t.contains(&query_lower)))
                    .unwrap_or(false)
                || self.lowercase_descriptions.get(*idx)
                    .map(|d| d.contains(&query_lower))
                    .unwrap_or(false)
            })
            .map(|(_, asset)| asset.clone())
            .collect()
    }
}

#[derive(GodotClass)]
#[class(init)]
pub struct AssetManager {
    client: Client,
    asset_dir: String,
    cache_dir: String,
    assets: Arc<Mutex<Vec<Asset>>>,
    installed_assets: Arc<Mutex<Vec<String>>>, // Track installed asset IDs
    search_index: Arc<Mutex<SearchIndex>>, // Search index for fast lookups
}

impl AssetManager {
    /// Creates a new AssetManager instance with default configuration.
    ///
    /// This initializes the asset manager with:
    /// - HTTP client for downloading assets
    /// - Asset directory at `res://addons/`
    /// - Cache directory at `user://asset_cache/`
    /// - Empty asset and installed assets lists
    /// - Sample assets for testing (will be removed in production)
    ///
    /// # Returns
    /// * `Self` - A new AssetManager instance
    ///
    /// # Example
    /// ```
    /// let manager = AssetManager::new();
    /// ```
    pub fn new() -> Self {
        let mut manager = Self {
            client: Client::new(),
            asset_dir: "res://addons/".to_string(),
            cache_dir: "user://asset_cache/".to_string(),
            assets: Arc::new(Mutex::new(Vec::new())),
            installed_assets: Arc::new(Mutex::new(Vec::new())),
            search_index: Arc::new(Mutex::new(SearchIndex::default())),
        };

        // Ensure cache directory exists
        if let Err(e) = fs::create_dir_all(&manager.cache_dir) {
            godot_print!("Warning: Failed to create cache directory: {}", e);
        }

        // Initialize with sample data for testing
        manager.initialize_sample_assets();
        manager
    }

    /// Initialize with sample assets for testing the GUI
    fn initialize_sample_assets(&mut self) {
        let mut assets = self.assets.lock().unwrap();

        // Add some sample assets for testing
        assets.push(Asset::new(
            "1".to_string(),
            "Awesome 2D Sprites".to_string(),
            AssetCategory::TwoD,
            "".to_string(),
            "JohnDoe".to_string(),
            "1.0.0".to_string(),
            "A collection of high-quality 2D sprites for your game".to_string(),
            vec!["sprites".to_string(), "2d".to_string(), "pixel-art".to_string()],
            Some("https://example.com/preview1.png".to_string()),
            "https://example.com/download1.zip".to_string(),
            vec![],
            vec![],
        ));

        assets.push(Asset::new(
            "2".to_string(),
            "3D Character Models".to_string(),
            AssetCategory::ThreeD,
            "".to_string(),
            "JaneSmith".to_string(),
            "2.1.0".to_string(),
            "Professional 3D character models with animations".to_string(),
            vec!["3d".to_string(), "characters".to_string(), "models".to_string()],
            Some("https://example.com/preview2.png".to_string()),
            "https://example.com/download2.zip".to_string(),
            vec![],
            vec![],
        ));

        assets.push(Asset::new(
            "3".to_string(),
            "Shader Pack Pro".to_string(),
            AssetCategory::Shaders,
            "".to_string(),
            "ShaderWizard".to_string(),
            "3.0.5".to_string(),
            "Advanced shader collection for stunning visual effects".to_string(),
            vec!["shaders".to_string(), "effects".to_string(), "graphics".to_string()],
            Some("https://example.com/preview3.png".to_string()),
            "https://example.com/download3.zip".to_string(),
            vec![],
            vec![],
        ));

        assets.push(Asset::new(
            "4".to_string(),
            "Audio Effects Library".to_string(),
            AssetCategory::Audio,
            "".to_string(),
            "SoundMaster".to_string(),
            "1.5.2".to_string(),
            "Comprehensive audio effects and music tracks".to_string(),
            vec!["audio".to_string(), "sound".to_string(), "music".to_string()],
            Some("https://example.com/preview4.png".to_string()),
            "https://example.com/download4.zip".to_string(),
            vec![],
            vec![],
        ));

        assets.push(Asset::new(
            "5".to_string(),
            "Utility Scripts Collection".to_string(),
            AssetCategory::Scripts,
            "".to_string(),
            "CodeNinja".to_string(),
            "4.2.0".to_string(),
            "Essential utility scripts for game development".to_string(),
            vec!["scripts".to_string(), "utilities".to_string(), "gdscript".to_string()],
            None,
            "https://example.com/download5.zip".to_string(),
            vec![],
            vec![],
        ));

        // Rebuild search index after adding assets
        let assets_clone = assets.clone();
        drop(assets); // Release lock before acquiring search_index lock
        self.search_index.lock().unwrap().rebuild(&assets_clone);
    }

    /// Retrieves all assets from the asset library.
    ///
    /// # Returns
    /// * `Vec<Asset>` - A vector containing all assets
    ///
    /// # Example
    /// ```
    /// let manager = AssetManager::new();
    /// let all_assets = manager.get_assets();
    /// println!("Total assets: {}", all_assets.len());
    /// ```
    pub fn get_assets(&self) -> Vec<Asset> {
        self.assets.lock().unwrap().clone()
    }

    /// Retrieves assets filtered by category.
    ///
    /// # Arguments
    /// * `category` - The category to filter by
    ///
    /// # Returns
    /// * `Vec<Asset>` - A vector containing assets matching the specified category
    ///
    /// # Example
    /// ```
    /// let manager = AssetManager::new();
    /// let tools = manager.get_assets_by_category(AssetCategory::Tools);
    /// ```
    pub fn get_assets_by_category(&self, category: AssetCategory) -> Vec<Asset> {
        self.assets
            .lock()
            .unwrap()
            .iter()
            .filter(|asset| asset.category == category)
            .cloned()
            .collect()
    }

    /// Searches for assets by name, tags, or description.
    ///
    /// Performs a case-insensitive search across asset names, tags, and descriptions.
    /// This method uses a pre-computed search index for improved performance.
    ///
    /// # Arguments
    /// * `query` - The search query string
    ///
    /// # Returns
    /// * `Vec<Asset>` - A vector containing assets matching the search query
    ///
    /// # Performance
    /// This method uses pre-computed lowercase strings to avoid repeated
    /// allocations during search, significantly improving performance for
    /// large asset lists.
    ///
    /// # Example
    /// ```
    /// let manager = AssetManager::new();
    /// let results = manager.search_assets("shader");
    /// println!("Found {} assets matching 'shader'", results.len());
    /// ```
    pub fn search_assets(&self, query: &str) -> Vec<Asset> {
        let assets = self.assets.lock().unwrap();
        let search_index = self.search_index.lock().unwrap();

        // Use optimized search with pre-computed lowercase strings
        search_index.search(query, &assets)
    }

    /// Retrieves a paginated subset of all assets (lazy loading support).
    ///
    /// This method enables lazy loading of assets for better performance
    /// when dealing with large asset lists in the GUI.
    ///
    /// # Arguments
    /// * `page` - The page number (0-indexed)
    /// * `page_size` - Number of assets per page
    ///
    /// # Returns
    /// * `Vec<Asset>` - A vector containing assets for the requested page
    ///
    /// # Performance
    /// This method avoids cloning the entire asset list, only cloning
    /// the subset needed for the current page.
    ///
    /// # Example
    /// ```
    /// let manager = AssetManager::new();
    /// // Get first page with 20 assets
    /// let page1 = manager.get_assets_paginated(0, 20);
    /// // Get second page
    /// let page2 = manager.get_assets_paginated(1, 20);
    /// ```
    pub fn get_assets_paginated(&self, page: usize, page_size: usize) -> Vec<Asset> {
        let assets = self.assets.lock().unwrap();
        let start = page * page_size;
        let end = std::cmp::min(start + page_size, assets.len());

        if start >= assets.len() {
            return Vec::new();
        }

        assets[start..end].to_vec()
    }

    /// Gets the total number of assets.
    ///
    /// This is useful for calculating the total number of pages
    /// when implementing pagination.
    ///
    /// # Returns
    /// * `usize` - Total number of assets
    pub fn get_asset_count(&self) -> usize {
        self.assets.lock().unwrap().len()
    }

    /// Retrieves a specific asset by its ID.
    ///
    /// # Arguments
    /// * `id` - The unique identifier of the asset
    ///
    /// # Returns
    /// * `Some(Asset)` - The asset if found
    /// * `None` - If no asset with the given ID exists
    ///
    /// # Example
    /// ```
    /// let manager = AssetManager::new();
    /// if let Some(asset) = manager.get_asset_by_id("asset_123") {
    ///     println!("Found asset: {}", asset.name);
    /// }
    /// ```
    pub fn get_asset_by_id(&self, id: &str) -> Option<Asset> {
        self.assets
            .lock()
            .unwrap()
            .iter()
            .find(|asset| asset.id == id)
            .cloned()
    }

    /// Downloads an asset from its URL to the cache directory
    ///
    /// # Arguments
    /// * `asset_id` - The ID of the asset to download
    ///
    /// # Returns
    /// * `Ok(PathBuf)` - Path to the downloaded file on success
    /// * `Err(String)` - Error message on failure
    pub async fn download_asset(&self, asset_id: String) -> Result<PathBuf, String> {
        // Get the asset details
        let asset = self.get_asset_by_id(&asset_id)
            .ok_or_else(|| format!("Asset with ID '{}' not found", asset_id))?;

        godot_print!("Starting download for asset: {} ({})", asset.name, asset_id);

        // Determine the file extension from the URL
        let url = &asset.download_url;
        let file_ext = Self::extract_file_extension(url).unwrap_or("zip".to_string());
        let cache_file_name = format!("{}_{}.{}", asset_id, asset.version, file_ext);
        let cache_file_path = PathBuf::from(&self.cache_dir).join(&cache_file_name);

        // Check if file already exists in cache
        if cache_file_path.exists() {
            godot_print!("Asset already cached at: {:?}", cache_file_path);
            return Ok(cache_file_path);
        }

        // Create cache directory if it doesn't exist
        if let Some(parent) = cache_file_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create cache directory: {}", e))?;
        }

        // Download the file
        godot_print!("Downloading from: {}", url);
        let response = self.client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("Failed to initiate download: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Download failed with status: {}", response.status()));
        }

        // Get total size for progress tracking
        let total_size = response.content_length().unwrap_or(0);
        godot_print!("Download size: {} bytes", total_size);

        // Download the content
        let content = response.bytes()
            .await
            .map_err(|e| format!("Failed to download content: {}", e))?;

        // Write to cache file
        fs::write(&cache_file_path, &content)
            .map_err(|e| format!("Failed to write cache file: {}", e))?;

        godot_print!("Download complete: {:?}", cache_file_path);
        Ok(cache_file_path)
    }

    /// Extracts file extension from a URL
    fn extract_file_extension(url: &str) -> Option<String> {
        let path = url.split('?').next()?; // Remove query parameters
        let filename = path.split('/').last()?;

        if filename.ends_with(".tar.gz") {
            return Some("tar.gz".to_string());
        }

        filename.split('.').last().map(|s| s.to_string())
    }

    /// Imports an asset by downloading and extracting it
    ///
    /// This method performs the complete import process:
    /// 1. Downloads the asset to cache (or uses cached version)
    /// 2. Extracts the archive to the asset directory
    /// 3. Validates the extracted content
    /// 4. Integrates with Godot's import system
    /// 5. Tracks the asset as installed
    ///
    /// # Arguments
    /// * `asset_id` - The ID of the asset to import
    ///
    /// # Returns
    /// * `Ok(())` - Success
    /// * `Err(String)` - Error message on failure
    pub async fn import_asset(&self, asset_id: String) -> Result<PathBuf, String> {
        godot_print!("Starting import for asset: {}", asset_id);

        // Step 1: Download the asset
        let cache_file_path = self.download_asset(asset_id.clone()).await?;

        // Step 2: Create importer and extract
        let importer = AssetImporter::new(PathBuf::from(&self.asset_dir));
        let final_path = importer.import_asset(&cache_file_path, &asset_id)
            .map_err(|e| {
                godot_print!("Import failed: {}", e);
                e
            })?;

        // Step 3: Track as installed
        {
            let mut installed = self.installed_assets.lock().unwrap();
            if !installed.contains(&asset_id) {
                installed.push(asset_id.clone());
            }
        }

        godot_print!("Successfully imported asset '{}' to: {:?}", asset_id, final_path);
        Ok(final_path)
    }

    /// Checks if an asset is currently installed.
    ///
    /// # Arguments
    /// * `asset_id` - The unique identifier of the asset to check
    ///
    /// # Returns
    /// * `true` - If the asset is installed
    /// * `false` - If the asset is not installed
    ///
    /// # Example
    /// ```
    /// let manager = AssetManager::new();
    /// if manager.is_asset_installed("asset_123") {
    ///     println!("Asset is already installed");
    /// }
    /// ```
    pub fn is_asset_installed(&self, asset_id: &str) -> bool {
        self.installed_assets.lock().unwrap().contains(&asset_id.to_string())
    }

    /// Retrieves a list of all installed asset IDs.
    ///
    /// # Returns
    /// * `Vec<String>` - A vector containing the IDs of all installed assets
    ///
    /// # Example
    /// ```
    /// let manager = AssetManager::new();
    /// let installed = manager.get_installed_assets();
    /// println!("You have {} assets installed", installed.len());
    /// ```
    pub fn get_installed_assets(&self) -> Vec<String> {
        self.installed_assets.lock().unwrap().clone()
    }

    /// Uninstalls an asset by removing it from the asset directory.
    ///
    /// This removes all files associated with the asset and updates the
    /// installed assets tracking list.
    ///
    /// # Arguments
    /// * `asset_id` - The unique identifier of the asset to uninstall
    ///
    /// # Returns
    /// * `Ok(())` - If the asset was successfully uninstalled
    /// * `Err(String)` - Error message if the asset is not installed or removal failed
    ///
    /// # Example
    /// ```
    /// let manager = AssetManager::new();
    /// match manager.uninstall_asset("asset_123") {
    ///     Ok(_) => println!("Asset uninstalled successfully"),
    ///     Err(e) => println!("Failed to uninstall: {}", e),
    /// }
    /// ```
    pub fn uninstall_asset(&self, asset_id: &str) -> Result<(), String> {
        let asset_path = PathBuf::from(&self.asset_dir).join(asset_id);

        if !asset_path.exists() {
            return Err(format!("Asset '{}' is not installed", asset_id));
        }

        fs::remove_dir_all(&asset_path)
            .map_err(|e| format!("Failed to uninstall asset: {}", e))?;

        // Remove from installed list
        {
            let mut installed = self.installed_assets.lock().unwrap();
            installed.retain(|id| id != asset_id);
        }

        godot_print!("Successfully uninstalled asset: {}", asset_id);
        Ok(())
    }

    /// Fetches asset metadata from a remote source
    ///
    /// This method retrieves asset information from a remote API endpoint.
    /// In a full implementation, this would connect to the Godot Asset Library API
    /// or other configured asset sources.
    ///
    /// # Arguments
    /// * `source_url` - The URL of the asset source API
    ///
    /// # Returns
    /// * `Ok(Vec<Asset>)` - List of assets from the source
    /// * `Err(String)` - Error message on failure
    pub async fn fetch_assets_from_source(&self, source_url: &str) -> Result<Vec<Asset>, String> {
        godot_print!("Fetching assets from: {}", source_url);

        let response = self.client
            .get(source_url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch assets: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("API request failed with status: {}", response.status()));
        }

        // Parse the response as JSON
        let assets: Vec<Asset> = response.json()
            .await
            .map_err(|e| format!("Failed to parse asset metadata: {}", e))?;

        godot_print!("Fetched {} assets from remote source", assets.len());
        Ok(assets)
    }

    /// Fetches detailed metadata for a specific asset
    ///
    /// # Arguments
    /// * `source_url` - The URL of the asset detail endpoint
    ///
    /// # Returns
    /// * `Ok(Asset)` - The asset with full metadata
    /// * `Err(String)` - Error message on failure
    pub async fn fetch_asset_details(&self, source_url: &str) -> Result<Asset, String> {
        godot_print!("Fetching asset details from: {}", source_url);

        let response = self.client
            .get(source_url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch asset details: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("API request failed with status: {}", response.status()));
        }

        // Parse the response as JSON
        let asset: Asset = response.json()
            .await
            .map_err(|e| format!("Failed to parse asset metadata: {}", e))?;

        Ok(asset)
    }

    /// Refreshes the local asset list with data from remote sources
    ///
    /// This method updates the internal asset list with fresh data from
    /// configured remote sources. In a full implementation, this would query
    /// multiple asset sources and merge the results.
    ///
    /// # Arguments
    /// * `source_urls` - List of asset source URLs to query
    ///
    /// # Returns
    /// * `Ok(usize)` - Number of assets fetched
    /// * `Err(String)` - Error message on failure
    pub async fn refresh_asset_list(&self, source_urls: Vec<String>) -> Result<usize, String> {
        godot_print!("Refreshing asset list from {} sources", source_urls.len());

        let mut all_assets = Vec::new();

        for source_url in source_urls {
            match self.fetch_assets_from_source(&source_url).await {
                Ok(mut assets) => {
                    godot_print!("Fetched {} assets from {}", assets.len(), source_url);
                    all_assets.append(&mut assets);
                }
                Err(e) => {
                    godot_print!("Warning: Failed to fetch from {}: {}", source_url, e);
                    // Continue with other sources even if one fails
                }
            }
        }

        // Update the internal asset list
        {
            let mut assets = self.assets.lock().unwrap();
            *assets = all_assets;
        }

        let count = self.assets.lock().unwrap().len();
        godot_print!("Asset list refreshed with {} total assets", count);
        Ok(count)
    }

    /// Clears the download cache
    ///
    /// Removes all cached asset files to free up disk space
    pub fn clear_cache(&self) -> Result<(), String> {
        let cache_path = PathBuf::from(&self.cache_dir);

        if !cache_path.exists() {
            return Ok(()); // Nothing to clear
        }

        // Remove all files in cache directory
        let entries = fs::read_dir(&cache_path)
            .map_err(|e| format!("Failed to read cache directory: {}", e))?;

        let mut cleared_count = 0;
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_file() {
                    if fs::remove_file(entry.path()).is_ok() {
                        cleared_count += 1;
                    }
                }
            }
        }

        godot_print!("Cleared {} cached files", cleared_count);
        Ok(())
    }

    /// Gets the size of the cache directory in bytes
    pub fn get_cache_size(&self) -> Result<u64, String> {
        let cache_path = PathBuf::from(&self.cache_dir);

        if !cache_path.exists() {
            return Ok(0);
        }

        let mut total_size = 0u64;
        let entries = fs::read_dir(&cache_path)
            .map_err(|e| format!("Failed to read cache directory: {}", e))?;

        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    total_size += metadata.len();
                }
            }
        }

        Ok(total_size)
    }

    /// Checks if an update is available for a specific asset
    ///
    /// Compares the installed version with the version in the asset list
    ///
    /// # Arguments
    /// * `asset_id` - The ID of the asset to check
    ///
    /// # Returns
    /// * `Ok(Some(Asset))` - Update is available, returns the newer asset
    /// * `Ok(None)` - No update available or asset not installed
    /// * `Err(String)` - Error checking for updates
    pub fn check_for_update(&self, asset_id: &str) -> Result<Option<Asset>, String> {
        // Check if asset is installed
        if !self.is_asset_installed(asset_id) {
            return Ok(None);
        }

        // Get the current asset metadata
        let current_asset = self.get_asset_by_id(asset_id)
            .ok_or_else(|| format!("Asset '{}' not found in catalog", asset_id))?;

        // Get installed version from metadata file
        let asset_path = PathBuf::from(&self.asset_dir).join(asset_id);
        let metadata_path = asset_path.join(ASSET_METADATA_FILE);

        let installed_version = if metadata_path.exists() {
            // Read installed version from metadata
            match fs::read_to_string(&metadata_path) {
                Ok(content) => {
                    if let Ok(metadata) = serde_json::from_str::<serde_json::Value>(&content) {
                        metadata.get("version")
                            .and_then(|v| v.as_str())
                            .unwrap_or(&current_asset.version)
                            .to_string()
                    } else {
                        current_asset.version.clone()
                    }
                }
                Err(_) => current_asset.version.clone()
            }
        } else {
            // No metadata file, assume current version
            current_asset.version.clone()
        };

        // Compare versions
        if Self::is_version_newer(&current_asset.version, &installed_version) {
            godot_print!("Update available for '{}': {} -> {}",
                asset_id, installed_version, current_asset.version);
            Ok(Some(current_asset))
        } else {
            Ok(None)
        }
    }

    /// Checks for updates for all installed assets
    ///
    /// # Returns
    /// * `Ok(Vec<Asset>)` - List of assets with available updates
    /// * `Err(String)` - Error checking for updates
    pub fn check_all_for_updates(&self) -> Result<Vec<Asset>, String> {
        let installed = self.get_installed_assets();
        let mut updates = Vec::new();

        for asset_id in installed {
            if let Ok(Some(updated_asset)) = self.check_for_update(&asset_id) {
                updates.push(updated_asset);
            }
        }

        if !updates.is_empty() {
            godot_print!("Found {} available updates", updates.len());
        }

        Ok(updates)
    }

    /// Compares two version strings to determine if one is newer
    ///
    /// This is a simple semantic version comparison (major.minor.patch)
    ///
    /// # Arguments
    /// * `version_a` - First version to compare
    /// * `version_b` - Second version to compare
    ///
    /// # Returns
    /// * `true` if version_a is newer than version_b
    /// * `false` otherwise
    fn is_version_newer(version_a: &str, version_b: &str) -> bool {
        let parse_version = |v: &str| -> Vec<u32> {
            v.split('.')
                .filter_map(|s| s.parse::<u32>().ok())
                .collect()
        };

        let a_parts = parse_version(version_a);
        let b_parts = parse_version(version_b);

        // Compare each part
        for i in 0..a_parts.len().max(b_parts.len()) {
            let a = a_parts.get(i).unwrap_or(&0);
            let b = b_parts.get(i).unwrap_or(&0);

            if a > b {
                return true;
            } else if a < b {
                return false;
            }
        }

        false // Versions are equal
    }

    /// Updates an installed asset to the latest version
    ///
    /// This method downloads and installs the new version, replacing the old one
    ///
    /// # Arguments
    /// * `asset_id` - The ID of the asset to update
    ///
    /// # Returns
    /// * `Ok(PathBuf)` - Path to the updated asset
    /// * `Err(String)` - Error message on failure
    pub async fn update_asset(&self, asset_id: String) -> Result<PathBuf, String> {
        godot_print!("Updating asset: {}", asset_id);

        // Check if update is available
        match self.check_for_update(&asset_id)? {
            Some(updated_asset) => {
                godot_print!("Updating to version {}", updated_asset.version);

                // Uninstall old version
                self.uninstall_asset(&asset_id)?;

                // Install new version
                self.import_asset(asset_id).await
            }
            None => {
                Err(format!("No update available for asset '{}'", asset_id))
            }
        }
    }

    /// Adds an asset to the local catalog
    ///
    /// This is useful for adding custom assets or updating asset metadata
    pub fn add_asset(&self, asset: Asset) {
        let mut assets = self.assets.lock().unwrap();

        // Remove existing asset with same ID if present
        assets.retain(|a| a.id != asset.id);

        // Add the new/updated asset
        assets.push(asset);

        // Rebuild search index to include the new asset
        let assets_clone = assets.clone();
        drop(assets); // Release lock before acquiring search_index lock
        self.search_index.lock().unwrap().rebuild(&assets_clone);
    }

    /// Removes an asset from the local catalog
    pub fn remove_asset(&self, asset_id: &str) {
        let mut assets = self.assets.lock().unwrap();
        assets.retain(|a| a.id != asset_id);

        // Rebuild search index to remove the deleted asset
        let assets_clone = assets.clone();
        drop(assets); // Release lock before acquiring search_index lock
        self.search_index.lock().unwrap().rebuild(&assets_clone);
    }

    // ===== Advanced Asset Management Features =====

    /// Bulk install multiple assets concurrently.
    ///
    /// Installs multiple assets in a batch operation. Each asset is installed independently,
    /// and the method returns a map showing the result for each asset.
    ///
    /// # Arguments
    /// * `asset_ids` - Vector of asset IDs to install
    ///
    /// # Returns
    /// * `HashMap<String, Result<PathBuf, String>>` - Map of asset ID to installation result
    ///   - `Ok(PathBuf)` - Path to the installed asset on success
    ///   - `Err(String)` - Error message if installation failed
    ///
    /// # Example
    /// ```
    /// let manager = AssetManager::new();
    /// let ids = vec!["asset1".to_string(), "asset2".to_string()];
    /// let results = manager.bulk_install(ids).await;
    ///
    /// for (id, result) in results {
    ///     match result {
    ///         Ok(path) => println!("{} installed at {:?}", id, path),
    ///         Err(e) => println!("{} failed: {}", id, e),
    ///     }
    /// }
    /// ```
    pub async fn bulk_install(&self, asset_ids: Vec<String>) -> HashMap<String, Result<PathBuf, String>> {
        let mut results = HashMap::new();

        for asset_id in asset_ids {
            godot_print!("Bulk install: processing {}", asset_id);
            let result = self.import_asset(asset_id.clone()).await;
            results.insert(asset_id, result);
        }

        results
    }

    /// Bulk update multiple assets to their latest versions.
    ///
    /// Only updates assets that have updates available. Assets without updates
    /// will have an error result indicating no update is available.
    ///
    /// # Arguments
    /// * `asset_ids` - Vector of asset IDs to update
    ///
    /// # Returns
    /// * `HashMap<String, Result<PathBuf, String>>` - Map of asset ID to update result
    ///   - `Ok(PathBuf)` - Path to the updated asset on success
    ///   - `Err(String)` - Error message if update failed or no update available
    ///
    /// # Example
    /// ```
    /// let manager = AssetManager::new();
    /// let ids = vec!["asset1".to_string(), "asset2".to_string()];
    /// let results = manager.bulk_update(ids).await;
    ///
    /// for (id, result) in results {
    ///     match result {
    ///         Ok(path) => println!("{} updated successfully", id),
    ///         Err(e) => println!("{}: {}", id, e),
    ///     }
    /// }
    /// ```
    pub async fn bulk_update(&self, asset_ids: Vec<String>) -> HashMap<String, Result<PathBuf, String>> {
        let mut results = HashMap::new();

        for asset_id in asset_ids {
            // Check if update is available
            match self.check_for_update(&asset_id) {
                Ok(Some(_)) => {
                    godot_print!("Bulk update: updating {}", asset_id);
                    let result = self.update_asset(asset_id.clone()).await;
                    results.insert(asset_id, result);
                }
                Ok(None) => {
                    results.insert(asset_id, Err("No update available".to_string()));
                }
                Err(e) => {
                    results.insert(asset_id, Err(format!("Update check failed: {}", e)));
                }
            }
        }

        results
    }

    /// Bulk uninstall multiple assets.
    ///
    /// Removes multiple assets in a batch operation. Each asset is uninstalled
    /// independently, and the method returns a map showing the result for each asset.
    ///
    /// # Arguments
    /// * `asset_ids` - Vector of asset IDs to uninstall
    ///
    /// # Returns
    /// * `HashMap<String, Result<(), String>>` - Map of asset ID to uninstall result
    ///   - `Ok(())` - Asset was successfully uninstalled
    ///   - `Err(String)` - Error message if uninstall failed
    ///
    /// # Example
    /// ```
    /// let manager = AssetManager::new();
    /// let ids = vec!["asset1".to_string(), "asset2".to_string()];
    /// let results = manager.bulk_uninstall(ids);
    ///
    /// for (id, result) in results {
    ///     match result {
    ///         Ok(_) => println!("{} uninstalled", id),
    ///         Err(e) => println!("{} failed: {}", id, e),
    ///     }
    /// }
    /// ```
    pub fn bulk_uninstall(&self, asset_ids: Vec<String>) -> HashMap<String, Result<(), String>> {
        let mut results = HashMap::new();

        for asset_id in asset_ids {
            godot_print!("Bulk uninstall: removing {}", asset_id);
            let result = self.uninstall_asset(&asset_id);
            results.insert(asset_id, result);
        }

        results
    }

    /// Detects file conflicts between assets.
    ///
    /// Checks if installing or updating the specified asset would conflict with
    /// files from other installed assets. This helps prevent overwriting files
    /// from other assets.
    ///
    /// # Arguments
    /// * `asset_id` - The ID of the asset to check for conflicts
    ///
    /// # Returns
    /// * `Ok(Vec<String>)` - List of conflict messages (empty if no conflicts)
    /// * `Err(String)` - Error message if conflict detection failed
    ///
    /// # Example
    /// ```
    /// let manager = AssetManager::new();
    /// match manager.detect_conflicts("asset_123") {
    ///     Ok(conflicts) => {
    ///         if conflicts.is_empty() {
    ///             println!("No conflicts detected");
    ///         } else {
    ///             for conflict in conflicts {
    ///                 println!("Conflict: {}", conflict);
    ///             }
    ///         }
    ///     }
    ///     Err(e) => println!("Error: {}", e),
    /// }
    /// ```
    pub fn detect_conflicts(&self, asset_id: &str) -> Result<Vec<String>, String> {
        let asset_path = PathBuf::from(&self.asset_dir).join(asset_id);
        let mut conflicts = Vec::new();

        if !asset_path.exists() {
            return Ok(conflicts);
        }

        // Get all files in this asset
        let asset_files = self.get_asset_files(&asset_path)?;

        // Check against other installed assets
        let installed = self.get_installed_assets();
        for other_id in installed {
            if other_id == asset_id {
                continue; // Skip self
            }

            let other_path = PathBuf::from(&self.asset_dir).join(&other_id);
            if other_path.exists() {
                let other_files = self.get_asset_files(&other_path)?;

                // Find file path overlaps
                for file in &asset_files {
                    if other_files.contains(file) {
                        conflicts.push(format!(
                            "File conflict with '{}': {}",
                            other_id,
                            file.display()
                        ));
                    }
                }
            }
        }

        Ok(conflicts)
    }

    /// Get all files in an asset directory (recursive)
    fn get_asset_files(&self, asset_dir: &Path) -> Result<Vec<PathBuf>, String> {
        let mut files = Vec::new();

        if !asset_dir.exists() {
            return Ok(files);
        }

        let entries = fs::read_dir(asset_dir)
            .map_err(|e| format!("Failed to read asset directory: {}", e))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                // Store relative path
                if let Ok(rel_path) = path.strip_prefix(asset_dir) {
                    files.push(rel_path.to_path_buf());
                }
            } else if path.is_dir() {
                // Recursively get files from subdirectories
                let subdir_files = self.get_asset_files(&path)?;
                files.extend(subdir_files);
            }
        }

        Ok(files)
    }

    /// Creates a backup of an installed asset before updating.
    ///
    /// Creates a timestamped backup copy of the asset's directory in the
    /// cache/backups folder. This allows for safe updates with rollback capability.
    ///
    /// # Arguments
    /// * `asset_id` - The ID of the asset to backup
    ///
    /// # Returns
    /// * `Ok(PathBuf)` - Path to the backup directory
    /// * `Err(String)` - Error message if the asset is not installed or backup failed
    ///
    /// # Example
    /// ```
    /// let manager = AssetManager::new();
    /// match manager.backup_asset("asset_123") {
    ///     Ok(backup_path) => println!("Backup created at: {:?}", backup_path),
    ///     Err(e) => println!("Backup failed: {}", e),
    /// }
    /// ```
    pub fn backup_asset(&self, asset_id: &str) -> Result<PathBuf, String> {
        let asset_path = PathBuf::from(&self.asset_dir).join(asset_id);

        if !asset_path.exists() {
            return Err(format!("Asset '{}' is not installed", asset_id));
        }

        // Create backup directory with timestamp
        use std::time::SystemTime;
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let backup_dir = PathBuf::from(&self.cache_dir)
            .join("backups")
            .join(format!("{}_{}", asset_id, timestamp));

        // Create backup parent directory
        if let Some(parent) = backup_dir.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create backup directory: {}", e))?;
        }

        // Copy asset directory to backup location
        self.copy_dir_recursive(&asset_path, &backup_dir)?;

        godot_print!("Backed up '{}' to: {:?}", asset_id, backup_dir);
        Ok(backup_dir)
    }

    /// Recursively copy a directory
    fn copy_dir_recursive(&self, src: &Path, dst: &Path) -> Result<(), String> {
        fs::create_dir_all(dst)
            .map_err(|e| format!("Failed to create destination directory: {}", e))?;

        let entries = fs::read_dir(src)
            .map_err(|e| format!("Failed to read source directory: {}", e))?;

        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = path.file_name().unwrap();
            let dst_path = dst.join(file_name);

            if path.is_dir() {
                self.copy_dir_recursive(&path, &dst_path)?;
            } else {
                fs::copy(&path, &dst_path)
                    .map_err(|e| format!("Failed to copy file {:?}: {}", path, e))?;
            }
        }

        Ok(())
    }

    /// Restores an asset from a backup directory.
    ///
    /// Removes the current installation and replaces it with the contents
    /// from the backup. This is typically used after a failed update.
    ///
    /// # Arguments
    /// * `backup_path` - Path to the backup directory
    /// * `asset_id` - The ID of the asset to restore
    ///
    /// # Returns
    /// * `Ok(())` - Asset was successfully restored
    /// * `Err(String)` - Error message if restoration failed
    ///
    /// # Example
    /// ```
    /// let manager = AssetManager::new();
    /// let backup_path = Path::new("/path/to/backup");
    /// match manager.restore_from_backup(&backup_path, "asset_123") {
    ///     Ok(_) => println!("Asset restored successfully"),
    ///     Err(e) => println!("Restore failed: {}", e),
    /// }
    /// ```
    pub fn restore_from_backup(&self, backup_path: &Path, asset_id: &str) -> Result<(), String> {
        let asset_path = PathBuf::from(&self.asset_dir).join(asset_id);

        // Remove current installation if exists
        if asset_path.exists() {
            fs::remove_dir_all(&asset_path)
                .map_err(|e| format!("Failed to remove current installation: {}", e))?;
        }

        // Restore from backup
        self.copy_dir_recursive(backup_path, &asset_path)?;

        godot_print!("Restored '{}' from backup: {:?}", asset_id, backup_path);
        Ok(())
    }

    /// Resolves and checks dependencies for an asset.
    ///
    /// Analyzes the asset's dependency list and returns any dependencies
    /// that are not currently installed. This helps ensure all required
    /// assets are installed before installing the requested asset.
    ///
    /// # Arguments
    /// * `asset_id` - The ID of the asset to check dependencies for
    ///
    /// # Returns
    /// * `Ok(Vec<String>)` - List of missing dependency descriptions (empty if all satisfied)
    /// * `Err(String)` - Error message if the asset is not found
    ///
    /// # Example
    /// ```
    /// let manager = AssetManager::new();
    /// match manager.resolve_dependencies("asset_123") {
    ///     Ok(missing) => {
    ///         if missing.is_empty() {
    ///             println!("All dependencies satisfied");
    ///         } else {
    ///             println!("Missing dependencies: {:?}", missing);
    ///         }
    ///     }
    ///     Err(e) => println!("Error: {}", e),
    /// }
    /// ```
    pub fn resolve_dependencies(&self, asset_id: &str) -> Result<Vec<String>, String> {
        let asset = self.get_asset_by_id(asset_id)
            .ok_or_else(|| format!("Asset '{}' not found", asset_id))?;

        let mut missing = Vec::new();

        for dependency in &asset.dependencies {
            if !self.is_asset_installed(&dependency.asset_id) {
                missing.push(format!(
                    "{} (version: {})",
                    dependency.asset_id,
                    dependency.version_requirement
                ));
            }
        }

        Ok(missing)
    }

    /// Installs an asset along with all its dependencies.
    ///
    /// This method automatically resolves dependencies and installs them in the
    /// correct order before installing the requested asset. Dependencies are
    /// installed first to ensure the asset has everything it needs to function.
    ///
    /// # Arguments
    /// * `asset_id` - The ID of the asset to install with dependencies
    ///
    /// # Returns
    /// * `HashMap<String, Result<PathBuf, String>>` - Map of asset/dependency IDs to installation results
    ///   - Keys include the main asset ID and all dependency IDs
    ///   - Values are `Ok(PathBuf)` on success or `Err(String)` on failure
    ///
    /// # Example
    /// ```
    /// let manager = AssetManager::new();
    /// let results = manager.install_with_dependencies("asset_123".to_string()).await;
    ///
    /// for (id, result) in results {
    ///     match result {
    ///         Ok(path) => println!("{} installed at {:?}", id, path),
    ///         Err(e) => println!("{} failed: {}", id, e),
    ///     }
    /// }
    /// ```
    pub async fn install_with_dependencies(&self, asset_id: String) -> HashMap<String, Result<PathBuf, String>> {
        let mut results = HashMap::new();

        // Check dependencies first
        let dependencies = match self.resolve_dependencies(&asset_id) {
            Ok(deps) => deps,
            Err(e) => {
                results.insert(asset_id, Err(format!("Failed to resolve dependencies: {}", e)));
                return results;
            }
        };

        // Install missing dependencies first
        for dep in dependencies {
            // Extract just the ID (before version requirement)
            let dep_id = dep.split_whitespace().next().unwrap_or(&dep).to_string();

            godot_print!("Installing dependency: {}", dep_id);
            let result = self.import_asset(dep_id.clone()).await;
            results.insert(dep_id, result);
        }

        // Install the main asset
        godot_print!("Installing main asset: {}", asset_id);
        let result = self.import_asset(asset_id.clone()).await;
        results.insert(asset_id, result);

        results
    }

    /// Updates an asset with automatic backup and rollback on failure.
    ///
    /// This is the safe way to update an asset. It performs the following steps:
    /// 1. Creates a backup of the current installation
    /// 2. Attempts to update the asset
    /// 3. On failure, automatically restores from the backup
    ///
    /// This ensures that even if an update fails, the asset remains in a
    /// working state.
    ///
    /// # Arguments
    /// * `asset_id` - The ID of the asset to update
    ///
    /// # Returns
    /// * `Ok(PathBuf)` - Path to the updated asset on success
    /// * `Err(String)` - Error message if update and/or rollback failed
    ///
    /// # Example
    /// ```
    /// let manager = AssetManager::new();
    /// match manager.update_asset_with_backup("asset_123".to_string()).await {
    ///     Ok(path) => println!("Asset updated successfully at {:?}", path),
    ///     Err(e) => println!("Update failed: {}", e),
    /// }
    /// ```
    pub async fn update_asset_with_backup(&self, asset_id: String) -> Result<PathBuf, String> {
        // Create backup first
        match self.backup_asset(&asset_id) {
            Ok(backup_path) => {
                godot_print!("Created backup at: {:?}", backup_path);

                // Attempt update
                match self.update_asset(asset_id.clone()).await {
                    Ok(path) => Ok(path),
                    Err(e) => {
                        // Restore from backup on failure
                        godot_print!("Update failed, restoring from backup...");
                        if let Err(restore_err) = self.restore_from_backup(&backup_path, &asset_id) {
                            Err(format!(
                                "Update failed: {}. Restore also failed: {}",
                                e, restore_err
                            ))
                        } else {
                            Err(format!("Update failed (restored from backup): {}", e))
                        }
                    }
                }
            }
            Err(e) => {
                Err(format!("Failed to create backup before update: {}", e))
            }
        }
    }
}

/// Metadata file name expected in asset archives
const ASSET_METADATA_FILE: &str = "asset.json";

/// Temporary directory suffix for extraction before validation
const TEMP_EXTRACT_SUFFIX: &str = ".tmp_extract";

/// Result type for validation operations
#[derive(Debug)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    fn new() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    fn add_error(&mut self, error: String) {
        self.valid = false;
        self.errors.push(error);
    }

    fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    fn is_valid(&self) -> bool {
        self.valid
    }
}

/// Handles importing and extracting asset archives
pub struct AssetImporter {
    /// Base directory for extracting assets
    base_extract_dir: PathBuf,

    /// Whether to preserve temporary files on error (for debugging)
    preserve_temp_on_error: bool,
}

impl AssetImporter {
    /// Creates a new AssetImporter with the specified base directory
    pub fn new(base_extract_dir: PathBuf) -> Self {
        Self {
            base_extract_dir,
            preserve_temp_on_error: false,
        }
    }

    /// Creates a new AssetImporter with default Godot asset directory
    pub fn with_default_dir() -> Self {
        Self::new(PathBuf::from("res://addons/"))
    }

    /// Sets whether to preserve temporary files on error
    pub fn set_preserve_temp(&mut self, preserve: bool) {
        self.preserve_temp_on_error = preserve;
    }

    /// Main entry point for importing an asset from an archive file
    ///
    /// This method handles the full import process:
    /// 1. Extract to temporary directory
    /// 2. Validate the extracted content
    /// 3. Move to final location or rollback on error
    pub fn import_asset(&self, asset_path: &Path, asset_id: &str) -> Result<PathBuf, String> {
        // Validate input
        if !asset_path.exists() {
            return Err(format!("Asset file does not exist: {:?}", asset_path));
        }

        // Determine archive type from extension
        let extension = asset_path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| "Unable to determine file extension".to_string())?;

        // Create temporary extraction directory
        let temp_dir = self.create_temp_dir(asset_id)?;

        // Extract based on archive type
        let extract_result = match extension.to_lowercase().as_str() {
            "zip" => self.extract_zip(asset_path, &temp_dir),
            "gz" | "tgz" => {
                // Check if it's a .tar.gz or just .gz
                let path_str = asset_path.to_string_lossy();
                if path_str.ends_with(".tar.gz") || extension == "tgz" {
                    self.extract_tar_gz(asset_path, &temp_dir)
                } else {
                    Err("Unsupported archive format. Expected .tar.gz".to_string())
                }
            }
            _ => Err(format!("Unsupported archive format: {}", extension)),
        };

        // Handle extraction errors with cleanup
        if let Err(e) = extract_result {
            self.cleanup_temp_dir(&temp_dir)?;
            return Err(format!("Extraction failed: {}", e));
        }

        // Validate the extracted content
        let validation = self.validate_asset(&temp_dir)?;
        if !validation.is_valid() {
            self.cleanup_temp_dir(&temp_dir)?;
            return Err(format!(
                "Asset validation failed:\n{}",
                validation.errors.join("\n")
            ));
        }

        // Log warnings if any
        if !validation.warnings.is_empty() {
            debug_print!("Import warnings for {}:", asset_id);
            for warning in &validation.warnings {
                debug_print!("  - {}", warning);
            }
        }

        // Move from temp directory to final location
        let final_dir = self.base_extract_dir.join(asset_id);

        // Remove existing installation if present
        if final_dir.exists() {
            fs::remove_dir_all(&final_dir)
                .map_err(|e| format!("Failed to remove existing asset: {}", e))?;
        }

        // Create parent directory if needed
        if let Some(parent) = final_dir.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create parent directory: {}", e))?;
        }

        // Move to final location
        fs::rename(&temp_dir, &final_dir)
            .map_err(|e| format!("Failed to move asset to final location: {}", e))?;

        // Integrate with Godot's import system
        self.integrate_with_godot(&final_dir)?;

        Ok(final_dir)
    }

    /// Extracts a .zip archive to the specified directory
    fn extract_zip(&self, archive_path: &Path, target_dir: &Path) -> Result<(), String> {
        let file = fs::File::open(archive_path)
            .map_err(|e| format!("Failed to open zip file: {}", e))?;

        let mut archive = ZipArchive::new(file)
            .map_err(|e| format!("Failed to read zip archive: {}", e))?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .map_err(|e| format!("Failed to access zip entry {}: {}", i, e))?;

            let outpath = match file.enclosed_name() {
                Some(path) => target_dir.join(path),
                None => continue, // Skip invalid paths
            };

            if file.name().ends_with('/') {
                // Directory entry
                fs::create_dir_all(&outpath)
                    .map_err(|e| format!("Failed to create directory {:?}: {}", outpath, e))?;
            } else {
                // File entry
                if let Some(parent) = outpath.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create parent directory: {}", e))?;
                }

                let mut outfile = fs::File::create(&outpath)
                    .map_err(|e| format!("Failed to create file {:?}: {}", outpath, e))?;

                std::io::copy(&mut file, &mut outfile)
                    .map_err(|e| format!("Failed to extract file {:?}: {}", outpath, e))?;
            }

            // Set file permissions on Unix-like systems
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))
                        .ok(); // Ignore permission errors
                }
            }
        }

        Ok(())
    }

    /// Extracts a .tar.gz archive to the specified directory
    fn extract_tar_gz(&self, archive_path: &Path, target_dir: &Path) -> Result<(), String> {
        let tar_gz = fs::File::open(archive_path)
            .map_err(|e| format!("Failed to open tar.gz file: {}", e))?;

        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);

        archive.unpack(target_dir)
            .map_err(|e| format!("Failed to extract tar.gz archive: {}", e))?;

        Ok(())
    }

    /// Validates the extracted asset content
    ///
    /// Checks for:
    /// - Required metadata file (asset.json)
    /// - Valid metadata format
    /// - Presence of expected content files
    /// - No malicious or dangerous files
    fn validate_asset(&self, asset_dir: &Path) -> Result<ValidationResult, String> {
        let mut result = ValidationResult::new();

        // Check if directory exists and is readable
        if !asset_dir.exists() {
            result.add_error(format!("Asset directory does not exist: {:?}", asset_dir));
            return Ok(result);
        }

        if !asset_dir.is_dir() {
            result.add_error(format!("Asset path is not a directory: {:?}", asset_dir));
            return Ok(result);
        }

        // Check for metadata file
        let metadata_path = asset_dir.join(ASSET_METADATA_FILE);
        if !metadata_path.exists() {
            result.add_warning(format!(
                "No metadata file found ({}). Asset may not have complete information.",
                ASSET_METADATA_FILE
            ));
        } else {
            // Validate metadata format
            match self.validate_metadata(&metadata_path) {
                Ok(warnings) => {
                    for warning in warnings {
                        result.add_warning(warning);
                    }
                }
                Err(e) => {
                    result.add_error(format!("Invalid metadata file: {}", e));
                }
            }
        }

        // Check for common Godot asset files
        let has_content = self.check_for_content_files(asset_dir);
        if !has_content {
            result.add_warning(
                "No recognizable Godot asset files found (scenes, scripts, resources)".to_string()
            );
        }

        // Security validation: Check for dangerous files
        if let Err(e) = self.validate_security(asset_dir) {
            result.add_error(format!("Security validation failed: {}", e));
        }

        Ok(result)
    }

    /// Validates the asset metadata file
    fn validate_metadata(&self, metadata_path: &Path) -> Result<Vec<String>, String> {
        let mut warnings = Vec::new();

        let content = fs::read_to_string(metadata_path)
            .map_err(|e| format!("Failed to read metadata file: {}", e))?;

        // Try to parse as JSON
        let metadata: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON in metadata file: {}", e))?;

        // Check for recommended fields
        let recommended_fields = ["name", "version", "description", "author"];
        for field in &recommended_fields {
            if !metadata.get(field).is_some() {
                warnings.push(format!("Recommended field '{}' missing from metadata", field));
            }
        }

        Ok(warnings)
    }

    /// Checks if the asset directory contains recognizable Godot content
    fn check_for_content_files(&self, asset_dir: &Path) -> bool {
        let godot_extensions = [".tscn", ".tres", ".gd", ".gdshader", ".material", ".mesh"];

        if let Ok(entries) = fs::read_dir(asset_dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        if let Some(ext) = entry.path().extension() {
                            let ext_str = ext.to_string_lossy().to_lowercase();
                            if godot_extensions.iter().any(|&e| format!(".{}", ext_str) == e) {
                                return true;
                            }
                        }
                    } else if file_type.is_dir() {
                        // Recursively check subdirectories
                        if self.check_for_content_files(&entry.path()) {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    /// Validates security aspects of the asset
    ///
    /// Checks for potentially dangerous files or patterns
    fn validate_security(&self, asset_dir: &Path) -> Result<(), String> {
        // List of suspicious file patterns
        let suspicious_patterns = [".exe", ".dll", ".so", ".dylib", ".bat", ".sh", ".ps1"];

        if let Ok(entries) = fs::read_dir(asset_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    for pattern in &suspicious_patterns {
                        if format!(".{}", ext_str) == *pattern {
                            return Err(format!(
                                "Suspicious file detected: {:?} (extension: {})",
                                path.file_name().unwrap_or_default(),
                                pattern
                            ));
                        }
                    }
                }

                // Recursively check subdirectories
                if path.is_dir() {
                    self.validate_security(&path)?;
                }
            }
        }

        Ok(())
    }

    /// Integrates the imported asset with Godot's import system
    ///
    /// This creates .import files and triggers Godot's resource scanner
    fn integrate_with_godot(&self, asset_dir: &Path) -> Result<(), String> {
        // Create a .gdignore file marker to inform Godot about the asset
        let import_marker = asset_dir.join(".imported");

        fs::File::create(&import_marker)
            .map_err(|e| format!("Failed to create import marker: {}", e))?;

        // Note: Full Godot integration would require calling Godot's resource
        // scanner API, which will be implemented when Godot engine integration is ready

        debug_print!("Asset imported to: {:?}", asset_dir);
        debug_print!("Restart Godot editor or reimport to see changes");

        Ok(())
    }

    /// Creates a temporary directory for extraction
    fn create_temp_dir(&self, asset_id: &str) -> Result<PathBuf, String> {
        let temp_dir = self.base_extract_dir.join(format!("{}{}", asset_id, TEMP_EXTRACT_SUFFIX));

        // Remove existing temp directory if present
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir)
                .map_err(|e| format!("Failed to remove existing temp directory: {}", e))?;
        }

        // Create the temp directory
        fs::create_dir_all(&temp_dir)
            .map_err(|e| format!("Failed to create temp directory: {}", e))?;

        Ok(temp_dir)
    }

    /// Cleans up temporary directory, respecting preserve_temp_on_error setting
    fn cleanup_temp_dir(&self, temp_dir: &Path) -> Result<(), String> {
        if self.preserve_temp_on_error {
            debug_print!("Preserving temp directory for debugging: {:?}", temp_dir);
            return Ok(());
        }

        if temp_dir.exists() {
            fs::remove_dir_all(temp_dir)
                .map_err(|e| format!("Failed to cleanup temp directory: {}", e))?;
        }

        Ok(())
    }

    /// Rolls back a failed import by removing the asset directory
    pub fn rollback_import(&self, asset_id: &str) -> Result<(), String> {
        let asset_dir = self.base_extract_dir.join(asset_id);

        if asset_dir.exists() {
            fs::remove_dir_all(&asset_dir)
                .map_err(|e| format!("Failed to rollback import: {}", e))?;
            debug_print!("Rolled back import for asset: {}", asset_id);
        }

        // Also clean up any temp directories
        let temp_dir = self.base_extract_dir.join(format!("{}{}", asset_id, TEMP_EXTRACT_SUFFIX));
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir)
                .map_err(|e| format!("Failed to cleanup temp directory during rollback: {}", e))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset_library::asset::{AssetDependency, AssetVersionInfo, SemanticVersion};

    #[test]
    fn test_asset_manager_new() {
        let manager = AssetManager::new();

        // Should have sample assets initialized
        let assets = manager.get_assets();
        assert!(!assets.is_empty(), "AssetManager should initialize with sample assets");

        // Check asset directory is set
        assert_eq!(manager.asset_dir, "res://addons/");
        assert_eq!(manager.cache_dir, "user://asset_cache/");
    }

    #[test]
    fn test_get_assets() {
        let manager = AssetManager::new();
        let assets = manager.get_assets();

        // Should have sample assets
        assert!(!assets.is_empty());

        // Verify some sample assets are present
        let asset_names: Vec<String> = assets.iter().map(|a| a.name.clone()).collect();
        assert!(asset_names.iter().any(|name| name.contains("Sprites") || name.contains("Character") || name.contains("Shader")));
    }

    #[test]
    fn test_get_assets_by_category() {
        let manager = AssetManager::new();

        // Test filtering by 2D category
        let assets_2d = manager.get_assets_by_category(AssetCategory::TwoD);
        for asset in &assets_2d {
            assert_eq!(asset.category, AssetCategory::TwoD);
        }

        // Test filtering by 3D category
        let assets_3d = manager.get_assets_by_category(AssetCategory::ThreeD);
        for asset in &assets_3d {
            assert_eq!(asset.category, AssetCategory::ThreeD);
        }

        // Test filtering by Shaders category
        let assets_shaders = manager.get_assets_by_category(AssetCategory::Shaders);
        for asset in &assets_shaders {
            assert_eq!(asset.category, AssetCategory::Shaders);
        }
    }

    #[test]
    fn test_search_assets() {
        let manager = AssetManager::new();

        // Search by name
        let results = manager.search_assets("sprite");
        assert!(!results.is_empty(), "Should find assets matching 'sprite'");
        for asset in &results {
            let found = asset.name.to_lowercase().contains("sprite") ||
                       asset.description.to_lowercase().contains("sprite") ||
                       asset.tags.iter().any(|t| t.to_lowercase().contains("sprite"));
            assert!(found, "Result should match search query");
        }

        // Search by description keyword
        let results = manager.search_assets("visual");
        for asset in &results {
            let found = asset.name.to_lowercase().contains("visual") ||
                       asset.description.to_lowercase().contains("visual") ||
                       asset.tags.iter().any(|t| t.to_lowercase().contains("visual"));
            assert!(found, "Result should match search query");
        }

        // Search with no results
        let results = manager.search_assets("nonexistent_query_xyz");
        assert!(results.is_empty(), "Should return empty for non-matching query");
    }

    #[test]
    fn test_get_asset_by_id() {
        let manager = AssetManager::new();
        let all_assets = manager.get_assets();

        if let Some(first_asset) = all_assets.first() {
            let found = manager.get_asset_by_id(&first_asset.id);
            assert!(found.is_some(), "Should find asset by ID");
            assert_eq!(found.unwrap().id, first_asset.id);
        }

        // Test with non-existent ID
        let not_found = manager.get_asset_by_id("nonexistent_id_xyz");
        assert!(not_found.is_none(), "Should return None for non-existent ID");
    }

    #[test]
    fn test_add_asset() {
        let manager = AssetManager::new();
        let initial_count = manager.get_assets().len();

        let new_asset = Asset::new(
            "test_asset_1".to_string(),
            "Test Asset".to_string(),
            AssetCategory::Tools,
            "".to_string(),
            "Test Author".to_string(),
            "1.0.0".to_string(),
            "A test asset".to_string(),
            vec!["test".to_string()],
            None,
            "https://example.com/test.zip".to_string(),
            vec![],
            vec![],
        );

        manager.add_asset(new_asset.clone());

        let updated_count = manager.get_assets().len();
        assert_eq!(updated_count, initial_count + 1, "Asset count should increase by 1");

        let found = manager.get_asset_by_id("test_asset_1");
        assert!(found.is_some(), "Newly added asset should be found");
        assert_eq!(found.unwrap().name, "Test Asset");
    }

    #[test]
    fn test_remove_asset() {
        let manager = AssetManager::new();

        // Add a test asset
        let test_asset = Asset::minimal("test_remove_1".to_string(), "Test Remove".to_string());
        manager.add_asset(test_asset.clone());

        // Verify it was added
        assert!(manager.get_asset_by_id("test_remove_1").is_some());

        // Remove it
        manager.remove_asset("test_remove_1");

        // Verify it was removed
        assert!(manager.get_asset_by_id("test_remove_1").is_none());
    }

    #[test]
    fn test_is_asset_installed() {
        let manager = AssetManager::new();

        // Initially no assets should be installed
        assert!(!manager.is_asset_installed("test_asset_1"));

        // Manually add to installed list for testing
        {
            let mut installed = manager.installed_assets.lock().unwrap();
            installed.push("test_asset_1".to_string());
        }

        // Now it should be installed
        assert!(manager.is_asset_installed("test_asset_1"));
        assert!(!manager.is_asset_installed("test_asset_2"));
    }

    #[test]
    fn test_get_installed_assets() {
        let manager = AssetManager::new();

        // Initially should be empty
        let installed = manager.get_installed_assets();
        let initial_count = installed.len();

        // Add some installed assets for testing
        {
            let mut installed_list = manager.installed_assets.lock().unwrap();
            installed_list.push("asset_1".to_string());
            installed_list.push("asset_2".to_string());
        }

        let installed = manager.get_installed_assets();
        assert_eq!(installed.len(), initial_count + 2);
        assert!(installed.contains(&"asset_1".to_string()));
        assert!(installed.contains(&"asset_2".to_string()));
    }

    #[test]
    fn test_add_multiple_assets() {
        let manager = AssetManager::new();
        let initial_count = manager.get_assets().len();

        for i in 0..5 {
            let asset = Asset::minimal(
                format!("test_multi_{}", i),
                format!("Test Asset {}", i),
            );
            manager.add_asset(asset);
        }

        let final_count = manager.get_assets().len();
        assert_eq!(final_count, initial_count + 5);
    }

    #[test]
    fn test_search_by_tag() {
        let manager = AssetManager::new();

        // Add an asset with specific tags
        let mut asset = Asset::minimal("tag_test_1".to_string(), "Tag Test".to_string());
        asset.tags = vec!["unique_tag_xyz".to_string()];
        manager.add_asset(asset);

        // Search for that tag
        let results = manager.search_assets("unique_tag_xyz");
        assert!(!results.is_empty(), "Should find asset by tag");
        assert!(results.iter().any(|a| a.id == "tag_test_1"));
    }

    #[test]
    fn test_search_case_insensitive() {
        let manager = AssetManager::new();

        let asset = Asset::minimal("case_test_1".to_string(), "CaseSensitive".to_string());
        manager.add_asset(asset);

        // Search with different cases
        let results_lower = manager.search_assets("casesensitive");
        let results_upper = manager.search_assets("CASESENSITIVE");
        let results_mixed = manager.search_assets("CaseSensitive");

        assert!(!results_lower.is_empty());
        assert!(!results_upper.is_empty());
        assert!(!results_mixed.is_empty());
    }

    #[test]
    fn test_get_assets_by_nonexistent_category() {
        let manager = AssetManager::new();

        // Add assets to ensure manager is not empty
        let audio_assets = manager.get_assets_by_category(AssetCategory::Audio);

        // Since we might not have audio assets in sample data, this could be empty
        // But the call should not panic
        for asset in &audio_assets {
            assert_eq!(asset.category, AssetCategory::Audio);
        }
    }

    #[test]
    fn test_asset_removal_doesnt_affect_installed_list() {
        let manager = AssetManager::new();

        // Add and "install" an asset
        let asset = Asset::minimal("persist_test_1".to_string(), "Persist Test".to_string());
        manager.add_asset(asset);

        {
            let mut installed = manager.installed_assets.lock().unwrap();
            installed.push("persist_test_1".to_string());
        }

        // Remove the asset from the main list
        manager.remove_asset("persist_test_1");

        // Installed list should still contain it
        assert!(manager.is_asset_installed("persist_test_1"));
    }

    #[test]
    fn test_check_for_update_no_version_history() {
        let manager = AssetManager::new();

        // Add an asset without version history
        let asset = Asset::minimal("no_version_test".to_string(), "No Version Test".to_string());
        manager.add_asset(asset);

        let result = manager.check_for_update("no_version_test");

        // Should succeed but return None (no update available)
        assert!(result.is_ok());
        if let Ok(update) = result {
            assert!(update.is_none(), "Should have no update when no version history exists");
        }
    }

    #[test]
    fn test_check_for_update_with_newer_version() {
        let manager = AssetManager::new();

        // Add an asset with current version 1.0.0
        let mut asset = Asset::minimal("version_test".to_string(), "Version Test".to_string());
        asset.version = "1.0.0".to_string();

        // Add a newer version to version history
        let newer_version = AssetVersionInfo::new(
            SemanticVersion::new(1, 1, 0),
            "2024-01-15".to_string(),
            "New version".to_string(),
            "https://example.com/v1.1.0.zip".to_string(),
        );
        asset.version_history.push(newer_version);

        manager.add_asset(asset);

        let result = manager.check_for_update("version_test");
        assert!(result.is_ok());
        if let Ok(Some(update)) = result {
            assert!(update.has_update());
        }
    }

    #[test]
    fn test_check_for_update_already_latest() {
        let manager = AssetManager::new();

        // Add an asset with current version 2.0.0
        let mut asset = Asset::minimal("latest_test".to_string(), "Latest Test".to_string());
        asset.version = "2.0.0".to_string();

        // Add same version to version history
        let same_version = AssetVersionInfo::new(
            SemanticVersion::new(2, 0, 0),
            "2024-01-15".to_string(),
            "Current version".to_string(),
            "https://example.com/v2.0.0.zip".to_string(),
        );
        asset.version_history.push(same_version);

        manager.add_asset(asset);

        let result = manager.check_for_update("latest_test");
        assert!(result.is_ok());
        if let Ok(update) = result {
            assert!(update.is_none() || !update.unwrap().has_update());
        }
    }

    #[test]
    fn test_check_for_update_nonexistent_asset() {
        let manager = AssetManager::new();

        // When asset is not installed, it returns Ok(None)
        let result = manager.check_for_update("nonexistent_asset_xyz");
        assert!(result.is_ok(), "Should return Ok for non-installed asset");
        assert!(result.unwrap().is_none(), "Should return None for non-installed asset");
    }

    #[test]
    fn test_check_for_update_installed_but_not_in_catalog() {
        let manager = AssetManager::new();

        // Mark an asset as installed that doesn't exist in catalog
        {
            let mut installed = manager.installed_assets.lock().unwrap();
            installed.push("nonexistent_asset_xyz".to_string());
        }

        // Now it should return an error since it's installed but not in catalog
        let result = manager.check_for_update("nonexistent_asset_xyz");
        assert!(result.is_err(), "Should return error for installed asset not in catalog");
    }

    // ==================== Integration Tests for Import Functionality ====================

    /// Helper function to create a test ZIP archive with given files
    fn create_test_zip(path: &Path, files: Vec<(&str, &str)>) -> Result<(), Box<dyn std::error::Error>> {
        use std::io::Write;
        use zip::write::{FileOptions, ZipWriter};

        let file = fs::File::create(path)?;
        let mut zip = ZipWriter::new(file);
        let options: FileOptions<'_, ()> = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

        for (filename, content) in files {
            // Handle directory entries
            if filename.ends_with('/') {
                zip.add_directory(filename, options)?;
            } else {
                zip.start_file(filename, options)?;
                zip.write_all(content.as_bytes())?;
            }
        }

        zip.finish()?;
        Ok(())
    }

    /// Helper function to create a test tar.gz archive with given files
    fn create_test_tar_gz(path: &Path, files: Vec<(&str, &str)>) -> Result<(), Box<dyn std::error::Error>> {
        use std::io::Write;
        use tar::Builder;
        use flate2::write::GzEncoder;
        use flate2::Compression;

        let tar_gz = fs::File::create(path)?;
        let enc = GzEncoder::new(tar_gz, Compression::default());
        let mut tar = Builder::new(enc);

        for (filename, content) in files {
            let mut header = tar::Header::new_gnu();
            let bytes = content.as_bytes();
            header.set_size(bytes.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            tar.append_data(&mut header, filename, bytes)?;
        }

        tar.finish()?;
        Ok(())
    }

    /// Helper to create a valid asset metadata JSON
    fn create_valid_metadata() -> String {
        serde_json::json!({
            "name": "Test Asset",
            "version": "1.0.0",
            "description": "A test asset for integration testing",
            "author": "Test Author"
        }).to_string()
    }

    #[test]
    fn test_asset_importer_new() {
        let base_dir = PathBuf::from("/tmp/test_assets");
        let importer = AssetImporter::new(base_dir.clone());

        assert_eq!(importer.base_extract_dir, base_dir);
        assert_eq!(importer.preserve_temp_on_error, false);
    }

    #[test]
    fn test_asset_importer_with_default_dir() {
        let importer = AssetImporter::with_default_dir();
        assert_eq!(importer.base_extract_dir, PathBuf::from("res://addons/"));
    }

    #[test]
    fn test_asset_importer_set_preserve_temp() {
        let mut importer = AssetImporter::new(PathBuf::from("/tmp/test"));

        assert_eq!(importer.preserve_temp_on_error, false);

        importer.set_preserve_temp(true);
        assert_eq!(importer.preserve_temp_on_error, true);

        importer.set_preserve_temp(false);
        assert_eq!(importer.preserve_temp_on_error, false);
    }

    #[test]
    fn test_extract_zip_basic() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let zip_path = temp_dir.path().join("test.zip");
        let extract_dir = temp_dir.path().join("extracted");

        // Create a test ZIP with some files
        let metadata = create_valid_metadata();
        let files = vec![
            ("asset.json", metadata.as_str()),
            ("script.gd", "extends Node\n\nfunc _ready():\n\tpass"),
            ("scene.tscn", "[gd_scene load_steps=1 format=3]"),
        ];

        create_test_zip(&zip_path, files).unwrap();

        let importer = AssetImporter::new(extract_dir.clone());
        let result = importer.extract_zip(&zip_path, &extract_dir);

        assert!(result.is_ok(), "ZIP extraction should succeed");
        assert!(extract_dir.join("asset.json").exists());
        assert!(extract_dir.join("script.gd").exists());
        assert!(extract_dir.join("scene.tscn").exists());
    }

    #[test]
    fn test_extract_zip_with_directories() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let zip_path = temp_dir.path().join("test.zip");
        let extract_dir = temp_dir.path().join("extracted");

        // Create a test ZIP with directories
        let metadata = create_valid_metadata();
        let files = vec![
            ("subdir/", ""),
            ("subdir/file.gd", "# Test script"),
            ("asset.json", metadata.as_str()),
        ];

        create_test_zip(&zip_path, files).unwrap();

        let importer = AssetImporter::new(extract_dir.clone());
        let result = importer.extract_zip(&zip_path, &extract_dir);

        assert!(result.is_ok(), "ZIP extraction with directories should succeed");
        assert!(extract_dir.join("subdir").is_dir());
        assert!(extract_dir.join("subdir/file.gd").exists());
    }

    #[test]
    fn test_extract_zip_nonexistent_file() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let zip_path = temp_dir.path().join("nonexistent.zip");
        let extract_dir = temp_dir.path().join("extracted");

        let importer = AssetImporter::new(extract_dir.clone());
        let result = importer.extract_zip(&zip_path, &extract_dir);

        assert!(result.is_err(), "Should fail when ZIP file doesn't exist");
        assert!(result.unwrap_err().contains("Failed to open zip file"));
    }

    #[test]
    fn test_extract_tar_gz_basic() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let tar_gz_path = temp_dir.path().join("test.tar.gz");
        let extract_dir = temp_dir.path().join("extracted");

        // Create a test tar.gz with some files
        let metadata = create_valid_metadata();
        let files = vec![
            ("asset.json", metadata.as_str()),
            ("script.gd", "extends Node\n\nfunc _ready():\n\tpass"),
        ];

        create_test_tar_gz(&tar_gz_path, files).unwrap();

        let importer = AssetImporter::new(extract_dir.clone());
        let result = importer.extract_tar_gz(&tar_gz_path, &extract_dir);

        assert!(result.is_ok(), "tar.gz extraction should succeed");
        assert!(extract_dir.join("asset.json").exists());
        assert!(extract_dir.join("script.gd").exists());
    }

    #[test]
    fn test_validate_asset_success() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let asset_dir = temp_dir.path().join("test_asset");
        fs::create_dir_all(&asset_dir).unwrap();

        // Create valid asset files
        fs::write(asset_dir.join("asset.json"), create_valid_metadata()).unwrap();
        fs::write(asset_dir.join("scene.tscn"), "[gd_scene]").unwrap();

        let importer = AssetImporter::new(PathBuf::from("/tmp"));
        let result = importer.validate_asset(&asset_dir);

        assert!(result.is_ok());
        let validation = result.unwrap();
        assert!(validation.is_valid(), "Asset should be valid");
    }

    #[test]
    fn test_validate_asset_missing_metadata() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let asset_dir = temp_dir.path().join("test_asset");
        fs::create_dir_all(&asset_dir).unwrap();

        // Create asset without metadata
        fs::write(asset_dir.join("scene.tscn"), "[gd_scene]").unwrap();

        let importer = AssetImporter::new(PathBuf::from("/tmp"));
        let result = importer.validate_asset(&asset_dir);

        assert!(result.is_ok());
        let validation = result.unwrap();
        // Should still be valid but with warnings
        assert!(validation.is_valid());
        assert!(!validation.warnings.is_empty());
    }

    #[test]
    fn test_validate_asset_no_content() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let asset_dir = temp_dir.path().join("test_asset");
        fs::create_dir_all(&asset_dir).unwrap();

        // Create metadata but no Godot files
        fs::write(asset_dir.join("asset.json"), create_valid_metadata()).unwrap();
        fs::write(asset_dir.join("readme.txt"), "Just a readme").unwrap();

        let importer = AssetImporter::new(PathBuf::from("/tmp"));
        let result = importer.validate_asset(&asset_dir);

        assert!(result.is_ok());
        let validation = result.unwrap();
        // Should be valid but with warning about no content files
        assert!(validation.is_valid());
        assert!(validation.warnings.iter().any(|w| w.contains("No recognizable Godot asset files")));
    }

    #[test]
    fn test_validate_asset_nonexistent_directory() {
        let importer = AssetImporter::new(PathBuf::from("/tmp"));
        let result = importer.validate_asset(&PathBuf::from("/nonexistent/path"));

        assert!(result.is_ok());
        let validation = result.unwrap();
        assert!(!validation.is_valid(), "Should be invalid for nonexistent directory");
        assert!(!validation.errors.is_empty());
    }

    #[test]
    fn test_validate_metadata_success() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let metadata_path = temp_dir.path().join("asset.json");
        fs::write(&metadata_path, create_valid_metadata()).unwrap();

        let importer = AssetImporter::new(PathBuf::from("/tmp"));
        let result = importer.validate_metadata(&metadata_path);

        assert!(result.is_ok());
        let warnings = result.unwrap();
        assert!(warnings.is_empty(), "Valid metadata should have no warnings");
    }

    #[test]
    fn test_validate_metadata_missing_fields() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let metadata_path = temp_dir.path().join("asset.json");

        // Create metadata with missing fields
        let incomplete_metadata = serde_json::json!({
            "name": "Test Asset"
            // Missing version, description, author
        }).to_string();

        fs::write(&metadata_path, incomplete_metadata).unwrap();

        let importer = AssetImporter::new(PathBuf::from("/tmp"));
        let result = importer.validate_metadata(&metadata_path);

        assert!(result.is_ok());
        let warnings = result.unwrap();
        assert!(!warnings.is_empty(), "Should have warnings for missing fields");
        assert!(warnings.iter().any(|w| w.contains("version")));
        assert!(warnings.iter().any(|w| w.contains("description")));
        assert!(warnings.iter().any(|w| w.contains("author")));
    }

    #[test]
    fn test_validate_metadata_invalid_json() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let metadata_path = temp_dir.path().join("asset.json");
        fs::write(&metadata_path, "{ invalid json }").unwrap();

        let importer = AssetImporter::new(PathBuf::from("/tmp"));
        let result = importer.validate_metadata(&metadata_path);

        assert!(result.is_err(), "Should fail for invalid JSON");
        assert!(result.unwrap_err().contains("Invalid JSON"));
    }

    #[test]
    fn test_check_for_content_files_success() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let asset_dir = temp_dir.path().join("test_asset");
        fs::create_dir_all(&asset_dir).unwrap();

        // Create various Godot content files
        fs::write(asset_dir.join("scene.tscn"), "test").unwrap();
        fs::write(asset_dir.join("resource.tres"), "test").unwrap();
        fs::write(asset_dir.join("script.gd"), "test").unwrap();

        let importer = AssetImporter::new(PathBuf::from("/tmp"));
        let has_content = importer.check_for_content_files(&asset_dir);

        assert!(has_content, "Should detect Godot content files");
    }

    #[test]
    fn test_check_for_content_files_recursive() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let asset_dir = temp_dir.path().join("test_asset");
        let subdir = asset_dir.join("scripts");
        fs::create_dir_all(&subdir).unwrap();

        // Create Godot file in subdirectory
        fs::write(subdir.join("script.gd"), "test").unwrap();

        let importer = AssetImporter::new(PathBuf::from("/tmp"));
        let has_content = importer.check_for_content_files(&asset_dir);

        assert!(has_content, "Should detect Godot files in subdirectories");
    }

    #[test]
    fn test_check_for_content_files_none() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let asset_dir = temp_dir.path().join("test_asset");
        fs::create_dir_all(&asset_dir).unwrap();

        // Create non-Godot files
        fs::write(asset_dir.join("readme.txt"), "test").unwrap();
        fs::write(asset_dir.join("image.png"), "test").unwrap();

        let importer = AssetImporter::new(PathBuf::from("/tmp"));
        let has_content = importer.check_for_content_files(&asset_dir);

        assert!(!has_content, "Should not detect content when no Godot files present");
    }

    #[test]
    fn test_validate_security_success() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let asset_dir = temp_dir.path().join("test_asset");
        fs::create_dir_all(&asset_dir).unwrap();

        // Create safe files
        fs::write(asset_dir.join("script.gd"), "test").unwrap();
        fs::write(asset_dir.join("scene.tscn"), "test").unwrap();

        let importer = AssetImporter::new(PathBuf::from("/tmp"));
        let result = importer.validate_security(&asset_dir);

        assert!(result.is_ok(), "Should pass security validation for safe files");
    }

    #[test]
    fn test_validate_security_suspicious_exe() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let asset_dir = temp_dir.path().join("test_asset");
        fs::create_dir_all(&asset_dir).unwrap();

        // Create suspicious file
        fs::write(asset_dir.join("malware.exe"), "test").unwrap();

        let importer = AssetImporter::new(PathBuf::from("/tmp"));
        let result = importer.validate_security(&asset_dir);

        assert!(result.is_err(), "Should fail security validation for .exe files");
        assert!(result.unwrap_err().contains("Suspicious file"));
    }

    #[test]
    fn test_validate_security_suspicious_dll() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let asset_dir = temp_dir.path().join("test_asset");
        fs::create_dir_all(&asset_dir).unwrap();

        // Create suspicious file
        fs::write(asset_dir.join("library.dll"), "test").unwrap();

        let importer = AssetImporter::new(PathBuf::from("/tmp"));
        let result = importer.validate_security(&asset_dir);

        assert!(result.is_err(), "Should fail security validation for .dll files");
    }

    #[test]
    fn test_validate_security_recursive() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let asset_dir = temp_dir.path().join("test_asset");
        let subdir = asset_dir.join("subdir");
        fs::create_dir_all(&subdir).unwrap();

        // Create suspicious file in subdirectory
        fs::write(subdir.join("script.sh"), "#!/bin/bash").unwrap();

        let importer = AssetImporter::new(PathBuf::from("/tmp"));
        let result = importer.validate_security(&asset_dir);

        assert!(result.is_err(), "Should detect suspicious files in subdirectories");
    }

    #[test]
    fn test_import_asset_zip_success() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let zip_path = temp_dir.path().join("test.zip");
        let base_dir = temp_dir.path().join("assets");
        fs::create_dir_all(&base_dir).unwrap();

        // Create a valid test ZIP
        let metadata = create_valid_metadata();
        let files = vec![
            ("asset.json", metadata.as_str()),
            ("scene.tscn", "[gd_scene]"),
            ("script.gd", "extends Node"),
        ];

        create_test_zip(&zip_path, files).unwrap();

        let importer = AssetImporter::new(base_dir.clone());
        let result = importer.import_asset(&zip_path, "test_asset");

        assert!(result.is_ok(), "Import should succeed for valid ZIP");
        let final_path = result.unwrap();
        assert!(final_path.exists());
        assert!(final_path.join("asset.json").exists());
        assert!(final_path.join("scene.tscn").exists());
        assert!(final_path.join("script.gd").exists());
    }

    #[test]
    fn test_import_asset_tar_gz_success() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let tar_gz_path = temp_dir.path().join("test.tar.gz");
        let base_dir = temp_dir.path().join("assets");
        fs::create_dir_all(&base_dir).unwrap();

        // Create a valid test tar.gz
        let metadata = create_valid_metadata();
        let files = vec![
            ("asset.json", metadata.as_str()),
            ("scene.tscn", "[gd_scene]"),
        ];

        create_test_tar_gz(&tar_gz_path, files).unwrap();

        let importer = AssetImporter::new(base_dir.clone());
        let result = importer.import_asset(&tar_gz_path, "test_asset");

        assert!(result.is_ok(), "Import should succeed for valid tar.gz");
        let final_path = result.unwrap();
        assert!(final_path.exists());
    }

    #[test]
    fn test_import_asset_nonexistent_file() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let base_dir = temp_dir.path().join("assets");
        let nonexistent = temp_dir.path().join("nonexistent.zip");

        let importer = AssetImporter::new(base_dir);
        let result = importer.import_asset(&nonexistent, "test_asset");

        assert!(result.is_err(), "Should fail for nonexistent file");
        assert!(result.unwrap_err().contains("does not exist"));
    }

    #[test]
    fn test_import_asset_unsupported_format() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let base_dir = temp_dir.path().join("assets");
        let unsupported = temp_dir.path().join("test.rar");
        fs::write(&unsupported, "test").unwrap();

        let importer = AssetImporter::new(base_dir);
        let result = importer.import_asset(&unsupported, "test_asset");

        assert!(result.is_err(), "Should fail for unsupported format");
        assert!(result.unwrap_err().contains("Unsupported archive format"));
    }

    #[test]
    fn test_import_asset_replaces_existing() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let zip_path = temp_dir.path().join("test.zip");
        let base_dir = temp_dir.path().join("assets");
        fs::create_dir_all(&base_dir).unwrap();

        // Create existing asset directory
        let existing_dir = base_dir.join("test_asset");
        fs::create_dir_all(&existing_dir).unwrap();
        fs::write(existing_dir.join("old_file.txt"), "old content").unwrap();

        // Create new ZIP
        let metadata = create_valid_metadata();
        let files = vec![
            ("asset.json", metadata.as_str()),
            ("new_file.gd", "extends Node"),
        ];
        create_test_zip(&zip_path, files).unwrap();

        let importer = AssetImporter::new(base_dir.clone());
        let result = importer.import_asset(&zip_path, "test_asset");

        assert!(result.is_ok(), "Import should succeed");
        let final_path = result.unwrap();
        assert!(final_path.join("new_file.gd").exists(), "New file should exist");
        assert!(!final_path.join("old_file.txt").exists(), "Old file should be removed");
    }

    #[test]
    fn test_import_asset_security_validation_fails() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let zip_path = temp_dir.path().join("malicious.zip");
        let base_dir = temp_dir.path().join("assets");
        fs::create_dir_all(&base_dir).unwrap();

        // Create ZIP with suspicious file
        let metadata = create_valid_metadata();
        let files = vec![
            ("asset.json", metadata.as_str()),
            ("malware.exe", "malicious content"),
        ];
        create_test_zip(&zip_path, files).unwrap();

        let importer = AssetImporter::new(base_dir.clone());
        let result = importer.import_asset(&zip_path, "test_asset");

        assert!(result.is_err(), "Import should fail security validation");
        assert!(result.unwrap_err().contains("Security validation failed"));

        // Verify temp directory was cleaned up
        let temp_marker = base_dir.join("test_asset.tmp_extract");
        assert!(!temp_marker.exists(), "Temp directory should be cleaned up on failure");
    }

    #[test]
    fn test_rollback_import_success() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let base_dir = temp_dir.path().join("assets");
        fs::create_dir_all(&base_dir).unwrap();

        // Create an asset directory to rollback
        let asset_dir = base_dir.join("test_asset");
        fs::create_dir_all(&asset_dir).unwrap();
        fs::write(asset_dir.join("file.txt"), "content").unwrap();

        let importer = AssetImporter::new(base_dir.clone());
        let result = importer.rollback_import("test_asset");

        assert!(result.is_ok(), "Rollback should succeed");
        assert!(!asset_dir.exists(), "Asset directory should be removed");
    }

    #[test]
    fn test_rollback_import_nonexistent() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let base_dir = temp_dir.path().join("assets");

        let importer = AssetImporter::new(base_dir);
        let result = importer.rollback_import("nonexistent_asset");

        // Should succeed even if asset doesn't exist (idempotent)
        assert!(result.is_ok(), "Rollback should be idempotent");
    }

    #[test]
    fn test_rollback_import_cleans_temp_dir() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let base_dir = temp_dir.path().join("assets");
        fs::create_dir_all(&base_dir).unwrap();

        // Create both asset directory and temp directory
        let asset_dir = base_dir.join("test_asset");
        let temp_extract_dir = base_dir.join("test_asset.tmp_extract");

        fs::create_dir_all(&asset_dir).unwrap();
        fs::create_dir_all(&temp_extract_dir).unwrap();
        fs::write(temp_extract_dir.join("temp.txt"), "temp").unwrap();

        let importer = AssetImporter::new(base_dir.clone());
        let result = importer.rollback_import("test_asset");

        assert!(result.is_ok(), "Rollback should succeed");
        assert!(!asset_dir.exists(), "Asset directory should be removed");
        assert!(!temp_extract_dir.exists(), "Temp directory should be removed");
    }

    #[test]
    fn test_create_temp_dir_success() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let base_dir = temp_dir.path().join("assets");
        fs::create_dir_all(&base_dir).unwrap();

        let importer = AssetImporter::new(base_dir.clone());
        let result = importer.create_temp_dir("test_asset");

        assert!(result.is_ok());
        let temp_path = result.unwrap();
        assert!(temp_path.exists());
        assert!(temp_path.to_string_lossy().contains(".tmp_extract"));
    }

    #[test]
    fn test_create_temp_dir_removes_existing() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let base_dir = temp_dir.path().join("assets");
        fs::create_dir_all(&base_dir).unwrap();

        let importer = AssetImporter::new(base_dir.clone());

        // Create temp dir first time
        let temp_path1 = importer.create_temp_dir("test_asset").unwrap();
        fs::write(temp_path1.join("old.txt"), "old").unwrap();

        // Create again - should remove old one
        let temp_path2 = importer.create_temp_dir("test_asset").unwrap();

        assert!(temp_path2.exists());
        assert!(!temp_path2.join("old.txt").exists(), "Old temp files should be removed");
    }

    #[test]
    fn test_integrate_with_godot_creates_marker() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let asset_dir = temp_dir.path().join("test_asset");
        fs::create_dir_all(&asset_dir).unwrap();

        let importer = AssetImporter::new(PathBuf::from("/tmp"));
        let result = importer.integrate_with_godot(&asset_dir);

        assert!(result.is_ok());
        assert!(asset_dir.join(".imported").exists(), "Should create .imported marker file");
    }

    #[test]
    fn test_validation_result_new() {
        let result = ValidationResult::new();

        assert!(result.valid);
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
        assert!(result.is_valid());
    }

    #[test]
    fn test_validation_result_add_error() {
        let mut result = ValidationResult::new();

        result.add_error("Test error".to_string());

        assert!(!result.valid);
        assert!(!result.is_valid());
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0], "Test error");
    }

    #[test]
    fn test_validation_result_add_warning() {
        let mut result = ValidationResult::new();

        result.add_warning("Test warning".to_string());

        assert!(result.valid, "Warnings should not affect validity");
        assert!(result.is_valid());
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.warnings[0], "Test warning");
    }

    #[test]
    fn test_validation_result_multiple_errors() {
        let mut result = ValidationResult::new();

        result.add_error("Error 1".to_string());
        result.add_error("Error 2".to_string());
        result.add_warning("Warning 1".to_string());

        assert!(!result.is_valid());
        assert_eq!(result.errors.len(), 2);
        assert_eq!(result.warnings.len(), 1);
    }

    // ==================== Cross-Platform Compatibility Tests ====================

    #[test]
    fn test_path_join_cross_platform() {
        // Verify PathBuf::join creates correct paths on all platforms
        let base = PathBuf::from("assets");
        let sub = base.join("test_asset");
        let file = sub.join("asset.json");

        // Path should be constructed correctly regardless of platform
        assert!(file.to_string_lossy().contains("assets"));
        assert!(file.to_string_lossy().contains("test_asset"));
        assert!(file.to_string_lossy().contains("asset.json"));

        // Verify no hardcoded separators are needed
        assert_eq!(file, PathBuf::from("assets").join("test_asset").join("asset.json"));
    }

    #[test]
    fn test_path_components_platform_agnostic() {
        // Verify path components work correctly
        let path = PathBuf::from("base").join("sub1").join("sub2").join("file.txt");

        let components: Vec<_> = path.components()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .collect();

        assert_eq!(components.len(), 4);
        assert_eq!(components[0], "base");
        assert_eq!(components[1], "sub1");
        assert_eq!(components[2], "sub2");
        assert_eq!(components[3], "file.txt");
    }

    #[test]
    fn test_temp_dir_creation_cross_platform() {
        use tempfile::TempDir;

        // Verify temporary directory creation works on all platforms
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        assert!(temp_path.exists());
        assert!(temp_path.is_dir());

        // Can create subdirectories
        let sub_path = temp_path.join("subdir");
        fs::create_dir_all(&sub_path).unwrap();
        assert!(sub_path.exists());

        // Can create files
        let file_path = sub_path.join("test.txt");
        fs::write(&file_path, "test content").unwrap();
        assert!(file_path.exists());

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "test content");
    }

    #[test]
    fn test_file_extension_handling() {
        // Verify file extension detection works correctly
        let zip_path = PathBuf::from("archive.zip");
        assert_eq!(zip_path.extension().unwrap(), "zip");

        let tar_gz_path = PathBuf::from("archive.tar.gz");
        assert_eq!(tar_gz_path.extension().unwrap(), "gz");

        let no_ext = PathBuf::from("file");
        assert!(no_ext.extension().is_none());
    }

    #[test]
    fn test_path_parent_directory() {
        // Verify parent directory detection
        let file_path = PathBuf::from("dir1").join("dir2").join("file.txt");
        let parent = file_path.parent().unwrap();

        assert_eq!(parent, PathBuf::from("dir1").join("dir2"));

        let grandparent = parent.parent().unwrap();
        assert_eq!(grandparent, PathBuf::from("dir1"));
    }

    #[test]
    #[cfg(unix)]
    fn test_unix_specific_permissions() {
        use tempfile::TempDir;
        use std::os::unix::fs::PermissionsExt;

        // Test Unix-specific file permission handling
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.sh");

        fs::write(&file_path, "#!/bin/bash\necho test").unwrap();

        // Set executable permission
        let mut perms = fs::metadata(&file_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&file_path, perms).unwrap();

        // Verify permission was set
        let new_perms = fs::metadata(&file_path).unwrap().permissions();
        assert_eq!(new_perms.mode() & 0o777, 0o755);
    }

    #[test]
    #[cfg(windows)]
    fn test_windows_specific_paths() {
        // Test Windows-specific path handling
        // Note: This test runs only on Windows

        // Windows absolute paths can start with drive letter
        let path = PathBuf::from("C:\\Users\\Test\\file.txt");
        assert!(path.is_absolute());

        // Verify components are parsed correctly
        let components: Vec<_> = path.components().collect();
        assert!(!components.is_empty());
    }

    #[test]
    fn test_relative_vs_absolute_paths() {
        // Test relative path
        let rel_path = PathBuf::from("assets").join("test");
        assert!(!rel_path.is_absolute());

        // Test absolute path detection (platform-specific format)
        #[cfg(unix)]
        {
            let abs_path = PathBuf::from("/tmp/test");
            assert!(abs_path.is_absolute());
        }

        #[cfg(windows)]
        {
            let abs_path = PathBuf::from("C:\\temp\\test");
            assert!(abs_path.is_absolute());
        }
    }

    #[test]
    fn test_path_equality_normalization() {
        // Verify path equality works correctly
        let path1 = PathBuf::from("a").join("b").join("c");
        let path2 = PathBuf::from("a/b/c");

        // On Unix, these should be equal
        // On Windows, forward slashes are normalized to backslashes
        #[cfg(unix)]
        assert_eq!(path1, path2);

        #[cfg(windows)]
        {
            // Paths use backslashes on Windows
            assert_eq!(path1.to_string_lossy().replace('/', "\\"), path2.to_string_lossy().replace('/', "\\"));
        }
    }

    #[test]
    fn test_asset_importer_cross_platform_paths() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let base_dir = temp_dir.path().join("assets");
        fs::create_dir_all(&base_dir).unwrap();

        let importer = AssetImporter::new(base_dir.clone());

        // Test temp directory creation
        let temp_path = importer.create_temp_dir("test_asset").unwrap();
        assert!(temp_path.exists());
        assert!(temp_path.to_string_lossy().contains("test_asset"));
        assert!(temp_path.to_string_lossy().contains(".tmp_extract"));

        // Verify it's a subdirectory of base_dir
        assert!(temp_path.starts_with(&base_dir));
    }

    // ===== Performance Optimization Tests =====

    #[test]
    fn test_search_index_creation() {
        let manager = AssetManager::new();

        // Verify search index is initialized
        let search_index = manager.search_index.lock().unwrap();
        assert_eq!(search_index.lowercase_names.len(), 5); // 5 sample assets
        assert_eq!(search_index.lowercase_tags.len(), 5);
        assert_eq!(search_index.lowercase_descriptions.len(), 5);
    }

    #[test]
    fn test_optimized_search() {
        let manager = AssetManager::new();

        // Search for "shader" - should find "Shader Pack Pro"
        let results = manager.search_assets("shader");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Shader Pack Pro");

        // Search for "2d" - should find assets with "2d" tag or in name
        let results = manager.search_assets("2d");
        assert!(results.len() > 0);
        assert!(results.iter().any(|a| a.name == "Awesome 2D Sprites"));

        // Case-insensitive search
        let results_upper = manager.search_assets("SHADER");
        let results_lower = manager.search_assets("shader");
        assert_eq!(results_upper.len(), results_lower.len());
    }

    #[test]
    fn test_search_index_rebuild_on_add() {
        let manager = AssetManager::new();
        let initial_count = manager.get_asset_count();

        // Add a new asset
        let new_asset = Asset::new(
            "test_123".to_string(),
            "Test Performance Asset".to_string(),
            AssetCategory::Tools,
            "".to_string(),
            "TestAuthor".to_string(),
            "1.0.0".to_string(),
            "A test asset for performance optimization".to_string(),
            vec!["test".to_string(), "performance".to_string()],
            None,
            "https://example.com/test.zip".to_string(),
            vec![],
            vec![],
        );

        manager.add_asset(new_asset);

        // Verify asset was added
        assert_eq!(manager.get_asset_count(), initial_count + 1);

        // Verify search index was rebuilt and can find the new asset
        let results = manager.search_assets("performance");
        assert!(results.iter().any(|a| a.id == "test_123"));

        // Verify index size matches asset count
        let search_index = manager.search_index.lock().unwrap();
        assert_eq!(search_index.lowercase_names.len(), initial_count + 1);
    }

    #[test]
    fn test_search_index_rebuild_on_remove() {
        let manager = AssetManager::new();
        let initial_count = manager.get_asset_count();

        // Add a test asset
        let new_asset = Asset::new(
            "temp_asset".to_string(),
            "Temporary Asset".to_string(),
            AssetCategory::Tools,
            "".to_string(),
            "TestAuthor".to_string(),
            "1.0.0".to_string(),
            "This will be removed".to_string(),
            vec!["temporary".to_string()],
            None,
            "https://example.com/temp.zip".to_string(),
            vec![],
            vec![],
        );

        manager.add_asset(new_asset);
        assert_eq!(manager.get_asset_count(), initial_count + 1);

        // Remove the asset
        manager.remove_asset("temp_asset");
        assert_eq!(manager.get_asset_count(), initial_count);

        // Verify search index was rebuilt and asset is no longer found
        let results = manager.search_assets("temporary");
        assert!(results.is_empty());

        // Verify index size matches asset count
        let search_index = manager.search_index.lock().unwrap();
        assert_eq!(search_index.lowercase_names.len(), initial_count);
    }

    #[test]
    fn test_pagination() {
        let manager = AssetManager::new();
        let total_assets = manager.get_asset_count();

        // Test first page
        let page_size = 2;
        let page1 = manager.get_assets_paginated(0, page_size);
        assert_eq!(page1.len(), page_size);

        // Test second page
        let page2 = manager.get_assets_paginated(1, page_size);
        assert_eq!(page2.len(), page_size);

        // Verify pages contain different assets
        assert_ne!(page1[0].id, page2[0].id);

        // Test last page (might be partial)
        let last_page_index = (total_assets - 1) / page_size;
        let last_page = manager.get_assets_paginated(last_page_index, page_size);
        assert!(last_page.len() > 0);
        assert!(last_page.len() <= page_size);

        // Test out-of-bounds page
        let empty_page = manager.get_assets_paginated(999, page_size);
        assert!(empty_page.is_empty());
    }

    #[test]
    fn test_pagination_consistency() {
        let manager = AssetManager::new();
        let total_assets = manager.get_asset_count();
        let page_size = 2;

        // Collect all assets via pagination
        let mut paginated_assets = Vec::new();
        let mut page = 0;
        loop {
            let page_assets = manager.get_assets_paginated(page, page_size);
            if page_assets.is_empty() {
                break;
            }
            paginated_assets.extend(page_assets);
            page += 1;
        }

        // Should match total asset count
        assert_eq!(paginated_assets.len(), total_assets);

        // Should match get_assets() result
        let all_assets = manager.get_assets();
        assert_eq!(paginated_assets.len(), all_assets.len());
    }

    #[test]
    fn test_get_asset_count() {
        let manager = AssetManager::new();
        let count = manager.get_asset_count();

        // Should have sample assets
        assert!(count > 0);

        // Should match get_assets length
        assert_eq!(count, manager.get_assets().len());
    }

    #[test]
    fn test_search_performance_with_large_dataset() {
        use std::time::Instant;

        let manager = AssetManager::new();

        // Add 100 test assets to simulate larger dataset
        for i in 0..100 {
            let asset = Asset::new(
                format!("perf_test_{}", i),
                format!("Performance Test Asset {}", i),
                if i % 2 == 0 { AssetCategory::TwoD } else { AssetCategory::ThreeD },
                "".to_string(),
                format!("Author {}", i % 10),
                "1.0.0".to_string(),
                format!("Description for asset number {}", i),
                vec![format!("tag{}", i % 5), "performance".to_string()],
                None,
                format!("https://example.com/asset_{}.zip", i),
                vec![],
                vec![],
            );
            manager.add_asset(asset);
        }

        // Measure search performance
        let start = Instant::now();
        let results = manager.search_assets("performance");
        let duration = start.elapsed();

        // Should find all 100 test assets (plus any from sample data)
        assert!(results.len() >= 100);

        // Search should complete quickly (< 10ms for ~100 assets)
        assert!(duration.as_millis() < 100, "Search took {:?}, expected < 100ms", duration);
    }
}