use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

/// Cryptographic key pair for signing transactions
#[derive(Debug, Clone)]
pub struct KeyPair {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
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

    /// Get private key bytes (use carefully!)
    pub fn private_key_bytes(&self) -> [u8; 32] {
        self.signing_key.to_bytes()
    }
}

/// Public key wrapper
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
        let message = b"Open Syria Blockchain";
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
}
