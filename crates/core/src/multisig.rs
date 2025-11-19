use crate::crypto::PublicKey;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Multi-signature account configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MultisigAccount {
    /// List of authorized signers
    pub signers: Vec<PublicKey>,
    /// Minimum signatures required (M-of-N)
    pub threshold: u8,
    // NOTE: Nonce is now tracked in StateStorage, not here (prevents replay attacks)
}

impl MultisigAccount {
    /// Create a new multisig account
    pub fn new(signers: Vec<PublicKey>, threshold: u8) -> Result<Self, MultisigError> {
        if signers.is_empty() {
            return Err(MultisigError::NoSigners);
        }

        if threshold == 0 {
            return Err(MultisigError::InvalidThreshold);
        }

        if threshold as usize > signers.len() {
            return Err(MultisigError::ThresholdTooHigh {
                threshold,
                signers: signers.len(),
            });
        }

        // Check for duplicate signers
        let mut unique_signers = signers.clone();
        unique_signers.sort_by_key(|k| k.0);
        unique_signers.dedup();
        if unique_signers.len() != signers.len() {
            return Err(MultisigError::DuplicateSigners);
        }

        Ok(Self {
            signers,
            threshold,
        })
    }

    /// Get the multisig account address (deterministic hash of configuration)
    pub fn address(&self) -> PublicKey {
        let mut hasher = Sha256::new();

        // Hash sorted signers for deterministic address
        let mut sorted_signers = self.signers.clone();
        sorted_signers.sort_by_key(|k| k.0);

        for signer in &self.signers {
            hasher.update(signer.0);
        }
        hasher.update([self.threshold]);

        let hash = hasher.finalize();
        PublicKey(hash.into())
    }

    /// Check if a public key is an authorized signer
    pub fn is_signer(&self, pubkey: &PublicKey) -> bool {
        self.signers.contains(pubkey)
    }

    /// Get the number of signers
    pub fn num_signers(&self) -> usize {
        self.signers.len()
    }
}

/// Multi-signature transaction with multiple signatures
/// 
/// SECURITY NOTE: Nonce must be validated against StateStorage before execution
/// to prevent replay attacks. The nonce field here is included in signatures but
/// MUST be checked against the persistent state during transaction validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultisigTransaction {
    /// Multisig account configuration
    pub account: MultisigAccount,
    /// Recipient's public key
    pub to: PublicKey,
    /// Amount in smallest unit
    pub amount: u64,
    /// Transaction fee
    pub fee: u64,
    /// Account nonce (MUST be validated against StateStorage)
    pub nonce: u64,
    /// List of signatures from different signers
    pub signatures: Vec<SignatureEntry>,
    /// Optional transaction metadata
    pub data: Option<Vec<u8>>,
    /// Expiry block height (transaction invalid after this)
    pub expiry_height: Option<u64>,
}

/// Single signature entry with signer identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureEntry {
    /// Public key of the signer
    pub signer: PublicKey,
    /// Signature bytes
    pub signature: Vec<u8>,
}

impl MultisigTransaction {
    /// Create a new unsigned multisig transaction
    pub fn new(account: MultisigAccount, to: PublicKey, amount: u64, fee: u64, nonce: u64) -> Self {
        Self {
            account,
            to,
            amount,
            fee,
            nonce,
            signatures: Vec::new(),
            data: None,
            expiry_height: None,
        }
    }

    /// Set expiry block height
    pub fn with_expiry(mut self, expiry_height: u64) -> Self {
        self.expiry_height = Some(expiry_height);
        self
    }

    /// Add optional data payload
    pub fn with_data(mut self, data: Vec<u8>) -> Self {
        self.data = Some(data);
        self
    }

    /// Check if transaction has expired
    pub fn is_expired(&self, current_height: u64) -> bool {
        if let Some(expiry) = self.expiry_height {
            current_height > expiry
        } else {
            false
        }
    }

    /// Get signing hash (what each signer signs)
    pub fn signing_hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();

        // Include multisig address
        let address = self.account.address();
        hasher.update(address.0);

        hasher.update(self.to.0);
        hasher.update(self.amount.to_le_bytes());
        hasher.update(self.fee.to_le_bytes());
        hasher.update(self.nonce.to_le_bytes());

        if let Some(data) = &self.data {
            hasher.update(data);
        }

        hasher.finalize().into()
    }

    /// Add a signature from one of the signers
    pub fn add_signature(
        &mut self,
        signer: PublicKey,
        signature: Vec<u8>,
    ) -> Result<(), MultisigError> {
        // Check if signer is authorized
        if !self.account.is_signer(&signer) {
            return Err(MultisigError::UnauthorizedSigner);
        }

        // Check for duplicate signature from same signer
        if self.signatures.iter().any(|s| s.signer == signer) {
            return Err(MultisigError::DuplicateSignature);
        }

        // Verify the signature
        let message = self.signing_hash();
        signer
            .verify(&message, &signature)
            .map_err(|_| MultisigError::InvalidSignature)?;

        self.signatures.push(SignatureEntry { signer, signature });

        Ok(())
    }

    /// Verify all signatures meet threshold requirement
    pub fn verify(&self) -> Result<(), MultisigError> {
        // Check minimum signatures
        if self.signatures.len() < self.account.threshold as usize {
            return Err(MultisigError::InsufficientSignatures {
                required: self.account.threshold,
                provided: self.signatures.len() as u8,
            });
        }

        let message = self.signing_hash();

        // Verify each signature
        for entry in &self.signatures {
            // Verify signer is authorized
            if !self.account.is_signer(&entry.signer) {
                return Err(MultisigError::UnauthorizedSigner);
            }

            // Verify signature
            entry
                .signer
                .verify(&message, &entry.signature)
                .map_err(|_| MultisigError::InvalidSignature)?;
        }

        Ok(())
    }

    /// Calculate transaction hash (unique ID)
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.signing_hash());

        // Include all signatures in hash
        for entry in &self.signatures {
            hasher.update(entry.signer.0);
            hasher.update(&entry.signature);
        }

        hasher.finalize().into()
    }

    /// Get the multisig account address (sender)
    pub fn from(&self) -> PublicKey {
        self.account.address()
    }

    /// Check if transaction has enough signatures
    pub fn is_ready(&self) -> bool {
        self.signatures.len() >= self.account.threshold as usize
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MultisigError {
    NoSigners,
    InvalidThreshold,
    ThresholdTooHigh { threshold: u8, signers: usize },
    DuplicateSigners,
    UnauthorizedSigner,
    DuplicateSignature,
    InvalidSignature,
    InsufficientSignatures { required: u8, provided: u8 },
}

impl std::fmt::Display for MultisigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MultisigError::NoSigners => write!(f, "Multisig account must have at least one signer"),
            MultisigError::InvalidThreshold => write!(f, "Threshold must be at least 1"),
            MultisigError::ThresholdTooHigh { threshold, signers } => {
                write!(
                    f,
                    "Threshold ({}) exceeds number of signers ({})",
                    threshold, signers
                )
            }
            MultisigError::DuplicateSigners => write!(f, "Duplicate signers not allowed"),
            MultisigError::UnauthorizedSigner => {
                write!(f, "Signer not authorized for this account")
            }
            MultisigError::DuplicateSignature => write!(f, "Duplicate signature from same signer"),
            MultisigError::InvalidSignature => write!(f, "Invalid signature"),
            MultisigError::InsufficientSignatures { required, provided } => {
                write!(
                    f,
                    "Insufficient signatures: {} required, {} provided",
                    required, provided
                )
            }
        }
    }
}

impl std::error::Error for MultisigError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::KeyPair;

    #[test]
    fn test_multisig_account_creation() {
        let signer1 = KeyPair::generate();
        let signer2 = KeyPair::generate();
        let signer3 = KeyPair::generate();

        let account = MultisigAccount::new(
            vec![
                signer1.public_key(),
                signer2.public_key(),
                signer3.public_key(),
            ],
            2, // 2-of-3
        )
        .unwrap();

        assert_eq!(account.num_signers(), 3);
        assert_eq!(account.threshold, 2);
        assert!(account.is_signer(&signer1.public_key()));
        assert!(account.is_signer(&signer2.public_key()));
        assert!(account.is_signer(&signer3.public_key()));
    }

    #[test]
    fn test_multisig_account_address_deterministic() {
        let signer1 = KeyPair::generate();
        let signer2 = KeyPair::generate();

        let account1 =
            MultisigAccount::new(vec![signer1.public_key(), signer2.public_key()], 2).unwrap();

        let account2 =
            MultisigAccount::new(vec![signer1.public_key(), signer2.public_key()], 2).unwrap();

        assert_eq!(account1.address(), account2.address());
    }

    #[test]
    fn test_multisig_invalid_threshold() {
        let signer = KeyPair::generate();

        // Zero threshold
        assert!(matches!(
            MultisigAccount::new(vec![signer.public_key()], 0),
            Err(MultisigError::InvalidThreshold)
        ));

        // Threshold too high
        assert!(matches!(
            MultisigAccount::new(vec![signer.public_key()], 5),
            Err(MultisigError::ThresholdTooHigh { .. })
        ));
    }

    #[test]
    fn test_multisig_duplicate_signers() {
        let signer = KeyPair::generate();

        let result = MultisigAccount::new(vec![signer.public_key(), signer.public_key()], 1);

        assert!(matches!(result, Err(MultisigError::DuplicateSigners)));
    }

    #[test]
    fn test_multisig_transaction_2_of_3() {
        let signer1 = KeyPair::generate();
        let signer2 = KeyPair::generate();
        let signer3 = KeyPair::generate();
        let recipient = KeyPair::generate();

        let account = MultisigAccount::new(
            vec![
                signer1.public_key(),
                signer2.public_key(),
                signer3.public_key(),
            ],
            2, // 2-of-3
        )
        .unwrap();

        let mut tx = MultisigTransaction::new(account, recipient.public_key(), 1_000_000, 100, 0);

        // Sign with first signer
        let msg = tx.signing_hash();
        let sig1 = signer1.sign(&msg);
        tx.add_signature(signer1.public_key(), sig1).unwrap();

        // Not ready yet (need 2 signatures)
        assert!(!tx.is_ready());

        // Sign with second signer
        let sig2 = signer2.sign(&msg);
        tx.add_signature(signer2.public_key(), sig2).unwrap();

        // Now ready
        assert!(tx.is_ready());

        // Verify should pass
        assert!(tx.verify().is_ok());
    }

    #[test]
    fn test_multisig_unauthorized_signer() {
        let signer1 = KeyPair::generate();
        let signer2 = KeyPair::generate();
        let unauthorized = KeyPair::generate();
        let recipient = KeyPair::generate();

        let account =
            MultisigAccount::new(vec![signer1.public_key(), signer2.public_key()], 2).unwrap();

        let mut tx = MultisigTransaction::new(account, recipient.public_key(), 1_000_000, 100, 0);

        let msg = tx.signing_hash();
        let sig = unauthorized.sign(&msg);
        let result = tx.add_signature(unauthorized.public_key(), sig);

        assert!(matches!(result, Err(MultisigError::UnauthorizedSigner)));
    }

    #[test]
    fn test_multisig_duplicate_signature() {
        let signer1 = KeyPair::generate();
        let signer2 = KeyPair::generate();
        let recipient = KeyPair::generate();

        let account =
            MultisigAccount::new(vec![signer1.public_key(), signer2.public_key()], 2).unwrap();

        let mut tx = MultisigTransaction::new(account, recipient.public_key(), 1_000_000, 100, 0);

        let msg = tx.signing_hash();
        let sig1 = signer1.sign(&msg);

        tx.add_signature(signer1.public_key(), sig1.clone())
            .unwrap();
        let result = tx.add_signature(signer1.public_key(), sig1);

        assert!(matches!(result, Err(MultisigError::DuplicateSignature)));
    }

    #[test]
    fn test_multisig_insufficient_signatures() {
        let signer1 = KeyPair::generate();
        let signer2 = KeyPair::generate();
        let signer3 = KeyPair::generate();
        let recipient = KeyPair::generate();

        let account = MultisigAccount::new(
            vec![
                signer1.public_key(),
                signer2.public_key(),
                signer3.public_key(),
            ],
            3, // 3-of-3 (all required)
        )
        .unwrap();

        let mut tx = MultisigTransaction::new(account, recipient.public_key(), 1_000_000, 100, 0);

        // Only 2 signatures
        let msg = tx.signing_hash();
        tx.add_signature(signer1.public_key(), signer1.sign(&msg))
            .unwrap();
        tx.add_signature(signer2.public_key(), signer2.sign(&msg))
            .unwrap();

        assert!(matches!(
            tx.verify(),
            Err(MultisigError::InsufficientSignatures {
                required: 3,
                provided: 2
            })
        ));
    }
}
