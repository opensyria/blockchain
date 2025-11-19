use serde::{Deserialize, Serialize};

/// Heritage metadata for cultural tokens
#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct HeritageMetadata {
    /// Primary name (English/transliterated)
    pub name: String,

    /// Arabic name
    pub name_ar: Option<String>,

    /// Detailed description
    pub description: String,

    /// Arabic description
    pub description_ar: Option<String>,

    /// Location information
    pub location: Option<Location>,

    /// Historical period/date
    pub historical_period: Option<String>,

    /// UNESCO status
    pub unesco_status: Option<UNESCOStatus>,

    /// Languages associated
    pub languages: Vec<Language>,

    /// Tags for categorization
    pub tags: Vec<String>,

    /// External references (URLs, documents)
    pub references: Vec<String>,

    /// IPFS hash for multimedia content
    pub content_hash: Option<String>,

    /// Creator/curator information
    pub creator: Option<String>,

    /// License information
    pub license: Option<String>,
}

/// Geographic location
#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct Location {
    /// City
    pub city: String,

    /// Arabic city name
    pub city_ar: Option<String>,

    /// Governorate/Province
    pub governorate: Option<String>,

    /// Coordinates (latitude, longitude)
    pub coordinates: Option<(f64, f64)>,

    /// Address or specific location
    pub address: Option<String>,
}

/// UNESCO heritage status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub enum UNESCOStatus {
    /// World Heritage Site
    WorldHeritage,

    /// Intangible Cultural Heritage
    IntangibleHeritage,

    /// Memory of the World
    MemoryOfWorld,

    /// Endangered
    Endangered,

    /// Tentative list
    Tentative,
}

/// Language classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub enum Language {
    /// Modern Standard Arabic
    Arabic,

    /// Syrian Arabic dialect
    SyrianArabic,

    /// Kurdish
    Kurdish,

    /// Armenian
    Armenian,

    /// Assyrian/Aramaic
    Aramaic,

    /// Circassian
    Circassian,

    /// Turkish
    Turkish,

    /// French (historical)
    French,

    /// English
    English,

    /// Other language
    Other(String),
}

impl HeritageMetadata {
    /// Create new metadata with minimal information
    pub fn new(name: String, description: String, name_ar: Option<String>) -> Self {
        Self {
            name,
            name_ar,
            description,
            description_ar: None,
            location: None,
            historical_period: None,
            unesco_status: None,
            languages: vec![Language::Arabic],
            tags: Vec::new(),
            references: Vec::new(),
            content_hash: None,
            creator: None,
            license: None,
        }
    }

    /// Add location information
    pub fn with_location(mut self, location: Location) -> Self {
        self.location = Some(location);
        self
    }

    /// Add historical period
    pub fn with_period(mut self, period: String) -> Self {
        self.historical_period = Some(period);
        self
    }

    /// Add UNESCO status
    pub fn with_unesco_status(mut self, status: UNESCOStatus) -> Self {
        self.unesco_status = Some(status);
        self
    }

    /// Add tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Add content hash (IPFS)
    pub fn with_content_hash(mut self, hash: String) -> Self {
        self.content_hash = Some(hash);
        self
    }

    /// Add creator information
    pub fn with_creator(mut self, creator: String) -> Self {
        self.creator = Some(creator);
        self
    }

    /// Add license
    pub fn with_license(mut self, license: String) -> Self {
        self.license = Some(license);
        self
    }
}

impl Location {
    /// Create new location
    pub fn new(city: String, city_ar: Option<String>) -> Self {
        Self {
            city,
            city_ar,
            governorate: None,
            coordinates: None,
            address: None,
        }
    }

    /// Add coordinates
    pub fn with_coordinates(mut self, lat: f64, lon: f64) -> Self {
        self.coordinates = Some((lat, lon));
        self
    }

    /// Add governorate
    pub fn with_governorate(mut self, governorate: String) -> Self {
        self.governorate = Some(governorate);
        self
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::Arabic => write!(f, "Arabic | عربي"),
            Language::SyrianArabic => write!(f, "Syrian Arabic | عربي سوري"),
            Language::Kurdish => write!(f, "Kurdish | كردي"),
            Language::Armenian => write!(f, "Armenian | أرمني"),
            Language::Aramaic => write!(f, "Aramaic | آرامي"),
            Language::Circassian => write!(f, "Circassian | شركسي"),
            Language::Turkish => write!(f, "Turkish | تركي"),
            Language::French => write!(f, "French | فرنسي"),
            Language::English => write!(f, "English | إنجليزي"),
            Language::Other(lang) => write!(f, "{}", lang),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_metadata() {
        let metadata = HeritageMetadata::new(
            "Palmyra".to_string(),
            "Ancient city".to_string(),
            Some("تدمر".to_string()),
        );

        assert_eq!(metadata.name, "Palmyra");
        assert_eq!(metadata.name_ar, Some("تدمر".to_string()));
    }

    #[test]
    fn test_location_with_coordinates() {
        let location = Location::new("Damascus".to_string(), Some("دمشق".to_string()))
            .with_coordinates(33.5138, 36.2765)
            .with_governorate("Damascus".to_string());

        assert_eq!(location.city, "Damascus");
        assert!(location.coordinates.is_some());
    }

    #[test]
    fn test_metadata_builder() {
        let metadata =
            HeritageMetadata::new("Test Site".to_string(), "Description".to_string(), None)
                .with_unesco_status(UNESCOStatus::WorldHeritage)
                .with_period("Ancient".to_string())
                .with_tags(vec!["archaeology".to_string(), "monument".to_string()]);

        assert_eq!(metadata.unesco_status, Some(UNESCOStatus::WorldHeritage));
        assert_eq!(metadata.tags.len(), 2);
    }
}
