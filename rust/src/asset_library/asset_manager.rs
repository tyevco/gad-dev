use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use godot::prelude::*;
use reqwest::Client;
use crate::asset_library::asset::{Asset, AssetCategory};

#[derive(GodotClass)]
#[class(init)]
pub struct AssetManager {
    client: Client,
    asset_dir: String,
    assets: Arc<Mutex<Vec<Asset>>>,
}

impl AssetManager {
    pub fn new() -> Self {
        let mut manager = Self {
            client: Client::new(),
            asset_dir: "res://assets/".to_string(),
            assets: Arc::new(Mutex::new(Vec::new())),
        };

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
    }

    /// Get all assets
    pub fn get_assets(&self) -> Vec<Asset> {
        self.assets.lock().unwrap().clone()
    }

    /// Get assets by category
    pub fn get_assets_by_category(&self, category: AssetCategory) -> Vec<Asset> {
        self.assets
            .lock()
            .unwrap()
            .iter()
            .filter(|asset| asset.category == category)
            .cloned()
            .collect()
    }

    /// Search assets by name or tags
    pub fn search_assets(&self, query: &str) -> Vec<Asset> {
        let query_lower = query.to_lowercase();
        self.assets
            .lock()
            .unwrap()
            .iter()
            .filter(|asset| {
                asset.name.to_lowercase().contains(&query_lower)
                    || asset.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower))
                    || asset.description.to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect()
    }

    /// Get an asset by ID
    pub fn get_asset_by_id(&self, id: &str) -> Option<Asset> {
        self.assets
            .lock()
            .unwrap()
            .iter()
            .find(|asset| asset.id == id)
            .cloned()
    }

    pub async fn download_asset(&self, asset_id: String) -> Result<(), String> {
        // Implementation commented out for now
        Ok(())
    }

    pub async fn import_asset(&self, asset_id: String) -> Result<(), String> {
        // Implementation commented out for now
        Ok(())
    }
}

pub struct AssetImporter {
    // Add fields for asset importing here
}

impl AssetImporter {
    pub fn new() -> Self {
        Self {
            // Initialize fields here
        }
    }

    pub fn import_asset(&self, asset_path: &Path) -> Result<(), String> {
        // Implement asset importing logic here
        Ok(())
    }
}