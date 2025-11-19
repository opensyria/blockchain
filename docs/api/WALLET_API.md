# Wallet REST API

REST API server for OpenSyria blockchain wallet operations.

## Features

- **Transaction Submission**: Submit signed transactions to the blockchain
- **Transaction Creation**: Create and sign transactions in one step (development mode)
- **Balance Queries**: Check account balances and nonces
- **Blockchain Info**: Get chain height, difficulty, and statistics  
- **Mempool Status**: View pending transactions

## Installation

```bash
cargo build --release -p opensyria-wallet-api
```

## Usage

### Start Server

```bash
# Default (port 8080, ~/.opensyria/node)
opensyria-wallet-api

# Custom configuration
opensyria-wallet-api -d /path/to/node --port 3001 --host 0.0.0.0
```

### API Endpoints

#### Health Check
```bash
GET /health
```

Response:
```json
{
  "status": "healthy",
  "service": "opensyria-wallet-api"
}
```

#### Blockchain Info
```bash
GET /api/v1/blockchain/info
```

Response:
```json
{
  "chain_height": 6,
  "latest_block_hash": "0000812fab2b9535f66d912b4f72c2d4f410ea170da8be8249bc890866f4ad51",
  "difficulty": 16,
  "total_transactions": 15
}
```

#### Account Balance
```bash
GET /api/v1/account/{address}/balance
```

Response:
```json
{
  "address": "b1946ac92492d2347c6235b4d2611184ac13518c...",
  "balance": 5000000000,
  "nonce": 3
}
```

#### Mempool Status
```bash
GET /api/v1/mempool/status
```

Response:
```json
{
  "pending_count": 2,
  "total_fees": 200
}
```

#### Submit Transaction
```bash
POST /api/v1/transaction/submit
Content-Type: application/json

{
  "from": "hex_encoded_public_key",
  "to": "hex_encoded_public_key",
  "amount": 1000000,
  "fee": 100,
  "signature": "hex_encoded_signature"
}
```

Response:
```json
{
  "success": true,
  "tx_hash": "2df5fb0314db70a822bc09eb8e7db44fb00eb325e48eab75...",
  "message": "Transaction submitted successfully"
}
```

Error Response:
```json
{
  "error": "Insufficient balance"
}
```

#### Create and Sign Transaction (Development Only)
```bash
POST /api/v1/transaction/create
Content-Type: application/json

{
  "from": "hex_encoded_public_key",
  "to": "hex_encoded_public_key",
  "amount": 1000000,
  "fee": 100,
  "private_key": "hex_encoded_private_key"
}
```

**⚠️ Warning**: This endpoint should only be used in development. Never send private keys over the network in production.

## Architecture

The wallet API is built on:
- **Axum**: Fast, ergonomic web framework
- **Tokio**: Async runtime
- **Tower**: Middleware (CORS, tracing)
- **Node Library**: Direct access to blockchain and state

### Request Flow

1. HTTP request received by Axum server
2. Request validated and parsed
3. Node state accessed via RwLock (concurrent reads, exclusive writes)
4. Blockchain/state operations performed
5. JSON response returned

### State Management

The API uses Arc<RwLock<Node>> for thread-safe access:
- Multiple read requests can query balances/blockchain simultaneously
- Write requests (transaction submission) require exclusive access
- No database queries needed - direct access to node state

## Examples

### Check Balance

```bash
# Get your public key
PUBKEY=$(wallet info alice | grep "Public Key" | cut -d: -f2 | tr -d ' ')

# Query balance
curl "http://localhost:8080/api/v1/account/${PUBKEY}/balance" | jq .
```

### Submit Transaction

```bash
# Create transaction with wallet CLI
wallet send --from alice --to bob --amount 1000000 --output tx.json

# Submit via API
curl -X POST http://localhost:8080/api/v1/transaction/submit \
  -H "Content-Type: application/json" \
  -d @tx.json
```

### Monitor Mempool

```bash
# Watch pending transactions
watch -n 2 'curl -s http://localhost:8080/api/v1/mempool/status | jq .'
```

## Security Considerations

### Production Deployment

1. **HTTPS Only**: Always use TLS in production
2. **Rate Limiting**: Implement request throttling
3. **Authentication**: Add API keys or JWT tokens
4. **Input Validation**: Already validates signatures and balances
5. **CORS**: Configure allowed origins (currently allows all)
6. **Private Keys**: Never use `/transaction/create` endpoint in production

### Network Configuration

```bash
# Localhost only (secure)
opensyria-wallet-api --host 127.0.0.1

# All interfaces (requires firewall)
opensyria-wallet-api --host 0.0.0.0

# Behind reverse proxy (recommended for production)
# nginx/caddy handles TLS, rate limiting, auth
```

## Testing

```bash
# Run test script
./test-wallet-api.sh

# Manual testing
./target/debug/opensyria-wallet-api -d data &
curl http://localhost:8080/health
curl http://localhost:8080/api/v1/blockchain/info
pkill opensyria-wallet-api
```

## Error Handling

The API returns appropriate HTTP status codes:

- `200 OK`: Successful request
- `400 Bad Request`: Invalid input (bad address, signature, etc.)
- `500 Internal Server Error`: Node/storage errors

All errors include a JSON response with an `error` field describing the issue.

## Performance

- **Concurrent Reads**: Multiple balance queries can run simultaneously
- **Async I/O**: Non-blocking network operations
- **Memory Efficient**: Direct access to node state, no data duplication
- **Fast Response**: <10ms for balance queries on local node

## Future Enhancements

- [ ] WebSocket support for real-time updates
- [ ] Transaction history endpoint
- [ ] Batch transaction submission
- [ ] Fee estimation endpoint
- [ ] Transaction status tracking
- [ ] GraphQL API
- [ ] OpenAPI/Swagger documentation
- [ ] Rate limiting middleware
- [ ] Authentication/authorization
- [ ] Metrics and monitoring endpoints
