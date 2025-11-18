import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export type Language = 'en' | 'ar';
export type Direction = 'ltr' | 'rtl';

interface LanguageState {
  language: Language;
  direction: Direction;
  setLanguage: (lang: Language) => void;
  toggleLanguage: () => void;
}

export const useLanguageStore = create<LanguageState>()(
  persist(
    (set, get) => ({
      language: 'en',
      direction: 'ltr',
      
      setLanguage: (lang: Language) => {
        const dir: Direction = lang === 'ar' ? 'rtl' : 'ltr';
        set({ language: lang, direction: dir });
        
        // Update HTML attributes
        document.documentElement.setAttribute('dir', dir);
        document.documentElement.setAttribute('lang', lang);
      },
      
      toggleLanguage: () => {
        const currentLang = get().language;
        const newLang: Language = currentLang === 'en' ? 'ar' : 'en';
        get().setLanguage(newLang);
      },
    }),
    {
      name: 'opensyria-language',
    }
  )
);
