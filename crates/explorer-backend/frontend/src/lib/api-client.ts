import axios, { AxiosInstance } from 'axios';
import type {
  ChainStats,
  BlockSummary,
  BlockDetail,
  Transaction,
  AddressInfo,
  SearchResult,
  PaginatedResponse,
} from '@/types/api';

class ApiClient {
  private client: AxiosInstance;

  constructor(baseURL: string = '/api') {
    this.client = axios.create({
      baseURL,
      timeout: 10000,
      headers: {
        'Content-Type': 'application/json',
      },
    });

    // Request interceptor for logging
    this.client.interceptors.request.use(
      (config) => {
        console.debug(`[API] ${config.method?.toUpperCase()} ${config.url}`);
        return config;
      },
      (error) => Promise.reject(error)
    );

    // Response interceptor for error handling
    this.client.interceptors.response.use(
      (response) => response,
      (error) => {
        console.error('[API Error]', error.response?.data || error.message);
        return Promise.reject(error);
      }
    );
  }

  // Chain statistics
  async getChainStats(): Promise<ChainStats> {
    const { data } = await this.client.get<ChainStats>('/stats');
    return data;
  }

  // Blocks
  async getRecentBlocks(
    page: number = 1,
    perPage: number = 20
  ): Promise<PaginatedResponse<BlockSummary>> {
    const { data } = await this.client.get<PaginatedResponse<BlockSummary>>(
      '/blocks',
      { params: { page, per_page: perPage } }
    );
    return data;
  }

  async getBlockByHeight(height: number): Promise<BlockDetail> {
    const { data } = await this.client.get<BlockDetail>(`/blocks/${height}`);
    return data;
  }

  async getBlockByHash(hash: string): Promise<BlockDetail> {
    const { data } = await this.client.get<BlockDetail>(`/blocks/hash/${hash}`);
    return data;
  }

  // Transactions
  async getTransaction(hash: string): Promise<Transaction> {
    const { data } = await this.client.get<Transaction>(`/transactions/${hash}`);
    return data;
  }

  // Address
  async getAddressInfo(address: string): Promise<AddressInfo> {
    const { data } = await this.client.get<AddressInfo>(`/address/${address}`);
    return data;
  }

  // Search
  async search(query: string): Promise<SearchResult> {
    const { data } = await this.client.get<SearchResult>(`/search/${encodeURIComponent(query)}`);
    return data;
  }
}

export const apiClient = new ApiClient();
