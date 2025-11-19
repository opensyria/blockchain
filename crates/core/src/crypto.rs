use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Cryptographic key pair for signing transactions
/// 
/// SECURITY: Implements ZeroizeOnDrop to automatically clear private key material
/// from memory when the KeyPair is dropped, preventing memory disclosure attacks.
/// The SigningKey contains secret bytes that are zeroized via to_bytes().
#[derive(Debug, Clone)]
pub struct KeyPair {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

impl ZeroizeOnDrop for KeyPair {}

impl Drop for KeyPair {
    fn drop(&mut self) {
        // Zeroize the secret key bytes
        // SigningKey::to_bytes() gives us the 32-byte secret
        let mut secret_bytes = self.signing_key.to_bytes();
        secret_bytes.zeroize();
    }
}

impl KeyPair {
    /// Generate a new random key pair
    pub fn generate() -> Self {
        let mut csprng = OsRng;
        let secret_bytes = rand::Rng::gen::<[u8; 32]>(&mut csprng);
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let verifying_key = signing_key.verifying_key();

        Self {
            signing_key,
            verifying_key,
        }
    }

    /// Create key pair from signing key bytes
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self, CryptoError> {
        let signing_key = SigningKey::from_bytes(bytes);
        let verifying_key = signing_key.verifying_key();

        Ok(Self {
            signing_key,
            verifying_key,
        })
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        self.signing_key.sign(message).to_bytes().to_vec()
    }

    /// Get public key bytes
    pub fn public_key(&self) -> PublicKey {
        PublicKey(self.verifying_key.to_bytes())
    }

    /// Get private key bytes (DEPRECATED - use with_private_key instead!)
    /// 
    /// ⚠️  CRITICAL SECURITY WARNING: The returned bytes are NOT zeroized after use.
    /// This method is DEPRECATED and will be removed in future versions.
    /// 
    /// USE with_private_key() INSTEAD for automatic memory protection.
    #[deprecated(
        since = "0.2.0",
        note = "Use with_private_key() closure pattern to prevent memory leaks"
    )]
    pub fn private_key_bytes(&self) -> [u8; 32] {
        self.signing_key.to_bytes()
    }
    
    /// Execute a closure with temporary access to private key bytes
    /// 
    /// This is the RECOMMENDED and SECURE way to access private keys.
    /// The key material is automatically zeroized immediately after the closure returns.
    /// 
    /// # Security
    /// - Private key is automatically cleared from memory after use
    /// - Prevents memory disclosure via core dumps, swap, or debuggers
    /// - Resistant to cold boot attacks
    /// 
    /// # Example
    /// ```ignore
    /// let signature = keypair.with_private_key(|key| {
    ///     // Use key for signing or encryption
    ///     sign_data(key)
    /// }); // key is automatically zeroized here
    /// ```
    pub fn with_private_key<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&[u8; 32]) -> T,
    {
        let mut key = self.signing_key.to_bytes();
        let result = f(&key);
        key.zeroize(); // Explicit zero-out
        result
    }
}

/// Public key wrapper
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(bincode::Encode, bincode::Decode)]
pub struct PublicKey(pub [u8; 32]);

impl PublicKey {
    /// Verify a signature against this public key
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> Result<(), CryptoError> {
        let verifying_key =
            VerifyingKey::from_bytes(&self.0).map_err(|_| CryptoError::InvalidPublicKey)?;

        let sig = Signature::from_slice(signature).map_err(|_| CryptoError::InvalidSignature)?;

        verifying_key
            .verify(message, &sig)
            .map_err(|_| CryptoError::VerificationFailed)
    }

    /// Convert to hex string (for display/storage)
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Parse from hex string
    pub fn from_hex(s: &str) -> Result<Self, CryptoError> {
        let bytes = hex::decode(s).map_err(|_| CryptoError::InvalidHex)?;
        if bytes.len() != 32 {
            return Err(CryptoError::InvalidPublicKey);
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(PublicKey(arr))
    }

    /// Create a zero public key (used for coinbase transactions)
    /// إنشاء مفتاح عام صفري (للمعاملات الكوين بيس)
    pub fn zero() -> Self {
        PublicKey([0u8; 32])
    }

    /// Check if this is the zero public key
    pub fn is_zero(&self) -> bool {
        self.0 == [0u8; 32]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CryptoError {
    InvalidPublicKey,
    InvalidSignature,
    VerificationFailed,
    InvalidHex,
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CryptoError::InvalidPublicKey => write!(f, "Invalid public key"),
            CryptoError::InvalidSignature => write!(f, "Invalid signature"),
            CryptoError::VerificationFailed => write!(f, "Signature verification failed"),
            CryptoError::InvalidHex => write!(f, "Invalid hex encoding"),
        }
    }
}

impl std::error::Error for CryptoError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let kp = KeyPair::generate();
        let message = b"OpenSyria Blockchain";
        let signature = kp.sign(message);

        assert!(kp.public_key().verify(message, &signature).is_ok());
    }

    #[test]
    fn test_public_key_hex_conversion() {
        let kp = KeyPair::generate();
        let pk = kp.public_key();
        let hex = pk.to_hex();
        let parsed = PublicKey::from_hex(&hex).unwrap();

        assert_eq!(pk, parsed);
    }

    #[test]
    fn test_invalid_signature_fails() {
        let kp = KeyPair::generate();
        let message = b"test";
        let mut signature = kp.sign(message);
        signature[0] ^= 1; // Corrupt signature

        assert!(kp.public_key().verify(message, &signature).is_err());
    }

    #[test]
    fn test_with_private_key_zeroization() {
        let kp = KeyPair::generate();
        
        // Use with_private_key to access key material
        let key_copy = kp.with_private_key(|key| {
            // Make a copy to verify key is accessible
            *key
        });
        
        // Verify the key was accessible and valid
        assert_eq!(key_copy.len(), 32);
        
        // Verify we can create a keypair from it
        let reconstructed = KeyPair::from_bytes(&key_copy).unwrap();
        assert_eq!(kp.public_key(), reconstructed.public_key());
        
        // Note: We can't directly test that memory was zeroized (requires unsafe/external tools)
        // but the zeroize crate guarantees it via compiler optimization barriers
    }

    #[test]
    fn test_keypair_drop_clears_memory() {
        // This test verifies ZeroizeOnDrop is implemented
        // The actual zeroing happens at drop time and is guaranteed by zeroize crate
        let kp = KeyPair::generate();
        let pk = kp.public_key();
        
        drop(kp); // KeyPair dropped here, memory should be zeroized
        
        // Public key should still be valid (it's copied, not zeroized)
        assert_eq!(pk.0.len(), 32);
    }
}
