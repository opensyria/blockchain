import { useLanguageStore } from '@/store/language-store';

/**
 * Locale-aware formatting utilities for numbers, currency, dates, and times
 * Supports English (en-US) and Arabic (ar-SY) with proper numeral systems
 */
export function useLocaleFormatter() {
  const { language } = useLanguageStore();
  
  /**
   * Format numbers with locale-specific separators and numerals
   * English: 1,234,567
   * Arabic:  ١٬٢٣٤٬٥٦٧ (Eastern Arabic numerals)
   */
  const formatNumber = (num: number | undefined): string => {
    if (num === undefined || num === null) return '0';
    return num.toLocaleString(language === 'ar' ? 'ar-SY' : 'en-US');
  };
  
  /**
   * Format currency with locale-specific number format and unit
   * English: 1,234,567 SYL
   * Arabic:  ١٬٢٣٤٬٥٦٧ ل.س.ر
   */
  const formatCurrency = (amount: number): string => {
    const formatted = formatNumber(amount);
    return language === 'ar' ? `${formatted} ل.س.ر` : `${formatted} SYL`;
  };
  
  /**
   * Format large numbers with K/M/B suffixes
   * English: 1.2M, 3.4K
   * Arabic:  ١٫٢ م, ٣٫٤ ك
   */
  const formatCompactNumber = (num: number): string => {
    const abs = Math.abs(num);
    const sign = num < 0 ? '-' : '';
    
    if (abs >= 1e9) {
      const val = (num / 1e9).toFixed(1);
      return language === 'ar' ? `${sign}${formatNumber(parseFloat(val))} ب` : `${sign}${val}B`;
    }
    if (abs >= 1e6) {
      const val = (num / 1e6).toFixed(1);
      return language === 'ar' ? `${sign}${formatNumber(parseFloat(val))} م` : `${sign}${val}M`;
    }
    if (abs >= 1e3) {
      const val = (num / 1e3).toFixed(1);
      return language === 'ar' ? `${sign}${formatNumber(parseFloat(val))} ك` : `${sign}${val}K`;
    }
    return formatNumber(num);
  };
  
  /**
   * Format Unix timestamp to localized date string
   * English: November 18, 2025, 3:45 PM
   * Arabic:  ١٨ نوفمبر ٢٠٢٥، ٣:٤٥ م
   */
  const formatDate = (timestamp: number): string => {
    const date = new Date(timestamp * 1000);
    return date.toLocaleDateString(
      language === 'ar' ? 'ar-SY' : 'en-US',
      {
        year: 'numeric',
        month: 'long',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
      }
    );
  };
  
  /**
   * Format Unix timestamp to short date
   * English: Nov 18, 2025
   * Arabic:  ١٨ نوفمبر ٢٠٢٥
   */
  const formatDateShort = (timestamp: number): string => {
    const date = new Date(timestamp * 1000);
    return date.toLocaleDateString(
      language === 'ar' ? 'ar-SY' : 'en-US',
      {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
      }
    );
  };
  
  /**
   * Format relative time (e.g., "5 minutes ago")
   * English: 5 minutes ago, 3 hours ago, 2 days ago
   * Arabic:  منذ ٥ دقائق، منذ ٣ ساعات، منذ يومين
   */
  const formatRelativeTime = (timestamp: number): string => {
    const now = Date.now() / 1000;
    const diff = now - timestamp;
    
    const rtf = new Intl.RelativeTimeFormat(
      language === 'ar' ? 'ar' : 'en',
      { numeric: 'auto' }
    );
    
    if (diff < 60) return rtf.format(-Math.floor(diff), 'second');
    if (diff < 3600) return rtf.format(-Math.floor(diff / 60), 'minute');
    if (diff < 86400) return rtf.format(-Math.floor(diff / 3600), 'hour');
    return rtf.format(-Math.floor(diff / 86400), 'day');
  };
  
  /**
   * Format hash rate with proper unit
   * English: 123.4 MH/s
   * Arabic:  ١٢٣٫٤ م ت/ث
   */
  const formatHashRate = (hashesPerSecond: number): string => {
    const abs = Math.abs(hashesPerSecond);
    
    if (abs >= 1e15) {
      const val = (hashesPerSecond / 1e15).toFixed(2);
      return language === 'ar' ? `${formatNumber(parseFloat(val))} ب ت/ث` : `${val} PH/s`;
    }
    if (abs >= 1e12) {
      const val = (hashesPerSecond / 1e12).toFixed(2);
      return language === 'ar' ? `${formatNumber(parseFloat(val))} ت ت/ث` : `${val} TH/s`;
    }
    if (abs >= 1e9) {
      const val = (hashesPerSecond / 1e9).toFixed(2);
      return language === 'ar' ? `${formatNumber(parseFloat(val))} ج ت/ث` : `${val} GH/s`;
    }
    if (abs >= 1e6) {
      const val = (hashesPerSecond / 1e6).toFixed(2);
      return language === 'ar' ? `${formatNumber(parseFloat(val))} م ت/ث` : `${val} MH/s`;
    }
    if (abs >= 1e3) {
      const val = (hashesPerSecond / 1e3).toFixed(2);
      return language === 'ar' ? `${formatNumber(parseFloat(val))} ك ت/ث` : `${val} KH/s`;
    }
    return language === 'ar' ? `${formatNumber(hashesPerSecond)} ت/ث` : `${hashesPerSecond} H/s`;
  };
  
  /**
   * Format byte size with proper unit
   * English: 1.2 KB, 3.4 MB
   * Arabic:  ١٫٢ ك.ب، ٣٫٤ م.ب
   */
  const formatBytes = (bytes: number): string => {
    const abs = Math.abs(bytes);
    
    if (abs >= 1e9) {
      const val = (bytes / 1e9).toFixed(2);
      return language === 'ar' ? `${formatNumber(parseFloat(val))} ج.ب` : `${val} GB`;
    }
    if (abs >= 1e6) {
      const val = (bytes / 1e6).toFixed(2);
      return language === 'ar' ? `${formatNumber(parseFloat(val))} م.ب` : `${val} MB`;
    }
    if (abs >= 1e3) {
      const val = (bytes / 1e3).toFixed(2);
      return language === 'ar' ? `${formatNumber(parseFloat(val))} ك.ب` : `${val} KB`;
    }
    return language === 'ar' ? `${formatNumber(bytes)} ب` : `${bytes} B`;
  };
  
  /**
   * Format percentage with locale-specific decimal separator
   * English: 50.5%
   * Arabic:  ٥٠٫٥٪
   */
  const formatPercentage = (value: number, decimals: number = 1): string => {
    const formatted = value.toFixed(decimals);
    const localized = formatNumber(parseFloat(formatted));
    return language === 'ar' ? `٪${localized}` : `${localized}%`;
  };
  
  return {
    formatNumber,
    formatCurrency,
    formatCompactNumber,
    formatDate,
    formatDateShort,
    formatRelativeTime,
    formatHashRate,
    formatBytes,
    formatPercentage,
  };
}
