import { useState, FormEvent } from 'react';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import './SearchBar.css';

export function SearchBar() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [query, setQuery] = useState('');

  const handleSubmit = (e: FormEvent) => {
    e.preventDefault();
    const trimmed = query.trim();
    
    if (!trimmed) return;

    // Determine search type and navigate accordingly
    if (/^\d+$/.test(trimmed)) {
      // Numeric = block height
      navigate(`/block/${trimmed}`);
    } else if (/^[0-9a-fA-F]{64}$/.test(trimmed)) {
      // 64 hex chars = hash (could be block or tx)
      navigate(`/search/${trimmed}`);
    } else {
      // Assume address or general search
      navigate(`/search/${encodeURIComponent(trimmed)}`);
    }

    setQuery('');
  };

  return (
    <form onSubmit={handleSubmit} className="search-bar">
      <input
        type="text"
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        placeholder={t('search.placeholder')}
        className="search-input"
      />
      <button type="submit" className="search-button">
        {t('search.button')}
      </button>
    </form>
  );
}
