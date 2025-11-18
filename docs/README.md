# OpenSyria Documentation

Complete technical documentation for the OpenSyria blockchain.

## 📚 Documentation Index

### Getting Started

| Document | Description |
|----------|-------------|
| [GETTING_STARTED.md](GETTING_STARTED.md) | Step-by-step tutorial for beginners (30-minute guide) |
| [FAQ.md](FAQ.md) | Frequently asked questions and troubleshooting |
| [../CONTRIBUTING.md](../CONTRIBUTING.md) | How to contribute to the project |

### Core Documentation

| Document | Description |
|----------|-------------|
| [ARCHITECTURE.md](ARCHITECTURE.md) | System architecture, components, data flows, security model |
| [DEPLOYMENT.md](DEPLOYMENT.md) | Installation, configuration, production deployment guide |

### Feature Documentation

#### Identity & Heritage System
- [CULTURAL_IDENTITY.md](identity/CULTURAL_IDENTITY.md) - Heritage token standard, metadata schema
- [CULTURAL_IDENTITY_SUMMARY.md](identity/CULTURAL_IDENTITY_SUMMARY.md) - Quick reference guide
- [SHOWCASE.md](identity/SHOWCASE.md) - Syrian heritage examples and use cases
- [IPFS_INTEGRATION.md](identity/IPFS_INTEGRATION.md) - Decentralized multimedia storage guide
- [IPFS_ARCHITECTURE.md](identity/IPFS_ARCHITECTURE.md) - IPFS system architecture and data flows

#### Networking
- [P2P_NETWORKING.md](network/P2P_NETWORKING.md) - libp2p architecture, protocols, performance
- [NETWORK_CLI.md](network/NETWORK_CLI.md) - Network commands, multi-node setup
- [NETWORK_IMPLEMENTATION.md](network/NETWORK_IMPLEMENTATION.md) - Implementation details

#### Governance
- [GOVERNANCE.md](governance/GOVERNANCE.md) - On-chain proposals, voting, execution system

#### API
- [WALLET_API.md](api/WALLET_API.md) - REST API endpoints for wallet operations

#### Testing
- [INTEGRATION_TESTS.md](tests/INTEGRATION_TESTS.md) - Multi-node integration test guide
- [INTEGRATION_TESTS_SUMMARY.md](tests/INTEGRATION_TESTS_SUMMARY.md) - Test summary

## 🗂️ Documentation Structure

```
opensyria/
├── README.md                        # Main project README
├── CONTRIBUTING.md                  # Contribution guidelines
├── CHANGELOG.md                     # Version history
├── LICENSE-MIT                      # MIT License
├── LICENSE-APACHE                   # Apache 2.0 License
│
├── docs/
│   ├── README.md                    # This file - documentation index
│   ├── GETTING_STARTED.md           # Beginner's tutorial
│   ├── FAQ.md                       # Frequently asked questions
│   ├── ARCHITECTURE.md              # System architecture
│   ├── DEPLOYMENT.md                # Deployment guide
│   │
│   ├── identity/                    # Cultural identity system
│   │   ├── CULTURAL_IDENTITY.md
│   │   ├── CULTURAL_IDENTITY_SUMMARY.md
│   │   ├── SHOWCASE.md
│   │   ├── IPFS_INTEGRATION.md
│   │   └── IPFS_ARCHITECTURE.md
│   │
│   ├── network/                     # P2P networking
│   │   ├── P2P_NETWORKING.md
│   │   ├── NETWORK_CLI.md
│   │   └── NETWORK_IMPLEMENTATION.md
│   │
│   ├── governance/                  # Governance system
│   │   └── GOVERNANCE.md
│   │
│   ├── api/                         # API documentation
│   │   └── WALLET_API.md
│   │
│   └── tests/                       # Testing guides
│       ├── INTEGRATION_TESTS.md
│       └── INTEGRATION_TESTS_SUMMARY.md
│
└── scripts/
    ├── README.md                    # Test scripts documentation
    ├── test-network.sh
    ├── test-multisig.sh
    ├── test-pool.sh
    ├── test-ipfs.sh
    ├── test-wallet-api.sh
    └── test-daemon.sh
```

## 🚀 Quick Start Guides

### New to Open Syria?
**Start here:** [GETTING_STARTED.md](GETTING_STARTED.md) - Complete 30-minute beginner tutorial

### For Developers
1. [GETTING_STARTED.md](GETTING_STARTED.md) - Installation and first steps
2. [ARCHITECTURE.md](ARCHITECTURE.md) - Understand system design
3. [../CONTRIBUTING.md](../CONTRIBUTING.md) - Contribution guidelines
4. [INTEGRATION_TESTS.md](tests/INTEGRATION_TESTS.md) - Run tests

### For Node Operators
1. [GETTING_STARTED.md](GETTING_STARTED.md) - Basic setup
2. [DEPLOYMENT.md](DEPLOYMENT.md) - Production deployment
3. [NETWORK_CLI.md](network/NETWORK_CLI.md) - Network operations
4. [P2P_NETWORKING.md](network/P2P_NETWORKING.md) - Network architecture

### For Heritage Contributors
1. [GETTING_STARTED.md#creating-heritage-tokens](GETTING_STARTED.md#creating-heritage-tokens) - Quick intro
2. [CULTURAL_IDENTITY.md](identity/CULTURAL_IDENTITY.md) - Token system overview
3. [SHOWCASE.md](identity/SHOWCASE.md) - Heritage examples
4. [IPFS_INTEGRATION.md](identity/IPFS_INTEGRATION.md) - Upload multimedia content

### For API Users
1. [WALLET_API.md](api/WALLET_API.md) - REST API reference
2. [FAQ.md#can-i-build-applications-on-top](FAQ.md#can-i-build-applications-on-top) - Building apps

### For Governance Participants
1. [GOVERNANCE.md](governance/GOVERNANCE.md) - Proposal and voting system
2. [FAQ.md#governance](FAQ.md#governance) - Common questions

## 📖 Documentation by Topic

### Blockchain Core
- **Architecture**: [ARCHITECTURE.md](ARCHITECTURE.md)
- **Consensus**: [ARCHITECTURE.md#consensus](ARCHITECTURE.md) - PoW implementation
- **Storage**: [ARCHITECTURE.md#storage](ARCHITECTURE.md) - RocksDB persistence
- **Transactions**: [ARCHITECTURE.md#transactions](ARCHITECTURE.md) - Transaction flow

### Cultural Heritage
- **Token Standard**: [CULTURAL_IDENTITY.md](identity/CULTURAL_IDENTITY.md)
- **Metadata Schema**: [CULTURAL_IDENTITY.md#metadata](identity/CULTURAL_IDENTITY.md)
- **IPFS Storage**: [IPFS_INTEGRATION.md](identity/IPFS_INTEGRATION.md)
- **Use Cases**: [SHOWCASE.md](identity/SHOWCASE.md)

### Networking
- **P2P Protocol**: [P2P_NETWORKING.md](network/P2P_NETWORKING.md)
- **Node Setup**: [NETWORK_CLI.md](network/NETWORK_CLI.md)
- **Implementation**: [NETWORK_IMPLEMENTATION.md](network/NETWORK_IMPLEMENTATION.md)

### Governance
- **Proposals**: [GOVERNANCE.md#proposals](governance/GOVERNANCE.md)
- **Voting**: [GOVERNANCE.md#voting](governance/GOVERNANCE.md)
- **Execution**: [GOVERNANCE.md#execution](governance/GOVERNANCE.md)

### Development
- **Testing**: [INTEGRATION_TESTS.md](tests/INTEGRATION_TESTS.md)
- **API Development**: [WALLET_API.md](api/WALLET_API.md)
- **Deployment**: [DEPLOYMENT.md](DEPLOYMENT.md)

## 🔍 Find Documentation

### By Component
- **Core Blockchain**: ARCHITECTURE.md, DEPLOYMENT.md
- **Identity System**: identity/*.md
- **P2P Network**: network/*.md
- **Governance**: governance/GOVERNANCE.md
- **APIs**: api/*.md
- **Testing**: tests/*.md

### By Role
- **Blockchain Developer**: ARCHITECTURE.md, tests/INTEGRATION_TESTS.md
- **Node Operator**: DEPLOYMENT.md, network/NETWORK_CLI.md
- **Heritage Contributor**: identity/CULTURAL_IDENTITY.md, identity/IPFS_INTEGRATION.md
- **dApp Developer**: api/WALLET_API.md
- **Governance Member**: governance/GOVERNANCE.md

### By Task
- **Setup Node**: DEPLOYMENT.md → network/NETWORK_CLI.md
- **Create Heritage Token**: identity/CULTURAL_IDENTITY.md → identity/SHOWCASE.md
- **Upload Media**: identity/IPFS_INTEGRATION.md
- **Join Network**: network/P2P_NETWORKING.md → network/NETWORK_CLI.md
- **Create Proposal**: governance/GOVERNANCE.md
- **Build API Integration**: api/WALLET_API.md
- **Run Tests**: tests/INTEGRATION_TESTS.md

## 📝 Documentation Standards

All documentation follows these standards:
- **Bilingual**: Arabic/English where applicable
- **Code Examples**: Practical, working examples
- **Architecture Diagrams**: Visual system overviews
- **CLI Commands**: Copy-paste ready terminal commands
- **Test Coverage**: Testing instructions included
- **Troubleshooting**: Common issues and solutions

## 🆕 Recent Updates

### November 2025
- ✅ Added Getting Started tutorial (beginner-friendly 30-minute guide)
- ✅ Added comprehensive FAQ with 40+ questions
- ✅ Added CONTRIBUTING.md with contribution guidelines
- ✅ Added dual MIT/Apache-2.0 licensing
- ✅ Added scripts/README.md documenting test scripts
- ✅ Added IPFS integration documentation
- ✅ Reorganized docs into logical subdirectories
- ✅ Created comprehensive documentation index
- ✅ Updated all cross-references

### October 2025
- ✅ Added governance system documentation
- ✅ Added wallet API documentation
- ✅ Added integration test guides

### September 2025
- ✅ Added P2P networking documentation
- ✅ Added network CLI guides
- ✅ Added cultural identity documentation

## 🤝 Contributing to Documentation

Documentation improvements are welcome! When contributing:

1. **Update existing docs** rather than creating new ones when possible
2. **Follow the structure** outlined in this README
3. **Include examples** - code, CLI commands, JSON samples
4. **Add diagrams** for complex concepts (ASCII art is fine)
5. **Test commands** before documenting them
6. **Update this index** when adding new documentation

## 📧 Support

- **Technical Issues**: Check relevant .md files in this directory
- **Feature Requests**: See governance/GOVERNANCE.md for proposal process
- **Community**: Join OpenSyria discussions (links in main README.md)

## 📄 License

Documentation is licensed under MIT License, same as the codebase.
