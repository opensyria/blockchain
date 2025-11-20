use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use opensyria_core::transaction::Transaction;
use opensyria_wallet::{EncryptedWalletStorage, WalletStorage};
use rpassword::read_password;

#[derive(Parser)]
#[command(name = "wallet")]
#[command(about = "Syrian Digital Lira Wallet (OpenSyria) | محفظة الليرة السورية الرقمية (أوبن سيريا)", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new encrypted wallet account | إنشاء حساب مشفر جديد
    Create {
        /// Account name | اسم الحساب
        #[arg(short, long)]
        name: String,
    },

    /// Create HD wallet from mnemonic | إنشاء محفظة HD من العبارة الاحتياطية
    CreateHd {
        /// Account name | اسم الحساب
        #[arg(short, long)]
        name: String,

        /// 12 or 24 word mnemonic phrase | عبارة احتياطية 12 أو 24 كلمة
        #[arg(short, long)]
        mnemonic: Option<String>,
    },

    /// Display QR code for account address | عرض رمز QR لعنوان الحساب
    Qr {
        /// Account name | اسم الحساب
        name: String,
    },

    /// Migrate plaintext wallet to encrypted | ترحيل محفظة نصية إلى مشفرة
    Migrate {
        /// Account name | اسم الحساب
        name: String,
    },

    /// List all wallet accounts | عرض جميع الحسابات
    List,

    /// Show account details | عرض تفاصيل الحساب
    Info {
        /// Account name | اسم الحساب
        name: String,
    },

    /// Create and sign a transaction | إنشاء معاملة جديدة
    Send {
        /// Sender account name | اسم حساب المرسل
        #[arg(short, long)]
        from: String,

        /// Recipient address (hex) | عنوان المستلم
        #[arg(short, long)]
        to: String,

        /// Amount in Lira | المبلغ بالليرة
        #[arg(short, long)]
        amount: f64,

        /// Transaction fee | رسوم المعاملة
        #[arg(short = 'f', long, default_value = "0.0001")]
        fee: f64,

        /// Transaction nonce | رقم المعاملة
        #[arg(short, long, default_value = "0")]
        nonce: u64,
    },

    /// Delete an account | حذف حساب
    Delete {
        /// Account name | اسم الحساب
        name: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let encrypted_storage = EncryptedWalletStorage::new()?;

    match cli.command {
        Commands::Create { name } => {
            println!("{}", "Enter password | أدخل كلمة المرور: ".cyan());
            let password = read_password()?;
            
            println!("{}", "Confirm password | تأكيد كلمة المرور: ".cyan());
            let confirm = read_password()?;
            
            if password != confirm {
                println!("{}", "✗ Passwords don't match | كلمات المرور غير متطابقة".red());
                return Ok(());
            }
            
            if password.len() < 8 {
                println!("{}", "✗ Password must be at least 8 characters | يجب أن تكون كلمة المرور 8 أحرف على الأقل".red());
                return Ok(());
            }

            let account = opensyria_wallet::encrypted::EncryptedAccount::new(name.clone(), &password)?;
            encrypted_storage.save_account(&account)?;

            println!(
                "{}",
                "✓ Encrypted account created successfully | تم إنشاء الحساب المشفر بنجاح".green()
            );
            println!();
            println!("{}: {}", "Name | الاسم".cyan(), name);
            println!(
                "{}: {}",
                "Address | العنوان".cyan(),
                account.address.to_hex()
            );
            println!();
            println!(
                "{}",
                "🔐 Your wallet is encrypted with AES-256-GCM | محفظتك مشفرة بـ AES-256-GCM".green()
            );
            println!(
                "{}",
                "⚠ NEVER share your password | لا تشارك كلمة المرور أبداً".yellow()
            );
        }

        Commands::CreateHd { name, mnemonic } => {
            println!("{}", "Enter password | أدخل كلمة المرور: ".cyan());
            let password = read_password()?;
            
            let hd_wallet = if let Some(phrase) = mnemonic {
                opensyria_wallet::HDWallet::from_phrase(&phrase)?
            } else {
                let wallet = opensyria_wallet::HDWallet::generate(12)?;
                println!();
                println!("{}", "📝 BACKUP YOUR MNEMONIC PHRASE | احفظ العبارة الاحتياطية".yellow().bold());
                println!("{}", "═".repeat(60).yellow());
                opensyria_wallet::display_mnemonic_warning();
                println!();
                println!("{}", wallet.get_phrase()?.cyan().bold());
                println!();
                println!("{}", "═".repeat(60).yellow());
                println!("{}", "⚠ Write this down on paper and store it safely | اكتب هذه العبارة على ورقة واحفظها بأمان".yellow());
                println!();
                wallet
            };
            
            let keypair = hd_wallet.derive_account(0)?;
            let private_key = keypair.private_key_bytes();
            
            // Create encrypted account from HD wallet
            let account = opensyria_wallet::encrypted::EncryptedAccount::from_private_key(
                name.clone(),
                &private_key,
                &password
            )?;
            encrypted_storage.save_account(&account)?;

            println!(
                "{}",
                "✓ HD wallet account created | تم إنشاء حساب محفظة HD".green()
            );
            println!();
            println!("{}: {}", "Name | الاسم".cyan(), name);
            println!(
                "{}: {}",
                "Address | العنوان".cyan(),
                account.address.to_hex()
            );
        }

        Commands::Qr { name } => {
            let account = encrypted_storage.load_account(&name)?;
            let address = account.address.to_hex();
            
            println!();
            println!("{}", format!("QR Code for {} | رمز QR لـ {}", name, name).cyan().bold());
            println!("{}", "─".repeat(50).dimmed());
            println!();
            
            match qr2term::print_qr(&address) {
                Ok(_) => {
                    println!();
                    println!("{}: {}", "Address | العنوان".cyan(), address);
                }
                Err(e) => {
                    println!("{}", format!("✗ Failed to generate QR code: {}", e).red());
                    println!("{}: {}", "Address | العنوان".cyan(), address);
                }
            }
        }

        Commands::Migrate { name } => {
            // Load from plaintext storage
            let plaintext_storage = WalletStorage::new()?;
            let old_account = plaintext_storage.load_account(&name)?;
            
            println!("{}", "⚠ Migrating to encrypted wallet | الترحيل إلى محفظة مشفرة".yellow().bold());
            println!("{}", "Enter new password | أدخل كلمة المرور الجديدة: ".cyan());
            let password = read_password()?;
            
            println!("{}", "Confirm password | تأكيد كلمة المرور: ".cyan());
            let confirm = read_password()?;
            
            if password != confirm {
                println!("{}", "✗ Passwords don't match | كلمات المرور غير متطابقة".red());
                return Ok(());
            }
            
            // Create encrypted account from plaintext
            let private_key = old_account.keypair()?.private_key_bytes();
            let encrypted_account = opensyria_wallet::encrypted::EncryptedAccount::from_private_key(
                name.clone(),
                &private_key,
                &password
            )?;
            
            encrypted_storage.save_account(&encrypted_account)?;
            plaintext_storage.delete_account(&name)?;
            
            println!(
                "{}",
                "✓ Account migrated successfully | تم ترحيل الحساب بنجاح".green()
            );
            println!("{}", "🔐 Your wallet is now encrypted | محفظتك مشفرة الآن".green());
        }

        Commands::List => {
            let accounts = encrypted_storage.list_accounts()?;

            if accounts.is_empty() {
                println!("{}", "No accounts found | لا توجد حسابات".yellow());
                println!(
                    "{}",
                    "Create one with: wallet create --name <name>".dimmed()
                );
            } else {
                println!("{}", "Wallet Accounts | الحسابات".cyan().bold());
                println!("{}", "─".repeat(50).dimmed());

                for name in accounts {
                    let account = encrypted_storage.load_account(&name)?;
                    println!(
                        "{} {} {}",
                        "●".green(),
                        name.bold(),
                        format!("({}...)", &account.address.to_hex()[..16]).dimmed()
                    );
                }
            }
        }

        Commands::Info { name } => {
            let account = encrypted_storage.load_account(&name)?;
            let created = format_timestamp(account.created_at);

            println!("{}", "Account Information | معلومات الحساب".cyan().bold());
            println!("{}", "─".repeat(50).dimmed());
            println!();
            println!("{}: {}", "Name | الاسم".cyan(), name);
            println!(
                "{}: {}",
                "Address | العنوان".cyan(),
                account.address.to_hex()
            );
            println!("{}: {}", "Created | تاريخ الإنشاء".cyan(), created);
            println!();
            println!("Balance | الرصيد: {} (coming soon)", "0.00 SYL".bold());
        }

        Commands::Send {
            from,
            to,
            amount,
            fee,
            nonce,
        } => {
            let account = encrypted_storage.load_account(&from)?;
            
            println!("{}", "Enter password | أدخل كلمة المرور: ".cyan());
            let password = read_password()?;
            
            let keypair = account.decrypt_keypair(&password)?;
            let recipient = opensyria_core::crypto::PublicKey::from_hex(&to)?;

            // Convert Lira to smallest unit (1 Lira = 1_000_000 units)
            let amount_units = (amount * 1_000_000.0) as u64;
            let fee_units = (fee * 1_000_000.0) as u64;

            let mut tx =
                Transaction::new(account.address, recipient, amount_units, fee_units, nonce);

            let sig_hash = tx.signing_hash();
            tx = tx.with_signature(keypair.sign(&sig_hash));

            // Verify transaction
            tx.verify()?;

            let tx_json = serde_json::to_string_pretty(&tx)?;

            println!(
                "{}",
                "✓ Transaction created and signed | تم إنشاء المعاملة وتوقيعها".green()
            );
            println!();
            println!("{}", "Transaction Details | تفاصيل المعاملة".cyan().bold());
            println!("{}", "─".repeat(50).dimmed());
            println!();
            println!("{}: {}", "From | من".cyan(), from);
            println!("{}: {}...", "To | إلى".cyan(), &to[..16]);
            println!("{}: {} SYL", "Amount | المبلغ".cyan(), amount);
            println!("{}: {} SYL", "Fee | الرسوم".cyan(), fee);
            println!("{}: {}", "Nonce | الرقم".cyan(), nonce);
            println!();
            println!("{}", "Signed Transaction (JSON):".dimmed());
            println!("{}", tx_json.dimmed());
        }

        Commands::Delete { name } => {
            println!(
                "{}",
                format!("⚠ Delete encrypted account '{}'? This cannot be undone!", name).yellow()
            );
            println!("{}", "Type 'yes' to confirm: ".dimmed());

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;

            if input.trim() == "yes" {
                encrypted_storage.delete_account(&name)?;
                println!("{}", "✓ Account deleted | تم حذف الحساب".green());
            } else {
                println!("{}", "Cancelled | تم الإلغاء".yellow());
            }
        }
    }

    Ok(())
}

fn format_timestamp(unix_secs: u64) -> String {
    use std::time::{Duration, UNIX_EPOCH};

    let datetime = UNIX_EPOCH + Duration::from_secs(unix_secs);
    let datetime: chrono::DateTime<chrono::Utc> = datetime.into();
    datetime.format("%Y-%m-%d %H:%M UTC").to_string()
}
