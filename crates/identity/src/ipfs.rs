use anyhow::{Context, Result};
use reqwest::multipart;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

/// IPFS client for uploading and retrieving content
pub struct IpfsClient {
    api_url: String,
    gateway_url: String,
    provider: IpfsProvider,
}

/// IPFS provider configuration
#[derive(Debug, Clone)]
pub enum IpfsProvider {
    /// Local IPFS daemon
    Local,
    /// Pinata cloud service
    Pinata { api_key: String, api_secret: String },
    /// Infura IPFS service
    Infura { project_id: String, project_secret: String },
}

/// IPFS add response
#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
struct IpfsAddResponse {
    #[serde(rename = "Hash")]
    hash: String,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Size")]
    size: String,
}

/// Content metadata stored alongside IPFS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentMetadata {
    /// IPFS CID (Content Identifier)
    pub cid: String,
    /// Original filename
    pub filename: String,
    /// File size in bytes
    pub size: u64,
    /// MIME type
    pub mime_type: String,
    /// SHA-256 hash of content
    pub content_hash: String,
    /// Upload timestamp
    pub uploaded_at: u64,
}

impl IpfsClient {
    /// Create a new IPFS client with local daemon
    pub fn new(api_url: Option<String>, gateway_url: Option<String>) -> Self {
        Self {
            api_url: api_url.unwrap_or_else(|| "http://127.0.0.1:5001".to_string()),
            gateway_url: gateway_url.unwrap_or_else(|| "http://127.0.0.1:8080".to_string()),
            provider: IpfsProvider::Local,
        }
    }

    /// Create a new IPFS client with Pinata
    pub fn with_pinata(api_key: String, api_secret: String) -> Self {
        Self {
            api_url: "https://api.pinata.cloud".to_string(),
            gateway_url: "https://gateway.pinata.cloud".to_string(),
            provider: IpfsProvider::Pinata { api_key, api_secret },
        }
    }

    /// Create a new IPFS client with Infura
    pub fn with_infura(project_id: String, project_secret: String) -> Self {
        Self {
            api_url: format!("https://ipfs.infura.io:5001"),
            gateway_url: format!("https://ipfs.infura.io/ipfs"),
            provider: IpfsProvider::Infura { project_id, project_secret },
        }
    }

    /// Upload a file to IPFS
    pub async fn upload_file<P: AsRef<Path>>(&self, path: P) -> Result<ContentMetadata> {
        let path = path.as_ref();
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .context("Invalid filename")?
            .to_string();

        // Read file content
        let content = tokio::fs::read(path).await.context("Failed to read file")?;

        self.upload_bytes(&content, &filename).await
    }

    /// Upload bytes to IPFS
    pub async fn upload_bytes(&self, data: &[u8], filename: &str) -> Result<ContentMetadata> {
        match &self.provider {
            IpfsProvider::Local => self.upload_to_local(data, filename).await,
            IpfsProvider::Pinata { api_key, api_secret } => {
                self.upload_to_pinata(data, filename, api_key, api_secret).await
            }
            IpfsProvider::Infura { project_id, project_secret } => {
                self.upload_to_infura(data, filename, project_id, project_secret).await
            }
        }
    }

    /// Upload to local IPFS daemon
    async fn upload_to_local(&self, data: &[u8], filename: &str) -> Result<ContentMetadata> {
        // Calculate SHA-256 hash
        let mut hasher = Sha256::new();
        hasher.update(data);
        let content_hash = hex::encode(hasher.finalize());

        // Detect MIME type
        let mime_type = self.detect_mime_type(filename);

        // Create multipart form
        let form = multipart::Form::new().part(
            "file",
            multipart::Part::bytes(data.to_vec())
                .file_name(filename.to_string())
                .mime_str(&mime_type)?,
        );

        // Upload to IPFS
        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/api/v0/add", self.api_url))
            .multipart(form)
            .send()
            .await
            .context("Failed to upload to IPFS")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("IPFS upload failed ({}): {}", status, error_text);
        }

        let ipfs_response: IpfsAddResponse = response
            .json()
            .await
            .context("Failed to parse IPFS response")?;

        Ok(ContentMetadata {
            cid: ipfs_response.hash,
            filename: filename.to_string(),
            size: data.len() as u64,
            mime_type,
            content_hash,
            uploaded_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
        })
    }

    /// Upload to Pinata cloud service
    async fn upload_to_pinata(
        &self,
        data: &[u8],
        filename: &str,
        api_key: &str,
        api_secret: &str,
    ) -> Result<ContentMetadata> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let content_hash = hex::encode(hasher.finalize());

        let mime_type = self.detect_mime_type(filename);

        let form = multipart::Form::new().part(
            "file",
            multipart::Part::bytes(data.to_vec())
                .file_name(filename.to_string())
                .mime_str(&mime_type)?,
        );

        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/pinning/pinFileToIPFS", self.api_url))
            .header("pinata_api_key", api_key)
            .header("pinata_secret_api_key", api_secret)
            .multipart(form)
            .send()
            .await
            .context("Failed to upload to Pinata")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Pinata upload failed ({}): {}", status, error_text);
        }

        #[derive(Deserialize)]
        struct PinataResponse {
            #[serde(rename = "IpfsHash")]
            ipfs_hash: String,
        }

        let pinata_response: PinataResponse = response
            .json()
            .await
            .context("Failed to parse Pinata response")?;

        Ok(ContentMetadata {
            cid: pinata_response.ipfs_hash,
            filename: filename.to_string(),
            size: data.len() as u64,
            mime_type,
            content_hash,
            uploaded_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
        })
    }

    /// Upload to Infura IPFS service
    async fn upload_to_infura(
        &self,
        data: &[u8],
        filename: &str,
        project_id: &str,
        project_secret: &str,
    ) -> Result<ContentMetadata> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let content_hash = hex::encode(hasher.finalize());

        let mime_type = self.detect_mime_type(filename);

        let form = multipart::Form::new().part(
            "file",
            multipart::Part::bytes(data.to_vec())
                .file_name(filename.to_string())
                .mime_str(&mime_type)?,
        );

        let auth = format!("{}:{}", project_id, project_secret);
        use base64::Engine;
        let auth_header = format!("Basic {}", base64::engine::general_purpose::STANDARD.encode(auth));

        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/api/v0/add", self.api_url))
            .header("Authorization", auth_header)
            .multipart(form)
            .send()
            .await
            .context("Failed to upload to Infura")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Infura upload failed ({}): {}", status, error_text);
        }

        let ipfs_response: IpfsAddResponse = response
            .json()
            .await
            .context("Failed to parse Infura response")?;

        Ok(ContentMetadata {
            cid: ipfs_response.hash,
            filename: filename.to_string(),
            size: data.len() as u64,
            mime_type,
            content_hash,
            uploaded_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
        })
    }

    /// Upload text content to IPFS
    pub async fn upload_text(&self, text: &str, filename: &str) -> Result<ContentMetadata> {
        self.upload_bytes(text.as_bytes(), filename).await
    }

    /// Upload JSON data to IPFS
    pub async fn upload_json<T: Serialize>(
        &self,
        data: &T,
        filename: &str,
    ) -> Result<ContentMetadata> {
        let json = serde_json::to_string_pretty(data)?;
        self.upload_text(&json, filename).await
    }

    /// Retrieve content from IPFS by CID
    pub async fn retrieve(&self, cid: &str) -> Result<Vec<u8>> {
        let client = reqwest::Client::new();
        let url = format!("{}/ipfs/{}", self.gateway_url, cid);

        let response = client
            .get(&url)
            .send()
            .await
            .context("Failed to retrieve from IPFS")?;

        if !response.status().is_success() {
            anyhow::bail!("IPFS retrieval failed: {}", response.status());
        }

        let bytes = response
            .bytes()
            .await
            .context("Failed to read IPFS response")?;

        Ok(bytes.to_vec())
    }

    /// Retrieve text content from IPFS
    pub async fn retrieve_text(&self, cid: &str) -> Result<String> {
        let bytes = self.retrieve(cid).await?;
        String::from_utf8(bytes).context("Invalid UTF-8 in IPFS content")
    }

    /// Retrieve JSON data from IPFS
    pub async fn retrieve_json<T: for<'de> Deserialize<'de>>(&self, cid: &str) -> Result<T> {
        let text = self.retrieve_text(cid).await?;
        serde_json::from_str(&text).context("Failed to parse JSON from IPFS")
    }

    /// Get gateway URL for a CID
    pub fn gateway_url(&self, cid: &str) -> String {
        format!("{}/ipfs/{}", self.gateway_url, cid)
    }

    /// Pin content to ensure it's retained
    pub async fn pin(&self, cid: &str) -> Result<()> {
        let client = reqwest::Client::new();
        let url = format!("{}/api/v0/pin/add?arg={}", self.api_url, cid);

        let response = client
            .post(&url)
            .send()
            .await
            .context("Failed to pin content")?;

        if !response.status().is_success() {
            anyhow::bail!("IPFS pin failed: {}", response.status());
        }

        Ok(())
    }

    /// Unpin content
    pub async fn unpin(&self, cid: &str) -> Result<()> {
        let client = reqwest::Client::new();
        let url = format!("{}/api/v0/pin/rm?arg={}", self.api_url, cid);

        let response = client
            .post(&url)
            .send()
            .await
            .context("Failed to unpin content")?;

        if !response.status().is_success() {
            anyhow::bail!("IPFS unpin failed: {}", response.status());
        }

        Ok(())
    }

    /// Check if IPFS daemon is running
    pub async fn is_available(&self) -> bool {
        let client = reqwest::Client::new();
        client
            .post(format!("{}/api/v0/version", self.api_url))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    /// Detect MIME type from filename
    fn detect_mime_type(&self, filename: &str) -> String {
        let extension = filename.split('.').next_back().unwrap_or("");
        match extension.to_lowercase().as_str() {
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "gif" => "image/gif",
            "svg" => "image/svg+xml",
            "mp4" => "video/mp4",
            "mp3" => "audio/mpeg",
            "wav" => "audio/wav",
            "pdf" => "application/pdf",
            "json" => "application/json",
            "txt" => "text/plain",
            "md" => "text/markdown",
            "html" => "text/html",
            _ => "application/octet-stream",
        }
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_upload_text() {
        let client = IpfsClient::new(None, None);

        // Skip test if IPFS daemon not running
        if !client.is_available().await {
            println!("Skipping IPFS test - daemon not running");
            return;
        }

        let content = "Syrian Cultural Heritage Content";
        let result = client.upload_text(content, "test.txt").await;

        if let Ok(metadata) = result {
            assert!(!metadata.cid.is_empty());
            assert_eq!(metadata.filename, "test.txt");
            assert_eq!(metadata.size, content.len() as u64);
            println!("Uploaded to IPFS: {}", metadata.cid);
        }
    }

    #[tokio::test]
    async fn test_mime_detection() {
        let client = IpfsClient::new(None, None);

        assert_eq!(client.detect_mime_type("image.jpg"), "image/jpeg");
        assert_eq!(client.detect_mime_type("video.mp4"), "video/mp4");
        assert_eq!(client.detect_mime_type("document.pdf"), "application/pdf");
        assert_eq!(client.detect_mime_type("data.json"), "application/json");
    }
}
