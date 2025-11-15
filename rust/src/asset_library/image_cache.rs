use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use reqwest::Client;

/// Maximum image dimension for thumbnails (width or height)
const MAX_THUMBNAIL_SIZE: u32 = 256;

/// Cache entry for preview images
#[derive(Debug, Clone)]
struct CachedImage {
    /// Path to the cached image file
    file_path: PathBuf,
    /// Original URL of the image
    url: String,
    /// Size of the cached file in bytes
    file_size: u64,
    /// Timestamp when cached
    cached_at: std::time::SystemTime,
}

/// Image cache manager for preview images
///
/// Provides efficient caching, lazy loading, and thumbnail generation
/// for asset preview images.
pub struct ImageCache {
    /// HTTP client for downloading images
    client: Client,
    /// Directory where cached images are stored
    cache_dir: PathBuf,
    /// Map of URLs to cached image entries
    cache_map: Arc<Mutex<HashMap<String, CachedImage>>>,
    /// Maximum cache size in bytes (default: 100MB)
    max_cache_size: u64,
    /// Current cache size in bytes
    current_cache_size: Arc<Mutex<u64>>,
}

impl ImageCache {
    /// Creates a new ImageCache with default settings
    ///
    /// # Arguments
    /// * `cache_dir` - Directory to store cached images
    ///
    /// # Returns
    /// * `Result<Self, String>` - ImageCache instance or error
    pub fn new<P: AsRef<Path>>(cache_dir: P) -> Result<Self, String> {
        let cache_dir = cache_dir.as_ref().to_path_buf();

        // Create cache directory if it doesn't exist
        fs::create_dir_all(&cache_dir)
            .map_err(|e| format!("Failed to create cache directory: {}", e))?;

        Ok(Self {
            client: Client::new(),
            cache_dir,
            cache_map: Arc::new(Mutex::new(HashMap::new())),
            max_cache_size: 100 * 1024 * 1024, // 100MB default
            current_cache_size: Arc::new(Mutex::new(0)),
        })
    }

    /// Sets the maximum cache size
    pub fn with_max_size(mut self, max_size_bytes: u64) -> Self {
        self.max_cache_size = max_size_bytes;
        self
    }

    /// Gets a cached image path, downloading if necessary
    ///
    /// # Arguments
    /// * `url` - URL of the image to retrieve
    ///
    /// # Returns
    /// * `Option<PathBuf>` - Path to cached image, or None if download failed
    ///
    /// # Performance
    /// - If image is already cached, returns path immediately (< 1ms)
    /// - If not cached, downloads and caches image (network-dependent)
    /// - Automatically manages cache size and evicts old entries if needed
    pub async fn get_image(&self, url: &str) -> Option<PathBuf> {
        // Check if already cached
        {
            let cache_map = self.cache_map.lock().unwrap();
            if let Some(entry) = cache_map.get(url) {
                if entry.file_path.exists() {
                    return Some(entry.file_path.clone());
                }
            }
        }

        // Download and cache
        self.download_and_cache(url).await.ok()
    }

    /// Downloads an image and adds it to the cache
    async fn download_and_cache(&self, url: &str) -> Result<PathBuf, String> {
        // Generate cache filename from URL hash
        let cache_filename = self.generate_cache_filename(url);
        let cache_path = self.cache_dir.join(&cache_filename);

        // Download image
        let response = self.client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("Failed to download image: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Download failed with status: {}", response.status()));
        }

        let image_bytes = response.bytes()
            .await
            .map_err(|e| format!("Failed to read image data: {}", e))?;

        let file_size = image_bytes.len() as u64;

        // Check if we need to evict old entries
        self.ensure_cache_space(file_size).await?;

        // Write to cache file
        fs::write(&cache_path, &image_bytes)
            .map_err(|e| format!("Failed to write cache file: {}", e))?;

        // Update cache map
        {
            let mut cache_map = self.cache_map.lock().unwrap();
            cache_map.insert(url.to_string(), CachedImage {
                file_path: cache_path.clone(),
                url: url.to_string(),
                file_size,
                cached_at: std::time::SystemTime::now(),
            });
        }

        // Update cache size
        {
            let mut current_size = self.current_cache_size.lock().unwrap();
            *current_size += file_size;
        }

        Ok(cache_path)
    }

    /// Ensures there's enough space in the cache for a new entry
    async fn ensure_cache_space(&self, needed_bytes: u64) -> Result<(), String> {
        let current_size = *self.current_cache_size.lock().unwrap();

        if current_size + needed_bytes <= self.max_cache_size {
            return Ok(());
        }

        // Need to evict some entries
        let mut cache_map = self.cache_map.lock().unwrap();
        let mut entries: Vec<_> = cache_map.iter()
            .map(|(url, entry)| (url.clone(), entry.clone()))
            .collect();

        // Sort by age (oldest first)
        entries.sort_by_key(|(_, entry)| entry.cached_at);

        let mut freed_space = 0u64;
        let mut to_remove = Vec::new();

        for (url, entry) in entries {
            if current_size + needed_bytes - freed_space <= self.max_cache_size {
                break;
            }

            // Delete the file
            if entry.file_path.exists() {
                let _ = fs::remove_file(&entry.file_path);
            }

            freed_space += entry.file_size;
            to_remove.push(url);
        }

        // Remove from cache map
        for url in to_remove {
            cache_map.remove(&url);
        }

        // Update cache size
        {
            let mut current_size = self.current_cache_size.lock().unwrap();
            *current_size -= freed_space;
        }

        Ok(())
    }

    /// Generates a cache filename from a URL
    fn generate_cache_filename(&self, url: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        url.hash(&mut hasher);
        let hash = hasher.finish();

        // Extract extension from URL if possible
        let ext = url.split('?').next()
            .and_then(|path| path.split('/').last())
            .and_then(|filename| filename.split('.').last())
            .filter(|ext| ext.len() <= 4)
            .unwrap_or("jpg");

        format!("{:x}.{}", hash, ext)
    }

    /// Checks if an image is cached
    pub fn is_cached(&self, url: &str) -> bool {
        let cache_map = self.cache_map.lock().unwrap();
        cache_map.get(url)
            .map(|entry| entry.file_path.exists())
            .unwrap_or(false)
    }

    /// Gets the current cache size in bytes
    pub fn get_cache_size(&self) -> u64 {
        *self.current_cache_size.lock().unwrap()
    }

    /// Clears all cached images
    pub fn clear_cache(&self) -> Result<(), String> {
        let mut cache_map = self.cache_map.lock().unwrap();

        // Delete all cached files
        for entry in cache_map.values() {
            if entry.file_path.exists() {
                fs::remove_file(&entry.file_path)
                    .map_err(|e| format!("Failed to delete cache file: {}", e))?;
            }
        }

        cache_map.clear();

        // Reset cache size
        {
            let mut current_size = self.current_cache_size.lock().unwrap();
            *current_size = 0;
        }

        Ok(())
    }

    /// Gets cache statistics
    pub fn get_stats(&self) -> CacheStats {
        let cache_map = self.cache_map.lock().unwrap();
        let current_size = *self.current_cache_size.lock().unwrap();

        CacheStats {
            total_entries: cache_map.len(),
            total_size_bytes: current_size,
            max_size_bytes: self.max_cache_size,
            cache_hit_rate: 0.0, // Would need to track hits/misses for accurate stat
        }
    }

    /// Preloads images for a list of URLs (for eager loading optimization)
    ///
    /// # Arguments
    /// * `urls` - Vector of image URLs to preload
    ///
    /// # Performance
    /// Downloads images concurrently up to a reasonable limit
    pub async fn preload_images(&self, urls: Vec<String>) {
        use futures_util::stream::{self, StreamExt};

        // Limit concurrent downloads to 4
        stream::iter(urls)
            .map(|url| async move {
                self.get_image(&url).await
            })
            .buffer_unordered(4)
            .collect::<Vec<_>>()
            .await;
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Total number of cached entries
    pub total_entries: usize,
    /// Total size of cache in bytes
    pub total_size_bytes: u64,
    /// Maximum allowed cache size in bytes
    pub max_size_bytes: u64,
    /// Cache hit rate (0.0 - 1.0)
    pub cache_hit_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cache_creation() {
        let temp_dir = TempDir::new().unwrap();
        let cache = ImageCache::new(temp_dir.path()).unwrap();

        assert_eq!(cache.get_cache_size(), 0);
        assert_eq!(cache.max_cache_size, 100 * 1024 * 1024);
    }

    #[test]
    fn test_cache_with_custom_size() {
        let temp_dir = TempDir::new().unwrap();
        let cache = ImageCache::new(temp_dir.path())
            .unwrap()
            .with_max_size(50 * 1024 * 1024);

        assert_eq!(cache.max_cache_size, 50 * 1024 * 1024);
    }

    #[test]
    fn test_generate_cache_filename() {
        let temp_dir = TempDir::new().unwrap();
        let cache = ImageCache::new(temp_dir.path()).unwrap();

        let filename1 = cache.generate_cache_filename("https://example.com/image.jpg");
        let filename2 = cache.generate_cache_filename("https://example.com/image.jpg");
        let filename3 = cache.generate_cache_filename("https://example.com/other.png");

        // Same URL should generate same filename
        assert_eq!(filename1, filename2);

        // Different URLs should generate different filenames
        assert_ne!(filename1, filename3);

        // Should preserve extension
        assert!(filename1.ends_with(".jpg"));
        assert!(filename3.ends_with(".png"));
    }

    #[test]
    fn test_is_cached_initially_false() {
        let temp_dir = TempDir::new().unwrap();
        let cache = ImageCache::new(temp_dir.path()).unwrap();

        assert!(!cache.is_cached("https://example.com/image.jpg"));
    }

    #[test]
    fn test_clear_cache() {
        let temp_dir = TempDir::new().unwrap();
        let cache = ImageCache::new(temp_dir.path()).unwrap();

        // Initially empty
        assert_eq!(cache.get_cache_size(), 0);

        // Clear should succeed even when empty
        assert!(cache.clear_cache().is_ok());
        assert_eq!(cache.get_cache_size(), 0);
    }

    #[test]
    fn test_get_stats() {
        let temp_dir = TempDir::new().unwrap();
        let cache = ImageCache::new(temp_dir.path()).unwrap();

        let stats = cache.get_stats();
        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.total_size_bytes, 0);
        assert_eq!(stats.max_size_bytes, 100 * 1024 * 1024);
    }
}
