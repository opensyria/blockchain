// Auto-generated types matching backend API responses

export interface ChainStats {
  height: number;
  total_blocks: number;
  total_transactions: number;
  difficulty: string;
  hash_rate: number;
  avg_block_time: number;
  circulating_supply: number;
}

export interface BlockSummary {
  height: number;
  hash: string;
  previous_hash: string;
  timestamp: number;
  transactions_count: number;
  miner: string;
  difficulty: string;
  nonce: number;
}

export interface BlockDetail extends BlockSummary {
  transactions: Transaction[];
  merkle_root: string;
  size: number;
  confirmations: number;
}

export interface Transaction {
  hash: string;
  from: string;
  to: string;
  amount: number;
  fee: number;
  timestamp: number;
  signature: string;
  block_height?: number;
  confirmations?: number;
}

export interface AddressInfo {
  address: string;
  balance: number;
  total_sent: number;
  total_received: number;
  transaction_count: number;
  transactions: Transaction[];
}

export interface SearchResult {
  result_type: 'block' | 'transaction' | 'address';
  data: BlockDetail | Transaction | AddressInfo;
}

export interface PaginatedResponse<T> {
  data: T[];
  page: number;
  per_page: number;
  total: number;
  total_pages: number;
}

export interface ErrorResponse {
  error: string;
  message: string;
}

// WebSocket message types
export interface WSMessage {
  type: 'new_block' | 'new_transaction' | 'stats_update';
  data: BlockSummary | Transaction | ChainStats;
}
