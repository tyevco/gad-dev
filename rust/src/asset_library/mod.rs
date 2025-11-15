mod asset;
mod asset_manager;
mod config_manager;
mod asset_source_registry;
mod godot_asset_library_api;
mod repository_adapter;
mod authentication;
mod health_checker;
mod download_manager;
mod user_features;
mod image_cache;

pub use asset::{Asset, AssetCategory, AssetDependency, AssetVersionInfo, SemanticVersion};
pub use asset_manager::AssetManager;
pub use config_manager::{AssetSource, Config, ConfigManager, UserPreferences};
pub use asset_source_registry::{
    AssetSourceRegistry, AuthMethod, ManagedAssetSource, SourceCapabilities, SourceHealth,
    SourceHealthStats,
};
pub use godot_asset_library_api::{
    AssetListParams, AssetListResponse, GodotAssetDetail, GodotAssetLibraryClient,
    GodotAssetListItem, GODOT_ASSET_LIBRARY_API,
};
pub use repository_adapter::{
    GodotAssetLibraryAdapter, JsonFileAdapter, LocalDirectoryAdapter, RepositoryAdapter,
    RepositoryAdapterFactory, RepositorySearchOptions, RepositorySearchResult, RepositoryType,
};
pub use authentication::{AuthenticationManager, AuthenticationStore, Credentials};
pub use health_checker::{
    CircuitBreaker, CircuitState, FallbackOrchestrator, HealthCheckResult, HealthChecker,
    HealthHistory, SourceHealthStats as HealthStats,
};
pub use download_manager::{DownloadInfo, DownloadManager, DownloadManagerConfig, DownloadStatus};
pub use user_features::{AssetCollection, HistoryAction, HistoryEntry, UserFeatures, UserStatistics};
pub use image_cache::{ImageCache, CacheStats};