import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Layout } from '@/components/Layout';
import { BlockList } from '@/components/BlockList';
import { useRecentBlocks } from '@/hooks/use-api';
import './BlocksPage.css';

export function BlocksPage() {
  const { t } = useTranslation();
  const [page, setPage] = useState(1);
  const perPage = 20;
  const { data, isLoading } = useRecentBlocks(page, perPage);

  return (
    <Layout>
      <div className="container">
        <div className="blocks-page">
          <div className="page-header">
            <h1 className="page-title">{t('nav.blocks')}</h1>
            {data && (
              <p className="page-subtitle">
                {data.total.toLocaleString()} {t('stats.blocks').toLowerCase()}
              </p>
            )}
          </div>

          <BlockList blocks={data?.data || []} loading={isLoading} />

          {data && data.total_pages > 1 && (
            <div className="pagination">
              <button
                onClick={() => setPage((p) => Math.max(1, p - 1))}
                disabled={page === 1}
                className="pagination-btn"
              >
                ← {t('common.previous')}
              </button>

              <span className="pagination-info">
                {t('common.page')} {page} {t('common.of')} {data.total_pages}
              </span>

              <button
                onClick={() => setPage((p) => Math.min(data.total_pages, p + 1))}
                disabled={page === data.total_pages}
                className="pagination-btn"
              >
                {t('common.next')} →
              </button>
            </div>
          )}
        </div>
      </div>
    </Layout>
  );
}
