use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use opensyria_core::transaction::Transaction;
use opensyria_wallet::WalletStorage;

#[derive(Parser)]
#[command(name = "wallet")]
#[command(about = "Syrian Digital Lira Wallet (OpenSyria) | محفظة الليرة الرقمية السورية (أوبن سيريا)", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new wallet account | إنشاء حساب جديد
    Create {
        /// Account name | اسم الحساب
        #[arg(short, long)]
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
    let storage = WalletStorage::new()?;

    match cli.command {
        Commands::Create { name } => {
            let account = opensyria_wallet::storage::Account::new(name.clone());
            storage.save_account(&account)?;

            println!(
                "{}",
                "✓ Account created successfully | تم إنشاء الحساب بنجاح".green()
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
                "⚠ Keep your wallet files secure | احفظ ملفات المحفظة بأمان".yellow()
            );
        }

        Commands::List => {
            let accounts = storage.list_accounts()?;

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
                    let account = storage.load_account(&name)?;
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
            let account = storage.load_account(&name)?;
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
            let account = storage.load_account(&from)?;
            let recipient = opensyria_core::crypto::PublicKey::from_hex(&to)?;

            // Convert Lira to smallest unit (1 Lira = 1_000_000 units)
            let amount_units = (amount * 1_000_000.0) as u64;
            let fee_units = (fee * 1_000_000.0) as u64;

            let mut tx =
                Transaction::new(account.address, recipient, amount_units, fee_units, nonce);

            let keypair = account.keypair()?;
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
                format!("⚠ Delete account '{}'? This cannot be undone!", name).yellow()
            );
            println!("{}", "Type 'yes' to confirm: ".dimmed());

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;

            if input.trim() == "yes" {
                storage.delete_account(&name)?;
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
