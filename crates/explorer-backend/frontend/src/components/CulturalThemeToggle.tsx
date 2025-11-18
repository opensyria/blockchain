import { useState, useEffect } from 'react';
import './CulturalThemeToggle.css';

export function CulturalThemeToggle() {
  const [culturalTheme, setCulturalTheme] = useState(() => {
    return localStorage.getItem('cultural-theme') === 'true';
  });

  useEffect(() => {
    if (culturalTheme) {
      document.body.classList.add('cultural-theme');
    } else {
      document.body.classList.remove('cultural-theme');
    }
    localStorage.setItem('cultural-theme', culturalTheme.toString());
  }, [culturalTheme]);

  const toggleTheme = () => {
    setCulturalTheme(!culturalTheme);
  };

  return (
    <button
      onClick={toggleTheme}
      className="cultural-theme-toggle"
      aria-label="Toggle cultural theme"
      title={culturalTheme ? 'Disable cultural theme' : 'Enable cultural theme'}
    >
      <span className="toggle-icon">{culturalTheme ? '🎨' : '⚪'}</span>
      <span className="toggle-label">
        {culturalTheme ? 'Cultural' : 'Default'}
      </span>
    </button>
  );
}
