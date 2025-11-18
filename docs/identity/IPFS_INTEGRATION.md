# IPFS Integration for Cultural Heritage

The OpenSyria blockchain integrates IPFS (InterPlanetary File System) to enable decentralized storage of cultural heritage multimedia content. This allows identity tokens to reference images, videos, audio recordings, and documents related to Syrian heritage sites, traditional crafts, oral traditions, and historical artifacts.

## Overview

IPFS provides content-addressed storage, meaning files are identified by their cryptographic hash (Content Identifier or CID) rather than location. This ensures:

- **Permanent addressing:** Content cannot be altered without changing the CID
- **Decentralization:** Files stored across multiple IPFS nodes worldwide
- **Censorship resistance:** No central authority controls access
- **Deduplication:** Identical files share the same CID
- **Integrity verification:** SHA-256 hashing ensures content hasn't been tampered with

## Architecture

```
Identity Token (On-Chain)
├── id: "palmyra-ruins"
├── name: "Ruins of Palmyra"
├── metadata: {...}
└── ipfs_cid: "QmXyz..." ───────┐
                                 │
                                 ├──> IPFS Network
                                 │    ├── Images (photos of ruins)
                                 │    ├── Videos (3D tours)
                                 │    ├── Audio (oral histories)
                                 │    └── Documents (UNESCO reports)
                                 │
                                 └──> Gateway URLs
                                      ├── Local: http://127.0.0.1:8080/ipfs/QmXyz...
                                      └── Public: https://ipfs.io/ipfs/QmXyz...
```

## Prerequisites

### Install IPFS

**macOS:**
```bash
brew install ipfs
```

**Linux:**
```bash
wget https://dist.ipfs.io/go-ipfs/v0.24.0/go-ipfs_v0.24.0_linux-amd64.tar.gz
tar -xvzf go-ipfs_v0.24.0_linux-amd64.tar.gz
cd go-ipfs
sudo bash install.sh
```

**Windows:**
Download from https://dist.ipfs.io/#go-ipfs and add to PATH.

### Initialize and Start IPFS

```bash
# Initialize IPFS repository
ipfs init

# Start IPFS daemon
ipfs daemon

# Verify it's running
curl http://127.0.0.1:5001/api/v0/version
```

You should see:
```json
{"Version":"0.24.0","Commit":"...","Repo":"...","System":"...","Golang":"..."}
```

## CLI Commands

### Upload Heritage Content

Upload files to IPFS and optionally link them to identity tokens:

```bash
# Upload a file
./target/release/identity upload --file heritage-photo.jpg

# Upload and link to token
./target/release/identity upload \
  --file palmyra-restoration.mp4 \
  --token-id palmyra-ruins

# Custom IPFS endpoints
./target/release/identity upload \
  --file document.pdf \
  --token-id damascus-steel-001 \
  --api-url http://localhost:5001 \
  --gateway-url http://localhost:8080
```

**Output:**
```
════════════════════════════════════════════════════════════
  Upload to IPFS | رفع إلى IPFS
════════════════════════════════════════════════════════════

Uploading: palmyra-restoration.mp4

✓ Upload Successful | تم الرفع بنجاح

CID: QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG
Size: 12.5 MB
Type: video/mp4
Hash: a3f5b8c9e2d1...

Gateway URL: http://127.0.0.1:8080/ipfs/QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG

Linking to token: palmyra-ruins
```

### Retrieve Content

Download content from IPFS by CID:

```bash
# Retrieve file
./target/release/identity retrieve QmYwAPJzv... --output video.mp4

# Custom gateway
./target/release/identity retrieve QmYwAPJzv... \
  --output video.mp4 \
  --gateway-url https://ipfs.io
```

**Output:**
```
════════════════════════════════════════════════════════════
  Retrieve from IPFS | استرجاع من IPFS
════════════════════════════════════════════════════════════

Retrieving CID: QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG
Output: video.mp4

✓ Retrieved Successfully | تم الاسترجاع بنجاح

Size: 12.5 MB
Saved to: video.mp4
```

### Link Existing Content

Associate an existing IPFS CID with an identity token:

```bash
./target/release/identity link \
  --token-id damascus-umayyad \
  --cid QmXyz...
```

**Output:**
```
════════════════════════════════════════════════════════════
  Link IPFS to Token | ربط IPFS برمز
════════════════════════════════════════════════════════════

✓ Linked Successfully | تم الربط بنجاح

Token ID: damascus-umayyad
IPFS CID: QmXyz...
Updated: damascus-umayyad.json
```

## Use Cases

### 1. Heritage Site Documentation

Upload photos and videos of Syrian heritage sites:

```bash
# Create token
./target/release/identity create \
  --id bosra-amphitheatre \
  --name "Bosra Roman Theatre" \
  --name-ar "مسرح بصرى الروماني" \
  --description "Well-preserved Roman theatre from 2nd century CE" \
  --token-type heritage \
  --category ancient \
  --city Bosra

# Upload 360° video tour
./target/release/identity upload \
  --file bosra-360-tour.mp4 \
  --token-id bosra-amphitheatre
```

### 2. Traditional Craft Records

Document traditional Syrian crafts with video demonstrations:

```bash
# Create craft token
./target/release/identity create \
  --id damascus-silk-weaving \
  --name "Damascus Silk Weaving" \
  --name-ar "نسيج الحرير الدمشقي" \
  --description "Traditional silk weaving techniques" \
  --token-type craft \
  --category islamic

# Upload instructional video
./target/release/identity upload \
  --file silk-weaving-tutorial.mp4 \
  --token-id damascus-silk-weaving
```

### 3. Oral Traditions

Preserve oral histories and storytelling:

```bash
# Create oral tradition token
./target/release/identity create \
  --id aleppo-folk-tales \
  --name "Aleppo Folk Tales" \
  --name-ar "حكايات حلب الشعبية" \
  --description "Traditional folk stories from Aleppo" \
  --token-type oral \
  --category regional

# Upload audio recordings
./target/release/identity upload \
  --file folk-tale-01.mp3 \
  --token-id aleppo-folk-tales
```

### 4. Historical Documents

Archive historical documents and manuscripts:

```bash
# Create document token
./target/release/identity create \
  --id ottoman-archives-001 \
  --name "Ottoman Damascus Registry" \
  --name-ar "سجل دمشق العثماني" \
  --description "Administrative records from Ottoman period" \
  --token-type document \
  --category ottoman

# Upload scanned document
./target/release/identity upload \
  --file registry-1875.pdf \
  --token-id ottoman-archives-001
```

## Token Schema

Identity tokens with IPFS content include the `ipfs_cid` field:

```json
{
  "id": "palmyra-ruins",
  "owner": "0xabc123...",
  "token_type": "HeritageSite",
  "category": "Ancient",
  "metadata": {
    "name": "Ruins of Palmyra",
    "name_ar": "آثار تدمر",
    "description": "Ancient Semitic city ruins",
    "location": {
      "city": "Palmyra",
      "city_ar": "تدمر"
    },
    "historical_period": "1st-3rd Century CE",
    "unesco_status": "Endangered"
  },
  "created_at": 1735084800,
  "ipfs_cid": "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG"
}
```

## Content Metadata

Uploaded files generate metadata including:

```rust
ContentMetadata {
    cid: String,           // IPFS Content Identifier
    filename: String,      // Original filename
    size: u64,            // File size in bytes
    mime_type: String,    // Detected MIME type
    content_hash: String, // SHA-256 hash for integrity
    uploaded_at: u64,     // Unix timestamp
}
```

**Supported MIME types:**
- Images: `image/jpeg`, `image/png`, `image/gif`, `image/svg+xml`
- Videos: `video/mp4`, `video/webm`, `video/quicktime`
- Audio: `audio/mpeg`, `audio/wav`, `audio/ogg`
- Documents: `application/pdf`, `application/json`, `text/plain`

## Content Integrity

All uploads include SHA-256 content hashing for verification:

```bash
# Upload file
./target/release/identity upload --file heritage.jpg

# Output includes content hash
Hash: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855

# Verify after retrieval
sha256sum heritage.jpg
```

The hash ensures the retrieved content matches the original upload.

## Pinning

IPFS uses "pinning" to keep content available on your node:

```bash
# Pin content (via IpfsClient)
ipfs pin add QmYwAPJzv...

# List pinned content
ipfs pin ls --type=recursive

# Unpin content
ipfs pin rm QmYwAPJzv...
```

Pinned content remains on your local IPFS node and won't be garbage collected.

## Public Gateways

Access content via public IPFS gateways:

- **IPFS.io:** `https://ipfs.io/ipfs/QmYwAPJzv...`
- **Cloudflare:** `https://cloudflare-ipfs.com/ipfs/QmYwAPJzv...`
- **Pinata:** `https://gateway.pinata.cloud/ipfs/QmYwAPJzv...`

⚠️ **Note:** Public gateways may be slower or rate-limited. For production use, consider:
- Running your own IPFS gateway
- Using pinning services (Pinata, Infura, NFT.Storage)
- Deploying IPFS nodes in multiple geographic regions

## Testing

Run the IPFS integration test script:

```bash
./test-ipfs.sh
```

**Requirements:**
- IPFS daemon running on `127.0.0.1:5001`
- Identity CLI built (`cargo build -p opensyria-identity --release`)

**Test Steps:**
1. ✓ Check IPFS daemon status
2. ✓ Create cultural heritage token
3. ✓ Upload sample content to IPFS
4. ✓ Link CID to token
5. ✓ Retrieve content from IPFS
6. ✓ Verify content integrity
7. ✓ Upload JSON metadata

## Best Practices

### 1. Content Organization

Structure heritage content with descriptive filenames:

```
heritage/
├── sites/
│   ├── palmyra-temple-bel-2024.jpg
│   ├── damascus-umayyad-mosque-interior.mp4
│   └── bosra-amphitheatre-360.mp4
├── crafts/
│   ├── damascus-steel-forging-process.mp4
│   ├── silk-weaving-loom-setup.jpg
│   └── mosaic-technique-tutorial.mp4
└── documents/
    ├── unesco-reports/
    └── historical-manuscripts/
```

### 2. Metadata Standards

Include comprehensive metadata in token descriptions:

```json
{
  "description": "Palmyra Temple of Bel - Photo documentation of the ancient temple complex before its destruction in 2015. Captured by UNESCO heritage team.",
  "photographer": "UNESCO Heritage Documentation Project",
  "date_captured": "2010-06-15",
  "license": "CC BY-SA 4.0",
  "resolution": "4096x3072",
  "camera": "Canon EOS 5D Mark III"
}
```

### 3. Redundancy

Ensure content availability through redundancy:

```bash
# Pin to multiple IPFS nodes
ipfs pin add QmYwAPJzv... # Local node
ipfs --api /ip4/node2/tcp/5001 pin add QmYwAPJzv... # Remote node

# Use pinning services
# Pinata: https://www.pinata.cloud/
# Infura: https://infura.io/product/ipfs
# NFT.Storage: https://nft.storage/
```

### 4. Access Control

For sensitive heritage content, consider:

- **Encryption:** Encrypt files before uploading to IPFS
- **Private IPFS networks:** Use IPFS swarm keys for closed networks
- **Permissioned tokens:** Add access control to identity tokens

### 5. Content Versioning

Track content updates with versioning:

```bash
# Initial upload
./target/release/identity upload \
  --file palmyra-v1.jpg \
  --token-id palmyra-ruins
# CID: QmOldVersion...

# Updated content
./target/release/identity upload \
  --file palmyra-v2.jpg \
  --token-id palmyra-ruins-v2
# CID: QmNewVersion...

# Maintain history in metadata
```

## Performance Considerations

### Upload Speed

IPFS upload speed depends on:
- Local network bandwidth
- File size and chunking
- Number of connected peers
- IPFS daemon configuration

**Optimize:**
```bash
# Increase connection limits
ipfs config --json Swarm.ConnMgr.HighWater 900
ipfs config --json Swarm.ConnMgr.LowWater 600
```

### Retrieval Speed

IPFS retrieval speed depends on:
- Content availability (number of providers)
- Geographic proximity to providers
- Gateway server load

**Optimize:**
- Pin critical content on multiple nodes
- Use CDN-backed gateways (Cloudflare IPFS)
- Deploy regional IPFS gateways

## Troubleshooting

### IPFS Daemon Not Running

```bash
# Check if daemon is running
ipfs id
# Error: api not running

# Start daemon
ipfs daemon &
```

### Upload Fails

```bash
# Check IPFS API endpoint
curl http://127.0.0.1:5001/api/v0/version

# Verify IPFS daemon logs
tail -f ~/.ipfs/logs/ipfs.log

# Test with simple file
echo "test" | ipfs add
```

### Content Not Found

```bash
# Check if content is pinned
ipfs pin ls QmYwAPJzv...

# Manually add to network
ipfs refs QmYwAPJzv...

# Try alternative gateway
curl https://ipfs.io/ipfs/QmYwAPJzv...
```

### Slow Retrieval

```bash
# Check peer count
ipfs swarm peers | wc -l

# Bootstrap to more peers
ipfs bootstrap add /ip4/104.131.131.82/tcp/4001/p2p/QmaCpDMGvV2BGHeYERUEnRQAwe3N8SzbUtfsmvsqQLuvuJ

# Use public gateway as fallback
curl https://cloudflare-ipfs.com/ipfs/QmYwAPJzv...
```

## Future Enhancements

- [ ] **IPFS Cluster:** Multi-node pinning coordination
- [ ] **IPNS:** Mutable pointers to IPFS content
- [ ] **Filecoin Integration:** Incentivized long-term storage
- [ ] **Content Moderation:** Community-driven content verification
- [ ] **Heritage DAOs:** Decentralized governance for cultural content
- [ ] **NFT Minting:** Convert heritage tokens to tradeable NFTs
- [ ] **Mobile IPFS:** Lightweight IPFS for mobile heritage apps

## Resources

- **IPFS Documentation:** https://docs.ipfs.io/
- **IPFS Desktop:** https://docs.ipfs.io/install/ipfs-desktop/
- **Public Gateways:** https://ipfs.github.io/public-gateway-checker/
- **Pinning Services:** https://docs.ipfs.io/concepts/persistence/#pinning-services
- **IPFS Companion:** Browser extension for IPFS integration

## Related Documentation

- **[IPFS Architecture](IPFS_ARCHITECTURE.md)** - Technical architecture diagrams and data flows
- **[Cultural Identity](CULTURAL_IDENTITY.md)** - Identity token standard and metadata schema
- **[Cultural Showcase](SHOWCASE.md)** - Syrian heritage examples
- **[Documentation Index](../README.md)** - Complete documentation catalog

## License

IPFS integration code licensed under MIT License. Heritage content may have separate licenses (CC BY-SA, public domain, etc.).
