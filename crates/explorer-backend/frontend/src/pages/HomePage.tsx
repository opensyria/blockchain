import { useTranslation } from 'react-i18next';
import { Layout } from '@/components/Layout';
import { SearchBar } from '@/components/SearchBar';
import { StatCard } from '@/components/StatCard';
import { BlockList } from '@/components/BlockList';
import { useChainStats, useRecentBlocks } from '@/hooks/use-api';
import { useWebSocket, WsMessage } from '@/hooks/use-websocket';
import { useEffect, useState } from 'react';
import './HomePage.css';

export function HomePage() {
  const { t } = useTranslation();
  const { data: stats, isLoading: statsLoading, refetch: refetchStats } = useChainStats();
  const { data: blocksData, isLoading: blocksLoading, refetch: refetchBlocks } = useRecentBlocks(1, 10);
  const [wsConnected, setWsConnected] = useState(false);

  // WebSocket for real-time updates
  const { isConnected } = useWebSocket({
    onMessage: (msg: WsMessage) => {
      if (msg.type === 'new_block' || msg.type === 'stats_update') {
        // Refetch data when new block arrives
        refetchStats();
        refetchBlocks();
      }
    },
  });

  useEffect(() => {
    setWsConnected(isConnected);
  }, [isConnected]);

  return (
    <Layout>
      <section className="hero gradient-heritage pattern-islamic">
          <div className="container animate-fade-in-down">
            <div className="corner-ornament">
              <h1 className="hero-title">{t('app.title')}</h1>
              <p className="hero-subtitle">{t('app.subtitle')}</p>
              {wsConnected && (
                <div className="live-indicator animate-fade-in">
                  <span className="live-dot"></span>
                  <span className="live-text">Live Updates</span>
                </div>
              )}
            </div>
            <div className="hero-search animate-scale-in">
              <SearchBar />
            </div>
          </div>
        </section>

      <section className="stats-section">
        <div className="container">
          <div className="stats-grid stagger-children">
            <div className="card-cultural hover-lift">
              <StatCard
                label={t('stats.height')}
                value={stats?.height.toLocaleString() || 0}
                loading={statsLoading}
              />
            </div>
            <div className="card-cultural hover-lift">
              <StatCard
                label={t('stats.transactions')}
                value={stats?.total_transactions.toLocaleString() || 0}
                loading={statsLoading}
              />
            </div>
            <div className="card-cultural hover-lift">
              <StatCard
                label={t('stats.hashRate')}
                value={stats ? `${(stats.hash_rate / 1_000_000).toFixed(2)} MH/s` : '0'}
                loading={statsLoading}
              />
            </div>
            <div className="card-cultural hover-lift">
              <StatCard
                label={t('stats.blockTime')}
                value={stats ? `${stats.avg_block_time.toFixed(1)}s` : '0'}
                loading={statsLoading}
              />
            </div>
          </div>
        </div>
      </section>      <section className="blocks-section">
        <div className="container">
          <div className="divider-calligraphic">
            <span>✦</span>
          </div>
          <h2 className="section-title animate-fade-in-up">{t('common.recent')} {t('nav.blocks')}</h2>
          <div className="animate-fade-in-up">
            <BlockList
              blocks={blocksData?.data || []}
              loading={blocksLoading}
            />
          </div>
        </div>
      </section>
    </Layout>
  );
}
