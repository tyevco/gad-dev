use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use godot::prelude::*;
use reqwest::Client;

/// Progress information for downloads
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    /// Bytes downloaded so far
    pub bytes_downloaded: u64,
    /// Total bytes to download (if known)
    pub total_bytes: Option<u64>,
    /// Progress percentage (0-100)
    pub percentage: f32,
}

impl DownloadProgress {
    pub fn new(bytes_downloaded: u64, total_bytes: Option<u64>) -> Self {
        let percentage = if let Some(total) = total_bytes {
            if total > 0 {
                (bytes_downloaded as f32 / total as f32) * 100.0
            } else {
                0.0
            }
        } else {
            0.0
        };

        Self {
            bytes_downloaded,
            total_bytes,
            percentage,
        }
    }
}

/// Type alias for progress callback function
pub type ProgressCallback = Box<dyn Fn(DownloadProgress) + Send + Sync>;

#[derive(GodotClass)]
#[class(init)]
pub struct AssetManager {
    client: Client,
    asset_dir: String,
}

impl AssetManager {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            asset_dir: "res://assets/".to_string(),
        }
    }

    /// Downloads an asset from the given URL to the asset directory
    ///
    /// # Arguments
    /// * `download_url` - The URL to download the asset from
    /// * `filename` - The filename to save as (e.g., "asset_123.zip")
    /// * `progress_callback` - Optional callback for progress updates
    ///
    /// # Returns
    /// The path to the downloaded file
    pub async fn download_asset(
        &self,
        download_url: &str,
        filename: &str,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<PathBuf, String> {
        // Validate URL
        if download_url.is_empty() {
            return Err("Download URL cannot be empty".to_string());
        }

        // Create asset directory if it doesn't exist
        let asset_dir = PathBuf::from(&self.asset_dir);
        if !asset_dir.exists() {
            fs::create_dir_all(&asset_dir)
                .map_err(|e| format!("Failed to create asset directory: {}", e))?;
        }

        // Construct full file path
        let file_path = asset_dir.join(filename);

        // Send HTTP GET request
        let response = self
            .client
            .get(download_url)
            .send()
            .await
            .map_err(|e| format!("Failed to send download request: {}", e))?;

        // Check if request was successful
        if !response.status().is_success() {
            return Err(format!(
                "Download failed with status: {}",
                response.status()
            ));
        }

        // Get content length for progress tracking
        let total_bytes = response.content_length();

        // Create the output file
        let mut file = fs::File::create(&file_path)
            .map_err(|e| format!("Failed to create file: {}", e))?;

        // Download and write in chunks
        let mut bytes_downloaded = 0u64;
        let mut stream = response.bytes_stream();

        // Import the Stream trait to use next()
        use futures_util::StreamExt;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result
                .map_err(|e| format!("Error while downloading chunk: {}", e))?;

            // Write chunk to file
            file.write_all(&chunk)
                .map_err(|e| format!("Failed to write to file: {}", e))?;

            // Update progress
            bytes_downloaded += chunk.len() as u64;

            // Call progress callback if provided
            if let Some(ref callback) = progress_callback {
                let progress = DownloadProgress::new(bytes_downloaded, total_bytes);
                callback(progress);
            }
        }

        // Flush file to ensure all data is written
        file.flush()
            .map_err(|e| format!("Failed to flush file: {}", e))?;

        Ok(file_path)
    }

    /// Downloads an asset by ID (stub - requires asset lookup)
    pub async fn download_asset_by_id(&self, _asset_id: String) -> Result<(), String> {
        // This will be implemented once we have an asset registry/database
        // For now, it's a placeholder
        Err("Asset registry not yet implemented".to_string())
    }

    pub async fn import_asset(&self, _asset_id: String) -> Result<(), String> {
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