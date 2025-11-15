use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use reqwest::Client;
use tokio::sync::Semaphore;
use tokio::task::JoinHandle;

/// Status of a download
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadStatus {
    /// Download is queued but not started
    Queued,
    /// Download is actively in progress
    InProgress,
    /// Download is paused
    Paused,
    /// Download completed successfully
    Completed,
    /// Download failed
    Failed,
    /// Download was cancelled
    Cancelled,
}

/// Information about a download
#[derive(Debug, Clone)]
pub struct DownloadInfo {
    /// Unique ID for this download
    pub id: String,
    /// URL to download from
    pub url: String,
    /// Destination file path
    pub destination: PathBuf,
    /// Current status
    pub status: DownloadStatus,
    /// Total size in bytes (if known)
    pub total_size: Option<u64>,
    /// Downloaded bytes so far
    pub downloaded_bytes: u64,
    /// Download speed in bytes per second
    pub speed_bps: f64,
    /// Estimated time remaining in seconds
    pub eta_seconds: Option<u64>,
    /// Number of retry attempts
    pub retry_count: u32,
    /// Error message (if failed)
    pub error: Option<String>,
    /// When the download started
    pub started_at: Option<Instant>,
    /// When the download was last updated
    pub updated_at: Instant,
}

impl DownloadInfo {
    /// Creates a new download info
    pub fn new(id: String, url: String, destination: PathBuf) -> Self {
        Self {
            id,
            url,
            destination,
            status: DownloadStatus::Queued,
            total_size: None,
            downloaded_bytes: 0,
            speed_bps: 0.0,
            eta_seconds: None,
            retry_count: 0,
            error: None,
            started_at: None,
            updated_at: Instant::now(),
        }
    }

    /// Calculates progress percentage (0-100)
    pub fn progress_percent(&self) -> f32 {
        if let Some(total) = self.total_size {
            if total > 0 {
                return (self.downloaded_bytes as f64 / total as f64 * 100.0) as f32;
            }
        }
        0.0
    }

    /// Updates download progress
    pub fn update_progress(&mut self, downloaded: u64, total: Option<u64>) {
        self.downloaded_bytes = downloaded;
        self.total_size = total;

        // Calculate speed and ETA
        if let Some(start_time) = self.started_at {
            let elapsed = start_time.elapsed().as_secs_f64();
            if elapsed > 0.0 {
                self.speed_bps = self.downloaded_bytes as f64 / elapsed;

                if let Some(total_bytes) = self.total_size {
                    let remaining = total_bytes.saturating_sub(self.downloaded_bytes);
                    if self.speed_bps > 0.0 {
                        self.eta_seconds = Some((remaining as f64 / self.speed_bps) as u64);
                    }
                }
            }
        }

        self.updated_at = Instant::now();
    }
}

/// Configuration for the download manager
#[derive(Debug, Clone)]
pub struct DownloadManagerConfig {
    /// Maximum concurrent downloads
    pub max_concurrent: usize,
    /// Maximum retry attempts for failed downloads
    pub max_retries: u32,
    /// Retry delay in seconds
    pub retry_delay_seconds: u64,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Download chunk size for progress updates
    pub chunk_size: usize,
    /// Bandwidth throttle in bytes per second (None for unlimited)
    pub bandwidth_limit_bps: Option<u64>,
}

impl Default for DownloadManagerConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 3,
            max_retries: 3,
            retry_delay_seconds: 5,
            timeout_seconds: 300,
            chunk_size: 8192,
            bandwidth_limit_bps: None,
        }
    }
}

/// Download manager that handles concurrent downloads with queue management
pub struct DownloadManager {
    /// Configuration
    config: DownloadManagerConfig,
    /// HTTP client
    client: Client,
    /// Active and queued downloads
    downloads: Arc<Mutex<HashMap<String, DownloadInfo>>>,
    /// Semaphore for limiting concurrent downloads
    concurrent_limit: Arc<Semaphore>,
    /// Active download tasks
    tasks: Arc<Mutex<HashMap<String, JoinHandle<()>>>>,
}

impl DownloadManager {
    /// Creates a new download manager with default configuration
    pub fn new() -> Self {
        Self::with_config(DownloadManagerConfig::default())
    }

    /// Creates a new download manager with custom configuration
    pub fn with_config(config: DownloadManagerConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .unwrap_or_else(|_| Client::new());

        let concurrent_limit = Arc::new(Semaphore::new(config.max_concurrent));

        Self {
            config,
            client,
            downloads: Arc::new(Mutex::new(HashMap::new())),
            concurrent_limit,
            tasks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Adds a download to the queue.
    ///
    /// Creates a new download entry in the queued state. The download must be
    /// explicitly started using `start_download()`.
    ///
    /// # Arguments
    /// * `id` - Unique identifier for this download
    /// * `url` - URL to download from
    /// * `destination` - Path where the file should be saved
    ///
    /// # Returns
    /// * `Ok(())` - Download was successfully queued
    /// * `Err(String)` - Error if a download with the same ID already exists
    ///
    /// # Example
    /// ```
    /// let manager = DownloadManager::new();
    /// manager.queue_download(
    ///     "asset_123".to_string(),
    ///     "https://example.com/asset.zip".to_string(),
    ///     PathBuf::from("/path/to/save/asset.zip")
    /// )?;
    /// ```
    pub fn queue_download(&self, id: String, url: String, destination: PathBuf) -> Result<(), String> {
        let mut downloads = self.downloads.lock().unwrap();

        // Check if download already exists
        if downloads.contains_key(&id) {
            return Err(format!("Download '{}' already exists", id));
        }

        // Create download info
        let info = DownloadInfo::new(id.clone(), url, destination);
        downloads.insert(id, info);

        Ok(())
    }

    /// Starts a queued or paused download.
    ///
    /// Begins downloading the file from the URL to the destination path.
    /// The download runs asynchronously in the background. Use `get_download_info()`
    /// to check progress.
    ///
    /// Downloads are subject to the concurrent download limit configured in
    /// `DownloadManagerConfig::max_concurrent`.
    ///
    /// # Arguments
    /// * `id` - Unique identifier of the download to start
    ///
    /// # Returns
    /// * `Ok(())` - Download was successfully started
    /// * `Err(String)` - Error if download not found or not in queued/paused state
    ///
    /// # Example
    /// ```
    /// let manager = DownloadManager::new();
    /// manager.queue_download(
    ///     "asset_123".to_string(),
    ///     "https://example.com/asset.zip".to_string(),
    ///     PathBuf::from("/path/to/save/asset.zip")
    /// )?;
    /// manager.start_download("asset_123".to_string()).await?;
    /// ```
    pub async fn start_download(&self, id: String) -> Result<(), String> {
        // Get download info
        let (url, destination) = {
            let mut downloads = self.downloads.lock().unwrap();
            let info = downloads.get_mut(&id)
                .ok_or_else(|| format!("Download '{}' not found", id))?;

            if info.status != DownloadStatus::Queued && info.status != DownloadStatus::Paused {
                return Err(format!("Download '{}' is not queued or paused", id));
            }

            info.status = DownloadStatus::InProgress;
            info.started_at = Some(Instant::now());
            (info.url.clone(), info.destination.clone())
        };

        // Spawn download task
        let client = self.client.clone();
        let downloads = self.downloads.clone();
        let concurrent_limit = self.concurrent_limit.clone();
        let config = self.config.clone();
        let id_clone = id.clone();

        let task = tokio::spawn(async move {
            // Acquire semaphore permit
            let _permit = concurrent_limit.acquire().await.unwrap();

            // Perform the download with retries
            let result = Self::download_with_retries(
                &client,
                &url,
                &destination,
                &id_clone,
                &downloads,
                &config,
            ).await;

            // Update status based on result
            {
                let mut downloads = downloads.lock().unwrap();
                if let Some(info) = downloads.get_mut(&id_clone) {
                    match result {
                        Ok(_) => {
                            info.status = DownloadStatus::Completed;
                        }
                        Err(e) => {
                            info.status = DownloadStatus::Failed;
                            info.error = Some(e);
                        }
                    }
                }
            }
        });

        // Store task handle
        self.tasks.lock().unwrap().insert(id, task);

        Ok(())
    }

    /// Downloads a file with retry logic
    async fn download_with_retries(
        client: &Client,
        url: &str,
        destination: &PathBuf,
        id: &str,
        downloads: &Arc<Mutex<HashMap<String, DownloadInfo>>>,
        config: &DownloadManagerConfig,
    ) -> Result<(), String> {
        let mut attempts = 0;

        loop {
            match Self::download_file(client, url, destination, id, downloads, config).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    attempts += 1;

                    // Update retry count
                    {
                        let mut downloads = downloads.lock().unwrap();
                        if let Some(info) = downloads.get_mut(id) {
                            info.retry_count = attempts;
                        }
                    }

                    if attempts >= config.max_retries {
                        return Err(format!("Download failed after {} attempts: {}", attempts, e));
                    }

                    // Wait before retrying
                    tokio::time::sleep(Duration::from_secs(config.retry_delay_seconds)).await;
                }
            }
        }
    }

    /// Downloads a single file
    async fn download_file(
        client: &Client,
        url: &str,
        destination: &PathBuf,
        id: &str,
        downloads: &Arc<Mutex<HashMap<String, DownloadInfo>>>,
        config: &DownloadManagerConfig,
    ) -> Result<(), String> {
        use tokio::io::AsyncWriteExt;

        // Make the request
        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()));
        }

        // Get total size
        let total_size = response.content_length();

        // Create parent directories
        if let Some(parent) = destination.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("Failed to create directories: {}", e))?;
        }

        // Create output file
        let mut file = tokio::fs::File::create(destination)
            .await
            .map_err(|e| format!("Failed to create file: {}", e))?;

        // Download in chunks
        let mut downloaded = 0u64;
        let mut stream = response.bytes_stream();

        use futures_util::StreamExt;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| format!("Failed to read chunk: {}", e))?;

            file.write_all(&chunk)
                .await
                .map_err(|e| format!("Failed to write to file: {}", e))?;

            downloaded += chunk.len() as u64;

            // Update progress
            {
                let mut downloads = downloads.lock().unwrap();
                if let Some(info) = downloads.get_mut(id) {
                    // Check if download was cancelled or paused
                    if info.status == DownloadStatus::Cancelled {
                        return Err("Download cancelled".to_string());
                    }
                    if info.status == DownloadStatus::Paused {
                        return Err("Download paused".to_string());
                    }

                    info.update_progress(downloaded, total_size);
                }
            }

            // Apply bandwidth throttling if configured
            if let Some(limit_bps) = config.bandwidth_limit_bps {
                let chunk_duration = (chunk.len() as f64 / limit_bps as f64) * 1000.0;
                tokio::time::sleep(Duration::from_millis(chunk_duration as u64)).await;
            }
        }

        file.flush()
            .await
            .map_err(|e| format!("Failed to flush file: {}", e))?;

        Ok(())
    }

    /// Pauses an active download.
    ///
    /// The download can be resumed later using `start_download()`. Downloaded
    /// progress is preserved.
    ///
    /// # Arguments
    /// * `id` - Unique identifier of the download to pause
    ///
    /// # Returns
    /// * `Ok(())` - Download was successfully paused
    /// * `Err(String)` - Error if download not found or not in progress
    ///
    /// # Example
    /// ```
    /// let manager = DownloadManager::new();
    /// manager.pause_download("asset_123")?;
    /// // Later...
    /// manager.start_download("asset_123".to_string()).await?;
    /// ```
    pub fn pause_download(&self, id: &str) -> Result<(), String> {
        let mut downloads = self.downloads.lock().unwrap();
        let info = downloads.get_mut(id)
            .ok_or_else(|| format!("Download '{}' not found", id))?;

        if info.status != DownloadStatus::InProgress {
            return Err(format!("Download '{}' is not in progress", id));
        }

        info.status = DownloadStatus::Paused;
        Ok(())
    }

    /// Cancels a download and aborts the download task.
    ///
    /// The download is stopped and cannot be resumed. To start the download
    /// again, you must queue it again.
    ///
    /// # Arguments
    /// * `id` - Unique identifier of the download to cancel
    ///
    /// # Returns
    /// * `Ok(())` - Download was successfully cancelled
    /// * `Err(String)` - Error if download not found
    ///
    /// # Example
    /// ```
    /// let manager = DownloadManager::new();
    /// manager.cancel_download("asset_123")?;
    /// ```
    pub fn cancel_download(&self, id: &str) -> Result<(), String> {
        let mut downloads = self.downloads.lock().unwrap();
        let info = downloads.get_mut(id)
            .ok_or_else(|| format!("Download '{}' not found", id))?;

        info.status = DownloadStatus::Cancelled;

        // Remove task if exists
        if let Some(task) = self.tasks.lock().unwrap().remove(id) {
            task.abort();
        }

        Ok(())
    }

    /// Retrieves information about a specific download.
    ///
    /// Returns a snapshot of the download's current state including progress,
    /// status, speed, and ETA.
    ///
    /// # Arguments
    /// * `id` - Unique identifier of the download
    ///
    /// # Returns
    /// * `Some(DownloadInfo)` - Download information if found
    /// * `None` - If no download with the given ID exists
    ///
    /// # Example
    /// ```
    /// let manager = DownloadManager::new();
    /// if let Some(info) = manager.get_download_info("asset_123") {
    ///     println!("Progress: {:.1}%", info.progress_percent());
    ///     println!("Speed: {} bytes/sec", info.speed_bps);
    /// }
    /// ```
    pub fn get_download_info(&self, id: &str) -> Option<DownloadInfo> {
        self.downloads.lock().unwrap().get(id).cloned()
    }

    /// Retrieves information about all downloads.
    ///
    /// Returns a vector containing information about all downloads, regardless
    /// of their status.
    ///
    /// # Returns
    /// * `Vec<DownloadInfo>` - Vector of all download information
    ///
    /// # Example
    /// ```
    /// let manager = DownloadManager::new();
    /// let all = manager.get_all_downloads();
    /// println!("Total downloads: {}", all.len());
    /// ```
    pub fn get_all_downloads(&self) -> Vec<DownloadInfo> {
        self.downloads.lock().unwrap().values().cloned().collect()
    }

    /// Retrieves information about currently active (in-progress) downloads.
    ///
    /// # Returns
    /// * `Vec<DownloadInfo>` - Vector of active download information
    ///
    /// # Example
    /// ```
    /// let manager = DownloadManager::new();
    /// let active = manager.get_active_downloads();
    /// println!("{} downloads in progress", active.len());
    /// ```
    pub fn get_active_downloads(&self) -> Vec<DownloadInfo> {
        self.downloads
            .lock()
            .unwrap()
            .values()
            .filter(|info| info.status == DownloadStatus::InProgress)
            .cloned()
            .collect()
    }

    /// Retrieves information about queued downloads.
    ///
    /// # Returns
    /// * `Vec<DownloadInfo>` - Vector of queued download information
    ///
    /// # Example
    /// ```
    /// let manager = DownloadManager::new();
    /// let queued = manager.get_queued_downloads();
    /// println!("{} downloads waiting", queued.len());
    /// ```
    pub fn get_queued_downloads(&self) -> Vec<DownloadInfo> {
        self.downloads
            .lock()
            .unwrap()
            .values()
            .filter(|info| info.status == DownloadStatus::Queued)
            .cloned()
            .collect()
    }

    /// Removes a completed or failed download from the tracking list.
    ///
    /// This cleans up completed or failed downloads to free memory. Active
    /// downloads cannot be removed and must be cancelled first.
    ///
    /// # Arguments
    /// * `id` - Unique identifier of the download to remove
    ///
    /// # Returns
    /// * `Ok(())` - Download was successfully removed
    /// * `Err(String)` - Error if download not found or still active
    ///
    /// # Example
    /// ```
    /// let manager = DownloadManager::new();
    /// // After download completes
    /// manager.remove_download("asset_123")?;
    /// ```
    pub fn remove_download(&self, id: &str) -> Result<(), String> {
        let mut downloads = self.downloads.lock().unwrap();
        let info = downloads.get(id)
            .ok_or_else(|| format!("Download '{}' not found", id))?;

        if info.status == DownloadStatus::InProgress {
            return Err(format!("Cannot remove active download '{}'", id));
        }

        downloads.remove(id);
        Ok(())
    }

    /// Clears all completed and failed downloads
    pub fn clear_completed(&self) {
        let mut downloads = self.downloads.lock().unwrap();
        downloads.retain(|_, info| {
            info.status != DownloadStatus::Completed && info.status != DownloadStatus::Failed
        });
    }
}

impl Default for DownloadManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_download_info_new() {
        let info = DownloadInfo::new(
            "test_1".to_string(),
            "https://example.com/file.zip".to_string(),
            PathBuf::from("/tmp/file.zip"),
        );

        assert_eq!(info.id, "test_1");
        assert_eq!(info.url, "https://example.com/file.zip");
        assert_eq!(info.status, DownloadStatus::Queued);
        assert_eq!(info.downloaded_bytes, 0);
        assert_eq!(info.retry_count, 0);
        assert!(info.error.is_none());
        assert!(info.started_at.is_none());
    }

    #[test]
    fn test_download_info_progress_percent() {
        let mut info = DownloadInfo::new(
            "test".to_string(),
            "https://example.com/file.zip".to_string(),
            PathBuf::from("/tmp/file.zip"),
        );

        // No total size - should return 0%
        assert_eq!(info.progress_percent(), 0.0);

        // With total size
        info.update_progress(500, Some(1000));
        assert_eq!(info.progress_percent(), 50.0);

        info.update_progress(1000, Some(1000));
        assert_eq!(info.progress_percent(), 100.0);

        info.update_progress(250, Some(1000));
        assert_eq!(info.progress_percent(), 25.0);
    }

    #[test]
    fn test_download_info_update_progress() {
        let mut info = DownloadInfo::new(
            "test".to_string(),
            "https://example.com/file.zip".to_string(),
            PathBuf::from("/tmp/file.zip"),
        );

        info.update_progress(1024, Some(2048));
        assert_eq!(info.downloaded_bytes, 1024);
        assert_eq!(info.total_size, Some(2048));
    }

    #[test]
    fn test_download_info_speed_calculation() {
        let mut info = DownloadInfo::new(
            "test".to_string(),
            "https://example.com/file.zip".to_string(),
            PathBuf::from("/tmp/file.zip"),
        );

        // Set start time
        info.started_at = Some(Instant::now() - Duration::from_secs(1));

        // Update with 1000 bytes after 1 second
        info.update_progress(1000, Some(2000));

        // Speed should be approximately 1000 bytes/sec
        assert!(info.speed_bps > 900.0 && info.speed_bps < 1100.0);

        // ETA should be approximately 1 second (for remaining 1000 bytes)
        if let Some(eta) = info.eta_seconds {
            assert!(eta <= 2); // Allow some margin
        }
    }

    #[test]
    fn test_download_manager_new() {
        let manager = DownloadManager::new();
        assert!(manager.get_all_downloads().is_empty());
    }

    #[test]
    fn test_download_manager_with_config() {
        let config = DownloadManagerConfig {
            max_concurrent: 2,
            max_retries: 5,
            retry_delay_seconds: 10,
            timeout_seconds: 300,
            chunk_size: 16384,
            bandwidth_limit_bps: Some(1024 * 1024),
        };

        let manager = DownloadManager::with_config(config);
        assert!(manager.get_all_downloads().is_empty());
    }

    #[test]
    fn test_queue_download() {
        let manager = DownloadManager::new();

        let result = manager.queue_download(
            "test_1".to_string(),
            "https://example.com/file.zip".to_string(),
            PathBuf::from("/tmp/file.zip"),
        );

        assert!(result.is_ok());

        let downloads = manager.get_all_downloads();
        assert_eq!(downloads.len(), 1);
        assert_eq!(downloads[0].id, "test_1");
        assert_eq!(downloads[0].status, DownloadStatus::Queued);
    }

    #[test]
    fn test_queue_duplicate_download() {
        let manager = DownloadManager::new();

        manager.queue_download(
            "test_1".to_string(),
            "https://example.com/file.zip".to_string(),
            PathBuf::from("/tmp/file.zip"),
        ).unwrap();

        // Try to queue same ID again
        let result = manager.queue_download(
            "test_1".to_string(),
            "https://example.com/file2.zip".to_string(),
            PathBuf::from("/tmp/file2.zip"),
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already exists"));
    }

    #[test]
    fn test_queue_multiple_downloads() {
        let manager = DownloadManager::new();

        for i in 0..5 {
            let result = manager.queue_download(
                format!("test_{}", i),
                format!("https://example.com/file{}.zip", i),
                PathBuf::from(format!("/tmp/file{}.zip", i)),
            );
            assert!(result.is_ok());
        }

        assert_eq!(manager.get_all_downloads().len(), 5);
        assert_eq!(manager.get_queued_downloads().len(), 5);
    }

    #[test]
    fn test_get_download_info() {
        let manager = DownloadManager::new();

        manager.queue_download(
            "test_1".to_string(),
            "https://example.com/file.zip".to_string(),
            PathBuf::from("/tmp/file.zip"),
        ).unwrap();

        let info = manager.get_download_info("test_1");
        assert!(info.is_some());
        assert_eq!(info.unwrap().id, "test_1");

        let not_found = manager.get_download_info("nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_get_queued_downloads() {
        let manager = DownloadManager::new();

        manager.queue_download(
            "test_1".to_string(),
            "https://example.com/file1.zip".to_string(),
            PathBuf::from("/tmp/file1.zip"),
        ).unwrap();

        manager.queue_download(
            "test_2".to_string(),
            "https://example.com/file2.zip".to_string(),
            PathBuf::from("/tmp/file2.zip"),
        ).unwrap();

        let queued = manager.get_queued_downloads();
        assert_eq!(queued.len(), 2);
        assert!(queued.iter().all(|d| d.status == DownloadStatus::Queued));
    }

    #[test]
    fn test_pause_download() {
        let manager = DownloadManager::new();

        manager.queue_download(
            "test_1".to_string(),
            "https://example.com/file.zip".to_string(),
            PathBuf::from("/tmp/file.zip"),
        ).unwrap();

        // Cannot pause queued download (must be in progress)
        let result = manager.pause_download("test_1");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not in progress"));

        // Manually set to InProgress for testing pause
        {
            let mut downloads = manager.downloads.lock().unwrap();
            downloads.get_mut("test_1").unwrap().status = DownloadStatus::InProgress;
        }

        // Now can pause in-progress download
        let result = manager.pause_download("test_1");
        assert!(result.is_ok());

        let info = manager.get_download_info("test_1").unwrap();
        assert_eq!(info.status, DownloadStatus::Paused);

        // Cannot pause nonexistent download
        let result = manager.pause_download("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_cancel_download() {
        let manager = DownloadManager::new();

        manager.queue_download(
            "test_1".to_string(),
            "https://example.com/file.zip".to_string(),
            PathBuf::from("/tmp/file.zip"),
        ).unwrap();

        let result = manager.cancel_download("test_1");
        assert!(result.is_ok());

        let info = manager.get_download_info("test_1").unwrap();
        assert_eq!(info.status, DownloadStatus::Cancelled);
    }

    #[test]
    fn test_remove_download() {
        let manager = DownloadManager::new();

        manager.queue_download(
            "test_1".to_string(),
            "https://example.com/file.zip".to_string(),
            PathBuf::from("/tmp/file.zip"),
        ).unwrap();

        assert!(manager.get_download_info("test_1").is_some());

        let result = manager.remove_download("test_1");
        assert!(result.is_ok());

        assert!(manager.get_download_info("test_1").is_none());
    }

    #[test]
    fn test_clear_completed() {
        let manager = DownloadManager::new();

        // Add several downloads
        for i in 0..5 {
            manager.queue_download(
                format!("test_{}", i),
                format!("https://example.com/file{}.zip", i),
                PathBuf::from(format!("/tmp/file{}.zip", i)),
            ).unwrap();
        }

        // Manually mark some as completed/failed for testing
        {
            let mut downloads = manager.downloads.lock().unwrap();
            downloads.get_mut("test_0").unwrap().status = DownloadStatus::Completed;
            downloads.get_mut("test_1").unwrap().status = DownloadStatus::Failed;
            downloads.get_mut("test_2").unwrap().status = DownloadStatus::Queued;
        }

        manager.clear_completed();

        let remaining = manager.get_all_downloads();
        assert_eq!(remaining.len(), 3);
        assert!(remaining.iter().all(|d|
            d.status != DownloadStatus::Completed &&
            d.status != DownloadStatus::Failed
        ));
    }

    #[test]
    fn test_download_status_equality() {
        assert_eq!(DownloadStatus::Queued, DownloadStatus::Queued);
        assert_ne!(DownloadStatus::Queued, DownloadStatus::InProgress);
        assert_eq!(DownloadStatus::Completed, DownloadStatus::Completed);
    }

    #[test]
    fn test_download_info_clone() {
        let info = DownloadInfo::new(
            "test".to_string(),
            "https://example.com/file.zip".to_string(),
            PathBuf::from("/tmp/file.zip"),
        );

        let cloned = info.clone();
        assert_eq!(info.id, cloned.id);
        assert_eq!(info.url, cloned.url);
        assert_eq!(info.status, cloned.status);
    }

    #[test]
    fn test_get_active_downloads() {
        let manager = DownloadManager::new();

        manager.queue_download(
            "test_1".to_string(),
            "https://example.com/file1.zip".to_string(),
            PathBuf::from("/tmp/file1.zip"),
        ).unwrap();

        manager.queue_download(
            "test_2".to_string(),
            "https://example.com/file2.zip".to_string(),
            PathBuf::from("/tmp/file2.zip"),
        ).unwrap();

        // Initially no active downloads
        assert_eq!(manager.get_active_downloads().len(), 0);

        // Manually set one to InProgress for testing
        {
            let mut downloads = manager.downloads.lock().unwrap();
            downloads.get_mut("test_1").unwrap().status = DownloadStatus::InProgress;
        }

        let active = manager.get_active_downloads();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, "test_1");
    }

    #[test]
    fn test_download_manager_default() {
        let manager = DownloadManager::default();
        assert!(manager.get_all_downloads().is_empty());
    }

    #[test]
    fn test_download_info_progress_with_zero_total() {
        let mut info = DownloadInfo::new(
            "test".to_string(),
            "https://example.com/file.zip".to_string(),
            PathBuf::from("/tmp/file.zip"),
        );

        // Total size is 0
        info.update_progress(100, Some(0));
        assert_eq!(info.progress_percent(), 0.0);
    }

    #[test]
    fn test_multiple_pause_cancel_operations() {
        let manager = DownloadManager::new();

        manager.queue_download(
            "test_1".to_string(),
            "https://example.com/file.zip".to_string(),
            PathBuf::from("/tmp/file.zip"),
        ).unwrap();

        // Set to InProgress to allow pause
        {
            let mut downloads = manager.downloads.lock().unwrap();
            downloads.get_mut("test_1").unwrap().status = DownloadStatus::InProgress;
        }

        // Pause
        manager.pause_download("test_1").unwrap();
        assert_eq!(
            manager.get_download_info("test_1").unwrap().status,
            DownloadStatus::Paused
        );

        // Cancel after pause
        manager.cancel_download("test_1").unwrap();
        assert_eq!(
            manager.get_download_info("test_1").unwrap().status,
            DownloadStatus::Cancelled
        );
    }
}
