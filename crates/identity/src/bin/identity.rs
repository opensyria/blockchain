use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use opensyria_core::crypto::KeyPair;
use opensyria_identity::ipfs::IpfsClient;
use opensyria_identity::*;

#[derive(Parser)]
#[command(name = "identity")]
#[command(about = "Syrian Cultural Identity Manager | مدير الهوية الثقافية السورية", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new cultural identity token | إنشاء رمز هوية ثقافية جديد
    Create {
        /// Token ID
        #[arg(short, long)]
        id: String,

        /// Token name
        #[arg(short, long)]
        name: String,

        /// Arabic name
        #[arg(long)]
        name_ar: Option<String>,

        /// Description
        #[arg(short, long)]
        description: String,

        /// Token type
        #[arg(short, long)]
        token_type: String,

        /// Cultural category
        #[arg(short, long)]
        category: String,

        /// City/location
        #[arg(long)]
        city: Option<String>,

        /// Historical period
        #[arg(long)]
        period: Option<String>,

        /// Tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,
    },

    /// List all identity tokens | عرض جميع رموز الهوية
    List {
        /// Filter by owner address
        #[arg(short, long)]
        owner: Option<String>,

        /// Filter by type
        #[arg(short, long)]
        type_filter: Option<String>,
    },

    /// Show token details | عرض تفاصيل الرمز
    Info {
        /// Token ID
        token_id: String,
    },

    /// Upload content to IPFS | رفع محتوى إلى IPFS
    Upload {
        /// File path to upload
        #[arg(short, long)]
        file: String,

        /// Optional token ID to associate with
        #[arg(short, long)]
        token_id: Option<String>,

        /// IPFS API URL (default: http://127.0.0.1:5001)
        #[arg(long)]
        api_url: Option<String>,

        /// IPFS Gateway URL (default: http://127.0.0.1:8080)
        #[arg(long)]
        gateway_url: Option<String>,
    },

    /// Retrieve content from IPFS | استرجاع محتوى من IPFS
    Retrieve {
        /// IPFS CID to retrieve
        cid: String,

        /// Output file path
        #[arg(short, long)]
        output: String,

        /// IPFS API URL (default: http://127.0.0.1:5001)
        #[arg(long)]
        api_url: Option<String>,

        /// IPFS Gateway URL (default: http://127.0.0.1:8080)
        #[arg(long)]
        gateway_url: Option<String>,
    },

    /// Link IPFS content to an identity token | ربط محتوى IPFS برمز هوية
    Link {
        /// Token ID
        #[arg(short, long)]
        token_id: String,

        /// IPFS CID
        #[arg(short, long)]
        cid: String,
    },

    /// Show examples | عرض أمثلة
    Examples,
}

fn main() -> Result<()> {
    tokio::runtime::Runtime::new()?.block_on(async_main())
}

async fn async_main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create {
            id,
            name,
            name_ar,
            description,
            token_type,
            category,
            city,
            period,
            tags,
        } => {
            println!("{}", "═".repeat(60).cyan());
            println!("{}", "  Creating Cultural Identity Token  ".cyan().bold());
            println!("{}", "═".repeat(60).cyan());
            println!();

            // Parse token type
            let token_type = parse_token_type(&token_type)?;

            // Parse category
            let category = parse_category(&category)?;

            // Create metadata
            let mut metadata = HeritageMetadata::new(name.clone(), description.clone(), name_ar);

            if let Some(city_name) = city {
                let location = Location::new(city_name, None);
                metadata = metadata.with_location(location);
            }

            if let Some(p) = period {
                metadata = metadata.with_period(p);
            }

            if let Some(tag_str) = tags {
                let tag_list: Vec<String> =
                    tag_str.split(',').map(|s| s.trim().to_string()).collect();
                metadata = metadata.with_tags(tag_list);
            }

            // Create token (using dummy owner for demo)
            let owner = KeyPair::generate().public_key();
            let token = IdentityToken::new(
                id.clone(),
                owner,
                token_type.clone(),
                category.clone(),
                metadata,
            );

            // Display result
            println!(
                "{}",
                "✓ Token Created Successfully | تم إنشاء الرمز بنجاح".green()
            );
            println!();
            print_token_info(&token);

            // Export to JSON
            let json = serde_json::to_string_pretty(&token)?;
            let filename = format!("{}.json", id);
            std::fs::write(&filename, json)?;
            println!();
            println!("{}", format!("Saved to: {}", filename).dimmed());
        }

        Commands::List { owner, type_filter } => {
            println!("{}", "═".repeat(60).cyan());
            println!(
                "{}",
                "  Cultural Identity Tokens | رموز الهوية الثقافية  "
                    .cyan()
                    .bold()
            );
            println!("{}", "═".repeat(60).cyan());
            println!();

            // Demo: Show predefined examples
            let examples = get_example_tokens();

            let filtered: Vec<_> = examples
                .iter()
                .filter(|t| {
                    if let Some(ref owner_addr) = owner {
                        hex::encode(t.owner.0).starts_with(owner_addr)
                    } else {
                        true
                    }
                })
                .filter(|t| {
                    if let Some(ref type_str) = type_filter {
                        format!("{:?}", t.token_type)
                            .to_lowercase()
                            .contains(&type_str.to_lowercase())
                    } else {
                        true
                    }
                })
                .collect();

            if filtered.is_empty() {
                println!("{}", "No tokens found | لا توجد رموز".yellow());
            } else {
                for token in &filtered {
                    println!("{} {}", "●".green(), token.metadata.name.bold());
                    println!("  ID: {}", token.id.dimmed());
                    println!("  Type: {}", token.token_type);
                    println!("  Category: {}", token.category);
                    println!();
                }
                println!("{}: {}", "Total".cyan(), filtered.len());
            }
        }

        Commands::Info { token_id } => {
            println!("{}", "═".repeat(60).cyan());
            println!("{}", "  Token Information | معلومات الرمز  ".cyan().bold());
            println!("{}", "═".repeat(60).cyan());
            println!();

            // Try to load from file
            let filename = format!("{}.json", token_id);
            match std::fs::read_to_string(&filename) {
                Ok(json) => {
                    let token: IdentityToken = serde_json::from_str(&json)?;
                    print_token_info(&token);
                }
                Err(_) => {
                    println!("{}", format!("Token not found: {}", token_id).red());
                    println!("{}", "Try: identity examples".dimmed());
                }
            }
        }

        Commands::Examples => {
            println!("{}", "═".repeat(60).cyan());
            println!(
                "{}",
                "  Syrian Heritage Examples | أمثلة التراث السوري  "
                    .cyan()
                    .bold()
            );
            println!("{}", "═".repeat(60).cyan());
            println!();

            print_examples();
        }

        Commands::Upload {
            file,
            token_id,
            api_url,
            gateway_url,
        } => {
            handle_upload(file, token_id, api_url, gateway_url).await?;
        }

        Commands::Retrieve {
            cid,
            output,
            api_url,
            gateway_url,
        } => {
            handle_retrieve(cid, output, api_url, gateway_url).await?;
        }

        Commands::Link { token_id, cid } => {
            handle_link(token_id, cid)?;
        }
    }

    Ok(())
}

fn print_token_info(token: &IdentityToken) {
    println!("{}: {}", "Token ID".cyan(), token.id);
    println!("{}: {}", "Name".cyan(), token.metadata.name);
    if let Some(ref name_ar) = token.metadata.name_ar {
        println!("{}: {}", "Arabic Name".cyan(), name_ar);
    }
    println!();
    println!("{}: {}", "Type".yellow(), token.token_type);
    println!("{}: {}", "Category".yellow(), token.category);
    println!();
    println!("{}:", "Description".cyan());
    println!("{}", token.metadata.description);
    println!();

    if let Some(ref location) = token.metadata.location {
        println!("{}: {}", "Location".cyan(), location.city);
    }

    if let Some(ref period) = token.metadata.historical_period {
        println!("{}: {}", "Period".cyan(), period);
    }

    if !token.metadata.tags.is_empty() {
        println!("{}: {}", "Tags".cyan(), token.metadata.tags.join(", "));
    }

    if let Some(ref unesco) = token.metadata.unesco_status {
        println!("{}: {:?}", "UNESCO Status".cyan(), unesco);
    }

    println!();
    println!("{}: {}", "Owner".dimmed(), hex::encode(&token.owner.0[..8]));
    println!(
        "{}: {}",
        "Verified".dimmed(),
        if token.is_verified() { "Yes" } else { "No" }
    );
}

fn print_examples() {
    let examples = vec![
        (
            "Umayyad Mosque",
            "مسجد بني أمية الكبير",
            "Damascus",
            "Islamic heritage site built 705-715 CE",
        ),
        (
            "Palmyra",
            "تدمر",
            "Tadmur",
            "Ancient Semitic city, UNESCO World Heritage",
        ),
        (
            "Aleppo Citadel",
            "قلعة حلب",
            "Aleppo",
            "Medieval fortified palace, 3rd millennium BCE origins",
        ),
        (
            "Damascus Steel",
            "الفولاذ الدمشقي",
            "Damascus",
            "Legendary metalworking technique, 300 BCE - 1700 CE",
        ),
        (
            "Dabke",
            "دبكة",
            "Syria",
            "Traditional folk dance of the Levant",
        ),
        (
            "Kibbeh",
            "كبة",
            "Syria",
            "National dish - bulgur and meat delicacy",
        ),
    ];

    for (name, name_ar, location, desc) in examples {
        println!("{} {}", "▸".green(), name.bold());
        println!("  {}: {}", "Arabic".dimmed(), name_ar);
        println!("  {}: {}", "Location".dimmed(), location);
        println!("  {}", desc.dimmed());
        println!();
    }

    println!("{}", "Create your own with:".yellow());
    println!("{}", "identity create --id <id> --name <name> --description <desc> --token-type <type> --category <cat>".dimmed());
}

fn get_example_tokens() -> Vec<IdentityToken> {
    let owner = KeyPair::generate().public_key();

    vec![
        IdentityToken::new(
            "umayyad-mosque".to_string(),
            owner,
            token::TokenType::HeritageSite,
            token::CulturalCategory::Islamic,
            HeritageMetadata::new(
                "Umayyad Mosque".to_string(),
                "Great Mosque of Damascus, built 705-715 CE".to_string(),
                Some("مسجد بني أمية الكبير".to_string()),
            )
            .with_location(Location::new(
                "Damascus".to_string(),
                Some("دمشق".to_string()),
            ))
            .with_period("8th Century CE".to_string())
            .with_unesco_status(metadata::UNESCOStatus::WorldHeritage),
        ),
        IdentityToken::new(
            "palmyra".to_string(),
            owner,
            token::TokenType::HeritageSite,
            token::CulturalCategory::Ancient,
            HeritageMetadata::new(
                "Palmyra".to_string(),
                "Ancient Semitic city in the Syrian Desert".to_string(),
                Some("تدمر".to_string()),
            )
            .with_location(Location::new(
                "Tadmur".to_string(),
                Some("تدمر".to_string()),
            ))
            .with_period("1st-3rd Century CE".to_string())
            .with_unesco_status(metadata::UNESCOStatus::Endangered),
        ),
    ]
}

fn parse_token_type(s: &str) -> Result<token::TokenType> {
    match s.to_lowercase().as_str() {
        "heritage" | "site" => Ok(token::TokenType::HeritageSite),
        "craft" => Ok(token::TokenType::TraditionalCraft),
        "document" => Ok(token::TokenType::HistoricalDocument),
        "performing" | "arts" => Ok(token::TokenType::PerformingArts),
        "culinary" | "food" => Ok(token::TokenType::CulinaryHeritage),
        "oral" => Ok(token::TokenType::OralTradition),
        "language" => Ok(token::TokenType::LanguageHeritage),
        "community" => Ok(token::TokenType::CommunityIdentity),
        "personal" => Ok(token::TokenType::PersonalContribution),
        "digital" => Ok(token::TokenType::DigitalCulture),
        _ => anyhow::bail!("Unknown token type: {}", s),
    }
}

fn parse_category(s: &str) -> Result<token::CulturalCategory> {
    match s.to_lowercase().as_str() {
        "ancient" => Ok(token::CulturalCategory::Ancient),
        "islamic" => Ok(token::CulturalCategory::Islamic),
        "ottoman" => Ok(token::CulturalCategory::Ottoman),
        "modern" => Ok(token::CulturalCategory::Modern),
        "contemporary" => Ok(token::CulturalCategory::Contemporary),
        _ => Ok(token::CulturalCategory::Regional {
            region: s.to_string(),
        }),
    }
}

async fn handle_upload(
    file_path: String,
    token_id: Option<String>,
    api_url: Option<String>,
    gateway_url: Option<String>,
) -> Result<()> {
    println!("{}", "═".repeat(60).cyan());
    println!("{}", "  Upload to IPFS | رفع إلى IPFS  ".cyan().bold());
    println!("{}", "═".repeat(60).cyan());
    println!();

    // Create IPFS client
    let client = IpfsClient::new(api_url, gateway_url);

    println!("{}", format!("Uploading: {}", file_path).dimmed());

    // Upload file
    let metadata = client.upload_file(&file_path).await?;

    println!();
    println!("{}", "✓ Upload Successful | تم الرفع بنجاح".green());
    println!();
    println!("{}: {}", "CID".cyan(), metadata.cid);
    println!(
        "{}: {}",
        "Size".cyan(),
        format_bytes(metadata.size as usize)
    );
    println!("{}: {}", "Type".cyan(), metadata.mime_type);
    println!("{}: {}", "Hash".cyan(), metadata.content_hash);
    println!();
    println!(
        "{}: {}",
        "Gateway URL".yellow(),
        client.gateway_url(&metadata.cid)
    );
    println!();

    // If token_id provided, link it
    if let Some(token_id) = token_id {
        println!("{}", format!("Linking to token: {}", token_id).dimmed());
        handle_link(token_id, metadata.cid)?;
    }

    Ok(())
}

async fn handle_retrieve(
    cid: String,
    output: String,
    api_url: Option<String>,
    gateway_url: Option<String>,
) -> Result<()> {
    println!("{}", "═".repeat(60).cyan());
    println!(
        "{}",
        "  Retrieve from IPFS | استرجاع من IPFS  ".cyan().bold()
    );
    println!("{}", "═".repeat(60).cyan());
    println!();

    // Create IPFS client
    let client = IpfsClient::new(api_url, gateway_url);

    println!("{}", format!("Retrieving CID: {}", cid).dimmed());
    println!("{}", format!("Output: {}", output).dimmed());

    // Retrieve content
    let data = client.retrieve(&cid).await?;

    // Write to file
    std::fs::write(&output, &data)?;

    println!();
    println!("{}", "✓ Retrieved Successfully | تم الاسترجاع بنجاح".green());
    println!();
    println!("{}: {}", "Size".cyan(), format_bytes(data.len()));
    println!("{}: {}", "Saved to".cyan(), output);
    println!();

    Ok(())
}

fn handle_link(token_id: String, cid: String) -> Result<()> {
    println!("{}", "═".repeat(60).cyan());
    println!("{}", "  Link IPFS to Token | ربط IPFS برمز  ".cyan().bold());
    println!("{}", "═".repeat(60).cyan());
    println!();

    // Load token
    let filename = format!("{}.json", token_id);
    let json = std::fs::read_to_string(&filename)?;
    let mut token: IdentityToken = serde_json::from_str(&json)?;

    // Update IPFS CID
    token.ipfs_cid = Some(cid.clone());

    // Save token
    let updated_json = serde_json::to_string_pretty(&token)?;
    std::fs::write(&filename, updated_json)?;

    println!("{}", "✓ Linked Successfully | تم الربط بنجاح".green());
    println!();
    println!("{}: {}", "Token ID".cyan(), token_id);
    println!("{}: {}", "IPFS CID".cyan(), cid);
    println!("{}: {}", "Updated".cyan(), filename);
    println!();

    Ok(())
}

fn format_bytes(bytes: usize) -> String {
    const KB: usize = 1024;
    const MB: usize = KB * 1024;
    const GB: usize = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}
