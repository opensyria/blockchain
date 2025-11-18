import { useParams, Link } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { Layout } from '@/components/Layout';
import { useAddress } from '@/hooks/use-api';
import { format } from 'date-fns';
import './AddressPage.css';

export function AddressPage() {
  const { t } = useTranslation();
  const { address } = useParams<{ address: string }>();
  const { data, isLoading, error } = useAddress(address || '');

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

  if (error || !data) {
    return (
      <Layout>
        <div className="container">
          <div className="error-state">
            <h2>{t('common.notFound')}</h2>
            <p>Address not found</p>
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
        <div className="address-page">
          <div className="page-header">
            <h1 className="page-title">{t('address.address')}</h1>
            <div className="address-display monospace">{data.address}</div>
          </div>

          <div className="balance-cards">
            <div className="balance-card primary">
              <div className="balance-label">{t('address.balance')}</div>
              <div className="balance-value">
                {data.balance.toLocaleString()} {t('units.lira')}
              </div>
            </div>

            <div className="balance-card">
              <div className="balance-label">{t('address.received')}</div>
              <div className="balance-value">
                {data.total_received.toLocaleString()} {t('units.lira')}
              </div>
            </div>

            <div className="balance-card">
              <div className="balance-label">{t('address.sent')}</div>
              <div className="balance-value">
                {data.total_sent.toLocaleString()} {t('units.lira')}
              </div>
            </div>

            <div className="balance-card">
              <div className="balance-label">{t('address.txCount')}</div>
              <div className="balance-value">
                {data.transaction_count.toLocaleString()}
              </div>
            </div>
          </div>

          <div className="detail-card">
            <h2 className="card-title">{t('address.history')}</h2>
            {data.transactions && data.transactions.length > 0 ? (
              <div className="tx-history">
                {data.transactions.map((tx) => {
                  const isReceived = tx.to === data.address;
                  return (
                    <Link
                      key={tx.hash}
                      to={`/tx/${tx.hash}`}
                      className="tx-history-item"
                    >
                      <div className="tx-type">
                        <span className={`type-badge ${isReceived ? 'received' : 'sent'}`}>
                          {isReceived ? '📥' : '📤'}
                        </span>
                      </div>

                      <div className="tx-info">
                        <div className="tx-hash monospace">{tx.hash}</div>
                        <div className="tx-parties">
                          <span className="monospace">
                            {isReceived ? tx.from.slice(0, 16) : tx.to.slice(0, 16)}...
                          </span>
                        </div>
                      </div>

                      <div className="tx-amount-col">
                        <div className={`tx-amount ${isReceived ? 'positive' : 'negative'}`}>
                          {isReceived ? '+' : '-'}{tx.amount} {t('units.lira')}
                        </div>
                        <div className="tx-time">
                          {format(new Date(tx.timestamp * 1000), 'PP')}
                        </div>
                      </div>
                    </Link>
                  );
                })}
              </div>
            ) : (
              <div className="empty-state">
                <p>{t('common.notFound')}</p>
              </div>
            )}
          </div>
        </div>
      </div>
    </Layout>
  );
}
