use super::asset::{Asset, AssetCategory};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Godot Asset Library API base URL
pub const GODOT_ASSET_LIBRARY_API: &str = "https://godotengine.org/asset-library/api";

/// Asset listing request parameters
#[derive(Debug, Clone, Default)]
pub struct AssetListParams {
    /// Search query string
    pub query: Option<String>,
    /// Category filter
    pub category: Option<String>,
    /// Godot version filter (e.g., "4.0", "3.5")
    pub godot_version: Option<String>,
    /// Support level filter (official, community, testing)
    pub support: Option<String>,
    /// Sort field (rating, cost, name, updated)
    pub sort: Option<String>,
    /// Sort order (asc, desc)
    pub sort_order: Option<String>,
    /// Page number (0-indexed)
    pub page: Option<u32>,
    /// Maximum results per page
    pub max_results: Option<u32>,
}

impl AssetListParams {
    /// Creates a new empty params builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the search query
    pub fn with_query(mut self, query: String) -> Self {
        self.query = Some(query);
        self
    }

    /// Sets the category filter
    pub fn with_category(mut self, category: String) -> Self {
        self.category = Some(category);
        self
    }

    /// Sets the Godot version filter
    pub fn with_godot_version(mut self, version: String) -> Self {
        self.godot_version = Some(version);
        self
    }

    /// Sets the support level filter
    pub fn with_support(mut self, support: String) -> Self {
        self.support = Some(support);
        self
    }

    /// Sets the sort field and order
    pub fn with_sort(mut self, field: String, order: String) -> Self {
        self.sort = Some(field);
        self.sort_order = Some(order);
        self
    }

    /// Sets the page number
    pub fn with_page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    /// Sets the maximum results per page
    pub fn with_max_results(mut self, max_results: u32) -> Self {
        self.max_results = Some(max_results);
        self
    }

    /// Builds the query string for the API request
    pub fn to_query_string(&self) -> String {
        let mut params = Vec::new();

        if let Some(ref query) = self.query {
            params.push(format!("filter={}", urlencoding::encode(query)));
        }
        if let Some(ref category) = self.category {
            params.push(format!("category={}", urlencoding::encode(category)));
        }
        if let Some(ref version) = self.godot_version {
            params.push(format!("godot_version={}", urlencoding::encode(version)));
        }
        if let Some(ref support) = self.support {
            params.push(format!("support={}", urlencoding::encode(support)));
        }
        if let Some(ref sort) = self.sort {
            params.push(format!("sort={}", urlencoding::encode(sort)));
        }
        if let Some(ref order) = self.sort_order {
            params.push(format!("reverse={}", if order == "desc" { "true" } else { "false" }));
        }
        if let Some(page) = self.page {
            params.push(format!("page={}", page));
        }
        if let Some(max) = self.max_results {
            params.push(format!("max_results={}", max));
        }

        if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        }
    }
}

/// Response from the asset listing endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetListResponse {
    /// List of assets
    pub result: Vec<GodotAssetListItem>,
    /// Total number of pages
    pub pages: u32,
    /// Current page number
    pub page: u32,
    /// Total number of results
    pub total_items: u32,
}

/// Asset item in the listing response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GodotAssetListItem {
    /// Asset ID
    pub asset_id: String,
    /// Asset title/name
    pub title: String,
    /// Asset author
    pub author: String,
    /// Author ID
    pub author_id: String,
    /// Category
    pub category: String,
    /// Category ID
    pub category_id: String,
    /// Godot version
    pub godot_version: String,
    /// Rating (0-5)
    pub rating: f32,
    /// Cost (usually "MIT" or similar for free assets)
    pub cost: String,
    /// Support level (official, community, testing)
    pub support_level: String,
    /// Icon URL
    pub icon_url: String,
    /// Version string
    pub version: String,
    /// Version ID
    pub version_id: String,
    /// Modification date
    pub modify_date: String,
}

/// Detailed asset information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GodotAssetDetail {
    /// Asset ID
    pub asset_id: String,
    /// Asset title/name
    pub title: String,
    /// Asset description
    pub description: String,
    /// Asset author
    pub author: String,
    /// Author ID
    pub author_id: String,
    /// Category
    pub category: String,
    /// Category ID
    pub category_id: String,
    /// Godot version
    pub godot_version: String,
    /// Rating (0-5)
    pub rating: f32,
    /// Cost/License
    pub cost: String,
    /// Support level
    pub support_level: String,
    /// Icon URL
    pub icon_url: String,
    /// Version string
    pub version: String,
    /// Version ID
    pub version_id: String,
    /// Download URL
    pub download_url: String,
    /// Download hash
    pub download_hash: String,
    /// Browse URL (repository/website)
    pub browse_url: String,
    /// Issues URL
    pub issues_url: String,
    /// Preview images
    pub previews: Vec<AssetPreview>,
    /// Download count
    pub download_count: u32,
    /// Modification date
    pub modify_date: String,
}

/// Asset preview image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetPreview {
    /// Preview ID
    pub preview_id: String,
    /// Preview type (image or video)
    pub preview_type: String,
    /// Link to the preview
    pub link: String,
    /// Thumbnail URL
    pub thumbnail: String,
}

/// Category information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetCategoryInfo {
    /// Category ID
    pub id: String,
    /// Category name
    pub name: String,
    /// Category type
    pub category_type: String,
}

/// Configure response for getting available options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigureResponse {
    /// Available categories
    pub categories: Vec<AssetCategoryInfo>,
}

/// Godot Asset Library API client
pub struct GodotAssetLibraryClient {
    /// Base API URL
    base_url: String,
    /// Optional authentication token
    auth_token: Option<String>,
}

impl GodotAssetLibraryClient {
    /// Creates a new API client with the default Godot Asset Library URL
    pub fn new() -> Self {
        Self {
            base_url: GODOT_ASSET_LIBRARY_API.to_string(),
            auth_token: None,
        }
    }

    /// Creates a new API client with a custom base URL (for private repositories)
    pub fn with_url(base_url: String) -> Self {
        Self {
            base_url,
            auth_token: None,
        }
    }

    /// Sets the authentication token
    pub fn with_auth(mut self, token: String) -> Self {
        self.auth_token = Some(token);
        self
    }

    /// Fetches a list of assets with optional filters
    pub async fn list_assets(&self, params: AssetListParams) -> Result<AssetListResponse, String> {
        let url = format!("{}/asset{}", self.base_url, params.to_query_string());
        self.fetch_json(&url).await
    }

    /// Fetches detailed information about a specific asset
    pub async fn get_asset_detail(&self, asset_id: &str) -> Result<GodotAssetDetail, String> {
        let url = format!("{}/asset/{}", self.base_url, asset_id);
        self.fetch_json(&url).await
    }

    /// Fetches the configuration (categories, etc.)
    pub async fn get_configure(&self) -> Result<ConfigureResponse, String> {
        let url = format!("{}/configure", self.base_url);
        self.fetch_json(&url).await
    }

    /// Searches for assets by query string
    pub async fn search(&self, query: &str, page: u32) -> Result<AssetListResponse, String> {
        let params = AssetListParams::new()
            .with_query(query.to_string())
            .with_page(page);
        self.list_assets(params).await
    }

    /// Gets assets by category
    pub async fn get_by_category(
        &self,
        category: &str,
        page: u32,
    ) -> Result<AssetListResponse, String> {
        let params = AssetListParams::new()
            .with_category(category.to_string())
            .with_page(page);
        self.list_assets(params).await
    }

    /// Converts a Godot API asset to our internal Asset structure
    pub fn convert_to_asset(&self, godot_asset: &GodotAssetDetail) -> Asset {
        let category = self.map_category(&godot_asset.category);

        Asset {
            id: godot_asset.asset_id.clone(),
            name: godot_asset.title.clone(),
            description: godot_asset.description.clone(),
            author: godot_asset.author.clone(),
            version: godot_asset.version.clone(),
            category,
            path: String::new(), // Will be set when downloaded
            tags: vec![
                godot_asset.category.clone(),
                godot_asset.godot_version.clone(),
                godot_asset.support_level.clone(),
            ],
            preview_url: Some(godot_asset.icon_url.clone()),
            download_url: godot_asset.download_url.clone(),
            dependencies: Vec::new(),
            version_history: Vec::new(),
        }
    }

    /// Maps Godot Asset Library category to our internal category
    fn map_category(&self, category_name: &str) -> AssetCategory {
        match category_name.to_lowercase().as_str() {
            "2d tools" | "2d" => AssetCategory::TwoD,
            "3d tools" | "3d" => AssetCategory::ThreeD,
            "shaders" => AssetCategory::Shaders,
            "materials" => AssetCategory::Materials,
            "audio" => AssetCategory::Audio,
            "tools" => AssetCategory::Tools,
            "scripts" => AssetCategory::Scripts,
            "templates" | "projects" => AssetCategory::Templates,
            "demos" => AssetCategory::Demos,
            _ => AssetCategory::Misc,
        }
    }

    /// Internal method to fetch JSON from the API
    async fn fetch_json<T: for<'de> Deserialize<'de>>(&self, url: &str) -> Result<T, String> {
        // This is a placeholder for actual HTTP request implementation
        // In a real implementation, you would use reqwest or similar:
        //
        // let client = reqwest::Client::new();
        // let mut request = client.get(url);
        //
        // if let Some(ref token) = self.auth_token {
        //     request = request.header("Authorization", format!("Bearer {}", token));
        // }
        //
        // let response = request.send().await
        //     .map_err(|e| format!("HTTP request failed: {}", e))?;
        //
        // if !response.status().is_success() {
        //     return Err(format!("API returned error: {}", response.status()));
        // }
        //
        // response.json::<T>().await
        //     .map_err(|e| format!("Failed to parse JSON: {}", e))

        Err(format!(
            "HTTP client not implemented yet. Would fetch: {}",
            url
        ))
    }

    /// Checks if the API is accessible (health check)
    pub async fn health_check(&self) -> Result<bool, String> {
        // Try to fetch the configure endpoint as a health check
        match self.get_configure().await {
            Ok(_) => Ok(true),
            Err(e) => Err(format!("Health check failed: {}", e)),
        }
    }
}

impl Default for GodotAssetLibraryClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple URL encoding module (placeholder)
/// In production, use the `urlencoding` crate
mod urlencoding {
    pub fn encode(s: &str) -> String {
        // Simplified encoding - in production use proper URL encoding
        s.replace(' ', "%20")
            .replace('&', "%26")
            .replace('=', "%3D")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_list_params_query_string() {
        let params = AssetListParams::new()
            .with_query("platformer".to_string())
            .with_category("2d".to_string())
            .with_page(1)
            .with_max_results(20);

        let query = params.to_query_string();
        assert!(query.contains("filter=platformer"));
        assert!(query.contains("category=2d"));
        assert!(query.contains("page=1"));
        assert!(query.contains("max_results=20"));
    }

    #[test]
    fn test_empty_params_query_string() {
        let params = AssetListParams::new();
        let query = params.to_query_string();
        assert_eq!(query, "");
    }

    #[test]
    fn test_client_creation() {
        let client = GodotAssetLibraryClient::new();
        assert_eq!(client.base_url, GODOT_ASSET_LIBRARY_API);
        assert!(client.auth_token.is_none());
    }

    #[test]
    fn test_client_with_auth() {
        let client = GodotAssetLibraryClient::new()
            .with_auth("test_token".to_string());
        assert_eq!(client.auth_token, Some("test_token".to_string()));
    }

    #[test]
    fn test_custom_url() {
        let custom_url = "https://custom.repo.com/api";
        let client = GodotAssetLibraryClient::with_url(custom_url.to_string());
        assert_eq!(client.base_url, custom_url);
    }

    #[test]
    fn test_category_mapping() {
        let client = GodotAssetLibraryClient::new();

        assert_eq!(client.map_category("2D Tools"), AssetCategory::TwoD);
        assert_eq!(client.map_category("Shaders"), AssetCategory::Shaders);
        assert_eq!(client.map_category("Scripts"), AssetCategory::Scripts);
        assert_eq!(client.map_category("Unknown"), AssetCategory::Misc);
    }
}
