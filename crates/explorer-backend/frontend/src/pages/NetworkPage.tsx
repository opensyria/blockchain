import { Layout } from '@/components/Layout';
import { useWebSocket, WsMessage } from '@/hooks/use-websocket';
import { useState } from 'react';
import './NetworkPage.css';

interface PeerInfo {
  id: string;
  address: string;
  connected_at: number;
  blocks_received: number;
  blocks_sent: number;
}

export function NetworkPage() {
  const [peers] = useState<PeerInfo[]>([
    {
      id: '12D3KooWABC123...',
      address: '/ip4/192.168.1.100/tcp/9000',
      connected_at: Date.now() / 1000 - 3600,
      blocks_received: 42,
      blocks_sent: 38,
    },
    {
      id: '12D3KooWDEF456...',
      address: '/ip4/192.168.1.101/tcp/9000',
      connected_at: Date.now() / 1000 - 1800,
      blocks_received: 28,
      blocks_sent: 31,
    },
    {
      id: '12D3KooWGHI789...',
      address: '/ip4/192.168.1.102/tcp/9000',
      connected_at: Date.now() / 1000 - 900,
      blocks_received: 15,
      blocks_sent: 12,
    },
  ]);

  const { isConnected } = useWebSocket({
    onMessage: (msg: WsMessage) => {
      // Handle peer updates in the future
      console.log('Network message:', msg);
    },
  });

  return (
    <Layout>
      <div className="container">
        <div className="network-page">
          <div className="page-header">
            <div>
              <h1 className="page-title">Network Topology</h1>
              <p className="page-subtitle">Connected peers and network health</p>
            </div>
            <div className="connection-status">
              <span className={`status-dot ${isConnected ? 'connected' : 'disconnected'}`}></span>
              <span className="status-text">
                {isConnected ? 'Monitoring' : 'Offline'}
              </span>
            </div>
          </div>

          <div className="network-stats">
            <div className="stat-box">
              <div className="stat-icon">🌐</div>
              <div className="stat-label">Connected Peers</div>
              <div className="stat-value">{peers.length}</div>
            </div>
            <div className="stat-box">
              <div className="stat-icon">📊</div>
              <div className="stat-label">Total Blocks Synced</div>
              <div className="stat-value">
                {peers.reduce((sum, p) => sum + p.blocks_received, 0)}
              </div>
            </div>
            <div className="stat-box">
              <div className="stat-icon">📤</div>
              <div className="stat-label">Blocks Propagated</div>
              <div className="stat-value">
                {peers.reduce((sum, p) => sum + p.blocks_sent, 0)}
              </div>
            </div>
          </div>

          <div className="detail-card">
            <h2 className="card-title">Peer Nodes</h2>
            <div className="peers-list">
              {peers.map((peer) => (
                <div key={peer.id} className="peer-item">
                  <div className="peer-header">
                    <span className="peer-status online">●</span>
                    <span className="peer-id monospace">{peer.id}</span>
                  </div>
                  <div className="peer-address monospace">{peer.address}</div>
                  <div className="peer-stats">
                    <div className="peer-stat">
                      <span className="peer-stat-label">Connected:</span>
                      <span className="peer-stat-value">
                        {Math.floor((Date.now() / 1000 - peer.connected_at) / 60)}m ago
                      </span>
                    </div>
                    <div className="peer-stat">
                      <span className="peer-stat-label">Blocks ↓:</span>
                      <span className="peer-stat-value">{peer.blocks_received}</span>
                    </div>
                    <div className="peer-stat">
                      <span className="peer-stat-label">Blocks ↑:</span>
                      <span className="peer-stat-value">{peer.blocks_sent}</span>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>

          <div className="detail-card">
            <h2 className="card-title">Network Visualization</h2>
            <div className="network-viz">
              <div className="viz-container">
                <div className="node center-node">
                  <div className="node-label">This Node</div>
                </div>
                {peers.map((peer, index) => (
                  <div
                    key={peer.id}
                    className="node peer-node"
                    style={{
                      transform: `rotate(${(360 / peers.length) * index}deg) translate(150px) rotate(-${(360 / peers.length) * index}deg)`,
                    }}
                  >
                    <div className="connection-line"></div>
                    <div className="node-label">{peer.id.slice(0, 10)}...</div>
                  </div>
                ))}
              </div>
              <div className="viz-legend">
                <span className="legend-item">
                  <span className="legend-dot online">●</span> Online
                </span>
                <span className="legend-item">
                  <span className="legend-dot">●</span> Offline
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Layout>
  );
}
