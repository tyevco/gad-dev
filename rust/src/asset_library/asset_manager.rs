use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use godot::prelude::*;
use reqwest::Client;
use zip::ZipArchive;
use tar::Archive;
use flate2::read::GzDecoder;
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
            godot_print!("Import warnings for {}:", asset_id);
            for warning in &validation.warnings {
                godot_print!("  - {}", warning);
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

        godot_print!("Asset imported to: {:?}", asset_dir);
        godot_print!("Restart Godot editor or reimport to see changes");

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
            godot_print!("Preserving temp directory for debugging: {:?}", temp_dir);
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
            godot_print!("Rolled back import for asset: {}", asset_id);
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