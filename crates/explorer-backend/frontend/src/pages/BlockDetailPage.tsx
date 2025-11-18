import { useParams, Link } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { Layout } from '@/components/Layout';
import { useBlock, useBlockByHash } from '@/hooks/use-api';
import { format } from 'date-fns';
import './BlockDetailPage.css';

export function BlockDetailPage() {
  const { t } = useTranslation();
  const { heightOrHash } = useParams<{ heightOrHash: string }>();

  // Determine if it's a height (number) or hash
  const isHeight = /^\d+$/.test(heightOrHash || '');
  
  const { data: blockByHeight, isLoading: loadingHeight, error: errorHeight } = useBlock(
    isHeight ? parseInt(heightOrHash!) : 0,
    { enabled: isHeight }
  );
  
  const { data: blockByHash, isLoading: loadingHash, error: errorHash } = useBlockByHash(
    !isHeight ? heightOrHash! : '',
    { enabled: !isHeight }
  );

  const block = isHeight ? blockByHeight : blockByHash;
  const isLoading = isHeight ? loadingHeight : loadingHash;
  const error = isHeight ? errorHeight : errorHash;

  if (isLoading) {
    return (
      <Layout>
        <div className="container">
          <div className="loading-state">
            <div className="loading-spinner"></div>
            <p>{t('common.loading')}</p>
          </div>
        </div>
      </Layout>
    );
  }

  if (error || !block) {
    return (
      <Layout>
        <div className="container">
          <div className="error-state">
            <h2>{t('common.notFound')}</h2>
            <p>Block not found</p>
            <Link to="/" className="btn-primary">
              {t('nav.home')}
            </Link>
          </div>
        </div>
      </Layout>
    );
  }

  return (
    <Layout>
      <div className="container">
        <div className="block-detail-page">
          <div className="page-header">
            <h1 className="page-title">
              {t('block.height')} #{block.height.toLocaleString()}
            </h1>
            <div className="page-meta">
              {format(new Date(block.timestamp * 1000), 'PPpp')}
            </div>
          </div>

          <div className="detail-card">
            <h2 className="card-title">{t('block.hash')}</h2>
            <div className="detail-grid">
              <DetailRow label={t('block.hash')} value={block.hash} mono />
              <DetailRow label={t('block.previousHash')} value={block.previous_hash} mono />
              <DetailRow label={t('block.merkleRoot')} value={block.merkle_root} mono />
              <DetailRow label={t('block.height')} value={block.height.toLocaleString()} />
              <DetailRow label={t('block.timestamp')} value={format(new Date(block.timestamp * 1000), 'PPpp')} />
              <DetailRow label={t('block.miner')} value={block.miner} mono />
              <DetailRow label={t('block.difficulty')} value={block.difficulty} />
              <DetailRow label={t('block.nonce')} value={block.nonce.toString()} />
              <DetailRow label={t('block.transactions')} value={block.transactions_count.toString()} />
              <DetailRow label={t('block.confirmations')} value={block.confirmations?.toLocaleString() || '0'} />
              <DetailRow label={t('block.size')} value={`${block.size?.toLocaleString() || 0} ${t('units.bytes')}`} />
            </div>
          </div>

          {block.transactions && block.transactions.length > 0 && (
            <div className="detail-card">
              <h2 className="card-title">
                {t('block.transactions')} ({block.transactions.length})
              </h2>
              <div className="transactions-list">
                {block.transactions.map((tx, index) => (
                  <Link
                    key={tx.hash || index}
                    to={`/tx/${tx.hash}`}
                    className="transaction-item"
                  >
                    <div className="tx-hash monospace">{tx.hash}</div>
                    <div className="tx-details">
                      <span>{tx.from.slice(0, 16)}...</span>
                      <span>→</span>
                      <span>{tx.to.slice(0, 16)}...</span>
                      <span className="tx-amount">{tx.amount} {t('units.lira')}</span>
                    </div>
                  </Link>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>
    </Layout>
  );
}

function DetailRow({ label, value, mono }: { label: string; value: string; mono?: boolean }) {
  return (
    <div className="detail-row">
      <div className="detail-label">{label}</div>
      <div className={`detail-value ${mono ? 'monospace' : ''}`}>{value}</div>
    </div>
  );
}
