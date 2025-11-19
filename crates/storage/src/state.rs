use crate::StorageError;
use opensyria_core::crypto::PublicKey;
use opensyria_core::multisig::MultisigAccount;
use opensyria_core::Transaction;
use rocksdb::{Options, WriteBatch, DB};
use std::collections::HashMap;
use std::path::PathBuf;

/// State storage for account balances and metadata
/// تخزين حالة أرصدة الحسابات والبيانات الوصفية
pub struct StateStorage {
    db: DB,
}

const TOTAL_SUPPLY_KEY: &[u8] = b"total_supply";

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
        // Use checked_add to detect overflow instead of silent saturation
        let new_balance = current
            .checked_add(amount)
            .ok_or(StorageError::BalanceOverflow)?;
        self.set_balance(address, new_balance)
    }

    /// Get total supply across all accounts
    pub fn get_total_supply(&self) -> Result<u64, StorageError> {
        match self.db.get(TOTAL_SUPPLY_KEY)? {
            Some(data) => {
                let bytes: [u8; 8] = data.try_into().map_err(|_| StorageError::InvalidChain)?;
                Ok(u64::from_le_bytes(bytes))
            }
            None => Ok(0),
        }
    }

    /// Set total supply
    fn set_total_supply(&self, supply: u64) -> Result<(), StorageError> {
        self.db.put(TOTAL_SUPPLY_KEY, supply.to_le_bytes())?;
        Ok(())
    }

    /// Increase total supply (for coinbase/minting)
    pub fn increase_supply(&self, amount: u64) -> Result<(), StorageError> {
        use opensyria_core::MAX_SUPPLY;
        
        let current = self.get_total_supply()?;
        let new_supply = current
            .checked_add(amount)
            .ok_or(StorageError::BalanceOverflow)?;
        
        if new_supply > MAX_SUPPLY {
            return Err(StorageError::InvalidChain); // Exceeds maximum supply
        }
        
        self.set_total_supply(new_supply)
    }

    /// Decrease total supply (for coin burns)
    pub fn decrease_supply(&self, amount: u64) -> Result<(), StorageError> {
        let current = self.get_total_supply()?;
        if current < amount {
            return Err(StorageError::InsufficientBalance);
        }
        self.set_total_supply(current - amount)
    }

    /// Verify total supply matches sum of all balances (for validation)
    pub fn verify_total_supply(&self) -> Result<bool, StorageError> {
        let recorded_supply = self.get_total_supply()?;
        let balances = self.get_all_balances()?;
        let computed_supply: u64 = balances.values().sum();
        
        Ok(recorded_supply == computed_supply)
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

    /// Get multisig nonce
    pub fn get_multisig_nonce(&self, address: &PublicKey) -> Result<u64, StorageError> {
        // Multisig accounts use the same nonce mechanism as regular accounts
        self.get_nonce(address)
    }

    /// Increment multisig nonce
    pub fn increment_multisig_nonce(&self, address: &PublicKey) -> Result<(), StorageError> {
        self.increment_nonce(address)
    }

    /// Store partial multisig transaction (for incremental signing)
    pub fn store_partial_multisig(
        &self,
        tx_hash: &[u8; 32],
        transaction_data: &[u8],
    ) -> Result<(), StorageError> {
        let key = Self::partial_multisig_key(tx_hash);
        self.db.put(&key, transaction_data)?;
        Ok(())
    }

    /// Get partial multisig transaction
    pub fn get_partial_multisig(&self, tx_hash: &[u8; 32]) -> Result<Option<Vec<u8>>, StorageError> {
        let key = Self::partial_multisig_key(tx_hash);
        Ok(self.db.get(&key)?)
    }

    /// Delete partial multisig transaction (after execution or expiry)
    pub fn delete_partial_multisig(&self, tx_hash: &[u8; 32]) -> Result<(), StorageError> {
        let key = Self::partial_multisig_key(tx_hash);
        self.db.delete(&key)?;
        Ok(())
    }

    fn partial_multisig_key(tx_hash: &[u8; 32]) -> Vec<u8> {
        let mut key = Vec::with_capacity(48);
        key.extend_from_slice(b"partial_multisig_");
        key.extend_from_slice(tx_hash);
        key
    }

    /// Apply block transactions atomically (all-or-nothing)
    /// تطبيق معاملات الكتلة بشكل ذري (كل شيء أو لا شيء)
    pub fn apply_block_atomic(&self, transactions: &[Transaction]) -> Result<(), StorageError> {
        let mut batch = WriteBatch::default();
        
        // Track balance/nonce changes in memory before batching
        let mut balance_changes: HashMap<PublicKey, i128> = HashMap::new();
        let mut nonce_changes: HashMap<PublicKey, u64> = HashMap::new();
        let mut supply_increase: u64 = 0;

        // Calculate all state changes
        for tx in transactions {
            // Skip coinbase transactions (miner rewards)
            if tx.is_coinbase() {
                *balance_changes.entry(tx.to).or_insert(0) += tx.amount as i128;
                supply_increase = supply_increase
                    .checked_add(tx.amount)
                    .ok_or(StorageError::BalanceOverflow)?;
                continue;
            }

            // Regular transactions: deduct from sender, add to receiver
            let total_debit = tx
                .amount
                .checked_add(tx.fee)
                .ok_or(StorageError::BalanceOverflow)?;

            *balance_changes.entry(tx.from).or_insert(0) -= total_debit as i128;
            *balance_changes.entry(tx.to).or_insert(0) += tx.amount as i128;
            
            // Track nonce increment
            *nonce_changes.entry(tx.from).or_insert(0) += 1;
        }

        // Check total supply will not exceed MAX_SUPPLY
        if supply_increase > 0 {
            use opensyria_core::MAX_SUPPLY;
            let current_supply = self.get_total_supply()?;
            let new_supply = current_supply
                .checked_add(supply_increase)
                .ok_or(StorageError::BalanceOverflow)?;
            if new_supply > MAX_SUPPLY {
                return Err(StorageError::InvalidChain);
            }
        }

        // Validate all balances are sufficient
        for (address, change) in &balance_changes {
            let current_balance = self.get_balance(address)?;
            let new_balance = (current_balance as i128) + change;
            
            if new_balance < 0 {
                return Err(StorageError::InsufficientBalance);
            }
        }

        // Apply balance changes to batch
        for (address, change) in balance_changes {
            let current_balance = self.get_balance(&address)?;
            let new_balance = ((current_balance as i128) + change) as u64;
            
            let key = Self::balance_key(&address);
            batch.put(&key, new_balance.to_le_bytes());
        }

        // Apply nonce changes to batch
        for (address, increment) in nonce_changes {
            let current_nonce = self.get_nonce(&address)?;
            let new_nonce = current_nonce + increment;
            
            let key = Self::nonce_key(&address);
            batch.put(&key, new_nonce.to_le_bytes());
        }

        // Update total supply if there were coinbase transactions
        if supply_increase > 0 {
            let current_supply = self.get_total_supply()?;
            let new_supply = current_supply + supply_increase;
            batch.put(TOTAL_SUPPLY_KEY, new_supply.to_le_bytes());
        }

        // Atomic commit - ALL or NOTHING
        self.db.write(batch)?;

        Ok(())
    }

    /// Revert block transactions atomically (for chain reorgs)
    /// عكس معاملات الكتلة بشكل ذري (لإعادة تنظيم السلسلة)
    pub fn revert_block_atomic(&self, transactions: &[Transaction]) -> Result<(), StorageError> {
        let mut batch = WriteBatch::default();

        // Reverse all operations in reverse order
        for tx in transactions.iter().rev() {
            // Skip coinbase transactions
            if tx.is_coinbase() {
                let receiver_balance = self.get_balance(&tx.to)?;
                if receiver_balance < tx.amount {
                    return Err(StorageError::InsufficientBalance);
                }
                let new_receiver_balance = receiver_balance - tx.amount;

                let receiver_key = Self::balance_key(&tx.to);
                batch.put(&receiver_key, new_receiver_balance.to_le_bytes());
                continue;
            }

            // Return funds to sender
            let sender_balance = self.get_balance(&tx.from)?;
            let total_credit = tx
                .amount
                .checked_add(tx.fee)
                .ok_or(StorageError::BalanceOverflow)?;
            let new_sender_balance = sender_balance
                .checked_add(total_credit)
                .ok_or(StorageError::BalanceOverflow)?;

            let sender_key = Self::balance_key(&tx.from);
            batch.put(&sender_key, new_sender_balance.to_le_bytes());

            // Deduct from receiver
            let receiver_balance = self.get_balance(&tx.to)?;
            if receiver_balance < tx.amount {
                return Err(StorageError::InsufficientBalance);
            }
            let new_receiver_balance = receiver_balance - tx.amount;

            let receiver_key = Self::balance_key(&tx.to);
            batch.put(&receiver_key, new_receiver_balance.to_le_bytes());

            // Decrement sender nonce
            let sender_nonce = self.get_nonce(&tx.from)?;
            if sender_nonce == 0 {
                return Err(StorageError::InvalidChain);
            }
            let nonce_key = Self::nonce_key(&tx.from);
            batch.put(&nonce_key, (sender_nonce - 1).to_le_bytes());
        }

        // Atomic commit
        self.db.write(batch)?;

        Ok(())
    }

    /// Compact the database to reclaim disk space
    /// ضغط قاعدة البيانات لاستعادة مساحة القرص
    pub fn compact_database(&self) -> Result<(), StorageError> {
        self.db.compact_range::<&[u8], &[u8]>(None, None);
        Ok(())
    }

    /// Prune zero-balance accounts older than specified height
    /// حذف الحسابات ذات الرصيد الصفري
    pub fn prune_zero_balances(&self) -> Result<usize, StorageError> {
        let mut batch = WriteBatch::default();
        let mut pruned_count = 0;
        
        let balances = self.get_all_balances()?;
        for (address, balance) in balances {
            if balance == 0 {
                let key = Self::balance_key(&address);
                batch.delete(&key);
                pruned_count += 1;
            }
        }
        
        if pruned_count > 0 {
            self.db.write(batch)?;
        }
        
        Ok(pruned_count)
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
        assert_eq!(*balances.get(&alice).unwrap(), 1_000_000);
        assert_eq!(*balances.get(&bob).unwrap(), 2_000_000);
    }

    #[test]
    fn test_atomic_block_apply() {
        let dir = tempdir().unwrap();
        let storage = StateStorage::open(dir.path().to_path_buf()).unwrap();

        let alice_kp = KeyPair::generate();
        let bob_kp = KeyPair::generate();
        let charlie_kp = KeyPair::generate();

        let alice = alice_kp.public_key();
        let bob = bob_kp.public_key();
        let charlie = charlie_kp.public_key();

        // Give Alice initial balance
        storage.set_balance(&alice, 2_000_000).unwrap();

        // Create transactions: (from, to, amount, fee, nonce)
        let tx1 = Transaction::new(alice, bob, 1_000_000, 500, 0); // 500 fee, nonce 0
        let tx2 = Transaction::new(alice, charlie, 500_000, 500, 1); // 500 fee, nonce 1

        let transactions = vec![tx1, tx2];

        // Apply atomically
        storage.apply_block_atomic(&transactions).unwrap();

        // Verify all changes applied
        // Alice: 2M - (1M + 500) - (500K + 500) = 499,000
        assert_eq!(storage.get_balance(&alice).unwrap(), 499_000); 
        assert_eq!(storage.get_balance(&bob).unwrap(), 1_000_000);
        assert_eq!(storage.get_balance(&charlie).unwrap(), 500_000);
        assert_eq!(storage.get_nonce(&alice).unwrap(), 2);
    }

    #[test]
    fn test_atomic_block_revert() {
        let dir = tempdir().unwrap();
        let storage = StateStorage::open(dir.path().to_path_buf()).unwrap();

        let alice_kp = KeyPair::generate();
        let bob_kp = KeyPair::generate();

        let alice = alice_kp.public_key();
        let bob = bob_kp.public_key();

        // Setup: Alice sends to Bob
        storage.set_balance(&alice, 2_000_000).unwrap();
        storage.set_nonce(&alice, 0).unwrap();

        let tx = Transaction::new(alice, bob, 1_000_000, 1_000, 0); // 1K fee
        storage.apply_block_atomic(&vec![tx.clone()]).unwrap();

        // Verify applied: Alice pays 1M + 1K fee
        assert_eq!(storage.get_balance(&alice).unwrap(), 999_000); // 2M - 1M - 1K
        assert_eq!(storage.get_balance(&bob).unwrap(), 1_000_000);
        assert_eq!(storage.get_nonce(&alice).unwrap(), 1);

        // Revert transaction
        storage.revert_block_atomic(&vec![tx]).unwrap();

        // Verify reverted
        assert_eq!(storage.get_balance(&alice).unwrap(), 2_000_000);
        assert_eq!(storage.get_balance(&bob).unwrap(), 0);
        assert_eq!(storage.get_nonce(&alice).unwrap(), 0);
    }

    #[test]
    fn test_balance_overflow_protection() {
        let dir = tempdir().unwrap();
        let storage = StateStorage::open(dir.path().to_path_buf()).unwrap();

        let alice = KeyPair::generate().public_key();

        storage.set_balance(&alice, u64::MAX - 100).unwrap();

        // Should error on overflow instead of saturating
        assert!(storage.add_balance(&alice, 200).is_err());
    }
}
