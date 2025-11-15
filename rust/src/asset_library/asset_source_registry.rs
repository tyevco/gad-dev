use super::config_manager::AssetSource;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Authentication method for asset sources
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuthMethod {
    /// No authentication required
    None,
    /// Bearer token authentication
    BearerToken(String),
    /// API key in header
    ApiKey { header_name: String, key: String },
    /// Basic authentication
    BasicAuth { username: String, password: String },
}

/// Source health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceHealth {
    /// Source is healthy and responding
    Healthy,
    /// Source is experiencing issues but may still work
    Degraded,
    /// Source is not responding or failing
    Unhealthy,
    /// Source health has not been checked yet
    Unknown,
}

/// Source capabilities and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceCapabilities {
    /// Whether the source supports search functionality
    pub supports_search: bool,
    /// Whether the source supports pagination
    pub supports_pagination: bool,
    /// Whether the source supports filtering by categories
    pub supports_categories: bool,
    /// Whether the source supports asset previews
    pub supports_previews: bool,
    /// Whether the source supports asset ratings
    pub supports_ratings: bool,
    /// Maximum results per page (if pagination supported)
    pub max_page_size: Option<usize>,
    /// API version supported
    pub api_version: Option<String>,
}

impl Default for SourceCapabilities {
    fn default() -> Self {
        Self {
            supports_search: true,
            supports_pagination: true,
            supports_categories: true,
            supports_previews: true,
            supports_ratings: true,
            max_page_size: Some(100),
            api_version: None,
        }
    }
}

/// Extended asset source with health tracking and capabilities
#[derive(Debug, Clone)]
pub struct ManagedAssetSource {
    /// The base asset source configuration
    pub source: AssetSource,
    /// Authentication method
    pub auth_method: AuthMethod,
    /// Source capabilities
    pub capabilities: SourceCapabilities,
    /// Current health status
    pub health: SourceHealth,
    /// Last health check timestamp
    pub last_health_check: Option<SystemTime>,
    /// Number of consecutive failures
    pub failure_count: u32,
    /// Priority (lower number = higher priority)
    pub priority: u32,
}

impl ManagedAssetSource {
    /// Creates a new managed asset source
    pub fn new(source: AssetSource, priority: u32) -> Self {
        let auth_method = if let Some(ref token) = source.auth_token {
            AuthMethod::BearerToken(token.clone())
        } else {
            AuthMethod::None
        };

        Self {
            source,
            auth_method,
            capabilities: SourceCapabilities::default(),
            health: SourceHealth::Unknown,
            last_health_check: None,
            failure_count: 0,
            priority,
        }
    }

    /// Creates a managed source with custom authentication
    pub fn with_auth(mut self, auth_method: AuthMethod) -> Self {
        self.auth_method = auth_method;
        self
    }

    /// Creates a managed source with custom capabilities
    pub fn with_capabilities(mut self, capabilities: SourceCapabilities) -> Self {
        self.capabilities = capabilities;
        self
    }

    /// Records a successful operation
    pub fn record_success(&mut self) {
        self.failure_count = 0;
        self.health = SourceHealth::Healthy;
        self.last_health_check = Some(SystemTime::now());
    }

    /// Records a failed operation
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_health_check = Some(SystemTime::now());

        // Update health based on failure count
        self.health = match self.failure_count {
            1..=2 => SourceHealth::Degraded,
            _ => SourceHealth::Unhealthy,
        };
    }

    /// Checks if the source should be retried based on failure count and health
    pub fn should_retry(&self, max_failures: u32) -> bool {
        self.failure_count < max_failures && self.source.enabled
    }

    /// Checks if health check is stale (older than given duration)
    pub fn is_health_check_stale(&self, max_age: Duration) -> bool {
        match self.last_health_check {
            None => true,
            Some(last_check) => {
                SystemTime::now()
                    .duration_since(last_check)
                    .map(|age| age > max_age)
                    .unwrap_or(true)
            }
        }
    }

    /// Gets the authentication header for HTTP requests
    pub fn get_auth_header(&self) -> Option<(String, String)> {
        match &self.auth_method {
            AuthMethod::None => None,
            AuthMethod::BearerToken(token) => {
                Some(("Authorization".to_string(), format!("Bearer {}", token)))
            }
            AuthMethod::ApiKey { header_name, key } => {
                Some((header_name.clone(), key.clone()))
            }
            AuthMethod::BasicAuth { username, password } => {
                let credentials = base64::encode(format!("{}:{}", username, password));
                Some(("Authorization".to_string(), format!("Basic {}", credentials)))
            }
        }
    }
}

/// Asset source registry with health checking and fallback support
pub struct AssetSourceRegistry {
    /// Managed asset sources
    sources: HashMap<String, ManagedAssetSource>,
    /// Health check interval
    health_check_interval: Duration,
    /// Maximum failures before marking source as unhealthy
    max_failures: u32,
    /// Whether to enable automatic fallback
    enable_fallback: bool,
}

impl AssetSourceRegistry {
    /// Creates a new asset source registry
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
            health_check_interval: Duration::from_secs(300), // 5 minutes
            max_failures: 3,
            enable_fallback: true,
        }
    }

    /// Registers a new asset source
    pub fn register_source(&mut self, source: AssetSource, priority: u32) -> Result<(), String> {
        // Validate the source
        source.validate()?;

        // Check if source already exists
        if self.sources.contains_key(&source.name) {
            return Err(format!("Source '{}' is already registered", source.name));
        }

        let managed_source = ManagedAssetSource::new(source.clone(), priority);
        self.sources.insert(source.name.clone(), managed_source);

        Ok(())
    }

    /// Registers a source with custom authentication
    pub fn register_source_with_auth(
        &mut self,
        source: AssetSource,
        auth_method: AuthMethod,
        priority: u32,
    ) -> Result<(), String> {
        // Validate the source
        source.validate()?;

        // Check if source already exists
        if self.sources.contains_key(&source.name) {
            return Err(format!("Source '{}' is already registered", source.name));
        }

        let managed_source = ManagedAssetSource::new(source.clone(), priority)
            .with_auth(auth_method);
        self.sources.insert(source.name.clone(), managed_source);

        Ok(())
    }

    /// Unregisters an asset source
    pub fn unregister_source(&mut self, name: &str) -> Result<(), String> {
        self.sources
            .remove(name)
            .map(|_| ())
            .ok_or_else(|| format!("Source '{}' not found", name))
    }

    /// Gets a source by name
    pub fn get_source(&self, name: &str) -> Option<&ManagedAssetSource> {
        self.sources.get(name)
    }

    /// Gets a mutable reference to a source
    pub fn get_source_mut(&mut self, name: &str) -> Option<&mut ManagedAssetSource> {
        self.sources.get_mut(name)
    }

    /// Gets all enabled sources sorted by priority
    pub fn get_enabled_sources(&self) -> Vec<&ManagedAssetSource> {
        let mut sources: Vec<&ManagedAssetSource> = self
            .sources
            .values()
            .filter(|s| s.source.enabled)
            .collect();

        // Sort by priority (lower number = higher priority)
        sources.sort_by_key(|s| s.priority);
        sources
    }

    /// Gets healthy sources sorted by priority
    pub fn get_healthy_sources(&self) -> Vec<&ManagedAssetSource> {
        let mut sources: Vec<&ManagedAssetSource> = self
            .sources
            .values()
            .filter(|s| {
                s.source.enabled
                    && (s.health == SourceHealth::Healthy || s.health == SourceHealth::Unknown)
            })
            .collect();

        sources.sort_by_key(|s| s.priority);
        sources
    }

    /// Gets fallback sources when primary source fails
    pub fn get_fallback_sources(&self, failed_source: &str) -> Vec<&ManagedAssetSource> {
        if !self.enable_fallback {
            return Vec::new();
        }

        let mut sources: Vec<&ManagedAssetSource> = self
            .sources
            .values()
            .filter(|s| {
                s.source.name != failed_source
                    && s.source.enabled
                    && s.should_retry(self.max_failures)
            })
            .collect();

        // Sort by health (healthy first) then by priority
        sources.sort_by(|a, b| {
            match (a.health, b.health) {
                (SourceHealth::Healthy, SourceHealth::Healthy) => a.priority.cmp(&b.priority),
                (SourceHealth::Healthy, _) => std::cmp::Ordering::Less,
                (_, SourceHealth::Healthy) => std::cmp::Ordering::Greater,
                (SourceHealth::Degraded, SourceHealth::Degraded) => a.priority.cmp(&b.priority),
                (SourceHealth::Degraded, _) => std::cmp::Ordering::Less,
                (_, SourceHealth::Degraded) => std::cmp::Ordering::Greater,
                _ => a.priority.cmp(&b.priority),
            }
        });

        sources
    }

    /// Marks sources needing health checks
    pub fn get_sources_needing_health_check(&self) -> Vec<&ManagedAssetSource> {
        self.sources
            .values()
            .filter(|s| s.source.enabled && s.is_health_check_stale(self.health_check_interval))
            .collect()
    }

    /// Records a successful operation for a source
    pub fn record_success(&mut self, source_name: &str) {
        if let Some(source) = self.sources.get_mut(source_name) {
            source.record_success();
        }
    }

    /// Records a failed operation for a source
    pub fn record_failure(&mut self, source_name: &str) {
        if let Some(source) = self.sources.get_mut(source_name) {
            source.record_failure();
        }
    }

    /// Updates source capabilities
    pub fn update_capabilities(&mut self, source_name: &str, capabilities: SourceCapabilities) {
        if let Some(source) = self.sources.get_mut(source_name) {
            source.capabilities = capabilities;
        }
    }

    /// Enables or disables a source
    pub fn set_source_enabled(&mut self, source_name: &str, enabled: bool) -> Result<(), String> {
        let source = self
            .sources
            .get_mut(source_name)
            .ok_or_else(|| format!("Source '{}' not found", source_name))?;

        source.source.enabled = enabled;
        Ok(())
    }

    /// Sets the health check interval
    pub fn set_health_check_interval(&mut self, interval: Duration) {
        self.health_check_interval = interval;
    }

    /// Enables or disables automatic fallback
    pub fn set_fallback_enabled(&mut self, enabled: bool) {
        self.enable_fallback = enabled;
    }

    /// Gets the count of registered sources
    pub fn source_count(&self) -> usize {
        self.sources.len()
    }

    /// Gets statistics about source health
    pub fn get_health_stats(&self) -> SourceHealthStats {
        let total = self.sources.len();
        let mut healthy = 0;
        let mut degraded = 0;
        let mut unhealthy = 0;
        let mut unknown = 0;

        for source in self.sources.values() {
            match source.health {
                SourceHealth::Healthy => healthy += 1,
                SourceHealth::Degraded => degraded += 1,
                SourceHealth::Unhealthy => unhealthy += 1,
                SourceHealth::Unknown => unknown += 1,
            }
        }

        SourceHealthStats {
            total,
            healthy,
            degraded,
            unhealthy,
            unknown,
        }
    }
}

impl Default for AssetSourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about source health
#[derive(Debug, Clone)]
pub struct SourceHealthStats {
    pub total: usize,
    pub healthy: usize,
    pub degraded: usize,
    pub unhealthy: usize,
    pub unknown: usize,
}

// Note: base64 encoding function (simplified version)
// In a real implementation, use the base64 crate
mod base64 {
    pub fn encode(input: String) -> String {
        // This is a placeholder - in production use the base64 crate
        // For now, return a simple encoding indicator
        format!("base64({})", input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_source(name: &str, url: &str) -> AssetSource {
        AssetSource::new(name.to_string(), url.to_string())
    }

    #[test]
    fn test_managed_source_creation() {
        let source = create_test_source("Test", "https://test.com");
        let managed = ManagedAssetSource::new(source, 1);

        assert_eq!(managed.health, SourceHealth::Unknown);
        assert_eq!(managed.failure_count, 0);
        assert_eq!(managed.priority, 1);
    }

    #[test]
    fn test_failure_tracking() {
        let source = create_test_source("Test", "https://test.com");
        let mut managed = ManagedAssetSource::new(source, 1);

        // First failure should be degraded
        managed.record_failure();
        assert_eq!(managed.health, SourceHealth::Degraded);
        assert_eq!(managed.failure_count, 1);

        // Multiple failures should be unhealthy
        managed.record_failure();
        managed.record_failure();
        assert_eq!(managed.health, SourceHealth::Unhealthy);
        assert_eq!(managed.failure_count, 3);

        // Success should reset
        managed.record_success();
        assert_eq!(managed.health, SourceHealth::Healthy);
        assert_eq!(managed.failure_count, 0);
    }

    #[test]
    fn test_registry_operations() {
        let mut registry = AssetSourceRegistry::new();

        let source1 = create_test_source("Source1", "https://source1.com");
        let source2 = create_test_source("Source2", "https://source2.com");

        assert!(registry.register_source(source1, 1).is_ok());
        assert!(registry.register_source(source2, 2).is_ok());

        assert_eq!(registry.source_count(), 2);

        // Duplicate should fail
        let source1_dup = create_test_source("Source1", "https://source1.com");
        assert!(registry.register_source(source1_dup, 1).is_err());
    }

    #[test]
    fn test_priority_sorting() {
        let mut registry = AssetSourceRegistry::new();

        let source_low = create_test_source("LowPriority", "https://low.com");
        let source_high = create_test_source("HighPriority", "https://high.com");

        registry.register_source(source_low, 10).unwrap();
        registry.register_source(source_high, 1).unwrap();

        let sources = registry.get_enabled_sources();
        assert_eq!(sources.len(), 2);
        assert_eq!(sources[0].source.name, "HighPriority");
        assert_eq!(sources[1].source.name, "LowPriority");
    }

    #[test]
    fn test_fallback_sources() {
        let mut registry = AssetSourceRegistry::new();

        let source1 = create_test_source("Source1", "https://source1.com");
        let source2 = create_test_source("Source2", "https://source2.com");
        let source3 = create_test_source("Source3", "https://source3.com");

        registry.register_source(source1, 1).unwrap();
        registry.register_source(source2, 2).unwrap();
        registry.register_source(source3, 3).unwrap();

        // Mark source2 as healthy
        registry.record_success("Source2");

        let fallbacks = registry.get_fallback_sources("Source1");
        assert_eq!(fallbacks.len(), 2);
        // Source2 should be first (healthy)
        assert_eq!(fallbacks[0].source.name, "Source2");
    }

    #[test]
    fn test_health_stats() {
        let mut registry = AssetSourceRegistry::new();

        let source1 = create_test_source("Source1", "https://source1.com");
        let source2 = create_test_source("Source2", "https://source2.com");

        registry.register_source(source1, 1).unwrap();
        registry.register_source(source2, 2).unwrap();

        registry.record_success("Source1");
        registry.record_failure("Source2");

        let stats = registry.get_health_stats();
        assert_eq!(stats.total, 2);
        assert_eq!(stats.healthy, 1);
        assert_eq!(stats.degraded, 1);
    }

    #[test]
    fn test_auth_methods() {
        let source = create_test_source("Test", "https://test.com");
        let managed = ManagedAssetSource::new(source, 1)
            .with_auth(AuthMethod::BearerToken("test_token".to_string()));

        let header = managed.get_auth_header();
        assert!(header.is_some());
        let (name, value) = header.unwrap();
        assert_eq!(name, "Authorization");
        assert_eq!(value, "Bearer test_token");
    }
}
