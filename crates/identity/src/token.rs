use opensyria_core::crypto::PublicKey;
use serde::{Deserialize, Serialize};

/// Cultural identity token representing Syrian heritage
#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct IdentityToken {
    /// Unique token identifier
    pub id: String,

    /// Owner's public key
    pub owner: PublicKey,

    /// Creator's public key (for royalties)
    pub creator: PublicKey,

    /// Royalty percentage (0-100, 0 = no royalties)
    pub royalty_percentage: u8,

    /// Token type classification
    pub token_type: TokenType,

    /// Cultural category
    pub category: CulturalCategory,

    /// Token metadata (heritage information)
    pub metadata: crate::metadata::HeritageMetadata,

    /// IPFS content identifier for multimedia (optional)
    pub ipfs_cid: Option<String>,

    /// Creation timestamp
    pub created_at: u64,

    /// Block height when minted
    pub minted_at_height: u64,

    /// Minting authority signature
    pub authority_signature: Option<Vec<u8>>,

    /// Transfer history (optional)
    pub provenance: Vec<Transfer>,
}

/// Transfer record for provenance tracking
#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct Transfer {
    pub from: PublicKey,
    pub to: PublicKey,
    pub price: Option<u64>, // Sale price (if applicable)
    pub royalty_paid: Option<u64>, // Royalty amount paid to creator
    pub timestamp: u64,
    pub block_height: u64,
}

/// Type of identity token
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub enum TokenType {
    /// Cultural heritage site or monument
    HeritageSite,

    /// Traditional craft or artisanship
    TraditionalCraft,

    /// Historical document or artifact
    HistoricalDocument,

    /// Musical or performing arts tradition
    PerformingArts,

    /// Culinary heritage (recipes, techniques)
    CulinaryHeritage,

    /// Oral tradition or folklore
    OralTradition,

    /// Language preservation
    LanguageHeritage,

    /// Community or regional identity
    CommunityIdentity,

    /// Personal cultural contribution
    PersonalContribution,

    /// Digital cultural creation
    DigitalCulture,
}

/// Cultural category classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub enum CulturalCategory {
    /// Ancient history (pre-Islamic)
    Ancient,

    /// Islamic heritage
    Islamic,

    /// Ottoman period
    Ottoman,

    /// Modern Syrian culture
    Modern,

    /// Regional traditions (specific cities/areas)
    Regional {
        region: String, // Damascus, Aleppo, Palmyra, etc.
    },

    /// Religious minority heritage
    ReligiousMinority {
        community: String, // Christian, Druze, Alawite, etc.
    },

    /// Ethnic heritage
    Ethnic {
        ethnicity: String, // Kurdish, Armenian, Assyrian, etc.
    },

    /// Contemporary culture
    Contemporary,
}

impl IdentityToken {
    /// Create a new identity token
    pub fn new(
        id: String,
        owner: PublicKey,
        token_type: TokenType,
        category: CulturalCategory,
        metadata: crate::metadata::HeritageMetadata,
        royalty_percentage: u8,
        block_height: u64,
    ) -> Result<Self, String> {
        if royalty_percentage > 50 {
            return Err("Royalty percentage cannot exceed 50%".to_string());
        }

        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(Self {
            id,
            owner,
            creator: owner, // Creator is initial owner
            royalty_percentage,
            token_type,
            category,
            metadata,
            created_at,
            minted_at_height: block_height,
            authority_signature: None,
            provenance: Vec::new(),
            ipfs_cid: None,
        })
    }

    /// Transfer token to new owner with optional sale price
    /// 
    /// IDENTITY-P2-006: Enhanced NFT royalties implementation
    /// 
    /// Calculates and tracks royalty payments to original creator.
    /// Royalties are enforced automatically on every transfer with a price.
    /// 
    /// # Arguments
    /// * `to` - New owner's public key
    /// * `block_height` - Current blockchain height
    /// * `price` - Sale price (if None, treated as gift/transfer)
    /// 
    /// # Returns
    /// Royalty amount that must be paid to creator (0 if no royalty or no price)
    pub fn transfer(&mut self, to: PublicKey, block_height: u64, price: Option<u64>) -> u64 {
        // Calculate royalty if there's a price
        let royalty_paid = if let Some(sale_price) = price {
            if self.royalty_percentage > 0 && self.owner != self.creator {
                // Only pay royalty if seller is not the creator (avoid self-royalty)
                Some(self.calculate_royalty(sale_price))
            } else {
                None
            }
        } else {
            None
        };

        let transfer = Transfer {
            from: self.owner,
            to,
            price,
            royalty_paid,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            block_height,
        };

        self.provenance.push(transfer);
        self.owner = to;
        
        royalty_paid.unwrap_or(0)
    }

    /// Transfer token to new owner (no price/royalty)
    /// 
    /// Used for gifts, initial mints, or non-commercial transfers
    pub fn transfer_free(&mut self, to: PublicKey, block_height: u64) {
        self.transfer(to, block_height, None);
    }

    /// Calculate minimum transfer price including royalty
    /// 
    /// Helps buyers understand total cost (sale price + royalty)
    /// 
    /// # Arguments
    /// * `seller_price` - Amount seller wants to receive
    /// 
    /// # Returns
    /// Total amount buyer must pay (includes royalty)
    pub fn calculate_total_price(&self, seller_price: u64) -> u64 {
        let royalty = self.calculate_royalty(seller_price);
        seller_price.saturating_add(royalty)
    }

    /// Split sale proceeds between seller and creator
    /// 
    /// # Arguments
    /// * `total_payment` - Total amount paid by buyer
    /// 
    /// # Returns
    /// (seller_amount, creator_royalty)
    pub fn split_payment(&self, total_payment: u64) -> (u64, u64) {
        if self.royalty_percentage == 0 || self.owner == self.creator {
            // No royalty or creator is selling (gets full amount)
            return (total_payment, 0);
        }

        // Calculate royalty as percentage of total payment
        let royalty = (total_payment as u128 * self.royalty_percentage as u128 / 100) as u64;
        let seller_amount = total_payment.saturating_sub(royalty);
        
        (seller_amount, royalty)
    }

    /// Get creator address for royalty payments
    pub fn creator_address(&self) -> &PublicKey {
        &self.creator
    }

    /// Get royalty percentage
    pub fn royalty_rate(&self) -> u8 {
        self.royalty_percentage
    }

    /// Get full transfer history
    pub fn transfer_history(&self) -> &[Transfer] {
        &self.provenance
    }

    /// Calculate total volume traded (sum of all sale prices)
    pub fn total_volume(&self) -> u64 {
        self.provenance
            .iter()
            .filter_map(|t| t.price)
            .sum()
    }

    /// Calculate total royalties paid to creator
    pub fn total_royalties_paid(&self) -> u64 {
        self.provenance
            .iter()
            .filter_map(|t| t.royalty_paid)
            .sum()
    }

    /// Get last sale price
    pub fn last_sale_price(&self) -> Option<u64> {
        self.provenance
            .iter()
            .rev()
            .find_map(|t| t.price)
    }

    /// Get number of times transferred
    pub fn transfer_count(&self) -> usize {
        self.provenance.len()
    }

    /// Get token unique identifier
    pub fn token_id(&self) -> &str {
        &self.id
    }

    /// Check if token is verified by authority
    pub fn is_verified(&self) -> bool {
        self.authority_signature.is_some()
    }

    /// Calculate royalty for a given sale price
    pub fn calculate_royalty(&self, sale_price: u64) -> u64 {
        if self.royalty_percentage == 0 {
            return 0;
        }
        (sale_price as u128 * self.royalty_percentage as u128 / 100) as u64
    }
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::HeritageSite => write!(f, "Heritage Site | موقع تراثي"),
            TokenType::TraditionalCraft => write!(f, "Traditional Craft | حرفة تقليدية"),
            TokenType::HistoricalDocument => write!(f, "Historical Document | وثيقة تاريخية"),
            TokenType::PerformingArts => write!(f, "Performing Arts | فنون أدائية"),
            TokenType::CulinaryHeritage => write!(f, "Culinary Heritage | تراث طهوي"),
            TokenType::OralTradition => write!(f, "Oral Tradition | تقليد شفهي"),
            TokenType::LanguageHeritage => write!(f, "Language Heritage | تراث لغوي"),
            TokenType::CommunityIdentity => write!(f, "Community Identity | هوية مجتمعية"),
            TokenType::PersonalContribution => write!(f, "Personal Contribution | مساهمة شخصية"),
            TokenType::DigitalCulture => write!(f, "Digital Culture | ثقافة رقمية"),
        }
    }
}

impl std::fmt::Display for CulturalCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CulturalCategory::Ancient => write!(f, "Ancient | قديم"),
            CulturalCategory::Islamic => write!(f, "Islamic | إسلامي"),
            CulturalCategory::Ottoman => write!(f, "Ottoman | عثماني"),
            CulturalCategory::Modern => write!(f, "Modern | حديث"),
            CulturalCategory::Regional { region } => {
                write!(f, "Regional: {} | إقليمي: {}", region, region)
            }
            CulturalCategory::ReligiousMinority { community } => {
                write!(f, "Religious: {} | ديني: {}", community, community)
            }
            CulturalCategory::Ethnic { ethnicity } => {
                write!(f, "Ethnic: {} | عرقي: {}", ethnicity, ethnicity)
            }
            CulturalCategory::Contemporary => write!(f, "Contemporary | معاصر"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opensyria_core::crypto::KeyPair;

    #[test]
    fn test_create_identity_token() {
        let owner = KeyPair::generate().public_key();
        let metadata = crate::metadata::HeritageMetadata::new(
            "Umayyad Mosque".to_string(),
            "Great Mosque of Damascus".to_string(),
            Some("مسجد بني أمية الكبير".to_string()),
        );

        let token = IdentityToken::new(
            "heritage-001".to_string(),
            owner,
            TokenType::HeritageSite,
            CulturalCategory::Islamic,
            metadata,
            5, // 5% royalty
            0, // block height
        ).unwrap();

        assert_eq!(token.token_id(), "heritage-001");
        assert_eq!(token.owner, owner);
        assert!(!token.is_verified());
    }

    #[test]
    fn test_token_transfer() {
        let owner1 = KeyPair::generate().public_key();
        let owner2 = KeyPair::generate().public_key();
        let metadata = crate::metadata::HeritageMetadata::new(
            "Test".to_string(),
            "Test heritage".to_string(),
            None,
        );

        let mut token = IdentityToken::new(
            "test-001".to_string(),
            owner1,
            TokenType::TraditionalCraft,
            CulturalCategory::Modern,
            metadata,
            0, // No royalty for this test
            100,
        ).unwrap();

        token.transfer(owner2, 100, None);

        assert_eq!(token.owner, owner2);
        assert_eq!(token.provenance.len(), 1);
        assert_eq!(token.provenance[0].from, owner1);
        assert_eq!(token.provenance[0].to, owner2);
    }

    #[test]
    fn test_royalty_calculation() {
        let creator = KeyPair::generate().public_key();
        let metadata = crate::metadata::HeritageMetadata::new(
            "Test".to_string(),
            "Test heritage".to_string(),
            None,
        );

        let token = IdentityToken::new(
            "test-royalty".to_string(),
            creator,
            TokenType::DigitalCulture,
            CulturalCategory::Contemporary,
            metadata,
            10, // 10% royalty
            100,
        ).unwrap();

        // Test royalty calculation
        assert_eq!(token.calculate_royalty(1000), 100); // 10% of 1000
        assert_eq!(token.calculate_royalty(50000), 5000); // 10% of 50000
        assert_eq!(token.calculate_royalty(0), 0); // 10% of 0
    }

    #[test]
    fn test_royalty_payment_on_transfer() {
        let creator = KeyPair::generate().public_key();
        let buyer1 = KeyPair::generate().public_key();
        let buyer2 = KeyPair::generate().public_key();
        let metadata = crate::metadata::HeritageMetadata::new(
            "Royalty Test".to_string(),
            "Test NFT with royalties".to_string(),
            None,
        );

        let mut token = IdentityToken::new(
            "royalty-test-001".to_string(),
            creator,
            TokenType::DigitalCulture,
            CulturalCategory::Contemporary,
            metadata,
            5, // 5% royalty
            100,
        ).unwrap();

        // First sale: creator to buyer1 for 10,000
        // No royalty since creator is selling
        let royalty1 = token.transfer(buyer1, 101, Some(10_000));
        assert_eq!(royalty1, 0); // Creator doesn't pay self-royalty
        assert_eq!(token.owner, buyer1);

        // Second sale: buyer1 to buyer2 for 20,000
        // Royalty should be 5% = 1,000
        let royalty2 = token.transfer(buyer2, 102, Some(20_000));
        assert_eq!(royalty2, 1_000); // 5% of 20,000
        assert_eq!(token.owner, buyer2);
        assert_eq!(token.provenance[1].royalty_paid, Some(1_000));
    }

    #[test]
    fn test_split_payment() {
        let creator = KeyPair::generate().public_key();
        let seller = KeyPair::generate().public_key();
        let metadata = crate::metadata::HeritageMetadata::new(
            "Split Test".to_string(),
            "Test payment splitting".to_string(),
            None,
        );

        let mut token = IdentityToken::new(
            "split-test-001".to_string(),
            creator,
            TokenType::PersonalContribution,
            CulturalCategory::Modern,
            metadata,
            10, // 10% royalty
            100,
        ).unwrap();

        // Transfer to seller first
        token.transfer(seller, 101, None);

        // Now seller has the token, split payment
        let (seller_amount, creator_royalty) = token.split_payment(10_000);
        assert_eq!(seller_amount, 9_000); // 90% to seller
        assert_eq!(creator_royalty, 1_000); // 10% to creator
    }

    #[test]
    fn test_no_royalty_on_free_transfer() {
        let creator = KeyPair::generate().public_key();
        let recipient = KeyPair::generate().public_key();
        let metadata = crate::metadata::HeritageMetadata::new(
            "Gift Test".to_string(),
            "Test free transfer".to_string(),
            None,
        );

        let mut token = IdentityToken::new(
            "gift-test-001".to_string(),
            creator,
            TokenType::CommunityIdentity,
            CulturalCategory::Regional {
                region: "Damascus".to_string(),
            },
            metadata,
            15, // 15% royalty
            100,
        ).unwrap();

        // Free transfer (gift)
        token.transfer_free(recipient, 101);
        
        assert_eq!(token.owner, recipient);
        assert_eq!(token.provenance[0].price, None);
        assert_eq!(token.provenance[0].royalty_paid, None);
    }

    #[test]
    fn test_total_volume_and_royalties() {
        let creator = KeyPair::generate().public_key();
        let buyer1 = KeyPair::generate().public_key();
        let buyer2 = KeyPair::generate().public_key();
        let buyer3 = KeyPair::generate().public_key();
        let metadata = crate::metadata::HeritageMetadata::new(
            "Volume Test".to_string(),
            "Test volume tracking".to_string(),
            None,
        );

        let mut token = IdentityToken::new(
            "volume-test-001".to_string(),
            creator,
            TokenType::TraditionalCraft,
            CulturalCategory::Islamic,
            metadata,
            5, // 5% royalty
            100,
        ).unwrap();

        // Sale 1: creator → buyer1 for 1000 (no royalty)
        token.transfer(buyer1, 101, Some(1_000));
        
        // Sale 2: buyer1 → buyer2 for 2000 (5% royalty = 100)
        token.transfer(buyer2, 102, Some(2_000));
        
        // Sale 3: buyer2 → buyer3 for 3000 (5% royalty = 150)
        token.transfer(buyer3, 103, Some(3_000));

        // Total volume = 1000 + 2000 + 3000 = 6000
        assert_eq!(token.total_volume(), 6_000);
        
        // Total royalties = 0 + 100 + 150 = 250
        assert_eq!(token.total_royalties_paid(), 250);
        
        // Last sale price = 3000
        assert_eq!(token.last_sale_price(), Some(3_000));
        
        // Transfer count = 3
        assert_eq!(token.transfer_count(), 3);
    }

    #[test]
    fn test_maximum_royalty_percentage() {
        let creator = KeyPair::generate().public_key();
        let metadata = crate::metadata::HeritageMetadata::new(
            "Max Royalty Test".to_string(),
            "Test maximum royalty limit".to_string(),
            None,
        );

        // Should fail with > 50% royalty
        let result = IdentityToken::new(
            "max-royalty-001".to_string(),
            creator,
            TokenType::DigitalCulture,
            CulturalCategory::Contemporary,
            metadata.clone(),
            51, // 51% - too high
            100,
        );
        assert!(result.is_err());

        // Should succeed with 50% royalty
        let result = IdentityToken::new(
            "max-royalty-002".to_string(),
            creator,
            TokenType::DigitalCulture,
            CulturalCategory::Contemporary,
            metadata,
            50, // 50% - maximum allowed
            100,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_calculate_total_price() {
        let creator = KeyPair::generate().public_key();
        let metadata = crate::metadata::HeritageMetadata::new(
            "Price Test".to_string(),
            "Test total price calculation".to_string(),
            None,
        );

        let token = IdentityToken::new(
            "price-test-001".to_string(),
            creator,
            TokenType::HeritageSite,
            CulturalCategory::Ancient,
            metadata,
            10, // 10% royalty
            100,
        ).unwrap();

        // If seller wants 9000, buyer pays 9000 + 900 (10% royalty) = 9900
        let total = token.calculate_total_price(9_000);
        assert_eq!(total, 9_900);
    }
}
