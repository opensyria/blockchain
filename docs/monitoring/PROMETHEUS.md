# OpenSyria Prometheus Metrics & Monitoring

## Overview

The OpenSyria blockchain exposes comprehensive Prometheus metrics for production monitoring, alerting, and performance analysis via Grafana dashboards.

## Metrics Endpoint

By default, metrics are exposed at:
```
http://localhost:9615/metrics
```

Health check endpoint:
```
http://localhost:9615/health
```

## Key Metrics

### Blockchain Metrics
- `opensyria_chain_height` - Current blockchain height
- `opensyria_total_supply_syl` - Total SYL in circulation
- `opensyria_block_time_seconds` - Time between blocks
- `opensyria_difficulty` - Mining difficulty
- `opensyria_blocks_mined_total{status}` - Blocks mined (accepted/orphaned)

### Network Metrics
- `opensyria_peer_count` - Total connected peers
- `opensyria_inbound_peers` - Inbound connections
- `opensyria_outbound_peers` - Outbound connections  
- `opensyria_network_rx_bytes_total{message_type}` - Bytes received
- `opensyria_network_tx_bytes_total{message_type}` - Bytes transmitted

### Mempool Metrics
- `opensyria_mempool_size` - Pending transactions
- `opensyria_mempool_bytes` - Mempool size in bytes
- `opensyria_mempool_accepted_total{tx_type}` - Transactions accepted
- `opensyria_mempool_rejected_total{reason}` - Transactions rejected

### Mining Metrics
- `opensyria_hashrate` - Estimated network hashrate
- `opensyria_blocks_mined_total` - Blocks mined by this node

### Storage Metrics
- `opensyria_db_size_bytes{db_name}` - Database sizes
- `opensyria_db_cache_hits_total{db_name}` - Cache hit rate
- `opensyria_db_cache_misses_total{db_name}` - Cache miss rate

### Performance Metrics
- `opensyria_block_validation_seconds{result}` - Block validation time histogram
- `opensyria_tx_validation_seconds{result}` - Transaction validation time histogram
- `opensyria_state_query_seconds{operation}` - State query latency histogram

### Governance Metrics
- `opensyria_active_proposals` - Active governance proposals
- `opensyria_governance_votes_total{proposal_id,vote_type}` - Votes cast

### Sync Metrics
- `opensyria_sync_progress_percent` - Sync progress (0-100%)
- `opensyria_blocks_behind` - Blocks behind network tip

### System Metrics
- `opensyria_node_uptime_seconds` - Node uptime

## Prometheus Configuration

Add this to your `prometheus.yml`:

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'opensyria'
    static_configs:
      - targets: ['localhost:9615']
        labels:
          network: 'mainnet'
          instance: 'node-1'
```

## Grafana Dashboard

Import the dashboard from `docs/monitoring/opensyria-dashboard.json` or create panels for:

### Chain Overview Panel
- Chain height (gauge)
- Block time (graph)
- Total supply (gauge)
- Difficulty (graph)

### Network Panel
- Peer count (graph)
- Inbound vs outbound peers (stacked graph)
- Network traffic (graph)

### Performance Panel
- Block validation time (heatmap)
- Transaction throughput (graph)
- Mempool size (graph)

### Mining Panel
- Hashrate (graph)
- Blocks mined (counter)

## Alerting Rules

Example `alerts.yml`:

```yaml
groups:
  - name: opensyria
    interval: 30s
    rules:
      - alert: NodeDown
        expr: up{job="opensyria"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "OpenSyria node is down"
          description: "Node {{ $labels.instance }} has been down for 1 minute"

      - alert: LowPeerCount
        expr: opensyria_peer_count < 3
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Low peer count"
          description: "Node {{ $labels.instance }} has only {{ $value }} peers"

      - alert: ChainNotSyncing
        expr: rate(opensyria_chain_height[5m]) == 0
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "Chain not syncing"
          description: "No new blocks in 10 minutes on {{ $labels.instance }}"

      - alert: HighMempoolSize
        expr: opensyria_mempool_size > 10000
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Mempool congestion"
          description: "Mempool has {{ $value }} pending transactions"

      - alert: SlowBlockValidation
        expr: histogram_quantile(0.95, rate(opensyria_block_validation_seconds_bucket[5m])) > 5
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "Slow block validation"
          description: "95th percentile validation time is {{ $value }}s"
```

## Example Queries

### Average Block Time (Last Hour)
```promql
rate(opensyria_chain_height[1h]) * 3600
```

### Transaction Throughput (TPS)
```promql
rate(opensyria_transactions_processed_total{status="valid"}[1m])
```

### Cache Hit Rate
```promql
sum(rate(opensyria_db_cache_hits_total[5m])) / 
(sum(rate(opensyria_db_cache_hits_total[5m])) + sum(rate(opensyria_db_cache_misses_total[5m])))
```

### Network Bandwidth Usage
```promql
rate(opensyria_network_tx_bytes_total[1m]) + rate(opensyria_network_rx_bytes_total[1m])
```

## Integration

The metrics server starts automatically with the node. To disable:
```bash
opensyria-node --no-metrics
```

To change the port:
```bash
opensyria-node --metrics-port 9616
```

## Security Considerations

⚠️ **Production Deployment:**
- Bind metrics endpoint to `127.0.0.1` only (localhost)
- Use firewall rules to restrict access
- Consider HTTP authentication for Prometheus scraper
- Never expose metrics publicly without authentication
- Rotate credentials regularly

## Troubleshooting

### Metrics endpoint not accessible
```bash
# Check if metrics server is running
curl http://localhost:9615/health

# Check firewall rules
sudo ufw status

# Verify node configuration
opensyria-node --help | grep metrics
```

### Missing metrics
- Ensure all storage backends are initialized
- Check logs for metric registration errors
- Verify Prometheus scrape configuration

### High cardinality warnings
If you see cardinality warnings, consider:
- Limiting label values (e.g., proposal_id)
- Aggregating before export
- Increasing Prometheus memory
