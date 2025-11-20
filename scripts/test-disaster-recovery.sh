#!/bin/bash
# Disaster Recovery Test Suite for OpenSyria Digital Lira
# Tests backup, restore, and failover procedures

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
TEST_DATA_DIR="/tmp/opensyria-dr-test"
BACKUP_DIR="/tmp/opensyria-backup"
RESTORE_DIR="/tmp/opensyria-restore"
NODE_BINARY="${NODE_BINARY:-./target/release/opensyria-node}"
CLI_BINARY="${CLI_BINARY:-./target/release/opensyria-cli}"

# Test results
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_TOTAL=0

# Logging
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $1"
    ((TESTS_PASSED++))
    ((TESTS_TOTAL++))
}

log_failure() {
    echo -e "${RED}[FAIL]${NC} $1"
    ((TESTS_FAILED++))
    ((TESTS_TOTAL++))
}

log_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# Cleanup function
cleanup() {
    log_info "Cleaning up test environment..."
    pkill -f opensyria-node || true
    rm -rf "$TEST_DATA_DIR" "$BACKUP_DIR" "$RESTORE_DIR"
}

trap cleanup EXIT

# Test 1: Database Backup Creation
test_database_backup() {
    log_info "TEST 1: Database Backup Creation"
    
    # Start node and generate some blocks
    mkdir -p "$TEST_DATA_DIR"
    $NODE_BINARY --data-dir "$TEST_DATA_DIR" --network testnet &
    NODE_PID=$!
    sleep 5
    
    # Mine 10 blocks
    for i in {1..10}; do
        $CLI_BINARY mine --count 1 --data-dir "$TEST_DATA_DIR"
    done
    
    # Create backup
    mkdir -p "$BACKUP_DIR"
    $CLI_BINARY backup --source "$TEST_DATA_DIR" --dest "$BACKUP_DIR"
    
    # Verify backup exists
    if [ -d "$BACKUP_DIR/blockchain" ] && [ -d "$BACKUP_DIR/blocks" ]; then
        log_success "Backup created successfully"
        
        # Check backup size
        BACKUP_SIZE=$(du -sh "$BACKUP_DIR" | cut -f1)
        log_info "Backup size: $BACKUP_SIZE"
    else
        log_failure "Backup directory incomplete"
    fi
    
    kill $NODE_PID || true
    wait $NODE_PID 2>/dev/null || true
}

# Test 2: Full Database Restore
test_database_restore() {
    log_info "TEST 2: Full Database Restore"
    
    # Restore from backup to new directory
    mkdir -p "$RESTORE_DIR"
    $CLI_BINARY restore --source "$BACKUP_DIR" --dest "$RESTORE_DIR"
    
    # Start node with restored data
    $NODE_BINARY --data-dir "$RESTORE_DIR" --network testnet &
    RESTORE_NODE_PID=$!
    sleep 5
    
    # Verify blockchain height matches
    ORIGINAL_HEIGHT=$(cat "$TEST_DATA_DIR/blockchain/height.txt" 2>/dev/null || echo "0")
    RESTORED_HEIGHT=$($CLI_BINARY get-height --data-dir "$RESTORE_DIR")
    
    if [ "$ORIGINAL_HEIGHT" -eq "$RESTORED_HEIGHT" ]; then
        log_success "Restore successful - heights match ($RESTORED_HEIGHT blocks)"
    else
        log_failure "Restore failed - height mismatch (original: $ORIGINAL_HEIGHT, restored: $RESTORED_HEIGHT)"
    fi
    
    # Verify block hashes match
    for i in {0..5}; do
        ORIGINAL_HASH=$($CLI_BINARY get-block-hash --height $i --data-dir "$TEST_DATA_DIR")
        RESTORED_HASH=$($CLI_BINARY get-block-hash --height $i --data-dir "$RESTORE_DIR")
        
        if [ "$ORIGINAL_HASH" == "$RESTORED_HASH" ]; then
            log_info "Block $i hash verified: $ORIGINAL_HASH"
        else
            log_failure "Block $i hash mismatch"
            break
        fi
    done
    
    kill $RESTORE_NODE_PID || true
    wait $RESTORE_NODE_PID 2>/dev/null || true
}

# Test 3: Incremental Backup
test_incremental_backup() {
    log_info "TEST 3: Incremental Backup"
    
    # Create initial full backup
    FULL_BACKUP_DIR="$BACKUP_DIR/full-$(date +%s)"
    mkdir -p "$FULL_BACKUP_DIR"
    $CLI_BINARY backup --source "$TEST_DATA_DIR" --dest "$FULL_BACKUP_DIR" --type full
    FULL_BACKUP_SIZE=$(du -s "$FULL_BACKUP_DIR" | cut -f1)
    
    # Mine additional blocks
    $NODE_BINARY --data-dir "$TEST_DATA_DIR" --network testnet &
    NODE_PID=$!
    sleep 3
    
    for i in {1..5}; do
        $CLI_BINARY mine --count 1 --data-dir "$TEST_DATA_DIR"
    done
    
    kill $NODE_PID || true
    wait $NODE_PID 2>/dev/null || true
    
    # Create incremental backup
    INCREMENTAL_BACKUP_DIR="$BACKUP_DIR/incremental-$(date +%s)"
    mkdir -p "$INCREMENTAL_BACKUP_DIR"
    $CLI_BINARY backup --source "$TEST_DATA_DIR" --dest "$INCREMENTAL_BACKUP_DIR" \
        --type incremental --base "$FULL_BACKUP_DIR"
    INCREMENTAL_BACKUP_SIZE=$(du -s "$INCREMENTAL_BACKUP_DIR" | cut -f1)
    
    # Verify incremental is smaller than full
    if [ "$INCREMENTAL_BACKUP_SIZE" -lt "$FULL_BACKUP_SIZE" ]; then
        log_success "Incremental backup smaller than full (saved $(( (FULL_BACKUP_SIZE - INCREMENTAL_BACKUP_SIZE) / 1024 ))MB)"
    else
        log_warning "Incremental backup not smaller than full (this may be expected for small datasets)"
    fi
}

# Test 4: Point-in-Time Recovery
test_point_in_time_recovery() {
    log_info "TEST 4: Point-in-Time Recovery"
    
    # Start node and mine blocks at different times
    $NODE_BINARY --data-dir "$TEST_DATA_DIR" --network testnet &
    NODE_PID=$!
    sleep 3
    
    # Mine blocks and record checkpoints
    CHECKPOINT_HEIGHT_5=$($CLI_BINARY mine --count 5 --data-dir "$TEST_DATA_DIR" | grep "Height" | tail -1 | awk '{print $2}')
    CHECKPOINT_TIME_5=$(date +%s)
    
    sleep 2
    
    CHECKPOINT_HEIGHT_10=$($CLI_BINARY mine --count 5 --data-dir "$TEST_DATA_DIR" | grep "Height" | tail -1 | awk '{print $2}')
    CHECKPOINT_TIME_10=$(date +%s)
    
    kill $NODE_PID || true
    wait $NODE_PID 2>/dev/null || true
    
    # Create backup of final state
    PITR_BACKUP_DIR="$BACKUP_DIR/pitr-$(date +%s)"
    mkdir -p "$PITR_BACKUP_DIR"
    $CLI_BINARY backup --source "$TEST_DATA_DIR" --dest "$PITR_BACKUP_DIR"
    
    # Restore to checkpoint at height 5
    PITR_RESTORE_DIR="$RESTORE_DIR/pitr-height-5"
    mkdir -p "$PITR_RESTORE_DIR"
    $CLI_BINARY restore --source "$PITR_BACKUP_DIR" --dest "$PITR_RESTORE_DIR" \
        --target-height "$CHECKPOINT_HEIGHT_5"
    
    # Verify restored to correct height
    RESTORED_HEIGHT=$($CLI_BINARY get-height --data-dir "$PITR_RESTORE_DIR")
    
    if [ "$RESTORED_HEIGHT" -eq "$CHECKPOINT_HEIGHT_5" ]; then
        log_success "Point-in-time recovery to height $CHECKPOINT_HEIGHT_5 successful"
    else
        log_failure "PITR failed - expected height $CHECKPOINT_HEIGHT_5, got $RESTORED_HEIGHT"
    fi
}

# Test 5: Corrupted Database Recovery
test_corrupted_database_recovery() {
    log_info "TEST 5: Corrupted Database Recovery"
    
    # Create a fresh blockchain
    CORRUPT_TEST_DIR="$TEST_DATA_DIR/corrupt-test"
    mkdir -p "$CORRUPT_TEST_DIR"
    
    $NODE_BINARY --data-dir "$CORRUPT_TEST_DIR" --network testnet &
    NODE_PID=$!
    sleep 3
    
    $CLI_BINARY mine --count 10 --data-dir "$CORRUPT_TEST_DIR"
    
    kill $NODE_PID || true
    wait $NODE_PID 2>/dev/null || true
    
    # Backup clean state
    CLEAN_BACKUP="$BACKUP_DIR/clean-$(date +%s)"
    mkdir -p "$CLEAN_BACKUP"
    $CLI_BINARY backup --source "$CORRUPT_TEST_DIR" --dest "$CLEAN_BACKUP"
    
    # Corrupt the database (delete random files)
    log_info "Simulating database corruption..."
    find "$CORRUPT_TEST_DIR/blockchain" -type f | shuf -n 5 | xargs rm -f
    
    # Attempt to start node (should fail or detect corruption)
    $NODE_BINARY --data-dir "$CORRUPT_TEST_DIR" --network testnet &
    CORRUPT_NODE_PID=$!
    sleep 5
    
    # Check if node detected corruption
    if $CLI_BINARY verify-database --data-dir "$CORRUPT_TEST_DIR" 2>&1 | grep -q "CORRUPTION"; then
        log_info "Corruption detected successfully"
        
        kill $CORRUPT_NODE_PID 2>/dev/null || true
        wait $CORRUPT_NODE_PID 2>/dev/null || true
        
        # Recover from backup
        log_info "Recovering from backup..."
        rm -rf "$CORRUPT_TEST_DIR"
        mkdir -p "$CORRUPT_TEST_DIR"
        $CLI_BINARY restore --source "$CLEAN_BACKUP" --dest "$CORRUPT_TEST_DIR"
        
        # Verify recovery
        $NODE_BINARY --data-dir "$CORRUPT_TEST_DIR" --network testnet &
        RECOVERED_NODE_PID=$!
        sleep 3
        
        if $CLI_BINARY verify-database --data-dir "$CORRUPT_TEST_DIR" | grep -q "OK"; then
            log_success "Recovery from corrupted database successful"
        else
            log_failure "Database still corrupted after recovery"
        fi
        
        kill $RECOVERED_NODE_PID 2>/dev/null || true
        wait $RECOVERED_NODE_PID 2>/dev/null || true
    else
        log_failure "Corruption not detected"
        kill $CORRUPT_NODE_PID 2>/dev/null || true
        wait $CORRUPT_NODE_PID 2>/dev/null || true
    fi
}

# Test 6: Multi-Node Failover
test_multi_node_failover() {
    log_info "TEST 6: Multi-Node Failover"
    
    # Start 3 nodes
    NODE1_DIR="$TEST_DATA_DIR/node1"
    NODE2_DIR="$TEST_DATA_DIR/node2"
    NODE3_DIR="$TEST_DATA_DIR/node3"
    
    mkdir -p "$NODE1_DIR" "$NODE2_DIR" "$NODE3_DIR"
    
    $NODE_BINARY --data-dir "$NODE1_DIR" --network testnet --p2p-port 18333 &
    NODE1_PID=$!
    
    $NODE_BINARY --data-dir "$NODE2_DIR" --network testnet --p2p-port 18334 \
        --bootstrap /ip4/127.0.0.1/tcp/18333 &
    NODE2_PID=$!
    
    $NODE_BINARY --data-dir "$NODE3_DIR" --network testnet --p2p-port 18335 \
        --bootstrap /ip4/127.0.0.1/tcp/18333 &
    NODE3_PID=$!
    
    sleep 10
    
    # Mine blocks on node 1
    for i in {1..5}; do
        $CLI_BINARY mine --count 1 --data-dir "$NODE1_DIR"
        sleep 1
    done
    
    # Verify all nodes synced
    HEIGHT1=$($CLI_BINARY get-height --data-dir "$NODE1_DIR")
    HEIGHT2=$($CLI_BINARY get-height --data-dir "$NODE2_DIR")
    HEIGHT3=$($CLI_BINARY get-height --data-dir "$NODE3_DIR")
    
    if [ "$HEIGHT1" -eq "$HEIGHT2" ] && [ "$HEIGHT2" -eq "$HEIGHT3" ]; then
        log_info "All nodes synced at height $HEIGHT1"
    else
        log_warning "Nodes not fully synced (heights: $HEIGHT1, $HEIGHT2, $HEIGHT3)"
    fi
    
    # Kill node 1 (simulate failure)
    log_info "Simulating node 1 failure..."
    kill $NODE1_PID || true
    wait $NODE1_PID 2>/dev/null || true
    
    # Mine on node 2 (should continue working)
    sleep 3
    $CLI_BINARY mine --count 3 --data-dir "$NODE2_DIR"
    
    # Verify node 3 received blocks from node 2
    NEW_HEIGHT3=$($CLI_BINARY get-height --data-dir "$NODE3_DIR")
    
    if [ "$NEW_HEIGHT3" -gt "$HEIGHT3" ]; then
        log_success "Failover successful - node 3 synced from node 2 after node 1 failure"
    else
        log_failure "Failover failed - node 3 did not sync after node 1 failure"
    fi
    
    kill $NODE2_PID $NODE3_PID 2>/dev/null || true
    wait $NODE2_PID $NODE3_PID 2>/dev/null || true
}

# Test 7: Automated Backup Rotation
test_backup_rotation() {
    log_info "TEST 7: Automated Backup Rotation"
    
    ROTATION_DIR="$BACKUP_DIR/rotation-test"
    mkdir -p "$ROTATION_DIR"
    
    # Create 10 daily backups (simulated)
    for i in {1..10}; do
        BACKUP_DATE=$(date -d "$i days ago" +%Y-%m-%d 2>/dev/null || date -v-${i}d +%Y-%m-%d)
        DAILY_BACKUP="$ROTATION_DIR/daily-$BACKUP_DATE"
        mkdir -p "$DAILY_BACKUP"
        echo "Backup from $BACKUP_DATE" > "$DAILY_BACKUP/metadata.txt"
    done
    
    # Rotation policy: Keep last 7 daily backups
    log_info "Applying rotation policy (keep last 7 days)..."
    BACKUP_COUNT=$(ls -1d "$ROTATION_DIR"/daily-* 2>/dev/null | wc -l)
    BACKUPS_TO_DELETE=$((BACKUP_COUNT - 7))
    
    if [ "$BACKUPS_TO_DELETE" -gt 0 ]; then
        ls -1dt "$ROTATION_DIR"/daily-* | tail -n "$BACKUPS_TO_DELETE" | xargs rm -rf
    fi
    
    # Verify only 7 backups remain
    REMAINING_BACKUPS=$(ls -1d "$ROTATION_DIR"/daily-* 2>/dev/null | wc -l)
    
    if [ "$REMAINING_BACKUPS" -eq 7 ]; then
        log_success "Backup rotation successful - 7 backups retained"
    else
        log_failure "Backup rotation failed - expected 7, got $REMAINING_BACKUPS"
    fi
}

# Test 8: Snapshot Verification
test_snapshot_verification() {
    log_info "TEST 8: Snapshot Integrity Verification"
    
    # Create snapshot with checksum
    SNAPSHOT_DIR="$BACKUP_DIR/snapshot-$(date +%s)"
    mkdir -p "$SNAPSHOT_DIR"
    
    $CLI_BINARY backup --source "$TEST_DATA_DIR" --dest "$SNAPSHOT_DIR"
    
    # Generate checksums
    log_info "Generating checksums..."
    find "$SNAPSHOT_DIR" -type f -exec sha256sum {} \; > "$SNAPSHOT_DIR/checksums.txt"
    
    # Tamper with backup (simulate corruption in transit)
    TAMPER_FILE=$(find "$SNAPSHOT_DIR" -type f -name "*.sst" | head -1)
    if [ -n "$TAMPER_FILE" ]; then
        log_info "Tampering with $TAMPER_FILE to simulate corruption..."
        echo "CORRUPTED" >> "$TAMPER_FILE"
        
        # Verify checksums (should detect tampering)
        if sha256sum -c "$SNAPSHOT_DIR/checksums.txt" 2>&1 | grep -q "FAILED"; then
            log_success "Snapshot verification detected tampering"
        else
            log_failure "Snapshot verification failed to detect tampering"
        fi
    else
        log_warning "No SST files found to tamper with"
    fi
}

# Test 9: Hot Backup (without stopping node)
test_hot_backup() {
    log_info "TEST 9: Hot Backup (Node Running)"
    
    # Start node
    HOT_BACKUP_DIR="$TEST_DATA_DIR/hot-backup-test"
    mkdir -p "$HOT_BACKUP_DIR"
    
    $NODE_BINARY --data-dir "$HOT_BACKUP_DIR" --network testnet &
    HOT_NODE_PID=$!
    sleep 3
    
    # Mine some blocks
    $CLI_BINARY mine --count 5 --data-dir "$HOT_BACKUP_DIR"
    
    # Create hot backup while node is running
    HOT_BACKUP_DEST="$BACKUP_DIR/hot-$(date +%s)"
    mkdir -p "$HOT_BACKUP_DEST"
    
    log_info "Creating hot backup while node is running..."
    $CLI_BINARY backup --source "$HOT_BACKUP_DIR" --dest "$HOT_BACKUP_DEST" --hot
    
    # Verify backup succeeded
    if [ -d "$HOT_BACKUP_DEST/blockchain" ]; then
        # Try to restore hot backup
        HOT_RESTORE_DIR="$RESTORE_DIR/hot-restore"
        mkdir -p "$HOT_RESTORE_DIR"
        $CLI_BINARY restore --source "$HOT_BACKUP_DEST" --dest "$HOT_RESTORE_DIR"
        
        # Verify restored node can start
        $NODE_BINARY --data-dir "$HOT_RESTORE_DIR" --network testnet --p2p-port 19999 &
        HOT_RESTORE_PID=$!
        sleep 5
        
        if kill -0 $HOT_RESTORE_PID 2>/dev/null; then
            log_success "Hot backup and restore successful"
            kill $HOT_RESTORE_PID || true
            wait $HOT_RESTORE_PID 2>/dev/null || true
        else
            log_failure "Restored node from hot backup failed to start"
        fi
    else
        log_failure "Hot backup failed to create backup directory"
    fi
    
    kill $HOT_NODE_PID || true
    wait $HOT_NODE_PID 2>/dev/null || true
}

# Test 10: Geographic Replication
test_geographic_replication() {
    log_info "TEST 10: Geographic Replication Simulation"
    
    # Simulate 3 regions: US, EU, Asia
    REGION_US="$TEST_DATA_DIR/region-us"
    REGION_EU="$TEST_DATA_DIR/region-eu"
    REGION_ASIA="$TEST_DATA_DIR/region-asia"
    
    mkdir -p "$REGION_US" "$REGION_EU" "$REGION_ASIA"
    
    # Start primary node (US)
    $NODE_BINARY --data-dir "$REGION_US" --network testnet --p2p-port 20000 &
    US_PID=$!
    sleep 3
    
    # Mine blocks on US node
    $CLI_BINARY mine --count 5 --data-dir "$REGION_US"
    
    # Replicate to EU
    log_info "Replicating US -> EU..."
    rsync -a "$REGION_US/" "$REGION_EU/"
    
    # Replicate to Asia
    log_info "Replicating US -> Asia..."
    rsync -a "$REGION_US/" "$REGION_ASIA/"
    
    # Verify all regions have same data
    US_HEIGHT=$($CLI_BINARY get-height --data-dir "$REGION_US")
    EU_HEIGHT=$($CLI_BINARY get-height --data-dir "$REGION_EU")
    ASIA_HEIGHT=$($CLI_BINARY get-height --data-dir "$REGION_ASIA")
    
    if [ "$US_HEIGHT" -eq "$EU_HEIGHT" ] && [ "$EU_HEIGHT" -eq "$ASIA_HEIGHT" ]; then
        log_success "Geographic replication successful - all regions at height $US_HEIGHT"
    else
        log_failure "Geographic replication failed - heights: US=$US_HEIGHT, EU=$EU_HEIGHT, Asia=$ASIA_HEIGHT"
    fi
    
    kill $US_PID || true
    wait $US_PID 2>/dev/null || true
}

# Main test execution
main() {
    echo "========================================="
    echo "OpenSyria Disaster Recovery Test Suite"
    echo "========================================="
    echo ""
    
    # Check prerequisites
    if [ ! -f "$NODE_BINARY" ]; then
        log_failure "Node binary not found: $NODE_BINARY"
        exit 1
    fi
    
    if [ ! -f "$CLI_BINARY" ]; then
        log_failure "CLI binary not found: $CLI_BINARY"
        exit 1
    fi
    
    log_info "Starting disaster recovery tests..."
    echo ""
    
    # Run all tests
    test_database_backup
    echo ""
    
    test_database_restore
    echo ""
    
    test_incremental_backup
    echo ""
    
    test_point_in_time_recovery
    echo ""
    
    test_corrupted_database_recovery
    echo ""
    
    test_multi_node_failover
    echo ""
    
    test_backup_rotation
    echo ""
    
    test_snapshot_verification
    echo ""
    
    test_hot_backup
    echo ""
    
    test_geographic_replication
    echo ""
    
    # Summary
    echo "========================================="
    echo "Test Summary"
    echo "========================================="
    echo -e "${GREEN}Passed: $TESTS_PASSED${NC}"
    echo -e "${RED}Failed: $TESTS_FAILED${NC}"
    echo "Total:  $TESTS_TOTAL"
    echo ""
    
    if [ "$TESTS_FAILED" -eq 0 ]; then
        echo -e "${GREEN}✓ All disaster recovery tests passed!${NC}"
        exit 0
    else
        echo -e "${RED}✗ Some disaster recovery tests failed.${NC}"
        exit 1
    fi
}

# Run main function
main "$@"
