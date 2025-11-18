use anyhow::{Context, Result};
use opensyria_core::crypto::{KeyPair, PublicKey};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Wallet account with keypair and metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub name: String,
    pub address: PublicKey,
    #[serde(with = "hex_serde")]
    private_key: [u8; 32],
    pub created_at: u64,
}

mod hex_serde {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8; 32], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let bytes = hex::decode(&s).map_err(serde::de::Error::custom)?;
        if bytes.len() != 32 {
            return Err(serde::de::Error::custom("Invalid key length"));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(arr)
    }
}

impl Account {
    /// Create new account with random keypair
    pub fn new(name: String) -> Self {
        let keypair = KeyPair::generate();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            name,
            address: keypair.public_key(),
            private_key: keypair.private_key_bytes(),
            created_at: timestamp,
        }
    }

    /// Get keypair for signing
    pub fn keypair(&self) -> Result<KeyPair> {
        KeyPair::from_bytes(&self.private_key).map_err(|e| anyhow::anyhow!("{}", e))
    }
}

/// Wallet storage manager
pub struct WalletStorage {
    wallet_dir: PathBuf,
}

impl WalletStorage {
    /// Initialize wallet storage in default directory
    pub fn new() -> Result<Self> {
        let wallet_dir = dirs::home_dir()
            .context("Could not find home directory")?
            .join(".opensyria")
            .join("wallet");

        fs::create_dir_all(&wallet_dir).context("Failed to create wallet directory")?;

        Ok(Self { wallet_dir })
    }

    /// Create wallet storage in custom directory
    pub fn with_path(path: PathBuf) -> Result<Self> {
        fs::create_dir_all(&path).context("Failed to create wallet directory")?;
        Ok(Self { wallet_dir: path })
    }

    /// Save account to disk
    pub fn save_account(&self, account: &Account) -> Result<()> {
        let filename = format!("{}.json", account.name);
        let path = self.wallet_dir.join(filename);

        let json = serde_json::to_string_pretty(account)
            .context("Failed to serialize account")?;

        fs::write(&path, json).context("Failed to write account file")?;

        Ok(())
    }

    /// Load account from disk
    pub fn load_account(&self, name: &str) -> Result<Account> {
        let filename = format!("{}.json", name);
        let path = self.wallet_dir.join(filename);

        let json = fs::read_to_string(&path)
            .context(format!("Account '{}' not found", name))?;

        let account: Account = serde_json::from_str(&json)
            .context("Failed to deserialize account")?;

        Ok(account)
    }

    /// List all account names
    pub fn list_accounts(&self) -> Result<Vec<String>> {
        let mut accounts = Vec::new();

        for entry in fs::read_dir(&self.wallet_dir).context("Failed to read wallet directory")? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    accounts.push(name.to_string());
                }
            }
        }

        Ok(accounts)
    }

    /// Delete account from disk
    pub fn delete_account(&self, name: &str) -> Result<()> {
        let filename = format!("{}.json", name);
        let path = self.wallet_dir.join(filename);

        fs::remove_file(&path)
            .context(format!("Failed to delete account '{}'", name))?;

        Ok(())
    }
}
