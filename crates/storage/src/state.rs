use crate::StorageError;
use opensyria_core::crypto::PublicKey;
use opensyria_core::multisig::MultisigAccount;
use rocksdb::{Options, DB};
use std::collections::HashMap;
use std::path::PathBuf;

/// State storage for account balances and metadata
pub struct StateStorage {
    db: DB,
}

impl StateStorage {
    /// Open state storage at path
    pub fn open(path: PathBuf) -> Result<Self, StorageError> {
        let mut opts = Options::default();
        opts.create_if_missing(true);

        let db = DB::open(&opts, path)?;

        Ok(Self { db })
    }

    /// Get account balance
    pub fn get_balance(&self, address: &PublicKey) -> Result<u64, StorageError> {
        let key = Self::balance_key(address);

        match self.db.get(&key)? {
            Some(data) => {
                let bytes: [u8; 8] = data.try_into().map_err(|_| StorageError::InvalidChain)?;
                Ok(u64::from_le_bytes(bytes))
            }
            None => Ok(0), // No balance = 0
        }
    }

    /// Set account balance
    pub fn set_balance(&self, address: &PublicKey, amount: u64) -> Result<(), StorageError> {
        let key = Self::balance_key(address);
        self.db.put(&key, amount.to_le_bytes())?;
        Ok(())
    }

    /// Add to account balance
    pub fn add_balance(&self, address: &PublicKey, amount: u64) -> Result<(), StorageError> {
        let current = self.get_balance(address)?;
        let new_balance = current.saturating_add(amount);
        self.set_balance(address, new_balance)
    }

    /// Subtract from account balance (returns error if insufficient)
    pub fn sub_balance(&self, address: &PublicKey, amount: u64) -> Result<(), StorageError> {
        let current = self.get_balance(address)?;

        if current < amount {
            return Err(StorageError::InvalidChain); // Insufficient balance
        }

        let new_balance = current - amount;
        self.set_balance(address, new_balance)
    }

    /// Transfer balance between accounts
    pub fn transfer(
        &self,
        from: &PublicKey,
        to: &PublicKey,
        amount: u64,
    ) -> Result<(), StorageError> {
        self.sub_balance(from, amount)?;
        self.add_balance(to, amount)?;
        Ok(())
    }

    /// Get account nonce (transaction counter)
    pub fn get_nonce(&self, address: &PublicKey) -> Result<u64, StorageError> {
        let key = Self::nonce_key(address);

        match self.db.get(&key)? {
            Some(data) => {
                let bytes: [u8; 8] = data.try_into().map_err(|_| StorageError::InvalidChain)?;
                Ok(u64::from_le_bytes(bytes))
            }
            None => Ok(0),
        }
    }

    /// Set account nonce
    pub fn set_nonce(&self, address: &PublicKey, nonce: u64) -> Result<(), StorageError> {
        let key = Self::nonce_key(address);
        self.db.put(&key, nonce.to_le_bytes())?;
        Ok(())
    }

    /// Increment account nonce
    pub fn increment_nonce(&self, address: &PublicKey) -> Result<(), StorageError> {
        let current = self.get_nonce(address)?;
        self.set_nonce(address, current + 1)
    }

    /// Get all account balances (for debugging/inspection)
    pub fn get_all_balances(&self) -> Result<HashMap<PublicKey, u64>, StorageError> {
        let mut balances = HashMap::new();
        let prefix = b"balance_";

        let iter = self.db.prefix_iterator(prefix);

        for item in iter {
            let (key, value) = item?;

            // Skip if not a balance key
            if !key.starts_with(prefix) {
                break;
            }

            // Extract public key from key (skip "balance_" prefix)
            if key.len() == prefix.len() + 32 {
                let mut pk_bytes = [0u8; 32];
                pk_bytes.copy_from_slice(&key[prefix.len()..]);
                let pk = PublicKey(pk_bytes);

                // Parse balance
                if value.len() == 8 {
                    let mut balance_bytes = [0u8; 8];
                    balance_bytes.copy_from_slice(&value);
                    let balance = u64::from_le_bytes(balance_bytes);
                    balances.insert(pk, balance);
                }
            }
        }

        Ok(balances)
    }

    // Helper functions
    fn balance_key(address: &PublicKey) -> Vec<u8> {
        let mut key = Vec::with_capacity(40);
        key.extend_from_slice(b"balance_");
        key.extend_from_slice(&address.0);
        key
    }

    fn nonce_key(address: &PublicKey) -> Vec<u8> {
        let mut key = Vec::with_capacity(38);
        key.extend_from_slice(b"nonce_");
        key.extend_from_slice(&address.0);
        key
    }

    fn multisig_key(address: &PublicKey) -> Vec<u8> {
        let mut key = Vec::with_capacity(48);
        key.extend_from_slice(b"multisig_");
        key.extend_from_slice(&address.0);
        key
    }

    /// Store multisig account configuration
    pub fn store_multisig_account(&self, account: &MultisigAccount) -> Result<(), StorageError> {
        let address = account.address();
        let key = Self::multisig_key(&address);

        // Serialize multisig account using bincode
        let serialized = bincode::serialize(account).map_err(|_e| StorageError::InvalidChain)?;

        self.db.put(&key, &serialized)?;
        Ok(())
    }

    /// Get multisig account configuration
    pub fn get_multisig_account(
        &self,
        address: &PublicKey,
    ) -> Result<Option<MultisigAccount>, StorageError> {
        let key = Self::multisig_key(address);

        match self.db.get(&key)? {
            Some(data) => {
                let account: MultisigAccount =
                    bincode::deserialize(&data).map_err(|_| StorageError::InvalidChain)?;
                Ok(Some(account))
            }
            None => Ok(None),
        }
    }

    /// Check if an address is a multisig account
    pub fn is_multisig_account(&self, address: &PublicKey) -> Result<bool, StorageError> {
        let key = Self::multisig_key(address);
        Ok(self.db.get(&key)?.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opensyria_core::crypto::KeyPair;
    use tempfile::tempdir;

    #[test]
    fn test_balance_operations() {
        let dir = tempdir().unwrap();
        let storage = StateStorage::open(dir.path().to_path_buf()).unwrap();

        let kp = KeyPair::generate();
        let addr = kp.public_key();

        // Initial balance is 0
        assert_eq!(storage.get_balance(&addr).unwrap(), 0);

        // Set balance
        storage.set_balance(&addr, 1_000_000).unwrap();
        assert_eq!(storage.get_balance(&addr).unwrap(), 1_000_000);

        // Add balance
        storage.add_balance(&addr, 500_000).unwrap();
        assert_eq!(storage.get_balance(&addr).unwrap(), 1_500_000);

        // Subtract balance
        storage.sub_balance(&addr, 300_000).unwrap();
        assert_eq!(storage.get_balance(&addr).unwrap(), 1_200_000);

        // Insufficient balance should error
        assert!(storage.sub_balance(&addr, 2_000_000).is_err());
    }

    #[test]
    fn test_transfer() {
        let dir = tempdir().unwrap();
        let storage = StateStorage::open(dir.path().to_path_buf()).unwrap();

        let alice = KeyPair::generate().public_key();
        let bob = KeyPair::generate().public_key();

        // Give Alice initial balance
        storage.set_balance(&alice, 1_000_000).unwrap();

        // Transfer to Bob
        storage.transfer(&alice, &bob, 300_000).unwrap();

        assert_eq!(storage.get_balance(&alice).unwrap(), 700_000);
        assert_eq!(storage.get_balance(&bob).unwrap(), 300_000);
    }

    #[test]
    fn test_nonce_operations() {
        let dir = tempdir().unwrap();
        let storage = StateStorage::open(dir.path().to_path_buf()).unwrap();

        let kp = KeyPair::generate();
        let addr = kp.public_key();

        assert_eq!(storage.get_nonce(&addr).unwrap(), 0);

        storage.set_nonce(&addr, 5).unwrap();
        assert_eq!(storage.get_nonce(&addr).unwrap(), 5);

        storage.increment_nonce(&addr).unwrap();
        assert_eq!(storage.get_nonce(&addr).unwrap(), 6);
    }

    #[test]
    fn test_get_all_balances() {
        let dir = tempdir().unwrap();
        let storage = StateStorage::open(dir.path().to_path_buf()).unwrap();

        let alice = KeyPair::generate().public_key();
        let bob = KeyPair::generate().public_key();

        storage.set_balance(&alice, 1_000_000).unwrap();
        storage.set_balance(&bob, 2_000_000).unwrap();

        let balances = storage.get_all_balances().unwrap();

        assert_eq!(balances.len(), 2);
        assert_eq!(balances.get(&alice), Some(&1_000_000));
        assert_eq!(balances.get(&bob), Some(&2_000_000));
    }
}
