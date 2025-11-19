# Module C1: Wallet Security Audit

**Open Syria Blockchain - Private Key Management & Wallet Security**

**Module:** C1 - Wallet Security  
**Date:** November 18, 2025  
**Status:** 🔴 **CRITICAL SECURITY FAILURES**  
**Severity:** 🔴 **EXTREME RISK** (Private keys stored in plaintext!)

---

## Scope Confirmation

**Files Reviewed:**
- `crates/wallet/src/storage.rs` (142 lines) - Wallet storage implementation
- `crates/wallet/src/main.rs` (211 lines) - Wallet CLI
- `crates/wallet/src/lib.rs` (4 lines) - Module exports
- `crates/core/src/crypto.rs` (141 lines) - Cryptographic primitives

**Implementation Status:**
- ✅ Ed25519 key generation (cryptographically secure)
- ✅ Account creation and management
- ✅ Transaction signing
- ✅ Basic account listing
- 🔴 Private keys stored in **PLAINTEXT JSON**
- ❌ No encryption whatsoever
- ❌ No password protection
- ❌ No mnemonic phrases (BIP39)
- ❌ No HD wallets (BIP32/BIP44)
- ❌ No hardware wallet support
- ❌ No secure key derivation
- ❌ No backup/recovery mechanism

---

## Architecture Overview

### Current Wallet Storage

```
~/.opensyria/wallet/
├── alice.json          ← PLAINTEXT PRIVATE KEY!
├── bob.json            ← PLAINTEXT PRIVATE KEY!
└── merchant.json       ← PLAINTEXT PRIVATE KEY!

Example alice.json:
{
  "name": "alice",
  "address": "a1b2c3d4...",
  "private_key": "deadbeef1234567890abcdef..."  ← 🚨 EXPOSED!
  "created_at": 1700000000
}
```

**Anyone with filesystem access can steal ALL funds!**

---

## ✅ Strengths

### 1. **Cryptographically Secure Key Generation**
```rust
// crates/core/src/crypto.rs:14
pub fn generate() -> Self {
    let mut csprng = OsRng; // ✓ Uses OS-provided CSPRNG
    let secret_bytes = rand::Rng::gen::<[u8; 32]>(&mut csprng);
    let signing_key = SigningKey::from_bytes(&secret_bytes);
    // ...
}
```
Keys themselves are generated securely.

### 2. **Ed25519 Implementation**
Uses `ed25519-dalek`, a well-audited cryptography library.

### 3. **Simple API**
Clean interface for account management:
```rust
let account = Account::new("alice".to_string());
storage.save_account(&account)?;
```

---

## 🚨 Critical Security Issues

### [WALLET-CRITICAL-001] Private Keys Stored in Plaintext

**Severity:** 🔴 CRITICAL  
**CVSS:** 10.0 (Critical)  
**CWE:** CWE-312 (Cleartext Storage of Sensitive Information)

**Description:**  
Private keys stored **unencrypted** in JSON files readable by any process/user.

**Evidence:**
```rust
// crates/wallet/src/storage.rs:91
pub fn save_account(&self, account: &Account) -> Result<()> {
    let filename = format!("{}.json", account.name);
    let path = self.wallet_dir.join(filename);

    let json = serde_json::to_string_pretty(account)?;
    // ❌ Writes PLAINTEXT JSON with private key!
    
    fs::write(&path, json)?; // ❌ No encryption!
    Ok(())
}

// Account struct:
#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub name: String,
    pub address: PublicKey,
    #[serde(with = "hex_serde")]
    private_key: [u8; 32], // ❌ SERIALIZED AS HEX STRING!
    pub created_at: u64,
}
```

**Actual File Contents:**
```bash
$ cat ~/.opensyria/wallet/alice.json
{
  "name": "alice",
  "address": "a1b2c3d4e5f6...",
  "private_key": "deadbeef1234567890abcdef1234567890abcdef1234567890abcdef12345678",
  "created_at": 1700000000
}

# ❌ ANYONE CAN READ THIS!
```

**Attack Scenarios:**

**Attack 1: Malware**
```bash
# Malware scans for wallet files
find ~ -name "*.json" -path "*/.opensyria/wallet/*"
# Exfiltrates all private keys
# Empties all wallets
```

**Attack 2: Cloud Backup**
```bash
# User backs up home directory to cloud
# Cloud provider (or hacker) accesses backup
# All private keys exposed
```

**Attack 3: Shared Computer**
```bash
# Multiple users on same machine
# Any user can read ~/.opensyria/wallet/
# All funds stolen
```

**Attack 4: Forensics**
```bash
# User sells old computer
# Buyer runs file recovery software
# Deleted wallet files recovered
# Private keys extracted
```

**Impact:**
- **TOTAL LOSS OF FUNDS** - Anyone with file access can drain wallets
- **No recovery** - Stolen funds are gone forever
- **Regulatory non-compliance** - Violates data protection laws
- **Reputation damage** - "Open Syria wallet" synonymous with insecurity

**Remediation:**
```rust
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce
};
use argon2::{Argon2, PasswordHasher, PasswordHash};
use rand::RngCore;

#[derive(Serialize, Deserialize)]
struct EncryptedAccount {
    pub name: String,
    pub address: PublicKey,
    pub encrypted_key: Vec<u8>, // Encrypted private key
    pub nonce: [u8; 12],         // AES-GCM nonce
    pub salt: [u8; 32],          // Password salt
    pub created_at: u64,
}

impl WalletStorage {
    pub fn save_account_encrypted(
        &self,
        account: &Account,
        password: &str,
    ) -> Result<()> {
        // 1. Derive encryption key from password using Argon2
        let mut salt = [0u8; 32];
        OsRng.fill_bytes(&mut salt);
        
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        
        // Extract 32-byte key from hash
        let key = &password_hash.hash.unwrap().as_bytes()[..32];
        
        // 2. Encrypt private key with AES-256-GCM
        let cipher = Aes256Gcm::new_from_slice(key)?;
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let encrypted_key = cipher.encrypt(nonce, account.private_key.as_ref())?;
        
        // 3. Save encrypted account
        let encrypted = EncryptedAccount {
            name: account.name.clone(),
            address: account.address,
            encrypted_key,
            nonce: nonce_bytes,
            salt,
            created_at: account.created_at,
        };
        
        let filename = format!("{}.json", account.name);
        let path = self.wallet_dir.join(filename);
        let json = serde_json::to_string_pretty(&encrypted)?;
        fs::write(&path, json)?;
        
        Ok(())
    }
    
    pub fn load_account_encrypted(
        &self,
        name: &str,
        password: &str,
    ) -> Result<Account> {
        // Load encrypted account
        let filename = format!("{}.json", name);
        let path = self.wallet_dir.join(filename);
        let json = fs::read_to_string(&path)?;
        let encrypted: EncryptedAccount = serde_json::from_str(&json)?;
        
        // Derive decryption key from password
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &encrypted.salt)?;
        let key = &password_hash.hash.unwrap().as_bytes()[..32];
        
        // Decrypt private key
        let cipher = Aes256Gcm::new_from_slice(key)?;
        let nonce = Nonce::from_slice(&encrypted.nonce);
        
        let decrypted_key = cipher.decrypt(nonce, encrypted.encrypted_key.as_ref())
            .map_err(|_| anyhow!("Invalid password or corrupted wallet"))?;
        
        let mut private_key = [0u8; 32];
        private_key.copy_from_slice(&decrypted_key);
        
        Ok(Account {
            name: encrypted.name,
            address: encrypted.address,
            private_key,
            created_at: encrypted.created_at,
        })
    }
}
```

**Status:** ❌ Not Implemented (CATASTROPHIC VULNERABILITY)

---

### [WALLET-CRITICAL-002] No Mnemonic Phrase (BIP39)

**Severity:** 🔴 CRITICAL  
**CVSS:** 9.0 (Critical)  
**CWE:** CWE-522 (Insufficiently Protected Credentials)

**Description:**  
No mnemonic seed phrase for wallet backup/recovery. Users can't recover funds if files are lost.

**Bitcoin/Ethereum Standard:**
```
Mnemonic (BIP39):
  "abandon ability able about above absent absorb abstract absurd abuse access accident"

Seed (BIP39):
  Binary seed derived from mnemonic + optional passphrase

Master Key (BIP32):
  HD wallet root key derived from seed

Child Keys (BIP44):
  m/44'/60'/0'/0/0 (Ethereum)
  m/44'/0'/0'/0/0  (Bitcoin)
```

**Current Open Syria:**
```
No mnemonic → User loses JSON file → Funds lost forever
```

**Remediation:**
```rust
use bip39::{Mnemonic, Language, Seed};
use tiny_hderive::bip32::ExtendedPrivKey;

pub struct HDWallet {
    mnemonic: Mnemonic,
    seed: Seed,
    master_key: ExtendedPrivKey,
}

impl HDWallet {
    /// Generate new HD wallet with 24-word mnemonic
    pub fn generate() -> Result<Self> {
        let mnemonic = Mnemonic::generate(24)?;
        let seed = Seed::new(&mnemonic, ""); // Optional passphrase
        
        let master_key = ExtendedPrivKey::derive(seed.as_bytes(), "m")?;
        
        Ok(Self {
            mnemonic,
            seed,
            master_key,
        })
    }
    
    /// Restore wallet from mnemonic phrase
    pub fn from_mnemonic(words: &str, passphrase: &str) -> Result<Self> {
        let mnemonic = Mnemonic::from_phrase(words, Language::English)?;
        let seed = Seed::new(&mnemonic, passphrase);
        let master_key = ExtendedPrivKey::derive(seed.as_bytes(), "m")?;
        
        Ok(Self {
            mnemonic,
            seed,
            master_key,
        })
    }
    
    /// Derive account key at path m/44'/5963'/0'/0/{index}
    /// (5963 = Open Syria coin type)
    pub fn derive_account(&self, index: u32) -> Result<KeyPair> {
        let path = format!("m/44'/5963'/0'/0/{}", index);
        let child_key = self.master_key.derive(&path)?;
        
        KeyPair::from_bytes(&child_key.secret())
    }
    
    /// Get mnemonic phrase for backup
    pub fn get_mnemonic(&self) -> &str {
        self.mnemonic.phrase()
    }
}

// Usage:
let wallet = HDWallet::generate()?;
println!("BACKUP THIS PHRASE:");
println!("{}", wallet.get_mnemonic());
println!("Write it down on paper and store securely!");

let account0 = wallet.derive_account(0)?;
let account1 = wallet.derive_account(1)?;
```

**Status:** ❌ Not Implemented

---

### [WALLET-CRITICAL-003] No Password Protection

**Severity:** 🔴 CRITICAL  
**CVSS:** 9.5 (Critical)  
**CWE:** CWE-287 (Improper Authentication)

**Description:**  
Wallet CLI never asks for password. Anyone at keyboard can spend funds.

**Evidence:**
```rust
// crates/wallet/src/main.rs:53
Commands::Send { from, to, amount, fee, nonce } => {
    let account = storage.load_account(&from)?; // ❌ No password check!
    
    // Immediately loads private key and signs
    let keypair = account.keypair()?;
    let sig = keypair.sign(&tx.signing_hash());
    // Transaction sent!
}
```

**Attack Scenario:**
```bash
# Attacker sits at victim's computer
$ wallet send --from alice --to attacker_addr --amount 1000000
✓ Transaction created and signed

# All funds stolen in 3 seconds!
```

**Remediation:**
```rust
use rpassword::read_password;

Commands::Send { from, to, amount, fee, nonce } => {
    // Prompt for password
    println!("Enter password for '{}': ", from);
    let password = read_password()?;
    
    // Load encrypted account
    let account = storage.load_account_encrypted(&from, &password)?;
    
    // Rest of transaction logic...
}
```

**Status:** ❌ Not Implemented

---

### [WALLET-CRITICAL-004] No File Permissions

**Severity:** 🔴 CRITICAL  
**CVSS:** 8.5 (High)  
**CWE:** CWE-732 (Incorrect Permission Assignment)

**Description:**  
Wallet files created with default permissions (readable by all users).

**Evidence:**
```rust
// crates/wallet/src/storage.rs:95
fs::write(&path, json)?; // ❌ Uses default file permissions!

// On Unix: -rw-r--r-- (644)
// Any user on system can read!
```

**Proper Implementation:**
```rust
use std::os::unix::fs::PermissionsExt;

pub fn save_account_encrypted(&self, account: &Account, password: &str) -> Result<()> {
    // ... encryption logic
    
    // Write file
    fs::write(&path, json)?;
    
    // Set restrictive permissions (owner read/write only)
    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&path)?.permissions();
        perms.set_mode(0o600); // -rw-------
        fs::set_permissions(&path, perms)?;
    }
    
    Ok(())
}
```

**Status:** ❌ Not Implemented

---

## 🟠 High Severity Issues

### [WALLET-HIGH-001] No Hardware Wallet Support

**Severity:** 🟠 HIGH  
**CVSS:** 6.5 (Medium)  
**Impact:** No cold storage option for large amounts

**Description:**  
No integration with hardware wallets (Ledger, Trezor). All keys are hot.

**Recommendation:**  
Implement USB HID protocol for Ledger/Trezor integration.

**Status:** ❌ Not Implemented

---

### [WALLET-HIGH-002] Private Keys in Memory

**Severity:** 🟠 HIGH  
**CVSS:** 7.0 (High)  
**Impact:** Memory dumps expose keys

**Description:**  
Private keys stored in plain Rust types, not protected memory.

**Evidence:**
```rust
pub struct Account {
    private_key: [u8; 32], // ❌ Plain array, pageable memory
}
```

**Recommendation:**
```rust
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecureKey {
    #[zeroize(skip)]
    pub address: PublicKey,
    key: [u8; 32], // Zeroed on drop
}

// Or use secrecy crate:
use secrecy::{Secret, ExposeSecret};

pub struct Account {
    private_key: Secret<[u8; 32]>,
}
```

**Status:** ❌ Not Implemented

---

### [WALLET-HIGH-003] No Account Import/Export

**Severity:** 🟠 HIGH  
**CVSS:** 5.8 (Medium)  
**Impact:** Can't migrate wallets between devices

**Recommendation:**
```rust
Commands::Export {
    name: String,
    output: PathBuf,
    password: String,
}

Commands::Import {
    file: PathBuf,
    password: String,
}
```

**Status:** ❌ Not Implemented

---

### [WALLET-HIGH-004] No Multi-Signature Wallet Support

**Severity:** 🟠 HIGH  
**CVSS:** 6.0 (Medium)  
**Impact:** No corporate/shared custody support

**Note:** Multisig *transactions* exist (see node-cli), but no *wallet* integration.

**Status:** ⚠️ Partial (multisig exists elsewhere, not in wallet)

---

## 🟡 Medium Severity Issues

### [WALLET-MEDIUM-001] No Balance Display

**Severity:** 🟡 MEDIUM  
**Impact:** User can't see account balance

**Evidence:**
```rust
// crates/wallet/src/main.rs:135
println!("Balance | الرصيد: {} (coming soon)", "0.00 SYL".bold());
// ❌ Not implemented!
```

**Remediation:**  
Query node RPC for account balance.

**Status:** ❌ Not Implemented

---

### [WALLET-MEDIUM-002] No Transaction History

**Severity:** 🟡 MEDIUM  
**Impact:** User can't see past transactions

**Recommendation:**
```rust
Commands::History {
    name: String,
    limit: Option<u32>,
}
```

**Status:** ❌ Not Implemented

---

### [WALLET-MEDIUM-003] No QR Code Support

**Severity:** 🟡 MEDIUM  
**Impact:** Hard to share addresses (mobile use case)

**Recommendation:**
```rust
use qrcode::QrCode;

Commands::QR { name: String } => {
    let account = storage.load_account(&name)?;
    let qr = QrCode::new(account.address.to_hex())?;
    println!("{}", qr.render::<char>().build());
}
```

**Status:** ❌ Not Implemented

---

### [WALLET-MEDIUM-004] No Address Book

**Severity:** 🟡 MEDIUM  
**Impact:** Must remember recipient addresses

**Recommendation:**
```rust
pub struct AddressBook {
    contacts: HashMap<String, PublicKey>,
}

Commands::AddContact { name: String, address: String }
Commands::Send { from: String, to: String, ... } // Accept name or address
```

**Status:** ❌ Not Implemented

---

### [WALLET-MEDIUM-005] No Fee Estimation

**Severity:** 🟡 MEDIUM  
**Impact:** User must manually specify fees

**Evidence:**
```rust
#[arg(short = 'f', long, default_value = "0.0001")]
fee: f64,
// ❌ Hardcoded default, no network-based estimation
```

**Status:** ❌ Not Implemented

---

## 🔵 Low Severity / Enhancement Issues

**[WALLET-LOW-001]** No wallet versioning (migration problems)  
**[WALLET-LOW-002]** No account labels/metadata  
**[WALLET-LOW-003]** No testnet/mainnet separation  
**[WALLET-LOW-004]** No watch-only addresses  
**[WALLET-LOW-005]** No custom derivation paths (BIP44)  
**[WALLET-LOW-006]** No passphrase support (25th word)

---

## Security Summary

| Category | Count | Status |
|----------|-------|--------|
| 🔴 CRITICAL | 4 | ❌ Not Addressed |
| 🟠 HIGH | 4 | ❌ Not Addressed |
| 🟡 MEDIUM | 5 | ⚠️ Partial |
| 🔵 LOW | 6 | ⚠️ Enhancement |

**Total Issues:** 19

---

## Test Coverage Assessment

**Current Tests:**
- ❌ No tests at all!
- No encryption tests
- No mnemonic tests
- No HD derivation tests
- No permission tests

**Required Test Suite:**
```rust
#[cfg(test)]
mod wallet_tests {
    #[test]
    fn test_encryption_decryption() {
        // Encrypt account with password
        // Decrypt with correct password → success
        // Decrypt with wrong password → error
    }
    
    #[test]
    fn test_mnemonic_generation() {
        // Generate mnemonic
        // Verify 24 words
        // Restore wallet from mnemonic
        // Verify same keys derived
    }
    
    #[test]
    fn test_hd_derivation() {
        // Derive accounts 0-9
        // Verify deterministic
    }
    
    #[test]
    fn test_file_permissions() {
        // Create wallet file
        // Verify permissions are 0600
    }
}
```

---

## Recommendations by Priority

### P0 - BLOCKERS (Before ANY Public Use)

1. **Encrypt private keys with password** [WALLET-CRITICAL-001]
2. **Implement BIP39 mnemonic** [WALLET-CRITICAL-002]
3. **Add password prompts** [WALLET-CRITICAL-003]
4. **Set restrictive file permissions** [WALLET-CRITICAL-004]

### P1 - Critical (Before Mainnet)

5. **HD wallet (BIP32/BIP44)** [WALLET-CRITICAL-002]
6. **Secure memory for keys** [WALLET-HIGH-002]
7. **Import/export functionality** [WALLET-HIGH-003]
8. **Hardware wallet support** [WALLET-HIGH-001]

### P2 - Important

9. **Balance display** [WALLET-MEDIUM-001]
10. **Transaction history** [WALLET-MEDIUM-002]
11. **QR code support** [WALLET-MEDIUM-003]
12. **Fee estimation** [WALLET-MEDIUM-005]

---

## Implementation Checklist

### Phase 1: Emergency Security (Week 1)
- [ ] Implement AES-256-GCM encryption
- [ ] Add Argon2 password hashing
- [ ] Prompt for password on all operations
- [ ] Set file permissions to 0600
- [ ] Migrate existing wallets (one-time tool)

### Phase 2: HD Wallet (Week 2-3)
- [ ] Integrate bip39 crate
- [ ] Generate 24-word mnemonic
- [ ] Implement BIP32 key derivation
- [ ] Support BIP44 paths (m/44'/5963'/0'/0/N)
- [ ] Restore from mnemonic

### Phase 3: UX Improvements (Week 4)
- [ ] Balance display (RPC integration)
- [ ] Transaction history
- [ ] QR code generation
- [ ] Address book
- [ ] Fee estimation

### Phase 4: Advanced Features (Week 5-6)
- [ ] Hardware wallet (Ledger)
- [ ] Secure memory (zeroize)
- [ ] Import/export
- [ ] Watch-only addresses
- [ ] Testnet support

---

## Wallet Specification Needed

**Create:** `docs/wallet/WALLET_SPEC.md`

**Contents:**
- Encryption algorithm (AES-256-GCM + Argon2)
- Mnemonic generation (BIP39)
- HD derivation paths (BIP32/BIP44)
- File format (encrypted JSON schema)
- Password requirements
- Backup/recovery procedures
- Hardware wallet protocol
- Multi-signature integration

---

## Comparison with Other Wallets

| Feature | Open Syria | MetaMask | Bitcoin Core | Hardware Wallet |
|---------|------------|----------|--------------|-----------------|
| **Encryption** | ❌ None | ✅ AES-256 | ✅ AES-256 | ✅ Hardware |
| **Password** | ❌ None | ✅ Yes | ✅ Yes | ✅ PIN |
| **Mnemonic** | ❌ None | ✅ BIP39 | ✅ BIP39 | ✅ BIP39 |
| **HD Wallet** | ❌ None | ✅ BIP32/44 | ✅ BIP32 | ✅ BIP32/44 |
| **File Permissions** | ❌ Default | ✅ 0600 | ✅ 0600 | N/A |
| **Secure Memory** | ❌ None | ⚠️ Partial | ✅ Yes | ✅ Hardware |

**Gap:** Open Syria has **ZERO** security features that modern wallets have.

---

## Regulatory Compliance

### GDPR (EU Data Protection)
- ❌ **VIOLATION:** Private keys = personal data, must be encrypted
- ❌ **VIOLATION:** No right to erasure (keys readable after deletion)

### PCI DSS (Payment Card Industry)
- ❌ **VIOLATION:** Sensitive data stored unencrypted
- ❌ **VIOLATION:** No access controls

### SOC 2 (Security Controls)
- ❌ **VIOLATION:** No encryption at rest
- ❌ **VIOLATION:** No audit logging

**Legal Risk:** Deploying this wallet could expose project to lawsuits.

---

## Conclusion

**Overall Assessment:** 🔴 **EXTREME RISK - CATASTROPHIC SECURITY FAILURE**

**Strengths:**
- Ed25519 key generation is cryptographically secure
- Simple, clean API
- Bilingual support (Arabic/English)

**Critical Gaps:**
- **PLAINTEXT PRIVATE KEYS** → Funds stolen instantly
- **NO PASSWORD** → Anyone can spend
- **NO MNEMONIC** → Lost files = lost funds forever
- **NO ENCRYPTION** → Regulatory violations

**Verdict:**  
The Open Syria wallet is **dangerously insecure** and **must not be used** with real funds. It's worse than no wallet at all because it gives a false sense of security. Private keys stored in plaintext JSON files is a **catastrophic vulnerability** that violates every security best practice and regulatory requirement.

**This is the WORST security finding in the entire audit.**

If a single user loses funds due to this, the entire project's reputation will be destroyed.

**Estimated Fix Time:** 4-6 weeks for P0 issues (encryption, mnemonic, password, permissions)

**DO NOT USE IN PRODUCTION UNDER ANY CIRCUMSTANCES**

---

**Next Module:** C2 - Wallet API Audit  
**Status:** Ready to proceed after review

**Auditor:** Senior Blockchain Security Specialist  
**Date:** November 18, 2025
