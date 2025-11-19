/// BIP39 mnemonic phrase support for wallet backup and recovery
/// دعم عبارة التذكير BIP39 للنسخ الاحتياطي واستعادة المحفظة

use anyhow::{anyhow, Result};
use bip39::{Language, Mnemonic};
use opensyria_core::crypto::KeyPair;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// HD Wallet with BIP39 mnemonic phrase
/// محفظة HD مع عبارة تذكير BIP39
#[derive(Debug, Serialize, Deserialize)]
pub struct HDWallet {
    /// Mnemonic phrase (12 or 24 words)
    #[serde(skip)]
    mnemonic: Option<Mnemonic>,
    /// Language for mnemonic (always English for now)
    language: String,
    /// Number of words (12 or 24)
    word_count: usize,
}

impl HDWallet {
    /// Generate new HD wallet with mnemonic phrase
    /// إنشاء محفظة HD جديدة مع عبارة تذكير
    pub fn generate(word_count: usize) -> Result<Self> {
        if word_count != 12 && word_count != 24 {
            return Err(anyhow!("Word count must be 12 or 24"));
        }

        // Calculate entropy size: 12 words = 128 bits (16 bytes), 24 words = 256 bits (32 bytes)
        let entropy_size = if word_count == 12 { 16 } else { 32 };
        let mut entropy = vec![0u8; entropy_size];
        rand::Rng::fill(&mut rand::thread_rng(), &mut entropy[..]);

        let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy)
            .map_err(|e| anyhow!("Failed to generate mnemonic: {}", e))?;

        Ok(Self {
            mnemonic: Some(mnemonic),
            language: "english".to_string(),
            word_count,
        })
    }

    /// Restore HD wallet from mnemonic phrase
    /// استعادة محفظة HD من عبارة التذكير
    pub fn from_phrase(phrase: &str) -> Result<Self> {
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, phrase)
            .map_err(|e| anyhow!("Invalid mnemonic phrase: {}", e))?;

        let word_count = phrase.split_whitespace().count();

        Ok(Self {
            mnemonic: Some(mnemonic),
            language: "english".to_string(),
            word_count,
        })
    }

    /// Get mnemonic phrase for backup (WARNING: Keep this secret!)
    /// الحصول على عبارة التذكير للنسخ الاحتياطي (تحذير: احتفظ بهذا سراً!)
    pub fn get_phrase(&self) -> Result<String> {
        self.mnemonic
            .as_ref()
            .map(|m| m.words().collect::<Vec<&str>>().join(" "))
            .ok_or_else(|| anyhow!("Mnemonic not available"))
    }

    /// Derive account keypair at specified index
    /// اشتقاق زوج مفاتيح الحساب عند الفهرس المحدد
    /// 
    /// Uses simplified derivation path: m/account_index
    /// For production, should use BIP44: m/44'/coin_type'/account'/change/address
    pub fn derive_account(&self, index: u32) -> Result<KeyPair> {
        let mnemonic = self
            .mnemonic
            .as_ref()
            .ok_or_else(|| anyhow!("Mnemonic not available"))?;

        // Get seed from mnemonic (with empty passphrase)
        let seed = mnemonic.to_seed("");

        // Simple derivation: Hash(seed || index) for OpenSyria
        // In production BIP44, would use proper HMAC-SHA512 chain
        let mut hasher = Sha256::new();
        hasher.update(&seed[..]);
        hasher.update(index.to_le_bytes());
        let derived = hasher.finalize();

        // Use first 32 bytes as private key
        let mut private_key = [0u8; 32];
        private_key.copy_from_slice(&derived[..32]);

        KeyPair::from_bytes(&private_key).map_err(|e| anyhow!("{}", e))
    }

    /// Get number of words in mnemonic
    pub fn word_count(&self) -> usize {
        self.word_count
    }

    /// Validate a mnemonic phrase without creating wallet
    /// التحقق من صحة عبارة التذكير دون إنشاء محفظة
    pub fn validate_phrase(phrase: &str) -> bool {
        Mnemonic::parse_in_normalized(Language::English, phrase).is_ok()
    }
}

/// Display warning about mnemonic security
/// عرض تحذير حول أمان عبارة التذكير
pub fn display_mnemonic_warning() {
    println!("\n⚠️  CRITICAL SECURITY WARNING | تحذير أمني حرج ⚠️");
    println!("═══════════════════════════════════════════════════════════════════════════════════");
    println!("Your mnemonic phrase is the MASTER KEY to your wallet.");
    println!("عبارة التذكير الخاصة بك هي المفتاح الرئيسي لمحفظتك.");
    println!();
    println!("Anyone with this phrase can access ALL your funds.");
    println!("أي شخص لديه هذه العبارة يمكنه الوصول إلى جميع أموالك.");
    println!();
    println!("NEVER share it with anyone! | لا تشاركها مع أي شخص أبداً!");
    println!("NEVER enter it on websites! | لا تدخلها على المواقع الإلكترونية أبداً!");
    println!("NEVER take screenshots! | لا تلتقط لقطات شاشة أبداً!");
    println!();
    println!("✓ Write it down on paper | اكتبها على ورقة");
    println!("✓ Store in a secure location (safe, safety deposit box) | احفظها في مكان آمن (خزنة، صندوق ودائع)");
    println!("✓ Consider making multiple copies in different locations | فكر في عمل نسخ متعددة في أماكن مختلفة");
    println!("═══════════════════════════════════════════════════════════════════════════════════\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_12_word_wallet() {
        let wallet = HDWallet::generate(12).expect("Failed to generate wallet");
        let phrase = wallet.get_phrase().expect("Failed to get phrase");
        
        let words: Vec<&str> = phrase.split_whitespace().collect();
        assert_eq!(words.len(), 12);
    }

    #[test]
    fn test_generate_24_word_wallet() {
        let wallet = HDWallet::generate(24).expect("Failed to generate wallet");
        let phrase = wallet.get_phrase().expect("Failed to get phrase");
        
        let words: Vec<&str> = phrase.split_whitespace().collect();
        assert_eq!(words.len(), 24);
    }

    #[test]
    fn test_invalid_word_count() {
        let result = HDWallet::generate(15);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must be 12 or 24"));
    }

    #[test]
    fn test_restore_from_phrase() {
        let original = HDWallet::generate(12).expect("Failed to generate wallet");
        let phrase = original.get_phrase().expect("Failed to get phrase");

        let restored = HDWallet::from_phrase(&phrase).expect("Failed to restore from phrase");

        assert_eq!(restored.word_count, 12);
        assert_eq!(
            restored.get_phrase().unwrap(),
            phrase
        );
    }

    #[test]
    fn test_derive_deterministic_accounts() {
        let wallet = HDWallet::generate(12).expect("Failed to generate wallet");

        // Derive account 0 twice - should be identical
        let account0_first = wallet.derive_account(0).expect("Failed to derive account 0");
        let account0_second = wallet.derive_account(0).expect("Failed to derive account 0 again");

        assert_eq!(
            account0_first.public_key(),
            account0_second.public_key()
        );

        // Derive account 1 - should be different from account 0
        let account1 = wallet.derive_account(1).expect("Failed to derive account 1");

        assert_ne!(
            account0_first.public_key(),
            account1.public_key()
        );
    }

    #[test]
    fn test_restore_derives_same_accounts() {
        let original = HDWallet::generate(24).expect("Failed to generate wallet");
        let phrase = original.get_phrase().expect("Failed to get phrase");

        let account0_original = original.derive_account(0).expect("Failed to derive account 0");
        let account1_original = original.derive_account(1).expect("Failed to derive account 1");

        // Restore from phrase
        let restored = HDWallet::from_phrase(&phrase).expect("Failed to restore wallet");

        let account0_restored = restored.derive_account(0).expect("Failed to derive account 0");
        let account1_restored = restored.derive_account(1).expect("Failed to derive account 1");

        // Accounts should match
        assert_eq!(
            account0_original.public_key(),
            account0_restored.public_key()
        );
        assert_eq!(
            account1_original.public_key(),
            account1_restored.public_key()
        );
    }

    #[test]
    fn test_validate_phrase() {
        let wallet = HDWallet::generate(12).unwrap();
        let valid_phrase = wallet.get_phrase().unwrap();

        assert!(HDWallet::validate_phrase(&valid_phrase));
        assert!(!HDWallet::validate_phrase("invalid random words that are not a valid mnemonic"));
        assert!(!HDWallet::validate_phrase(""));
    }

    #[test]
    fn test_invalid_phrase_restoration() {
        let result = HDWallet::from_phrase("not a valid mnemonic phrase at all");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid mnemonic"));
    }
}
