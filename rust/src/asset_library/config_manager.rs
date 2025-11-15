use godot::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Asset source configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssetSource {
    pub name: String,
    pub url: String,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_token: Option<String>,
}

impl AssetSource {
    pub fn new(name: String, url: String) -> Self {
        Self {
            name,
            url,
            enabled: true,
            auth_token: None,
        }
    }

    /// Validates the asset source URL format
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Asset source name cannot be empty".to_string());
        }
        if self.url.trim().is_empty() {
            return Err("Asset source URL cannot be empty".to_string());
        }
        // Basic URL validation
        if !self.url.starts_with("http://") && !self.url.starts_with("https://") {
            return Err("Asset source URL must start with http:// or https://".to_string());
        }
        Ok(())
    }
}

/// User preferences for the asset browser
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserPreferences {
    pub theme: String,
    pub layout: String,
    pub default_filters: Vec<String>,
    pub show_previews: bool,
    pub auto_update_check: bool,
    pub download_path: Option<String>,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            theme: "default".to_string(),
            layout: "grid".to_string(),
            default_filters: vec![],
            show_previews: true,
            auto_update_check: true,
            download_path: None,
        }
    }
}

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: u32,
    pub asset_sources: Vec<AssetSource>,
    pub preferences: UserPreferences,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: 1,
            asset_sources: vec![AssetSource::new(
                "Godot Asset Library".to_string(),
                "https://godotengine.org/asset-library/api".to_string(),
            )],
            preferences: UserPreferences::default(),
        }
    }
}

impl Config {
    /// Migrates configuration from an older version to the current version
    pub fn migrate(mut self, from_version: u32, to_version: u32) -> Result<Self, String> {
        if from_version >= to_version {
            return Ok(self);
        }

        // Perform migrations step by step
        let mut current_version = from_version;

        while current_version < to_version {
            match current_version {
                0 => {
                    // Migration from version 0 to 1
                    // Add any necessary field migrations here
                    current_version = 1;
                }
                _ => {
                    return Err(format!(
                        "Unknown migration path from version {} to {}",
                        current_version, to_version
                    ));
                }
            }
        }

        self.version = to_version;
        Ok(self)
    }
}

#[derive(GodotClass)]
#[class(init)]
pub struct ConfigManager {
    config: Config,
    config_path: PathBuf,
}

impl ConfigManager {
    /// Creates a new ConfigManager instance
    pub fn new() -> Self {
        let config_path = Self::get_config_path();
        Self {
            config: Config::default(),
            config_path,
        }
    }

    /// Gets the configuration file path in Godot's user directory
    fn get_config_path() -> PathBuf {
        // In Godot, this would use OS.get_user_data_dir()
        // For now, we'll use a relative path
        let mut path = PathBuf::from("user://");
        path.push("asset_browser_config.json");
        path
    }

    /// Resolves Godot's user:// protocol to actual filesystem path
    fn resolve_godot_path(path: &PathBuf) -> PathBuf {
        let path_str = path.to_string_lossy();
        if path_str.starts_with("user://") {
            // In a real implementation, this would use Godot's OS singleton
            // For now, we'll use a local directory
            let relative_path = path_str.strip_prefix("user://").unwrap();
            let mut resolved = PathBuf::from(".godot/config");
            resolved.push(relative_path);
            resolved
        } else {
            path.clone()
        }
    }

    /// Saves the current configuration to disk
    pub fn save_config(&self) -> Result<(), String> {
        // Serialize config to JSON
        let json = serde_json::to_string_pretty(&self.config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        // Resolve the path and ensure parent directory exists
        let resolved_path = Self::resolve_godot_path(&self.config_path);
        if let Some(parent) = resolved_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        // Write to file
        fs::write(&resolved_path, json)
            .map_err(|e| format!("Failed to write config file: {}", e))?;

        Ok(())
    }

    /// Loads configuration from disk, with fallback to default
    pub fn load_config(&mut self) -> Result<(), String> {
        let resolved_path = Self::resolve_godot_path(&self.config_path);

        // If file doesn't exist, use default config
        if !resolved_path.exists() {
            self.config = Config::default();
            return Ok(());
        }

        // Read file contents
        let contents = fs::read_to_string(&resolved_path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        // Try to deserialize
        let loaded_config: Config = serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse config file (may be corrupted): {}", e))?;

        // Check if migration is needed
        const CURRENT_VERSION: u32 = 1;
        let loaded_version = loaded_config.version;
        if loaded_version < CURRENT_VERSION {
            self.config = loaded_config
                .migrate(loaded_version, CURRENT_VERSION)
                .map_err(|e| format!("Failed to migrate config: {}", e))?;
            // Save migrated config
            self.save_config()?;
        } else {
            self.config = loaded_config;
        }

        Ok(())
    }

    /// Adds a new asset source to the configuration
    pub fn add_asset_source(&mut self, name: String, url: String) -> Result<(), String> {
        let source = AssetSource::new(name.clone(), url);

        // Validate the source
        source.validate()?;

        // Check if source with same name or URL already exists
        if self.config.asset_sources.iter().any(|s| s.name == name) {
            return Err(format!("Asset source with name '{}' already exists", name));
        }

        if self.config.asset_sources.iter().any(|s| s.url == source.url) {
            return Err(format!("Asset source with URL '{}' already exists", source.url));
        }

        // Add the source
        self.config.asset_sources.push(source);

        Ok(())
    }

    /// Adds an asset source with authentication token
    pub fn add_asset_source_with_auth(
        &mut self,
        name: String,
        url: String,
        auth_token: String,
    ) -> Result<(), String> {
        let mut source = AssetSource::new(name.clone(), url);
        source.auth_token = Some(auth_token);

        // Validate the source
        source.validate()?;

        // Check if source with same name or URL already exists
        if self.config.asset_sources.iter().any(|s| s.name == name) {
            return Err(format!("Asset source with name '{}' already exists", name));
        }

        if self.config.asset_sources.iter().any(|s| s.url == source.url) {
            return Err(format!("Asset source with URL '{}' already exists", source.url));
        }

        // Add the source
        self.config.asset_sources.push(source);

        Ok(())
    }

    /// Removes an asset source by name
    pub fn remove_asset_source(&mut self, name: &str) -> Result<(), String> {
        let initial_len = self.config.asset_sources.len();
        self.config.asset_sources.retain(|s| s.name != name);

        if self.config.asset_sources.len() == initial_len {
            return Err(format!("Asset source '{}' not found", name));
        }

        Ok(())
    }

    /// Gets all configured asset sources
    pub fn get_asset_sources(&self) -> &[AssetSource] {
        &self.config.asset_sources
    }

    /// Gets only enabled asset sources
    pub fn get_enabled_asset_sources(&self) -> Vec<&AssetSource> {
        self.config.asset_sources
            .iter()
            .filter(|s| s.enabled)
            .collect()
    }

    /// Enables or disables an asset source
    pub fn set_asset_source_enabled(&mut self, name: &str, enabled: bool) -> Result<(), String> {
        let source = self.config.asset_sources
            .iter_mut()
            .find(|s| s.name == name)
            .ok_or_else(|| format!("Asset source '{}' not found", name))?;

        source.enabled = enabled;
        Ok(())
    }

    /// Updates user preferences
    pub fn update_preferences(&mut self, preferences: UserPreferences) {
        self.config.preferences = preferences;
    }

    /// Gets current user preferences
    pub fn get_preferences(&self) -> &UserPreferences {
        &self.config.preferences
    }

    /// Sets the theme preference
    pub fn set_theme(&mut self, theme: String) {
        self.config.preferences.theme = theme;
    }

    /// Sets the layout preference
    pub fn set_layout(&mut self, layout: String) {
        self.config.preferences.layout = layout;
    }

    /// Sets the default filters
    pub fn set_default_filters(&mut self, filters: Vec<String>) {
        self.config.preferences.default_filters = filters;
    }

    /// Gets the config version
    pub fn get_version(&self) -> u32 {
        self.config.version
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_source_validation() {
        let valid_source = AssetSource::new(
            "Test".to_string(),
            "https://example.com".to_string(),
        );
        assert!(valid_source.validate().is_ok());

        let invalid_url = AssetSource::new(
            "Test".to_string(),
            "invalid-url".to_string(),
        );
        assert!(invalid_url.validate().is_err());

        let empty_name = AssetSource::new(
            "".to_string(),
            "https://example.com".to_string(),
        );
        assert!(empty_name.validate().is_err());
    }

    #[test]
    fn test_add_asset_source() {
        let mut manager = ConfigManager::new();

        assert!(manager.add_asset_source(
            "Test Source".to_string(),
            "https://test.com".to_string()
        ).is_ok());

        // Should fail to add duplicate
        assert!(manager.add_asset_source(
            "Test Source".to_string(),
            "https://test2.com".to_string()
        ).is_err());
    }

    #[test]
    fn test_remove_asset_source() {
        let mut manager = ConfigManager::new();

        manager.add_asset_source(
            "Test Source".to_string(),
            "https://test.com".to_string()
        ).unwrap();

        assert!(manager.remove_asset_source("Test Source").is_ok());
        assert!(manager.remove_asset_source("Nonexistent").is_err());
    }

    #[test]
    fn test_config_migration() {
        let config = Config {
            version: 0,
            asset_sources: vec![],
            preferences: UserPreferences::default(),
        };

        let migrated = config.migrate(0, 1).unwrap();
        assert_eq!(migrated.version, 1);
    }

    #[test]
    fn test_user_preferences() {
        let mut manager = ConfigManager::new();

        manager.set_theme("dark".to_string());
        assert_eq!(manager.get_preferences().theme, "dark");

        manager.set_layout("list".to_string());
        assert_eq!(manager.get_preferences().layout, "list");

        manager.set_default_filters(vec!["models".to_string(), "textures".to_string()]);
        assert_eq!(manager.get_preferences().default_filters.len(), 2);
    }
}
