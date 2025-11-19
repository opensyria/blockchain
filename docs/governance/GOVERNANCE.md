# Governance System Documentation

## Overview

The OpenSyria blockchain governance system enables on-chain decision-making through proposals and voting. This decentralized governance framework allows stakeholders to participate in protocol upgrades, parameter changes, and treasury management.

## Architecture

### Components

1. **Proposal System**: Create and manage governance proposals
2. **Voting Mechanism**: Stake-weighted voting with configurable thresholds
3. **Execution Engine**: Automatic execution of passed proposals
4. **Storage Layer**: Persistent storage for proposals and votes

### Proposal Types

```rust
pub enum ProposalType {
    DifficultyAdjustment {
        target_block_time: u64,
        adjustment_interval: u32,
    },
    MinimumFee { new_fee: u64 },
    BlockSizeLimit { new_limit: usize },
    BlockReward { new_reward: u64 },
    TreasurySpending {
        recipient: PublicKey,
        amount: u64,
        description: String,
    },
    ProtocolUpgrade {
        version: u32,
        activation_height: u64,
        description: String,
    },
    TextProposal { description: String },
}
```

## Voting Process

### 1. Proposal Creation

```bash
# Create a text proposal
governance-cli propose \
  --title "Increase Block Size" \
  --description "Proposal to increase block size limit to 2MB" \
  -t text

# Create a fee adjustment proposal
governance-cli propose \
  --title "Update Minimum Fee" \
  --description "Set minimum fee to 200 units" \
  -t min-fee
```

**Requirements:**
- Minimum stake: 1000 Lira (configurable)
- Title and description required
- Valid proposal type

### 2. Voting Period

Default: **10,080 blocks** (~1 week at 1 minute per block)

```bash
# Vote yes on proposal #1
governance-cli vote 1 --choice yes

# Vote no
governance-cli vote 1 --choice no

# Abstain
governance-cli vote 1 --choice abstain
```

**Voting Rules:**
- One vote per address per proposal
- Voting power based on stake at proposal creation
- Cannot change vote after casting

### 3. Proposal Finalization

Proposals are finalized when the voting period ends.

**Quorum & Threshold Requirements:**

| Proposal Type | Quorum | Threshold |
|--------------|--------|-----------|
| Protocol Upgrade | 50% | 75% |
| Treasury Spending | 40% | 66% |
| Block Reward | 40% | 66% |
| Difficulty Adjustment | 30% | 60% |
| Minimum Fee | 30% | 60% |
| Block Size Limit | 30% | 60% |
| Text Proposal | 20% | 50% |

**Example:**
- Total voting power: 100,000,000
- Protocol upgrade proposal needs:
  - Quorum: 50,000,000 votes cast (50%)
  - Threshold: 75% of votes must be "Yes"

### 4. Execution

Passed proposals enter an execution delay period (default: **1,440 blocks** or ~1 day).

After the delay, proposals are automatically executed by the node.

## CLI Usage

### Node CLI (Integrated Governance)

The recommended way to interact with governance is through the node CLI, which automatically integrates with your blockchain state:

```bash
# Create a proposal (requires hex-encoded public key)
opensyria-node-cli governance propose \
  --title "Your Proposal Title" \
  --description "Detailed description" \
  -t <proposal-type> \
  --proposer <hex-public-key>

# Vote on a proposal
opensyria-node-cli governance vote <proposal-id> \
  --choice <yes|no|abstain> \
  --voter <hex-public-key>

# List all proposals
opensyria-node-cli governance list

# Filter by status
opensyria-node-cli governance list --status active
opensyria-node-cli governance list --status passed

# Show proposal details
opensyria-node-cli governance show <proposal-id>

# View statistics
opensyria-node-cli governance stats

# Process proposals (finalize voting periods, execute passed proposals)
opensyria-node-cli governance process
```

**Advantages of Node CLI:**
- Voting power automatically calculated from account balances
- Proposals stored in node's data directory
- Automatic execution when proposals pass
- No separate governance daemon needed
- Current block height used for timestamps

### Standalone Governance CLI (For Testing)

For development and testing, a standalone governance CLI is also available:

```bash
governance-cli init
```

Creates governance storage in `data/governance/` directory.

### Create Proposals

```bash
governance-cli propose \
  --title "Your Proposal Title" \
  --description "Detailed description of what this proposal does" \
  -t <proposal-type>
```

Available types:
- `text` - Non-binding text proposal
- `min-fee` - Change minimum transaction fee
- More types coming soon

### List Proposals

```bash
# All proposals
governance-cli list

# Filter by status
governance-cli list --status active
governance-cli list --status passed
governance-cli list --status rejected
```

### View Proposal Details

```bash
governance-cli show <proposal-id>
```

Shows:
- Proposal metadata
- Voting statistics
- Quorum/threshold status
- All votes cast

### Cast Votes

```bash
governance-cli vote <proposal-id> --choice <yes|no|abstain>
```

### View Statistics

```bash
governance-cli stats
```

Displays:
- Total proposals
- Breakdown by status
- Total votes cast
- Configuration parameters

## Configuration

```rust
pub struct GovernanceConfig {
    /// Minimum stake required to create proposal (Lira units)
    pub min_proposal_stake: u64,  // Default: 1,000,000,000 (1000 Lira)
    
    /// Voting period in blocks
    pub default_voting_period: u64,  // Default: 10,080 blocks
    
    /// Execution delay in blocks
    pub default_execution_delay: u64,  // Default: 1,440 blocks
    
    /// Enable/disable governance
    pub enabled: bool,  // Default: true
}
```

## Proposal Lifecycle

```
┌─────────────┐
│   CREATED   │
│             │
│ - Proposal  │
│   submitted │
│ - Voting    │
│   period    │
│   starts    │
└──────┬──────┘
       │
       ▼
┌─────────────┐     ┌─────────────┐
│   ACTIVE    │────▶│  CANCELLED  │
│             │     │             │
│ - Voting    │     │ - Proposer  │
│   ongoing   │     │   cancelled │
│ - Can be    │     └─────────────┘
│   cancelled │
└──────┬──────┘
       │
       │ Voting ends
       ▼
┌─────────────┐     ┌─────────────┐
│   PASSED    │     │  REJECTED   │
│             │     │             │
│ - Quorum    │     │ - Failed    │
│   met       │     │   quorum or │
│ - Threshold │     │   threshold │
│   met       │     └─────────────┘
│ - Execution │
│   delay     │
└──────┬──────┘
       │
       │ After delay
       ▼
┌─────────────┐
│  EXECUTED   │
│             │
│ - Changes   │
│   applied   │
└─────────────┘
```

## Integration with Node

### Storage Structure

```
data/
├── blocks/          # Blockchain data
├── state/           # Account balances and nonces
└── governance/      # Governance proposals and votes
    ├── CURRENT
    ├── LOCK
    ├── LOG
    └── *.sst (RocksDB files)
```

### Automatic Processing

The node automatically:
1. **Initializes governance** when you run `init`
2. **Loads governance state** when opening the node
3. **Calculates voting power** from account balances in state
4. **Uses current block height** for proposal timestamps
5. **Executes passed proposals** when you call `governance process`

### Proposal Execution

When a proposal passes and the execution delay expires:

```bash
# Process proposals (call this periodically or after mining blocks)
opensyria-node-cli governance process
```

This command will:
- Finalize proposals with ended voting periods
- Execute proposals that are ready
- Apply parameter changes to node configuration
- Mark proposals as executed

**Currently Supported Executions:**
- ✅ Text proposals (logged, non-binding)
- 🔄 Minimum fee changes (logged, requires mempool integration)
- 🔄 Block size limits (logged, requires consensus integration)
- 🔄 Difficulty adjustment parameters (logged, requires consensus integration)
- ⏳ Treasury spending (future implementation)
- ⏳ Protocol upgrades (future implementation)

### Node Commands Summary

| Command | Description |
|---------|-------------|
| `governance propose` | Create new proposal |
| `governance vote` | Cast vote on proposal |
| `governance list` | List all proposals |
| `governance show` | Show proposal details |
| `governance stats` | Show statistics |
| `governance process` | Finalize and execute proposals |

**Using Governance with Node:**

```rust
use opensyria_node_cli::Node;
use opensyria_governance::{ProposalType, Vote};
use opensyria_core::crypto::PublicKey;

// Open node
let node = Node::open("/path/to/data".into())?;

// Create proposal
let proposer = PublicKey::from_hex("...")?;
let proposal_id = node.create_proposal(
    proposer,
    ProposalType::MinimumFee { new_fee: 200 },
    "Increase Fee".to_string(),
    "Double minimum fee".to_string(),
)?;

// Vote on proposal
let voter = PublicKey::from_hex("...")?;
node.vote_on_proposal(proposal_id, voter, Vote::Yes)?;

// Process proposals (finalize and execute)
let finalized_count = node.process_proposals()?;
```

**Standalone Usage:**

```rust
use opensyria_governance::{
    GovernanceManager, GovernanceConfig, ProposalType, Vote
};

// Initialize
let config = GovernanceConfig::default();
let mut manager = GovernanceManager::new(config);

// Create proposal
let proposal_id = manager.create_proposal(
    proposer_key,
    proposer_stake,
    ProposalType::MinimumFee { new_fee: 200 },
    "Increase Fee".to_string(),
    "Double minimum fee".to_string(),
    current_height,
    total_voting_power,
)?;

// Vote
manager.vote(
    proposal_id,
    voter_key,
    Vote::Yes,
    voting_power,
    current_height,
)?;

// Process proposals (call per block)
manager.process_proposals(current_height);

// Check ready for execution
let ready = manager.get_ready_for_execution(current_height);
for proposal in ready {
    // Execute proposal
    execute_proposal(proposal)?;
    manager.mark_proposal_executed(proposal.id)?;
}
```

## Security Considerations

### Stake Requirements

The minimum stake requirement prevents spam proposals:
- Default: 1000 Lira
- Stake is not locked or consumed
- Only checked at proposal creation

### Sybil Resistance

Voting power is based on stake, not number of addresses:
- One address with 10,000 Lira = 10,000 voting power
- Ten addresses with 1,000 Lira each = 10,000 voting power
- Makes vote buying expensive

### Time Delays

Execution delays provide:
- Time to detect malicious proposals
- Emergency response window
- User awareness period

### Quorum Requirements

Higher quorum for critical changes:
- Protocol upgrades: 50% quorum, 75% threshold
- Treasury spending: 40% quorum, 66% threshold
- Text proposals: 20% quorum, 50% threshold

## Examples

### Example 1: Protocol Upgrade

```bash
# Create proposal
governance-cli propose \
  --title "Upgrade to v2.0" \
  --description "Activate new features at block 100000" \
  -t protocol-upgrade

# Check proposal
governance-cli show 1

# Vote (multiple users)
governance-cli vote 1 --choice yes

# After voting period, check if passed
governance-cli show 1

# If passed, node automatically executes after delay
```

### Example 2: Fee Adjustment

```bash
# Propose fee change
governance-cli propose \
  --title "Reduce Transaction Fees" \
  --description "Lower minimum fee to improve accessibility" \
  -t min-fee

# Community votes
governance-cli vote 2 --choice yes

# Monitor progress
governance-cli list --status active
```

### Example 3: Treasury Spending

```bash
# Propose spending
governance-cli propose \
  --title "Fund Development Team" \
  --description "Allocate 100,000 Lira to core development" \
  -t treasury

# Vote and track
governance-cli vote 3 --choice yes
governance-cli show 3
```

## Future Enhancements

### Planned Features

1. **Delegation**: Delegate voting power to trusted addresses
2. **Quadratic Voting**: Reduce whale influence
3. **Multi-signature Proposals**: Require multiple proposers
4. **Proposal Deposits**: Lock stake during voting
5. **Veto Mechanism**: Emergency veto for critical issues
6. **Automated Execution**: More proposal types
7. **Governance Tokens**: Separate governance token (future PoS)

### Roadmap

**Phase 1** (Current):
- ✅ Basic proposal types
- ✅ Stake-weighted voting
- ✅ Quorum and threshold checks
- ✅ CLI interface
- ✅ Persistent storage

**Phase 2** (Next):
- 🔄 Node integration
- 🔄 Automatic execution
- 🔄 REST API endpoints
- 🔄 Web UI for governance

**Phase 3** (Future):
- ⏳ Vote delegation
- ⏳ More proposal types
- ⏳ Governance analytics
- ⏳ Multi-sig proposals

## Testing

```bash
# Run governance tests
cargo test -p opensyria-governance

# Test CLI
governance-cli --help
governance-cli init
governance-cli stats
```

**Test Coverage:**
- 23 unit tests
- Proposal lifecycle tests
- Voting mechanism tests
- Storage persistence tests
- State management tests

## Troubleshooting

### Common Issues

**"Governance not initialized"**
```bash
governance-cli init
```

**"Insufficient stake"**
- Ensure proposer has at least 1000 Lira
- Check balance with wallet CLI

**"Voting not active"**
- Check current block height
- View proposal with `governance-cli show <id>`
- Voting may not have started or may have ended

**"Already voted"**
- Each address can only vote once per proposal
- Use different address or wait for next proposal

## References

- [Proposal Types](../crates/governance/src/types.rs)
- [Governance Manager](../crates/governance/src/manager.rs)
- [Storage Layer](../crates/governance/src/storage.rs)
- [CLI Tool](../crates/governance/src/bin/governance-cli.rs)

---

**Last Updated:** November 2025  
**Version:** 0.1.0  
**Status:** Implemented ✅


## Related Documentation

- **[Cultural Identity](../identity/CULTURAL_IDENTITY.md)** - Heritage token standard
- **[IPFS Integration](../identity/IPFS_INTEGRATION.md)** - Decentralized storage
- **[Network CLI](../network/NETWORK_CLI.md)** - Network operations
- **[Wallet API](../api/WALLET_API.md)** - REST API reference
- **[Documentation Index](../README.md)** - Complete documentation catalog

