import { useTranslation } from 'react-i18next';
import { Layout } from '@/components/Layout';
import { useWebSocket, WsMessage } from '@/hooks/use-websocket';
import { useState, useEffect } from 'react';
import './MempoolPage.css';

interface PendingTransaction {
  hash: string;
  from: string;
  to: string;
  amount: number;
  fee: number;
  timestamp: number;
}

export function MempoolPage() {
  const { t } = useTranslation();
  const [transactions, setTransactions] = useState<PendingTransaction[]>([]);
  const [stats, setStats] = useState({
    pending_count: 0,
    total_fees: 0,
  });

  const { isConnected } = useWebSocket({
    onMessage: (msg: WsMessage) => {
      if (msg.type === 'new_transaction') {
        // Add new transaction to the list
        const newTx: PendingTransaction = {
          hash: msg.hash || '',
          from: msg.from || '',
          to: msg.to || '',
          amount: msg.amount || 0,
          fee: 0, // Would come from backend
          timestamp: Date.now() / 1000,
        };
        setTransactions((prev) => [newTx, ...prev].slice(0, 50)); // Keep last 50
      } else if (msg.type === 'mempool_update') {
        setStats({
          pending_count: msg.pending_count || 0,
          total_fees: msg.total_fees || 0,
        });
      }
    },
  });

  // Mock data for demonstration (remove when backend is ready)
  useEffect(() => {
    if (transactions.length === 0) {
      setTransactions([
        {
          hash: '0x' + '89abcdef'.repeat(8),
          from: '0x' + '1234567890abcdef'.repeat(4),
          to: '0x' + '5678901234abcdef'.repeat(4),
          amount: 10.5,
          fee: 0.001,
          timestamp: Date.now() / 1000 - 30,
        },
        {
          hash: '0x' + '78defabc'.repeat(8),
          from: '0x' + '9876543210fedcba'.repeat(4),
          to: '0x' + '4321098765fedcba'.repeat(4),
          amount: 25.0,
          fee: 0.002,
          timestamp: Date.now() / 1000 - 60,
        },
      ]);
      setStats({ pending_count: 2, total_fees: 0.003 });
    }
  }, [transactions.length]);

  return (
    <Layout>
      <div className="container">
        <div className="mempool-page">
          <div className="page-header">
            <h1 className="page-title">Transaction Pool (Mempool)</h1>
            <div className="connection-status">
              <span className={`status-dot ${isConnected ? 'connected' : 'disconnected'}`}></span>
              <span className="status-text">
                {isConnected ? 'Live' : 'Connecting...'}
              </span>
            </div>
          </div>

          <div className="mempool-stats">
            <div className="stat-box">
              <div className="stat-label">Pending Transactions</div>
              <div className="stat-value">{stats.pending_count}</div>
            </div>
            <div className="stat-box">
              <div className="stat-label">Total Fees</div>
              <div className="stat-value">{stats.total_fees} {t('units.lira')}</div>
            </div>
          </div>

          <div className="detail-card">
            <h2 className="card-title">Pending Transactions</h2>
            {transactions.length > 0 ? (
              <div className="tx-list">
                {transactions.map((tx) => (
                  <div key={tx.hash} className="mempool-tx-item">
                    <div className="tx-header">
                      <span className="tx-hash monospace">{tx.hash}</span>
                      <span className="tx-age">
                        {Math.floor((Date.now() / 1000 - tx.timestamp) / 60)}m ago
                      </span>
                    </div>
                    <div className="tx-details">
                      <div className="tx-flow">
                        <span className="monospace">{tx.from.slice(0, 16)}...</span>
                        <span className="arrow">→</span>
                        <span className="monospace">{tx.to.slice(0, 16)}...</span>
                      </div>
                      <div className="tx-amounts">
                        <span className="amount">{tx.amount} {t('units.lira')}</span>
                        <span className="fee">Fee: {tx.fee} {t('units.lira')}</span>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <div className="empty-state">
                <p>No pending transactions</p>
              </div>
            )}
          </div>
        </div>
      </div>
    </Layout>
  );
}
