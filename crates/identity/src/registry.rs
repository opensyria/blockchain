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

    /// Mint a new identity token
    pub fn mint(&mut self, token: IdentityToken) -> Result<(), RegistryError> {
        let token_id = token.id.clone();

        // Check if token ID already exists
        if self.tokens.contains_key(&token_id) {
            return Err(RegistryError::TokenExists);
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

    /// Transfer token to new owner
    pub fn transfer(
        &mut self,
        token_id: &str,
        from: &PublicKey,
        to: &PublicKey,
        block_height: u64,
    ) -> Result<(), RegistryError> {
        // Get token
        let token = self
            .tokens
            .get_mut(token_id)
            .ok_or(RegistryError::TokenNotFound)?;

        // Verify ownership
        if &token.owner != from {
            return Err(RegistryError::NotOwner);
        }

        // Remove from old owner
        if let Some(owner_tokens) = self.owners.get_mut(from) {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryError {
    TokenExists,
    TokenNotFound,
    NotOwner,
    NotAuthorized,
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryError::TokenExists => write!(f, "Token ID already exists"),
            RegistryError::TokenNotFound => write!(f, "Token not found"),
            RegistryError::NotOwner => write!(f, "Not the token owner"),
            RegistryError::NotAuthorized => write!(f, "Not authorized"),
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

        let token = IdentityToken::new(
            "token-001".to_string(),
            owner,
            TokenType::HeritageSite,
            CulturalCategory::Ancient,
            metadata,
        );

        assert!(registry.mint(token).is_ok());
        assert_eq!(registry.total_tokens(), 1);
        assert!(registry.get_token("token-001").is_some());
    }

    #[test]
    fn test_transfer_token() {
        let mut registry = IdentityRegistry::new();
        let owner1 = KeyPair::generate().public_key();
        let owner2 = KeyPair::generate().public_key();
        let metadata = HeritageMetadata::new("Test".to_string(), "Description".to_string(), None);

        let token = IdentityToken::new(
            "token-001".to_string(),
            owner1,
            TokenType::TraditionalCraft,
            CulturalCategory::Modern,
            metadata,
        );

        registry.mint(token).unwrap();

        // Transfer
        assert!(registry
            .transfer("token-001", &owner1, &owner2, 100)
            .is_ok());

        // Verify new owner
        let token = registry.get_token("token-001").unwrap();
        assert_eq!(token.owner, owner2);

        // Verify owner collections
        assert_eq!(registry.get_tokens_by_owner(&owner1).len(), 0);
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

        let token = IdentityToken::new(
            "token-001".to_string(),
            owner,
            TokenType::HeritageSite,
            CulturalCategory::Ancient,
            metadata,
        );

        registry.mint(token).unwrap();

        let results = registry.search_by_tag("ancient");
        assert_eq!(results.len(), 1);
    }
}
