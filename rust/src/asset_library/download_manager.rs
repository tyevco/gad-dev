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

    /// Adds a download to the queue
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

    /// Starts a queued download
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

    /// Pauses an active download
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

    /// Cancels a download
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

    /// Gets information about a download
    pub fn get_download_info(&self, id: &str) -> Option<DownloadInfo> {
        self.downloads.lock().unwrap().get(id).cloned()
    }

    /// Gets information about all downloads
    pub fn get_all_downloads(&self) -> Vec<DownloadInfo> {
        self.downloads.lock().unwrap().values().cloned().collect()
    }

    /// Gets active downloads
    pub fn get_active_downloads(&self) -> Vec<DownloadInfo> {
        self.downloads
            .lock()
            .unwrap()
            .values()
            .filter(|info| info.status == DownloadStatus::InProgress)
            .cloned()
            .collect()
    }

    /// Gets queued downloads
    pub fn get_queued_downloads(&self) -> Vec<DownloadInfo> {
        self.downloads
            .lock()
            .unwrap()
            .values()
            .filter(|info| info.status == DownloadStatus::Queued)
            .cloned()
            .collect()
    }

    /// Removes a completed or failed download from the list
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
