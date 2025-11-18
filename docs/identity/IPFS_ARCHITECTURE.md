# IPFS Integration Architecture

## System Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                      OpenSyria Blockchain                            │
│                                                                       │
│  ┌─────────────────┐         ┌──────────────────┐                   │
│  │ Identity Tokens │◄────────│ Identity Registry│                   │
│  └────────┬────────┘         └──────────────────┘                   │
│           │                                                           │
│           │ ipfs_cid: "QmXyz..."                                     │
│           │                                                           │
└───────────┼───────────────────────────────────────────────────────────┘
            │
            │ Content-Addressed Link
            │
            ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         IPFS Network                                 │
│                                                                       │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐          │
│  │ Local Node   │◄───│  Peer Node 1 │◄───│  Peer Node 2 │          │
│  │ 127.0.0.1    │    │  Public      │    │  Public      │          │
│  └──────┬───────┘    └──────────────┘    └──────────────┘          │
│         │                                                             │
│         │ Stores:                                                    │
│         │  - Heritage photos/videos                                 │
│         │  - Historical documents                                   │
│         │  - Cultural artifacts                                     │
│         │  - Oral tradition recordings                              │
│         │                                                             │
└─────────┼─────────────────────────────────────────────────────────────┘
          │
          │ HTTP Gateway
          │
          ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      Gateway URLs                                    │
│                                                                       │
│  Local:   http://127.0.0.1:8080/ipfs/QmXyz...                       │
│  Public:  https://ipfs.io/ipfs/QmXyz...                             │
│  CDN:     https://cloudflare-ipfs.com/ipfs/QmXyz...                 │
│                                                                       │
└─────────────────────────────────────────────────────────────────────┘
```

## Data Flow

### Upload Flow

```
┌──────────┐
│   User   │
└────┬─────┘
     │ 1. identity upload --file heritage.jpg --token-id palmyra
     ▼
┌────────────────┐
│ Identity CLI   │
└────┬───────────┘
     │ 2. Read file + compute SHA-256 hash
     ▼
┌────────────────┐
│  IpfsClient    │
└────┬───────────┘
     │ 3. HTTP POST /api/v0/add (multipart form)
     ▼
┌────────────────┐
│ IPFS Daemon    │
└────┬───────────┘
     │ 4. Store content chunks
     │ 5. Return CID (QmXyz...)
     ▼
┌────────────────┐
│ Identity Token │  ipfs_cid: "QmXyz..."
│  (JSON file)   │  content_hash: "abc123..."
└────────────────┘
```

### Retrieve Flow

```
┌──────────┐
│   User   │
└────┬─────┘
     │ 1. identity retrieve QmXyz... --output file.jpg
     ▼
┌────────────────┐
│ Identity CLI   │
└────┬───────────┘
     │ 2. Request content
     ▼
┌────────────────┐
│  IpfsClient    │
└────┬───────────┘
     │ 3. HTTP GET /ipfs/QmXyz... (via gateway)
     ▼
┌────────────────┐
│ IPFS Gateway   │
└────┬───────────┘
     │ 4. Retrieve from local/network
     │ 5. Return content bytes
     ▼
┌────────────────┐
│   Local File   │  file.jpg (verified via hash)
└────────────────┘
```

## Token Schema Evolution

### Before IPFS

```json
{
  "id": "palmyra-ruins",
  "owner": "0xabc...",
  "token_type": "HeritageSite",
  "metadata": {
    "name": "Ruins of Palmyra",
    "description": "Ancient city..."
  }
}
```

### After IPFS

```json
{
  "id": "palmyra-ruins",
  "owner": "0xabc...",
  "token_type": "HeritageSite",
  "metadata": {
    "name": "Ruins of Palmyra",
    "description": "Ancient city..."
  },
  "ipfs_cid": "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG"
}
```

## Content Metadata Structure

```rust
ContentMetadata {
    cid: "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG",
    filename: "palmyra-restoration.mp4",
    size: 13107200,  // 12.5 MB
    mime_type: "video/mp4",
    content_hash: "a3f5b8c9e2d1...",  // SHA-256
    uploaded_at: 1735084800,          // Unix timestamp
}
```

## CLI Command Architecture

```
identity upload
    ├─> Validate file exists
    ├─> Detect MIME type (.jpg → image/jpeg)
    ├─> Compute SHA-256 hash
    ├─> Create multipart form
    ├─> POST to IPFS API
    ├─> Parse CID from response
    ├─> If --token-id provided:
    │   ├─> Load token JSON
    │   ├─> Update ipfs_cid field
    │   └─> Save token JSON
    └─> Display CID + gateway URL

identity retrieve
    ├─> Validate CID format
    ├─> GET from IPFS gateway
    ├─> Stream response to file
    └─> Display size + path

identity link
    ├─> Load token JSON
    ├─> Update ipfs_cid field
    ├─> Save token JSON
    └─> Display confirmation
```

## Content Integrity Chain

```
Original File
    │
    ├─> SHA-256 Hash: a3f5b8c9...
    │
    ├─> Upload to IPFS
    │
    ├─> IPFS CID: QmXyz... (content-addressed)
    │
    ├─> Store in Token: ipfs_cid + content_hash
    │
    └─> Retrieve & Verify:
        ├─> Download via CID
        ├─> Compute SHA-256
        └─> Compare with stored hash ✓
```

## Use Case: Heritage Site Documentation

```
Palmyra Heritage Site
    │
    ├─> Identity Token (On-Chain)
    │   ├─> id: "palmyra-ruins"
    │   ├─> name: "Ruins of Palmyra"
    │   ├─> metadata: {...}
    │   └─> ipfs_cid: "QmAbc..."
    │
    └─> IPFS Content (Off-Chain)
        ├─> Photos/
        │   ├─> temple-of-bel.jpg
        │   ├─> tetrapylon.jpg
        │   └─> valley-of-tombs.jpg
        ├─> Videos/
        │   ├─> 360-tour.mp4
        │   └─> restoration-documentary.mp4
        ├─> Audio/
        │   └─> oral-history-interview.mp3
        └─> Documents/
            ├─> unesco-report-2010.pdf
            └─> archaeological-survey.json
```

## Future Enhancements Roadmap

```
Phase 1: Basic IPFS Integration ✓
    ├─> IpfsClient module
    ├─> Upload/retrieve/link commands
    ├─> Content metadata tracking
    └─> SHA-256 integrity verification

Phase 2: Advanced Features
    ├─> IPNS (mutable content pointers)
    ├─> IPFS Cluster (multi-node pinning)
    ├─> Content encryption (sensitive heritage)
    └─> Batch upload (entire heritage collections)

Phase 3: Incentivized Storage
    ├─> Filecoin integration
    ├─> Storage marketplace
    ├─> Redundancy guarantees
    └─> Long-term preservation contracts

Phase 4: Heritage NFTs
    ├─> Convert tokens to tradeable NFTs
    ├─> Royalty mechanisms for artists
    ├─> Provenance tracking
    └─> Digital exhibitions

Phase 5: Decentralized Governance
    ├─> Heritage DAOs
    ├─> Community content verification
    ├─> Collaborative documentation
    └─> Cultural preservation funding
```
