use super::asset::Asset;
use super::godot_asset_library_api::{GodotAssetLibraryClient, AssetListParams};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Repository type identifier
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RepositoryType {
    /// Official Godot Asset Library API
    GodotAssetLibrary,
    /// Custom server using Godot Asset Library compatible API
    GodotCompatible,
    /// Simple JSON file repository
    JsonFile,
    /// Git repository with asset manifest
    GitRepository,
    /// Local filesystem directory
    LocalDirectory,
}

/// Search and filter options for repositories
#[derive(Debug, Clone, Default)]
pub struct RepositorySearchOptions {
    /// Search query string
    pub query: Option<String>,
    /// Category filter
    pub category: Option<String>,
    /// Tags filter
    pub tags: Vec<String>,
    /// Sort by field
    pub sort_by: Option<String>,
    /// Sort ascending or descending
    pub sort_ascending: bool,
    /// Page number for pagination
    pub page: u32,
    /// Items per page
    pub page_size: u32,
}

impl RepositorySearchOptions {
    pub fn new() -> Self {
        Self {
            page_size: 20,
            ..Default::default()
        }
    }

    pub fn with_query(mut self, query: String) -> Self {
        self.query = Some(query);
        self
    }

    pub fn with_category(mut self, category: String) -> Self {
        self.category = Some(category);
        self
    }

    pub fn with_page(mut self, page: u32) -> Self {
        self.page = page;
        self
    }
}

/// Result of a repository search
#[derive(Debug, Clone)]
pub struct RepositorySearchResult {
    /// List of assets found
    pub assets: Vec<Asset>,
    /// Total number of results
    pub total_count: u32,
    /// Current page
    pub page: u32,
    /// Total pages
    pub total_pages: u32,
}

/// Trait for repository adapters
pub trait RepositoryAdapter: Send + Sync {
    /// Gets the repository type
    fn repository_type(&self) -> RepositoryType;

    /// Searches for assets in the repository
    fn search(&self, options: RepositorySearchOptions) -> Result<RepositorySearchResult, String>;

    /// Gets detailed information about a specific asset
    fn get_asset(&self, asset_id: &str) -> Result<Asset, String>;

    /// Lists all available categories
    fn list_categories(&self) -> Result<Vec<String>, String>;

    /// Performs a health check on the repository
    fn health_check(&self) -> Result<bool, String>;

    /// Gets the repository base URL or path
    fn get_location(&self) -> String;
}

/// Adapter for Godot Asset Library (official and compatible)
pub struct GodotAssetLibraryAdapter {
    client: GodotAssetLibraryClient,
    location: String,
}

impl GodotAssetLibraryAdapter {
    /// Creates a new adapter for the official Godot Asset Library
    pub fn new_official() -> Self {
        Self {
            client: GodotAssetLibraryClient::new(),
            location: super::godot_asset_library_api::GODOT_ASSET_LIBRARY_API.to_string(),
        }
    }

    /// Creates a new adapter for a custom Godot-compatible API
    pub fn new_custom(base_url: String) -> Self {
        Self {
            client: GodotAssetLibraryClient::with_url(base_url.clone()),
            location: base_url,
        }
    }

    /// Creates a new adapter with authentication
    pub fn with_auth(mut self, token: String) -> Self {
        self.client = self.client.with_auth(token);
        self
    }
}

impl RepositoryAdapter for GodotAssetLibraryAdapter {
    fn repository_type(&self) -> RepositoryType {
        if self.location == super::godot_asset_library_api::GODOT_ASSET_LIBRARY_API {
            RepositoryType::GodotAssetLibrary
        } else {
            RepositoryType::GodotCompatible
        }
    }

    fn search(&self, options: RepositorySearchOptions) -> Result<RepositorySearchResult, String> {
        // Convert our search options to Godot API params
        let mut params = AssetListParams::new()
            .with_page(options.page)
            .with_max_results(options.page_size);

        if let Some(query) = options.query {
            params = params.with_query(query);
        }

        if let Some(category) = options.category {
            params = params.with_category(category);
        }

        if let Some(sort) = options.sort_by {
            let order = if options.sort_ascending {
                "asc".to_string()
            } else {
                "desc".to_string()
            };
            params = params.with_sort(sort, order);
        }

        // Note: This would be async in a real implementation
        // For now, return a placeholder error
        // params would be used in: self.client.list_assets(params).await
        let _ = params; // Suppress unused warning for placeholder code
        Err("Async operations not implemented in this context".to_string())
    }

    fn get_asset(&self, _asset_id: &str) -> Result<Asset, String> {
        // This would call self.client.get_asset_detail(asset_id).await
        // and convert the result
        Err("Async operations not implemented in this context".to_string())
    }

    fn list_categories(&self) -> Result<Vec<String>, String> {
        // This would call self.client.get_configure().await
        // and extract category names
        Ok(vec![
            "2D Tools".to_string(),
            "3D Tools".to_string(),
            "Shaders".to_string(),
            "Materials".to_string(),
            "Tools".to_string(),
            "Scripts".to_string(),
            "Misc".to_string(),
        ])
    }

    fn health_check(&self) -> Result<bool, String> {
        // In a real implementation, this would be:
        // self.client.health_check().await
        Ok(true)
    }

    fn get_location(&self) -> String {
        self.location.clone()
    }
}

/// JSON file-based repository adapter
pub struct JsonFileAdapter {
    file_path: PathBuf,
    assets: Vec<Asset>,
}

impl JsonFileAdapter {
    /// Creates a new JSON file adapter
    pub fn new(file_path: PathBuf) -> Result<Self, String> {
        // In a real implementation, load and parse the JSON file
        Ok(Self {
            file_path,
            assets: Vec::new(),
        })
    }

    /// Loads assets from the JSON file
    fn load_assets(&mut self) -> Result<(), String> {
        // In a real implementation:
        // let contents = std::fs::read_to_string(&self.file_path)?;
        // self.assets = serde_json::from_str(&contents)?;
        Ok(())
    }
}

impl RepositoryAdapter for JsonFileAdapter {
    fn repository_type(&self) -> RepositoryType {
        RepositoryType::JsonFile
    }

    fn search(&self, options: RepositorySearchOptions) -> Result<RepositorySearchResult, String> {
        let mut filtered_assets = self.assets.clone();

        // Filter by query
        if let Some(ref query) = options.query {
            let query_lower = query.to_lowercase();
            filtered_assets.retain(|asset| {
                asset.name.to_lowercase().contains(&query_lower)
                    || asset.description.to_lowercase().contains(&query_lower)
            });
        }

        // Filter by category
        if let Some(ref category) = options.category {
            filtered_assets.retain(|asset| asset.category.to_string() == *category);
        }

        // Filter by tags
        if !options.tags.is_empty() {
            filtered_assets.retain(|asset| {
                options
                    .tags
                    .iter()
                    .any(|tag| asset.tags.contains(tag))
            });
        }

        // Calculate pagination
        let total_count = filtered_assets.len() as u32;
        let total_pages = (total_count + options.page_size - 1) / options.page_size;
        let start_idx = (options.page * options.page_size) as usize;
        let end_idx = ((options.page + 1) * options.page_size) as usize;

        let paginated_assets = filtered_assets
            .into_iter()
            .skip(start_idx)
            .take(end_idx - start_idx)
            .collect();

        Ok(RepositorySearchResult {
            assets: paginated_assets,
            total_count,
            page: options.page,
            total_pages,
        })
    }

    fn get_asset(&self, asset_id: &str) -> Result<Asset, String> {
        self.assets
            .iter()
            .find(|asset| asset.id == asset_id)
            .cloned()
            .ok_or_else(|| format!("Asset '{}' not found", asset_id))
    }

    fn list_categories(&self) -> Result<Vec<String>, String> {
        let mut categories: Vec<String> = self
            .assets
            .iter()
            .map(|asset| asset.category.to_string())
            .collect();
        categories.sort();
        categories.dedup();
        Ok(categories)
    }

    fn health_check(&self) -> Result<bool, String> {
        Ok(self.file_path.exists())
    }

    fn get_location(&self) -> String {
        self.file_path.to_string_lossy().to_string()
    }
}

/// Local directory adapter for file-based repositories
pub struct LocalDirectoryAdapter {
    directory: PathBuf,
    manifest_file: String,
}

impl LocalDirectoryAdapter {
    /// Creates a new local directory adapter
    pub fn new(directory: PathBuf, manifest_file: String) -> Self {
        Self {
            directory,
            manifest_file,
        }
    }

    /// Scans the directory for asset manifest files
    fn scan_directory(&self) -> Result<Vec<Asset>, String> {
        // In a real implementation:
        // - Walk through the directory
        // - Find manifest files (e.g., asset.json)
        // - Parse each manifest into an Asset
        Ok(Vec::new())
    }
}

impl RepositoryAdapter for LocalDirectoryAdapter {
    fn repository_type(&self) -> RepositoryType {
        RepositoryType::LocalDirectory
    }

    fn search(&self, options: RepositorySearchOptions) -> Result<RepositorySearchResult, String> {
        let assets = self.scan_directory()?;

        // Apply filters similar to JsonFileAdapter
        let total_count = assets.len() as u32;
        let total_pages = (total_count + options.page_size - 1) / options.page_size;

        Ok(RepositorySearchResult {
            assets,
            total_count,
            page: options.page,
            total_pages,
        })
    }

    fn get_asset(&self, asset_id: &str) -> Result<Asset, String> {
        let assets = self.scan_directory()?;
        assets
            .into_iter()
            .find(|asset| asset.id == asset_id)
            .ok_or_else(|| format!("Asset '{}' not found", asset_id))
    }

    fn list_categories(&self) -> Result<Vec<String>, String> {
        let assets = self.scan_directory()?;
        let mut categories: Vec<String> = assets
            .iter()
            .map(|asset| asset.category.to_string())
            .collect();
        categories.sort();
        categories.dedup();
        Ok(categories)
    }

    fn health_check(&self) -> Result<bool, String> {
        Ok(self.directory.exists() && self.directory.is_dir())
    }

    fn get_location(&self) -> String {
        self.directory.to_string_lossy().to_string()
    }
}

/// Repository adapter factory
pub struct RepositoryAdapterFactory;

impl RepositoryAdapterFactory {
    /// Creates a repository adapter based on configuration
    pub fn create_adapter(
        repo_type: RepositoryType,
        location: String,
        auth_token: Option<String>,
    ) -> Result<Box<dyn RepositoryAdapter>, String> {
        match repo_type {
            RepositoryType::GodotAssetLibrary => {
                let mut adapter = GodotAssetLibraryAdapter::new_official();
                if let Some(token) = auth_token {
                    adapter = adapter.with_auth(token);
                }
                Ok(Box::new(adapter))
            }
            RepositoryType::GodotCompatible => {
                let mut adapter = GodotAssetLibraryAdapter::new_custom(location);
                if let Some(token) = auth_token {
                    adapter = adapter.with_auth(token);
                }
                Ok(Box::new(adapter))
            }
            RepositoryType::JsonFile => {
                let path = PathBuf::from(location);
                let adapter = JsonFileAdapter::new(path)?;
                Ok(Box::new(adapter))
            }
            RepositoryType::LocalDirectory => {
                let path = PathBuf::from(location);
                let adapter = LocalDirectoryAdapter::new(path, "asset.json".to_string());
                Ok(Box::new(adapter))
            }
            RepositoryType::GitRepository => {
                Err("Git repository adapter not yet implemented".to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_search_options() {
        let options = RepositorySearchOptions::new()
            .with_query("platformer".to_string())
            .with_category("2D Tools".to_string())
            .with_page(1);

        assert_eq!(options.query, Some("platformer".to_string()));
        assert_eq!(options.category, Some("2D Tools".to_string()));
        assert_eq!(options.page, 1);
        assert_eq!(options.page_size, 20);
    }

    #[test]
    fn test_godot_adapter_creation() {
        let adapter = GodotAssetLibraryAdapter::new_official();
        assert_eq!(adapter.repository_type(), RepositoryType::GodotAssetLibrary);
    }

    #[test]
    fn test_custom_adapter_creation() {
        let custom_url = "https://custom.repo.com/api".to_string();
        let adapter = GodotAssetLibraryAdapter::new_custom(custom_url.clone());
        assert_eq!(adapter.repository_type(), RepositoryType::GodotCompatible);
        assert_eq!(adapter.get_location(), custom_url);
    }

    #[test]
    fn test_adapter_factory() {
        let result = RepositoryAdapterFactory::create_adapter(
            RepositoryType::GodotAssetLibrary,
            String::new(),
            None,
        );
        assert!(result.is_ok());
    }
}
