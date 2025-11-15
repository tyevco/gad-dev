use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// User preferences and features for asset management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFeatures {
    /// Favorite/bookmarked asset IDs
    favorites: HashSet<String>,
    /// Asset ratings (asset_id -> rating 1-5)
    ratings: HashMap<String, u8>,
    /// Installation history with timestamps
    history: Vec<HistoryEntry>,
    /// User-created collections/bundles
    collections: HashMap<String, AssetCollection>,
    /// Path to store user features data
    #[serde(skip)]
    data_file: PathBuf,
}

/// A history entry for asset installation/update/removal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Asset ID
    pub asset_id: String,
    /// Action performed (Install, Update, Uninstall)
    pub action: HistoryAction,
    /// Timestamp (Unix epoch seconds)
    pub timestamp: u64,
    /// Asset version at the time
    pub version: String,
    /// Optional notes
    pub notes: Option<String>,
}

/// Actions that can be recorded in history
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HistoryAction {
    Install,
    Update,
    Uninstall,
    Download,
}

impl HistoryAction {
    pub fn as_str(&self) -> &str {
        match self {
            HistoryAction::Install => "Install",
            HistoryAction::Update => "Update",
            HistoryAction::Uninstall => "Uninstall",
            HistoryAction::Download => "Download",
        }
    }
}

/// A collection/bundle of assets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetCollection {
    /// Collection name
    pub name: String,
    /// Description
    pub description: String,
    /// Asset IDs in this collection
    pub assets: Vec<String>,
    /// When this collection was created
    pub created_at: u64,
    /// When this collection was last modified
    pub modified_at: u64,
}

impl UserFeatures {
    /// Creates a new UserFeatures instance with empty data.
    ///
    /// # Arguments
    /// * `data_file` - Path where user features data will be saved
    ///
    /// # Returns
    /// * `Self` - A new UserFeatures instance
    ///
    /// # Example
    /// ```
    /// let features = UserFeatures::new(PathBuf::from("user://user_features.json"));
    /// ```
    pub fn new(data_file: PathBuf) -> Self {
        Self {
            favorites: HashSet::new(),
            ratings: HashMap::new(),
            history: Vec::new(),
            collections: HashMap::new(),
            data_file,
        }
    }

    /// Loads user features from disk.
    ///
    /// If the file doesn't exist, returns a new empty instance. This method
    /// deserializes the JSON data and restores all favorites, ratings, history,
    /// and collections.
    ///
    /// # Arguments
    /// * `data_file` - Path to the user features data file
    ///
    /// # Returns
    /// * `Ok(UserFeatures)` - Loaded user features or new instance if file doesn't exist
    /// * `Err(String)` - Error message if file exists but can't be read or parsed
    ///
    /// # Example
    /// ```
    /// let features = UserFeatures::load(PathBuf::from("user://user_features.json"))?;
    /// println!("Loaded {} favorites", features.get_favorites().len());
    /// ```
    pub fn load(data_file: PathBuf) -> Result<Self, String> {
        if !data_file.exists() {
            return Ok(Self::new(data_file));
        }

        let content = fs::read_to_string(&data_file)
            .map_err(|e| format!("Failed to read user features file: {}", e))?;

        let mut features: UserFeatures = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse user features: {}", e))?;

        features.data_file = data_file;
        Ok(features)
    }

    /// Saves user features to disk.
    ///
    /// Serializes all user data (favorites, ratings, history, collections) to
    /// JSON format and writes it to the data file. Creates the parent directory
    /// if it doesn't exist.
    ///
    /// # Returns
    /// * `Ok(())` - Data was successfully saved
    /// * `Err(String)` - Error message if save failed
    ///
    /// # Example
    /// ```
    /// let mut features = UserFeatures::load(PathBuf::from("user://user_features.json"))?;
    /// features.add_favorite("asset_123".to_string());
    /// features.save()?; // Save changes to disk
    /// ```
    pub fn save(&self) -> Result<(), String> {
        // Create parent directory if needed
        if let Some(parent) = self.data_file.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create user features directory: {}", e))?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize user features: {}", e))?;

        fs::write(&self.data_file, content)
            .map_err(|e| format!("Failed to write user features file: {}", e))?;

        Ok(())
    }

    // ===== Favorites Management =====

    /// Adds an asset to the favorites list.
    ///
    /// If the asset is already a favorite, this method does nothing and returns false.
    /// The changes are automatically saved to disk.
    ///
    /// # Arguments
    /// * `asset_id` - The ID of the asset to add to favorites
    ///
    /// # Returns
    /// * `true` - Asset was added to favorites
    /// * `false` - Asset was already in favorites
    ///
    /// # Example
    /// ```
    /// let mut features = UserFeatures::new(PathBuf::from("user://user_features.json"));
    /// if features.add_favorite("asset_123".to_string()) {
    ///     println!("Added to favorites!");
    /// }
    /// ```
    pub fn add_favorite(&mut self, asset_id: String) -> bool {
        let added = self.favorites.insert(asset_id);
        if added {
            let _ = self.save();
        }
        added
    }

    /// Remove an asset from favorites
    pub fn remove_favorite(&mut self, asset_id: &str) -> bool {
        let removed = self.favorites.remove(asset_id);
        if removed {
            let _ = self.save();
        }
        removed
    }

    /// Check if an asset is favorited
    pub fn is_favorite(&self, asset_id: &str) -> bool {
        self.favorites.contains(asset_id)
    }

    /// Get all favorite asset IDs
    pub fn get_favorites(&self) -> Vec<String> {
        self.favorites.iter().cloned().collect()
    }

    /// Toggle favorite status
    pub fn toggle_favorite(&mut self, asset_id: String) -> bool {
        if self.is_favorite(&asset_id) {
            self.remove_favorite(&asset_id);
            false
        } else {
            self.add_favorite(asset_id);
            true
        }
    }

    // ===== Ratings Management =====

    /// Set a rating for an asset (1-5 stars)
    pub fn set_rating(&mut self, asset_id: String, rating: u8) -> Result<(), String> {
        if rating < 1 || rating > 5 {
            return Err("Rating must be between 1 and 5".to_string());
        }

        self.ratings.insert(asset_id, rating);
        self.save()?;
        Ok(())
    }

    /// Get the rating for an asset
    pub fn get_rating(&self, asset_id: &str) -> Option<u8> {
        self.ratings.get(asset_id).copied()
    }

    /// Remove a rating
    pub fn remove_rating(&mut self, asset_id: &str) -> bool {
        let removed = self.ratings.remove(asset_id).is_some();
        if removed {
            let _ = self.save();
        }
        removed
    }

    /// Get all rated assets
    pub fn get_rated_assets(&self) -> Vec<(String, u8)> {
        self.ratings
            .iter()
            .map(|(id, rating)| (id.clone(), *rating))
            .collect()
    }

    // ===== History Management =====

    /// Add an entry to the history
    pub fn add_history(&mut self, asset_id: String, action: HistoryAction, version: String) {
        self.add_history_with_notes(asset_id, action, version, None);
    }

    /// Add an entry to the history with notes
    pub fn add_history_with_notes(
        &mut self,
        asset_id: String,
        action: HistoryAction,
        version: String,
        notes: Option<String>,
    ) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let entry = HistoryEntry {
            asset_id,
            action,
            timestamp,
            version,
            notes,
        };

        self.history.push(entry);
        let _ = self.save();
    }

    /// Get all history entries
    pub fn get_history(&self) -> &[HistoryEntry] {
        &self.history
    }

    /// Get history entries for a specific asset
    pub fn get_asset_history(&self, asset_id: &str) -> Vec<&HistoryEntry> {
        self.history
            .iter()
            .filter(|entry| entry.asset_id == asset_id)
            .collect()
    }

    /// Get history entries by action type
    pub fn get_history_by_action(&self, action: HistoryAction) -> Vec<&HistoryEntry> {
        self.history
            .iter()
            .filter(|entry| entry.action == action)
            .collect()
    }

    /// Get recent history entries (last N entries)
    pub fn get_recent_history(&self, count: usize) -> Vec<&HistoryEntry> {
        let len = self.history.len();
        if count >= len {
            self.history.iter().collect()
        } else {
            self.history[len - count..].iter().collect()
        }
    }

    /// Clear all history
    pub fn clear_history(&mut self) -> Result<(), String> {
        self.history.clear();
        self.save()
    }

    /// Clear history for a specific asset
    pub fn clear_asset_history(&mut self, asset_id: &str) -> Result<(), String> {
        self.history.retain(|entry| entry.asset_id != asset_id);
        self.save()
    }

    // ===== Collections Management =====

    /// Creates a new asset collection.
    ///
    /// Collections are groups of related assets that can be managed together.
    /// They're useful for organizing assets by project, type, or any custom criteria.
    ///
    /// # Arguments
    /// * `id` - Unique identifier for the collection
    /// * `name` - Display name for the collection
    /// * `description` - Description of the collection's purpose
    ///
    /// # Returns
    /// * `Ok(())` - Collection was successfully created
    /// * `Err(String)` - Error if a collection with the same ID already exists
    ///
    /// # Example
    /// ```
    /// let mut features = UserFeatures::new(PathBuf::from("user://user_features.json"));
    /// features.create_collection(
    ///     "my_project".to_string(),
    ///     "My Game Project".to_string(),
    ///     "Assets for my RPG game".to_string()
    /// )?;
    /// ```
    pub fn create_collection(&mut self, id: String, name: String, description: String) -> Result<(), String> {
        if self.collections.contains_key(&id) {
            return Err(format!("Collection '{}' already exists", id));
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let collection = AssetCollection {
            name,
            description,
            assets: Vec::new(),
            created_at: timestamp,
            modified_at: timestamp,
        };

        self.collections.insert(id, collection);
        self.save()?;
        Ok(())
    }

    /// Delete a collection
    pub fn delete_collection(&mut self, collection_id: &str) -> Result<(), String> {
        if self.collections.remove(collection_id).is_none() {
            return Err(format!("Collection '{}' not found", collection_id));
        }

        self.save()?;
        Ok(())
    }

    /// Add an asset to a collection
    pub fn add_to_collection(&mut self, collection_id: &str, asset_id: String) -> Result<(), String> {
        let collection = self.collections.get_mut(collection_id)
            .ok_or_else(|| format!("Collection '{}' not found", collection_id))?;

        if collection.assets.contains(&asset_id) {
            return Err(format!("Asset '{}' already in collection", asset_id));
        }

        collection.assets.push(asset_id);
        collection.modified_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.save()?;
        Ok(())
    }

    /// Remove an asset from a collection
    pub fn remove_from_collection(&mut self, collection_id: &str, asset_id: &str) -> Result<(), String> {
        let collection = self.collections.get_mut(collection_id)
            .ok_or_else(|| format!("Collection '{}' not found", collection_id))?;

        let initial_len = collection.assets.len();
        collection.assets.retain(|id| id != asset_id);

        if collection.assets.len() == initial_len {
            return Err(format!("Asset '{}' not in collection", asset_id));
        }

        collection.modified_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.save()?;
        Ok(())
    }

    /// Get a collection
    pub fn get_collection(&self, collection_id: &str) -> Option<&AssetCollection> {
        self.collections.get(collection_id)
    }

    /// Get all collections
    pub fn get_all_collections(&self) -> Vec<(&String, &AssetCollection)> {
        self.collections.iter().collect()
    }

    /// Update collection metadata
    pub fn update_collection(
        &mut self,
        collection_id: &str,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<(), String> {
        let collection = self.collections.get_mut(collection_id)
            .ok_or_else(|| format!("Collection '{}' not found", collection_id))?;

        if let Some(name) = name {
            collection.name = name;
        }

        if let Some(description) = description {
            collection.description = description;
        }

        collection.modified_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.save()?;
        Ok(())
    }

    /// Get collections containing a specific asset
    pub fn get_collections_with_asset(&self, asset_id: &str) -> Vec<&String> {
        self.collections
            .iter()
            .filter(|(_, collection)| collection.assets.contains(&asset_id.to_string()))
            .map(|(id, _)| id)
            .collect()
    }

    // ===== Export/Import =====

    /// Exports a collection to a shareable JSON format.
    ///
    /// Creates a JSON representation of the collection that can be shared with
    /// other users or backed up. The exported data includes the collection name,
    /// description, asset list, and timestamps.
    ///
    /// # Arguments
    /// * `collection_id` - The ID of the collection to export
    ///
    /// # Returns
    /// * `Ok(String)` - JSON string representation of the collection
    /// * `Err(String)` - Error if collection not found or serialization failed
    ///
    /// # Example
    /// ```
    /// let features = UserFeatures::load(PathBuf::from("user://user_features.json"))?;
    /// let json = features.export_collection("my_project")?;
    /// std::fs::write("my_collection.json", json)?;
    /// ```
    pub fn export_collection(&self, collection_id: &str) -> Result<String, String> {
        let collection = self.collections.get(collection_id)
            .ok_or_else(|| format!("Collection '{}' not found", collection_id))?;

        serde_json::to_string_pretty(collection)
            .map_err(|e| format!("Failed to serialize collection: {}", e))
    }

    /// Imports a collection from a JSON string.
    ///
    /// Reads a collection that was previously exported and adds it to the user's
    /// collections. This is useful for sharing collections between users or
    /// restoring from backups.
    ///
    /// # Arguments
    /// * `id` - Unique ID to assign to the imported collection
    /// * `json_data` - JSON string containing the collection data
    ///
    /// # Returns
    /// * `Ok(())` - Collection was successfully imported
    /// * `Err(String)` - Error if collection ID exists, JSON invalid, or save failed
    ///
    /// # Example
    /// ```
    /// let mut features = UserFeatures::load(PathBuf::from("user://user_features.json"))?;
    /// let json = std::fs::read_to_string("shared_collection.json")?;
    /// features.import_collection("imported".to_string(), &json)?;
    /// ```
    pub fn import_collection(&mut self, id: String, json_data: &str) -> Result<(), String> {
        if self.collections.contains_key(&id) {
            return Err(format!("Collection '{}' already exists", id));
        }

        let collection: AssetCollection = serde_json::from_str(json_data)
            .map_err(|e| format!("Failed to parse collection: {}", e))?;

        self.collections.insert(id, collection);
        self.save()?;
        Ok(())
    }

    // ===== Statistics =====

    /// Get user statistics
    pub fn get_statistics(&self) -> UserStatistics {
        UserStatistics {
            total_favorites: self.favorites.len(),
            total_ratings: self.ratings.len(),
            total_history_entries: self.history.len(),
            total_collections: self.collections.len(),
            average_rating: if self.ratings.is_empty() {
                0.0
            } else {
                self.ratings.values().map(|&r| r as f32).sum::<f32>() / self.ratings.len() as f32
            },
        }
    }
}

/// User statistics
#[derive(Debug, Clone)]
pub struct UserStatistics {
    pub total_favorites: usize,
    pub total_ratings: usize,
    pub total_history_entries: usize,
    pub total_collections: usize,
    pub average_rating: f32,
}
