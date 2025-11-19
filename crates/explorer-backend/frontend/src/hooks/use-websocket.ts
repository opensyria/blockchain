import { useEffect, useRef, useState, useCallback } from 'react';

export type WsMessageType = 
  | 'new_block'
  | 'new_transaction'
  | 'stats_update'
  | 'mempool_update'
  | 'ping'
  | 'pong';

export interface WsMessage {
  type: WsMessageType;
  height?: number;
  hash?: string;
  transactions?: number;
  timestamp?: number;
  from?: string;
  to?: string;
  amount?: number;
  total_transactions?: number;
  difficulty?: string;
  hash_rate?: number;
  pending_count?: number;
  total_fees?: number;
}

interface UseWebSocketOptions {
  url?: string;
  reconnectAttempts?: number;
  reconnectInterval?: number;
  onMessage?: (message: WsMessage) => void;
}

// Determine secure WebSocket URL based on page protocol
function getSecureWebSocketUrl(defaultUrl?: string): string {
  if (defaultUrl) {
    // If URL provided, validate it
    if (import.meta.env.PROD && !defaultUrl.startsWith('wss://')) {
      console.warn('[WebSocket] Insecure URL provided in production, upgrading to WSS');
      return defaultUrl.replace(/^ws:\/\//, 'wss://');
    }
    return defaultUrl;
  }
  
  // Auto-detect based on page protocol
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  const hostname = window.location.hostname;
  const port = window.location.port || (protocol === 'wss:' ? '443' : '80');
  return `${protocol}//${hostname}:${port}/ws`;
}

// Validate WebSocket message structure
function validateWsMessage(data: unknown): WsMessage | null {
  if (typeof data !== 'object' || data === null) {
    console.warn('[WebSocket] Invalid message: not an object');
    return null;
  }
  
  const msg = data as Record<string, unknown>;
  const validTypes: WsMessageType[] = ['new_block', 'new_transaction', 'stats_update', 'mempool_update', 'ping', 'pong'];
  
  if (!msg.type || !validTypes.includes(msg.type as WsMessageType)) {
    console.warn('[WebSocket] Invalid message type:', msg.type);
    return null;
  }
  
  // Type-specific validation
  if (msg.type === 'new_block') {
    if (typeof msg.height !== 'number' || typeof msg.hash !== 'string') {
      console.warn('[WebSocket] Malformed new_block message');
      return null;
    }
  } else if (msg.type === 'new_transaction') {
    if (typeof msg.hash !== 'string') {
      console.warn('[WebSocket] Malformed new_transaction message');
      return null;
    }
  }
  
  return msg as unknown as WsMessage;
}

export function useWebSocket({
  url,
  reconnectAttempts = 5,
  reconnectInterval = 3000,
  onMessage,
}: UseWebSocketOptions = {}) {
  const [isConnected, setIsConnected] = useState(false);
  const [lastMessage, setLastMessage] = useState<WsMessage | null>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectCountRef = useRef(0);
  const reconnectTimeoutRef = useRef<number>();

  const connect = useCallback(() => {
    const secureUrl = getSecureWebSocketUrl(url);
    
    // Refuse insecure connections in production
    if (import.meta.env.PROD && !secureUrl.startsWith('wss://')) {
      console.error('[WebSocket] Refusing insecure connection in production');
      return;
    }
    
    try {
      const ws = new WebSocket(secureUrl);

      ws.onopen = () => {
        console.log('[WebSocket] Connected');
        setIsConnected(true);
        reconnectCountRef.current = 0;
      };

      ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          const message = validateWsMessage(data);
          
          if (!message) {
            console.error('[WebSocket] Invalid message received, rejecting');
            return;
          }
          
          setLastMessage(message);
          onMessage?.(message);
        } catch (error) {
          console.error('[WebSocket] Failed to parse message:', error);
        }
      };

      ws.onerror = (error) => {
        console.error('[WebSocket] Error:', error);
      };

      ws.onclose = () => {
        console.log('[WebSocket] Disconnected');
        setIsConnected(false);
        wsRef.current = null;

        // Attempt reconnect
        if (reconnectCountRef.current < reconnectAttempts) {
          reconnectCountRef.current++;
          console.log(
            `[WebSocket] Reconnecting... (${reconnectCountRef.current}/${reconnectAttempts})`
          );
          reconnectTimeoutRef.current = setTimeout(() => {
            connect();
          }, reconnectInterval);
        } else {
          console.log('[WebSocket] Max reconnection attempts reached');
        }
      };

      wsRef.current = ws;
    } catch (error) {
      console.error('[WebSocket] Connection failed:', error);
    }
  }, [url, reconnectAttempts, reconnectInterval, onMessage]);

  const disconnect = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
    }
    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }
  }, []);

  const sendMessage = useCallback((message: WsMessage) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify(message));
    } else {
      console.warn('[WebSocket] Cannot send message: not connected');
    }
  }, []);

  useEffect(() => {
    connect();

    return () => {
      disconnect();
    };
  }, [connect, disconnect]);

  return {
    isConnected,
    lastMessage,
    sendMessage,
    reconnect: connect,
    disconnect,
  };
}
