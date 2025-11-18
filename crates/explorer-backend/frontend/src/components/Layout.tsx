import { ReactNode } from 'react';
import { Link } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useLanguageStore } from '@/store/language-store';
import { CulturalThemeToggle } from './CulturalThemeToggle';
import './Layout.css';

interface LayoutProps {
  children: ReactNode;
}

export function Layout({ children }: LayoutProps) {
  const { t } = useTranslation();
  const { language, toggleLanguage } = useLanguageStore();

  return (
    <div className="layout">
      <header className="header">
        <div className="container">
          <div className="header-content">
            <Link to="/" className="logo">
              <h1 className="logo-text text-gradient">{t('app.title')}</h1>
              <p className="logo-subtitle">{t('app.subtitle')}</p>
            </Link>

            <nav className="nav">
              <Link to="/" className="nav-link hover-lift">{t('nav.home')}</Link>
              <Link to="/blocks" className="nav-link hover-lift">{t('nav.blocks')}</Link>
              <Link to="/mempool" className="nav-link hover-lift">{t('nav.mempool')}</Link>
              <Link to="/network" className="nav-link hover-lift">{t('nav.network')}</Link>
              <Link to="/analytics" className="nav-link hover-lift">{t('nav.analytics')}</Link>
              <Link to="/identity" className="nav-link hover-lift">{t('nav.identity')}</Link>
              <Link to="/governance" className="nav-link hover-lift">{t('nav.governance')}</Link>
              <Link to="/governance" className="nav-link hover-lift">{t('nav.governance')}</Link>
              <Link to="/identity" className="nav-link hover-lift">{t('nav.identity')}</Link>
              
              <div className="header-controls">
                <CulturalThemeToggle />
                <button onClick={toggleLanguage} className="lang-toggle">
                  {language === 'en' ? 'العربية' : 'English'}
                </button>
              </div>
            </nav>
          </div>
        </div>
      </header>

      <main className="main">
        {children}
      </main>

      <footer className="footer">
        <div className="container">
          <p className="footer-text">
            {t('app.title')} • Open Source • MIT License
          </p>
        </div>
      </footer>
    </div>
  );
}
