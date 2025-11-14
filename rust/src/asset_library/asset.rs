use serde::{Deserialize, Serialize};

/// Represents an asset in the Godot Asset Library
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Asset {
    /// Unique identifier for the asset
    pub id: String,

    /// Display name of the asset
    pub name: String,

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