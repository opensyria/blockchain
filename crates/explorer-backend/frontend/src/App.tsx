import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { useEffect } from 'react';
import { useLanguageStore } from '@/store/language-store';
import './styles/typography.css';
import './styles/cultural.css';
import './styles/animations.css';
import './components/PWABadge.css';
import { HomePage } from '@/pages/HomePage';
import { BlocksPage } from '@/pages/BlocksPage';
import { BlockDetailPage } from '@/pages/BlockDetailPage';
import { TransactionPage } from '@/pages/TransactionPage';
import { AddressPage } from '@/pages/AddressPage';
import { SearchResultPage } from '@/pages/SearchResultPage';
import { MempoolPage } from '@/pages/MempoolPage';
import { NetworkPage } from '@/pages/NetworkPage';
import { AnalyticsPage } from '@/pages/AnalyticsPage';
import { IdentityPage } from '@/pages/IdentityPage';
import { GovernancePage } from '@/pages/GovernancePage';
import { PWABadge } from '@/components/PWABadge';
import './lib/i18n';

function NotFoundPage() {
  const { t } = useTranslation();
  return (
    <div style={{ textAlign: 'center', padding: '2rem' }}>
      <h1>{t('common.notFound')}</h1>
    </div>
  );
}

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 5000,
      retry: 1,
      refetchOnWindowFocus: false,
    },
  },
});

function App() {
  const { i18n } = useTranslation();
  const { language, direction } = useLanguageStore();

  // Sync i18n with zustand store
  useEffect(() => {
    if (i18n.language !== language) {
      i18n.changeLanguage(language);
    }
  }, [language, i18n]);

  // Apply direction to root element
  useEffect(() => {
    document.documentElement.dir = direction;
  }, [direction]);

  return (
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <Routes>
          <Route path="/" element={<HomePage />} />
          <Route path="/blocks" element={<BlocksPage />} />
          <Route path="/block/:heightOrHash" element={<BlockDetailPage />} />
          <Route path="/tx/:hash" element={<TransactionPage />} />
          <Route path="/address/:address" element={<AddressPage />} />
          <Route path="/mempool" element={<MempoolPage />} />
          <Route path="/network" element={<NetworkPage />} />
          <Route path="/analytics" element={<AnalyticsPage />} />
          <Route path="/identity" element={<IdentityPage />} />
          <Route path="/governance" element={<GovernancePage />} />
          <Route path="/search/:query" element={<SearchResultPage />} />
          <Route path="*" element={<NotFoundPage />} />
        </Routes>
        <PWABadge />
      </BrowserRouter>
    </QueryClientProvider>
  );
}

export default App;
