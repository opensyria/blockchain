import { useParams, Link } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { Layout } from '@/components/Layout';
import { useTransaction } from '@/hooks/use-api';
import { format } from 'date-fns';
import './TransactionPage.css';

export function TransactionPage() {
  const { t } = useTranslation();
  const { hash } = useParams<{ hash: string }>();
  const { data: tx, isLoading, error } = useTransaction(hash || '');

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

  if (error || !tx) {
    return (
      <Layout>
        <div className="container">
          <div className="error-state">
            <h2>{t('common.notFound')}</h2>
            <p>Transaction not found</p>
            <Link to="/" className="btn-primary">
              {t('nav.home')}
            </Link>
          </div>
        </div>
      </Layout>
    );
  }

  const isConfirmed = (tx.confirmations || 0) > 0;

  return (
    <Layout>
      <div className="container">
        <div className="transaction-page">
          <div className="page-header">
            <h1 className="page-title">{t('transaction.hash')}</h1>
            <div className="tx-status">
              <span className={`status-badge ${isConfirmed ? 'confirmed' : 'pending'}`}>
                {isConfirmed ? t('transaction.confirmed') : t('transaction.pending')}
              </span>
            </div>
          </div>

          <div className="detail-card">
            <div className="tx-hash-display monospace">{tx.hash}</div>
          </div>

          <div className="detail-card">
            <h2 className="card-title">{t('transaction.status')}</h2>
            <div className="detail-grid">
              <DetailRow
                label={t('transaction.from')}
                value={tx.from}
                mono
                linkTo={`/address/${tx.from}`}
              />
              <DetailRow
                label={t('transaction.to')}
                value={tx.to}
                mono
                linkTo={`/address/${tx.to}`}
              />
              <DetailRow
                label={t('transaction.amount')}
                value={`${tx.amount} ${t('units.lira')}`}
                highlight
              />
              <DetailRow
                label={t('transaction.fee')}
                value={`${tx.fee} ${t('units.lira')}`}
              />
              <DetailRow
                label={t('transaction.timestamp')}
                value={format(new Date(tx.timestamp * 1000), 'PPpp')}
              />
              {tx.block_height && (
                <DetailRow
                  label={t('block.height')}
                  value={`#${tx.block_height.toLocaleString()}`}
                  linkTo={`/block/${tx.block_height}`}
                />
              )}
              <DetailRow
                label={t('block.confirmations')}
                value={(tx.confirmations || 0).toLocaleString()}
              />
              <DetailRow
                label={t('transaction.signature')}
                value={tx.signature}
                mono
              />
            </div>
          </div>

          <div className="detail-card tx-flow">
            <h2 className="card-title">Transaction Flow</h2>
            <div className="flow-diagram">
              <div className="flow-node">
                <div className="flow-label">{t('transaction.from')}</div>
                <Link to={`/address/${tx.from}`} className="flow-address monospace">
                  {tx.from}
                </Link>
              </div>
              
              <div className="flow-arrow">
                <div className="arrow-line"></div>
                <div className="arrow-amount">
                  {tx.amount} {t('units.lira')}
                </div>
              </div>
              
              <div className="flow-node">
                <div className="flow-label">{t('transaction.to')}</div>
                <Link to={`/address/${tx.to}`} className="flow-address monospace">
                  {tx.to}
                </Link>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Layout>
  );
}

function DetailRow({
  label,
  value,
  mono,
  highlight,
  linkTo,
}: {
  label: string;
  value: string;
  mono?: boolean;
  highlight?: boolean;
  linkTo?: string;
}) {
  return (
    <div className="detail-row">
      <div className="detail-label">{label}</div>
      <div className={`detail-value ${mono ? 'monospace' : ''} ${highlight ? 'highlight' : ''}`}>
        {linkTo ? (
          <Link to={linkTo} className="detail-link">
            {value}
          </Link>
        ) : (
          value
        )}
      </div>
    </div>
  );
}
