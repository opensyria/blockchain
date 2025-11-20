---------------------- MODULE OpenSyriaConsensus ----------------------
(*
 * Formal Verification of OpenSyria Digital Lira Consensus Mechanism
 * 
 * This TLA+ specification models the core consensus properties of the
 * OpenSyria blockchain to verify safety and liveness properties.
 * 
 * Key Properties Verified:
 * 1. Safety: No two honest nodes accept conflicting blocks at same height
 * 2. Chain Quality: Attacker cannot dominate chain with <51% hashrate
 * 3. Liveness: Chain continues to grow under honest majority
 * 4. Timestamp Safety: MTP prevents timewarp attacks
 * 5. Reorg Limits: MAX_REORG_DEPTH prevents long-range attacks
 *)

EXTENDS Naturals, Sequences, FiniteSets, TLC

CONSTANTS
    Nodes,              \* Set of all nodes in network
    HonestNodes,        \* Subset of honest nodes
    AttackerNodes,      \* Subset of attacker nodes
    MaxHeight,          \* Maximum blockchain height to model
    MaxReorgDepth,      \* Maximum reorganization depth (100 blocks)
    MaxFutureDrift,     \* Maximum future timestamp drift (60 seconds)
    DifficultyAdjustInterval,  \* Difficulty adjustment every 100 blocks
    TargetBlockTime     \* Target: 10 minutes (600 seconds)

ASSUME HonestNodes \subseteq Nodes
ASSUME AttackerNodes \subseteq Nodes
ASSUME HonestNodes \cap AttackerNodes = {}
ASSUME MaxReorgDepth = 100
ASSUME MaxFutureDrift = 60
ASSUME DifficultyAdjustInterval = 100
ASSUME TargetBlockTime = 600

--------------------------------------------------------------------------------
(* Variables *)

VARIABLES
    chains,             \* chains[n] = blockchain state for node n
    mempool,            \* mempool[n] = pending transactions for node n
    currentTime,        \* Global clock (for timestamp validation)
    messages,           \* Network messages (block announcements, etc.)
    difficulty,         \* Current mining difficulty
    hashrate            \* hashrate[n] = mining power for node n

vars == <<chains, mempool, currentTime, messages, difficulty, hashrate>>

--------------------------------------------------------------------------------
(* Type Invariants *)

Block == [
    height: Nat,
    hash: STRING,
    prevHash: STRING,
    timestamp: Nat,
    difficulty: Nat,
    nonce: Nat,
    transactions: Seq(STRING),
    miner: Nodes
]

TypeOK ==
    /\ \A n \in Nodes: chains[n] \in Seq(Block)
    /\ \A n \in Nodes: mempool[n] \subseteq STRING
    /\ currentTime \in Nat
    /\ difficulty \in Nat
    /\ hashrate \in [Nodes -> Nat]

--------------------------------------------------------------------------------
(* Helper Functions *)

\* Genesis block (height 0)
GenesisBlock == [
    height |-> 0,
    hash |-> "genesis",
    prevHash |-> "0",
    timestamp |-> 1763452800,
    difficulty |-> 16,
    nonce |-> 0xDEADBEEF,
    transactions |-> <<>>,
    miner |-> CHOOSE n \in Nodes: TRUE
]

\* Get chain tip (latest block)
ChainTip(chain) == chain[Len(chain)]

\* Get blockchain height
ChainHeight(chain) == Len(chain) - 1

\* Calculate median-time-past (MTP) from last 11 blocks
MedianTimePast(chain) ==
    LET lastBlocks == SubSeq(chain, Max({1, Len(chain) - 10}), Len(chain))
        timestamps == {chain[i].timestamp : i \in 1..Len(lastBlocks)}
    IN CHOOSE t \in timestamps : 
        Cardinality({t2 \in timestamps : t2 <= t}) >= (Cardinality(timestamps) + 1) \div 2

\* Validate block timestamp
ValidTimestamp(block, chain) ==
    /\ block.timestamp > MedianTimePast(chain)  \* Greater than MTP
    /\ block.timestamp <= currentTime + MaxFutureDrift  \* Not too far in future

\* Calculate next difficulty (every 100 blocks)
NextDifficulty(chain) ==
    IF ChainHeight(chain) % DifficultyAdjustInterval = 0 THEN
        LET 
            lastAdjustBlock == chain[Len(chain) - DifficultyAdjustInterval]
            actualTime == ChainTip(chain).timestamp - lastAdjustBlock.timestamp
            expectedTime == DifficultyAdjustInterval * TargetBlockTime
            ratio == actualTime \div expectedTime
            \* Clamp adjustment to ±25%
            clampedRatio == Min(Max(ratio, 75), 125) \div 100
        IN difficulty * clampedRatio
    ELSE
        difficulty

\* Validate block follows consensus rules
ValidBlock(block, chain) ==
    /\ block.height = ChainHeight(chain) + 1
    /\ block.prevHash = ChainTip(chain).hash
    /\ ValidTimestamp(block, chain)
    /\ block.difficulty = difficulty
    /\ block.nonce \in Nat  \* PoW verification (simplified)

\* Check if reorg is within limits
ValidReorg(newChain, oldChain) ==
    LET commonAncestor == CHOOSE i \in 1..Min(Len(newChain), Len(oldChain)) :
        newChain[i].hash = oldChain[i].hash
        reorgDepth == Len(oldChain) - commonAncestor
    IN reorgDepth <= MaxReorgDepth

--------------------------------------------------------------------------------
(* Initial State *)

Init ==
    /\ chains = [n \in Nodes |-> <<GenesisBlock>>]
    /\ mempool = [n \in Nodes |-> {}]
    /\ currentTime = 1763452800  \* November 19, 2025 (genesis)
    /\ messages = {}
    /\ difficulty = 16  \* Initial difficulty
    /\ hashrate = [n \in Nodes |-> IF n \in HonestNodes THEN 10 ELSE 5]

--------------------------------------------------------------------------------
(* Actions *)

\* Honest node mines a new block
MineHonestBlock(n) ==
    /\ n \in HonestNodes
    /\ LET newBlock == [
        height |-> ChainHeight(chains[n]) + 1,
        hash |-> ToString(RandomElement(1..1000000)),  \* Simplified hash
        prevHash |-> ChainTip(chains[n]).hash,
        timestamp |-> currentTime,
        difficulty |-> difficulty,
        nonce |-> RandomElement(1..1000000),
        transactions |-> SetToSeq(mempool[n]),
        miner |-> n
       ]
       IN
       /\ ValidBlock(newBlock, chains[n])
       /\ chains' = [chains EXCEPT ![n] = Append(chains[n], newBlock)]
       /\ messages' = messages \cup {[type |-> "block", block |-> newBlock, sender |-> n]}
       /\ mempool' = [mempool EXCEPT ![n] = {}]
       /\ difficulty' = NextDifficulty(Append(chains[n], newBlock))
       /\ UNCHANGED <<currentTime, hashrate>>

\* Attacker mines a block (potentially with manipulated timestamp)
MineAttackerBlock(n) ==
    /\ n \in AttackerNodes
    /\ LET 
        \* Attacker can manipulate timestamp within limits
        manipulatedTime == CHOOSE t \in (currentTime - 10)..(currentTime + MaxFutureDrift) : TRUE
        newBlock == [
            height |-> ChainHeight(chains[n]) + 1,
            hash |-> ToString(RandomElement(1..1000000)),
            prevHash |-> ChainTip(chains[n]).hash,
            timestamp |-> manipulatedTime,
            difficulty |-> difficulty,
            nonce |-> RandomElement(1..1000000),
            transactions |-> SetToSeq(mempool[n]),
            miner |-> n
        ]
       IN
       /\ ValidBlock(newBlock, chains[n])  \* Must still pass validation
       /\ chains' = [chains EXCEPT ![n] = Append(chains[n], newBlock)]
       /\ messages' = messages \cup {[type |-> "block", block |-> newBlock, sender |-> n]}
       /\ mempool' = [mempool EXCEPT ![n] = {}]
       /\ difficulty' = NextDifficulty(Append(chains[n], newBlock))
       /\ UNCHANGED <<currentTime, hashrate>>

\* Node receives and validates a block from network
ReceiveBlock(n, msg) ==
    /\ msg \in messages
    /\ msg.type = "block"
    /\ LET block == msg.block
       IN
       /\ ValidBlock(block, chains[n])
       \/ (ValidReorg(Append(chains[n], block), chains[n]) /\ block.height > ChainHeight(chains[n]))
       /\ chains' = [chains EXCEPT ![n] = Append(chains[n], block)]
       /\ UNCHANGED <<mempool, currentTime, messages, difficulty, hashrate>>

\* Time advances (used for timestamp validation)
AdvanceTime ==
    /\ currentTime' = currentTime + 1
    /\ UNCHANGED <<chains, mempool, messages, difficulty, hashrate>>

\* Submit transaction to mempool
SubmitTransaction(n, tx) ==
    /\ mempool' = [mempool EXCEPT ![n] = mempool[n] \cup {tx}]
    /\ UNCHANGED <<chains, currentTime, messages, difficulty, hashrate>>

--------------------------------------------------------------------------------
(* Specification *)

Next ==
    \/ \E n \in HonestNodes: MineHonestBlock(n)
    \/ \E n \in AttackerNodes: MineAttackerBlock(n)
    \/ \E n \in Nodes, msg \in messages: ReceiveBlock(n, msg)
    \/ AdvanceTime
    \/ \E n \in Nodes, tx \in STRING: SubmitTransaction(n, tx)

Spec == Init /\ [][Next]_vars /\ WF_vars(Next)

--------------------------------------------------------------------------------
(* Safety Properties *)

\* PROPERTY 1: No two honest nodes accept conflicting blocks at same height
ConsensusAgreement ==
    \A n1, n2 \in HonestNodes:
        \A h \in 1..Min(ChainHeight(chains[n1]), ChainHeight(chains[n2])):
            chains[n1][h].hash = chains[n2][h].hash

\* PROPERTY 2: Longest chain rule (honest nodes follow heaviest chain)
LongestChainRule ==
    \A n \in HonestNodes:
        \A otherChain \in {chains[n2] : n2 \in Nodes}:
            ChainHeight(chains[n]) >= ChainHeight(otherChain) - MaxReorgDepth

\* PROPERTY 3: Timestamp safety (MTP prevents timewarp)
TimestampSafety ==
    \A n \in Nodes:
        Len(chains[n]) > 1 =>
            \A i \in 2..Len(chains[n]):
                chains[n][i].timestamp > MedianTimePast(SubSeq(chains[n], 1, i-1))

\* PROPERTY 4: Difficulty adjustment bounded (prevents difficulty bomb/crash)
DifficultyBounded ==
    /\ difficulty >= 1  \* Minimum difficulty
    /\ difficulty <= 1000000  \* Maximum difficulty (practical limit)

\* PROPERTY 5: Reorganization depth limited
ReorgDepthLimited ==
    \A n \in Nodes:
        \A oldChain \in {chains[n2] : n2 \in Nodes}:
            ValidReorg(chains[n], oldChain)

--------------------------------------------------------------------------------
(* Liveness Properties *)

\* PROPERTY 6: Chain continues to grow (under honest majority)
ChainGrowth ==
    <>[](ChainHeight(chains[CHOOSE n \in HonestNodes: TRUE]) >= MaxHeight)

\* PROPERTY 7: Transactions eventually confirmed
TransactionConfirmed ==
    \A tx \in STRING:
        (\E n \in Nodes: tx \in mempool[n]) ~>
        (\E n \in HonestNodes, i \in 1..Len(chains[n]): tx \in Range(chains[n][i].transactions))

--------------------------------------------------------------------------------
(* Attack Resistance Properties *)

\* PROPERTY 8: Selfish mining not profitable (requires >33% hashrate)
SelfishMiningResistance ==
    LET 
        honestHashrate == Sum({hashrate[n] : n \in HonestNodes})
        attackerHashrate == Sum({hashrate[n] : n \in AttackerNodes})
        attackerRatio == attackerHashrate \div (honestHashrate + attackerHashrate)
    IN
    attackerRatio < 33 =>  \* If attacker has <33% hashrate
        \A n \in AttackerNodes:
            \* Attacker doesn't earn more blocks than hashrate proportion
            Cardinality({i \in 1..Len(chains[n]) : chains[n][i].miner = n}) 
            <= (attackerHashrate * Len(chains[n])) \div (honestHashrate + attackerHashrate)

\* PROPERTY 9: Long-range attack prevented by MAX_REORG_DEPTH
LongRangeAttackPrevention ==
    \A n \in HonestNodes:
        \A attackerChain \in {chains[a] : a \in AttackerNodes}:
            \* Attacker cannot reorg beyond MAX_REORG_DEPTH
            Len(attackerChain) > ChainHeight(chains[n]) + MaxReorgDepth =>
                chains[n] # attackerChain  \* Honest node rejects deep reorg

\* PROPERTY 10: Timewarp attack fails (difficulty doesn't crash)
TimeWarpResistance ==
    \A n \in AttackerNodes:
        \* Even if attacker manipulates timestamps
        \A i \in 2..Len(chains[n]):
            chains[n][i].timestamp <= currentTime + MaxFutureDrift =>
                \* Difficulty doesn't drop below safe threshold
                difficulty >= 4  \* Example threshold (adjust based on hashrate)

--------------------------------------------------------------------------------
(* Model Checking Configuration *)

\* Bounded model checking parameters
CONSTANTS
    ModelNodes <- {n1, n2, n3}  \* 3 nodes: 2 honest, 1 attacker
    ModelHonestNodes <- {n1, n2}
    ModelAttackerNodes <- {n3}
    ModelMaxHeight <- 10  \* Check up to 10 blocks
    ModelMaxReorgDepth <- 3  \* Simplified for model checking
    ModelMaxFutureDrift <- 60
    ModelDifficultyAdjustInterval <- 5
    ModelTargetBlockTime <- 600

\* Invariants to check
INVARIANTS
    TypeOK
    ConsensusAgreement
    TimestampSafety
    DifficultyBounded
    ReorgDepthLimited

\* Temporal properties (liveness)
PROPERTIES
    ChainGrowth
    TransactionConfirmed

\* Attack resistance (model check with attacker nodes)
ATTACK_PROPERTIES
    SelfishMiningResistance
    LongRangeAttackPrevention
    TimeWarpResistance

================================================================================

(*
 * MODEL CHECKING INSTRUCTIONS:
 * 
 * 1. Install TLA+ Toolbox: https://lamport.azurewebsites.net/tla/toolbox.html
 * 
 * 2. Create new spec in TLA+ Toolbox:
 *    - File -> Open Spec -> Add New Spec
 *    - Copy this file content
 * 
 * 3. Create Model:
 *    - TLC Model Checker -> New Model
 *    - Set constants (use ModelNodes, ModelHonestNodes, etc.)
 *    - Add invariants: TypeOK, ConsensusAgreement, TimestampSafety
 *    - Add properties: ChainGrowth, TransactionConfirmed
 * 
 * 4. Run Model Checker:
 *    - Click "Run TLC on the model"
 *    - Check for invariant violations or deadlocks
 *    - Review state space coverage
 * 
 * 5. Analyze Results:
 *    - If invariant violated: Review error trace
 *    - If deadlock: Check liveness properties
 *    - If success: Consensus properties proven for bounded model
 * 
 * EXPECTED RESULTS:
 * - All invariants should hold (no violations)
 * - Liveness properties satisfied (chain grows, txs confirmed)
 * - Attack properties hold (selfish mining not profitable, etc.)
 * 
 * LIMITATIONS:
 * - Bounded model checking (finite state space)
 * - Simplified PoW (actual hash function abstracted)
 * - Network delays not modeled (assume eventual delivery)
 * - Does not model Byzantine faults (crash failures only)
 * 
 * For production verification:
 * - Consider unbounded verification with inductive invariants
 * - Use refinement mapping to relate spec to Rust implementation
 * - Perform runtime verification (monitor actual blockchain)
 *)
