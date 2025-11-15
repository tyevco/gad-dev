use super::asset::{Asset, AssetCategory};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use reqwest::Client;

/// Godot Asset Library API base URL
pub const GODOT_ASSET_LIBRARY_API: &str = "https://godotengine.org/asset-library/api";

/// Default cache TTL (time to live) in seconds
const DEFAULT_CACHE_TTL: u64 = 300; // 5 minutes

/// Default rate limit: requests per minute
const DEFAULT_RATE_LIMIT: u32 = 60;

/// Cache entry for API responses
#[derive(Debug, Clone)]
struct CacheEntry {
    /// Cached JSON string
    data: String,
    /// When this entry was created
    created_at: Instant,
    /// Time-to-live in seconds
    ttl: Duration,
}

impl CacheEntry {
    /// Creates a new cache entry with default TTL
    fn new(data: String) -> Self {
        Self {
            data,
            created_at: Instant::now(),
            ttl: Duration::from_secs(DEFAULT_CACHE_TTL),
        }
    }

    /// Creates a new cache entry with custom TTL
    fn with_ttl(data: String, ttl_seconds: u64) -> Self {
        Self {
            data,
            created_at: Instant::now(),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    /// Checks if this cache entry has expired
    fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }
}

/// Rate limiter to prevent API abuse
#[derive(Debug)]
struct RateLimiter {
    /// Request timestamps within the current window
    requests: Vec<Instant>,
    /// Maximum requests per minute
    max_requests: u32,
    /// Time window for rate limiting
    window: Duration,
}

impl RateLimiter {
    /// Creates a new rate limiter
    fn new(max_requests_per_minute: u32) -> Self {
        Self {
            requests: Vec::new(),
            max_requests: max_requests_per_minute,
            window: Duration::from_secs(60),
        }
    }

    /// Checks if a request can be made, and if so, records it
    fn check_and_record(&mut self) -> Result<(), String> {
        let now = Instant::now();

        // Remove old requests outside the window
        self.requests.retain(|&timestamp| now.duration_since(timestamp) < self.window);

        // Check if we've hit the limit
        if self.requests.len() >= self.max_requests as usize {
            return Err(format!(
                "Rate limit exceeded: {} requests per minute",
                self.max_requests
            ));
        }

        // Record this request
        self.requests.push(now);
        Ok(())
    }

    /// Gets the time to wait before the next request can be made
    fn time_until_available(&self) -> Option<Duration> {
        if self.requests.len() < self.max_requests as usize {
            return None;
        }

        // Find the oldest request
        self.requests.first().map(|&oldest| {
            let elapsed = oldest.elapsed();
            if elapsed < self.window {
                self.window - elapsed
            } else {
                Duration::from_secs(0)
            }
        })
    }
}

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
    /// HTTP client for making requests
    client: Client,
    /// Response cache
    cache: Arc<Mutex<HashMap<String, CacheEntry>>>,
    /// Rate limiter
    rate_limiter: Arc<Mutex<RateLimiter>>,
    /// Cache enabled flag
    cache_enabled: bool,
}

impl GodotAssetLibraryClient {
    /// Creates a new API client with the default Godot Asset Library URL
    pub fn new() -> Self {
        Self {
            base_url: GODOT_ASSET_LIBRARY_API.to_string(),
            auth_token: None,
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| Client::new()),
            cache: Arc::new(Mutex::new(HashMap::new())),
            rate_limiter: Arc::new(Mutex::new(RateLimiter::new(DEFAULT_RATE_LIMIT))),
            cache_enabled: true,
        }
    }

    /// Creates a new API client with a custom base URL (for private repositories)
    pub fn with_url(base_url: String) -> Self {
        Self {
            base_url,
            auth_token: None,
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| Client::new()),
            cache: Arc::new(Mutex::new(HashMap::new())),
            rate_limiter: Arc::new(Mutex::new(RateLimiter::new(DEFAULT_RATE_LIMIT))),
            cache_enabled: true,
        }
    }

    /// Sets the authentication token
    pub fn with_auth(mut self, token: String) -> Self {
        self.auth_token = Some(token);
        self
    }

    /// Enables or disables caching
    pub fn with_cache(mut self, enabled: bool) -> Self {
        self.cache_enabled = enabled;
        self
    }

    /// Sets a custom rate limit (requests per minute)
    pub fn with_rate_limit(self, requests_per_minute: u32) -> Self {
        *self.rate_limiter.lock().unwrap() = RateLimiter::new(requests_per_minute);
        self
    }

    /// Clears the response cache
    pub fn clear_cache(&self) {
        self.cache.lock().unwrap().clear();
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
        // Check cache first if enabled
        if self.cache_enabled {
            let cache = self.cache.lock().unwrap();
            if let Some(entry) = cache.get(url) {
                if !entry.is_expired() {
                    // Cache hit - deserialize and return
                    return serde_json::from_str(&entry.data)
                        .map_err(|e| format!("Failed to deserialize cached data: {}", e));
                }
            }
        }

        // Check rate limit
        {
            let mut limiter = self.rate_limiter.lock().unwrap();
            limiter.check_and_record().map_err(|e| {
                if let Some(wait_time) = limiter.time_until_available() {
                    format!("{} - retry in {} seconds", e, wait_time.as_secs())
                } else {
                    e
                }
            })?;
        }

        // Build the HTTP request
        let mut request = self.client.get(url);

        if let Some(ref token) = self.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        // Add user agent
        request = request.header("User-Agent", "GodotAssetBrowser/1.0");

        // Send the request
        let response = request
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        // Check response status
        if !response.status().is_success() {
            return Err(format!("API returned error: {}", response.status()));
        }

        // Get the response text for caching
        let response_text = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response body: {}", e))?;

        // Store in cache if enabled
        if self.cache_enabled {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(url.to_string(), CacheEntry::new(response_text.clone()));
        }

        // Deserialize and return
        serde_json::from_str(&response_text)
            .map_err(|e| format!("Failed to parse JSON: {}", e))
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
