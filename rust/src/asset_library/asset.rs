use serde::{Deserialize, Serialize};

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
}