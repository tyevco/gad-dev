use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

/// Current version of the plugin
pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// GitHub repository information
const GITHUB_REPO_OWNER: &str = "tyevco";
const GITHUB_REPO_NAME: &str = "gad-dev";
const GITHUB_API_BASE: &str = "https://api.github.com";

/// Represents a semantic version
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    /// Parses a version string (e.g., "v1.2.3" or "1.2.3")
    pub fn parse(s: &str) -> Result<Self, String> {
        let s = s.trim_start_matches('v');
        let parts: Vec<&str> = s.split('.').collect();

        if parts.len() != 3 {
            return Err(format!("Invalid version format: {}", s));
        }

        Ok(Version {
            major: parts[0]
                .parse()
                .map_err(|_| format!("Invalid major version: {}", parts[0]))?,
            minor: parts[1]
                .parse()
                .map_err(|_| format!("Invalid minor version: {}", parts[1]))?,
            patch: parts[2]
                .parse()
                .map_err(|_| format!("Invalid patch version: {}", parts[2]))?,
        })
    }

    /// Converts to string format "v1.2.3"
    pub fn to_string(&self) -> String {
        format!("v{}.{}.{}", self.major, self.minor, self.patch)
    }

    /// Checks if this version is newer than another
    pub fn is_newer_than(&self, other: &Version) -> bool {
        self > other
    }
}

/// GitHub release information
#[derive(Debug, Clone, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: String,
    pub body: String,
    pub html_url: String,
    pub published_at: String,
    pub assets: Vec<GitHubAsset>,
}

/// GitHub release asset
#[derive(Debug, Clone, Deserialize)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

/// Update information
#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub current_version: Version,
    pub latest_version: Version,
    pub is_update_available: bool,
    pub release_notes: String,
    pub download_url: String,
    pub release_url: String,
}

/// Update check result
#[derive(Debug, Clone)]
pub enum UpdateCheckResult {
    /// Update is available
    UpdateAvailable(UpdateInfo),
    /// Already on latest version
    UpToDate(Version),
    /// Check failed
    Error(String),
}

/// Version checker for checking plugin updates
pub struct VersionChecker {
    client: Client,
    last_check: Option<SystemTime>,
    check_interval: Duration,
}

impl VersionChecker {
    /// Creates a new version checker
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            last_check: None,
            check_interval: Duration::from_secs(24 * 60 * 60), // 24 hours
        }
    }

    /// Sets the check interval
    pub fn with_check_interval(mut self, interval: Duration) -> Self {
        self.check_interval = interval;
        self
    }

    /// Gets the current version
    pub fn get_current_version() -> Result<Version, String> {
        Version::parse(CURRENT_VERSION)
    }

    /// Checks if enough time has passed since last check
    pub fn should_check(&self) -> bool {
        match self.last_check {
            Some(last) => {
                if let Ok(elapsed) = SystemTime::now().duration_since(last) {
                    elapsed >= self.check_interval
                } else {
                    true // If time went backwards, check anyway
                }
            }
            None => true, // Never checked before
        }
    }

    /// Checks for updates from GitHub
    pub async fn check_for_updates(&mut self) -> UpdateCheckResult {
        // Parse current version
        let current_version = match Self::get_current_version() {
            Ok(v) => v,
            Err(e) => return UpdateCheckResult::Error(format!("Failed to parse current version: {}", e)),
        };

        // Fetch latest release from GitHub
        let latest_release = match self.fetch_latest_release().await {
            Ok(r) => r,
            Err(e) => return UpdateCheckResult::Error(format!("Failed to fetch latest release: {}", e)),
        };

        // Parse latest version
        let latest_version = match Version::parse(&latest_release.tag_name) {
            Ok(v) => v,
            Err(e) => return UpdateCheckResult::Error(format!("Failed to parse latest version: {}", e)),
        };

        // Update last check time
        self.last_check = Some(SystemTime::now());

        // Compare versions
        if latest_version.is_newer_than(&current_version) {
            // Find appropriate download URL for current platform
            let download_url = self.get_platform_download_url(&latest_release);

            UpdateCheckResult::UpdateAvailable(UpdateInfo {
                current_version,
                latest_version,
                is_update_available: true,
                release_notes: latest_release.body,
                download_url,
                release_url: latest_release.html_url,
            })
        } else {
            UpdateCheckResult::UpToDate(current_version)
        }
    }

    /// Fetches the latest release from GitHub
    async fn fetch_latest_release(&self) -> Result<GitHubRelease, String> {
        let url = format!(
            "{}/repos/{}/{}/releases/latest",
            GITHUB_API_BASE, GITHUB_REPO_OWNER, GITHUB_REPO_NAME
        );

        let response = self
            .client
            .get(&url)
            .header("User-Agent", "Godot-Asset-Browser")
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("GitHub API returned status: {}", response.status()));
        }

        response
            .json::<GitHubRelease>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    /// Gets the appropriate download URL for the current platform
    fn get_platform_download_url(&self, release: &GitHubRelease) -> String {
        #[cfg(target_os = "linux")]
        let platform_suffix = "linux-x64.tar.gz";

        #[cfg(target_os = "windows")]
        let platform_suffix = "windows-x64.zip";

        #[cfg(target_os = "macos")]
        let platform_suffix = "macos-universal.tar.gz";

        // Find matching asset
        for asset in &release.assets {
            if asset.name.contains(platform_suffix) {
                return asset.browser_download_url.clone();
            }
        }

        // Fallback to release page
        release.html_url.clone()
    }

    /// Downloads an update to a specified path
    pub async fn download_update(&self, url: &str, destination: &str) -> Result<(), String> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("Download failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Download failed with status: {}", response.status()));
        }

        let content = response
            .bytes()
            .await
            .map_err(|e| format!("Failed to read download: {}", e))?;

        std::fs::write(destination, content)
            .map_err(|e| format!("Failed to write file: {}", e))?;

        Ok(())
    }
}

impl Default for VersionChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        let v1 = Version::parse("1.2.3").unwrap();
        assert_eq!(v1.major, 1);
        assert_eq!(v1.minor, 2);
        assert_eq!(v1.patch, 3);

        let v2 = Version::parse("v0.1.0").unwrap();
        assert_eq!(v2.major, 0);
        assert_eq!(v2.minor, 1);
        assert_eq!(v2.patch, 0);
    }

    #[test]
    fn test_version_parsing_invalid() {
        assert!(Version::parse("1.2").is_err());
        assert!(Version::parse("abc").is_err());
        assert!(Version::parse("1.2.3.4").is_err());
    }

    #[test]
    fn test_version_comparison() {
        let v1 = Version::parse("1.0.0").unwrap();
        let v2 = Version::parse("1.0.1").unwrap();
        let v3 = Version::parse("1.1.0").unwrap();
        let v4 = Version::parse("2.0.0").unwrap();

        assert!(v2.is_newer_than(&v1));
        assert!(v3.is_newer_than(&v2));
        assert!(v4.is_newer_than(&v3));
        assert!(!v1.is_newer_than(&v2));
    }

    #[test]
    fn test_version_to_string() {
        let v = Version {
            major: 1,
            minor: 2,
            patch: 3,
        };
        assert_eq!(v.to_string(), "v1.2.3");
    }

    #[test]
    fn test_current_version() {
        let current = VersionChecker::get_current_version();
        assert!(current.is_ok());
    }

    #[test]
    fn test_version_checker_creation() {
        let checker = VersionChecker::new();
        assert!(checker.should_check());
    }

    #[test]
    fn test_check_interval() {
        let checker = VersionChecker::new()
            .with_check_interval(Duration::from_secs(3600));
        assert_eq!(checker.check_interval, Duration::from_secs(3600));
    }
}
