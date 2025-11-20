# 🚨 OpenSyria Incident Response Playbook
## الليرة السورية الرقمية - دليل الاستجابة للحوادث

**Version:** 1.0.0  
**Last Updated:** November 19, 2025  
**Owner:** Security Operations Team  
**Classification:** CONFIDENTIAL

---

## 📋 Table of Contents

1. [Overview](#overview)
2. [Incident Classification](#incident-classification)
3. [Response Team Structure](#response-team-structure)
4. [Emergency Contacts](#emergency-contacts)
5. [Incident Response Procedures](#incident-response-procedures)
6. [Post-Incident Review](#post-incident-review)

---

## Overview

This playbook defines procedures for detecting, responding to, and recovering from security incidents affecting the OpenSyria Digital Lira blockchain.

### Objectives

- **Detect** incidents within 5 minutes of occurrence
- **Contain** critical incidents within 15 minutes
- **Resolve** incidents within 4 hours (P1) or 24 hours (P2)
- **Document** all incidents for root cause analysis

### Scope

- Mainnet blockchain infrastructure
- Testnet infrastructure (separate procedures for non-critical)
- Wallet services and APIs
- Explorer and public-facing services
- Internal development infrastructure

---

## Incident Classification

### Severity Levels

#### P0: CRITICAL - Immediate Response Required
**Response Time:** 5 minutes  
**Resolution Time:** 1 hour  
**Escalation:** Page on-call + CTO immediately

**Scenarios:**
- 🔴 **Consensus Failure**: Chain halted, blocks not being produced
- 🔴 **Mass Fund Theft**: >$100K stolen from wallets
- 🔴 **51% Attack**: Network hashrate compromised
- 🔴 **Private Key Exposure**: Master keys or validator keys leaked
- 🔴 **Total Service Outage**: All nodes unreachable (mainnet)

**Example Alert:**
```
CRITICAL: Mainnet chain halted at block 125,847
No new blocks in 60 minutes. Investigate immediately.
```

---

#### P1: HIGH - Urgent Response Required
**Response Time:** 15 minutes  
**Resolution Time:** 4 hours  
**Escalation:** Notify security team + engineering lead

**Scenarios:**
- 🟠 **Double-Spend Detected**: Successful double-spend attack confirmed
- 🟠 **Governance Takeover**: Malicious proposal passed or vote manipulation
- 🟠 **API Breach**: Unauthorized access to wallet API or admin endpoints
- 🟠 **Database Corruption**: RocksDB integrity failure
- 🟠 **Partial Outage**: 50%+ of nodes offline

**Example Alert:**
```
HIGH: Double-spend detected in transaction 0xabc123...
2 confirmations on conflicting transactions. Analyze ASAP.
```

---

#### P2: MEDIUM - Standard Response
**Response Time:** 1 hour  
**Resolution Time:** 24 hours  
**Escalation:** Assign to on-call engineer

**Scenarios:**
- 🟡 **DDoS Attack**: Network or API under sustained attack
- 🟡 **NFT Theft**: Unauthorized NFT transfers
- 🟡 **Performance Degradation**: Query latency >5 seconds
- 🟡 **Single Node Failure**: One node crashed or unreachable
- 🟡 **Suspicious Activity**: Unusual transaction patterns

**Example Alert:**
```
MEDIUM: API latency spiked to 8.5s (threshold: 1s)
Possible DDoS or database overload. Investigate.
```

---

#### P3: LOW - Informational
**Response Time:** 24 hours  
**Resolution Time:** 7 days  
**Escalation:** Standard ticket queue

**Scenarios:**
- 🔵 **Minor Bug**: Non-critical feature malfunction
- 🔵 **Disk Space Warning**: Storage 70% full (30% remaining)
- 🔵 **Test Environment Issue**: Testnet node offline
- 🔵 **Documentation Error**: Incorrect API documentation

---

## Response Team Structure

### Incident Commander (IC)
**Primary:** CTO / Security Lead  
**Backup:** Senior DevOps Engineer

**Responsibilities:**
- Declare incident severity
- Coordinate response team
- Approve emergency changes
- Communicate with stakeholders

### Technical Lead (TL)
**Primary:** Lead Blockchain Engineer  
**Backup:** Senior Rust Developer

**Responsibilities:**
- Root cause analysis
- Implement fixes
- Test solutions
- Deploy patches

### Communications Lead (CL)
**Primary:** Community Manager  
**Backup:** Marketing Director

**Responsibilities:**
- Draft public statements
- Update status page
- Notify exchanges/partners
- Monitor social media

### Security Analyst (SA)
**Primary:** Security Engineer  
**Backup:** Penetration Tester

**Responsibilities:**
- Forensic analysis
- Threat intelligence
- Attack vector identification
- Evidence preservation

---

## Emergency Contacts

### Internal Team

| Role | Name | Phone | Signal | Email |
|------|------|-------|--------|-------|
| **Incident Commander** | [TBD] | +963-XXX-XXXX | @IC_OpenSyria | ic@opensyria.org |
| **Technical Lead** | [TBD] | +963-YYY-YYYY | @TL_OpenSyria | tl@opensyria.org |
| **Security Analyst** | [TBD] | +963-ZZZ-ZZZZ | @SA_OpenSyria | security@opensyria.org |
| **CTO** | [TBD] | +963-AAA-AAAA | @CTO_OpenSyria | cto@opensyria.org |

### External Partners

| Entity | Contact | Purpose |
|--------|---------|---------|
| **Trail of Bits** | audit@trailofbits.com | Security incident escalation |
| **AWS Support** | Premium Support Ticket | Infrastructure issues |
| **Cloudflare** | Enterprise Support | DDoS mitigation |
| **Exchanges** | [Exchange contacts] | Trading halt coordination |
| **Law Enforcement** | [Local authorities] | Criminal activity reporting |

### Escalation Tree

```
P0 Incident Detected
    ↓
Alert On-Call Engineer (via PagerDuty)
    ↓ (5 min timeout)
Page Backup Engineer
    ↓ (5 min timeout)
Page CTO + Security Lead
    ↓ (5 min timeout)
Auto-escalate to ALL team members
```

---

## Incident Response Procedures

### 1. Consensus Failure (Chain Halt)

#### Detection
**Alert:** No new blocks in >30 minutes

**Automated Checks:**
```bash
# Monitor script (runs every 60 seconds)
#!/bin/bash
LAST_BLOCK_TIME=$(curl -s http://localhost:8080/api/blockchain/tip | jq .timestamp)
CURRENT_TIME=$(date +%s)
TIME_DIFF=$((CURRENT_TIME - LAST_BLOCK_TIME))

if [ $TIME_DIFF -gt 1800 ]; then  # 30 minutes
  curl -X POST "https://pagerduty.com/api/v1/incidents" \
    -H "Authorization: Token token=YOUR_API_KEY" \
    -d '{"incident":{"type":"incident","title":"CRITICAL: Mainnet chain halted"}}'
fi
```

#### Response Procedure

**Step 1: Assess Impact (0-5 minutes)**
```bash
# Check node status
ssh node1.opensyria.org "systemctl status opensyria-node"
ssh node2.opensyria.org "systemctl status opensyria-node"
ssh node3.opensyria.org "systemctl status opensyria-node"

# Check peer connectivity
curl http://node1.opensyria.org:8080/api/network/peers | jq '.peers | length'

# Check mempool size (might be full)
curl http://node1.opensyria.org:8080/api/mempool/size
```

**Step 2: Identify Root Cause (5-15 minutes)**

**Common Causes:**
- **Stuck transaction**: Invalid transaction blocking mempool
- **Consensus bug**: Nodes rejecting valid blocks
- **Network partition**: Miners isolated from nodes
- **Database corruption**: RocksDB integrity failure

**Diagnostics:**
```bash
# Check logs for errors
journalctl -u opensyria-node --since "30 minutes ago" | grep -i error

# Check block validation errors
tail -n 1000 /var/log/opensyria/blockchain.log | grep "block rejected"

# Check database integrity
rocksdb_dump --db=/data/blockchain --stats_per_interval
```

**Step 3: Containment (15-30 minutes)**

**Option A: Restart Nodes (if simple crash)**
```bash
# Restart all nodes in sequence
for node in node1 node2 node3; do
  ssh $node.opensyria.org "systemctl restart opensyria-node"
  sleep 30  # Wait for startup
done
```

**Option B: Emergency Patch (if consensus bug)**
```bash
# Deploy hotfix to all nodes
git checkout -b hotfix/consensus-halt
# ... apply fix ...
cargo build --release
ansible-playbook deploy-emergency-patch.yml
```

**Option C: Hard Fork (if database corruption)**
```bash
# Coordinate with miners to hard fork
# Create checkpoint at last valid block
LAST_VALID_BLOCK=125847
opensyria-cli create-checkpoint --height $LAST_VALID_BLOCK

# Broadcast to all nodes
curl -X POST http://localhost:8080/api/admin/set-checkpoint \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -d "{\"height\": $LAST_VALID_BLOCK}"
```

**Step 4: Verification (30-45 minutes)**
```bash
# Verify blocks resuming
watch -n 10 'curl -s http://localhost:8080/api/blockchain/tip | jq .height'

# Check all nodes in sync
for node in node1 node2 node3; do
  HEIGHT=$(curl -s http://$node.opensyria.org:8080/api/blockchain/tip | jq .height)
  echo "$node: $HEIGHT"
done
```

**Step 5: Communication (Immediate)**
```markdown
**Status Update: Mainnet Service Disruption**

We are aware of a temporary service disruption affecting block production 
on the OpenSyria mainnet. Our team is actively investigating and working 
on a resolution.

- Issue detected: 14:23 UTC
- Current status: Investigating root cause
- Impact: Transactions delayed, no fund loss
- ETA: Resolution within 1 hour

Updates: https://status.opensyria.org
```

**Step 6: Post-Incident (After resolution)**
- [ ] Document timeline of events
- [ ] Root cause analysis (RCA)
- [ ] Identify preventative measures
- [ ] Update monitoring alerts
- [ ] Schedule post-mortem meeting

---

### 2. 51% Attack Detection

#### Detection
**Indicators:**
- Large reorganizations (>6 blocks)
- Same miner winning 51%+ of blocks in 100-block window
- Rapid difficulty fluctuations
- Double-spend transactions detected

**Automated Detection:**
```python
# monitor_51_attack.py (runs every block)
def detect_51_attack(blockchain):
    recent_blocks = blockchain.get_last_blocks(100)
    
    # Check 1: Single miner dominance
    miner_distribution = {}
    for block in recent_blocks:
        miner_distribution[block.miner] = miner_distribution.get(block.miner, 0) + 1
    
    max_percentage = max(miner_distribution.values()) / len(recent_blocks)
    if max_percentage > 0.51:
        alert("CRITICAL: Single miner has 51%+ of last 100 blocks!")
    
    # Check 2: Large reorganization
    for reorg in blockchain.get_recent_reorgs():
        if reorg.depth > 6:
            alert(f"CRITICAL: {reorg.depth}-block reorganization detected!")
    
    # Check 3: Double-spend detection
    for tx in blockchain.get_recent_transactions(1000):
        if blockchain.is_double_spend(tx):
            alert(f"CRITICAL: Double-spend detected: {tx.hash}")
```

#### Response Procedure

**Step 1: Confirm Attack (0-10 minutes)**
```bash
# Check recent block miners
opensyria-cli stats --last-blocks 100 | grep "Unique miners"

# Identify attacker address
opensyria-cli find-dominant-miner --window 100

# Check for double-spends
opensyria-cli scan-double-spends --since "1 hour ago"
```

**Step 2: Emergency Actions (10-30 minutes)**

**Action A: Alert Exchanges**
```markdown
URGENT: Potential 51% attack on OpenSyria mainnet.
Recommend pausing SYL deposits/withdrawals until resolved.
Monitoring situation closely. Updates in 15 minutes.
```

**Action B: Increase Confirmation Requirements**
```bash
# Update recommended confirmations (via API and docs)
curl -X POST http://status.opensyria.org/api/update \
  -d '{"recommended_confirmations": 100, "reason": "51% attack mitigation"}'
```

**Action C: Community Alert**
```markdown
⚠️ SECURITY ALERT ⚠️

We have detected suspicious mining activity indicating a potential 51% attack.

IMMEDIATE ACTIONS:
- Do NOT accept transactions with <100 confirmations
- Exchanges: Pause deposits/withdrawals
- Users: Avoid large transactions until further notice

Status: https://status.opensyria.org
```

**Step 3: Mitigation (30 minutes - 4 hours)**

**Option 1: Wait It Out (if attacker stops)**
- Monitor block production
- Wait for normal distribution to resume
- Maintain high confirmation requirements

**Option 2: Emergency Hard Fork (if attack continues)**
```bash
# Coordinate with community on emergency hard fork
# Fork at last pre-attack block
FORK_HEIGHT=125000

# Create new chain ID to prevent replay
NEW_CHAIN_ID=963001  # Was 963 for mainnet

# Deploy updated client
git checkout -b emergency/fork-$FORK_HEIGHT
# ... update CHAIN_ID and add checkpoint ...
cargo build --release
ansible-playbook deploy-emergency-fork.yml
```

**Option 3: PoW Algorithm Change (nuclear option)**
```rust
// Switch from SHA-256 to different algorithm
// Requires coordinated hard fork
pub fn calculate_block_hash_v2(block: &Block) -> Hash {
    // Use SHA-3 or Blake3 instead of SHA-256
    blake3::hash(block.serialize())
}
```

**Step 4: Recovery (4-24 hours)**
- Monitor attacker activity (do they stop?)
- Coordinate with mining pools (increase honest hashrate)
- Consider temporary PoS checkpoints (community vote)

---

### 3. Private Key Exposure

#### Detection
**Sources:**
- GitHub secret scanning alerts
- Security researcher disclosure
- Unexpected wallet drains
- Leaked credentials in logs

**Immediate Verification:**
```bash
# Check if key is active
opensyria-cli check-key-usage --public-key $EXPOSED_KEY

# Check recent transactions from exposed address
opensyria-cli get-transactions --address $ADDRESS --since "24 hours ago"
```

#### Response Procedure

**Step 1: Freeze Affected Accounts (0-5 minutes)**
```bash
# If it's a multisig admin key, remove from multisig
opensyria-cli governance execute-emergency-proposal \
  --action "remove_signer" \
  --signer $EXPOSED_KEY \
  --require-votes 3  # Emergency multisig

# If it's a user wallet, attempt recovery
opensyria-cli emergency-freeze --address $ADDRESS \
  --admin-signature $ADMIN_SIG
```

**Step 2: Revoke Compromised Keys (5-15 minutes)**
```bash
# Rotate all related keys
opensyria-cli rotate-validator-keys --old-key $EXPOSED_KEY --new-key $NEW_KEY

# Update all services using the key
ansible-playbook rotate-api-keys.yml --extra-vars "old_key=$EXPOSED_KEY"

# Revoke from any OAuth/JWT systems
curl -X POST https://auth.opensyria.org/revoke \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -d "{\"key_id\": \"$EXPOSED_KEY\"}"
```

**Step 3: Assess Damage (15-60 minutes)**
```bash
# Check if funds were stolen
opensyria-cli audit-balance-changes \
  --address $ADDRESS \
  --since "time of exposure"

# Check if any malicious transactions signed
opensyria-cli list-signed-transactions \
  --signer $EXPOSED_KEY \
  --since "time of exposure"

# Check if key used for governance votes
opensyria-cli governance list-votes \
  --voter $ADDRESS \
  --since "time of exposure"
```

**Step 4: Public Disclosure (if necessary)**
```markdown
**Security Incident: Key Exposure**

A validator signing key was inadvertently exposed in [source].
The key has been immediately revoked and rotated.

- Exposure time: [timestamp]
- Revocation time: [timestamp]
- Funds at risk: None (key had limited permissions)
- User action required: None

Full post-mortem will be published within 72 hours.
```

---

### 4. Database Corruption

#### Detection
**Symptoms:**
- RocksDB integrity check failures
- Merkle root mismatches
- Block hash inconsistencies
- UTXO index out of sync

**Automated Check:**
```bash
# Hourly integrity check (cron)
#!/bin/bash
opensyria-cli verify-database --full > /tmp/db_check.log 2>&1

if grep -q "CORRUPTION" /tmp/db_check.log; then
  curl -X POST https://pagerduty.com/api/v1/incidents \
    -d '{"incident":{"type":"incident","title":"CRITICAL: Database corruption detected"}}'
fi
```

#### Response Procedure

**Step 1: Stop Node (Prevent Further Corruption)**
```bash
systemctl stop opensyria-node

# Create snapshot of corrupted state (for forensics)
cp -r /data/blockchain /backup/corrupted-$(date +%s)
```

**Step 2: Attempt Repair**
```bash
# Option A: RocksDB repair utility
rocksdb_dump --db=/data/blockchain --repair

# Option B: Re-index from blocks
opensyria-cli reindex --data-dir /data/blockchain

# Option C: Restore from backup
rsync -avz backup.opensyria.org::blockchain-backup /data/blockchain/
```

**Step 3: Verify Integrity**
```bash
# Full verification before restarting
opensyria-cli verify-database --full --verbose

# Check specific components
opensyria-cli verify-merkle-roots
opensyria-cli verify-utxo-index
opensyria-cli verify-block-hashes
```

**Step 4: Restart and Monitor**
```bash
systemctl start opensyria-node

# Monitor sync progress
watch -n 5 'journalctl -u opensyria-node -n 50'

# Verify catching up
curl http://localhost:8080/api/blockchain/sync-status
```

---

### 5. DDoS Attack

#### Detection
**Indicators:**
- API latency >5 seconds (normal: <100ms)
- Connection count >10,000 (normal: <500)
- Traffic >10 Gbps (normal: <100 Mbps)
- Error rate >50% (normal: <1%)

**Cloudflare Dashboard:**
- Check requests/sec (spike to >100K/sec)
- Check unique IPs (legitimate vs bots)
- Check geographic distribution

#### Response Procedure

**Step 1: Enable DDoS Protection (0-5 minutes)**
```bash
# Cloudflare: Enable "I'm Under Attack" mode
curl -X PATCH "https://api.cloudflare.com/client/v4/zones/$ZONE_ID/settings/security_level" \
  -H "X-Auth-Email: admin@opensyria.org" \
  -H "X-Auth-Key: $CLOUDFLARE_API_KEY" \
  -d '{"value":"under_attack"}'

# AWS: Enable Shield Advanced
aws shield create-protection \
  --name opensyria-ddos-protection \
  --resource-arn $LOAD_BALANCER_ARN
```

**Step 2: Rate Limiting (5-10 minutes)**
```nginx
# nginx rate limiting
limit_req_zone $binary_remote_addr zone=api_limit:10m rate=10r/s;
limit_req_zone $binary_remote_addr zone=explorer_limit:10m rate=100r/s;

server {
    location /api/ {
        limit_req zone=api_limit burst=20 nodelay;
        limit_req_status 429;
    }
}
```

**Step 3: Block Malicious IPs (10-30 minutes)**
```bash
# Analyze attack sources
tail -n 100000 /var/log/nginx/access.log | \
  awk '{print $1}' | sort | uniq -c | sort -rn | head -100

# Block top offending IPs
for IP in $(cat malicious_ips.txt); do
  iptables -A INPUT -s $IP -j DROP
done

# Cloudflare firewall rule
curl -X POST "https://api.cloudflare.com/client/v4/zones/$ZONE_ID/firewall/rules" \
  -d '{
    "filter": {"expression": "ip.src in {1.2.3.4 5.6.7.8}"},
    "action": "block"
  }'
```

**Step 4: Scale Infrastructure (if legitimate traffic spike)**
```bash
# AWS Auto Scaling
aws autoscaling set-desired-capacity \
  --auto-scaling-group-name opensyria-nodes \
  --desired-capacity 10  # Scale from 3 to 10 nodes

# Kubernetes (if applicable)
kubectl scale deployment opensyria-api --replicas=20
```

---

## Post-Incident Review

### Post-Mortem Template

**Incident:** [Brief description]  
**Severity:** [P0/P1/P2/P3]  
**Date:** [YYYY-MM-DD]  
**Duration:** [Start time - End time]  
**Incident Commander:** [Name]

#### Timeline
```
14:23 UTC - Incident detected (automated alert)
14:25 UTC - On-call engineer paged
14:28 UTC - Incident commander joined
14:35 UTC - Root cause identified
14:50 UTC - Fix deployed to canary nodes
15:10 UTC - Fix validated, rolling out to all nodes
15:45 UTC - Incident resolved
```

#### Root Cause
[Detailed technical explanation of what happened]

#### Impact
- **Users Affected:** 12,500 (~5% of active wallets)
- **Downtime:** 82 minutes (mainnet block production)
- **Financial Impact:** $0 (no funds lost)
- **Reputational Impact:** Moderate (trending on Twitter)

#### What Went Well
- ✅ Automated detection within 2 minutes
- ✅ Fast escalation to incident commander
- ✅ Clear communication with users
- ✅ No data loss or fund theft

#### What Went Wrong
- ❌ Insufficient monitoring for specific failure mode
- ❌ Runbook outdated (referenced old server names)
- ❌ Slow deployment (manual SSH to each node)

#### Action Items
1. **Add monitoring** for [specific metric] - Owner: [Name] - Due: [Date]
2. **Update runbooks** to reflect current infrastructure - Owner: [Name] - Due: [Date]
3. **Automate deployment** with Ansible playbook - Owner: [Name] - Due: [Date]
4. **Conduct fire drill** to practice this scenario - Owner: [Name] - Due: [Date]

---

## Appendix: Useful Commands

### Quick Diagnostics
```bash
# Check node health
curl http://localhost:8080/health

# Get blockchain tip
curl http://localhost:8080/api/blockchain/tip

# Check peer count
curl http://localhost:8080/api/network/peers | jq '.peers | length'

# Check mempool size
curl http://localhost:8080/api/mempool/size

# Get system metrics
top -bn1 | head -20
df -h
free -h
```

### Emergency Actions
```bash
# Restart node
systemctl restart opensyria-node

# Force database repair
opensyria-cli repair-database --force

# Emergency checkpoint
opensyria-cli create-checkpoint --height [HEIGHT] --admin-key [KEY]

# Pause governance
opensyria-cli governance emergency-pause --admin-signature [SIG]
```

---

**Document Owner:** Security Operations Team  
**Review Schedule:** Quarterly  
**Next Review:** February 19, 2026

*"Hope for the best, prepare for the worst, respond with speed."*  
*"نأمل الأفضل، نستعد للأسوأ، نستجيب بسرعة"*
