use opensyria_core::crypto::PublicKey;
use serde::{Deserialize, Serialize};

/// Cultural identity token representing Syrian heritage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityToken {
    /// Unique token identifier
    pub id: String,

    /// Owner's public key
    pub owner: PublicKey,

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

    /// Minting authority signature
    pub authority_signature: Option<Vec<u8>>,

    /// Transfer history (optional)
    pub provenance: Vec<Transfer>,
}

/// Transfer record for provenance tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transfer {
    pub from: PublicKey,
    pub to: PublicKey,
    pub timestamp: u64,
    pub block_height: u64,
}

/// Type of identity token
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    ) -> Self {
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id,
            owner,
            token_type,
            category,
            metadata,
            created_at,
            authority_signature: None,
            provenance: Vec::new(),
            ipfs_cid: None,
        }
    }

    /// Transfer token to new owner
    pub fn transfer(&mut self, to: PublicKey, block_height: u64) {
        let transfer = Transfer {
            from: self.owner,
            to,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            block_height,
        };

        self.provenance.push(transfer);
        self.owner = to;
    }

    /// Get token unique identifier
    pub fn token_id(&self) -> &str {
        &self.id
    }

    /// Check if token is verified by authority
    pub fn is_verified(&self) -> bool {
        self.authority_signature.is_some()
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
        );

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
        );

        token.transfer(owner2, 100);

        assert_eq!(token.owner, owner2);
        assert_eq!(token.provenance.len(), 1);
        assert_eq!(token.provenance[0].from, owner1);
        assert_eq!(token.provenance[0].to, owner2);
    }
}
