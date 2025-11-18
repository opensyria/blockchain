import { useQuery, UseQueryOptions } from '@tanstack/react-query';
import { apiClient } from '@/lib/api-client';
import type {
  ChainStats,
  BlockSummary,
  BlockDetail,
  Transaction,
  AddressInfo,
  SearchResult,
  PaginatedResponse,
} from '@/types/api';

// Query keys factory
export const queryKeys = {
  stats: ['stats'] as const,
  blocks: {
    all: ['blocks'] as const,
    list: (page: number, perPage: number) => ['blocks', 'list', page, perPage] as const,
    byHeight: (height: number) => ['blocks', 'height', height] as const,
    byHash: (hash: string) => ['blocks', 'hash', hash] as const,
  },
  transactions: {
    all: ['transactions'] as const,
    byHash: (hash: string) => ['transactions', hash] as const,
  },
  addresses: {
    all: ['addresses'] as const,
    byAddress: (address: string) => ['addresses', address] as const,
  },
  search: (query: string) => ['search', query] as const,
};

// Hook: Chain statistics
export function useChainStats(options?: UseQueryOptions<ChainStats>) {
  return useQuery({
    queryKey: queryKeys.stats,
    queryFn: () => apiClient.getChainStats(),
    refetchInterval: 10000, // Refresh every 10 seconds
    ...options,
  });
}

// Hook: Recent blocks
export function useRecentBlocks(
  page: number = 1,
  perPage: number = 20,
  options?: UseQueryOptions<PaginatedResponse<BlockSummary>>
) {
  return useQuery({
    queryKey: queryKeys.blocks.list(page, perPage),
    queryFn: () => apiClient.getRecentBlocks(page, perPage),
    refetchInterval: 15000, // Refresh every 15 seconds
    ...options,
  });
}

// Hook: Block by height
export function useBlock(
  height: number,
  options?: Omit<UseQueryOptions<BlockDetail>, 'queryKey' | 'queryFn'>
) {
  return useQuery({
    queryKey: queryKeys.blocks.byHeight(height),
    queryFn: () => apiClient.getBlockByHeight(height),
    enabled: height > 0,
    ...options,
  });
}

// Hook: Block by hash
export function useBlockByHash(
  hash: string,
  options?: Omit<UseQueryOptions<BlockDetail>, 'queryKey' | 'queryFn'>
) {
  return useQuery({
    queryKey: queryKeys.blocks.byHash(hash),
    queryFn: () => apiClient.getBlockByHash(hash),
    enabled: hash.length > 0,
    ...options,
  });
}

// Hook: Transaction
export function useTransaction(
  hash: string,
  options?: UseQueryOptions<Transaction>
) {
  return useQuery({
    queryKey: queryKeys.transactions.byHash(hash),
    queryFn: () => apiClient.getTransaction(hash),
    enabled: hash.length > 0,
    ...options,
  });
}

// Hook: Address info
export function useAddress(
  address: string,
  options?: UseQueryOptions<AddressInfo>
) {
  return useQuery({
    queryKey: queryKeys.addresses.byAddress(address),
    queryFn: () => apiClient.getAddressInfo(address),
    enabled: address.length > 0,
    ...options,
  });
}

// Hook: Search
export function useSearch(
  query: string,
  options?: UseQueryOptions<SearchResult>
) {
  return useQuery({
    queryKey: queryKeys.search(query),
    queryFn: () => apiClient.search(query),
    enabled: query.length > 0,
    retry: false, // Don't retry failed searches
    ...options,
  });
}
