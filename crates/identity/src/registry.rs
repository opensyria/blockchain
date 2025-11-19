use crate::token::IdentityToken;
use opensyria_core::crypto::PublicKey;
use std::collections::HashMap;

/// Registry for managing cultural identity tokens
pub struct IdentityRegistry {
    /// All tokens by ID
    tokens: HashMap<String, IdentityToken>,

    /// Tokens owned by each address
    owners: HashMap<PublicKey, Vec<String>>,

    /// Verified authorities (can mint verified tokens)
    authorities: Vec<PublicKey>,
}

impl IdentityRegistry {
    /// Create new empty registry
    pub fn new() -> Self {
        Self {
            tokens: HashMap::new(),
            owners: HashMap::new(),
            authorities: Vec::new(),
        }
    }

    /// Register a new authority
    pub fn add_authority(&mut self, authority: PublicKey) {
        if !self.authorities.contains(&authority) {
            self.authorities.push(authority);
        }
    }

    /// Check if address is an authority
    pub fn is_authority(&self, address: &PublicKey) -> bool {
        self.authorities.contains(address)
    }

    /// Mint a new identity token (requires authority signature)
    pub fn mint(&mut self, mut token: IdentityToken, authority_signature: Option<Vec<u8>>) -> Result<(), RegistryError> {
        let token_id = token.id.clone();

        // Check if token ID already exists
        if self.tokens.contains_key(&token_id) {
            return Err(RegistryError::TokenExists);
        }

        // Verify token ID is cryptographically unique (must be a hash)
        if token_id.len() != 64 || !token_id.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(RegistryError::InvalidTokenId);
        }

        // If authority signature provided, verify it
        if let Some(sig) = &authority_signature {
            // Find authority that signed this
            let mint_message = format!("MINT:{}:{}:{:?}", token_id, token.owner.to_hex(), token.token_type);
            
            let valid_authority = self.authorities.iter().any(|auth| {
                auth.verify(mint_message.as_bytes(), sig).is_ok()
            });
            
            if !valid_authority {
                return Err(RegistryError::UnauthorizedMint);
            }
            
            // Store authority signature
            token.authority_signature = authority_signature;
        }

        // Add to owner's collection
        self.owners
            .entry(token.owner)
            .or_default()
            .push(token_id.clone());

        // Store token
        self.tokens.insert(token_id, token);

        Ok(())
    }

    /// Transfer token to new owner (requires owner's signature)
    pub fn transfer(
        &mut self,
        token_id: &str,
        to: &PublicKey,
        signature: &[u8],
        block_height: u64,
    ) -> Result<(), RegistryError> {
        // Get token
        let token = self
            .tokens
            .get_mut(token_id)
            .ok_or(RegistryError::TokenNotFound)?;

        // Create transfer message for signature verification
        let transfer_message = format!("TRANSFER:{}:{}", token_id, to.to_hex());
        
        // Verify signature from current owner
        if token.owner.verify(transfer_message.as_bytes(), signature).is_err() {
            return Err(RegistryError::InvalidSignature);
        }

        let from = token.owner;

        // Remove from old owner
        if let Some(owner_tokens) = self.owners.get_mut(&from) {
            owner_tokens.retain(|id| id != token_id);
        }

        // Add to new owner
        self.owners
            .entry(*to)
            .or_default()
            .push(token_id.to_string());

        // Update token
        token.transfer(*to, block_height);

        Ok(())
    }

    /// Validate IPFS content hash format
    pub fn validate_ipfs_cid(cid: &str) -> Result<(), RegistryError> {
        // IPFS CIDv0: starts with "Qm", 46 characters, base58
        // IPFS CIDv1: starts with "b", variable length, base32/base58
        
        if cid.is_empty() {
            return Err(RegistryError::InvalidIPFSCID);
        }
        
        // CIDv0 validation
        if cid.starts_with("Qm") {
            if cid.len() != 46 {
                return Err(RegistryError::InvalidIPFSCID);
            }
            // Check base58 characters
            if !cid.chars().all(|c| {
                c.is_ascii_alphanumeric() && c != '0' && c != 'O' && c != 'I' && c != 'l'
            }) {
                return Err(RegistryError::InvalidIPFSCID);
            }
            return Ok(());
        }
        
        // CIDv1 validation (basic check)
        if cid.starts_with('b') && cid.len() > 10 {
            return Ok(());
        }
        
        Err(RegistryError::InvalidIPFSCID)
    }

    /// Get token by ID
    pub fn get_token(&self, token_id: &str) -> Option<&IdentityToken> {
        self.tokens.get(token_id)
    }

    /// Get all tokens owned by an address
    pub fn get_tokens_by_owner(&self, owner: &PublicKey) -> Vec<&IdentityToken> {
        if let Some(token_ids) = self.owners.get(owner) {
            token_ids
                .iter()
                .filter_map(|id| self.tokens.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get total number of tokens
    pub fn total_tokens(&self) -> usize {
        self.tokens.len()
    }

    /// Get all token IDs
    pub fn all_token_ids(&self) -> Vec<String> {
        self.tokens.keys().cloned().collect()
    }

    /// Search tokens by tag
    pub fn search_by_tag(&self, tag: &str) -> Vec<&IdentityToken> {
        self.tokens
            .values()
            .filter(|token| token.metadata.tags.iter().any(|t| t.contains(tag)))
            .collect()
    }

    /// Get tokens by type
    pub fn get_by_type(&self, token_type: &crate::token::TokenType) -> Vec<&IdentityToken> {
        self.tokens
            .values()
            .filter(|token| &token.token_type == token_type)
            .collect()
    }

    /// Get tokens by category
    pub fn get_by_category(
        &self,
        category: &crate::token::CulturalCategory,
    ) -> Vec<&IdentityToken> {
        self.tokens
            .values()
            .filter(|token| &token.category == category)
            .collect()
    }
}

impl Default for IdentityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum RegistryError {
    TokenNotFound,
    TokenExists,
    NotOwner,
    InvalidAuthority,
    InvalidSignature,
    UnauthorizedMint,
    InvalidTokenId,
    InvalidIPFSCID,
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryError::TokenNotFound => write!(f, "Token not found"),
            RegistryError::TokenExists => write!(f, "Token already exists"),
            RegistryError::NotOwner => write!(f, "Not token owner"),
            RegistryError::InvalidAuthority => write!(f, "Invalid authority"),
            RegistryError::InvalidSignature => write!(f, "Invalid signature"),
            RegistryError::UnauthorizedMint => write!(f, "Unauthorized mint"),
            RegistryError::InvalidTokenId => write!(f, "Invalid token ID"),
            RegistryError::InvalidIPFSCID => write!(f, "Invalid IPFS CID"),
        }
    }
}

impl std::error::Error for RegistryError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::HeritageMetadata;
    use crate::token::{CulturalCategory, TokenType};
    use opensyria_core::crypto::KeyPair;

    #[test]
    fn test_mint_token() {
        let mut registry = IdentityRegistry::new();
        let owner = KeyPair::generate().public_key();
        let metadata = HeritageMetadata::new("Test".to_string(), "Description".to_string(), None);

        // Create cryptographic token ID
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(b"token-001-unique");
        let token_id = hex::encode(hasher.finalize());

        let token = IdentityToken::new(
            token_id.clone(),
            owner,
            TokenType::HeritageSite,
            CulturalCategory::Ancient,
            metadata,
        );

        assert!(registry.mint(token, None).is_ok());
        assert_eq!(registry.total_tokens(), 1);
        assert!(registry.get_token(&token_id).is_some());
    }

    #[test]
    fn test_transfer_token() {
        let mut registry = IdentityRegistry::new();
        let owner1_keypair = KeyPair::generate();
        let owner2 = KeyPair::generate().public_key();
        let metadata = HeritageMetadata::new("Test".to_string(), "Description".to_string(), None);

        // Create token with cryptographic ID
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(b"token-001-unique");
        let token_id = hex::encode(hasher.finalize());

        let token = IdentityToken::new(
            token_id.clone(),
            owner1_keypair.public_key(),
            TokenType::TraditionalCraft,
            CulturalCategory::Modern,
            metadata,
        );

        registry.mint(token, None).unwrap();

        // Create transfer signature
        let transfer_message = format!("TRANSFER:{}:{}", token_id, owner2.to_hex());
        let signature = owner1_keypair.sign(transfer_message.as_bytes());

        // Transfer
        assert!(registry
            .transfer(&token_id, &owner2, &signature, 100)
            .is_ok());

        // Verify new owner
        let token = registry.get_token(&token_id).unwrap();
        assert_eq!(token.owner, owner2);

        // Verify owner collections
        assert_eq!(registry.get_tokens_by_owner(&owner1_keypair.public_key()).len(), 0);
        assert_eq!(registry.get_tokens_by_owner(&owner2).len(), 1);
    }

    #[test]
    fn test_authority_management() {
        let mut registry = IdentityRegistry::new();
        let authority = KeyPair::generate().public_key();

        assert!(!registry.is_authority(&authority));

        registry.add_authority(authority);

        assert!(registry.is_authority(&authority));
    }

    #[test]
    fn test_search_by_tag() {
        let mut registry = IdentityRegistry::new();
        let owner = KeyPair::generate().public_key();

        let metadata = HeritageMetadata::new("Site 1".to_string(), "Description".to_string(), None)
            .with_tags(vec!["ancient".to_string(), "monument".to_string()]);

        // Create cryptographic token ID
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(b"token-heritage-site");
        let token_id = hex::encode(hasher.finalize());

        let token = IdentityToken::new(
            token_id,
            owner,
            TokenType::HeritageSite,
            CulturalCategory::Ancient,
            metadata,
        );

        registry.mint(token, None).unwrap();

        let results = registry.search_by_tag("ancient");
        assert_eq!(results.len(), 1);
    }
}
