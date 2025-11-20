/// Node configuration management
/// إدارة تكوين العقدة

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Node configuration
/// تكوين العقدة
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Node data directory
    #[serde(default = "default_data_dir")]
    pub data_dir: String,

    /// Network configuration
    #[serde(default)]
    pub network: NetworkConfig,

    /// Mining configuration
    #[serde(default)]
    pub mining: MiningConfig,

    /// Daemon configuration
    #[serde(default)]
    pub daemon: DaemonConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// P2P listen port
    #[serde(default = "default_port")]
    pub port: u16,

    /// Bootstrap nodes (multiaddr format)
    #[serde(default)]
    pub bootstrap_nodes: Vec<String>,

    /// Maximum number of peers
    #[serde(default = "default_max_peers")]
    pub max_peers: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningConfig {
    /// Mining difficulty
    #[serde(default = "default_difficulty")]
    pub difficulty: u32,

    /// Number of mining threads
    #[serde(default = "default_mining_threads")]
    pub threads: usize,

    /// Mining address (hex public key)
    pub mining_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    /// Enable auto-mining when running as daemon
    #[serde(default)]
    pub auto_mine: bool,

    /// Log file path
    #[serde(default = "default_log_file")]
    pub log_file: String,

    /// Log rotation: max file size in MB
    #[serde(default = "default_log_max_size_mb")]
    pub log_max_size_mb: u64,

    /// Log rotation: number of backup files to keep
    #[serde(default = "default_log_backups")]
    pub log_backups: usize,
}

// Default values
fn default_data_dir() -> String {
    "~/.opensyria/node".to_string()
}

fn default_port() -> u16 {
    9000
}

fn default_max_peers() -> usize {
    50
}

fn default_difficulty() -> u32 {
    16
}

fn default_mining_threads() -> usize {
    num_cpus::get()
}

fn default_log_file() -> String {
    "~/.opensyria/node/opensyria.log".to_string()
}

fn default_log_max_size_mb() -> u64 {
    100
}

fn default_log_backups() -> usize {
    7
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            bootstrap_nodes: Vec::new(),
            max_peers: default_max_peers(),
        }
    }
}

impl Default for MiningConfig {
    fn default() -> Self {
        Self {
            difficulty: default_difficulty(),
            threads: default_mining_threads(),
            mining_address: None,
        }
    }
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            auto_mine: false,
            log_file: default_log_file(),
            log_max_size_mb: default_log_max_size_mb(),
            log_backups: default_log_backups(),
        }
    }
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            data_dir: default_data_dir(),
            network: NetworkConfig::default(),
            mining: MiningConfig::default(),
            daemon: DaemonConfig::default(),
        }
    }
}

impl NodeConfig {
    /// Load config from TOML file
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref())
            .context(format!("Failed to read config file: {}", path.as_ref().display()))?;

        let config: NodeConfig = toml::from_str(&content)
            .context("Failed to parse TOML config")?;

        config.validate()?;
        Ok(config)
    }

    /// Save config to TOML file
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config to TOML")?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)
                .context(format!("Failed to create directory: {}", parent.display()))?;
        }

        fs::write(path.as_ref(), content)
            .context(format!("Failed to write config file: {}", path.as_ref().display()))?;

        Ok(())
    }

    /// Load config with fallback to default
    pub fn load_or_default(path: impl AsRef<Path>) -> Self {
        match Self::load_from_file(&path) {
            Ok(config) => config,
            Err(_) => {
                // If config doesn't exist, create default
                let config = Self::default();
                let _ = config.save_to_file(&path);
                config
            }
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate port range
        if self.network.port == 0 {
            anyhow::bail!("Invalid port: must be > 0");
        }

        // Validate bootstrap nodes
        for node in &self.network.bootstrap_nodes {
            if node.is_empty() {
                anyhow::bail!("Bootstrap node address cannot be empty");
            }
            // Basic multiaddr validation
            if !node.starts_with("/ip4/") && !node.starts_with("/ip6/") && !node.starts_with("/dns/") {
                anyhow::bail!("Invalid bootstrap node multiaddr format: {}", node);
            }
        }

        // Validate difficulty
        if self.mining.difficulty < 1 || self.mining.difficulty > 255 {
            anyhow::bail!("Mining difficulty must be between 1 and 255");
        }

        // Validate threads
        if self.mining.threads == 0 {
            anyhow::bail!("Mining threads must be > 0");
        }

        // Validate mining address if provided
        if let Some(addr) = &self.mining.mining_address {
            if addr.len() != 64 {
                anyhow::bail!("Mining address must be 64 hex characters (32 bytes)");
            }
            hex::decode(addr).context("Invalid mining address: not valid hex")?;
        }

        Ok(())
    }

    /// Get default config file path
    pub fn default_config_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".opensyria")
            .join("config.toml")
    }

    /// Create example config file with comments
    pub fn create_example(path: impl AsRef<Path>) -> Result<()> {
        let example = r#"# OpenSyria Node Configuration
# تكوين عقدة OpenSyria

# Node data directory | دليل بيانات العقدة
data_dir = "~/.opensyria/node"

[network]
# P2P listen port | منفذ الاستماع P2P
port = 9000

# Bootstrap nodes (multiaddr format) | عقد البداية (صيغة multiaddr)
bootstrap_nodes = [
    "/ip4/127.0.0.1/tcp/9000/p2p/12D3KooWExamplePeerId1",
    "/ip4/192.168.1.100/tcp/9000/p2p/12D3KooWExamplePeerId2",
]

# Maximum number of peers | الحد الأقصى لعدد الأقران
max_peers = 50

[mining]
# Mining difficulty (1-255) | صعوبة التعدين
difficulty = 16

# Number of mining threads | عدد خيوط التعدين
threads = 4

# Mining reward address (hex public key) | عنوان مكافأة التعدين
# mining_address = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"

[daemon]
# Enable auto-mining when running as daemon | تفعيل التعدين التلقائي عند التشغيل كخدمة
auto_mine = false

# Log file path | مسار ملف السجل
log_file = "~/.opensyria/node/opensyria.log"

# Maximum log file size in MB | الحجم الأقصى لملف السجل بالميجابايت
log_max_size_mb = 100

# Number of log backup files to keep | عدد ملفات السجل الاحتياطية للاحتفاظ بها
log_backups = 7
"#;

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path.as_ref(), example)
            .context(format!("Failed to write example config: {}", path.as_ref().display()))?;

        Ok(())
    }

    /// Expand tilde in paths
    pub fn expand_paths(&mut self) {
        self.data_dir = shellexpand::tilde(&self.data_dir).to_string();
        self.daemon.log_file = shellexpand::tilde(&self.daemon.log_file).to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let config = NodeConfig::default();
        assert_eq!(config.network.port, 9000);
        assert_eq!(config.mining.difficulty, 16);
        assert!(!config.daemon.auto_mine);
    }

    #[test]
    fn test_config_validation() {
        let mut config = NodeConfig::default();
        assert!(config.validate().is_ok());

        // Invalid port
        config.network.port = 0;
        assert!(config.validate().is_err());

        config.network.port = 9000;
        assert!(config.validate().is_ok());

        // Invalid difficulty
        config.mining.difficulty = 0;
        assert!(config.validate().is_err());

        config.mining.difficulty = 256;
        assert!(config.validate().is_err());

        config.mining.difficulty = 16;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_save_and_load() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.toml");

        let mut config = NodeConfig::default();
        config.network.port = 8888;
        config.mining.difficulty = 20;

        // Save
        config.save_to_file(&config_path).unwrap();

        // Load
        let loaded = NodeConfig::load_from_file(&config_path).unwrap();
        assert_eq!(loaded.network.port, 8888);
        assert_eq!(loaded.mining.difficulty, 20);
    }
}
