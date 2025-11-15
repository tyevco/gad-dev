use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Semantic version following the semver specification (major.minor.patch)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SemanticVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    /// Optional pre-release identifier (e.g., "alpha", "beta.1", "rc.2")
    pub pre_release: Option<String>,
}

impl SemanticVersion {
    /// Creates a new semantic version
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: None,
        }
    }

    /// Creates a new semantic version with pre-release identifier
    pub fn with_pre_release(major: u32, minor: u32, patch: u32, pre_release: String) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: Some(pre_release),
        }
    }

    /// Parses a version string (e.g., "1.2.3" or "2.0.0-beta.1")
    pub fn parse(version_str: &str) -> Result<Self, String> {
        let (version_part, pre_release) = if let Some(dash_pos) = version_str.find('-') {
            let (v, p) = version_str.split_at(dash_pos);
            (v, Some(p[1..].to_string()))
        } else {
            (version_str, None)
        };

        let parts: Vec<&str> = version_part.split('.').collect();
        if parts.len() != 3 {
            return Err(format!("Invalid version format: {}", version_str));
        }

        let major = parts[0]
            .parse::<u32>()
            .map_err(|_| format!("Invalid major version: {}", parts[0]))?;
        let minor = parts[1]
            .parse::<u32>()
            .map_err(|_| format!("Invalid minor version: {}", parts[1]))?;
        let patch = parts[2]
            .parse::<u32>()
            .map_err(|_| format!("Invalid patch version: {}", parts[2]))?;

        Ok(Self {
            major,
            minor,
            patch,
            pre_release,
        })
    }

    /// Converts the version to a string representation
    pub fn to_string(&self) -> String {
        let base = format!("{}.{}.{}", self.major, self.minor, self.patch);
        if let Some(pre) = &self.pre_release {
            format!("{}-{}", base, pre)
        } else {
            base
        }
    }

    /// Checks if this version is compatible with another (same major version)
    pub fn is_compatible_with(&self, other: &SemanticVersion) -> bool {
        self.major == other.major && self.major > 0
    }
}

impl PartialOrd for SemanticVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SemanticVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => match self.minor.cmp(&other.minor) {
                Ordering::Equal => match self.patch.cmp(&other.patch) {
                    Ordering::Equal => {
                        // Pre-release versions have lower precedence
                        match (&self.pre_release, &other.pre_release) {
                            (None, None) => Ordering::Equal,
                            (None, Some(_)) => Ordering::Greater,
                            (Some(_), None) => Ordering::Less,
                            (Some(a), Some(b)) => a.cmp(b),
                        }
                    }
                    other => other,
                },
                other => other,
            },
            other => other,
        }
    }
}

impl Default for SemanticVersion {
    fn default() -> Self {
        Self::new(0, 0, 0)
    }
}

/// Information about a specific version of an asset
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetVersionInfo {
    /// Semantic version
    pub version: SemanticVersion,

    /// Release date (ISO 8601 format string)
    pub release_date: String,

    /// Changelog or release notes
    pub changelog: String,

    /// Download URL for this specific version
    pub download_url: String,

    /// Whether this version is marked as deprecated
    pub deprecated: bool,
}

impl AssetVersionInfo {
    /// Creates a new asset version info
    pub fn new(
        version: SemanticVersion,
        release_date: String,
        changelog: String,
        download_url: String,
    ) -> Self {
        Self {
            version,
            release_date,
            changelog,
            download_url,
            deprecated: false,
        }
    }

    /// Marks this version as deprecated
    pub fn mark_deprecated(&mut self) {
        self.deprecated = true;
    }
}

/// Category/type of asset in the Godot Asset Library
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetCategory {
    /// 2D tools, sprites, and resources
    #[serde(rename = "2D")]
    TwoD,

    /// 3D models, meshes, and scenes
    #[serde(rename = "3D")]
    ThreeD,

    /// Shader files and shader graphs
    Shaders,

    /// Materials and textures
    Materials,

    /// Audio files (music, sound effects, etc.)
    Audio,

    /// Script files (GDScript, C#, etc.)
    Scripts,

    /// Editor tools and plugins
    Tools,

    /// Templates and project starters
    Templates,

    /// Demos and example projects
    Demos,

    /// Miscellaneous assets
    Misc,
}

impl AssetCategory {
    /// Returns a human-readable display name for the category
    pub fn display_name(&self) -> &'static str {
        match self {
            AssetCategory::TwoD => "2D",
            AssetCategory::ThreeD => "3D",
            AssetCategory::Shaders => "Shaders",
            AssetCategory::Materials => "Materials",
            AssetCategory::Audio => "Audio",
            AssetCategory::Scripts => "Scripts",
            AssetCategory::Tools => "Tools",
            AssetCategory::Templates => "Templates",
            AssetCategory::Demos => "Demos",
            AssetCategory::Misc => "Miscellaneous",
        }
    }

    /// Returns all available categories
    pub fn all() -> Vec<AssetCategory> {
        vec![
            AssetCategory::TwoD,
            AssetCategory::ThreeD,
            AssetCategory::Shaders,
            AssetCategory::Materials,
            AssetCategory::Audio,
            AssetCategory::Scripts,
            AssetCategory::Tools,
            AssetCategory::Templates,
            AssetCategory::Demos,
            AssetCategory::Misc,
        ]
    }
}

impl Default for AssetCategory {
    fn default() -> Self {
        AssetCategory::Misc
    }
}

/// Represents a dependency on another asset
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetDependency {
    /// ID of the dependent asset
    pub asset_id: String,

    /// Name of the dependent asset
    pub asset_name: String,

    /// Required version or version constraint (e.g., "1.0.0", ">=2.0.0", "^1.5.0")
    pub version_requirement: String,

    /// Whether this dependency is optional or required
    pub optional: bool,
}

impl AssetDependency {
    /// Creates a new required dependency
    pub fn new(asset_id: String, asset_name: String, version_requirement: String) -> Self {
        Self {
            asset_id,
            asset_name,
            version_requirement,
            optional: false,
        }
    }

    /// Creates a new optional dependency
    pub fn optional(asset_id: String, asset_name: String, version_requirement: String) -> Self {
        Self {
            asset_id,
            asset_name,
            version_requirement,
            optional: true,
        }
    }

    /// Checks if this dependency is required
    pub fn is_required(&self) -> bool {
        !self.optional
    }
}

/// Represents an asset in the Godot Asset Library
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Asset {
    /// Unique identifier for the asset
    pub id: String,

    /// Display name of the asset
    pub name: String,

    /// Category/type of the asset
    pub category: AssetCategory,

    /// Local filesystem path where the asset is stored (if downloaded)
    pub path: String,

    /// Author/creator of the asset
    pub author: String,

    /// Version string (e.g., "1.0.0", "2.1.3")
    pub version: String,

    /// Detailed description of the asset
    pub description: String,

    /// Tags/keywords for categorization and search
    pub tags: Vec<String>,

    /// URL to the preview/thumbnail image (optional)
    pub preview_url: Option<String>,

    /// URL to download the asset
    pub download_url: String,

    /// Dependencies on other assets
    pub dependencies: Vec<AssetDependency>,

    /// Version history (all available versions, sorted from newest to oldest)
    pub version_history: Vec<AssetVersionInfo>,
}

impl Asset {
    /// Creates a new Asset with the given parameters
    pub fn new(
        id: String,
        name: String,
        category: AssetCategory,
        path: String,
        author: String,
        version: String,
        description: String,
        tags: Vec<String>,
        preview_url: Option<String>,
        download_url: String,
        dependencies: Vec<AssetDependency>,
        version_history: Vec<AssetVersionInfo>,
    ) -> Self {
        Self {
            id,
            name,
            category,
            path,
            author,
            version,
            description,
            tags,
            preview_url,
            download_url,
            dependencies,
            version_history,
        }
    }

    /// Creates a minimal Asset (useful for testing or placeholders)
    pub fn minimal(id: String, name: String) -> Self {
        Self {
            id,
            name,
            category: AssetCategory::default(),
            path: String::new(),
            author: String::new(),
            version: String::from("0.0.0"),
            description: String::new(),
            tags: Vec::new(),
            preview_url: None,
            download_url: String::new(),
            dependencies: Vec::new(),
            version_history: Vec::new(),
        }
    }

    /// Adds a dependency to this asset
    pub fn add_dependency(&mut self, dependency: AssetDependency) {
        self.dependencies.push(dependency);
    }

    /// Checks if this asset has any dependencies
    pub fn has_dependencies(&self) -> bool {
        !self.dependencies.is_empty()
    }

    /// Gets all required dependencies (non-optional)
    pub fn required_dependencies(&self) -> Vec<&AssetDependency> {
        self.dependencies
            .iter()
            .filter(|dep| dep.is_required())
            .collect()
    }

    /// Gets all optional dependencies
    pub fn optional_dependencies(&self) -> Vec<&AssetDependency> {
        self.dependencies
            .iter()
            .filter(|dep| dep.optional)
            .collect()
    }

    /// Checks if this asset depends on another asset by ID
    pub fn depends_on(&self, asset_id: &str) -> bool {
        self.dependencies.iter().any(|dep| dep.asset_id == asset_id)
    }

    /// Adds a version to the version history
    pub fn add_version(&mut self, version_info: AssetVersionInfo) {
        self.version_history.push(version_info);
        // Sort by version (newest first)
        self.version_history.sort_by(|a, b| b.version.cmp(&a.version));
    }

    /// Gets the latest version from version history
    pub fn latest_version(&self) -> Option<&AssetVersionInfo> {
        self.version_history.first()
    }

    /// Gets a specific version by version string
    pub fn get_version(&self, version_str: &str) -> Option<&AssetVersionInfo> {
        if let Ok(target_version) = SemanticVersion::parse(version_str) {
            self.version_history
                .iter()
                .find(|v| v.version == target_version)
        } else {
            None
        }
    }

    /// Gets all non-deprecated versions
    pub fn available_versions(&self) -> Vec<&AssetVersionInfo> {
        self.version_history
            .iter()
            .filter(|v| !v.deprecated)
            .collect()
    }

    /// Checks if an update is available (compared to current version)
    pub fn has_update(&self) -> bool {
        if let Ok(current) = SemanticVersion::parse(&self.version) {
            if let Some(latest) = self.latest_version() {
                return latest.version > current;
            }
        }
        false
    }

    /// Gets the parsed current version as SemanticVersion
    pub fn current_semantic_version(&self) -> Result<SemanticVersion, String> {
        SemanticVersion::parse(&self.version)
    }
}