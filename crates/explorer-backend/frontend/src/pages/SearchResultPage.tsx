import { useParams, Navigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { Layout } from '@/components/Layout';
import { useSearch } from '@/hooks/use-api';
import './SearchResultPage.css';

export function SearchResultPage() {
  const { t } = useTranslation();
  const { query } = useParams<{ query: string }>();
  const { data, isLoading, error } = useSearch(query || '');

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
          <div className="search-result-page">
            <div className="error-state">
              <h2>{t('search.noResults')}</h2>
              <p>Query: {query}</p>
              <p className="error-hint">{t('search.error')}</p>
            </div>
          </div>
        </div>
      </Layout>
    );
  }

  // Redirect to appropriate page based on result type
  switch (data.result_type) {
    case 'block': {
      const blockData = data.data as { height: number };
      return <Navigate to={`/block/${blockData.height}`} replace />;
    }
    case 'transaction': {
      const txData = data.data as { hash: string };
      return <Navigate to={`/tx/${txData.hash}`} replace />;
    }
    case 'address': {
      const addrData = data.data as { address: string };
      return <Navigate to={`/address/${addrData.address}`} replace />;
    }
    default:
      return (
        <Layout>
          <div className="container">
            <div className="error-state">
              <h2>{t('search.noResults')}</h2>
            </div>
          </div>
        </Layout>
      );
  }
}
