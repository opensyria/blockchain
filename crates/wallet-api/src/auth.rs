/// API Key authentication middleware
/// مصادقة مفتاح API

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{Json, Response},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// API Key structure
/// بنية مفتاح API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    /// Unique key ID
    pub id: String,
    /// SHA-256 hash of the API key
    pub key_hash: String,
    /// Human-readable name/description
    pub name: String,
    /// Permissions granted to this key
    pub permissions: Vec<Permission>,
    /// Key creation timestamp
    pub created_at: u64,
    /// Optional expiration timestamp
    pub expires_at: Option<u64>,
    /// Whether key is currently active
    pub active: bool,
}

/// API permissions
/// أذونات API
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    /// Submit transactions
    SubmitTransaction,
    /// Query balances
    ReadBalance,
    /// Query blockchain info
    ReadBlockchain,
    /// Access mempool
    ReadMempool,
    /// Full admin access
    Admin,
}

/// API key manager
/// مدير مفاتيح API
#[derive(Clone)]
pub struct ApiKeyManager {
    keys: Arc<RwLock<HashMap<String, ApiKey>>>,
}

impl ApiKeyManager {
    /// Create new API key manager
    pub fn new() -> Self {
        Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generate a new API key
    /// إنشاء مفتاح API جديد
    pub async fn generate_key(
        &self,
        name: String,
        permissions: Vec<Permission>,
        expires_at: Option<u64>,
    ) -> (String, String) {
        use rand::Rng;

        // Generate random 32-byte key
        let mut rng = rand::thread_rng();
        let key_bytes: [u8; 32] = rng.gen();
        let api_key = format!("osy_{}", hex::encode(key_bytes));

        // SECURITY: Hash the key with Argon2 (not SHA-256!) for storage
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let key_hash = argon2
            .hash_password(api_key.as_bytes(), &salt)
            .expect("Failed to hash API key")
            .to_string();

        // Generate unique ID
        let id = format!("key_{}", hex::encode(&key_bytes[..8]));

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_secs();

        let api_key_entry = ApiKey {
            id: id.clone(),
            key_hash,
            name,
            permissions,
            created_at: timestamp,
            expires_at,
            active: true,
        };

        let mut keys = self.keys.write().await;
        keys.insert(id.clone(), api_key_entry);

        (id, api_key) // Return ID and raw key (only time raw key is visible!)
    }

    /// Verify an API key and return associated metadata
    /// التحقق من مفتاح API وإرجاع البيانات المرتبطة
    pub async fn verify_key(&self, api_key: &str) -> Option<ApiKey> {
        let keys = self.keys.read().await;

        // SECURITY: Use constant-time Argon2 verification (prevents timing attacks)
        for entry in keys.values() {
            if !entry.active {
                continue;
            }

            if let Ok(parsed_hash) = PasswordHash::new(&entry.key_hash) {
                if Argon2::default()
                    .verify_password(api_key.as_bytes(), &parsed_hash)
                    .is_ok()
                {
                    // Check expiration
                    if let Some(expires_at) = entry.expires_at {
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                            .as_secs();

                        if now > expires_at {
                            return None; // Key expired
                        }
                    }

                    return Some(entry.clone());
                }
            }
        }

        None
    }

    /// Revoke an API key
    /// إبطال مفتاح API
    pub async fn revoke_key(&self, key_id: &str) -> bool {
        let mut keys = self.keys.write().await;
        if let Some(key) = keys.get_mut(key_id) {
            key.active = false;
            true
        } else {
            false
        }
    }

    /// List all API keys (without showing actual keys)
    /// قائمة بجميع مفاتيح API (دون إظهار المفاتيح الفعلية)
    pub async fn list_keys(&self) -> Vec<ApiKey> {
        let keys = self.keys.read().await;
        keys.values().cloned().collect()
    }

    /// Check if a key has a specific permission
    /// التحقق من أن المفتاح لديه إذن معين
    pub fn has_permission(key: &ApiKey, permission: &Permission) -> bool {
        key.permissions.contains(permission) || key.permissions.contains(&Permission::Admin)
    }
}

/// Error response for authentication failures
#[derive(Debug, Serialize)]
pub struct AuthError {
    pub error: String,
}

/// Extract and validate API key from request headers
/// استخراج والتحقق من صحة مفتاح API من رؤوس الطلب
pub async fn auth_middleware(
    State(key_manager): State<Arc<ApiKeyManager>>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<AuthError>)> {
    // Extract API key from Authorization header
    let api_key = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(AuthError {
                    error: "Missing or invalid Authorization header".to_string(),
                }),
            )
        })?;

    // Verify API key
    let key_entry = key_manager.verify_key(api_key).await.ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(AuthError {
                error: "Invalid or expired API key".to_string(),
            }),
        )
    })?;

    // Check if key is active
    if !key_entry.active {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(AuthError {
                error: "API key has been revoked".to_string(),
            }),
        ));
    }

    // TODO: Add permission checking based on endpoint
    // For now, just verify the key exists

    Ok(next.run(request).await)
}

/// Create a default admin API key for initial setup
/// إنشاء مفتاح API مسؤول افتراضي للإعداد الأولي
pub async fn create_default_admin_key(manager: &ApiKeyManager) -> (String, String) {
    manager
        .generate_key(
            "Default Admin Key".to_string(),
            vec![Permission::Admin],
            None, // Never expires
        )
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_and_verify_key() {
        let manager = ApiKeyManager::new();

        let (key_id, api_key) = manager
            .generate_key(
                "test-key".to_string(),
                vec![Permission::SubmitTransaction],
                None,
            )
            .await;

        assert!(api_key.starts_with("osy_"));

        // Verify the key
        let verified = manager.verify_key(&api_key).await;
        assert!(verified.is_some());

        let key_entry = verified.unwrap();
        assert_eq!(key_entry.id, key_id);
        assert_eq!(key_entry.name, "test-key");
        assert!(key_entry.active);
    }

    #[tokio::test]
    async fn test_invalid_key_rejected() {
        let manager = ApiKeyManager::new();

        let verified = manager.verify_key("osy_invalid_key_12345").await;
        assert!(verified.is_none());
    }

    #[tokio::test]
    async fn test_revoke_key() {
        let manager = ApiKeyManager::new();

        let (key_id, api_key) = manager
            .generate_key("test-key".to_string(), vec![Permission::ReadBalance], None)
            .await;

        // Key should work initially
        assert!(manager.verify_key(&api_key).await.is_some());

        // Revoke the key
        assert!(manager.revoke_key(&key_id).await);

        // Key should no longer work
        assert!(manager.verify_key(&api_key).await.is_none());
    }

    #[tokio::test]
    async fn test_expired_key() {
        let manager = ApiKeyManager::new();

        // Create key that expired 1 second ago
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let expired = now - 1;

        let (_key_id, api_key) = manager
            .generate_key(
                "expired-key".to_string(),
                vec![Permission::ReadBalance],
                Some(expired),
            )
            .await;

        // Expired key should be rejected
        assert!(manager.verify_key(&api_key).await.is_none());
    }

    #[tokio::test]
    async fn test_list_keys() {
        let manager = ApiKeyManager::new();

        manager
            .generate_key("key1".to_string(), vec![Permission::ReadBalance], None)
            .await;
        manager
            .generate_key("key2".to_string(), vec![Permission::SubmitTransaction], None)
            .await;

        let keys = manager.list_keys().await;
        assert_eq!(keys.len(), 2);
    }

    #[tokio::test]
    async fn test_has_permission() {
        let key = ApiKey {
            id: "test".to_string(),
            key_hash: "hash".to_string(),
            name: "test".to_string(),
            permissions: vec![Permission::ReadBalance, Permission::ReadBlockchain],
            created_at: 0,
            expires_at: None,
            active: true,
        };

        assert!(ApiKeyManager::has_permission(&key, &Permission::ReadBalance));
        assert!(ApiKeyManager::has_permission(
            &key,
            &Permission::ReadBlockchain
        ));
        assert!(!ApiKeyManager::has_permission(
            &key,
            &Permission::SubmitTransaction
        ));
    }

    #[tokio::test]
    async fn test_admin_has_all_permissions() {
        let admin_key = ApiKey {
            id: "admin".to_string(),
            key_hash: "hash".to_string(),
            name: "admin".to_string(),
            permissions: vec![Permission::Admin],
            created_at: 0,
            expires_at: None,
            active: true,
        };

        // Admin should have all permissions
        assert!(ApiKeyManager::has_permission(
            &admin_key,
            &Permission::ReadBalance
        ));
        assert!(ApiKeyManager::has_permission(
            &admin_key,
            &Permission::SubmitTransaction
        ));
        assert!(ApiKeyManager::has_permission(
            &admin_key,
            &Permission::ReadBlockchain
        ));
    }
}
