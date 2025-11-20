# Formal Verification Guide for OpenSyria Consensus

## Overview

This directory contains formal specifications for verifying the correctness of OpenSyria's consensus mechanism using TLA+ (Temporal Logic of Actions).

## Files

- `OpenSyriaConsensus.tla` - Main TLA+ specification for PoW consensus
- `OpenSyriaConsensus.cfg` - TLA+ model configuration
- `verification_results.md` - Model checking results and analysis

## What is Formal Verification?

Formal verification mathematically proves that a system satisfies its specification. Unlike testing (which checks specific cases), formal verification provides guarantees across all possible executions within a bounded model.

## Properties Verified

### Safety Properties (Nothing bad ever happens)
1. **Consensus Agreement**: No two honest nodes accept conflicting blocks at same height
2. **Timestamp Safety**: Median-time-past prevents timewarp attacks
3. **Difficulty Bounded**: Difficulty adjustment stays within safe bounds
4. **Reorg Depth Limited**: Reorganizations cannot exceed MAX_REORG_DEPTH

### Liveness Properties (Something good eventually happens)
1. **Chain Growth**: Blockchain continues to grow under honest majority
2. **Transaction Confirmation**: Transactions eventually get confirmed

### Attack Resistance
1. **Selfish Mining**: Not profitable with <33% hashrate
2. **Long-Range Attack**: Prevented by MAX_REORG_DEPTH
3. **Timewarp Attack**: MTP prevents difficulty manipulation

## Running the Verification

### Prerequisites

1. **Install TLA+ Toolbox**
   ```bash
   # macOS
   brew install --cask tla-plus-toolbox
   
   # Linux
   wget https://github.com/tlaplus/tlaplus/releases/download/v1.8.0/TLAToolbox-1.8.0-linux.gtk.x86_64.zip
   unzip TLAToolbox-1.8.0-linux.gtk.x86_64.zip
   ```

2. **Open Specification**
   - Launch TLA+ Toolbox
   - File → Open Spec → Add New Spec
   - Select `OpenSyriaConsensus.tla`

### Model Configuration

Create a new model with these settings:

**Constants:**
```
Nodes <- {n1, n2, n3}
HonestNodes <- {n1, n2}
AttackerNodes <- {n3}
MaxHeight <- 10
MaxReorgDepth <- 3
MaxFutureDrift <- 60
DifficultyAdjustInterval <- 5
TargetBlockTime <- 600
```

**Invariants to Check:**
- TypeOK
- ConsensusAgreement
- TimestampSafety
- DifficultyBounded
- ReorgDepthLimited

**Temporal Properties:**
- ChainGrowth
- TransactionConfirmed

**Model Parameters:**
- Depth: 10 (for liveness)
- Workers: 4 (parallel model checking)

### Run Model Checker

1. Click "Run TLC on the model"
2. Wait for completion (may take 5-30 minutes)
3. Review results:
   - ✅ Green: All properties hold
   - ❌ Red: Invariant violated (review error trace)

## Interpreting Results

### Success Case
```
TLC finished checking.
States: 12,847
Distinct states: 8,923
State space fully explored.
No errors found.
```
**Meaning:** Consensus properties proven correct for bounded model.

### Failure Case
```
Invariant ConsensusAgreement is violated.
Error trace:
  State 1: [chains |-> ...]
  State 2: [chains |-> ...]
  ...
```
**Action:** Review error trace, identify bug, fix Rust implementation.

## Limitations

1. **Bounded Model Checking**: Only explores finite state space
2. **Abstraction**: PoW hash function simplified
3. **Network Model**: Assumes eventual message delivery
4. **Scalability**: Limited to ~3 nodes for tractable verification

## Extending Verification

### Additional Properties to Verify

1. **Transaction Validation**
   ```tla
   ValidTransaction ==
       \A tx \in Transactions:
           /\ tx.nonce = expected_nonce[tx.sender]
           /\ tx.signature = Ed25519.Sign(tx.sender.private_key, tx.data)
   ```

2. **Supply Enforcement**
   ```tla
   MaxSupplyInvariant ==
       Sum({balance[addr] : addr \in Addresses}) <= MAX_SUPPLY
   ```

3. **Fee Market**
   ```tla
   FeeOrdering ==
       \A tx1, tx2 \in Mempool:
           tx1.fee > tx2.fee => tx1.priority > tx2.priority
   ```

### Refinement Mapping

To prove Rust implementation matches TLA+ spec:

1. **Define Abstraction Function**
   ```rust
   fn abstract_state(blockchain: &Blockchain) -> TLAState {
       TLAState {
           chains: blockchain.chains.iter().map(|c| abstract_chain(c)).collect(),
           difficulty: blockchain.current_difficulty,
           // ... other fields
       }
   }
   ```

2. **Runtime Verification**
   ```rust
   #[cfg(test)]
   fn verify_consensus_agreement(blockchain: &Blockchain) {
       let tla_state = abstract_state(blockchain);
       assert!(tla_state.satisfies_consensus_agreement());
   }
   ```

## Resources

- **TLA+ Tutorial**: https://learntla.com/
- **Lamport's TLA+ Guide**: https://lamport.azurewebsites.net/tla/tla.html
- **Blockchain Verification**: https://github.com/tlaplus/Examples/tree/master/specifications/blockchain
- **Ethereum 2.0 TLA+**: https://github.com/ethereum/eth2.0-specs/tree/dev/specs/phase0

## Contact

For questions about formal verification:
- Security Team: security@opensyria.org
- Formal Methods Lead: [TBD]

---

*"In mathematics, we trust. In code, we verify."*  
*"في الرياضيات، نثق. في الكود، نتحقق"*
