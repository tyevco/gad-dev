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
        }
    }
}