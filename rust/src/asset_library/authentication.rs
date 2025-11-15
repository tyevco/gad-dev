use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Authentication credentials for different auth types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Credentials {
    /// No authentication
    None,
    /// Bearer token (API token)
    BearerToken {
        token: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        expires_at: Option<SystemTime>,
    },
    /// API key authentication
    ApiKey {
        key: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        header_name: Option<String>,
    },
    /// Basic authentication
    Basic { username: String, password: String },
    /// OAuth 2.0 authentication
    OAuth2 {
        access_token: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        refresh_token: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        expires_at: Option<SystemTime>,
    },
}

impl Credentials {
    /// Checks if credentials are expired
    pub fn is_expired(&self) -> bool {
        match self {
            Credentials::BearerToken {
                expires_at: Some(exp),
                ..
            } => SystemTime::now() > *exp,
            Credentials::OAuth2 {
                expires_at: Some(exp),
                ..
            } => SystemTime::now() > *exp,
            _ => false,
        }
    }

    /// Checks if credentials need refresh
    pub fn needs_refresh(&self) -> bool {
        match self {
            Credentials::OAuth2 {
                refresh_token: Some(_),
                expires_at: Some(exp),
                ..
            } => {
                // Refresh if expiring within 5 minutes
                if let Ok(time_until_expiry) = exp.duration_since(SystemTime::now()) {
                    time_until_expiry < Duration::from_secs(300)
                } else {
                    true
                }
            }
            _ => false,
        }
    }

    /// Gets the HTTP authorization header value
    pub fn to_auth_header(&self) -> Option<(String, String)> {
        match self {
            Credentials::None => None,
            Credentials::BearerToken { token, .. } => {
                Some(("Authorization".to_string(), format!("Bearer {}", token)))
            }
            Credentials::ApiKey { key, header_name } => {
                let header = header_name
                    .as_ref()
                    .unwrap_or(&"X-API-Key".to_string())
                    .clone();
                Some((header, key.clone()))
            }
            Credentials::Basic { username, password } => {
                let encoded = base64_encode(format!("{}:{}", username, password));
                Some(("Authorization".to_string(), format!("Basic {}", encoded)))
            }
            Credentials::OAuth2 { access_token, .. } => Some((
                "Authorization".to_string(),
                format!("Bearer {}", access_token),
            )),
        }
    }
}

/// Authentication storage and management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationStore {
    /// Map of source name to credentials
    credentials: HashMap<String, Credentials>,
    /// Whether to encrypt stored credentials
    #[serde(skip)]
    encrypt_at_rest: bool,
}

impl AuthenticationStore {
    /// Creates a new authentication store
    pub fn new() -> Self {
        Self {
            credentials: HashMap::new(),
            encrypt_at_rest: true,
        }
    }

    /// Stores credentials for a source
    pub fn store_credentials(&mut self, source_name: String, credentials: Credentials) {
        self.credentials.insert(source_name, credentials);
    }

    /// Retrieves credentials for a source
    pub fn get_credentials(&self, source_name: &str) -> Option<&Credentials> {
        self.credentials.get(source_name)
    }

    /// Removes credentials for a source
    pub fn remove_credentials(&mut self, source_name: &str) -> Option<Credentials> {
        self.credentials.remove(source_name)
    }

    /// Checks if credentials exist for a source
    pub fn has_credentials(&self, source_name: &str) -> bool {
        self.credentials.contains_key(source_name)
    }

    /// Gets all sources with expired credentials
    pub fn get_expired_credentials(&self) -> Vec<String> {
        self.credentials
            .iter()
            .filter(|(_, creds)| creds.is_expired())
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Gets all sources needing credential refresh
    pub fn get_credentials_needing_refresh(&self) -> Vec<String> {
        self.credentials
            .iter()
            .filter(|(_, creds)| creds.needs_refresh())
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Updates credentials for a source
    pub fn update_credentials(&mut self, source_name: &str, credentials: Credentials) -> Result<(), String> {
        if !self.credentials.contains_key(source_name) {
            return Err(format!("No credentials found for source '{}'", source_name));
        }
        self.credentials.insert(source_name.to_string(), credentials);
        Ok(())
    }

    /// Validates credentials by checking expiry
    pub fn validate_credentials(&self, source_name: &str) -> Result<bool, String> {
        let creds = self
            .credentials
            .get(source_name)
            .ok_or_else(|| format!("No credentials found for source '{}'", source_name))?;

        if creds.is_expired() {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    /// Clears all expired credentials
    pub fn clear_expired(&mut self) -> Vec<String> {
        let expired: Vec<String> = self.get_expired_credentials();
        for source in &expired {
            self.credentials.remove(source);
        }
        expired
    }

    /// Gets count of stored credentials
    pub fn count(&self) -> usize {
        self.credentials.len()
    }
}

impl Default for AuthenticationStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Authentication manager that integrates with the config system
pub struct AuthenticationManager {
    store: AuthenticationStore,
}

impl AuthenticationManager {
    /// Creates a new authentication manager
    pub fn new() -> Self {
        Self {
            store: AuthenticationStore::new(),
        }
    }

    /// Adds bearer token authentication for a source
    pub fn add_bearer_token(
        &mut self,
        source_name: String,
        token: String,
        expires_in: Option<Duration>,
    ) {
        let expires_at = expires_in.map(|duration| SystemTime::now() + duration);
        let credentials = Credentials::BearerToken { token, expires_at };
        self.store.store_credentials(source_name, credentials);
    }

    /// Adds API key authentication for a source
    pub fn add_api_key(
        &mut self,
        source_name: String,
        key: String,
        header_name: Option<String>,
    ) {
        let credentials = Credentials::ApiKey { key, header_name };
        self.store.store_credentials(source_name, credentials);
    }

    /// Adds basic authentication for a source
    pub fn add_basic_auth(&mut self, source_name: String, username: String, password: String) {
        let credentials = Credentials::Basic { username, password };
        self.store.store_credentials(source_name, credentials);
    }

    /// Adds OAuth2 authentication for a source
    pub fn add_oauth2(
        &mut self,
        source_name: String,
        access_token: String,
        refresh_token: Option<String>,
        expires_in: Option<Duration>,
    ) {
        let expires_at = expires_in.map(|duration| SystemTime::now() + duration);
        let credentials = Credentials::OAuth2 {
            access_token,
            refresh_token,
            expires_at,
        };
        self.store.store_credentials(source_name, credentials);
    }

    /// Gets authentication headers for a source
    pub fn get_auth_header(&self, source_name: &str) -> Option<(String, String)> {
        self.store
            .get_credentials(source_name)
            .and_then(|creds| creds.to_auth_header())
    }

    /// Removes authentication for a source
    pub fn remove_auth(&mut self, source_name: &str) -> bool {
        self.store.remove_credentials(source_name).is_some()
    }

    /// Validates that authentication is valid for a source
    pub fn validate(&self, source_name: &str) -> Result<bool, String> {
        self.store.validate_credentials(source_name)
    }

    /// Gets sources that need credential refresh
    pub fn get_sources_needing_refresh(&self) -> Vec<String> {
        self.store.get_credentials_needing_refresh()
    }

    /// Cleans up expired credentials
    pub fn cleanup_expired(&mut self) -> Vec<String> {
        self.store.clear_expired()
    }

    /// Checks if a source has authentication configured
    pub fn has_auth(&self, source_name: &str) -> bool {
        self.store.has_credentials(source_name)
    }

    /// Gets the internal store (for serialization)
    pub fn get_store(&self) -> &AuthenticationStore {
        &self.store
    }

    /// Sets the internal store (for deserialization)
    pub fn set_store(&mut self, store: AuthenticationStore) {
        self.store = store;
    }
}

impl Default for AuthenticationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function for base64 encoding
fn base64_encode(input: String) -> String {
    // In production, use the base64 crate
    // This is a simplified placeholder
    format!("base64({})", input)
}

/// Secure credential storage helpers
pub mod secure_storage {
    use super::*;

    /// Encrypts credentials before storage
    /// In production, this would use platform-specific secure storage
    pub fn encrypt_credentials(_credentials: &Credentials) -> Result<Vec<u8>, String> {
        // Placeholder: In production use platform keychain/credential manager
        // - macOS: Keychain
        // - Windows: Credential Manager
        // - Linux: Secret Service API (libsecret)
        Err("Encryption not implemented yet".to_string())
    }

    /// Decrypts credentials from storage
    pub fn decrypt_credentials(_data: &[u8]) -> Result<Credentials, String> {
        // Placeholder: In production use platform keychain/credential manager
        Err("Decryption not implemented yet".to_string())
    }

    /// Stores credentials securely using platform APIs
    pub fn store_secure(
        _source_name: &str,
        _credentials: &Credentials,
    ) -> Result<(), String> {
        // Placeholder: Use platform secure storage
        Err("Secure storage not implemented yet".to_string())
    }

    /// Retrieves credentials securely from platform APIs
    pub fn retrieve_secure(_source_name: &str) -> Result<Credentials, String> {
        // Placeholder: Use platform secure storage
        Err("Secure storage not implemented yet".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bearer_token_creation() {
        let creds = Credentials::BearerToken {
            token: "test_token".to_string(),
            expires_at: None,
        };

        let header = creds.to_auth_header();
        assert!(header.is_some());
        let (name, value) = header.unwrap();
        assert_eq!(name, "Authorization");
        assert_eq!(value, "Bearer test_token");
    }

    #[test]
    fn test_api_key_creation() {
        let creds = Credentials::ApiKey {
            key: "test_key".to_string(),
            header_name: Some("X-Custom-Key".to_string()),
        };

        let header = creds.to_auth_header();
        assert!(header.is_some());
        let (name, value) = header.unwrap();
        assert_eq!(name, "X-Custom-Key");
        assert_eq!(value, "test_key");
    }

    #[test]
    fn test_credentials_expiry() {
        let expired = Credentials::BearerToken {
            token: "test".to_string(),
            expires_at: Some(SystemTime::now() - Duration::from_secs(3600)),
        };
        assert!(expired.is_expired());

        let valid = Credentials::BearerToken {
            token: "test".to_string(),
            expires_at: Some(SystemTime::now() + Duration::from_secs(3600)),
        };
        assert!(!valid.is_expired());
    }

    #[test]
    fn test_auth_store_operations() {
        let mut store = AuthenticationStore::new();

        let creds = Credentials::BearerToken {
            token: "test".to_string(),
            expires_at: None,
        };

        store.store_credentials("source1".to_string(), creds.clone());
        assert!(store.has_credentials("source1"));
        assert_eq!(store.count(), 1);

        let retrieved = store.get_credentials("source1");
        assert!(retrieved.is_some());

        store.remove_credentials("source1");
        assert!(!store.has_credentials("source1"));
        assert_eq!(store.count(), 0);
    }

    #[test]
    fn test_auth_manager() {
        let mut manager = AuthenticationManager::new();

        manager.add_bearer_token("source1".to_string(), "token123".to_string(), None);
        assert!(manager.has_auth("source1"));

        let header = manager.get_auth_header("source1");
        assert!(header.is_some());

        manager.remove_auth("source1");
        assert!(!manager.has_auth("source1"));
    }

    #[test]
    fn test_expired_cleanup() {
        let mut store = AuthenticationStore::new();

        let expired = Credentials::BearerToken {
            token: "expired".to_string(),
            expires_at: Some(SystemTime::now() - Duration::from_secs(3600)),
        };

        let valid = Credentials::BearerToken {
            token: "valid".to_string(),
            expires_at: Some(SystemTime::now() + Duration::from_secs(3600)),
        };

        store.store_credentials("expired_source".to_string(), expired);
        store.store_credentials("valid_source".to_string(), valid);

        assert_eq!(store.count(), 2);

        let cleaned = store.clear_expired();
        assert_eq!(cleaned.len(), 1);
        assert_eq!(cleaned[0], "expired_source");
        assert_eq!(store.count(), 1);
        assert!(store.has_credentials("valid_source"));
    }
}
