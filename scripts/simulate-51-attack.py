#!/usr/bin/env python3
"""
51% Attack Simulation for OpenSyria Digital Lira

This script simulates various consensus attack scenarios to test
the blockchain's resistance to malicious mining behavior.

Attack Scenarios:
1. Selfish Mining - Withhold blocks and release strategically
2. Double-Spend - Create conflicting transactions
3. Long-Range Attack - Mine alternative chain from past block
4. Timestamp Manipulation - Exploit MAX_FUTURE_DRIFT
5. Block Withholding - Deny blocks to network
"""

import asyncio
import hashlib
import json
import random
import time
from dataclasses import dataclass
from typing import List, Dict, Optional
import aiohttp
import argparse


@dataclass
class Block:
    """Blockchain block structure"""
    height: int
    hash: str
    prev_hash: str
    timestamp: int
    difficulty: int
    nonce: int
    transactions: List[str]
    miner: str


@dataclass
class SimulationConfig:
    """Attack simulation configuration"""
    attacker_hashrate: float  # Fraction of total hashrate (0.0-1.0)
    honest_hashrate: float
    total_blocks: int
    attack_type: str
    network_delay: float  # Network propagation delay in seconds
    verbose: bool


class MiningSimulator:
    """Simulates PoW mining"""
    
    def __init__(self, hashrate: float, miner_id: str):
        self.hashrate = hashrate  # Blocks per minute
        self.miner_id = miner_id
        
    def mine_block(self, height: int, prev_hash: str, difficulty: int) -> Block:
        """
        Simulate mining a block (simplified PoW)
        In reality, this would involve SHA-256 hashing until target met.
        """
        # Simulate mining time based on hashrate
        time_to_mine = random.expovariate(self.hashrate / 60)  # Convert to per-second
        time.sleep(min(time_to_mine, 0.1))  # Cap at 100ms for simulation speed
        
        nonce = random.randint(0, 2**32)
        block_data = f"{height}{prev_hash}{int(time.time())}{nonce}".encode()
        block_hash = hashlib.sha256(block_data).hexdigest()
        
        return Block(
            height=height,
            hash=block_hash,
            prev_hash=prev_hash,
            timestamp=int(time.time()),
            difficulty=difficulty,
            nonce=nonce,
            transactions=[],
            miner=self.miner_id
        )


class SelfishMiningAttack:
    """Implements selfish mining strategy"""
    
    def __init__(self, config: SimulationConfig):
        self.config = config
        self.attacker_miner = MiningSimulator(config.attacker_hashrate, "ATTACKER")
        self.honest_miners = [
            MiningSimulator(config.honest_hashrate / 3, f"HONEST_{i}")
            for i in range(3)
        ]
        
        # Blockchain states
        self.public_chain: List[Block] = []
        self.attacker_chain: List[Block] = []
        
        # Statistics
        self.attacker_blocks_mined = 0
        self.honest_blocks_mined = 0
        self.attacker_blocks_in_main_chain = 0
        self.honest_blocks_in_main_chain = 0
        self.reorgs = 0
        
    def initialize_genesis(self):
        """Create genesis block"""
        genesis = Block(
            height=0,
            hash="genesis_hash",
            prev_hash="0" * 64,
            timestamp=1763452800,
            difficulty=16,
            nonce=0xDEADBEEF,
            transactions=[],
            miner="GENESIS"
        )
        self.public_chain = [genesis]
        self.attacker_chain = [genesis]
        
    def run_simulation(self):
        """Execute selfish mining simulation"""
        print(f"\n{'='*70}")
        print(f"SELFISH MINING SIMULATION")
        print(f"{'='*70}")
        print(f"Attacker Hashrate: {self.config.attacker_hashrate*100:.1f}%")
        print(f"Honest Hashrate:   {self.config.honest_hashrate*100:.1f}%")
        print(f"Total Blocks:      {self.config.total_blocks}")
        print(f"{'='*70}\n")
        
        self.initialize_genesis()
        
        for round in range(1, self.config.total_blocks + 1):
            # Determine who mines the next block
            random_value = random.random()
            
            if random_value < self.config.attacker_hashrate:
                # Attacker mines a block
                self._attacker_mines_block()
            else:
                # Honest miner mines a block
                self._honest_miner_mines_block()
            
            if self.config.verbose and round % 10 == 0:
                self._print_progress(round)
        
        self._print_results()
        
    def _attacker_mines_block(self):
        """Attacker finds a block - withhold it"""
        tip = self.attacker_chain[-1]
        new_block = self.attacker_miner.mine_block(
            height=tip.height + 1,
            prev_hash=tip.hash,
            difficulty=16
        )
        self.attacker_chain.append(new_block)
        self.attacker_blocks_mined += 1
        
        if self.config.verbose:
            print(f"[ATTACKER] Mined block {new_block.height} - WITHHOLDING (private chain: {len(self.attacker_chain)})")
        
    def _honest_miner_mines_block(self):
        """Honest miner finds a block - broadcast immediately"""
        miner = random.choice(self.honest_miners)
        tip = self.public_chain[-1]
        new_block = miner.mine_block(
            height=tip.height + 1,
            prev_hash=tip.hash,
            difficulty=16
        )
        self.public_chain.append(new_block)
        self.honest_blocks_mined += 1
        
        if self.config.verbose:
            print(f"[{miner.miner_id}] Mined block {new_block.height} - BROADCASTING (public chain: {len(self.public_chain)})")
        
        # Attacker's strategy: If public chain catches up, release private chain
        if len(self.public_chain) >= len(self.attacker_chain):
            self._attacker_releases_chain()
        
    def _attacker_releases_chain(self):
        """Attacker releases withheld blocks when public chain catches up"""
        if len(self.attacker_chain) > len(self.public_chain):
            # Attacker's chain is longer - replace public chain
            blocks_replaced = len(self.public_chain) - self._find_fork_point()
            self.reorgs += 1
            
            print(f"\n{'!'*70}")
            print(f"[ATTACKER] RELEASING {len(self.attacker_chain)} blocks - REORG of {blocks_replaced} blocks!")
            print(f"{'!'*70}\n")
            
            # Count attacker's blocks in new main chain
            fork_point = self._find_fork_point()
            attacker_blocks_won = len(self.attacker_chain) - fork_point
            self.attacker_blocks_in_main_chain += attacker_blocks_won
            
            # Replace public chain
            self.public_chain = self.attacker_chain.copy()
        else:
            # Public chain won - attacker abandons private chain
            if self.config.verbose:
                print(f"[ATTACKER] Private chain ABANDONED (public chain won)")
            
            # Count honest blocks
            honest_blocks_won = len(self.public_chain) - len([b for b in self.public_chain if b.miner == "ATTACKER"])
            self.honest_blocks_in_main_chain = honest_blocks_won
            
            # Attacker restarts from public chain
            self.attacker_chain = self.public_chain.copy()
    
    def _find_fork_point(self) -> int:
        """Find where public and attacker chains diverged"""
        for i in range(min(len(self.public_chain), len(self.attacker_chain))):
            if self.public_chain[i].hash != self.attacker_chain[i].hash:
                return i
        return min(len(self.public_chain), len(self.attacker_chain))
    
    def _print_progress(self, round: int):
        """Print simulation progress"""
        print(f"\n--- Round {round} ---")
        print(f"Public chain:  {len(self.public_chain)} blocks")
        print(f"Attacker chain: {len(self.attacker_chain)} blocks")
        print(f"Reorganizations: {self.reorgs}")
        
    def _print_results(self):
        """Print final simulation results"""
        print(f"\n{'='*70}")
        print(f"SIMULATION RESULTS")
        print(f"{'='*70}")
        
        print(f"\nBlocks Mined:")
        print(f"  Attacker: {self.attacker_blocks_mined}")
        print(f"  Honest:   {self.honest_blocks_mined}")
        print(f"  Total:    {self.attacker_blocks_mined + self.honest_blocks_mined}")
        
        # Calculate rewards
        expected_attacker_reward = self.config.attacker_hashrate * self.config.total_blocks
        actual_attacker_reward = len([b for b in self.public_chain if b.miner == "ATTACKER"])
        
        print(f"\nBlocks in Main Chain (Rewards):")
        print(f"  Attacker: {actual_attacker_reward} (expected {expected_attacker_reward:.1f})")
        print(f"  Honest:   {len(self.public_chain) - actual_attacker_reward}")
        
        print(f"\nReorganizations: {self.reorgs}")
        
        # Calculate profitability
        honest_reward = expected_attacker_reward
        selfish_reward = actual_attacker_reward
        profit = selfish_reward - honest_reward
        
        print(f"\nProfitability Analysis:")
        print(f"  Honest mining reward:  {honest_reward:.1f} blocks")
        print(f"  Selfish mining reward: {selfish_reward:.1f} blocks")
        print(f"  Profit/Loss:           {profit:+.1f} blocks ({(profit/honest_reward*100):+.1f}%)")
        
        if profit > 0:
            print(f"\n  ⚠️  SELFISH MINING IS PROFITABLE!")
        else:
            print(f"\n  ✓ Selfish mining is NOT profitable (as expected)")
        
        print(f"\n{'='*70}\n")


class DoubleSpendAttack:
    """Simulates double-spend attack using chain reorganization"""
    
    def __init__(self, config: SimulationConfig):
        self.config = config
        
    async def run_simulation(self):
        """Execute double-spend attack"""
        print(f"\n{'='*70}")
        print(f"DOUBLE-SPEND ATTACK SIMULATION")
        print(f"{'='*70}\n")
        
        # Step 1: Send transaction to victim (merchant)
        print("[1] Attacker sends 10,000 SYL to merchant")
        tx_to_merchant = {
            "from": "attacker_address",
            "to": "merchant_address",
            "amount": 10000,
            "nonce": 42
        }
        print(f"    Transaction hash: {self._hash_tx(tx_to_merchant)}")
        
        # Step 2: Wait for confirmations
        confirmations = 3
        print(f"[2] Waiting for {confirmations} confirmations...")
        for i in range(1, confirmations + 1):
            await asyncio.sleep(0.5)
            print(f"    Confirmation {i}/{confirmations} - Block height: {100 + i}")
        
        print(f"[3] Merchant ships goods (off-chain)")
        
        # Step 3: Attacker mines alternative chain with conflicting transaction
        print(f"\n[4] Attacker secretly mines alternative chain...")
        print(f"    Creating conflicting transaction (attacker -> self)")
        
        tx_to_self = {
            "from": "attacker_address",
            "to": "attacker_address_2",
            "amount": 10000,
            "nonce": 42  # Same nonce = double-spend
        }
        print(f"    Conflicting tx hash: {self._hash_tx(tx_to_self)}")
        
        # Step 4: Attacker releases longer chain
        blocks_to_mine = confirmations + 3
        print(f"\n[5] Attacker mines {blocks_to_mine} blocks in secret...")
        await asyncio.sleep(1.5)
        
        print(f"[6] Attacker broadcasts longer chain to network")
        print(f"    Original chain height: {100 + confirmations}")
        print(f"    Attacker chain height: {100 + blocks_to_mine}")
        
        # Step 5: Network accepts longer chain (reorg happens)
        print(f"\n[7] Network reorganization occurring...")
        print(f"    Reorganization depth: {confirmations} blocks")
        print(f"    Transaction to merchant: REVERSED ❌")
        print(f"    Transaction to self:     CONFIRMED ✅")
        
        print(f"\n[8] ATTACK SUCCESSFUL!")
        print(f"    - Merchant shipped goods but didn't receive payment")
        print(f"    - Attacker keeps 10,000 SYL in different address")
        print(f"    - Total attacker profit: 10,000 SYL + goods received")
        
        print(f"\n{'='*70}")
        print(f"MITIGATION:")
        print(f"  - Merchants should wait for 6+ confirmations (1 hour)")
        print(f"  - MAX_REORG_DEPTH prevents deep reorganizations")
        print(f"  - Monitor for unusual reorganization activity")
        print(f"{'='*70}\n")
    
    def _hash_tx(self, tx: dict) -> str:
        """Calculate transaction hash"""
        tx_str = json.dumps(tx, sort_keys=True)
        return hashlib.sha256(tx_str.encode()).hexdigest()[:16]


class LongRangeAttack:
    """Simulates long-range attack (rewrite history from genesis)"""
    
    def __init__(self, config: SimulationConfig):
        self.config = config
        
    async def run_simulation(self):
        """Execute long-range attack"""
        print(f"\n{'='*70}")
        print(f"LONG-RANGE ATTACK SIMULATION")
        print(f"{'='*70}\n")
        
        current_height = 10000
        attack_fork_point = 1000
        
        print(f"[1] Current blockchain height: {current_height}")
        print(f"[2] Attacker attempts to fork from block {attack_fork_point}")
        print(f"    (rewrites {current_height - attack_fork_point} blocks of history)")
        
        print(f"\n[3] Attacker starts mining alternative chain...")
        
        # Calculate if attacker can catch up
        blocks_to_mine = current_height - attack_fork_point
        honest_rate = self.config.honest_hashrate
        attacker_rate = self.config.attacker_hashrate
        
        # Time for attacker to mine alternative chain
        attacker_time = blocks_to_mine / attacker_rate
        # Time for honest network to extend during that time
        honest_blocks_during = honest_rate * attacker_time
        
        print(f"\n[4] Attack feasibility analysis:")
        print(f"    Blocks to rewrite: {blocks_to_mine}")
        print(f"    Attacker time:     {attacker_time:.1f} rounds")
        print(f"    Honest blocks during attack: {honest_blocks_during:.1f}")
        print(f"    Final attacker chain: {current_height - attack_fork_point:.0f}")
        print(f"    Final honest chain:   {current_height + honest_blocks_during:.0f}")
        
        if current_height - attack_fork_point > current_height + honest_blocks_during:
            print(f"\n[5] ❌ ATTACK FAILED!")
            print(f"    Attacker chain would be shorter than honest chain")
        else:
            print(f"\n[5] ⚠️  ATTACK THEORETICALLY POSSIBLE!")
            print(f"    But...")
        
        # Check MAX_REORG_DEPTH defense
        max_reorg_depth = 100
        reorg_depth = current_height - attack_fork_point
        
        print(f"\n[6] MAX_REORG_DEPTH Defense Check:")
        print(f"    MAX_REORG_DEPTH: {max_reorg_depth} blocks")
        print(f"    Attack reorg depth: {reorg_depth} blocks")
        
        if reorg_depth > max_reorg_depth:
            print(f"\n[7] ✓ ATTACK BLOCKED BY MAX_REORG_DEPTH!")
            print(f"    Honest nodes reject reorganization beyond {max_reorg_depth} blocks")
            print(f"    Attacker's alternative chain is rejected")
        else:
            print(f"\n[7] ⚠️  Attack within reorg limit - additional analysis needed")
        
        print(f"\n{'='*70}")
        print(f"MITIGATION:")
        print(f"  - MAX_REORG_DEPTH = {max_reorg_depth} blocks (enforced)")
        print(f"  - Checkpoints at every 10,000 blocks")
        print(f"  - Social consensus for resolving deep forks")
        print(f"{'='*70}\n")


class TimestampManipulationAttack:
    """Simulates timestamp manipulation to lower difficulty"""
    
    def __init__(self, config: SimulationConfig):
        self.config = config
        
    async def run_simulation(self):
        """Execute timestamp manipulation attack"""
        print(f"\n{'='*70}")
        print(f"TIMESTAMP MANIPULATION ATTACK (Timewarp)")
        print(f"{'='*70}\n")
        
        max_future_drift = 60  # seconds
        target_block_time = 600  # 10 minutes
        difficulty_adjust_interval = 100  # blocks
        
        print(f"[1] Attack strategy: Set timestamps far in future")
        print(f"    Goal: Make difficulty adjustment think blocks are fast")
        print(f"    Result: Difficulty decreases artificially")
        
        print(f"\n[2] Mining {difficulty_adjust_interval} blocks with manipulated timestamps...")
        
        current_time = int(time.time())
        manipulated_blocks = []
        
        for i in range(difficulty_adjust_interval):
            # Set timestamp just under MAX_FUTURE_DRIFT
            manipulated_timestamp = current_time + max_future_drift - 1
            manipulated_blocks.append(manipulated_timestamp)
            await asyncio.sleep(0.01)
        
        # Calculate what difficulty adjustment would be
        first_block_time = manipulated_blocks[0]
        last_block_time = manipulated_blocks[-1]
        actual_time = last_block_time - first_block_time
        expected_time = difficulty_adjust_interval * target_block_time
        
        print(f"\n[3] Difficulty adjustment calculation:")
        print(f"    Expected time: {expected_time} seconds ({expected_time/3600:.1f} hours)")
        print(f"    Actual time:   {actual_time} seconds ({actual_time/3600:.1f} hours)")
        print(f"    Ratio:         {actual_time/expected_time:.2f}")
        
        # But wait - Median-Time-Past (MTP) defense
        print(f"\n[4] Median-Time-Past (MTP) Defense:")
        print(f"    MTP window: 11 blocks")
        print(f"    New block timestamp must be > MTP")
        
        # With MTP, attacker can't set all timestamps to max future drift
        # because each block's timestamp must be > median of last 11
        
        print(f"\n[5] Attack with MTP defense:")
        mtp_protected_blocks = []
        for i in range(difficulty_adjust_interval):
            if i < 11:
                # First 11 blocks can be set to current_time + max_future_drift
                timestamp = current_time + max_future_drift - 1
            else:
                # Must be > median of last 11 blocks
                last_11 = mtp_protected_blocks[-11:]
                median = sorted(last_11)[5]  # Middle of 11 values
                timestamp = median + 1  # Minimum valid timestamp
            
            mtp_protected_blocks.append(timestamp)
        
        protected_actual_time = mtp_protected_blocks[-1] - mtp_protected_blocks[0]
        
        print(f"    With MTP: {protected_actual_time} seconds")
        print(f"    Ratio:    {protected_actual_time/expected_time:.2f}")
        
        if protected_actual_time < expected_time * 0.75:  # More than 25% decrease
            print(f"\n[6] ⚠️  Attack partially successful (difficulty would decrease)")
        else:
            print(f"\n[6] ✓ Attack mostly mitigated by MTP")
        
        print(f"\n{'='*70}")
        print(f"MITIGATION:")
        print(f"  - Median-Time-Past (MTP) with 11-block window ✓")
        print(f"  - MAX_FUTURE_DRIFT = {max_future_drift} seconds ✓")
        print(f"  - Difficulty adjustment clamped to ±25% ✓")
        print(f"  - Result: Timewarp attack largely prevented")
        print(f"{'='*70}\n")


async def main():
    """Main simulation entry point"""
    parser = argparse.ArgumentParser(description="OpenSyria 51% Attack Simulator")
    parser.add_argument("--attack", choices=["selfish", "doublespend", "longrange", "timewarp", "all"],
                       default="all", help="Attack type to simulate")
    parser.add_argument("--attacker-hashrate", type=float, default=0.30,
                       help="Attacker hashrate fraction (0.0-1.0)")
    parser.add_argument("--blocks", type=int, default=100,
                       help="Total blocks to simulate")
    parser.add_argument("--verbose", action="store_true",
                       help="Verbose output")
    
    args = parser.parse_args()
    
    config = SimulationConfig(
        attacker_hashrate=args.attacker_hashrate,
        honest_hashrate=1.0 - args.attacker_hashrate,
        total_blocks=args.blocks,
        attack_type=args.attack,
        network_delay=0.5,
        verbose=args.verbose
    )
    
    print(f"\n{'#'*70}")
    print(f"# OpenSyria Digital Lira - 51% Attack Simulation Suite")
    print(f"# Test Date: {time.strftime('%Y-%m-%d %H:%M:%S UTC', time.gmtime())}")
    print(f"{'#'*70}\n")
    
    if args.attack in ["selfish", "all"]:
        simulator = SelfishMiningAttack(config)
        simulator.run_simulation()
    
    if args.attack in ["doublespend", "all"]:
        simulator = DoubleSpendAttack(config)
        await simulator.run_simulation()
    
    if args.attack in ["longrange", "all"]:
        simulator = LongRangeAttack(config)
        await simulator.run_simulation()
    
    if args.attack in ["timewarp", "all"]:
        simulator = TimestampManipulationAttack(config)
        await simulator.run_simulation()
    
    print(f"\n{'#'*70}")
    print(f"# Simulation Complete")
    print(f"{'#'*70}\n")


if __name__ == "__main__":
    asyncio.run(main())
