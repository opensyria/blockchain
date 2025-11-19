/// Encrypted wallet storage with AES-256-GCM and Argon2 password hashing
/// تخزين المحفظة المشفرة مع AES-256-GCM وتجزئة كلمة المرور Argon2

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::{anyhow, Context, Result};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use opensyria_core::crypto::{KeyPair, PublicKey};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Encrypted account with password-protected private key
/// حساب مشفر مع مفتاح خاص محمي بكلمة مرور
#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedAccount {
    pub name: String,
    pub address: PublicKey,
    /// Encrypted private key (AES-256-GCM)
    pub encrypted_key: Vec<u8>,
    /// AES-GCM nonce (96 bits / 12 bytes)
    pub nonce: [u8; 12],
    /// Argon2 password hash for verification
    pub password_hash: String,
    /// Salt for password hashing
    pub salt: String,
    pub created_at: u64,
    /// Encryption version for future upgrades
    pub version: u32,
}

impl EncryptedAccount {
    /// Create new encrypted account with password protection
    pub fn new(name: String, password: &str) -> Result<Self> {
        let keypair = KeyPair::generate();
        let private_key = keypair.private_key_bytes();
        
        Self::from_private_key(name, &private_key, password)
    }

    /// Create encrypted account from existing private key
    /// إنشاء حساب مشفر من مفتاح خاص موجود
    pub fn from_private_key(name: String, private_key: &[u8; 32], password: &str) -> Result<Self> {
        let keypair = KeyPair::from_bytes(private_key)?;
        
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Generate salt for password hashing
        let salt = SaltString::generate(&mut OsRng);
        
        // Hash password with Argon2
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow!("Failed to hash password: {}", e))?
            .to_string();

        // Derive encryption key from password
        let parsed_hash = PasswordHash::new(&password_hash)
            .map_err(|e| anyhow!("Failed to parse password hash: {}", e))?;
        let hash_bytes = parsed_hash.hash.unwrap();
        let encryption_key = &hash_bytes.as_bytes()[..32];

        // Generate random nonce for AES-GCM
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt private key
        let cipher = Aes256Gcm::new_from_slice(encryption_key)
            .map_err(|e| anyhow!("Failed to create cipher: {}", e))?;
        let encrypted_key = cipher
            .encrypt(nonce, private_key.as_ref())
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;

        Ok(Self {
            name,
            address: keypair.public_key(),
            encrypted_key,
            nonce: nonce_bytes,
            password_hash,
            salt: salt.to_string(),
            created_at: timestamp,
            version: 1,
        })
    }

    /// Decrypt and get keypair (requires correct password)
    /// فك التشفير والحصول على زوج المفاتيح (يتطلب كلمة مرور صحيحة)
    pub fn decrypt_keypair(&self, password: &str) -> Result<KeyPair> {
        // Verify password
        let parsed_hash = PasswordHash::new(&self.password_hash)
            .map_err(|e| anyhow!("Failed to parse password hash: {}", e))?;
        
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| anyhow!("Invalid password"))?;

        // Derive decryption key from password
        let hash_bytes = parsed_hash.hash.unwrap();
        let encryption_key = &hash_bytes.as_bytes()[..32];

        // Decrypt private key
        let cipher = Aes256Gcm::new_from_slice(encryption_key)
            .map_err(|e| anyhow!("Failed to create cipher: {}", e))?;
        let nonce = Nonce::from_slice(&self.nonce);

        let decrypted_key = cipher
            .decrypt(nonce, self.encrypted_key.as_ref())
            .map_err(|_| anyhow!("Decryption failed - invalid password or corrupted wallet"))?;

        let mut private_key = [0u8; 32];
        private_key.copy_from_slice(&decrypted_key);

        KeyPair::from_bytes(&private_key).map_err(|e| anyhow!("{}", e))
    }

    /// Verify password without decrypting
    /// التحقق من كلمة المرور دون فك التشفير
    pub fn verify_password(&self, password: &str) -> bool {
        if let Ok(parsed_hash) = PasswordHash::new(&self.password_hash) {
            Argon2::default()
                .verify_password(password.as_bytes(), &parsed_hash)
                .is_ok()
        } else {
            false
        }
    }

    /// Change password (requires current password)
    /// تغيير كلمة المرور (يتطلب كلمة المرور الحالية)
    pub fn change_password(&mut self, old_password: &str, new_password: &str) -> Result<()> {
        // Decrypt with old password to get private key
        let keypair = self.decrypt_keypair(old_password)?;
        let private_key = keypair.private_key_bytes();

        // Generate new salt and hash
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(new_password.as_bytes(), &salt)
            .map_err(|e| anyhow!("Failed to hash new password: {}", e))?
            .to_string();

        // Derive new encryption key
        let parsed_hash = PasswordHash::new(&password_hash)
            .map_err(|e| anyhow!("Failed to parse password hash: {}", e))?;
        let hash_bytes = parsed_hash.hash.unwrap();
        let encryption_key = &hash_bytes.as_bytes()[..32];

        // Generate new nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Re-encrypt with new password
        let cipher = Aes256Gcm::new_from_slice(encryption_key)
            .map_err(|e| anyhow!("Failed to create cipher: {}", e))?;
        let encrypted_key = cipher
            .encrypt(nonce, private_key.as_ref())
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;

        // Update stored values
        self.encrypted_key = encrypted_key;
        self.nonce = nonce_bytes;
        self.password_hash = password_hash;
        self.salt = salt.to_string();

        Ok(())
    }
}

/// Encrypted wallet storage manager
/// مدير تخزين المحفظة المشفرة
pub struct EncryptedWalletStorage {
    wallet_dir: PathBuf,
}

impl EncryptedWalletStorage {
    /// Initialize encrypted wallet storage in default directory
    pub fn new() -> Result<Self> {
        let wallet_dir = dirs::home_dir()
            .context("Could not find home directory")?
            .join(".opensyria")
            .join("wallet");

        fs::create_dir_all(&wallet_dir).context("Failed to create wallet directory")?;

        Ok(Self { wallet_dir })
    }

    /// Create encrypted wallet storage in custom directory
    pub fn with_path(path: PathBuf) -> Result<Self> {
        fs::create_dir_all(&path).context("Failed to create wallet directory")?;
        Ok(Self { wallet_dir: path })
    }

    /// Save encrypted account to disk
    /// حفظ الحساب المشفر على القرص
    pub fn save_account(&self, account: &EncryptedAccount) -> Result<()> {
        let filename = format!("{}.enc.json", account.name);
        let path = self.wallet_dir.join(filename);

        let json = serde_json::to_string_pretty(account)
            .context("Failed to serialize encrypted account")?;

        fs::write(&path, json).context("Failed to write encrypted account file")?;

        Ok(())
    }

    /// Load encrypted account from disk
    /// تحميل الحساب المشفر من القرص
    pub fn load_account(&self, name: &str) -> Result<EncryptedAccount> {
        let filename = format!("{}.enc.json", name);
        let path = self.wallet_dir.join(filename);

        let json = fs::read_to_string(&path)
            .context(format!("Encrypted account '{}' not found", name))?;

        let account: EncryptedAccount =
            serde_json::from_str(&json).context("Failed to deserialize encrypted account")?;

        Ok(account)
    }

    /// List all encrypted account names
    pub fn list_accounts(&self) -> Result<Vec<String>> {
        let mut accounts = Vec::new();

        for entry in fs::read_dir(&self.wallet_dir).context("Failed to read wallet directory")? {
            let entry = entry?;
            let path = entry.path();

            // Look for .enc.json files
            if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                if filename.ends_with(".enc.json") {
                    let name = filename
                        .strip_suffix(".enc.json")
                        .unwrap_or(filename)
                        .to_string();
                    accounts.push(name);
                }
            }
        }

        Ok(accounts)
    }

    /// Delete encrypted account from disk
    /// حذف الحساب المشفر من القرص
    pub fn delete_account(&self, name: &str) -> Result<()> {
        let filename = format!("{}.enc.json", name);
        let path = self.wallet_dir.join(filename);

        fs::remove_file(&path)
            .context(format!("Failed to delete encrypted account '{}'", name))?;

        Ok(())
    }

    /// Check if account exists
    pub fn account_exists(&self, name: &str) -> bool {
        let filename = format!("{}.enc.json", name);
        let path = self.wallet_dir.join(filename);
        path.exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_encrypted_account_creation() {
        let account = EncryptedAccount::new("test_account".to_string(), "strong_password_123")
            .expect("Failed to create encrypted account");

        assert_eq!(account.name, "test_account");
        assert_eq!(account.version, 1);
        assert_eq!(account.nonce.len(), 12);
        assert!(!account.encrypted_key.is_empty());
    }

    #[test]
    fn test_decrypt_with_correct_password() {
        let password = "my_secure_password_456";
        let account = EncryptedAccount::new("alice".to_string(), password)
            .expect("Failed to create account");

        let keypair = account
            .decrypt_keypair(password)
            .expect("Failed to decrypt with correct password");

        assert_eq!(keypair.public_key(), account.address);
    }

    #[test]
    fn test_decrypt_with_wrong_password() {
        let account = EncryptedAccount::new("bob".to_string(), "correct_password")
            .expect("Failed to create account");

        let result = account.decrypt_keypair("wrong_password");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid password"));
    }

    #[test]
    fn test_verify_password() {
        let correct_password = "test_password_789";
        let account = EncryptedAccount::new("carol".to_string(), correct_password)
            .expect("Failed to create account");

        assert!(account.verify_password(correct_password));
        assert!(!account.verify_password("wrong_password"));
    }

    #[test]
    fn test_change_password() {
        let old_password = "old_password_123";
        let new_password = "new_password_456";

        let mut account = EncryptedAccount::new("dave".to_string(), old_password)
            .expect("Failed to create account");

        let original_address = account.address;

        // Change password
        account
            .change_password(old_password, new_password)
            .expect("Failed to change password");

        // Old password should no longer work
        assert!(!account.verify_password(old_password));
        assert!(account.decrypt_keypair(old_password).is_err());

        // New password should work
        assert!(account.verify_password(new_password));
        let keypair = account
            .decrypt_keypair(new_password)
            .expect("Failed to decrypt with new password");

        // Address should remain the same
        assert_eq!(keypair.public_key(), original_address);
    }

    #[test]
    fn test_encrypted_storage_save_and_load() {
        let dir = tempdir().unwrap();
        let storage = EncryptedWalletStorage::with_path(dir.path().to_path_buf()).unwrap();

        let password = "storage_test_password";
        let account = EncryptedAccount::new("test_user".to_string(), password).unwrap();

        // Save account
        storage.save_account(&account).expect("Failed to save account");

        // Load account
        let loaded = storage
            .load_account("test_user")
            .expect("Failed to load account");

        assert_eq!(loaded.name, account.name);
        assert_eq!(loaded.address, account.address);

        // Decrypt loaded account
        let keypair = loaded
            .decrypt_keypair(password)
            .expect("Failed to decrypt loaded account");

        assert_eq!(keypair.public_key(), account.address);
    }

    #[test]
    fn test_list_encrypted_accounts() {
        let dir = tempdir().unwrap();
        let storage = EncryptedWalletStorage::with_path(dir.path().to_path_buf()).unwrap();

        // Create multiple accounts
        let account1 = EncryptedAccount::new("alice".to_string(), "password1").unwrap();
        let account2 = EncryptedAccount::new("bob".to_string(), "password2").unwrap();

        storage.save_account(&account1).unwrap();
        storage.save_account(&account2).unwrap();

        let accounts = storage.list_accounts().unwrap();
        assert_eq!(accounts.len(), 2);
        assert!(accounts.contains(&"alice".to_string()));
        assert!(accounts.contains(&"bob".to_string()));
    }

    #[test]
    fn test_delete_encrypted_account() {
        let dir = tempdir().unwrap();
        let storage = EncryptedWalletStorage::with_path(dir.path().to_path_buf()).unwrap();

        let account = EncryptedAccount::new("to_delete".to_string(), "password").unwrap();
        storage.save_account(&account).unwrap();

        assert!(storage.account_exists("to_delete"));

        storage.delete_account("to_delete").unwrap();

        assert!(!storage.account_exists("to_delete"));
    }
}
